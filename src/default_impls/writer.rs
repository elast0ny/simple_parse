use std::cmp::{Eq, Ord};
use std::collections::*;
use std::ffi::{CStr, CString};
use std::hash::Hash;
use std::num::*;
use std::sync::atomic::*;

use crate::*;

impl_writer_all!(u8, prim_to_writer);
impl_writer_all!(u16, prim_to_writer);
impl_writer_all!(u32, prim_to_writer);
impl_writer_all!(u64, prim_to_writer);
impl_writer_all!(u128, prim_to_writer);
impl_writer_all!(usize, prim_to_writer);
impl_writer_all!(i8, prim_to_writer);
impl_writer_all!(i16, prim_to_writer);
impl_writer_all!(i32, prim_to_writer);
impl_writer_all!(i64, prim_to_writer);
impl_writer_all!(i128, prim_to_writer);
impl_writer_all!(isize, prim_to_writer);
impl_writer_all!(bool as u8, prim_to_writer);
impl_writer_all!(f32 as u32, prim_to_writer);
impl_writer_all!(f64 as u64, prim_to_writer);

impl_writer_all!(AtomicU8 as u8, prim_to_writer);
impl_writer_all!(AtomicU16 as u16, prim_to_writer);
impl_writer_all!(AtomicU32 as u32, prim_to_writer);
impl_writer_all!(AtomicU64 as u64, prim_to_writer);
impl_writer_all!(AtomicUsize as usize, prim_to_writer);
impl_writer_all!(AtomicI8 as i8, prim_to_writer);
impl_writer_all!(AtomicI16 as i16, prim_to_writer);
impl_writer_all!(AtomicI32 as i32, prim_to_writer);
impl_writer_all!(AtomicI64 as i64, prim_to_writer);
impl_writer_all!(AtomicIsize as isize, prim_to_writer);
impl_writer_all!(AtomicBool as bool, prim_to_writer);

impl_writer_all!(NonZeroU8 as u8, prim_to_writer);
impl_writer_all!(NonZeroU16 as u16, prim_to_writer);
impl_writer_all!(NonZeroU32 as u32, prim_to_writer);
impl_writer_all!(NonZeroU64 as u64, prim_to_writer);
impl_writer_all!(NonZeroU128 as u128, prim_to_writer);
impl_writer_all!(NonZeroUsize as usize, prim_to_writer);
impl_writer_all!(NonZeroI8 as i8, prim_to_writer);
impl_writer_all!(NonZeroI16 as i16, prim_to_writer);
impl_writer_all!(NonZeroI32 as i32, prim_to_writer);
impl_writer_all!(NonZeroI64 as i64, prim_to_writer);
impl_writer_all!(NonZeroI128 as i128, prim_to_writer);
impl_writer_all!(NonZeroIsize as isize, prim_to_writer);

/* String types */

macro_rules! asbytes_to_writer {
    ($self:ident, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        // Convert to &[u8]
        let s = $self.as_bytes();
        s.inner_to_writer($is_output_le, $prepend_count, $dst)
    }};
}
impl_writer!(&str, asbytes_to_writer);
impl_writer!(String, asbytes_to_writer);

macro_rules! tobytes_to_writer {
    ($self:ident, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        // Convert to &[u8]
        let s = $self.to_bytes_with_nul();
        s.inner_to_writer($is_output_le, $prepend_count, $dst)
    }};
}
impl_writer!(&CStr, tobytes_to_writer);
impl_writer!(CString, tobytes_to_writer);

/* Generic types */

macro_rules! option_to_writer {
    ($self:ident, $is_output_le:ident, $prepend_count:ident, $dst: ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)?)*) => {{
        let is_some: DefaultCountType;
        let mut total_sz: usize = 0;
        match $self {
            Some(v) => {
                if $prepend_count {
                    is_some = 1;
                    total_sz += is_some.inner_to_writer($is_output_le, $prepend_count, $dst)?;
                }
                total_sz += v.inner_to_writer($is_output_le, $prepend_count, $dst)?;
            }
            None => {
                if $prepend_count {
                    is_some = 0;
                    total_sz += is_some.inner_to_writer($is_output_le, $prepend_count, $dst)?
                }
            }
        }
        Ok(total_sz)
    }};
}
impl_writer!(Option<T>, option_to_writer, T);

impl_writer!(Vec<T>, iterator_to_writer, T);
impl_writer!(VecDeque<T>, iterator_to_writer, T);
impl_writer!(LinkedList<T>, iterator_to_writer, T);
impl_writer!(HashSet<K>, iterator_to_writer, K: Eq + Hash);
impl_writer!(BTreeSet<K>, iterator_to_writer, K: Ord);
impl_writer!(HashMap<K,V>, iterator_to_writer, K: Eq + Hash, V);
impl_writer!(BTreeMap<K,V>, iterator_to_writer, K: Ord, V);
impl_writer!(BinaryHeap<T>, iterator_to_writer, T: Ord);
