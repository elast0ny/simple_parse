use std::io::{Cursor, Read};

use crate::*;

/// The impl macros below expect a macro that generates the body of the unchecked reader.
/// The body macro's arguments are :
///     $typ:ty $(as $as_typ:ty)?   The type you are implementing
///     $reader:ident               The reader function name
///     unchecked_reader:ident      The unchecked reader function name
///     $checked_bytes:ident        Pointer to pre-validaded bytes
///     $src:expr                   The source bytes
///     $is_input_le:expr           Whether the input is lower endian
///     $count:expr                 An already parsed count for dynamic len types
///     $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)?)*
///
/// e.g :
/// macro_rules! mytype_readraw {
///     ($typ:ty, $reader:ident, unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
///         // <Code>
///     }};
/// }

/// Implements SpRead for $typ
#[macro_use]
macro_rules! impl_read {
    // Defaults to using Self::STATIC_SIZE
    ($typ:ty $(as $as_typ:ty)?, $body:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)?)*) => {
        impl_read!(inner, Self::STATIC_SIZE, $typ $(as $as_typ)?, $body $(, $generics $(: $bound $(+ $other)*)?)*);
    };
    // Used for generics because Self::STATIC_SIZE "depends" on the generic parameter thus cannot be used as const...
    // Callers need to provide a size that is not derived from a generic type
    ($typ:ty $(as $as_typ:ty)?, $static_size:expr, $body:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)?)*) => {
        impl_read!(inner, $static_size, $typ $(as $as_typ)?, $body $(, $generics $(: $bound $(+ $other)*)?)*);
    };
    (inner, $static_size:expr, $typ:ty $(as $as_typ:ty)?, $body:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)*)*) => {
        impl<'b, $($generics : SpRead + SpOptHints $(+ $bound$(+ $other)*)*),*> SpRead for $typ {
            fn inner_from_reader<R: Read + ?Sized>(
                src: &mut R,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized + SpOptHints {
                let mut tmp = [0u8; $static_size];

                // Remove count size if count was provided
                let dst = if Self::COUNT_SIZE > 0 && count.is_some() {
                    &mut tmp[..$static_size - Self::COUNT_SIZE]
                } else {
                    &mut tmp
                };

                validate_reader_exact(dst, src)?;
                unsafe {
                    Self::inner_from_reader_unchecked(dst.as_mut_ptr(), src, is_input_le, count)
                }
            }

            unsafe fn inner_from_reader_unchecked<R: Read + ?Sized>(
                checked_bytes: *mut u8,
                src: &mut R,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized + SpOptHints {
                    $body!(
                        $typ $(as $as_typ)?,
                        inner_from_reader,
                        inner_from_reader_unchecked,
                        checked_bytes,
                        src,
                        is_input_le,
                        count $(, $generics $(: $bound $(+ $other)*)*)*
                    )
                }
        }
    };
}
/// Implements SpReadRaw for $typ
#[macro_use]
macro_rules! impl_readraw {
    ($typ:ty $(as $as_typ:ty)?, $body:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)?)*) => {
        impl<'b, $($generics : SpReadRaw<'b> + SpOptHints $(+ $bound$(+ $other)*)?),*> SpReadRaw<'b> for $typ {
            fn inner_from_slice(
                src: &mut Cursor<&'b [u8]>,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized + SpOptHints {
                    let mut static_size = Self::STATIC_SIZE;
                    if Self::COUNT_SIZE > 0 && count.is_some() {
                        static_size -= Self::COUNT_SIZE;
                    }
                    let checked_bytes = validate_cursor(static_size, src)?;
                    unsafe {
                        Self::inner_from_slice_unchecked(checked_bytes, src, is_input_le, count)
                    }
                }
            unsafe fn inner_from_slice_unchecked(
                checked_bytes: *const u8,
                src: &mut Cursor<&'b [u8]>,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized + SpOptHints {
                    $body!(
                        $typ $(as $as_typ)?,
                        inner_from_slice,
                        inner_from_slice_unchecked,
                        checked_bytes,
                        src,
                        is_input_le,
                        count $(, $generics $(: $bound $(+ $other)*)?)*
                    )
                }
        }
    }
}

