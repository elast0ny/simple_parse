mod error;
pub use error::*;

mod default_impls;
pub use default_impls::*;

pub use simple_parse_derive::*;
pub trait SpRead<'b> {
    fn inner_from_bytes(
        input: &'b [u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&'b [u8], Self), crate::SpError>
    where
        Self: 'b + Sized;

    /// Convert arbitrary bytes to Self
    fn from_bytes(input: &'b [u8]) -> Result<(&'b [u8], Self), crate::SpError>
    where
        Self: 'b + Sized;
}

pub trait SpWrite {
    fn inner_to_bytes(
        &mut self,
        is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError>;

    /// Convert the current contents of the struct to bytes.
    /// This function potentially changes the content of self and
    /// can fail.
    fn to_bytes(&mut self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError>;
}

