use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro::TokenStream;
use syn::{Field, Fields, GenericParam, Generics};
use quote::quote;

mod read;
mod write;

#[proc_macro_derive(SpRead, attributes(sp))]
pub fn generate_read(input: TokenStream) -> TokenStream {
    read::generate(input)
}

#[proc_macro_derive(SpWrite, attributes(sp))]
pub fn generate_write(input: TokenStream) -> TokenStream {
    write::generate(input)
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(sp))]
pub(crate) struct StructAttributes {
    /// Specifies the endiannes for the whole enum. The data will
    /// be converted to the native endianness when necessary.
    #[darling(default)]
    endian: Option<String>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(sp))]
pub(crate) struct EnumAttributes {
    /// The type used to parse the variant id
    #[darling(default)]
    id_type: Option<String>,

    /// Specifies the endiannes for the whole enum. The data will
    /// be converted to the native endianness when necessary.
    #[darling(default)]
    endian: Option<String>,
}

#[derive(Debug, FromField)]
#[darling(attributes(sp))]
pub(crate) struct FieldAttributes {
    /// Points to the field name/index that contains the number of items to parse for the Vec
    /// e.g.
    /// ```Rust
    /// struct Test {
    ///     num_options: u8,
    ///     #[sp(count="num_options")]
    ///     options: Vec<Options>,
    /// }
    /// ```
    #[darling(default)]
    count: Option<String>,

    /// Allows the use of a custom byte reading function. This function must have the same
    /// return type as SpRead::inner_from_bytes
    /// Variables in scope :
    ///     input : The input bytes
    ///     is_input_le : If input is in little endian
    ///     count : An Option<number> that contains the number of items to parse
    /// e.g #[sp(reader="custom_reader(input, is_input_le, count)")]
    #[darling(default)]
    reader: Option<String>,

    /// Allows the use of a custom byte writing function.
    #[darling(default)]
    writer: Option<String>,

    /// Specifies the endiannes of the specific field. The data will
    /// be converted to the native endianness when necessary.
    #[darling(default)]
    endian: Option<String>,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(sp))]
pub(crate) struct VariantAttributes {
    id: usize,
}

/// Adds a bound to a generic parameter
pub(crate) fn add_trait_bounds(mut generics: Generics, new_bound: syn::TypeParamBound) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(new_bound.clone());
        }
    }
    generics
}

// Returns the name of a field.
// e.g
//      some_field  // Named
//      field_0     // Unnamed
//      my_struct.some_field    //with obj_name = Some(my_struct)
//      my_struct.0             //with obj_name = Some(my_struct)
pub(crate) fn generate_field_name(
    field: &Field,
    idx: usize,
    obj_name: Option<&str>,
) -> proc_macro2::TokenStream {
    match field.ident {
        Some(ref i) => {
            if let Some(name) = obj_name {
                format!("{}.{}", name, i)
            //Ident::new(&format!("{}.{}", name, i), proc_macro2::Span::call_site())
            } else {
                format!("{}", i)
                //Ident::new(&format!("{}", i), proc_macro2::Span::call_site())
            }
        }
        None => {
            if let Some(name) = obj_name {
                format!("{}.{}", name, idx)
            //Ident::new(&format!("{}.{}", name, idx), proc_macro2::Span::call_site())
            } else {
                format!("field_{}", idx)
                //Ident::new(&format!("field_{}", idx), proc_macro2::Span::call_site())
            }
        }
    }
    .parse()
    .unwrap()
}

// Returns either None or Some(<field_name>). The return value is meant to be used as the count argument to from_bytes/to_bytes
pub(crate) fn generate_count_field_name(
    count: Option<String>,
    fields: &Fields,
    obj_name: Option<&str>,
) -> Option<proc_macro2::TokenStream> {
    let count_val = match count {
        None => return None,
        Some(s) => s,
    };

    match fields {
        Fields::Unit => None,
        Fields::Named(_) => {
            let obj_name = if let Some(name) = obj_name {
                format!("{}.{}", name, count_val)
            //Ident::new(&format!("{}.{}", name, count_val),proc_macro2::Span::call_site())
            } else {
                format!("{}", count_val)
                //Ident::new(&format!("{}", count_val), proc_macro2::Span::call_site())
            };

            Some(obj_name.parse().unwrap())
        }
        Fields::Unnamed(_) => {
            let field_idx = count_val.parse::<usize>().unwrap();
            let obj_name = if let Some(name) = obj_name {
                format!("{}.{}", name, field_idx)
            //Ident::new(&format!("{}.{}", name, field_idx),proc_macro2::Span::call_site(),)
            } else {
                format!("field_{}", field_idx)
                //Ident::new(&format!("field_{}", field_idx),proc_macro2::Span::call_site(),)
            };
            Some(obj_name.parse().unwrap())
        }
    }
}

/// Returns whether the string is set to "little"
pub(crate) fn is_lower_endian(val: &str) -> bool {
    if val == "little" {
        true
    } else if val == "big" {
        false
    } else {
        panic!("Unknown endian specified : {}", val);
    }
}



/// Generates the proper code to initialize the object
/// e.g :
///     (field_0, field_1, field_2)
/// OR
///     {some_field, timestamp, secret_key}
fn generate_field_list(fields: &Fields, field_idents: Option<&Vec<proc_macro2::TokenStream>>) -> proc_macro2::TokenStream {
    let mut tmp;
    let field_idents = match field_idents {
        Some(f) => f,
        None => {
            tmp = Vec::with_capacity(fields.len());
            for (idx, field) in fields.iter().enumerate() {
                tmp.push(generate_field_name(field, idx, None));
            }
            &tmp
        }
    };

    match fields {
        Fields::Named(_) => quote! {
            {#(#field_idents),*}
        },
        Fields::Unnamed(_) => quote! {
            (#(#field_idents),*)
        },
        Fields::Unit => {
            quote! {}
        }
    }
}