use std::cmp::{Eq, Ord};
use std::collections::*;
use std::ffi::{CStr, CString};
use std::hash::Hash;
use std::num::*;
use std::sync::atomic::*;

use crate::*;

/* Primitive types */

// Implements all SpReadRaw variations on a primitive type without references
macro_rules! impl_primitive_noref {
    ($typ:ty, $as_copy:ident) => {
        impl_primitive_noref!($typ as $typ, $as_copy);
    };
    // Implements SpReadRaw & SpReadRawMut for T
    ($typ:ty as $as_typ:ty, $as_copy:ident) => {
        impl_readraw!($typ as $as_typ, $as_copy);
        impl_readrawmut!($typ as $as_typ, $as_copy);
    };
}

// Implements all SpReadRaw variations on a primitive type
macro_rules! impl_primitive {
    ($typ:ty, $as_copy:ident) => {
        impl_primitive!($typ as $typ, $as_copy);
    };
    // When no converter to ref and slice is provided, default to using primitve impls
    ($typ:ty as $as_typ:ty, $as_copy:ident) => {
        impl_primitive!($typ as $as_typ, $as_copy, mutref_from_ptr, mutslice_from_cursor);
    };
    // Implements SpReadRaw & SpReadRawMut for T, &T, &mut T, &[T] and &mut [T]
    ($typ:ty as $as_typ:ty, $as_copy:ident, $as_ref:ident, $as_slice:ident) => {
        // Copy
        impl_primitive_noref!($typ as $as_typ, $as_copy);
        // References
        impl_readraw!(&$typ as $as_typ, $as_ref);
        impl_readrawmut!(&$typ as $as_typ, $as_ref);
        impl_readrawmut!(&mut $typ as $as_typ, $as_ref);
        // Slices
        impl_readraw!(&[$typ] as $as_typ, $as_slice);
        impl_readrawmut!(&[$typ] as $as_typ, $as_slice);
        impl_readrawmut!(&mut [$typ] as $as_typ, $as_slice);
    };
}

/// Returns a slice of primitive types from a Cursor<[u8]>
macro_rules! mutslice_from_cursor {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{
        // Dont use checked_bytes if count is provided
        let count: usize = match $ctx.count {
            Some(c) => c,
            None => {
                <DefaultCountType>::$unchecked_reader($checked_bytes, $src, $ctx)?
                    as _
            }
        };

        // Make sure theres enough bytes for count * item_size
        let slice_ptr = validate_cursor(count * <$as_typ>::STATIC_SIZE, $src)?;

        // Make sure the items in the slice are properly aligned
        if std::mem::align_of::<$as_typ>() > 1
            && slice_ptr.align_offset(std::mem::align_of::<$as_typ>()) != 0
        {
            return Err(SpError::BadAlignment);
        }

        $ctx.cursor += count * <$as_typ>::STATIC_SIZE;

        // Return slice from pointer
        Ok(std::slice::from_raw_parts_mut(
            slice_ptr as *const $as_typ as _,
            count,
        ))
    }};
}

impl_primitive!(u8, prim_from_ptr);
impl_primitive!(u16, prim_from_ptr);
impl_primitive!(u32, prim_from_ptr);
impl_primitive!(u64, prim_from_ptr);
impl_primitive!(u128, prim_from_ptr);
impl_primitive!(usize, prim_from_ptr);
impl_primitive!(i8, prim_from_ptr);
impl_primitive!(i16, prim_from_ptr);
impl_primitive!(i32, prim_from_ptr);
impl_primitive!(i64, prim_from_ptr);
impl_primitive!(i128, prim_from_ptr);
impl_primitive!(isize, prim_from_ptr);
impl_primitive_noref!(bool as u8, bool_from_ptr);
impl_primitive!(f32 as u32, float_from_ptr);
impl_primitive!(f64 as u64, float_from_ptr);

