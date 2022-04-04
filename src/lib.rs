use std::io::{Read, Write};

#[cfg(feature = "verbose")]
pub use log::debug;

mod error;
pub use error::*;

mod default_impls;
pub use default_impls::*;

mod helpers;
pub use helpers::*;

pub use simple_parse_derive::*;

/// A context passed around [SpRead] and [SpWrite] functions
pub struct SpCtx {
    /// How many bytes have been read/written so far
    pub cursor: usize,
    /// This value should only be checked inside custom validators (which get called for both Read & Write)
    /// Its contents are considered invalid otherwise
    pub is_reading: bool,
    /// If the abitrary input/output bytes should be treated as little endian
    pub is_little_endian: bool,
    /// If a dynamically sized Self uses an external `len` field, and what its contents are
    pub len: Option<usize>,
}
impl Default for SpCtx {
    fn default() -> Self {
        Self {
            cursor: 0,
            is_reading: true,
            is_little_endian: cfg!(target_endian = "little"),
            len: None,
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
pub const MAX_ALLOC_SIZE: usize = 4096;

/// Provides optimization hints used by [SpRead] traits.
pub trait SpOptHints {
    /// Whether the type has a variable size
    const IS_VAR_SIZE: bool = true;
    /// How many bytes the `unchecked` parsing functions can assume are valid
    const STATIC_SIZE: usize = 0;
}

/// Parses `Self` from a source implementing [Read](std::io::Read) ([File](std::fs::File),[TcpStream](std::net::TcpStream), etc...)
///
/// This trait is most usefull when the bytes are coming from some kind of IO stream.
/// When possible, it is recommend to use [SpReadRaw] instead for better performance.
pub trait SpRead: Sized + SpOptHints {

    #[doc(hidden)]
    const STATIC_CHECKS: () = ();

    /// Converts bytes from a [Reader](std::io::Read) into `Self`
    ///
    /// This functions allows specifying endianness and `len` fields as opposed to using defaults with `from_reader`
    fn inner_from_reader<R: Read + ?Sized>(
        // Data source
        src: &mut R,
        // Parsing context
        ctx: &mut SpCtx,
        // Data that has already been read from src. Use this first
        existing: &[u8],
    ) -> Result<Self, crate::SpError>;

    /// Converts bytes from a `&mut Read` into `Self`
    fn from_reader<R: Read + ?Sized>(src: &mut R) -> Result<Self, crate::SpError>
    {
        let mut ctx = SpCtx::default();
        let r = Self::inner_from_reader(src, &mut ctx, &[]);
        #[cfg(feature="verbose")]
        ::log::debug!("  Read {} bytes", ctx.cursor);
        r
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
        let mut ctx = SpCtx::default();
        let r = self.inner_to_writer(&mut ctx, dst);
        #[cfg(feature="verbose")]
        ::log::debug!("  Wrote {} bytes", ctx.cursor);
        
        r
    }
}
