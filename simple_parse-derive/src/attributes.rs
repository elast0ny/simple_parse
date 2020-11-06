use crate::*;

/// 
/// This file contains a list of valid dervie attributes that simple_parse supports
/// 

/* Enums */

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(sp))]
/// Attributes that can be use on the top level enum declaration
pub(crate) struct EnumAttributes {
    /// The type used to parse the variant id
    #[darling(default)]
    pub id_type: Option<String>,

    /// Specifies the default  endiannesss for the whole enum.
    #[darling(default)]
    pub endian: Option<String>,
}

/// Attributes that can be use on each enum variant.
#[derive(Debug, FromVariant)]
#[darling(attributes(sp))]
pub(crate) struct VariantAttributes {
    /// The numerical representation of this variant.
    pub id: usize,
}

/* Structs */

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(sp))]
/// Attributes that can be use on the top level struct declaration
pub(crate) struct StructAttributes {
    /// Specifies the default endiannesss for the whole struct
    #[darling(default)]
    pub endian: Option<String>,
}

#[derive(Debug, FromField)]
#[darling(attributes(sp))]
/// Attributes that can be use on each field.
pub(crate) struct FieldAttributes {
    /// Points to the field name/index that contains the number of items to parse the dynamically size type
    /// e.g.
    /// ```Rust
    /// struct Test {
    ///     num_options: u8,
    ///     #[sp(count="num_options")]
    ///     options: Vec<Options>,
    /// }
    /// ```
    #[darling(default)]
    pub count: Option<String>,

    /// Allows the use of a custom byte reading function. This function must have the same
    /// return type as SpRead::inner_from_reader
    /// Variables in scope :
    ///     input : The input bytes
    ///     is_input_le : If input is in little endian
    ///     count : An Option<number> that contains the number of items to parse
    /// e.g #[sp(reader="custom_reader(input, is_input_le, count)")]
    #[darling(default)]
    pub reader: Option<String>,

    /// Allows the use of a custom byte writing function.
    #[darling(default)]
    pub writer: Option<String>,

    /// Specifies the endiannes of the specific field. The data will
    /// be converted to the native endianness when necessary.
    #[darling(default)]
    pub endian: Option<String>,
}