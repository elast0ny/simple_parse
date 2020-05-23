use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DataEnum, DataStruct, DeriveInput, Fields};

use crate::*;

pub fn generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Generate the code that implements the read()
    let generated_read = match input.data {
        // Parse as a struct
        Data::Struct(ref contents) => {
            let attrs = FromDeriveInput::from_derive_input(&input).unwrap();
            generate_struct_read(contents, attrs)
        }
        // Parse as enum
        Data::Enum(ref contents) => {
            let attrs = FromDeriveInput::from_derive_input(&input).unwrap();
            generate_enum_read(contents, attrs)
        }
        // Unhandled derive usage
        _ => unimplemented!("Cannot derive on this type"),
    };

    let name = input.ident;
    let generics = add_trait_bounds(input.generics, parse_quote! {simple_parse::SpRead});
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics simple_parse::SpRead for #name #ty_generics #where_clause {
            fn from_bytes(input: &[u8]) -> Result<(&[u8], Self), simple_parse::SpError> {
                Self::inner_from_bytes(input, true, None)
            }
            fn inner_from_bytes(
                input: &[u8],
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<(&[u8], Self), simple_parse::SpError>
            where
                Self: Sized
            {
                #generated_read
            }
        }
    };

    //println!("{}", expanded.to_string());
    proc_macro::TokenStream::from(expanded)
}

/// Generates code that parses bytes into a struct
fn generate_struct_read(data: &DataStruct, attrs: StructAttributes) -> TokenStream {
    let default_is_le: bool = match attrs.endian {
        None => {
            if cfg!(target_endian = "little") {
                true
            } else {
                false
            }
        }
        Some(ref e) => is_lower_endian(e),
    };

    let (field_idents, read_code) = generate_field_read(&data.fields, default_is_le);
    let field_list = generate_field_list(&data.fields, Some(&field_idents));

    quote! {
        let mut rest = input.as_ref();
        #read_code
        Ok((input, Self#field_list))
    }
}

/// Generates the code that parse bytes into an enum variant
fn generate_enum_read(data: &DataEnum, attrs: EnumAttributes) -> TokenStream {
    let id_type = syn::Ident::new(
        match attrs.id_type {
            Some(ref s) => s.as_str(),
            None => "u8",
        },
        proc_macro2::Span::call_site(),
    );

    let default_is_le: bool = match attrs.endian {
        None => {
            if cfg!(target_endian = "little") {
                true
            } else {
                false
            }
        }
        Some(ref e) => is_lower_endian(e),
    };

    let mut variant_code_gen = TokenStream::new();

    // Create a match case for every variant
    for variant in data.variants.iter() {
        let var_attrs: VariantAttributes = FromVariant::from_variant(&variant).unwrap();
        let variant_name = &variant.ident;
        let variant_id =
            syn::LitInt::new(&var_attrs.id.to_string(), proc_macro2::Span::call_site());
        let (field_idents, read_code) = generate_field_read(&variant.fields, default_is_le);
        let field_list = generate_field_list(&variant.fields, Some(&field_idents));
        variant_code_gen.extend(quote! {
            #variant_id => {
                #read_code
                Ok((rest, Self::#variant_name#field_list))
            },
        });
    }

    // Add match case to handle unknown IDs
    variant_code_gen.extend(quote! {
        unknown_id => {
            Err(simple_parse::SpError::UnknownEnumVariant(unknown_id as _))
        }
    });

    quote! {
        let mut rest = input.as_ref();
        let r =  #id_type::inner_from_bytes(rest, #default_is_le, None)?;
        rest = r.0;
        match r.1 {
            #variant_code_gen
        }
    }
}

/// Generates the code that calls `from_bytes` for the specific field
/// e.g :
///     let (rest , field_0) = u8::inner_from_bytes(...)?;
fn generate_field_read(fields: &Fields, default_is_le: bool) -> (Vec<TokenStream>, TokenStream) {
    let mut idents = Vec::with_capacity(fields.len());
    let mut generated_code = TokenStream::new();

    for (idx, field) in fields.iter().enumerate() {
        let field_attrs: FieldAttributes = FromField::from_field(&field).unwrap();
        let field_ident = generate_field_name(field, idx, None);
        idents.push(field_ident.clone());
        let field_type = &field.ty;
        let count_field = match generate_count_field_name(field_attrs.count, fields, None) {
            None => quote! {None},
            Some(field) => quote! {Some(#field as _)},
        };
        let is_input_le = match field_attrs.endian {
            None => default_is_le,
            Some(ref e) => is_lower_endian(e),
        };

        let read_call = match field_attrs.reader {
            Some(s) => {
                quote! {
                    {
                        let input = rest;
                        let is_input_le = #is_input_le;
                        let count = #count_field;
                        #s
                    }
                }
            }
            None => {
                quote! {
                    <#field_type>::inner_from_bytes(rest, #is_input_le, #count_field)
                }
            }
        };

        generated_code.extend(quote! {
            let read_func_result = #read_call?;
            rest = read_func_result.0;
            let #field_ident: #field_type = read_func_result.1;
        })
    }
    (idents, generated_code)
}
