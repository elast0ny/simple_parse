use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DataEnum, Fields, DeriveInput};

use crate::*;

pub(crate) fn generate(
    input: &mut DeriveInput,
) -> proc_macro2::TokenStream {
    // Generate the code that implements SpRead
    let (is_var_code, static_size_code, const_asserts) = match input.data {
        // Parse as a struct
        Data::Struct(ref contents) => {
            let r = generate_struct_hints(&contents.fields);
            let is_var = r.0;
            (quote!{#is_var}, r.1, r.2)
        }
        // Parse as enum
        Data::Enum(ref contents) => {
            let attrs = FromDeriveInput::from_derive_input(&input).unwrap();
            generate_enum_hints(contents, attrs)
        }
        // Unhandled derive usage
        _ => unimplemented!("Cannot derive on this type"),
    };

    add_trait_bounds(&mut input.generics, parse_quote! {simple_parse::SpRead});
    let name = &input.ident;
    let (_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Generate impl block for TYPE and &TYPE
    let res = quote! {
        #const_asserts
        unsafe impl #ty_generics ::simple_parse::SpOptHints for #name #ty_generics #where_clause {
           const IS_VAR_SIZE: bool = #is_var_code;
           const STATIC_SIZE: usize = #static_size_code;
        }
    };

    #[cfg(feature = "print-generated")]
    println!("{}", res);

    res
}

/// Returns whether any of the fields are variably sized and a TokenStream of all their sizes added until the var field 
/// e.g. field1::STATIC_SIZE + field2::STATIC_SIZE + var_field::STATIC_SIZE
fn generate_struct_hints(
    fields: &Fields,
) -> (bool, TokenStream, TokenStream) {
    
    if fields.is_empty() {
        return (false, quote!{0}, quote!{})
    }
    
    let mut static_size = TokenStream::new();
    let mut const_asserts = TokenStream::new();
    let mut got_var = false;

    // Add up each field's static_size until we hit a variable size field
    for (idx, field) in fields.iter().enumerate() {
        //let field_name = &field.ident;
        let field_type = &field.ty;
        let field_attrs: FieldAttributes = FromField::from_field(&field).unwrap();
        let is_var_type = is_var_size(field_type, Some(&field_attrs));
        
        if !got_var {
            if idx == 0 {
                static_size.extend(quote!{
                    <#field_type as ::simple_parse::SpOptHints>::STATIC_SIZE
                });
            } else {
                static_size.extend(quote!{
                    + <#field_type as ::simple_parse::SpOptHints>::STATIC_SIZE
                });
            }
        }
        
        if is_var_type {
            //let msg = syn::Ident::new(&format!("{}_{}_is_static_size", quote!{#field_name}.to_string(), quote!{#field_type}.to_string()), field.ty.span());
            got_var = true;
            const_asserts.extend(quote!{
                ::simple_parse::sa::const_assert!(<#field_type as ::simple_parse::SpOptHints>::IS_VAR_SIZE == true);
                
            });
        } else {
            //let msg = syn::Ident::new(&format!("{}_{}_is_variable_size", quote!{#field_name}.to_string(), quote!{#field_type}.to_string()), field.ty.span());
            const_asserts.extend(quote!{
                ::simple_parse::sa::const_assert!(<#field_type as ::simple_parse::SpOptHints>::IS_VAR_SIZE == false); 
            });
        }
    }

    (got_var, static_size, quote!{ #const_asserts })
}

/// Generates code for enums
fn generate_enum_hints(
    data: &DataEnum,
    attrs: EnumAttributes,
) -> (
    TokenStream, // Code for IS_VAR_SIZE
    TokenStream, // Code for STATIC_SIZE
    TokenStream  // Constant asserts to put before the impl block
) {
    // Get the size of the enum id
    let id_type = get_enum_id_type(data, &attrs);

    let mut is_var = false;

    let is_var_size;
    let mut static_size = quote!{0};
    let mut const_asserts = TokenStream::new();
    let mut variant_size_comparison = TokenStream::new();
    let num_variants = data.variants.len();
    if num_variants == 0 {
        return (quote!{false}, quote!{<#id_type>::STATIC_SIZE}, quote!{});
    }

    // Go through each variant to calculate their size
    for (idx, variant) in data.variants.iter().enumerate() {
        let (got_var, size, asserts) = generate_struct_hints(&variant.fields);
        static_size = size;
        const_asserts.extend(asserts);

        // If one variant has a var size, the whole enum is var
        if got_var {
            is_var = true;
        }

        // Build a comparison chain comparing variant sizes with the next variant. Returns true if any doesnt match
        if idx == 0 {
            variant_size_comparison.extend(quote!{
                #static_size
            });
        } else {
            variant_size_comparison.extend(quote!{
                != #static_size || #static_size
            });
        }

        // Close the comparison chain on the last variant
        if idx + 1 == num_variants {
            variant_size_comparison.extend(quote!{
                != #static_size
            });
        }
    }

    // If any variant was variably sized, enum is too
    if is_var {
        is_var_size = quote!{true};
    } else {
        // Return true if variants dont have the same size
        is_var_size = quote!{#variant_size_comparison};
    }
    
    // This sets the static size to [id_type + common static_size between ALL variants]
    static_size = quote!{
        <#id_type>::STATIC_SIZE + if !(#variant_size_comparison) {
            #static_size
        } else {
            0
        }
    };
    
    // Enum's static portion is simply its enum_id
    // static_size = quote!{<#id_type>::STATIC_SIZE};

    (is_var_size, static_size, const_asserts)
}