use crate::{SpError, SpRead, SpReadRaw, SpReadRawMut, SpWrite};
use std::convert::TryInto;
use std::io::{Cursor, Read, Write};
use std::sync::atomic::*;
use std::num::*;

/* Primitive Types */

/// Read -> type
macro_rules! reader_to_primitive {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        // Create dst
        let _ = $count;
        let mut val_dst = <$typ>::default();
        let dst = unsafe {
            std::slice::from_raw_parts_mut(
                (&mut val_dst) as *mut $typ as *mut u8,
                std::mem::size_of::<$typ>(),
            )
        };

        // Read into dst
        if $src.read(dst).is_err() {
            return Err(SpError::NotEnoughSpace);
        }

        // Convert endianness if needed
        let val = if $is_input_le {
            if cfg!(target_endian = "big") {
                val_dst.swap_bytes()
            } else {
                val_dst
            }
        } else {
            if cfg!(target_endian = "little") {
                val_dst.swap_bytes()
            } else {
                val_dst
            }
        };

        Ok(val)
    }};
}
/// &[u8] -> type
macro_rules! slice_to_primitive {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let val_ref = <&$typ>::$parse_func($src, $is_input_le, $count)?;
        let val = if $is_input_le {
            val_ref.to_le()
        } else {
            val_ref.to_be()
        };
        Ok(val)
    }};
}
/// &[u8] -> &type
macro_rules! slice_to_ref_primitive {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let _ = $count;
        let _ = $is_input_le;
        let idx = $src.position();
        // Size check
        let bytes = $src.get_ref();
        if idx + std::mem::size_of::<$inner_typ>() as u64 > bytes.len() as u64 {
            return Err(SpError::NotEnoughSpace);
        }
        // Cast to reference
        let val = unsafe { &*(bytes.as_ptr().add(idx.try_into().unwrap()) as *const $inner_typ) };
        // Move cursor forward
        $src.set_position(idx + std::mem::size_of::<$inner_typ>() as u64);
        Ok(val)
    }};
}
/// &[u8] -> &mut type
macro_rules! slice_to_mutref_primitive {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let ref_val =
            slice_to_ref_primitive!($typ, $inner_typ, $parse_func, $src, $is_input_le, $count)?;
        #[allow(clippy::cast_ref_to_mut)]
        let val = unsafe { &mut *(ref_val as *const $inner_typ as *mut $inner_typ) };
        Ok(val)
    }};
}
/// &[u8] -> & [type]
macro_rules! slice_to_refslice_primitive {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let num_items = match $count {
            Some(v) => v as u64,
            None => <u64>::$parse_func($src, $is_input_le, $count)?,
        };
        let sz_needed = num_items * std::mem::size_of::<$inner_typ>() as u64;
        let idx = $src.position();
        // Size check
        let bytes = $src.get_ref();
        if idx + sz_needed > bytes.len() as u64 {
            return Err(SpError::NotEnoughSpace);
        }
        let val = unsafe {
            std::slice::from_raw_parts(
                bytes.as_ptr().add(idx.try_into().unwrap()) as *const $inner_typ,
                num_items.try_into().unwrap(),
            )
        };
        $src.set_position(idx + sz_needed);
        Ok(val)
    }};
}
/// &[u8] -> &mut [type]
macro_rules! slice_to_mutrefslice_primitive {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let ref_val = slice_to_refslice_primitive!(
            $typ,
            $inner_typ,
            $parse_func,
            $src,
            $is_input_le,
            $count
        )?;
        let val = unsafe {
            std::slice::from_raw_parts_mut(
                ref_val.as_ptr() as *const $inner_typ as *mut $inner_typ,
                ref_val.len(),
            )
        };
        Ok(val)
    }};
}
/// Writes T into dst
macro_rules! primitive_to_writer {
    ($self:ident $(as $as_typ:ty)?, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
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
/// Implements all possible permutations for primitive types
macro_rules! impl_primitive {
    ($typ:ty) => {
        // Read - > Self
        impl_SpRead!($typ, _, reader_to_primitive);

        // SpReadRaw* -> Self
        impl_SpReadRaw!($typ, _, slice_to_primitive);
        impl_SpReadRawMut!($typ, _, slice_to_primitive);

        // Impl SpReadRaw* -> &[Self]
        impl_SpReadRaw!(&[$typ], $typ, slice_to_refslice_primitive);
        impl_SpReadRawMut!(&[$typ], $typ, slice_to_refslice_primitive);
        impl_SpReadRawMut!(&mut [$typ], $typ, slice_to_mutrefslice_primitive);

        // Impl SpReadRaw* to &Self
        impl_SpReadRaw!(&$typ, $typ, slice_to_ref_primitive);
        impl_SpReadRawMut!(&$typ, $typ, slice_to_ref_primitive);
        impl_SpReadRawMut!(&mut $typ, $typ, slice_to_mutref_primitive);

        // Write
        impl_SpWrite!($typ, primitive_to_writer);
    };
}

impl_primitive!(u8);
impl_primitive!(u16);
impl_primitive!(u32);
impl_primitive!(u64);
impl_primitive!(u128);
impl_primitive!(usize);
impl_primitive!(i8);
impl_primitive!(i16);
impl_primitive!(i32);
impl_primitive!(i64);
impl_primitive!(i128);
impl_primitive!(isize);

/* Atomics */

/// Read -> AtomicT
macro_rules! reader_to_atomic {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = reader_to_primitive!($inner_typ, _, $parse_func, $src, $is_input_le, $count)?;
        Ok(<$typ>::new(inner))
    }};
}
/// &[u8] -> AtomicT
macro_rules! slice_to_atomic {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = slice_to_primitive!($inner_typ, _, $parse_func, $src, $is_input_le, $count)?;
        Ok(<$typ>::new(inner))
    }};
}
/// &[u8] -> &AtomicT
macro_rules! slice_to_ref_atomic {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = slice_to_ref_primitive!(
            &$inner_typ,
            $inner_typ,
            $parse_func,
            $src,
            $is_input_le,
            $count
        )?;
        Ok(unsafe { &*(inner as *const _ as *const _) })
    }};
}
/// &[u8] -> &mut AtomicT
macro_rules! slice_to_mutref_atomic {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = slice_to_mutref_primitive!(
            &mut $inner_typ,
            $inner_typ,
            $parse_func,
            $src,
            $is_input_le,
            $count
        )?;
        Ok(unsafe { &mut *(inner as *mut _ as *mut _) })
    }};
}
/// &[u8] -> & [AtomicT]
macro_rules! slice_to_refslice_atomic {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = slice_to_refslice_primitive!(
            $inner_typ,
            $inner_typ,
            $parse_func,
            $src,
            $is_input_le,
            $count
        )?;
        Ok(unsafe { std::slice::from_raw_parts(inner.as_ptr() as *const _, inner.len()) })
    }};
}
/// &[u8] -> &mut [AtomicT]
macro_rules! slice_to_mutrefslice_atomic {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = slice_to_mutrefslice_primitive!(
            $inner_typ,
            $inner_typ,
            $parse_func,
            $src,
            $is_input_le,
            $count
        )?;
        Ok(unsafe { std::slice::from_raw_parts_mut(inner.as_mut_ptr() as *mut _, inner.len()) })
    }};
}
/// Writes AtomicT into dst
macro_rules! atomic_to_writer {
    ($self:ident $(as $as_typ:ty)?, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        let val = $self.load(Ordering::Relaxed);
        primitive_to_writer!(val, $is_output_le, $prepend_count, $dst)
    }};
}

