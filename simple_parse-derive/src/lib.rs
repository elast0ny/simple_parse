use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Field, Fields, GenericParam, Generics};

mod read;
mod write;
mod attributes;
pub (crate) use attributes::*;

pub (crate) enum ReaderType {
    Reader,
    Raw,
    RawMut
}

#[proc_macro_derive(SpRead, attributes(sp))]
/// For a list of valid `#[sp(X)]` attributes, consult [attributes.rs](https://github.com/elast0ny/simple_parse/tree/master/simple_parse-derive/src/attributes.rs)
pub fn generate_read(input: TokenStream) -> TokenStream {
    read::generate(input, ReaderType::Reader)
}
#[proc_macro_derive(SpReadRaw, attributes(sp))]
/// For a list of valid `#[sp(X)]` attributes, consult [attributes.rs](https://github.com/elast0ny/simple_parse/tree/master/simple_parse-derive/src/attributes.rs)
pub fn generate_readraw(input: TokenStream) -> TokenStream {
    read::generate(input, ReaderType::Raw)
}
#[proc_macro_derive(SpReadRawMut, attributes(sp))]
/// For a list of valid `#[sp(X)]` attributes, consult [attributes.rs](https://github.com/elast0ny/simple_parse/tree/master/simple_parse-derive/src/attributes.rs)
pub fn generate_readrawmut(input: TokenStream) -> TokenStream {
    read::generate(input, ReaderType::RawMut)
}

#[proc_macro_derive(SpWrite, attributes(sp))]
/// For a list of valid `#[sp(X)]` attributes, consult [attributes.rs](https://github.com/elast0ny/simple_parse/tree/master/simple_parse-derive/src/attributes.rs)
pub fn generate_write(input: TokenStream) -> TokenStream {
    write::generate(input)
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
    deref_references: bool,
) -> proc_macro2::TokenStream {
    let mut fname = match field.ident {
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
    };

    if deref_references {
        let field_type = &field.ty;
        let field_type = quote! {#field_type}.to_string();
        if field_type.starts_with('&') {
            fname = format!("*{}", fname);
        }
    }
    
    fname.parse()
    .unwrap()
}

/// Converts `count` into a useable field name. Returns None if `count` is None or if the value of `count` cannot be found.
/// eg. self.num_field | *self.num_field | field_01
pub(crate) fn generate_count_field_name(
    count: Option<String>,
    fields: &Fields,
    obj_name: Option<&str>,
    deref_references: bool
) -> Option<proc_macro2::TokenStream> {
    let count_field = match count {
        None => return None,
        Some(s) => s,
    };

    let mut count_field_name = None;
    for (idx, field) in fields.iter().enumerate() {
        let cur_field = match field.ident {
            Some(ref i) => i.to_string(),
            None => format!("field_{}", idx),
        };
        if cur_field == count_field {
            count_field_name = Some(generate_field_name(field, idx, obj_name, deref_references));
        }
    }

    count_field_name
}

/// Returns whether the string is set to "little"
pub(crate) fn is_lower_endian(val: &str) -> bool {
    if val == "little" {
        true
    } else if val == "big" {
        false
    } else {
        panic!("Unknown endianness : {}", val);
    }
}

/// Generates the proper code to initialize the object
/// e.g :
///     (field_0, field_1, field_2)
/// OR
///     {some_field, timestamp, secret_key}
fn generate_field_list(
    fields: &Fields,
    field_idents: Option<&Vec<proc_macro2::TokenStream>>,
    prefix: Option<&str>,
) -> proc_macro2::TokenStream {
    let mut tmp;

    if let Fields::Unit = fields {
        return quote! {};
    }

    let field_idents = match field_idents {
        Some(f) => f,
        None => {
            tmp = Vec::with_capacity(fields.len());
            for (idx, field) in fields.iter().enumerate() {
                tmp.push(generate_field_name(field, idx, None, false));
            }
            &tmp
        }
    };

    let prefix = match prefix {
        Some(s) => s.parse().unwrap(),
        None => proc_macro2::TokenStream::new(),
    };
    let mut field_list = proc_macro2::TokenStream::new();
    for f in field_idents {
        field_list.extend(quote! {
            #prefix #f,
        });
    }

    match fields {
        Fields::Named(_) => quote! {
            {#field_list}
        },
        Fields::Unnamed(_) => quote! {
            (#field_list)
        },
        _ => unreachable!(),
    }
}
