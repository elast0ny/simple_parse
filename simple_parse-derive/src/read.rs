use crate::*;
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DataEnum, DeriveInput, Fields};

fn generate_validate_size_code(static_size: &TokenStream, reader_type: &ReaderType) -> TokenStream {
    match reader_type {
        ReaderType::Reader => {
            quote! {
                tmp = [0u8; #static_size];            
                ::simple_parse::validate_reader_exact(&mut tmp, src)?;
                checked_bytes = tmp.as_mut_ptr();
            }
        },
        _ => {
            quote! {
                checked_bytes = ::simple_parse::validate_cursor(#static_size, src)?;
            }
        }
    }
}

pub(crate) fn generate(
    input: &mut DeriveInput,
    reader_type: ReaderType,
) -> proc_macro2::TokenStream {
    let mut log_call: TokenStream = TokenStream::new();
    let unsafe_read_code: TokenStream;
    let result_obj: TokenStream;

    // Generate the code that implements SpRead
    match input.data {
        // Parse as a struct
        Data::Struct(ref contents) => {
            let attrs: StructAttributes = FromDeriveInput::from_derive_input(&input).unwrap();
            let (read, fields) = generate_fields_read(
                &reader_type,
                &contents.fields,
                attrs.endian.as_deref()
            );
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
            let (read, res) = generate_enum_read(&reader_type, contents, &attrs);
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
    //      We could check whether the type has generics and instead of using STATIC_SIZE
    //      directly, we can call opt_hint.rs' generate functions to use the same expresion
    //      as it generated for the Self::STATIC_SIZE...
    let static_validation_code =
        generate_validate_size_code(&quote! {<#name as ::simple_parse::SpOptHints>::STATIC_SIZE}, &reader_type);

    // Generate impl block for TYPE and &TYPE
    let mut res = match reader_type {
        ReaderType::Reader => {
            quote! {
            impl ::simple_parse::SpRead for #name #ty_generics #where_clause {
                fn inner_from_reader<R: std::io::Read + ?Sized>(
                    src: &mut R,
                    is_input_le: bool,
                    count: Option<usize>,
                ) -> std::result::Result<Self, ::simple_parse::SpError>
                where
                    Self: Sized
                {
                    #log_call
                    let mut checked_bytes: *mut u8 = std::ptr::null_mut();
                    let mut tmp;
                    #static_validation_code
                    unsafe {
                        Self::inner_from_reader_unchecked(checked_bytes, src, is_input_le, count)
                    }
                }
                unsafe fn inner_from_reader_unchecked<R: std::io::Read + ?Sized>(
                    mut checked_bytes: *mut u8,
                    src: &mut R,
                    is_input_le: bool,
                    count: Option<usize>,
                ) -> Result<Self, ::simple_parse::SpError>
                where
                    Self: Sized + ::simple_parse::SpOptHints {
                        #unsafe_read_code
                        Ok(#result_obj)
                    }
            }}
        }
        ReaderType::Raw => {
            // Use the first lifetime parameter as the one bound to the input bytes
            let lifetime = if let Some(lt) = input.generics.lifetimes().next() {
                quote! {#lt}
            } else {
                quote! {'_}
            };

            quote! {
                impl #ty_generics ::simple_parse::SpReadRaw<#lifetime> for #name #ty_generics #where_clause {
                    fn inner_from_slice(
                        src: &mut std::io::Cursor<&#lifetime [u8]>,
                        is_input_le: bool,
                        count: Option<usize>,
                    ) -> std::result::Result<Self, ::simple_parse::SpError>
                    where
                        Self: Sized,
                    {
                        #log_call
                        let mut checked_bytes: *mut u8 = std::ptr::null_mut();
                        #static_validation_code
                        unsafe {
                            Self::inner_from_slice_unchecked(checked_bytes, src, is_input_le, count)
                        }
                    }

                    unsafe fn inner_from_slice_unchecked(
                        mut checked_bytes: *const u8,
                        src: &mut std::io::Cursor<&#lifetime [u8]>,
                        is_input_le: bool,
                        count: Option<usize>,
                    ) -> Result<Self, ::simple_parse::SpError>
                    where
                        Self: Sized + ::simple_parse::SpOptHints {
                            #unsafe_read_code
                            Ok(#result_obj)
                        }
                }
            }
        }
        ReaderType::RawMut => {
            // Use the first lifetime parameter as the one bound to the input bytes
            let lifetime = if let Some(lt) = input.generics.lifetimes().next() {
                quote! {#lt}
            } else {
                quote! {'_}
            };

            quote! {
                impl #ty_generics ::simple_parse::SpReadRawMut<#lifetime> for #name #ty_generics #where_clause {
                    fn inner_from_mut_slice(
                        src: &mut std::io::Cursor<&#lifetime mut [u8]>,
                        is_input_le: bool,
                        count: Option<usize>,
                    ) -> std::result::Result<Self, ::simple_parse::SpError>
                    where
                        Self: Sized,
                    {
                        #log_call
                        let mut checked_bytes: *mut u8 = std::ptr::null_mut();
                        #static_validation_code
                        unsafe {
                            Self::inner_from_slice_unchecked(checked_bytes, src, is_input_le, count)
                        }
                    }

                    unsafe fn inner_from_mut_slice_unchecked(
                        mut checked_bytes: *mut u8,
                        src: &mut std::io::Cursor<&#lifetime mut [u8]>,
                        is_input_le: bool,
                        count: Option<usize>,
                    ) -> Result<Self, ::simple_parse::SpError>
                    where
                        Self: Sized + ::simple_parse::SpOptHints {
                            #unsafe_read_code
                            Ok(#result_obj)
                        }
                }
            }
        }
    };

    // Automatically impl `SpOptHints` when deriving `SpRead`
    if let ReaderType::Reader = reader_type {
        let generate_opt_hints = crate::opt_hints::generate(input);
        
        #[cfg(feature = "print-generated")]
        println!("{}", res.to_string());

        res.extend(quote! {
            #generate_opt_hints
        })
    } else {
        #[cfg(feature = "print-generated")]
        println!("{}", res.to_string());
    }

    res
}

/// Generates code that parses bytes into a struct
fn generate_fields_read(
    reader_type: &ReaderType,
    fields: &Fields,
    endian: Option<&str>,
) -> (TokenStream, TokenStream) {
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

    for (idx, field) in fields.iter().enumerate() {
        let field_name = generate_field_name(field, idx, None, false);
        field_list.extend(quote! {#field_name,});
        let field_type = &field.ty;
        let field_attrs: FieldAttributes = FromField::from_field(field).unwrap();
        let is_var_type = is_var_size(field_type, Some(&field_attrs));

        // Get this field's endianness
        let is_input_le = match field_attrs.endian {
            None => default_is_le,
            Some(ref e) => is_lower_endian(e),
        };

        let mut cur_field_size = quote!{
            <#field_type as ::simple_parse::SpOptHints>::STATIC_SIZE
        };
        if field_attrs.count.is_some() {
            cur_field_size.extend(quote!{
                - <#field_type as ::simple_parse::SpOptHints>::COUNT_SIZE
            })
        }

        // Start aggregating sizes for static fields after hitting the first dyn field
        if hit_first_dyn {
            num_summed_sizes += 1;
            if num_summed_sizes > 1 {
                static_size_code.extend(quote! { + });
            }
            // Add this field to aggregated static field sizes
            static_size_code.extend(quote!{#cur_field_size});
        }

        // Get the count field
        let count_field = match generate_count_field_name(&field_attrs.count, fields, None, true) {
            None => {
                quote! {
                    None
                }
            }
            Some(c) => {
                quote! {Some(#c as _)}
            }
        };

        // Get custom reader if provided
        #[allow(unreachable_code, unused_variables)]
        let read_call = match field_attrs.reader {
            Some(ref s) => {
                panic!("Custom reader not implemented yet !");
                let s: TokenStream = s.parse().unwrap();
                quote! {
                    {
                        let is_input_le = #is_input_le;
                        let count: Option<usize> = #count_field;
                        #s
                    }
                }
            }
            None => {
                // Call regular function
                let fn_name = get_parse_fn_name(&reader_type, true);
                quote! {
                    <#field_type>::#fn_name(checked_bytes, src, #is_input_le, #count_field)
                }
            }
        };

        queued_read_code.extend(quote! {
            let #field_name: #field_type = #read_call?;
        });

        if is_var_type || idx + 1 == num_fields {
            if hit_first_dyn {
                let validate_static_size =
                    generate_validate_size_code(&quote! {#static_size_code}, &reader_type);
                read_code.extend(quote! {
                    let mut tmp;
                    #validate_static_size
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
fn generate_enum_read(
    reader_type: &ReaderType,
    data: &DataEnum,
    attrs: &EnumAttributes,
) -> (TokenStream, TokenStream) {
    if data.variants.is_empty() {
        return (quote! {}, quote! {Self});
    }

    let fn_name = get_parse_fn_name(reader_type, true);
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
            let (read, list) = generate_fields_read(
                reader_type,
                &variant.fields,
                var_attrs.endian.as_deref()
            );

            let res = opt_hints::generate_struct_hints(&variant.fields);
            let fields_size = res.static_size;
            let validate_field_size = generate_validate_size_code(&quote!{#fields_size}, reader_type);
            (
                quote!{
                    let mut tmp;
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
            let variant_id = <#id_type>::#fn_name(checked_bytes, src, #default_is_le, None)?;
            checked_bytes = checked_bytes.add(<#id_type as ::simple_parse::SpOptHints>::STATIC_SIZE);
            let res = match variant_id {
                #variant_read_code
                _ => return Err(::simple_parse::SpError::UnknownEnumVariant),
            };
        },
        quote! {res},
    )
}