impl_primitive!(AtomicU8 as u8, atomic_from_ptr);
impl_primitive!(AtomicU16 as u16, atomic_from_ptr);
impl_primitive!(AtomicU32 as u32, atomic_from_ptr);
impl_primitive!(AtomicU64 as u64, atomic_from_ptr);
impl_primitive!(AtomicUsize as usize, atomic_from_ptr);
impl_primitive!(AtomicI8 as i8, atomic_from_ptr);
impl_primitive!(AtomicI16 as i16, atomic_from_ptr);
impl_primitive!(AtomicI32 as i32, atomic_from_ptr);
impl_primitive!(AtomicI64 as i64, atomic_from_ptr);
impl_primitive!(AtomicIsize as isize, atomic_from_ptr);
impl_primitive_noref!(AtomicBool as bool, atomic_from_ptr);

// Converts a raw pointer to a NonZero reference validating the contents
macro_rules! nonzeroref_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{
        // First get ref to primitive type
        let prim_ref = <&$as_typ>::$unchecked_reader($checked_bytes, $src, $ctx)?;

        // Make sure the &NonZero is not 0
        if *prim_ref == 0 {
            return Err(SpError::InvalidBytes);
        }

        Ok(&mut *(prim_ref as *const $as_typ as *mut $as_typ as *mut _))
    }};
}

// Converts a raw pointer to a slice of NonZero validating the contents
macro_rules! nonzeroslice_from_ptr {
    ($typ:ty as $as_typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{
        // First get ref to primitive type
        let prim_slice = <&[$as_typ]>::$unchecked_reader($checked_bytes, $src, $ctx)?;

        // Make sure the &NonZero is not 0
        for v in prim_slice.iter() {
            if *v == 0 {
                return Err(SpError::InvalidBytes);
            }
        }

        // Convert to NonZeroT slice from &[T]
        let nz_slice = std::slice::from_raw_parts_mut(
            prim_slice.as_ptr() as *mut _,
            prim_slice.len()
        );

        Ok(nz_slice)
    }};
}

impl_primitive!(NonZeroU8 as u8, nonzero_from_ptr, nonzeroref_from_ptr, nonzeroslice_from_ptr);
impl_primitive!(NonZeroU16 as u16, nonzero_from_ptr, nonzeroref_from_ptr, nonzeroslice_from_ptr);
impl_primitive!(NonZeroU32 as u32, nonzero_from_ptr, nonzeroref_from_ptr, nonzeroslice_from_ptr);
impl_primitive!(NonZeroU64 as u64, nonzero_from_ptr, nonzeroref_from_ptr, nonzeroslice_from_ptr);
impl_primitive!(NonZeroU128 as u128, nonzero_from_ptr, nonzeroref_from_ptr, nonzeroslice_from_ptr);
impl_primitive!(NonZeroUsize as usize, nonzero_from_ptr, nonzeroref_from_ptr, nonzeroslice_from_ptr);
impl_primitive!(NonZeroI8 as i8, nonzero_from_ptr, nonzeroref_from_ptr, nonzeroslice_from_ptr);
impl_primitive!(NonZeroI16 as i16, nonzero_from_ptr, nonzeroref_from_ptr, nonzeroslice_from_ptr);
impl_primitive!(NonZeroI32 as i32, nonzero_from_ptr, nonzeroref_from_ptr, nonzeroslice_from_ptr);
impl_primitive!(NonZeroI64 as i64, nonzero_from_ptr, nonzeroref_from_ptr, nonzeroslice_from_ptr);
impl_primitive!(NonZeroI128 as i128, nonzero_from_ptr, nonzeroref_from_ptr, nonzeroslice_from_ptr);
impl_primitive!(NonZeroIsize as isize, nonzero_from_ptr, nonzeroref_from_ptr, nonzeroslice_from_ptr);

/* String types */

