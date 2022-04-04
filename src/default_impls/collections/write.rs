use std::{
    collections::*,
    convert::TryInto,
    ffi::{CStr, CString},
};

use core::hash::Hash;

use crate::*;

impl SpWrite for &str {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        ctx: &mut SpCtx,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.as_bytes().inner_to_writer(ctx, dst)
    }
}

impl SpWrite for String {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        ctx: &mut SpCtx,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.as_str().inner_to_writer(ctx, dst)
    }
}

impl SpWrite for &CStr {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        ctx: &mut SpCtx,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.to_bytes_with_nul().inner_to_writer(ctx, dst)
    }
}

impl SpWrite for CString {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        ctx: &mut SpCtx,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.as_c_str().inner_to_writer(ctx, dst)
    }
}

impl<T: SpWrite> SpWrite for Option<T> {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        ctx: &mut SpCtx,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        match self {
            Some(v) => {
                let o = 1u8;
                Ok(o.inner_to_writer(ctx, dst)? + v.inner_to_writer(ctx, dst)?)
            }
            None => {
                let o = 0u8;
                o.inner_to_writer(ctx, dst)
            }
        }
    }
}

macro_rules! iterator_write {
    ($typ:ty $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)?)*) => {
        impl<$($generics : SpWrite $(+ $bound$(+ $other)*)?),*> SpWrite for $typ {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                ctx: &mut SpCtx,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                let mut total_sz = 0;
                // Write size if needed
                if ctx.len.is_none() {
                    let len: DefaultCountType = match self.len().try_into() {
                        Ok(v) => v,
                        Err(_e) => return Err(SpError::CountFieldOverflow),
                    };
                    total_sz += len.inner_to_writer(ctx, dst)?;
                }

                // Dont propagate `len` field to inner types
                ctx.len = None;

                iterator_write!(inner, total_sz, self, ctx, dst $(+ $generics)*);

                Ok(total_sz)
            }
        }
    };
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

iterator_write!(&[T], T);
iterator_write!(Vec<T>, T);
iterator_write!(VecDeque<T>, T);
iterator_write!(LinkedList<T>, T);
iterator_write!(HashSet<K>, K: Eq + Hash);
iterator_write!(BTreeSet<K>, K: Ord);
iterator_write!(HashMap<K,V>, K: Eq + Hash, V);
iterator_write!(BTreeMap<K,V>, K: Ord, V);
iterator_write!(BinaryHeap<T>, T: Ord);
