
use std::mem::size_of;
use std::cmp::Eq;
use std::hash::Hash;
use std::num::*;
use std::sync::atomic::*;
use std::ffi::{CStr, CString};
use std::collections::*;

use crate::*;

/// Implements SpOptHints on static types
/// This macro also implements the trait for references and slices of &typ
macro_rules! impl_static {
    ($typ:ty) => {
        unsafe impl SpOptHints for $typ {
            const IS_VAR_SIZE: bool = false;
            const STATIC_SIZE: usize = size_of::<$typ>();
        }
        unsafe impl SpOptHints for &$typ {
            const IS_VAR_SIZE: bool = false;
            const STATIC_SIZE: usize = size_of::<$typ>();
        }
        unsafe impl SpOptHints for &mut $typ {
            const IS_VAR_SIZE: bool = false;
            const STATIC_SIZE: usize = size_of::<$typ>();
        }
        unsafe impl SpOptHints for &[$typ] {
            const STATIC_SIZE: usize = DefaultCountType::STATIC_SIZE;
        }
        unsafe impl SpOptHints for &mut [$typ] {
            const STATIC_SIZE: usize = DefaultCountType::STATIC_SIZE;
        }
    }
}

impl_static!(u8);
impl_static!(u16);
impl_static!(u32);
impl_static!(u64);
impl_static!(u128);
impl_static!(usize);
impl_static!(i8);
impl_static!(i16);
impl_static!(i32);
impl_static!(i64);
impl_static!(i128);
impl_static!(isize);
impl_static!(f32);
impl_static!(f64);
unsafe impl SpOptHints for bool {
    const IS_VAR_SIZE: bool = false;
    const STATIC_SIZE: usize = <u8>::STATIC_SIZE;
}

impl_static!(AtomicU8);
impl_static!(AtomicU16);
impl_static!(AtomicU32);
impl_static!(AtomicU64);
impl_static!(AtomicUsize);
impl_static!(AtomicI8);
impl_static!(AtomicI16);
impl_static!(AtomicI32);
impl_static!(AtomicI64);
impl_static!(AtomicIsize);
unsafe impl SpOptHints for AtomicBool {
    const IS_VAR_SIZE: bool = false;
    const STATIC_SIZE: usize = <bool>::STATIC_SIZE;
}

impl_static!(NonZeroU8);
impl_static!(NonZeroU16);
impl_static!(NonZeroU32);
impl_static!(NonZeroU64);
impl_static!(NonZeroU128);
impl_static!(NonZeroUsize);
impl_static!(NonZeroI8);
impl_static!(NonZeroI16);
impl_static!(NonZeroI32);
impl_static!(NonZeroI64);
impl_static!(NonZeroI128);
impl_static!(NonZeroIsize);

// Implements SpOptHints on dynamically sized types.
// All dynamically sized types have at least DefaultCountType as part of their STATIC_SIZE
// because this amount gets substracted from checked_bytes when a count it provided
macro_rules! impl_dynamic {
    ($typ:ty$(, $generics:tt $(: $bound:ident $(+ $other:ident)*)?)*) => {
        unsafe impl<$($generics :SpOptHints $(+ $bound$(+ $other)*)*),*> SpOptHints for $typ {
            const STATIC_SIZE: usize = DefaultCountType::STATIC_SIZE;
            const COUNT_SIZE: usize = DefaultCountType::STATIC_SIZE; 
        }
    }
}

// There should at least be 1 byte available (potentially null terminator) for CStrings
unsafe impl SpOptHints for &CStr {
    const STATIC_SIZE: usize = <u8>::STATIC_SIZE;
}
unsafe impl SpOptHints for CString {
    const STATIC_SIZE: usize = <u8>::STATIC_SIZE;
}
impl_dynamic!(&str);
impl_dynamic!(String);
unsafe impl<T> SpOptHints for Option<T> {
    const STATIC_SIZE: usize = <bool>::STATIC_SIZE;
    const COUNT_SIZE: usize = <bool>::STATIC_SIZE; 
}

impl_dynamic!(Vec<T>, T);
impl_dynamic!(VecDeque<T>, T);
impl_dynamic!(LinkedList<T>, T);
impl_dynamic!(HashSet<K>, K: Eq + Hash);
impl_dynamic!(BTreeSet<K>, K: Ord);
impl_dynamic!(HashMap<K,V>, K: Eq + Hash, V);
impl_dynamic!(BTreeMap<K,V>, K: Ord, V);
impl_dynamic!(BinaryHeap<T>, T: Ord);