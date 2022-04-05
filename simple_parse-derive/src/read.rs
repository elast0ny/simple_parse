use std::fmt::Write;

use crate::*;
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DataEnum, DeriveInput, Fields};

pub(crate) fn generate(input: &mut DeriveInput) -> proc_macro2::TokenStream {
    let mut log_call: TokenStream = TokenStream::new();
    let mut init_code: TokenStream;

    // Generate the code that implements SpRead
    match input.data {
        // Parse as a struct
        Data::Struct(ref contents) => {
            let attrs: StructAttributes = FromDeriveInput::from_derive_input(&input).unwrap();
            let (field_init, mut field_names) = generate_fields_read(&contents.fields, attrs.endian.as_deref());
            
            // Cast every field of Self to a &mut MaybeUninit<_>
            init_code = quote!{
                let p = dst.as_mut_ptr() as *mut Self;
            };
            for (name, typ) in field_names.drain(..) {
                init_code.extend(quote!{
                    let #name: &mut MaybeUninit::<#typ> = unsafe {&mut *(addr_of_mut!((*p).#name) as *mut MaybeUninit::<#typ>)};
                })
            }

            // Code that initializes each field
            init_code.extend(field_init);
    
            if cfg!(feature = "verbose") {
                let name = &input.ident;
                log_call = quote! {
                    ::simple_parse::debug!("Read struct {}", stringify!(#name));
                };
            }
        }
        // Parse as enum
        Data::Enum(ref contents) => {
            let attrs = FromDeriveInput::from_derive_input(&input).unwrap();
            let parsed_enum = generate_enum_read(contents, &attrs);
            
            init_code = quote!{
                dst.write({#parsed_enum});
            };

            if cfg!(feature = "verbose") {
                let name = &input.ident;
                log_call = quote! {
                    ::simple_parse::debug!("Read enum {}", stringify!(#name));
                };
            }
        }
        // Unhandled derive usage
        _ => unimplemented!("Cannot derive SpRead on this type"),
    };

    add_trait_bounds(&mut input.generics, parse_quote! {::simple_parse::SpRead});
    let name = &input.ident;
    let (_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Generate impl block
    let res = quote! {
        impl ::simple_parse::SpRead for #name #ty_generics #where_clause {
            fn inner_from_reader<'a, R: ::std::io::Read + ?Sized>(
                src: &mut R,
                ctx: &mut ::simple_parse::SpCtx,
                dst: &'a mut ::core::mem::MaybeUninit<Self>,
            ) -> ::std::result::Result<&'a mut Self, ::simple_parse::SpError>
            {
                use ::core::{ptr::addr_of_mut, mem::MaybeUninit};

                #log_call

                #init_code
                
                unsafe {
                    Ok(dst.assume_init_mut())
                }
            }
        }
    };

    #[cfg(feature = "print-generated")]
    println!("{}", res.to_string());

    res
}

/// Generates code that parses bytes into a struct
fn generate_fields_read(fields: &Fields, endian: Option<&str>) -> (TokenStream, Vec<(TokenStream, syn::Type)>) {
    
    let num_fields = fields.len();
    let mut init_code = TokenStream::new();
    let mut field_names = Vec::with_capacity(num_fields);

    // No fields
    if num_fields == 0 {
        return (init_code, field_names);
    }

    let default_is_le: bool = match endian {
        None => true,
        Some(s) => is_lower_endian(s),
    };
    
    let fields: Vec<&syn::Field> = fields.iter().collect();

    // holds the index of a field's `len` field
    let mut len_field_idx = Vec::with_capacity(fields.len());
    len_field_idx.resize(fields.len(), None);

    let mut string_field_names = Vec::with_capacity(fields.len());

    // Iterate through all fields and save #[sp(len)] references
    // This ensures that any reference is to a field that appears before the current field
    for (idx, field) in fields.iter().enumerate() {
        let field_attrs: FieldAttributes = FromField::from_field(field).unwrap();
        // save the simple field name for each field seen so far
        string_field_names.push(generate_field_name(field, idx, None, false).to_string());

        if let Some(count_field_name) = field_attrs.len.as_ref() {
            let mut found_idx = idx;
            for i in 0..idx {
                if count_field_name.as_str() == string_field_names[i].as_str() {
                    found_idx = i;
                    break;
                }
            }
            // `len` field not found
            if found_idx == idx {
                panic!("#[sp(len)] annotation on field '{}' refers to an unknown field '{}'. Valid values are {:?}", &string_field_names[idx], count_field_name, &string_field_names[..idx]);
            }

            // Save link from current field to `len` field
            len_field_idx[idx] = Some(found_idx);
        }
    }

    let mut prev_endian = false;
    for (idx, field) in fields.iter().enumerate() {
        let field_name = generate_field_name(field, idx, None, false);
        let field_type = strip_lifetimes(&field.ty);
        let field_attrs: FieldAttributes = FromField::from_field(field).unwrap();

        // Get this field's endianness
        let is_field_le = match field_attrs.endian {
            None => default_is_le,
            Some(ref e) => is_lower_endian(e),
        };

        // Get this field's sp(len) 
        if let Some(field_idx) = len_field_idx[idx] {
            let count_field_name = &field_names[field_idx].0;
            init_code.extend(quote! {ctx.len = Some(unsafe{*(#count_field_name.assume_init_mut())} as _);});
        }

        // Fixup endianness if it changed 
        if idx == 0 || prev_endian != is_field_le {
            prev_endian = is_field_le;
            init_code.extend(quote! {
                ctx.is_little_endian = #is_field_le;
            });
        }

        // Initialize the current field
        init_code.extend(quote! {
            <#field_type>::inner_from_reader(src, ctx, #field_name)?;
        });
        
        // Save the field name and field type for the caller
        field_names.push((field_name, field_type));
    }

    (init_code, field_names)
}

/// Generates the code that parse bytes into an enum variant
fn generate_enum_read(data: &DataEnum, attrs: &EnumAttributes) -> TokenStream {
    if data.variants.is_empty() {
        panic!("Unable to derive SpRead on empty enum");
    }

    // Pick the best size to use for the variant IDs
    let id_type = get_enum_id_type(data, attrs);

    // Read the id
    let mut init_code = TokenStream::new();
    let mut next_variant_id: usize = 0;
    for variant in data.variants.iter() {
        let var_attrs: VariantAttributes = FromVariant::from_variant(&variant).unwrap();
        let variant_name = &variant.ident;
        let variant_id = match var_attrs.id {
            Some(id) => {
                next_variant_id = id + 1;
                id
            }
            _ => {
                let cur = next_variant_id;
                next_variant_id += 1;
                cur
            }
        };
        let variant_id = syn::LitInt::new(&variant_id.to_string(), proc_macro2::Span::call_site());

        let mut read_code = TokenStream::new();
        let mut variant_content = TokenStream::new();

        if !variant.fields.is_empty() {

            let is_tuple = matches!(&variant.fields, syn::Fields::Unnamed(_));
            
            let variant_endianness = match var_attrs.endian.as_deref() {
                Some(v) => Some(v),
                None => attrs.endian.as_deref(),
            };

            let (field_init, field_list) = generate_fields_read(&variant.fields, variant_endianness);

            let mut stack_name = String::new();
            for (idx, (name, typ)) in field_list.iter().enumerate() {
                // We cant write directly into the enum variant as theres no way
                // to get a pointer into the "fields"
                // Therefore, we must create stack variables for each field and then
                // instantiate the enum with the stack vars
                stack_name.clear();
                write!(&mut stack_name, "s{idx}").unwrap();
                let s_name: TokenStream  = stack_name.parse().unwrap();

                read_code.extend(quote!{
                    let mut #s_name = MaybeUninit::<#typ>::uninit();
                    let #name = &mut #s_name;
                });

                if is_tuple {
                    variant_content.extend(quote!{
                        #s_name.assume_init(),
                    });
                } else {
                    variant_content.extend(quote!{
                        #name: #s_name.assume_init(),
                    });
                }
            }

            if is_tuple {
                variant_content = quote!{
                    (#variant_content)
                };
            } else {
                variant_content = quote!{
                    {#variant_content}
                };
            }

            read_code.extend(field_init);
        }

        let log_call = if cfg!(feature = "verbose") {
            quote! {
                ::simple_parse::debug!("Self::{}", stringify!(#variant_name));
            }
        } else {
            quote! {}
        };

        init_code.extend(quote! {
            #variant_id => {
                #log_call
                #read_code
                unsafe {
                    Self::#variant_name#variant_content
                }
            }
        })
    }

    quote!{
        let mut tmp = MaybeUninit::uninit();
        match *<#id_type>::inner_from_reader(src, ctx, &mut tmp)? {
            #init_code
            _ => return Err(::simple_parse::SpError::UnknownEnumVariant),
        }
    }
}
