use std::collections::HashMap;

use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DataEnum, DeriveInput, Field, Fields, GenericParam, Generics, Type};

mod attributes;
mod opt_hints;
mod read;
mod write;

pub(crate) use attributes::*;

pub(crate) enum ReaderType {
    Reader,
    Raw,
    RawMut,
}

#[proc_macro_derive(SpOptHints, attributes(sp))]
/// Implements SpOptHints
///
/// For a list of valid `#[sp(X)]` attributes, consult [attributes.rs](https://github.com/elast0ny/simple_parse/tree/master/simple_parse-derive/src/attributes.rs)
pub fn generate_opt_hints(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let res = opt_hints::generate(&mut input);
    proc_macro::TokenStream::from(res)
}

#[proc_macro_derive(SpRead, attributes(sp))]
/// Implements SpRead and SpOptHints
///
/// For a list of valid `#[sp(X)]` attributes, consult [attributes.rs](https://github.com/elast0ny/simple_parse/tree/master/simple_parse-derive/src/attributes.rs)
pub fn generate_read(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let res = read::generate(&mut input, ReaderType::Reader);
    proc_macro::TokenStream::from(res)
}
#[proc_macro_derive(SpReadRaw, attributes(sp))]
/// Implements SpReadRaw
///
/// For a list of valid `#[sp(X)]` attributes, consult [attributes.rs](https://github.com/elast0ny/simple_parse/tree/master/simple_parse-derive/src/attributes.rs)
pub fn generate_readraw(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let res = read::generate(&mut input, ReaderType::Raw);
    proc_macro::TokenStream::from(res)
}
#[proc_macro_derive(SpReadRawMut, attributes(sp))]
/// Implements SpReadRawMut
///
/// For a list of valid `#[sp(X)]` attributes, consult [attributes.rs](https://github.com/elast0ny/simple_parse/tree/master/simple_parse-derive/src/attributes.rs)
pub fn generate_readrawmut(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let res = read::generate(&mut input, ReaderType::RawMut);
    proc_macro::TokenStream::from(res)
}

#[proc_macro_derive(SpWrite, attributes(sp))]
/// Implements SpWrite
///
/// For a list of valid `#[sp(X)]` attributes, consult [attributes.rs](https://github.com/elast0ny/simple_parse/tree/master/simple_parse-derive/src/attributes.rs)
pub fn generate_write(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let res = write::generate(&mut input);
    proc_macro::TokenStream::from(res)
}

/// Adds a bound to a generic parameter
pub(crate) fn add_trait_bounds(generics: &mut Generics, new_bound: syn::TypeParamBound) {
    for param in generics.params.iter_mut() {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(new_bound.clone());
        }
    }
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

    fname.parse().unwrap()
}

