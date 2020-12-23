use std::io::{Cursor, Read, Write};

#[cfg(feature = "verbose")]
pub use log::debug;

#[doc(hidden)]
/// Allows for compile time assertions in the generated derive code
pub use static_assertions as sa;

mod error;
pub use error::*;

mod default_impls;
pub use default_impls::*;

pub use simple_parse_derive::*;

/// A context passed around [SpRead] and [SpWrite] functions
pub struct SpCtx {
    /// How many bytes have been read/written so far
    pub cursor: usize,
    /// If the abitrary input/output bytes should be treated as little endian
    pub is_little_endian: bool,
    /// If a dynamically sized Self uses an external count field, and what its contents are
    pub count: Option<usize>,
}
impl Default for SpCtx {
    fn default() -> Self {
        Self{
            cursor: 0,
            is_little_endian: cfg!(target_endian="little"),
            count: None,
        }
    }
}

#[doc(hidden)]
/// This type is used for dynamically sized type's default implementations
pub type DefaultCountType = u32;

#[doc(hidden)]
/// This is a safeguard against reading malicious/malformed dynamically sized types.
/// For example, when reading a String that says it contains INT_MAX characters, chunks of
/// MAX_ALLOC_SIZE will be read at a time instead of allocating INT_MAX bytes in one go.
pub const MAX_ALLOC_SIZE: usize = 1024;

/// Provides optimization hints used by [SpRead] traits.
///
/// # Safety
/// When not using defaults, implementors must be very careful not to return invalid values
/// as it may lead to buffer over reads (e.g. setting `STATIC_SIZE` to 4 and reading 8 bytes in the `unchecked` readers)
pub unsafe trait SpOptHints {
    /// Whether the type has a variable size
    const IS_VAR_SIZE: bool = true;
    /// How many bytes the `unchecked` parsing functions can assume are valid
    const STATIC_SIZE: usize = 0;
    /// Used to substract from STATIC_SIZE when a count is provided
    const COUNT_SIZE: usize = 0;
}

/// Parses `Self` from a source implementing [Read](std::io::Read) ([File](std::fs::File),[TcpStream](std::net::TcpStream), etc...)
///
/// This trait is most usefull when the bytes are coming from some kind of IO stream.
/// When possible, it is recommend to use [SpReadRaw] instead for better performance.
pub trait SpRead {
    /// Converts bytes from a [Reader](std::io::Read) into `Self`
    /// 
    /// This functions allows specifying endianness and count fields as opposed to using defaults with `from_reader`
    fn inner_from_reader<R: Read + ?Sized>(
        src: &mut R,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints;

    /// Converts bytes from a `&mut Read` into `Self` with some assumptions on checked_bytes
    ///
    /// # Safety
    /// This function assumes that checked_bytes points to at least Self::STATIC_SIZE bytes.
    ///
    /// If this is implemented on a dynamic type, the implementors MUST check if count is provided.
    /// If it is provided, Self::COUNT_SIZE less bytes can be trusted from checked_bytes.
    unsafe fn inner_from_reader_unchecked<R: Read + ?Sized>(
        checked_bytes: *mut u8,
        src: &mut R,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints;

    /// Converts bytes from a `&mut Read` into `Self`
    fn from_reader<R: Read + ?Sized>(src: &mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints,
    {
        Self::inner_from_reader(
            src,
            &mut SpCtx::default(),
        )
    }
}

/// Parses `Self` from a [Cursor<&\[u8\]>](std::io::Cursor)
pub trait SpReadRaw<'b> {
    /// Converts bytes from a `Cursor<&[u8]>` into `Self`
    /// 
    /// This functions allows specifying endianness and count fields as opposed to using defaults with `from_slice`
    fn inner_from_slice(
        src: &mut Cursor<&'b [u8]>,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints;

    /// Converts bytes from a `Cursor<&[u8]>` into `Self` with some assumptions on checked_bytes
    ///
    /// # Safety
    /// This function assumes that checked_bytes points to at least Self::STATIC_SIZE bytes.
    ///
    /// If this is implemented on a dynamic type, the implementors MUST check if count is provided.
    /// If it is provided, Self::COUNT_SIZE less bytes can be trusted from checked_bytes.
    /// 
    /// This function also allows returning references into the `Cursor<&[u8]>` when `Self` is a reference `&T`.
    /// This should not be done if `Self` itself contains non-primitive types, references, slices, etc...
    unsafe fn inner_from_slice_unchecked(
        checked_bytes: *const u8,
        src: &mut Cursor<&'b [u8]>,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints;

    /// Converts bytes from a `Cursor<&[u8]>` into `Self`
    fn from_slice(src: &mut Cursor<&'b [u8]>) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints,
    {
        Self::inner_from_slice(src, &mut SpCtx::default())
    }
}

/// Parses `Self` from a [Cursor<&mut \[u8\]>](std::io::Cursor)
pub trait SpReadRawMut<'b> {
    /// Converts bytes from a `Cursor<&mut [u8]>` into `Self`
    /// 
    /// This functions allows specifying endianness and count fields as opposed to using defaults with `from_slice`
    fn inner_from_mut_slice(
        src: &mut Cursor<&'b mut [u8]>,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints;

    /// Converts bytes from a `Cursor<&mut [u8]>` into `Self` with some assumptions on checked_bytes
    ///
    /// # Safety
    /// This function assumes that checked_bytes points to at least Self::STATIC_SIZE bytes.
    ///
    /// If this is implemented on a dynamic type, the implementors MUST check if count is provided.
    /// If it is provided, Self::COUNT_SIZE less bytes can be trusted from checked_bytes.
    /// 
    /// This function also allows returning references into the `Cursor<&[u8]>` when `Self` is a reference `&T` or `&mut T`.
    /// This should not be done if `Self` itself contains non-primitive types, references, slices, etc...
    unsafe fn inner_from_mut_slice_unchecked(
        checked_bytes: *mut u8,
        src: &mut Cursor<&'b mut [u8]>,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints;

    /// Converts bytes from a `Cursor<&mut [u8]>` into `Self`
    fn from_mut_slice(src: &mut Cursor<&'b mut [u8]>) -> Result<Self, crate::SpError>
    where
        Self: Sized + SpOptHints,
    {
        Self::inner_from_mut_slice(src, &mut SpCtx::default())
    }
}

/// Writes `T` into a [Writer](std::io::Write) ([File](std::fs::File),[TcpStream](std::net::TcpStream), etc...)
pub trait SpWrite {
    /// Writes the byte representation for Self into a `&mut Write` with control over endianness
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        ctx: &mut SpCtx,
        dst: &mut W,
    ) -> Result<usize, crate::SpError>;

    /// Writes the byte representation for Self into a `&mut Write`
    fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_writer(&mut SpCtx::default(), dst)
    }
}