/// Implements SpReadRawMut for $typ with the provided code generating macro
#[macro_use]
macro_rules! impl_readrawmut {
    ($typ:ty $(as $as_typ:ty)?, $body:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)?)*) => {
        impl<'b, $($generics : SpReadRawMut<'b> + SpOptHints $(+ $bound$(+ $other)*)*),*> SpReadRawMut<'b> for $typ {
            fn inner_from_mut_slice(
                src: &mut Cursor<&'b mut [u8]>,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized + SpOptHints {
                    let mut static_size = Self::STATIC_SIZE;
                    if Self::COUNT_SIZE > 0 && count.is_some() {
                        static_size -= Self::COUNT_SIZE;
                    }

                    let checked_bytes = validate_cursor(static_size, src)?;
                    unsafe {
                        Self::inner_from_mut_slice_unchecked(checked_bytes, src, is_input_le, count)
                    }
                }
            unsafe fn inner_from_mut_slice_unchecked(
                checked_bytes: *mut u8,
                src: &mut Cursor<&'b mut [u8]>,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized + SpOptHints {
                    $body!(
                        $typ $(as $as_typ)?,
                        inner_from_mut_slice,
                        inner_from_mut_slice_unchecked,
                        checked_bytes,
                        src,
                        is_input_le,
                        count $(, $generics $(: $bound $(+ $other)*)?)*
                    )
                }
        }
    }
}

/// Implements SpWrite for $typ
#[macro_use]
macro_rules! impl_writer {
    ($typ:ty $(as $inner_typ:ty)?, $writer:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)?)*) => {
        /// Write Self into writer
        impl<$($generics : SpWrite $(+ $bound$(+ $other)*)?),*> SpWrite for $typ {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                $writer!(self $(as $inner_typ)?, is_output_le, prepend_count, dst $(, $generics $(: $bound $(+ $other)*)?)*)
            }
        }
    }
}
#[macro_use]
macro_rules! impl_writer_all {
    ($typ:ty $(as $inner_typ:ty)?, $writer:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)?)*) => {
        impl_writer!($typ $(as $inner_typ)?, $writer $(, $generics $(: $bound $(+ $other)*)?)*);

        impl<$($generics : SpWrite $(+ $bound $(+ $other)*)?),*> SpWrite for [$typ] {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                iterator_to_writer!(self, is_output_le, prepend_count, dst, 1)
            }
        }
        impl<$($generics : SpWrite $(+ $bound $(+ $other)*)?),*> SpWrite for &[$typ] {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                iterator_to_writer!(self, is_output_le, prepend_count, dst, 1)
            }
        }
        impl<$($generics : SpWrite $(+ $bound$(+ $other)*)*),*> SpWrite for &mut [$typ] {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                iterator_to_writer!(self, is_output_le, prepend_count, dst, 1)
            }
        }
    }
}

/// Reads static_size bytes in chunks into dst
#[inline(always)]
pub fn validate_reader<R: Read + ?Sized>(
    static_size: usize,
    dst: &mut Vec<u8>,
    src: &mut R,
) -> Result<(), SpError> {
    let mut bytes_read = 0;

    while bytes_read < static_size {
        let cur_len = dst.len();
        let cur_chunk_len = std::cmp::min(MAX_ALLOC_SIZE, static_size - bytes_read);

        // Allocate an extra chunk at end of vec
        dst.reserve(cur_chunk_len);

        // Increase len and get slice to new chunk
        unsafe {
            dst.set_len(cur_len + cur_chunk_len);
        }
        let dst_slice = &mut dst.as_mut_slice()[cur_len..];

        // Read chunk into slice
        if let Err(e) = validate_reader_exact(dst_slice, src) {
            // Remove potentially uninit data from dst vec
            unsafe {
                dst.set_len(cur_len);
            }
            return Err(e);
        }

        bytes_read += cur_chunk_len;
    }

    Ok(())
}

