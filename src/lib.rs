use std::{
    io::{Read, Write},
    mem::MaybeUninit,
};

#[cfg(feature = "verbose")]
pub use log::debug;

mod error;
pub use error::*;

mod default_impls;
pub use default_impls::*;

mod helpers;
pub use helpers::*;

pub use simple_parse_derive::*;

const DEFAULT_IS_LITTLE_ENDIAN: bool = true;

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
            is_little_endian: DEFAULT_IS_LITTLE_ENDIAN,
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
pub const MAX_ALLOC_SIZE: usize = 4 * 1024 * 1024; // 4 MB

/// Parses untrusted bytes from a [Reader](std::io::Read) into a `Self`
///
/// This trait is most usefull when the bytes are coming from some kind of IO stream.
/// When possible, it is recommend to use [SpReadRaw] instead for better performance.
pub trait SpRead: Sized {
    #[doc(hidden)]
    const STATIC_CHECKS: () = ();

    #[doc(hidden)]
    /// Marks types that have the same byte representation on the wire and in memory
    const IS_SAFE_REPR: bool = false;

    /// Converts bytes from a `&mut Read` into `Self`
    fn from_reader<'a, R: Read + ?Sized>(
        src: &mut R,
        dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError> {
        let mut ctx = SpCtx::default();

        let v = Self::inner_from_reader(src, &mut ctx, dst)?;
        #[cfg(feature = "verbose")]
        ::log::debug!("  total : {} bytes", ctx.cursor);

        Ok(v)
    }

    /// Parses bytes from a [Reader](std::io::Read) into `dst` and returns a valid
    fn inner_from_reader<'a, R: Read + ?Sized>(
        src: &mut R,
        ctx: &mut SpCtx,
        dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError>;

    #[doc(hidden)]
    unsafe fn validate_contents<'a>(
        _ctx: &mut SpCtx,
        _dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError> {
        panic!("validate_content internal api should not be used !");
    }
}

/// Writes the binary representation of `Self` into a [Writer](std::io::Write)
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
        #[cfg(feature = "verbose")]
        ::log::debug!("  Wrote {} bytes", ctx.cursor);

        r
    }
}
