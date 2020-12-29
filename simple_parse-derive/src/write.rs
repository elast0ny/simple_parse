use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
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
                self.inner_to_writer(&mut ::simple_parse::SpCtx::default(), dst)
            }
            fn inner_to_writer<W: std::io::Write + ?Sized>(
                &self,
                ctx: &mut ::simple_parse::SpCtx,
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

    // Holds the index of a count field's contents
    let mut count_field_vals = Vec::with_capacity(fields.len());
    count_field_vals.resize(fields.len(), None);

    // Holds the index of a count field's index
    let mut count_field_idx = Vec::with_capacity(fields.len());
    count_field_idx.resize(fields.len(), None);

    let mut simple_field_names = Vec::with_capacity(fields.len());
    let fields: Vec<&syn::Field> = fields.iter().collect();

    // Iterate through fields to link count fields and populate validation functions
    for (idx, field) in fields.iter().enumerate() {
        let field_attrs: FieldAttributes = FromField::from_field(&field).unwrap();
        let field_ident = generate_field_name(field, idx, prefix, false);
        
        // save the simple field name for each field seen so far
        simple_field_names.push(generate_field_name(field, idx, None, false).to_string());
        
        // Get custom validator
        match field_attrs.validate {
            Some(ref s) => {
                let (fn_name, other_fields) = match split_custom_attr(s, &fields, idx, prefix, AllowFields::AfterCurrentAsSome) {
                    Ok(v) => v,
                    Err(e) => {
                        panic!("Invalid custom validator for field '{}', {}", &simple_field_names[idx], e);
                    }
                };
                write_code.extend(quote!{
                    ctx.is_reading = false;
                    #fn_name(&#field_ident, #other_fields ctx)?;
                })
            },
            None => {},
        };

        if let Some(count_field_name) = field_attrs.count.as_ref() {
            let mut field_idx = idx;
            for i in 0..idx {
                if count_field_name.as_str() == simple_field_names[i].as_str() {
                    field_idx = i;
                    break;
                }
            }
            // count field not found
            if field_idx == idx {
                panic!("#[sp(count)] annotation on field '{}' referers to an unknown field '{}'. Valid values are {:?}", &simple_field_names[idx], count_field_name, &simple_field_names[..idx]);
            }

            // Save link from count field to this field
            count_field_vals[field_idx] = Some(idx);
            // Save link from this field to its count field
            count_field_idx[idx] = Some(field_idx);
        }
    }
    
    let mut got_count = false;
    // Generate write call for every field
    for (idx, field) in fields.iter().enumerate() {
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

        let count_value;

        // If this field is a count field, write the len instead
        if let Some(field_idx) = count_field_vals[idx] {
            let content_field = fields[field_idx];
            let content_ty = strip_reference(&content_field.ty);
            let content_ident = generate_field_name(content_field, field_idx, prefix, false);
            
            let count_ident = generate_field_name(field, idx, None, false);
            let count_type = strip_reference(&field.ty);

            if !got_count {
                got_count = true;
                write_code.extend(quote!{
                    use std::convert::TryInto;
                });
            }

            let content_ty_str = quote!{#content_ty}.to_string();
            let count_ty_str = quote!{#count_type}.to_string();
            let count_decl = 
            if content_ty_str.starts_with("Option <") {
                if count_ty_str == "bool" {
                    quote!{
                        let #count_ident: #count_type = #content_ident.is_some();
                    }
                } else {
                    quote!{
                        let #count_ident: #count_type = if #content_ident.is_some() {
                            1
                        } else {
                            0
                        };
                    }
                }
            } else {
                quote!{
                    let #count_ident: #count_type = match #content_ident.len().try_into() {
                        Ok(v) => v,
                        Err(e) => return Err(::simple_parse::SpError::CountFieldOverflow),
                    };
                }
            };
            // Create temporary var to hold the real count value
            // then write this value
            write_code.extend(quote! {
                #count_decl
                ctx.is_little_endian = #is_output_le;
                written_len += #count_ident.inner_to_writer(ctx, dst)?;
            });
            continue;
        } else if let Some(field_idx) = count_field_idx[idx] {
            // The current field is annotated with count.
            // Pass the count field's value as Some to its `inner_to_writer()`
            let count_field = fields[field_idx];
            let count_ident = generate_field_name(count_field, field_idx, None, false);
            count_value = quote!{Some(#count_ident as usize)};
        } else {
            // current field is not annotated with count
            count_value = quote!{None};
        }

        // Pick between custom write or default
        let write_call = match field_attrs.writer {
            Some(ref s) => {
                let (fn_name, dependent_fields) = match split_custom_attr(s, &fields, idx, prefix, AllowFields::All) {
                    Ok(v) => v,
                    Err(e) => {
                        panic!("Invalid custom writer for field '{}', {}", &simple_field_names[idx], e);
                    }
                };
                quote!{
                    #fn_name(&#field_ident, #dependent_fields ctx, dst)
                }
            }
            None => {
                quote! {
                    #field_ident.inner_to_writer(ctx, dst)
                }
            }
        };

        // Add the generated code for this field
        write_code.extend(quote! {
            ctx.is_little_endian = #is_output_le;
            ctx.count = #count_value;
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
                ctx.is_little_endian = #default_is_le;
                written_len += var_id.inner_to_writer(ctx, dst)?;
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