/// Consumes dst bytes from the reader
#[inline(always)]
pub fn validate_reader_exact<R: Read + ?Sized>(dst: &mut [u8], src: &mut R) -> Result<(), SpError> {
    #[cfg(feature = "verbose")]
    crate::debug!("Read({})", dst.len());

    // Copy from reader into our stack variable
    if src.read_exact(dst).is_err() {
        return Err(SpError::NotEnoughSpace);
    }

    Ok(())
}

/// Consumes static_size bytes from the cursor returning
/// a raw pointer to the validated bytes
#[inline(always)]
pub fn validate_cursor<T: AsRef<[u8]>>(
    static_size: usize,
    src: &mut Cursor<T>,
) -> Result<*mut u8, crate::SpError> {
    let idx = src.position();
    let bytes = src.get_ref().as_ref();

    #[cfg(feature = "verbose")]
    {
        let len_left = if (bytes.len() as u64) < idx {
            0
        } else {
            (bytes.len() as u64) - idx
        };
        debug!("Check src.len({}) < {}", len_left, static_size);
    }

    // Check length
    if idx + static_size as u64 > bytes.len() as u64 {
        return Err(SpError::NotEnoughSpace);
    }

    // Return pointer to slice
    let checked_bytes = unsafe { bytes.as_ptr().add(idx as usize) as _ };

    // Advance cursor
    src.set_position(idx + static_size as u64);

    Ok(checked_bytes)
}

