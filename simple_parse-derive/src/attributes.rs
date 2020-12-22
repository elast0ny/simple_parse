use crate::*;

/// 
/// This file contains a list of valid derive attributes that simple_parse supports
/// 

/* Enums */

#[derive(Default, Debug, PartialEq)]
#[derive(FromDeriveInput)]
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
#[derive(Default, Debug, PartialEq)]
#[derive(FromVariant)]
#[darling(attributes(sp))]
pub(crate) struct VariantAttributes {
    /// The numerical representation of this variant.
    /// When not specified, C style ids are used (First variant starts at 0, subsequent are [prev + 1])
    #[darling(default)]
    pub id: Option<usize>,

    /// Specifies the default endiannesss for the whole Variant
    #[darling(default)]
    pub endian: Option<String>,
}

/* Structs */

#[derive(Default, Debug, PartialEq)]
#[derive(FromDeriveInput)]
#[darling(attributes(sp))]
/// Attributes that can be use on the top level struct declaration
pub(crate) struct StructAttributes {
    /// Specifies the default endiannesss for the whole struct
    #[darling(default)]
    pub endian: Option<String>,
}

#[derive(Default, Debug, PartialEq)]
#[derive(FromField)]
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

    // Not implemented yet
    // TODO
    //      Figure out how to allow end users to provide either a block of code
    //      or a function name that will be called instead of the default implementation 
    #[darling(default)]
    pub reader: Option<String>,
    #[darling(default)]
    pub writer: Option<String>,

    /// Specifies the endiannes of the specific field. The data will
    /// be converted to the native endianness when necessary.
    #[darling(default)]
    pub endian: Option<String>,

    /// Specifies whether this field's type is variably sized
    #[darling(default)]
    pub var_size: Option<()>
}