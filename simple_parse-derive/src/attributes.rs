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

    /// Allows for custom validation code to run directly after parsing or before writing a field.
    /// The provided value will be parsed as a comma seperated function name optionnaly followed by field names.
    /// When reading, any reference to fields __after__ the current field will be passed as `None` as their contents have not yet been parsed.
    /// When writing, any reference to fields __after__ the current field will be passed as `Some(&T)`.
    /// 
    /// This functions first parameter will always be a reference to the field it annotates.
    /// For example :
    /// ```Rust
    ///     struct MyStruct {
    ///         some_field: bool,
    ///         #[sp(validate="validate_magic, some_field")]
    ///         magic_value: u32    
    ///         field_after: String,
    ///     }
    /// ```
    /// The `validate_magic` function must have a signature :
    ///     fn validate_magic(this: &u32, some_field: &bool, field_after: Option<&String>, ctx: &mut SpCtx) -> Result<(), SpError>
    ///
    ///     When called as part of a Read :
    ///         ctx.is_reading = true;
    ///         validate_magic(&magic_value, &some_field, None, ctx)?;
    ///     When called as part of a Write :
    ///         ctx.is_reading = false;
    ///         validate_magic(&self.magic_value, &self.some_field, Some(self.field_after), ctx)?;
    #[darling(default)]
    pub validate: Option<String>,

    /// Allows for custom parsing functions to populate the annotated field.
    /// The provided String will be parsed as a comma seperated function name followed by field names that have already been populated.
    /// For example :
    /// ```Rust
    ///     struct MyStruct {
    ///         some_field: bool,
    ///         #[sp(reader="custom_parser, some_field")]
    ///         optionnal: Option<usize>,
    ///     }
    /// ```
    /// Will end up generating code that calls a function with signature :
    ///     fn custom_parser(some_field: &bool, src: &mut Read, ctx: &mut SpCtx) -> Result<T, SpError>
    #[darling(default)]
    pub reader: Option<String>,
    
    /// Allows for custom writing functions to convert the annotated field into bytes.
    /// The provided String will be parsed as a comma seperated function name optionally followed by any field name in the struct.
    /// This function's first parameter will always be a reference to the field it annotates.
    /// For example :
    /// ```Rust
    ///     struct MyStruct {
    ///         some_field: bool,
    ///         #[sp(writer="custom_writer")]
    ///         optionnal: Option<usize>,
    ///     }
    /// ```
    /// Will end up generating code that calls a function with signature :
    ///     fn custom_writer(this: &Option<usize>, ctx: &mut SpCtx, dst: &mut Write) -> Result<usize, SpError>
    #[darling(default)]
    pub writer: Option<String>,

    /// Specifies the endiannes of the specific field. The data will
    /// be converted to the native endianness when necessary.
    #[darling(default)]
    pub endian: Option<String>,

    /// Specifies whether this field's type is variably sized
    /// 
    /// This should only be required when a custom type has a variable size.
    #[darling(default)]
    pub var_size: Option<()>
}