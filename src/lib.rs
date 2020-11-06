use std::io::{Read, Write, Cursor};

mod error;
pub use error::*;

mod default_impls;
pub use default_impls::*;

pub use simple_parse_derive::*;

/// Parses T or &T from a byte slice
/// 
/// # Warning
/// ## Implementing for &T 
/// This trait allows implementors to return references into `src`.
/// This is only valid for types that do not contain any inner references themselves.
/// This should be done very carefully as it can quickly lead to unexpected issues.
/// For example :
/// ```rust
/// struct Example {
///     str_ref: &str,
/// }
/// 
/// let orig: [u8] = [
///     2,0,0,0,0,0,0,0,    // Length
///     b'A', b'Z'          // Data
/// ];
/// let mut cur = Cursor::new(&orig[..]);
/// 
/// // This is OK as the `from_slice` impl will be called for &str
/// // and will construct a proper fat pointer (&str)
///     <&str>::from_slice(&mut cur)
/// // OR
///     <Example>::from_slice(&mut cur)
/// 
/// // This is __not__ OK as the `from_slice` impl will only be called for `&Example`
/// // but will not be called when someone accesses `obj.str_ref`
/// // Instead, `obj.str_ref` will simply contain whatever is in the slice 
/// // at offset 0 which in that case isnt a pointer (0x0000_0000_0000_0002).
///     <&Example>::from_slice(&mut cur);
/// ```
pub trait SpReadRaw<'b> {
    /// Parses bytes from a `&[u8]` into Self with control over endianness and number of items (For dynamically sized types)
    fn inner_from_slice(
        src: &mut Cursor<&'b [u8]>,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized;

    /// Parses bytes from a `&[u8]` into Self
    fn from_slice(src: &mut Cursor<&'b [u8]>) -> Result<Self, crate::SpError>
    where
        Self: Sized;
}

/// Parses T, &T or &mut T from a mutable byte slice
/// 
/// # Warning
/// See [SpReadRaw](trait.SpReadRaw.html) warning.
pub trait SpReadRawMut<'b> {
    /// Parses bytes from a `&mut [u8]` into Self with control over endianness and number of items (For dynamically sized types)
    fn inner_from_mut_slice(
        src: &mut Cursor<&'b mut [u8]>,
        _is_input_le: bool,
        _count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized;

    /// Parses bytes from a `&mut [u8]` into Self
    fn from_mut_slice(src: &mut Cursor<&'b mut [u8]>) -> Result<Self, crate::SpError>
    where
        Self: Sized;
}

/// Parses T from a reader (File,TcpStream, etc...)
///
/// This trait is most usefull when the bytes are coming from some kind of IO stream.
/// When possible, it is recommend to use SpReadRaw[Mut] instead for better performance.
pub trait SpRead {
    /// Parses bytes from a `&mut Read` into Self with control over endianness and number of items (For dynamically sized types)
    fn inner_from_reader<R: Read + ?Sized>(
        src: &mut R,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized;

    /// Parses bytes from a `&mut Read` into Self
    fn from_reader<R: Read + ?Sized>(src: &mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized;
}

/// Writes T to the writer (File, TcpStream, Vec<u8>, etc...)
pub trait SpWrite {
    /// Writes the byte representation for Self into a `&mut Write` with control over endianness
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        is_output_le: bool,
        prepend_count: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError>;

    /// Writes the byte representation for Self into a `&mut Write`
    fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError>;
}
