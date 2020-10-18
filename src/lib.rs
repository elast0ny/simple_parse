mod error;
pub use error::*;

mod default_impls;
pub use default_impls::*;

pub use simple_parse_derive::*;

pub trait SpRead<'a> {
    fn inner_from_bytes(
        input: &'a [u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&'a [u8], Self), crate::SpError>
    where
        Self: Sized;

    /// Convert arbitrary bytes to Self
    fn from_bytes(input: &'a [u8]) -> Result<(&'a [u8], Self), crate::SpError>
    where
        Self: Sized;
}

pub trait SpWrite {
    fn inner_to_bytes(
        &self,
        is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError>;

    /// Convert the current contents of the struct to bytes.
    /// This function potentially changes the content of self and
    /// can fail.
    fn to_bytes(&self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError>;
}

