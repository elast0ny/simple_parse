use std::num::*;
use std::sync::atomic::*;
use std::ffi::CString;
use std::collections::*;
use std::cmp::{Eq, Ord};
use std::hash::Hash;

use crate::*;

/* Primitive types */

impl_read!(u8, prim_from_ptr);
impl_read!(u16, prim_from_ptr);
impl_read!(u32, prim_from_ptr);
impl_read!(u64, prim_from_ptr);
impl_read!(u128, prim_from_ptr);
impl_read!(usize, prim_from_ptr);
impl_read!(i8, prim_from_ptr);
impl_read!(i16, prim_from_ptr);
impl_read!(i32, prim_from_ptr);
impl_read!(i64, prim_from_ptr);
impl_read!(i128, prim_from_ptr);
impl_read!(isize, prim_from_ptr);

impl_read!(bool as u8, bool_from_ptr);
impl_read!(f32 as u32, float_from_ptr);
impl_read!(f64 as u64, float_from_ptr);

impl_read!(AtomicU8 as u8, atomic_from_ptr);
impl_read!(AtomicU16 as u16, atomic_from_ptr);
impl_read!(AtomicU32 as u32, atomic_from_ptr);
impl_read!(AtomicU64 as u64, atomic_from_ptr);
impl_read!(AtomicUsize as usize, atomic_from_ptr);
impl_read!(AtomicI8 as i8, atomic_from_ptr);
impl_read!(AtomicI16 as i16, atomic_from_ptr);
impl_read!(AtomicI32 as i32, atomic_from_ptr);
impl_read!(AtomicI64 as i64, atomic_from_ptr);
impl_read!(AtomicIsize as isize, atomic_from_ptr);

impl_read!(NonZeroU8 as u8, nonzero_from_ptr);
impl_read!(NonZeroU16 as u16, nonzero_from_ptr);
impl_read!(NonZeroU32 as u32, nonzero_from_ptr);
impl_read!(NonZeroU64 as u64, nonzero_from_ptr);
impl_read!(NonZeroU128 as u128, nonzero_from_ptr);
impl_read!(NonZeroUsize as usize, nonzero_from_ptr);
impl_read!(NonZeroI8 as i8, nonzero_from_ptr);
impl_read!(NonZeroI16 as i16, nonzero_from_ptr);
impl_read!(NonZeroI32 as i32, nonzero_from_ptr);
impl_read!(NonZeroI64 as i64, nonzero_from_ptr);
impl_read!(NonZeroI128 as i128, nonzero_from_ptr);
impl_read!(NonZeroIsize as isize, nonzero_from_ptr);

/* String types */

macro_rules! string_from_reader {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let bytes = <Vec<u8>>::$unchecked_reader($checked_bytes, $src, $is_input_le, $count)?;
        match String::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(_e) => Err(SpError::InvalidBytes)
        }
    }};
}
impl_read!(String, string_from_reader);

macro_rules! cstring_from_reader {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let _ = $is_input_le;
        let _ = $count;
        let mut bytes = Vec::new();

        // CString::from_vec_unchecked adds the null terminator

        if *$checked_bytes == 0 {
            return Ok(CString::from_vec_unchecked(bytes));
        }

        // Read one byte at a time adding them to bytes until we hit a null terminator
        let mut dst = [0u8];
        while let Ok(()) = validate_reader_exact(&mut dst, $src) {
            if dst[0] == 0x00 {
                break;
            }
            bytes.push(dst[0]);
        }
        
        Ok(CString::from_vec_unchecked(bytes))
    }};
}
impl_read!(CString, cstring_from_reader);

/* Generic types */

/// Returns an Option<T> from a Reader
macro_rules! option_from_reader {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr, $generic:tt) => {{
        // Dont use checked_bytes if count is provided
        let is_some: bool = match $count {
            Some(c) => c != 0,
            None => {
                <bool>::$unchecked_reader($checked_bytes, $src, $is_input_le, $count)?
            }
        };
        
        Ok(if is_some {
            None
        } else {
            Some(<$generic>::$reader($src, $is_input_le, $count)?)
        })
    }};
}
impl_read!(Option<T>, <bool>::STATIC_SIZE, option_from_reader, T);

