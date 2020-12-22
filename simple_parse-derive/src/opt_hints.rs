use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DataEnum, Fields, DeriveInput};

use crate::*;

pub (crate) enum IsVar {
    Bool(bool),
    TS(TokenStream),
}

pub (crate) struct GeneratedOptHints {
    is_var: IsVar,
    static_size: TokenStream,
    const_asserts: TokenStream,
}
impl Default for GeneratedOptHints {
    fn default() -> Self {
        Self {
            is_var: IsVar::Bool(false),
            static_size: quote!{0},
            const_asserts: TokenStream::new(),
        }
    }
}

pub(crate) fn generate(
    input: &mut DeriveInput,
) -> proc_macro2::TokenStream {
    // Generate the code that implements SpRead
    let res: GeneratedOptHints = match input.data {
        // Parse as a struct
        Data::Struct(ref contents) => {
            generate_struct_hints(&contents.fields)
        }
        // Parse as enum
        Data::Enum(ref contents) => {
            let attrs = FromDeriveInput::from_derive_input(&input).unwrap();
            generate_enum_hints(contents, attrs)
        }
        // Unhandled derive usage
        _ => unimplemented!("Cannot derive on this type"),
    };

    let (is_var_code, static_size_code, const_asserts) = (
        match res.is_var {
            IsVar::Bool(v) => quote!{#v},
            IsVar::TS(v) => v,
        },
        res.static_size,
        res.const_asserts,
    );

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

/// Returns aggregated SpOptHint information from fields.
pub (crate) fn generate_struct_hints(
    fields: &Fields,
) -> GeneratedOptHints {
    let mut r = GeneratedOptHints::default();
    if fields.is_empty() {
        return r;
    }

    r.static_size = TokenStream::new();
    
    let mut got_var = false;

    // Add up each field's static_size until we hit a variable size field
    for (idx, field) in fields.iter().enumerate() {
        //let field_name = &field.ident;
        let field_type = &field.ty;
        let field_attrs: FieldAttributes = FromField::from_field(&field).unwrap();
        let is_var_type = is_var_size(field_type, Some(&field_attrs));
        
        // Keep adding sizes as long as we havent hit a dynamically sized field
        if !got_var {
            if idx > 0 {
                r.static_size.extend(quote!{ + });
            }

            r.static_size.extend(get_static_size(field_type));
            
            // If field has a count annotation, remove the COUNT_SIZE from it
            if field_attrs.count.is_some() {
                r.static_size.extend(quote!{
                    - <#field_type as ::simple_parse::SpOptHints>::COUNT_SIZE
                });
            }
        }
        
        if is_var_type {
            got_var = true;
            r.const_asserts.extend(quote!{
                ::simple_parse::sa::const_assert!(<#field_type as ::simple_parse::SpOptHints>::IS_VAR_SIZE == true);
                
            });
        } else {
            r.const_asserts.extend(quote!{
                ::simple_parse::sa::const_assert!(<#field_type as ::simple_parse::SpOptHints>::IS_VAR_SIZE == false); 
            });
        }
    }

    r.is_var = IsVar::Bool(got_var);

    r
}

/// Generates aggregated SpOptHints for an enum
pub (crate) fn generate_enum_hints(
    data: &DataEnum,
    attrs: EnumAttributes,
) -> GeneratedOptHints {
    // Get the size of the enum id
    let id_type = get_enum_id_type(data, &attrs);
    let num_variants = data.variants.len();
    let mut r = GeneratedOptHints::default();
    
    if num_variants == 0 {
        // At minimum, read the variant ID
        r.static_size = quote!{<#id_type>::STATIC_SIZE};
        return r;
    }

    let mut is_var = false;
    let mut cur_variant_size = quote!{0};
    let mut variant_size_comparison = TokenStream::new();
    
    // Go through each variant to calculate their size
    for (idx, variant) in data.variants.iter().enumerate() {
        let field_hints = generate_struct_hints(&variant.fields);
        cur_variant_size = field_hints.static_size;
        r.const_asserts.extend(field_hints.const_asserts);

        // If one variant has a var size, the whole enum is var
        if let IsVar::Bool(v) = field_hints.is_var {
            if v {
                is_var = true;
            }
        }

        // Build a comparison chain comparing variant sizes with the next variant. Returns true if any doesnt match
        if idx == 0 {
            variant_size_comparison.extend(quote!{
                #cur_variant_size
            });
        } else {
            variant_size_comparison.extend(quote!{
                != #cur_variant_size || #cur_variant_size
            });
        }

        // Close the comparison chain on the last variant
        if idx + 1 == num_variants {
            variant_size_comparison.extend(quote!{
                != #cur_variant_size
            });
        }
    }

    // If any variant was variably sized, enum is too
    if is_var {
        r.is_var = IsVar::Bool(true);
    } else {
        // Return true if variants dont have the same size
        r.is_var = IsVar::TS(quote!{#variant_size_comparison});
    }
    
    // This sets the static size to [id_type + common static_size between ALL variants]
    r.static_size = quote!{
        <#id_type>::STATIC_SIZE + if !(#variant_size_comparison) {
            #cur_variant_size
        } else {
            0
        }
    };

    r
}