use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{parse_macro_input, parse_quote, Data, DataEnum, DataStruct, DeriveInput, Fields};

use crate::*;

pub fn generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Generate the code that implements the read()
    let generated_code = match input.data {
        // Sruct
        Data::Struct(ref contents) => {
            let attrs = FromDeriveInput::from_derive_input(&input).unwrap();
            generate_struct_write(&input, contents, attrs)
        }
        // Enum
        Data::Enum(ref contents) => {
            let attrs = FromDeriveInput::from_derive_input(&input).unwrap();
            generate_enum_write(&input, contents, attrs)
        }
        // Unhandled derive usage
        _ => unimplemented!("Cannot derive on this type"),
    };

    let name = input.ident;
    let generics = add_trait_bounds(input.generics, parse_quote! {simple_parse::SpRead});
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::simple_parse::SpWrite for #name #ty_generics #where_clause {
            fn to_writer<W: std::io::Write + ?Sized>(&self, dst: &mut W) -> Result<usize, ::simple_parse::SpError> {
                self.inner_to_writer(true, true, dst)
            }
            fn inner_to_writer<W: std::io::Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, ::simple_parse::SpError>
            {
                let mut written_len: usize = 0;
                #generated_code
                Ok(written_len)
            }
        }

        impl #impl_generics simple_parse::SpWrite for &#name #ty_generics #where_clause {
            fn to_writer<W: std::io::Write + ?Sized>(&self, dst: &mut W) -> Result<usize, ::simple_parse::SpError> {
                self.inner_to_writer(true, true, dst)
            }
            fn inner_to_writer<W: std::io::Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, ::simple_parse::SpError>
            {
                (**self).inner_to_writer(is_output_le, prepend_count, dst)
            }
        }

        impl #impl_generics simple_parse::SpWrite for &mut #name #ty_generics #where_clause {
            fn to_writer<W: std::io::Write + ?Sized>(&self, dst: &mut W) -> Result<usize, ::simple_parse::SpError> {
                self.inner_to_writer(true, true, dst)
            }
            fn inner_to_writer<W: std::io::Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, ::simple_parse::SpError>
            {
                (**self).inner_to_writer(is_output_le, prepend_count, dst)
            }
        }
    };

    //println!("{}", expanded.to_string());
    proc_macro::TokenStream::from(expanded)
}

/// Generates the code that dumps each field of the struct into the Vec<u8>
fn generate_struct_write(
    _input: &DeriveInput,
    data: &DataStruct,
    attrs: StructAttributes,
) -> TokenStream {
    let default_is_le: bool = match attrs.endian {
        None => cfg!(target_endian = "little"),
        Some(ref e) => is_lower_endian(e),
    };

    generate_field_write(&data.fields, Some("self"), default_is_le)
}

/// Generates the code that matches the current enum variant and dumps bytes
/// for each of its fields
fn generate_enum_write(input: &DeriveInput, data: &DataEnum, attrs: EnumAttributes) -> TokenStream {
    let name = &input.ident;
    let var_id_type = match attrs.id_type {
        None => quote! {u8},
        Some(t) => t.parse().unwrap(),
    };

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

        let field_list = generate_field_list(&variant.fields, None, None);
        let field_write_code = generate_field_write(&variant.fields, None, default_is_le);

        variant_code_gen.extend(quote! {
            #name::#variant_name#field_list => {
                let mut var_id: #var_id_type = #variant_id;
                written_len += var_id.inner_to_writer(#default_is_le, true, dst)?;
                #field_write_code
            },
        });
    }

    quote! {
        match self {
            #variant_code_gen
        };
    }
}

/// Generates code that calls inner_to_writer for every field and appends the resulting bytes to a vec
/// called `res`.
/// This function also modifies any `count` field to match their vec's len()
fn generate_field_write(
    fields: &Fields,
    obj_name: Option<&str>,
    default_is_le: bool,
) -> TokenStream {
    let mut dump_fields_code = TokenStream::new();
    let mut count_fields = HashMap::new();

    // Generate a map of fields that are counts to other fields
    for (idx, field) in fields.iter().enumerate() {
        let field_attrs: FieldAttributes = FromField::from_field(&field).unwrap();
        if field_attrs.count.is_none() {
            continue;
        }

        if let Some(field_name) = generate_count_field_name(field_attrs.count, fields, obj_name, false) {
            if count_fields.is_empty() {
                dump_fields_code.extend(quote! {use std::convert::TryInto;});
            }
            count_fields.insert(
                field_name.to_string(), // Name of the count=X field
                generate_field_name(field, idx, obj_name, false), // Current field that needs count
            );
        } else {
            panic!("count annotation for {} does not point to a valid field...", generate_field_name(field, idx, None, false).to_string());
        }
    }
    
    // Generate write call for every field
    for (idx, field) in fields.iter().enumerate() {
        let field_attrs: FieldAttributes = FromField::from_field(&field).unwrap();
        let field_ident = generate_field_name(field, idx, obj_name, false);

        let is_output_le = match field_attrs.endian {
            None => default_is_le,
            Some(ref e) => is_lower_endian(e),
        };

        let prepend_count = field_attrs.count.is_none();

        // If this field is a count field, write the len instead
        if let Some(ref ident) = count_fields.get(&field_ident.to_string()) {
            let ftype = if let ::syn::Type::Reference(ty) = &field.ty {
                let t = &ty.elem;
                quote!{#t}
            } else {
                let t = &field.ty;
                quote!{#t}
            };
            
            dump_fields_code.extend(quote! {
                let _f: #ftype = match #ident.len().try_into() {
                    Ok(v) => v,
                    Err(e) => return Err(::simple_parse::SpError::CountFieldOverflow),
                };
                written_len += _f.inner_to_writer(#is_output_le, true, dst)?;
            });
            continue;
        }

        // Pick between custom write or default
        let write_call = match field_attrs.writer {
            Some(s) => {
                let s: TokenStream = s.parse().unwrap();
                let ref_mut = if obj_name.is_some() {
                    quote! {
                        &mut
                    }
                } else {
                    quote! {}
                };
                quote! {
                    {
                        let input = #ref_mut #field_ident;
                        let is_output_le = #is_output_le;
                        #s
                    }
                }
            }
            None => {
                quote! {
                    #field_ident.inner_to_writer(#is_output_le, #prepend_count, dst)
                }
            }
        };

        // Add the generated code for this field
        dump_fields_code.extend(quote! {
            written_len += #write_call?;
        })
    }

    quote! {
        #dump_fields_code
    }
}