/// Implements all possible permutations for AtomicT types
macro_rules! impl_atomic {
    ($typ:ty, $as_typ:ty) => {
        // Read - > Self
        impl_SpRead!($typ, $as_typ, reader_to_atomic);

        // SpReadRaw* -> Self
        impl_SpReadRaw!($typ, $as_typ, slice_to_atomic);
        impl_SpReadRawMut!($typ, $as_typ, slice_to_atomic);

        // Impl SpReadRaw* -> &[Self]
        impl_SpReadRaw!(&[$typ], $as_typ, slice_to_refslice_atomic);
        impl_SpReadRawMut!(&[$typ], $as_typ, slice_to_refslice_atomic);
        impl_SpReadRawMut!(&mut [$typ], $as_typ, slice_to_mutrefslice_atomic);

        // Impl SpReadRaw* to &Self
        impl_SpReadRaw!(&$typ, $as_typ, slice_to_ref_atomic);
        impl_SpReadRawMut!(&$typ, $as_typ, slice_to_ref_atomic);
        impl_SpReadRawMut!(&mut $typ, $as_typ, slice_to_mutref_atomic);

        // Write
        impl_SpWrite!($typ as $as_typ, atomic_to_writer);
    };
}

impl_atomic!(AtomicI8, i8);
impl_atomic!(AtomicI16, i16);
impl_atomic!(AtomicI32, i32);
impl_atomic!(AtomicI64, i64);
impl_atomic!(AtomicIsize, isize);
impl_atomic!(AtomicU8, u8);
impl_atomic!(AtomicU16, u16);
impl_atomic!(AtomicU32, u32);
impl_atomic!(AtomicU64, u64);
impl_atomic!(AtomicUsize, usize);

