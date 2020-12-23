use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{parse_quote, Data, DataEnum, DeriveInput, Fields};

use crate::*;

pub fn generate(input: &mut DeriveInput) -> TokenStream {

    let generated_code = match input.data {
        Data::Struct(ref contents) => {
            let attrs: StructAttributes = FromDeriveInput::from_derive_input(&input).unwrap();
            generate_fields_write(&contents.fields, Some("self"), attrs.endian.as_deref()).0
        }
        Data::Enum(ref contents) => {
            let attrs = FromDeriveInput::from_derive_input(&input).unwrap();
            generate_enum_write(&input, contents, attrs)
        }
        // Unhandled derive usage
        _ => unimplemented!("Cannot derive on this type"),
    };

    add_trait_bounds(&mut input.generics, parse_quote! {simple_parse::SpRead});
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let res = quote! {
        impl #impl_generics ::simple_parse::SpWrite for #name #ty_generics #where_clause {
            fn to_writer<W: std::io::Write + ?Sized>(&self, dst: &mut W) -> std::result::Result<usize, ::simple_parse::SpError> {
                self.inner_to_writer(true, true, dst)
            }
            fn inner_to_writer<W: std::io::Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> std::result::Result<usize, ::simple_parse::SpError>
            {
                let mut written_len: usize = 0;
                #generated_code
                Ok(written_len)
            }
        }
    };

    #[cfg(feature = "print-generated")]
    println!("{}", res.to_string());

    res
}

/// Generates the code that dumps each field of the struct into the Vec<u8>
fn generate_fields_write(
    fields: &Fields,
    prefix: Option<&str>,
    endian: Option<&str>,
) -> (TokenStream, TokenStream) {

    let mut write_code = TokenStream::new();
    let mut field_list = TokenStream::new();

    let default_is_le: bool = match endian {
        None => cfg!(target_endian = "little"),
        Some(ref e) => is_lower_endian(e),
    };

    let mut count_fields = HashMap::new();

    // Generate a map of fields that are counts to other fields
    for (idx, field) in fields.iter().enumerate() {
        let field_attrs: FieldAttributes = FromField::from_field(&field).unwrap();
        if field_attrs.count.is_none() {
            continue;
        }

        if let Some(field_name) = generate_count_field_name(&field_attrs.count, fields, prefix, false) {
            if count_fields.is_empty() {
                write_code.extend(quote! {use std::convert::TryInto;});
            }
            count_fields.insert(
                field_name.to_string(), // Name of the count=X field
                generate_field_name(field, idx, prefix, false), // Current field that needs count
            );
        } else {
            panic!("count annotation for {} does not point to a valid field...", generate_field_name(field, idx, None, false).to_string());
        }
    }
    
    // Generate write call for every field
    for (idx, field) in fields.iter().enumerate() {
        let field_name = generate_field_name(field, idx, None, false);
        let field_attrs: FieldAttributes = FromField::from_field(&field).unwrap();
        let field_ident = generate_field_name(field, idx, prefix, false);

        field_list.extend(
            quote!{
                #field_ident,
            }
        );

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
            
            write_code.extend(quote! {
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
            Some(ref s) => {
                let (fn_name, dependent_fields) = match split_custom_attr(s, &fields, idx, None) {
                    Ok(v) => v,
                    Err(e) => {
                        panic!("Invalid custom writer for field '{}', {}",field_name.to_string(), e);
                    }
                };
                quote!{
                    #fn_name(&#field_ident, #dependent_fields #is_output_le, #prepend_count, dst)
                }
            }
            None => {
                quote! {
                    #field_ident.inner_to_writer(#is_output_le, #prepend_count, dst)
                }
            }
        };

        // Add the generated code for this field
        write_code.extend(quote! {
            written_len += #write_call?;
        })
    }

    (write_code, field_list)
}

/// Generates the code that matches the current enum variant and dumps bytes
/// for each of its fields
fn generate_enum_write(input: &DeriveInput, data: &DataEnum, attrs: EnumAttributes) -> TokenStream {
    let name = &input.ident;

    let id_type = get_enum_id_type(data, &attrs);

    let default_is_le: bool = match attrs.endian {
        None => cfg!(target_endian = "little"),
        Some(ref e) => is_lower_endian(e),
    };

    let mut variant_code_gen = TokenStream::new();
    let mut auto_variant_id:usize = 0;
    // Create a match case for every variant
    for variant in data.variants.iter() {
        let var_attrs: VariantAttributes = FromVariant::from_variant(&variant).unwrap();
        let variant_name = &variant.ident;
        let variant_id = match var_attrs.id {
            Some(id) => {
                auto_variant_id = id + 1;
                id
            },
            _ => {
                let cur = auto_variant_id;
                auto_variant_id += 1;
                cur
            },
        };
        let variant_id = syn::LitInt::new(&variant_id.to_string(), proc_macro2::Span::call_site());

        let (write_code, field_list) = if !variant.fields.is_empty() {
            let (write, list) = generate_fields_write(&variant.fields, None, var_attrs.endian.as_deref());
            (write, 
                if let syn::Fields::Unnamed(_r) = &variant.fields {
                    quote!{
                        (#list)
                    }
                } else {
                    quote!{
                        {#list}
                    }
                }
            )
        } else {
            (TokenStream::new(), TokenStream::new())
        };

        variant_code_gen.extend(quote! {
            #name::#variant_name#field_list => {
                let mut var_id: #id_type = #variant_id;
                written_len += var_id.inner_to_writer(#default_is_le, true, dst)?;
                #write_code
            },
        });
    }

    quote! {
        match self {
            #variant_code_gen
        };
    }
}