/// Writes a primitive type into a Writer
#[macro_use]
macro_rules! prim_to_writer {
    ($self:ident as $as_typ:ty, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        let s = unsafe { *($self as *const Self as *const $as_typ) };
        s.inner_to_writer($is_output_le, $prepend_count, $dst)
    }};
    ($self:ident, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        let _ = $prepend_count;

        let value = if $is_output_le {
            $self.to_le_bytes()
        } else {
            $self.to_be_bytes()
        };

        let bytes = value.as_ref();
        match $dst.write(bytes) {
            Ok(v) => Ok(v),
            Err(_) => Err(SpError::NotEnoughSpace),
        }
    }};
}
/// Generates the write code for types that implement `.iter()`
#[macro_use]
macro_rules! iterator_to_writer {
    ($self:ident, $is_output_le:ident, $prepend_count:ident, $dst: ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)?)*) => {{
        let mut total_sz = 0;
        // Write size if needed
        if $prepend_count {
            total_sz += ($self.len() as DefaultCountType).inner_to_writer($is_output_le, false, $dst)?;
        }

        iterator_to_writer!(inner, total_sz, $self, $is_output_le, $dst $(+ $generics)*);

        Ok(total_sz)
    }};
    // Iterator with 1 element
    (inner, $total_sz:ident, $self:ident, $is_output_le:ident, $dst: ident + $generic:tt) => {
        for t1 in $self.iter() {
            $total_sz += t1.inner_to_writer($is_output_le, false, $dst)?;
        }
    };
    // Iterator with 2 elements
    (inner, $total_sz:ident, $self:ident, $is_output_le:ident, $dst: ident + $generic1:tt + $generic2:tt) => {
        for (t1, t2) in $self.iter() {
            $total_sz += t1.inner_to_writer($is_output_le, false, $dst)?;
            $total_sz += t2.inner_to_writer($is_output_le, false, $dst)?;
        }
    };
}
/// Copies a primitive type from a raw pointer
#[macro_use]
macro_rules! prim_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let _ = $count;
        let _ = $src;

        #[cfg(feature = "verbose")]
        crate::debug!(
            "Copy {} : 0x{:X}",
            stringify!($typ),
            std::ptr::read_unaligned($checked_bytes as *mut $as_typ)
        );

        // We assume checked_bytes has been validated to hold at least Self::STATIC_SIZE
        let val: $typ = std::ptr::read_unaligned($checked_bytes as *const $typ);

        Ok(if $is_input_le {
            if cfg!(target_endian = "little") {
                val
            } else {
                #[cfg(feature = "verbose")]
                crate::debug!("swap to native (big) endian");
                val.swap_bytes()
            }
        } else {
            if cfg!(target_endian = "big") {
                val
            } else {
                #[cfg(feature = "verbose")]
                crate::debug!("swap to native (little) endian");
                val.swap_bytes()
            }
        })
    }};
}
/// Makes a mutable reference from a pointer if alignment is satified
#[macro_use]
macro_rules! mutref_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let _ = $count;
        let _ = $src;
        let _ = $is_input_le;

        #[cfg(feature = "verbose")]
        crate::debug!(
            "Copy {} : 0x{:X}",
            stringify!($typ),
            std::ptr::read_unaligned($checked_bytes as *mut $as_typ)
        );

        // Make sure to only make references to properly aligned pointers
        if std::mem::align_of::<$as_typ>() > 1
            && $checked_bytes.align_offset(std::mem::align_of::<$as_typ>()) != 0
        {
            return Err(SpError::BadAlignment);
        }

        // Convert pointer to Rust reference
        Ok(&mut *($checked_bytes as *mut $typ))
    }};
}
/// Copies a bool from a raw pointer
#[macro_use]
macro_rules! bool_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        crate::sa::const_assert!(std::mem::size_of::<$typ>() == std::mem::size_of::<$as_typ>());
        // Read as unsigned & proper endianness
        let val = <$as_typ>::$unchecked_reader($checked_bytes, $src, $is_input_le, $count)?;
        // Convert to bool
        Ok(val != 0)
    }};
}
/// Copies a floating type from a raw pointer
#[macro_use]
macro_rules! float_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        crate::sa::const_assert!(std::mem::size_of::<$typ>() == std::mem::size_of::<$as_typ>());
        // Read as unsigned & proper endianness
        let val = <$as_typ>::$unchecked_reader($checked_bytes, $src, $is_input_le, $count)?;
        // Convert to float
        Ok(*(&val as *const $as_typ as *const $typ))
    }};
}

/// Copies an Atomic from a raw pointer
#[macro_use]
macro_rules! atomic_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        crate::sa::const_assert!(std::mem::size_of::<$typ>() == std::mem::size_of::<$as_typ>());
        // Read as unsigned & proper endianness
        let val = <$as_typ>::$unchecked_reader($checked_bytes, $src, $is_input_le, $count)?;
        // Convert to atomic
        Ok(<$typ>::new(val))
    }};
}

/// Copies a NonZero from a raw pointer
#[macro_use]
macro_rules! nonzero_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        crate::sa::const_assert!(std::mem::size_of::<$typ>() == std::mem::size_of::<$as_typ>());
        // Read as unsigned & proper endianness
        let val = <$as_typ>::$unchecked_reader($checked_bytes, $src, $is_input_le, $count)?;
        // Convert to NonZero
        match <$typ>::new(val) {
            Some(s) => Ok(s),
            None => return Err(SpError::InvalidBytes),
        }
    }};
}

#[macro_use]
macro_rules! new_with_capacity {
    ($num_items:ident) => {
        Self::with_capacity($num_items)
    };
}
#[macro_use]
macro_rules! new_empty {
    ($num_items:ident) => {
        Self::new()
    };
}

// Implementations of SpOptHints
pub mod opt_hints;
pub use opt_hints::*;
// Implementations of SpReadRaw and SpReadRawMut
pub mod raw;
pub use raw::*;
// Implementations of SpRead
pub mod reader;
pub use reader::*;
// Implementations of SpWrite
pub mod writer;
pub use writer::*;