/// Returns a &str from a Cursor<[u8]>
macro_rules! str_from_cursor {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{
        // Read as u8 slice
        let byte_slice = <&[u8]>::$unchecked_reader($checked_bytes, $src, $ctx)?;

        // Make sure bytes are valid utf8
        match std::str::from_utf8(byte_slice) {
            Ok(v) => Ok(v),
            Err(_) => Err(SpError::InvalidBytes),
        }
    }};
}
impl_readraw!(&'b str, str_from_cursor);
impl_readrawmut!(&'b str, str_from_cursor);

/// Returns a String from a Cursor<[u8]>
macro_rules! string_from_cursor {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{
        let str_ref = <&str>::$unchecked_reader($checked_bytes, $src, $ctx)?;
        #[cfg(feature = "verbose")]
        crate::debug!("&str.to_string({} bytes)", str_ref.as_bytes().len());
        Ok(str_ref.to_string())
    }};
}
impl_readraw!(String, string_from_cursor);
impl_readrawmut!(String, string_from_cursor);

/// Returns a &CStr from a Cursor<[u8]>
macro_rules! cstr_from_cursor {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{

        if *$checked_bytes == 0 {
            $ctx.cursor += 1;
            let s = std::slice::from_raw_parts($checked_bytes, 1);
            return Ok(CStr::from_bytes_with_nul_unchecked(s));
        }

        let mut num_bytes: u64 = 0;
        let bytes_left: u64 = $src.get_ref().len() as u64 - $src.position();

        while num_bytes < bytes_left {
            $ctx.cursor += 1;
            #[cfg(feature = "verbose")]
            crate::debug!(
                "Check src.len({}) < 1",
                bytes_left
            );
            num_bytes += 1;
            if *$checked_bytes.add(num_bytes as usize) == 0 {
                let s = std::slice::from_raw_parts($checked_bytes, (num_bytes + 1) as usize);

                $src.set_position($src.position() + num_bytes);
                return Ok(CStr::from_bytes_with_nul_unchecked(s));
            }
        }

        return Err(SpError::InvalidBytes);
    }};
}
impl_readraw!(&'b CStr, cstr_from_cursor);
impl_readrawmut!(&'b CStr, cstr_from_cursor);

/// Returns a CString from a Cursor<[u8]>
macro_rules! cstring_from_cursor {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident) => {{
        let str_ref = <&CStr>::$unchecked_reader($checked_bytes, $src, $ctx)?;
        let cstr_bytes = str_ref.to_bytes();
        #[cfg(feature = "verbose")]
        crate::debug!("CStr.clone({} bytes)", cstr_bytes.len());
        Ok(CString::from_vec_unchecked(cstr_bytes.to_vec()))
    }};
}
impl_readraw!(CString, cstring_from_cursor);
impl_readrawmut!(CString, cstring_from_cursor);

/* Generic types */

/// Returns an Option<T> from a Cursor<[u8]>
macro_rules! option_from_cursor {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident, $generic:tt) => {{
        // Dont use checked_bytes if count is provided
        let is_some: bool = match $ctx.count {
            Some(c) => c != 0,
            None => <bool>::$unchecked_reader($checked_bytes, $src, $ctx)?,
        };
        $ctx.count = None;

        Ok(if !is_some {
            None
        } else {
            Some(<$generic>::$reader($src, $ctx)?)
        })
    }};
}
impl_readraw!(Option<T>, option_from_cursor, T);
impl_readrawmut!(Option<T>, option_from_cursor, T);

/// Generates code for populating generic types
#[macro_use]
macro_rules! generic_from_cursor {
    ($alloc_call:ident, $add_call:ident, $typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident, $generic:tt $(: $bound:ident $(+ $other:ident)*)? $(, $generics:tt $(: $bounds:ident $(+ $others:ident)*)?)*) => {{

        let count: usize = match $ctx.count {
            Some(c) => c,
            None => {
                <DefaultCountType>::$unchecked_reader($checked_bytes, $src, $ctx)? as _
            }
        };

        let mut res;
        $ctx.count = None;
        if !<$generic>::IS_VAR_SIZE $( && !<$generics>::IS_VAR_SIZE)* {
            // Every item has the same size, we can validate...
            let item_size = <$generic>::STATIC_SIZE $( + !<$generics>::STATIC_SIZE)*;
            let mut items_ptr = validate_cursor(count * item_size, $src)?;
            //...and preallocate
            res = $alloc_call!(count);
            for _i in 0..count {
                res.$add_call(
                {
                    let v = <$generic>::$unchecked_reader(items_ptr, $src, $ctx)?;
                    items_ptr = items_ptr.add(<$generic>::STATIC_SIZE);
                    v
                }
                $(,{
                    let v = <$generics>::$unchecked_reader(items_ptr, $src, $ctx)?;
                    items_ptr = items_ptr.add(<$generics>::STATIC_SIZE);
                    v
                })*
                );
            }
        } else {
            res = Self::new();
            // Slow path, every item may have a different size
            for _i in 0..count {
                res.$add_call(
                    <$generic>::$reader($src, $ctx)?
                    $(,<$generics>::$reader($src, $ctx)?)*
                );
            }
        }

        Ok(res)
    }};
}