/* NonZero */

/// Read -> NonZeroT
macro_rules! reader_to_nonzero {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = reader_to_primitive!($inner_typ, _, $parse_func, $src, $is_input_le, $count)?;
        match <$typ>::new(inner) {
            Some(s) => Ok(s),
            None => Err(SpError::InvalidBytes),
        }
    }};
}
/// &[u8] -> NonZeroT
macro_rules! slice_to_nonzero {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = slice_to_primitive!($inner_typ, _, $parse_func, $src, $is_input_le, $count)?;
        match <$typ>::new(inner) {
            Some(s) => Ok(s),
            None => Err(SpError::InvalidBytes),
        }
    }};
}
/// &[u8] -> &NonZeroT
macro_rules! slice_to_ref_nonzero {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = slice_to_ref_primitive!(
            &$inner_typ,
            $inner_typ,
            $parse_func,
            $src,
            $is_input_le,
            $count
        )?;
        unsafe {
            if *inner == 0 {
                Err(SpError::InvalidBytes)
            } else {
                Ok(&*(inner as *const _ as *const _))
            }
        }
    }};
}
/// &[u8] -> &mut NonZeroT
macro_rules! slice_to_mutref_nonzero {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = slice_to_mutref_primitive!(
            &mut $inner_typ,
            $inner_typ,
            $parse_func,
            $src,
            $is_input_le,
            $count
        )?;
        unsafe {
            if *inner == 0 {
                Err(SpError::InvalidBytes)
            } else {
                Ok(&mut *(inner as *mut _ as *mut _))
            }
        }
    }};
}
/// &[u8] -> & [NonZeroT]
macro_rules! slice_to_refslice_nonzero {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = slice_to_refslice_primitive!(
            $inner_typ,
            $inner_typ,
            $parse_func,
            $src,
            $is_input_le,
            $count
        )?;
        for v in inner.iter() {
            if *v == 0 {
                return Err(SpError::InvalidBytes);
            }
        }
        Ok(unsafe { std::slice::from_raw_parts(inner.as_ptr() as *const _, inner.len()) })
    }};
}
/// &[u8] -> &mut [NonZeroT]
macro_rules! slice_to_mutrefslice_nonzero {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = slice_to_mutrefslice_primitive!(
            $inner_typ,
            $inner_typ,
            $parse_func,
            $src,
            $is_input_le,
            $count
        )?;
        for v in inner.iter() {
            if *v == 0 {
                return Err(SpError::InvalidBytes);
            }
        }
        Ok(unsafe { std::slice::from_raw_parts_mut(inner.as_mut_ptr() as *mut _, inner.len()) })
    }};
}
/// Writes NonZeroT into dst
macro_rules! nonzero_to_writer {
    ($self:ident $(as $as_typ:ty)?, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        let val = $self.get();
        primitive_to_writer!(val, $is_output_le, $prepend_count, $dst)
    }};
}
/// Implements all possible permutations for NonZeroT types
macro_rules! impl_nonzero {
    ($typ:ty, $as_typ:ty) => {
        // Read - > Self
        impl_SpRead!($typ, $as_typ, reader_to_nonzero);

        // SpReadRaw* -> Self
        impl_SpReadRaw!($typ, $as_typ, slice_to_nonzero);
        impl_SpReadRawMut!($typ, $as_typ, slice_to_nonzero);

        // Impl SpReadRaw* -> &[Self]
        impl_SpReadRaw!(&[$typ], $as_typ, slice_to_refslice_nonzero);
        impl_SpReadRawMut!(&[$typ], $as_typ, slice_to_refslice_nonzero);
        impl_SpReadRawMut!(&mut [$typ], $as_typ, slice_to_mutrefslice_nonzero);

        // Impl SpReadRaw* to &Self
        impl_SpReadRaw!(&$typ, $as_typ, slice_to_ref_nonzero);
        impl_SpReadRawMut!(&$typ, $as_typ, slice_to_ref_nonzero);
        impl_SpReadRawMut!(&mut $typ, $as_typ, slice_to_mutref_nonzero);

        // Write
        impl_SpWrite!($typ as $as_typ, nonzero_to_writer);
    };
}

