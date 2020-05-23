use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DataEnum, DataStruct, DeriveInput, Fields};

use crate::*;

pub fn generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Generate the code that implements the read()
    let generated_code = match input.data {
        // Sruct
        Data::Struct(ref contents) => {
            let attrs = FromDeriveInput::from_derive_input(&input).unwrap();
            generate_struct_write(contents, attrs)
        }
        // Enum
        Data::Enum(ref contents) => {
            let attrs = FromDeriveInput::from_derive_input(&input).unwrap();
            generate_enum_write(contents, attrs)
        }
        // Unhandled derive usage
        _ => unimplemented!("Cannot derive on this type"),
    };

    let name = input.ident;
    let generics = add_trait_bounds(input.generics, parse_quote! {simple_parse::SpRead});
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics simple_parse::SpWrite for #name #ty_generics #where_clause {
            fn to_bytes(&mut self) -> Result<Vec<u8>, simple_parse::SpError> {
                self.inner_to_bytes(true)
            }
            fn inner_to_bytes(
                &mut self,
                is_output_le: bool,
            ) -> Result<Vec<u8>, simple_parse::SpError>
            {
                let mut res = Vec::new();
                #generated_code
                Ok(res)
            }
        }
    };

    //println!("{}", expanded.to_string());
    proc_macro::TokenStream::from(expanded)
}

/// Generates the code that dumps each field of the struct into the Vec<u8>
fn generate_struct_write(data: &DataStruct, attrs: StructAttributes) -> TokenStream {
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

    generate_field_write(&data.fields, Some("self"), default_is_le)
}

/// Generates the code that matches the current enum variant and dumps bytes
/// for each of its fields
fn generate_enum_write(data: &DataEnum, attrs: EnumAttributes) -> TokenStream {
    
    let var_id_type = match attrs.id_type {
        None => quote!{u8},
        Some(t) => t.parse().unwrap(),
    };

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
        
        let field_list = generate_field_list(&variant.fields, None);
        let field_write_code = generate_field_write(&variant.fields, None, default_is_le);
        
        variant_code_gen.extend(quote! {
            Self::#variant_name#field_list => {
                let mut var_id: #var_id_type = #variant_id;
                res.append(&mut var_id.inner_to_bytes(#default_is_le)?);
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

/// Generates code that calls inner_to_bytes for every field and appends the resulting bytes to a vec
/// called `res`.
/// This function also modifies any `count` field to match their vec's len()
fn generate_field_write(fields: &Fields, obj_name: Option<&str>, default_is_le: bool) -> TokenStream {
    let mut overwrite_count_code = quote! {use std::convert::TryInto;};
    let mut dump_fields_code = TokenStream::new();

    for (idx, field) in fields.iter().enumerate() {
        let field_attrs: FieldAttributes = FromField::from_field(&field).unwrap();
        let field_ident = generate_field_name(field, idx, obj_name);

        // Generate code that overwrites any `count` field with the vec's len
        if let Some(field_name) = generate_count_field_name(field_attrs.count, fields, obj_name)
        {
            if obj_name.is_none() {
                overwrite_count_code.extend(quote!{*});
            }
            overwrite_count_code.extend(quote! {
                #field_name = match #field_ident.len().try_into() {
                    Ok(v) => v,
                    Err(e) => return Err(simple_parse::SpError::CountFieldOverflow),
                };
            })
        };

        let is_input_le = match field_attrs.endian {
            None => default_is_le,
            Some(ref e) => is_lower_endian(e),
        };

        // Pick between custom write or default
        let write_call = match field_attrs.writer {
            Some(s) => {
                quote! {
                    {
                        let input = #field_ident;
                        let is_input_le = #is_input_le;
                        #s
                    }
                }
            }
            None => {
                quote! {
                    #field_ident.inner_to_bytes(#is_input_le)
                }
            }
        };

        // Add the generated code for this field
        dump_fields_code.extend(quote! {
            res.append(&mut #write_call?);
        })
    }

    quote! {
        #overwrite_count_code
        #dump_fields_code
    }
}
