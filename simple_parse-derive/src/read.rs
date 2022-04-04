use crate::*;
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DataEnum, DeriveInput, Fields};

pub(crate) fn generate(input: &mut DeriveInput) -> proc_macro2::TokenStream {
    let mut log_call: TokenStream = TokenStream::new();
    let unsafe_read_code: TokenStream;
    let result_obj: TokenStream;

    // Generate the code that implements SpRead
    match input.data {
        // Parse as a struct
        Data::Struct(ref contents) => {
            let attrs: StructAttributes = FromDeriveInput::from_derive_input(&input).unwrap();
            let (read, fields) = generate_fields_read(&contents.fields, attrs.endian.as_deref());
            unsafe_read_code = read;
            result_obj = quote! {Self{#fields}};

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
            let (read, res) = generate_enum_read(contents, &attrs);
            unsafe_read_code = read;
            result_obj = res;

            if cfg!(feature = "verbose") {
                let name = &input.ident;
                log_call = quote! {
                    ::simple_parse::debug!("Read enum {}", stringify!(#name));
                };
            }
        }
        // Unhandled derive usage
        _ => unimplemented!("Cannot derive on this type"),
    };

    add_trait_bounds(&mut input.generics, parse_quote! {simple_parse::SpRead});
    let name = &input.ident;
    let (_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // TODO When deriving a generic type, we run into const generic issues when using Self::STATIC_SIZE.
    //      We could check whether the type has generics and instead of using Self::STATIC_SIZE
    //      directly, we can use opt_hint.rs's generated STATIC_SIZE code ?
    let (stack_var, static_validation_code) = (
        quote! {
            let mut tmp_stack;
        },
        quote! {
            tmp_stack = [0u8; <Self as ::simple_parse::SpOptHints>::STATIC_SIZE];
            ::simple_parse::validate_reader_exact(ctx, &mut tmp_stack, src)?;
            checked_bytes = tmp_stack.as_mut_ptr();
        },
    );

    // Generate impl block
    let mut res = quote! {
        impl ::simple_parse::SpRead for #name #ty_generics #where_clause {
            fn inner_from_reader<R: std::io::Read + ?Sized>(
                src: &mut R,
                ctx: &mut ::simple_parse::SpCtx,
            ) -> std::result::Result<Self, ::simple_parse::SpError>
            where
                Self: Sized
            {
                #log_call
                let mut checked_bytes: *mut u8 = std::ptr::null_mut();
                #stack_var
                #static_validation_code
                unsafe {
                    Self::inner_from_reader_unchecked(checked_bytes, src, ctx)
                }
            }
            unsafe fn inner_from_reader_unchecked<R: std::io::Read + ?Sized>(
                mut checked_bytes: *mut u8,
                src: &mut R,
                ctx: &mut ::simple_parse::SpCtx,
            ) -> std::result::Result<Self, ::simple_parse::SpError>
            where
                Self: Sized + ::simple_parse::SpOptHints {
                    #unsafe_read_code
                    Ok(#result_obj)
                }
        }
    };

    // Automatically impl `SpOptHints` when deriving `SpRead`
    let generate_opt_hints = crate::opt_hints::generate(input);

    #[cfg(feature = "print-generated")]
    println!("{}", res.to_string());

    res.extend(quote! {
        #generate_opt_hints
    });

    res
}

/// Generates code that parses bytes into a struct
fn generate_fields_read(fields: &Fields, endian: Option<&str>) -> (TokenStream, TokenStream) {
    let mut read_code = TokenStream::new();
    let mut field_list = TokenStream::new();
    let num_fields = fields.len();

    // No fields
    if num_fields == 0 {
        return (read_code, field_list);
    }

    let default_is_le: bool = match endian {
        None => cfg!(target_endian = "little"),
        Some(s) => is_lower_endian(s),
    };

    let mut hit_first_dyn = false;
    let mut num_summed_sizes = 0;
    let mut static_size_code = TokenStream::new();
    let mut queued_read_code = TokenStream::new();

    let fields: Vec<&syn::Field> = fields.iter().collect();

    // holds the index of a field's `len` field
    let mut count_field_idx = Vec::with_capacity(fields.len());
    count_field_idx.resize(fields.len(), None);

    let mut simple_field_names = Vec::with_capacity(fields.len());

    // Iterate to link up content fields to their `len` field index
    for (idx, field) in fields.iter().enumerate() {
        let field_attrs: FieldAttributes = FromField::from_field(field).unwrap();
        // save the simple field name for each field seen so far
        simple_field_names.push(generate_field_name(field, idx, None, false).to_string());

        if let Some(count_field_name) = field_attrs.len.as_ref() {
            let mut found_idx = idx;
            for i in 0..idx {
                if count_field_name.as_str() == simple_field_names[i].as_str() {
                    found_idx = i;
                    break;
                }
            }
            // `len` field not found
            if found_idx == idx {
                panic!("#[sp(len)] annotation on field '{}' referers to an unknown field '{}'. Valid values are {:?}", &simple_field_names[idx], count_field_name, &simple_field_names[..idx]);
            }

            // Save link from current field to `len` field
            count_field_idx[idx] = Some(found_idx);
        }
    }

    for (idx, field) in fields.iter().enumerate() {
        let field_name = generate_field_name(field, idx, None, false);
        field_list.extend(quote! {#field_name,});
        let field_type = strip_lifetimes(&field.ty);
        let field_attrs: FieldAttributes = FromField::from_field(field).unwrap();
        let is_var_type = is_var_size(&field_type, Some(&field_attrs));

        // Get this field's endianness
        let is_input_le = match field_attrs.endian {
            None => default_is_le,
            Some(ref e) => is_lower_endian(e),
        };

        let mut cur_field_size = quote! {
            <#field_type as ::simple_parse::SpOptHints>::STATIC_SIZE
        };
        if field_attrs.len.is_some() {
            cur_field_size.extend(quote! {
                - <#field_type as ::simple_parse::SpOptHints>::COUNT_SIZE
            })
        }

        if field_attrs.reader.is_none() {
            // Start aggregating sizes for static fields after hitting the first dyn field
            if hit_first_dyn {
                num_summed_sizes += 1;
                if num_summed_sizes > 1 {
                    static_size_code.extend(quote! { + });
                }
                // Add this field to aggregated static field sizes
                static_size_code.extend(quote! {#cur_field_size});
            }
        }

        // Get `len` field
        let mut len_value = quote! {None};
        if let Some(count_idx) = count_field_idx[idx] {
            let count_field_name = generate_field_name(&fields[count_idx], count_idx, None, true);
            len_value = quote! {Some(#count_field_name as _)};
        }

        // Get custom validator
        let validate_call = match field_attrs.validate {
            Some(ref s) => {
                let (fn_name, other_fields) =
                    match split_custom_attr(s, &fields, idx, None, AllowFields::AfterCurrentAsNone)
                    {
                        Ok(v) => v,
                        Err(e) => {
                            panic!(
                                "Invalid custom validator for field '{}', {}",
                                &simple_field_names[idx], e
                            );
                        }
                    };
                quote! {
                    ctx.is_reading = true;
                    #fn_name(&mut #field_name, #other_fields ctx)?;
                }
            }
            None => TokenStream::new(),
        };

        // Get custom reader if provided
        let read_call = match field_attrs.reader {
            Some(ref s) => {
                let (fn_name, dependent_fields) =
                    match split_custom_attr(s, &fields, idx, None, AllowFields::BeforeCurrent) {
                        Ok(v) => v,
                        Err(e) => {
                            panic!(
                                "Invalid custom reader for field '{}', {}",
                                &simple_field_names[idx], e
                            );
                        }
                    };
                quote! {
                    #fn_name(#dependent_fields src, ctx)
                }
            }
            None => {
                // Call regular function
                quote! {
                    <#field_type>::inner_from_reader_unchecked(checked_bytes, src, ctx)
                }
            }
        };

        queued_read_code.extend(quote! {
            ctx.is_little_endian = #is_input_le;
            ctx.len = #len_value;
            let mut #field_name: #field_type = #read_call?;
            #validate_call
        });

        if is_var_type || idx + 1 == num_fields {
            if hit_first_dyn && num_summed_sizes > 0 {
                let (stack_var, validate_static_size) = (
                    quote! {
                        let mut tmp_stack;
                    },
                    quote! {
                        tmp_stack = [0u8; static_size];
                        ::simple_parse::validate_reader_exact(ctx, &mut tmp_stack, src)?;
                        checked_bytes = tmp_stack.as_mut_ptr();
                    },
                );

                read_code.extend(quote! {
                    #stack_var
                    {
                        const static_size: usize = #static_size_code;
                        if static_size > 0 {
                            #validate_static_size
                        }
                    }
                });

                num_summed_sizes = 0;
                // Reset the static size
                static_size_code = quote! {};
            }

            read_code.extend(queued_read_code);
            queued_read_code = TokenStream::new();

            if is_var_type {
                hit_first_dyn = true;
            }
        } else {
            // Move the checked_bytes pointer forward for the next field
            queued_read_code.extend(quote! {
                checked_bytes = checked_bytes.add(#cur_field_size);
            });
        }
    }

    (read_code, field_list)
}

/// Generates the code that parse bytes into an enum variant
fn generate_enum_read(data: &DataEnum, attrs: &EnumAttributes) -> (TokenStream, TokenStream) {
    if data.variants.is_empty() {
        return (quote! {}, quote! {Self});
    }

    let default_is_le: bool = match attrs.endian {
        None => cfg!(target_endian = "little"),
        Some(ref e) => is_lower_endian(e),
    };

    let id_type = get_enum_id_type(data, attrs);

    // Read the id
    let mut variant_read_code = TokenStream::new();
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

        let (read_code, field_list) = if !variant.fields.is_empty() {
            let (read, list) = generate_fields_read(&variant.fields, var_attrs.endian.as_deref());

            let res = opt_hints::generate_struct_hints(&variant.fields);
            let fields_size = res.static_size;
            let (stack_var, validate_field_size) = (
                quote! {
                    let mut tmp_stack;
                },
                quote! {
                    tmp_stack = [0u8; #fields_size];
                    ::simple_parse::validate_reader_exact(ctx, &mut tmp_stack, src)?;
                    checked_bytes = tmp_stack.as_mut_ptr();
                },
            );
            (
                quote! {
                    #stack_var
                    if <Self as ::simple_parse::SpOptHints>::STATIC_SIZE == <#id_type as ::simple_parse::SpOptHints>::STATIC_SIZE {
                        #validate_field_size
                    }
                    #read
                },
                if let syn::Fields::Unnamed(_r) = &variant.fields {
                    quote! {
                        (#list)
                    }
                } else {
                    quote! {
                        {#list}
                    }
                },
            )
        } else {
            (TokenStream::new(), TokenStream::new())
        };

        let log_call = if cfg!(feature = "verbose") {
            quote! {
                ::simple_parse::debug!("Read variant {}", stringify!(#variant_name));
            }
        } else {
            quote! {}
        };

        variant_read_code.extend(quote! {
            #variant_id => {
                #log_call
                #read_code
                Self::#variant_name#field_list
            }
        })
    }

    (
        quote! {
            ctx.is_little_endian = #default_is_le;
            ctx.len = None;
            let variant_id = <#id_type>::inner_from_reader_unchecked(checked_bytes, src, ctx)?;
            checked_bytes = checked_bytes.add(<#id_type as ::simple_parse::SpOptHints>::STATIC_SIZE);
            let res = match variant_id {
                #variant_read_code
                _ => return Err(::simple_parse::SpError::UnknownEnumVariant),
            };
        },
        quote! {res},
    )
}
