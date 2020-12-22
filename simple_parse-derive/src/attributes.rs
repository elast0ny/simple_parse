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

    /// Allows the use of a custom byte reading expression. The expression must have the same
    /// return type as the trait implementation (Result<SomeType, SpError>)
    /// Variables in scope :
    ///     src: Read or Cursor<&[u8]>
    ///     is_input_le: bool   | If input is in little endian
    ///     count: Option<usize>| Value of the count attribute's field
    /// e.g #[sp(reader="custom_reader(src, is_input_le, count)")]
    #[darling(default)]
    pub reader: Option<String>,

    /// Allows the use of a custom byte writing expression. The expression must have the same
    /// return type as the trait implementation (Result<usize, SpError>)
    /// Variables in scope :
    ///     _self: &self        | Reference to the current type
    ///     is_output_le: bool  | If output is little endian
    ///     prepend_count: bool | If you should prepend count
    ///     dst: Write          | The destination of the write
    /// e.g #[sp(reader="custom_writer(_self, is_input_le, count, dst)")]
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