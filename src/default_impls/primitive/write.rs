use core::mem::size_of;
use std::{sync::atomic::*, num::*};

use crate::{SpWrite, SpError};

macro_rules! primitive_write {
    ($typ:ty) => {
        primitive_write!($typ, $typ);
    };
    ($typ:ty, $as_typ: ty) => {
        impl SpWrite for $typ {
            fn inner_to_writer<W: std::io::Write + ?Sized>(
                &self,
                ctx: &mut crate::SpCtx,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                let mut v = unsafe{*(self as *const _ as *const $as_typ)};
                let v_ptr = &mut v as *mut _ as *mut u8;
        
                let buf = unsafe {
                    core::slice::from_raw_parts_mut(v_ptr, size_of::<$as_typ>())
                };

                if ctx.is_little_endian != cfg!(target_endian="little") {
                    buf.reverse();
                }
        
                if dst.write_all(buf).is_err() {
                    return Err(SpError::NotEnoughSpace);
                }
                
                ctx.cursor += size_of::<$as_typ>();
                Ok(size_of::<$as_typ>())        
            }
        }
    };
}

primitive_write!(u8);
primitive_write!(u16);
primitive_write!(u32);
primitive_write!(u64);
primitive_write!(u128);
primitive_write!(usize);
primitive_write!(i8);
primitive_write!(i16);
primitive_write!(i32);
primitive_write!(i64);
primitive_write!(i128);
primitive_write!(isize);
primitive_write!(f32, u32);
primitive_write!(f64, u64);

impl SpWrite for bool {
    fn inner_to_writer<W: std::io::Write + ?Sized>(
        &self,
        ctx: &mut crate::SpCtx,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        // Write bool as a u8
        if *self {
            1u8
        } else {
            0
        }.inner_to_writer(ctx, dst)
    }
}

primitive_write!(AtomicU8, u8);
primitive_write!(AtomicU16, u16);
primitive_write!(AtomicU32, u32);
primitive_write!(AtomicU64, u64);
primitive_write!(AtomicUsize, usize);
primitive_write!(AtomicI8, i8);
primitive_write!(AtomicI16, i16);
primitive_write!(AtomicI32, i32);
primitive_write!(AtomicI64, i64);
primitive_write!(AtomicIsize, isize);

impl SpWrite for AtomicBool {
    fn inner_to_writer<W: std::io::Write + ?Sized>(
        &self,
        ctx: &mut crate::SpCtx,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        // Write bool as a u8
        if self.load(Ordering::Relaxed) {
            1u8
        } else {
            0
        }.inner_to_writer(ctx, dst)
    }
}

primitive_write!(NonZeroU8, u8);
primitive_write!(NonZeroU16, u16);
primitive_write!(NonZeroU32, u32);
primitive_write!(NonZeroU64, u64);
primitive_write!(NonZeroU128, u128);
primitive_write!(NonZeroUsize, usize);
primitive_write!(NonZeroI8, i8);
primitive_write!(NonZeroI16, i16);
primitive_write!(NonZeroI32, i32);
primitive_write!(NonZeroI64, i64);
primitive_write!(NonZeroI128, i128);
primitive_write!(NonZeroIsize, isize);