/// Returns a Vec<T> from a Cursor<[u8]>
macro_rules! vec_from_cursor {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident, $generic:tt $(: $bound:ident $(+ $other:ident)*)?) => {
        generic_from_cursor!(new_with_capacity, push, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $ctx, $generic $(: $bound $(+ $other)*)?)
    };
}
impl_readraw!(Vec<T>, vec_from_cursor, T);
impl_readrawmut!(Vec<T>, vec_from_cursor, T);

/// Returns a VecDeque<T> from a Cursor<[u8]>
macro_rules! vecdeque_from_cursor {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident, $generic:tt $(: $bound:ident $(+ $other:ident)*)?) => {
        generic_from_cursor!(new_with_capacity, push_back, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $ctx, $generic $(: $bound $(+ $other)*)?)
    };
}
impl_readraw!(VecDeque<T>, vecdeque_from_cursor, T);
impl_readrawmut!(VecDeque<T>, vecdeque_from_cursor, T);

/// Returns a LinkedList<T> from a Cursor<[u8]>
macro_rules! linkedlist_from_cursor {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident, $generic:tt $(: $bound:ident $(+ $other:ident)*)?) => {
        generic_from_cursor!(new_empty, push_back, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $ctx, $generic $(: $bound $(+ $other)*)?)
    };
}
impl_readraw!(LinkedList<T>, linkedlist_from_cursor, T);
impl_readrawmut!(LinkedList<T>, linkedlist_from_cursor, T);

/// Returns a HashSet<K> from a Cursor<[u8]>
macro_rules! hashset_from_cursor {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident, $generic:tt $(: $bound:ident $(+ $other:ident)*)?) => {
        generic_from_cursor!(new_with_capacity, insert, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $ctx, $generic $(: $bound $(+ $other)*)?)
    };
}
impl_readraw!(HashSet<K>, hashset_from_cursor, K: Eq + Hash);
impl_readrawmut!(HashSet<K>, hashset_from_cursor, K: Eq + Hash);

/// Returns a BTreeSet<K> from a Cursor<[u8]>
macro_rules! btreeset_from_cursor {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident, $generic:tt $(: $bound:ident $(+ $other:ident)*)?) => {
        generic_from_cursor!(new_empty, insert, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $ctx, $generic $(: $bound $(+ $other)*)?)
    };
}
impl_readraw!(BTreeSet<K>, btreeset_from_cursor, K: Ord);
impl_readrawmut!(BTreeSet<K>, btreeset_from_cursor, K: Ord);

/// Returns a HashMap<K, V> from a Cursor<[u8]>
macro_rules! hashmap_from_cursor {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident, $generic1:tt $(: $bound1:ident $(+ $other1:ident)*)?, $generic2:tt $(: $bound2:ident $(+ $other2:ident)*)?) => {
        generic_from_cursor!(new_with_capacity, insert, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $ctx, $generic1 $(: $bound1 $(+ $other1)*)?, $generic2 $(: $bound2 $(+ $other2)*)?)
    };
}
impl_readraw!(HashMap<K,V>, hashmap_from_cursor, K: Eq + Hash, V);
impl_readrawmut!(HashMap<K,V>, hashmap_from_cursor, K: Eq + Hash, V);

/// Returns a BTreeMap<K, V> from a Cursor<[u8]>
macro_rules! btreemap_from_cursor {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $ctx:ident, $generic1:tt $(: $bound1:ident $(+ $other1:ident)*)?, $generic2:tt $(: $bound2:ident $(+ $other2:ident)*)?) => {
        generic_from_cursor!(new_empty, insert, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $ctx, $generic1 $(: $bound1 $(+ $other1)*)?, $generic2 $(: $bound2 $(+ $other2)*)?)
    };
}
impl_readraw!(BTreeMap<K,V>, btreemap_from_cursor, K: Ord, V);
impl_readrawmut!(BTreeMap<K,V>, btreemap_from_cursor, K: Ord, V);

impl_readraw!(BinaryHeap<T>, vec_from_cursor, T: Ord);
impl_readrawmut!(BinaryHeap<T>, vec_from_cursor, T: Ord);