/// Generates code for populating generic types
#[macro_use]
macro_rules! generic_from_reader {
    ($alloc_call:ident, $add_call:ident, $typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr, $generic:tt $(: $bound:ident $(+ $other:ident)*)? $(, $generics:tt $(: $bounds:ident $(+ $others:ident)*)?)*) => {{

        let count: usize = match $count {
            Some(c) => c,
            None => {
                <DefaultCountType>::$unchecked_reader($checked_bytes, $src, $is_input_le, $count)? as _
            }
        };

        let mut res;

        if !<$generic>::IS_VAR_SIZE $( && !<$generics>::IS_VAR_SIZE)* {
            let mut dst = Vec::new();
            // Every item has the same size, we can validate...
            let item_size = <$generic>::STATIC_SIZE $( + !<$generics>::STATIC_SIZE)*;
            // Read into dst vec
            validate_reader(count * item_size, &mut dst, $src)?;
            let mut items_ptr = dst.as_mut_ptr();
            res = $alloc_call!(count);

            for _i in 0..count {
                res.$add_call(
                {
                    let v = <$generic>::$unchecked_reader(items_ptr, $src, $is_input_le, None)?;
                    items_ptr = items_ptr.add(<$generic>::STATIC_SIZE);
                    v
                }
                $(,{
                    let v = <$generics>::$unchecked_reader(items_ptr, $src, $is_input_le, None)?;
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
                    <$generic>::$reader($src, $is_input_le, None)?
                    $(,<$generics>::$reader($src, $is_input_le, None)?)*
                );
            }
        }

        Ok(res)
    }};
}

/// Returns a Vec<T> from a Reader
macro_rules! vec_from_reader {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr, $generic:tt $(: $bound:ident $(+ $other:ident)*)?) => {
        generic_from_reader!(new_with_capacity, push, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $is_input_le, $count, $generic $(: $bound $(+ $other)*)?)
    };
}
impl_read!(Vec<T>, <DefaultCountType>::STATIC_SIZE, vec_from_reader, T);

/// Returns a VecDeque<T> from a Reader
macro_rules! vecdeque_from_reader {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr, $generic:tt $(: $bound:ident $(+ $other:ident)*)?) => {
        generic_from_reader!(new_with_capacity, push_back, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $is_input_le, $count, $generic $(: $bound $(+ $other)*)?)
    };
}
impl_read!(VecDeque<T>, <DefaultCountType>::STATIC_SIZE, vecdeque_from_reader, T);

/// Returns a LinkedList<T> from a Reader
macro_rules! linkedlist_from_reader {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr, $generic:tt $(: $bound:ident $(+ $other:ident)*)?) => {
        generic_from_reader!(new_empty, push_back, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $is_input_le, $count, $generic $(: $bound $(+ $other)*)?)
    };
}
impl_read!(LinkedList<T>, <DefaultCountType>::STATIC_SIZE, linkedlist_from_reader, T);

/// Returns a HashSet<K> from a Reader
macro_rules! hashset_from_reader {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr, $generic:tt $(: $bound:ident $(+ $other:ident)*)?) => {
        generic_from_reader!(new_with_capacity, insert, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $is_input_le, $count, $generic $(: $bound $(+ $other)*)?)
    };
}
impl_read!(HashSet<K>, <DefaultCountType>::STATIC_SIZE, hashset_from_reader, K: Eq + Hash);

/// Returns a BTreeSet<K> from a Reader
macro_rules! btreeset_from_reader {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr, $generic:tt $(: $bound:ident $(+ $other:ident)*)?) => {
        generic_from_reader!(new_empty, insert, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $is_input_le, $count, $generic $(: $bound $(+ $other)*)?)
    };
}
impl_read!(BTreeSet<K>, <DefaultCountType>::STATIC_SIZE, btreeset_from_reader, K: Ord);

/// Returns a HashMap<K, V> from a Reader
macro_rules! hashmap_from_reader {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr, $generic1:tt $(: $bound1:ident $(+ $other1:ident)*)?, $generic2:tt $(: $bound2:ident $(+ $other2:ident)*)?) => {
        generic_from_reader!(new_with_capacity, insert, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $is_input_le, $count, $generic1 $(: $bound1 $(+ $other1)*)?, $generic2 $(: $bound2 $(+ $other2)*)?)
    };
}
impl_read!(HashMap<K,V>, <DefaultCountType>::STATIC_SIZE, hashmap_from_reader, K: Eq + Hash, V);

/// Returns a BTreeMap<K, V> from a Reader
macro_rules! btreemap_from_reader {
    ($typ:ty, $reader:ident, $unchecked_reader:ident, $checked_bytes:ident, $src:expr, $is_input_le:expr, $count:expr, $generic1:tt $(: $bound1:ident $(+ $other1:ident)*)?, $generic2:tt $(: $bound2:ident $(+ $other2:ident)*)?) => {
        generic_from_reader!(new_empty, insert, $typ, $reader, $unchecked_reader, $checked_bytes, $src, $is_input_le, $count, $generic1 $(: $bound1 $(+ $other1)*)?, $generic2 $(: $bound2 $(+ $other2)*)?)
    };
}
impl_read!(BTreeMap<K,V>, <DefaultCountType>::STATIC_SIZE, btreemap_from_reader, K: Ord, V);
impl_read!(BinaryHeap<T>, <DefaultCountType>::STATIC_SIZE, vec_from_reader, T: Ord);
