use std::mem::size_of;
use std::num::*;
use std::sync::atomic::*;

use crate::*;

macro_rules! primitive_read {
    ($typ:ty) => {
        impl SpRead for $typ {
            const IS_SAFE_REPR: bool = true;
            unsafe fn validate_contents<'a>(
                ctx: &mut SpCtx,
                dst: &'a mut MaybeUninit<Self>,
            ) -> Result<&'a mut Self, crate::SpError> {
                // If the data coming in is little endian  but the host isnt, swap
                if size_of::<$typ>() > 1 && ctx.is_little_endian != cfg!(target_endian = "little") {
                    let raw_bytes = core::slice::from_raw_parts_mut(
                        dst.as_mut_ptr() as *mut u8,
                        size_of::<$typ>(),
                    );
                    raw_bytes.reverse();
                }

                let v = dst.assume_init_mut();

                #[cfg(feature = "verbose")]
                ::log::debug!("0x{v:X?}");

                Ok(v)
            }

            fn inner_from_reader<'a, R: Read + ?Sized>(
                // Data source
                src: &mut R,
                // Parsing context
                ctx: &mut SpCtx,
                // Data that has already been read from src. Use this first
                dst: &'a mut MaybeUninit<Self>,
            ) -> Result<&'a mut Self, crate::SpError> {
                static_size_from_reader::<Self, R, { size_of::<$typ>() }>(src, ctx, dst)?;

                let v = unsafe { Self::validate_contents(ctx, dst)? };

                Ok(v)
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
primitive_read!(f32);
primitive_read!(f64);

primitive_read!(AtomicU8);
primitive_read!(AtomicU16);
primitive_read!(AtomicU32);
primitive_read!(AtomicU64);
primitive_read!(AtomicUsize);
primitive_read!(AtomicI8);
primitive_read!(AtomicI16);
primitive_read!(AtomicI32);
primitive_read!(AtomicI64);
primitive_read!(AtomicIsize);

macro_rules! nonzero_read {
    ($typ:ty, $as_typ: ty) => {
        impl SpRead for $typ {
            const STATIC_CHECKS: () = {
                // Make sure we're casting to equivalent types in inner_from_reader
                const _: () = assert!(size_of::<$typ>() == size_of::<$as_typ>());
            };

            const IS_SAFE_REPR: bool = true;
            unsafe fn validate_contents<'a>(
                ctx: &mut SpCtx,
                dst: &'a mut MaybeUninit<Self>,
            ) -> Result<&'a mut Self, crate::SpError> {
                // Make sure the contents are not zero
                let v = dst.as_ptr() as *const $as_typ;
                if *v == 0 {
                    return Err(SpError::InvalidBytes);
                }

                // If the data coming in is little endian  but the host isnt, swap
                if size_of::<$typ>() > 1 && ctx.is_little_endian != cfg!(target_endian = "little") {
                    let raw_bytes = core::slice::from_raw_parts_mut(
                        dst.as_mut_ptr() as *mut u8,
                        size_of::<$typ>(),
                    );
                    raw_bytes.reverse();
                }

                Ok(dst.assume_init_mut())
            }

            fn inner_from_reader<'a, R: Read + ?Sized>(
                // Data source
                src: &mut R,
                // Parsing context
                ctx: &mut SpCtx,
                // Data that has already been read from src. Use this first
                dst: &'a mut MaybeUninit<Self>,
            ) -> Result<&'a mut Self, crate::SpError> {
                static_size_from_reader::<Self, R, { size_of::<$typ>() }>(src, ctx, dst)?;

                let v = unsafe { Self::validate_contents(ctx, dst)? };

                #[cfg(feature = "verbose")]
                ::log::debug!("0x{v:X?}");

                Ok(v)
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

impl SpRead for bool {
    const STATIC_CHECKS: () = {
        // Make sure we can safely cast between the two types
        const _: () = assert!(size_of::<bool>() == size_of::<u8>());
    };

    const IS_SAFE_REPR: bool = true;
    unsafe fn validate_contents<'a>(
        _ctx: &mut SpCtx,
        dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError> {
        // Set the bool to a valid rust representation
        let v = dst.as_ptr() as *const u8;

        dst.write(*v != 0);

        Ok(dst.assume_init_mut())
    }

    fn inner_from_reader<'a, R: Read + ?Sized>(
        // Data source
        src: &mut R,
        // Parsing context
        ctx: &mut SpCtx,
        // Data that has already been read from src. Use this first
        dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError> {
        // Bools dont have a defined internal representation
        // We must first read it as a u8 and then assign false/true to our `dst`

        let u8_dst = unsafe { &mut *(dst as *mut _ as *mut MaybeUninit<u8>) };
        <u8>::inner_from_reader(src, ctx, u8_dst)?;

        let v = unsafe { Self::validate_contents(ctx, dst)? };

        #[cfg(feature = "verbose")]
        ::log::debug!("{v:?}");

        Ok(v)
    }
}

impl SpRead for AtomicBool {
    const STATIC_CHECKS: () = {
        // Make sure we can safely cast between the two types
        const _: () = assert!(size_of::<AtomicBool>() == size_of::<u8>());
    };

    const IS_SAFE_REPR: bool = true;
    unsafe fn validate_contents<'a>(
        _ctx: &mut SpCtx,
        dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError> {
        // Set the bool to a valid rust representation
        let v = dst.as_ptr() as *const u8;

        dst.write(AtomicBool::new(*v != 0));

        Ok(dst.assume_init_mut())
    }

    fn inner_from_reader<'a, R: Read + ?Sized>(
        // Data source
        src: &mut R,
        // Parsing context
        ctx: &mut SpCtx,
        // Data that has already been read from src. Use this first
        dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError> {
        // Bools dont have a defined internal representation
        // We must first read it as a u8 and then assign false/true to our `dst`
        let u8_dst = unsafe { &mut *(dst as *mut _ as *mut MaybeUninit<u8>) };
        <u8>::inner_from_reader(src, ctx, u8_dst)?;

        let v = unsafe { Self::validate_contents(ctx, dst)? };

        #[cfg(feature = "verbose")]
        ::log::debug!("{v:?}");

        Ok(v)
    }
}
