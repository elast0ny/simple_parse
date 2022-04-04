use std::mem::size_of;
use std::num::*;
use std::sync::atomic::*;

use crate::*;

// For primtive types, we simply deref the checked bytes
// as a pointer to the primitive type.
macro_rules! primitive_read {
    ($typ:ty) => {
        primitive_read!($typ, $typ);
    };
    ($typ:ty, $as_typ: ty) => {
        impl SpRead for $typ {
            const STATIC_CHECKS: () = {
                const _: () = assert!(<$typ>::STATIC_SIZE == size_of::<$typ>());
                const _: () = assert!(size_of::<$typ>() == size_of::<$as_typ>());
            };

            #[inline(always)]
            fn inner_from_reader<R: Read + ?Sized>(
                src: &mut R,
                ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError> {
                default_from_reader::<{ Self::STATIC_SIZE }, Self, R>(src, ctx)
            }

            #[inline(always)]
            unsafe fn inner_from_reader_unchecked<R: Read + ?Sized>(
                checked_bytes: *mut u8,
                _src: &mut R,
                ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError>
            {
                let mut v = *(checked_bytes as *const $as_typ);

                // Swap the endianness if the current machine doesnt match
                // the wanted endianness
                if size_of::<$as_typ>() > 1
                    && ctx.is_little_endian != cfg!(target_endian = "little")
                {
                    v = v.swap_bytes();
                }
                
                #[cfg(feature="verbose")]
                ::log::debug!("  ({})\t{v}", stringify!($typ));
                
                Ok(v as $typ)
            }
        }
    };
}
primitive_read!(u8);
primitive_read!(u16);
primitive_read!(u32);
primitive_read!(u64);
primitive_read!(u128);
primitive_read!(usize);
primitive_read!(i8);
primitive_read!(i16);
primitive_read!(i32);
primitive_read!(i64);
primitive_read!(i128);
primitive_read!(isize);
primitive_read!(f32, u32);
primitive_read!(f64, u64);

// Treat bool as non-zero u8 == true
impl SpRead for bool {
    #[inline(always)]
    fn inner_from_reader<R: Read + ?Sized>(
        src: &mut R,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError> {
        Ok(<u8>::inner_from_reader(src, ctx)? > 0)
    }
    
    #[inline(always)]
    unsafe fn inner_from_reader_unchecked<R: Read + ?Sized>(
        checked_bytes: *mut u8,
        src: &mut R,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError>
    {
        Ok(<u8>::inner_from_reader_unchecked(checked_bytes, src, ctx)? > 0)
    }
}

macro_rules! atomic_read {
    ($typ:ty, $as_typ:ty) => {
        impl SpRead for $typ {
            const STATIC_CHECKS: () = {
                const _: () = assert!(<$typ>::STATIC_SIZE == size_of::<$typ>());
                const _: () = assert!(size_of::<$typ>() == size_of::<$as_typ>());
            };

            #[inline(always)]
            fn inner_from_reader<R: Read + ?Sized>(
                src: &mut R,
                ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError> {
                let v = <$as_typ>::inner_from_reader(src, ctx)?;
                unsafe {
                    Self::inner_from_reader_unchecked(&v as *const _ as _, src, ctx)
                }
            }

            #[inline(always)]
            unsafe fn inner_from_reader_unchecked<R: Read + ?Sized>(
                checked_bytes: *mut u8,
                _src: &mut R,
                _ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError>
            {
                Ok(<$typ>::new(*(checked_bytes as *const $as_typ)))
            }
        }
    };
}
atomic_read!(AtomicU8, u8);
atomic_read!(AtomicU16, u16);
atomic_read!(AtomicU32, u32);
atomic_read!(AtomicU64, u64);
atomic_read!(AtomicUsize, usize);
atomic_read!(AtomicI8, i8);
atomic_read!(AtomicI16, i16);
atomic_read!(AtomicI32, i32);
atomic_read!(AtomicI64, i64);
atomic_read!(AtomicIsize, isize);
atomic_read!(AtomicBool, bool);

macro_rules! nonzero_read {
    ($typ:ty, $as_typ:ty) => {
        impl SpRead for $typ {
            const STATIC_CHECKS: () = {
                const _: () = assert!(<$typ>::STATIC_SIZE == size_of::<$typ>());
                const _: () = assert!(size_of::<$typ>() == size_of::<$as_typ>());
            };

            #[inline(always)]
            fn inner_from_reader<R: Read + ?Sized>(
                src: &mut R,
                ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError> {
                let v = <$as_typ>::inner_from_reader(src, ctx)?;
                unsafe {
                    Self::inner_from_reader_unchecked(&v as *const _ as _, src, ctx)
                }                
            }

            #[inline(always)]
            unsafe fn inner_from_reader_unchecked<R: Read + ?Sized>(
                checked_bytes: *mut u8,
                _src: &mut R,
                _ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError>
            {
                match <$typ>::new(*(checked_bytes as *const $as_typ)) {
                    Some(v) => Ok(v),
                    None => Err(SpError::InvalidBytes),
                }
            }
        }
    };
}
nonzero_read!(NonZeroU8, u8);
nonzero_read!(NonZeroU16, u16);
nonzero_read!(NonZeroU32, u32);
nonzero_read!(NonZeroU64, u64);
nonzero_read!(NonZeroU128, u128);
nonzero_read!(NonZeroUsize, usize);
nonzero_read!(NonZeroI8, i8);
nonzero_read!(NonZeroI16, i16);
nonzero_read!(NonZeroI32, i32);
nonzero_read!(NonZeroI64, i64);
nonzero_read!(NonZeroI128, i128);
nonzero_read!(NonZeroIsize, isize);