/// Converts `count` into a useable field name. Returns None if `count` is None or if the value of `count` cannot be found.
/// eg. self.num_field | *self.num_field | field_01
pub(crate) fn generate_count_field_name(
    count: &Option<String>,
    fields: &Fields,
    obj_name: Option<&str>,
    deref_references: bool,
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
        if count_field == &cur_field {
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

/// Validates an enum variant's IDs and returns the smallest type that can fit the biggest variant id
pub(crate) fn get_enum_id_type(data: &DataEnum, attrs: &EnumAttributes) -> syn::Type {
    let mut seen_ids: HashMap<usize, String> = HashMap::new();
    // Go through every field to find the biggest variant ID
    let mut next_variant_id: usize = 0;
    for variant in data.variants.iter() {
        let var_attrs: darling::Result<VariantAttributes> = FromVariant::from_variant(&variant);
        let variant_name = &variant.ident;
        let variant_id = match var_attrs {
            Ok(v) if v.id.is_some() => {
                let specified_id = v.id.unwrap();
                next_variant_id = specified_id + 1;
                specified_id
            }
            _ => {
                let cur = next_variant_id;
                next_variant_id += 1;
                cur
            }
        };

        if let Some(v) = seen_ids.insert(variant_id, variant_name.to_string()) {
            panic!(
                "Field {} has the same ID as {} : {}",
                variant_name.to_string(),
                v,
                variant_id
            );
        }
    }

    if next_variant_id != 0 {
        next_variant_id -= 1;
    }
    let id_type: syn::Type = syn::parse_str(match attrs.id_type {
        Some(ref s) => s.as_str(),
        None => smallest_type_for_num(next_variant_id),
    })
    .unwrap();

    id_type
}

// Returns the small unsigned integer type for a given usize value
pub(crate) fn smallest_type_for_num(num: usize) -> &'static str {
    if num <= u8::MAX as _ {
        "u8"
    } else if num <= u16::MAX as _ {
        "u16"
    } else if num <= u32::MAX as _ {
        "u32"
    } else {
        "u64"
    }
}

pub(crate) fn get_parse_fn_name(
    reader_type: &ReaderType,
    unchecked: bool,
) -> proc_macro2::TokenStream {
    match (unchecked, reader_type) {
        (true, ReaderType::Reader) => {
            quote! {inner_from_reader_unchecked}
        }
        (true, ReaderType::Raw) => {
            quote! {inner_from_slice_unchecked}
        }
        (true, ReaderType::RawMut) => {
            quote! {inner_from_mut_slice_unchecked}
        }
        (false, ReaderType::Reader) => {
            quote! {inner_from_reader}
        }
        (false, ReaderType::Raw) => {
            quote! {inner_from_slice}
        }
        (false, ReaderType::RawMut) => {
            quote! {inner_from_mut_slice}
        }
    }
}

pub(crate) fn is_var_size(typ: &Type, attrs: Option<&FieldAttributes>) -> bool {
    if let Some(attrs) = attrs {
        // Types that take a count are always variably sized
        if attrs.count.is_some() || attrs.var_size.is_some() {
            return true;
        }
    }

    let field_ty: String =
    match typ {
        syn::Type::Reference(r) => {
            let t = r.elem.as_ref();
            (quote!{&#t}).to_string()
        },
        _ => {
            (quote!{#typ}).to_string()
        },
    };

    // All the types we know are dynamic
    if field_ty.starts_with("& [")
        || field_ty == "& str"
        || field_ty == "String"
        || field_ty == "& CStr"
        || field_ty == "CString"
        || field_ty.starts_with("Option <")
        || field_ty.starts_with("Vec <")
        || field_ty.starts_with("VecDeque <")
        || field_ty.starts_with("LinkedList <")
        || field_ty.starts_with("HashSet <")
        || field_ty.starts_with("BTreeSet <")
        || field_ty.starts_with("HashMap <")
        || field_ty.starts_with("BTreeMap <")
        || field_ty.starts_with("BinaryHeap <")
    {
        return true;
    }

    false
}

/// Returns the static size of a type
/// 
/// This is needed to get around an issue with const generics.
/// When Self is a generic type, it's Self::STATIC_SIZE cannot be evaluated as const so
/// we must use another non-generic type's STATIC_SIZE.
pub(crate) fn get_static_size(typ: &Type) -> proc_macro2::TokenStream {

    let field_ty = quote! {#typ}.to_string();
    
    // Return <bool>::STATIC_SIZE for Option<T>
    if field_ty == "Option <" {
        quote!{
            <bool as ::simple_parse::SpOptHints>::STATIC_SIZE
        }
    // Return <DefaultCountType>::STATIC_SIZE for generic containers
    } else if field_ty.starts_with("Vec <")
        || field_ty.starts_with("VecDeque <")
        || field_ty.starts_with("LinkedList <")
        || field_ty.starts_with("HashSet <")
        || field_ty.starts_with("BTreeSet <")
        || field_ty.starts_with("HashMap <")
        || field_ty.starts_with("BTreeMap <")
        || field_ty.starts_with("BinaryHeap <")
    {
        quote!{
            <::simple_parse::DefaultCountType as ::simple_parse::SpOptHints>::STATIC_SIZE
        }
    } else if field_ty.contains('<') {
        panic!("Generic type '{}' cannot be used as a field because Rust currently does not handle const generics properly (Required for SpOptHints::STATIC_SIZE)");
    } else {
        quote!{
            <#typ as ::simple_parse::SpOptHints>::STATIC_SIZE
        }
    }
}
