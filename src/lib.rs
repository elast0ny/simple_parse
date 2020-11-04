use std::io::{Read, Write};

mod error;
pub use error::*;

mod default_impls;
pub use default_impls::*;

pub use simple_parse_derive::*;

pub trait SpRead {
    fn inner_from_bytes<R: Read + ?Sized>(
        src: &mut R,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized;

    /// Convert arbitrary bytes to Self
    fn from_bytes<R: Read + ?Sized>(src: &mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized;
}

pub trait SpWrite {
    fn inner_to_bytes<W: Write + ?Sized>(
        &self,
        is_output_le: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError>;

    /// Convert the current contents of the struct to bytes.
    /// This function potentially changes the content of self and
    /// can fail.
    fn to_bytes<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError>;
}