impl_nonzero!(NonZeroI8, i8);
impl_nonzero!(NonZeroI16, i16);
impl_nonzero!(NonZeroI32, i32);
impl_nonzero!(NonZeroI64, i64);
impl_nonzero!(NonZeroIsize, isize);
impl_nonzero!(NonZeroU8, u8);
impl_nonzero!(NonZeroU16, u16);
impl_nonzero!(NonZeroU32, u32);
impl_nonzero!(NonZeroU64, u64);
impl_nonzero!(NonZeroUsize, usize);

/* Bools */

/* Read / &[u8] / &mut [u8] -> bool */
macro_rules! bool_read {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let val = u8::$parse_func($src, $is_input_le, $count)?;
        Ok(val != 0)
    }};
}
// bool -> Write
macro_rules! bool_to_writer {
    ($self:ident $(as $as_typ:ty)?, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        let val = if *$self { 1u8 } else { 0u8 };
        val.inner_to_writer($is_output_le, $prepend_count, $dst)
    }};
}

impl_SpRead!(bool, _, bool_read);
impl_SpReadRaw!(bool, _, bool_read);
impl_SpReadRawMut!(bool, _, bool_read);
impl_SpWrite!(bool, bool_to_writer);

macro_rules! atomicbool_read {
    ($typ:ty, $inner_typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let inner = <$inner_typ>::$parse_func($src, $is_input_le, $count)?;
        Ok(<$typ>::new(inner))
    }};
}
// bool -> Write
macro_rules! atomicbool_to_writer {
    ($self:ident $(as $as_typ:ty)?, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        let val = & $self.load(Ordering::Relaxed);
        bool_to_writer!(val, $is_output_le, $prepend_count, $dst)
    }};
}

impl_SpRead!(AtomicBool, bool, atomicbool_read);
impl_SpReadRaw!(AtomicBool, bool, atomicbool_read);
impl_SpReadRawMut!(AtomicBool, bool, atomicbool_read);
impl_SpWrite!(AtomicBool as bool, atomicbool_to_writer);

// Nothing about slices or references since the internal repr of a bool
// is unknown
