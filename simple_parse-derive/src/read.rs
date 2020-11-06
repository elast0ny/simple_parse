use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DataEnum, DataStruct, DeriveInput, Fields};

use crate::*;

pub(crate) fn generate(
    input: proc_macro::TokenStream,
    reader_type: ReaderType,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let generated_read;

    // Generate the code that implements SpRead
    match input.data {
        // Parse as a struct
        Data::Struct(ref contents) => {
            let attrs = FromDeriveInput::from_derive_input(&input).unwrap();
            generated_read = generate_struct_read(&reader_type, &input, contents, &attrs);
        }
        // Parse as enum
        Data::Enum(ref contents) => {
            let attrs = FromDeriveInput::from_derive_input(&input).unwrap();
            generated_read = generate_enum_read(&reader_type, &input, contents, &attrs);
        }
        // Unhandled derive usage
        _ => unimplemented!("Cannot derive on this type"),
    };

    let name = input.ident;
    let generics = add_trait_bounds(input.generics, parse_quote! {simple_parse::SpRead});
    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate impl block for TYPE and &TYPE
    let expanded = match reader_type {
        ReaderType::Reader => {
            quote! {
            impl ::simple_parse::SpRead for #name #ty_generics #where_clause {
                fn from_reader<R: std::io::Read + ?Sized>(src: &mut R) -> std::result::Result<Self, ::simple_parse::SpError>
                where
                    Self: Sized
                {
                    <#name>::inner_from_reader(src, true, None)
                }
                fn inner_from_reader<R: std::io::Read + ?Sized>(
                    src: &mut R,
                    is_input_le: bool,
                    count: Option<usize>,
                ) -> std::result::Result<Self, ::simple_parse::SpError>
                where
                    Self: Sized
                {
                    #generated_read
                }
            }}
        }
        ReaderType::Raw => {
            // Use the first lifetime parameter as the one bound to the input bytes
            let lifetime = if let Some(lt) = generics.lifetimes().next() {
                quote! {#lt}
            } else {
                quote! {'_}
            };

            quote! {
                impl #ty_generics ::simple_parse::SpReadRaw<#lifetime> for #name #ty_generics #where_clause {
                    fn from_slice(src: &mut std::io::Cursor<&#lifetime [u8]>) -> std::result::Result<Self, ::simple_parse::SpError>
                    where
                        Self: Sized,
                    {
                        <Self>::inner_from_slice(src, true, None)
                    }
                    fn inner_from_slice(
                        src: &mut std::io::Cursor<&#lifetime [u8]>,
                        is_input_le: bool,
                        count: Option<usize>,
                    ) -> std::result::Result<Self, ::simple_parse::SpError>
                    where
                        Self: Sized,
                    {
                        #generated_read
                    }
                }
            }
        }
        ReaderType::RawMut => {
            // Use the first lifetime parameter as the one bound to the input bytes
            let lifetime = if let Some(lt) = generics.lifetimes().next() {
                quote! {#lt}
            } else {
                quote! {'_}
            };

            quote! {
                impl #ty_generics ::simple_parse::SpReadRawMut<#lifetime> for #name #ty_generics #where_clause {
                    fn from_mut_slice(src: &mut std::io::Cursor<&#lifetime mut [u8]>) -> std::result::Result<Self, ::simple_parse::SpError>
                    where
                        Self: Sized,
                    {
                        <Self>::inner_from_mut_slice(src, true, None)
                    }
                    fn inner_from_mut_slice(
                        src: &mut std::io::Cursor<&#lifetime mut [u8]>,
                        is_input_le: bool,
                        count: Option<usize>,
                    ) -> std::result::Result<Self, ::simple_parse::SpError>
                    where
                        Self: Sized,
                    {
                        #generated_read
                    }
                }
            }
        }
    };

    //println!("{}", expanded.to_string());
    proc_macro::TokenStream::from(expanded)
}

/// Generates code that parses bytes into a struct
fn generate_struct_read(
    reader_type: &ReaderType,
    input: &DeriveInput,
    data: &DataStruct,
    attrs: &StructAttributes,
) -> TokenStream {
    let name = &input.ident;
    let default_is_le: bool = match attrs.endian {
        None => cfg!(target_endian = "little"),
        Some(ref e) => is_lower_endian(e),
    };

    let (field_idents, read_code) = generate_field_read(reader_type, &data.fields, default_is_le);
    let field_list = generate_field_list(&data.fields, Some(&field_idents), None);

    quote! {
        #read_code
        Ok(#name#field_list)
    }
}

/// Generates the code that parse bytes into an enum variant
fn generate_enum_read(
    reader_type: &ReaderType,
    input: &DeriveInput,
    data: &DataEnum,
    attrs: &EnumAttributes,
) -> TokenStream {
    let name = &input.ident;
    
    let id_type: syn::Type = syn::parse_str(match attrs.id_type {
            Some(ref s) => s.as_str(),
            None => "u8",
        }).unwrap();

    let default_is_le: bool = match attrs.endian {
        None => cfg!(target_endian = "little"),
        Some(ref e) => is_lower_endian(e),
    };

    let mut variant_code_gen = TokenStream::new();

    // Create a match case for every variant
    for variant in data.variants.iter() {
        let var_attrs: VariantAttributes = FromVariant::from_variant(&variant).unwrap();
        let variant_name = &variant.ident;
        let variant_id =
            syn::LitInt::new(&var_attrs.id.to_string(), proc_macro2::Span::call_site());
        let (field_idents, read_code) =
            generate_field_read(reader_type, &variant.fields, default_is_le);
        let field_list = generate_field_list(&variant.fields, Some(&field_idents), None);
        variant_code_gen.extend(quote! {
            #variant_id => {
                #read_code
                Ok(#name::#variant_name#field_list)
            },
        });
    }

    // Add match case to handle unknown IDs
    variant_code_gen.extend(quote! {
        _ => {
            use std::convert::TryInto;
            Err(::simple_parse::SpError::UnknownEnumVariant)
        }
    });

    match reader_type {
        ReaderType::Reader => {
            quote! {
                match <#id_type>::inner_from_reader(src, #default_is_le, None)? {
                    #variant_code_gen
                }
            }
        }
        ReaderType::Raw => {
            quote! {
                match <#id_type>::inner_from_slice(src, #default_is_le, None)? {
                    #variant_code_gen
                }
            }
        }
        ReaderType::RawMut => {
            quote! {
                match <#id_type>::inner_from_mut_slice(src, #default_is_le, None)? {
                    #variant_code_gen
                }
            }
        }
    }
}

/// Generates the code that calls `from_reader` for the specific field
/// e.g :
///     let (rest , field_0) = u8::inner_from_reader(...)?;
fn generate_field_read(
    reader_type: &ReaderType,
    fields: &Fields,
    default_is_le: bool,
) -> (Vec<TokenStream>, TokenStream) {
    let mut idents = Vec::with_capacity(fields.len());
    let mut generated_code = TokenStream::new();

    for (idx, field) in fields.iter().enumerate() {
        let field_attrs: FieldAttributes = FromField::from_field(&field).unwrap();
        let field_ident = generate_field_name(field, idx, None, false);
        idents.push(field_ident.clone());
        let field_type = &field.ty;

        let count_field = match generate_count_field_name(field_attrs.count, fields, None, true) {
            None => {
                quote! {
                    None
                }
            }
            Some(c) => {
                quote! {Some(#c as _)}
            }
        };

        let is_input_le = match field_attrs.endian {
            None => default_is_le,
            Some(ref e) => is_lower_endian(e),
        };

        let read_call = match field_attrs.reader {
            Some(s) => {
                let s: TokenStream = s.parse().unwrap();
                quote! {
                    {
                        let is_input_le = #is_input_le;
                        let count: Option<usize> = #count_field;
                        #s
                    }
                }
            }
            None => match reader_type {
                ReaderType::Reader => {
                    quote! {
                        <#field_type>::inner_from_reader(src, #is_input_le, #count_field)
                    }
                }
                ReaderType::Raw => {
                    quote! {
                        <#field_type>::inner_from_slice(src, #is_input_le, #count_field)
                    }
                }
                ReaderType::RawMut => {
                    quote! {
                        <#field_type>::inner_from_mut_slice(src, #is_input_le, #count_field)
                    }
                }
            },
        };

        generated_code.extend(quote! {
            let #field_ident: #field_type = #read_call?;
        })
    }
    (idents, generated_code)
}
