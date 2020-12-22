use std::io::{Cursor, Read, Write};

#[cfg(feature = "verbose")]
pub use log::debug;

pub use static_assertions as sa;

mod error;
pub use error::*;

mod default_impls;
pub use default_impls::*;

pub use simple_parse_derive::*;

/// This type MUST be included in dynamic types STATIC_SIZE.
/// When #[sp(count)] is provided, <DefaultCountType>::STATIC_SIZE is substracted from your type's STATIS_SIZE.
pub type DefaultCountType = u32;

/// This is a safeguard against reading malicious/malformed dynamically sized types.
/// For example, when reading a String that says it contains INT_MAX characters, chunks of
/// MAX_ALLOC_SIZE will be read at a time instead of allocating INT_MAX bytes in one go.
pub const MAX_ALLOC_SIZE: usize = 1024;

/// Provides optimization hints used by SpRead* traits.
///
/// # Safety
/// When not using defaults, implementors must be very careful not to return invalid values
/// as it may lead to buffer over reads (e.g. setting `static_size` to 4 and reading 8 bytes in the `unchecked` readers)
pub unsafe trait SpOptHints {
    /// Whether the type has a variable size
    const IS_VAR_SIZE: bool = true;
    /// How many bytes the `unchecked` parsing functions can assume are valid
    const STATIC_SIZE: usize = 0;
    /// Used to substract from STATIC_SIZE when a count is provided
    const COUNT_SIZE: usize = 0;
}

/// Parses T from a reader (File,TcpStream, etc...)
///
/// This trait is most usefull when the bytes are coming from some kind of IO stream.
/// When possible, it is recommend to use SpReadRaw\[Mut\] instead for better performance.
pub trait SpRead {
    /// Parses bytes from a `&mut Read` into Self with control over endianness and number of items (For dynamically sized types)
    fn inner_from_reader<R: Read + ?Sized>(
        src: &mut R,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints;

    /// Unchecked parser from a Reader
    ///
    /// # Safety
    /// This function assumes that checked_bytes points to at least Self::STATIC_SIZE bytes.
    ///
    /// If this is implemented on a dynamic type, the implementors MUST check if count is provided.
    /// If it is provided, Self::COUNT_SIZE less bytes can be trusted from checked_bytes.
    unsafe fn inner_from_reader_unchecked<R: Read + ?Sized>(
        checked_bytes: *mut u8,
        src: &mut R,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints;

    /// Parses bytes from a `&mut Read` into Self
    fn from_reader<R: Read + ?Sized>(src: &mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints,
    {
        Self::inner_from_reader(
            src,
            // Assume most of the time, bytes will be in the current host's endianness
            cfg!(target_endian = "little"),
            None,
        )
    }
}

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
        Self: Sized + SpOptHints;

    /// Unchecked parser from a &[u8]
    ///
    /// # Safety
    /// This function assumes that checked_bytes points to at least Self::STATIC_SIZE bytes.
    ///
    /// If this is implemented on a dynamic type, the implementors MUST check if count is provided.
    /// If it is provided, Self::COUNT_SIZE less bytes can be trusted from checked_bytes.
    unsafe fn inner_from_slice_unchecked(
        checked_bytes: *const u8,
        src: &mut Cursor<&'b [u8]>,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints;

    /// Parses bytes from a `&[u8]` into Self
    fn from_slice(src: &mut Cursor<&'b [u8]>) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints,
    {
        Self::inner_from_slice(src, cfg!(target_endian = "little"), None)
    }
}

/// Parses T, &T or &mut T from a mutable byte slice
///
/// # Warning
/// See [SpReadRaw](trait.SpReadRaw.html) warning.
pub trait SpReadRawMut<'b> {
    /// Parses bytes from a `&mut [u8]` into Self with control over endianness and number of items (For dynamically sized types)
    fn inner_from_mut_slice(
        src: &mut Cursor<&'b mut [u8]>,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints;

    /// Unchecked parser from a &mut[u8]
    ///
    /// # Safety
    /// This function assumes that checked_bytes points to at least Self::STATIC_SIZE bytes.
    ///
    /// If this is implemented on a dynamic type, the implementors MUST check if count is provided.
    /// If it is provided, Self::COUNT_SIZE less bytes can be trusted from checked_bytes.
    unsafe fn inner_from_mut_slice_unchecked(
        checked_bytes: *mut u8,
        src: &mut Cursor<&'b mut [u8]>,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints;

    /// Parses bytes from a `&mut [u8]` into Self
    fn from_mut_slice(src: &mut Cursor<&'b mut [u8]>) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints,
    {
        Self::inner_from_mut_slice(src, cfg!(target_endian = "little"), None)
    }
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
    fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_writer(cfg!(target_endian = "little"), false, dst)
    }
}
