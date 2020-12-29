
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
                ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized + SpOptHints {
                let mut tmp = [0u8; $static_size];

                // Remove count size if count was provided
                let dst = if Self::COUNT_SIZE > 0 && ctx.count.is_some() {
                    &mut tmp[..$static_size - Self::COUNT_SIZE]
                } else {
                    &mut tmp
                };

                validate_reader_exact(dst, src)?;
                unsafe {
                    Self::inner_from_reader_unchecked(dst.as_mut_ptr(), src, ctx)
                }
            }

            unsafe fn inner_from_reader_unchecked<R: Read + ?Sized>(
                checked_bytes: *mut u8,
                src: &mut R,
                ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized + SpOptHints {
                    $body!(
                        $typ $(as $as_typ)?,
                        inner_from_reader,
                        inner_from_reader_unchecked,
                        checked_bytes,
                        src,
                        ctx $(, $generics $(: $bound $(+ $other)*)*)*
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
                ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized + SpOptHints {
                    let mut static_size = Self::STATIC_SIZE;
                    if Self::COUNT_SIZE > 0 && ctx.count.is_some() {
                        static_size -= Self::COUNT_SIZE;
                    }
                    let checked_bytes = validate_cursor(static_size, src)?;
                    unsafe {
                        Self::inner_from_slice_unchecked(checked_bytes, src, ctx)
                    }
                }
            unsafe fn inner_from_slice_unchecked(
                checked_bytes: *const u8,
                src: &mut Cursor<&'b [u8]>,
                ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized + SpOptHints {
                    $body!(
                        $typ $(as $as_typ)?,
                        inner_from_slice,
                        inner_from_slice_unchecked,
                        checked_bytes,
                        src,
                        ctx $(, $generics $(: $bound $(+ $other)*)?)*
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
                ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized + SpOptHints {
                    let mut static_size = Self::STATIC_SIZE;
                    if Self::COUNT_SIZE > 0 && ctx.count.is_some() {
                        static_size -= Self::COUNT_SIZE;
                    }

                    let checked_bytes = validate_cursor(static_size, src)?;
                    unsafe {
                        Self::inner_from_mut_slice_unchecked(checked_bytes, src, ctx)
                    }
                }
            unsafe fn inner_from_mut_slice_unchecked(
                checked_bytes: *mut u8,
                src: &mut Cursor<&'b mut [u8]>,
                ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized + SpOptHints {
                    $body!(
                        $typ $(as $as_typ)?,
                        inner_from_mut_slice,
                        inner_from_mut_slice_unchecked,
                        checked_bytes,
                        src,
                        ctx $(, $generics $(: $bound $(+ $other)*)?)*
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
                ctx: &mut SpCtx,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                $writer!(self $(as $inner_typ)?, ctx, dst $(, $generics $(: $bound $(+ $other)*)?)*)
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
                ctx: &mut SpCtx,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                iterator_to_writer!(self, ctx, dst, 1)
            }
        }
        impl<$($generics : SpWrite $(+ $bound $(+ $other)*)?),*> SpWrite for &[$typ] {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                ctx: &mut SpCtx,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                iterator_to_writer!(self, ctx, dst, 1)
            }
        }
        impl<$($generics : SpWrite $(+ $bound$(+ $other)*)*),*> SpWrite for &mut [$typ] {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                ctx: &mut SpCtx,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                iterator_to_writer!(self, ctx, dst, 1)
            }
        }
    }
}

/// Writes a primitive type into a Writer
#[macro_use]
macro_rules! prim_to_writer {
    ($self:ident as $as_typ:ty, $ctx:ident, $dst: ident) => {{
        let s = unsafe { *($self as *const Self as *const $as_typ) };
        s.inner_to_writer($ctx, $dst)
    }};
    ($self:ident, $ctx:ident, $dst: ident) => {{
        let value = if $ctx.is_little_endian {
            $self.to_le_bytes()
        } else {
            $self.to_be_bytes()
        };

        let bytes = value.as_ref();
        match $dst.write(bytes) {
            Ok(v) => {
                $ctx.cursor += v;
                Ok(v)
            }
            Err(_) => Err(SpError::NotEnoughSpace),
        }
    }};
}
/// Generates the write code for types that implement `.iter()`
#[macro_use]
macro_rules! iterator_to_writer {
    ($self:ident, $ctx:ident, $dst: ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)?)*) => {{
        let mut total_sz = 0;
        // Write size if needed
        if $ctx.count.is_none() {
            total_sz += ($self.len() as DefaultCountType).inner_to_writer($ctx, $dst)?;
        }

        // Dont propagate count field to inner types
        $ctx.count = None;
        iterator_to_writer!(inner, total_sz, $self, $ctx, $dst $(+ $generics)*);

        $ctx.cursor += total_sz;
        Ok(total_sz)
    }};
    // Iterator with 1 element
    (inner, $total_sz:ident, $self:ident, $ctx:ident, $dst: ident + $generic:tt) => {
        for t1 in $self.iter() {
            $total_sz += t1.inner_to_writer($ctx, $dst)?;
        }
    };
    // Iterator with 2 elements
    (inner, $total_sz:ident, $self:ident, $ctx:ident, $dst: ident + $generic1:tt + $generic2:tt) => {
        for (t1, t2) in $self.iter() {
            $total_sz += t1.inner_to_writer($ctx, $dst)?;
            $total_sz += t2.inner_to_writer($ctx, $dst)?;
        }
    };
}
/// Copies a primitive type from a raw pointer
#[macro_use]
macro_rules! prim_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{
        let _ = $src;
        // Make sure we never accidentaly cast between invalid types
        crate::sa::const_assert_eq!(std::mem::size_of::<$typ>(), std::mem::size_of::<$as_typ>());

        #[cfg(feature = "verbose")]
        crate::debug!(
            "[0x{:X}] : 0x{:0width$X} [{}]",
            $ctx.cursor,
            std::ptr::read_unaligned($checked_bytes as *mut $as_typ),
            stringify!($typ),
            width = std::mem::size_of::<$as_typ>() * 2,
        );

        let val: $typ = std::ptr::read_unaligned($checked_bytes as *const $typ);

        $ctx.cursor += std::mem::size_of::<$typ>();

        Ok(if $ctx.is_little_endian {
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
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{
        let _ = $src;
        // Make sure we never accidentaly cast between invalid types
        crate::sa::const_assert_eq!(
            std::mem::align_of::<$typ>(),
            std::mem::align_of::<&$as_typ>()
        );

        #[cfg(feature = "verbose")]
        crate::debug!(
            "[0x{:X}] : *{:p} = 0x{:0width$X} [{}]",
            $ctx.cursor,
            $checked_bytes,
            std::ptr::read_unaligned($checked_bytes as *mut $as_typ),
            stringify!($typ),
            width = std::mem::size_of::<$as_typ>() * 2
        );

        // Make sure to only make references to properly aligned pointers
        if std::mem::align_of::<$as_typ>() > 1
            && $checked_bytes.align_offset(std::mem::align_of::<$as_typ>()) != 0
        {
            return Err(SpError::BadAlignment);
        }

        $ctx.cursor += std::mem::size_of::<$as_typ>();

        // Convert pointer to Rust reference
        let r: $typ = &mut *($checked_bytes as *mut $as_typ as *mut _);
        Ok(r)
    }};
}
/// Copies a bool from a raw pointer
#[macro_use]
macro_rules! bool_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{
        // Make sure we never accidentaly cast between invalid types
        crate::sa::const_assert!(std::mem::size_of::<$typ>() == std::mem::size_of::<$as_typ>());
        // Read as unsigned & proper endianness
        let val = <$as_typ>::$unchecked_reader($checked_bytes, $src, $ctx)?;
        // Convert to bool
        Ok(val != 0)
    }};
}
/// Copies a floating type from a raw pointer
#[macro_use]
macro_rules! float_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{
        // Make sure we never accidentaly cast between invalid types
        crate::sa::const_assert!(std::mem::size_of::<$typ>() == std::mem::size_of::<$as_typ>());
        // Read as unsigned & proper endianness
        let val = <$as_typ>::$unchecked_reader($checked_bytes, $src, $ctx)?;
        // Convert to float
        Ok(*(&val as *const $as_typ as *const $typ))
    }};
}

/// Copies an Atomic from a raw pointer
#[macro_use]
macro_rules! atomic_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{
        // Make sure we never accidentaly cast between invalid types
        crate::sa::const_assert!(std::mem::size_of::<$typ>() == std::mem::size_of::<$as_typ>());
        // Read as unsigned & proper endianness
        let val = <$as_typ>::$unchecked_reader($checked_bytes, $src, $ctx)?;
        // Convert to atomic
        Ok(<$typ>::new(val))
    }};
}

/// Copies a NonZero from a raw pointer
#[macro_use]
macro_rules! nonzero_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{
        // Make sure we never accidentaly cast between invalid types
        crate::sa::const_assert!(std::mem::size_of::<$typ>() == std::mem::size_of::<$as_typ>());
        // Read as unsigned & proper endianness
        let val = <$as_typ>::$unchecked_reader($checked_bytes, $src, $ctx)?;
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
mod opt_hints;
pub use opt_hints::*;
// Implementations of SpReadRaw and SpReadRawMut
mod raw;
pub use raw::*;
// Implementations of SpRead
mod reader;
pub use reader::*;
// Implementations of SpWrite
mod writer;
pub use writer::*;
