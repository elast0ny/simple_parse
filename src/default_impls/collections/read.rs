use core::mem::size_of;
use core::hash::Hash;
use std::{ffi::CString, collections::*};


use crate::*;

impl SpRead for String {
    const STATIC_CHECKS: () = {
        const _: () = assert!(String::STATIC_SIZE == <Vec<u8>>::STATIC_SIZE);
    };

    #[inline(always)]
    fn inner_from_reader<R: Read + ?Sized>(
        src: &mut R,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError> {
        let mut dst = [0u8; Self::STATIC_SIZE];
        validate_reader_exact(ctx, &mut dst, src)?;
        unsafe { Self::inner_from_reader_unchecked(dst.as_mut_ptr(), src, ctx) }
    }

    #[inline(always)]
    unsafe fn inner_from_reader_unchecked<R: Read + ?Sized>(
        checked_bytes: *mut u8,
        src: &mut R,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError>
    {
        let bytes = <Vec<u8>>::inner_from_reader_unchecked(checked_bytes, src, ctx)?;
        match String::from_utf8(bytes) {
            Ok(v) => Ok(v),
            Err(_) => Err(SpError::InvalidBytes),
        }
    }
}

impl SpRead for CString {
    #[inline(always)]
    fn inner_from_reader<R: Read + ?Sized>(
        src: &mut R,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError> {
        let mut dst = [0u8; Self::STATIC_SIZE];
        validate_reader_exact(ctx, &mut dst, src)?;
        unsafe { Self::inner_from_reader_unchecked(dst.as_mut_ptr(), src, ctx) }
    }

    #[inline(always)]
    unsafe fn inner_from_reader_unchecked<R: Read + ?Sized>(
        checked_bytes: *mut u8,
        src: &mut R,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError>
    {
        let mut bytes = Vec::new();
        if *checked_bytes == 0 {
            return Ok(CString::from_vec_unchecked(bytes));
        }
        bytes.push(*checked_bytes);     

        // Read one byte at a time adding them to bytes until we hit a null terminator
        let mut dst = [0u8];
        while let Ok(()) = validate_reader_exact(ctx, &mut dst, src) {
            if dst[0] == 0x00 {
                break;
            }
            bytes.push(dst[0]);
        }

        Ok(CString::from_vec_unchecked(bytes))
    }
}

impl<T: SpRead> SpRead for Option<T> {
    fn inner_from_reader<R: Read + ?Sized>(
        src: &mut R,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError> {
        let mut dst = [0u8; size_of::<u8>()];
        validate_reader_exact(ctx, &mut dst, src)?;
        unsafe { Self::inner_from_reader_unchecked(dst.as_mut_ptr(), src, ctx) }
    }

    unsafe fn inner_from_reader_unchecked<R: Read + ?Sized>(
        checked_bytes: *mut u8,
        src: &mut R,
        ctx: &mut SpCtx,
    ) -> Result<Self, crate::SpError>
    {
        if *checked_bytes == 0 {
            return Ok(None);
        }

        Ok(Some(T::inner_from_reader(src, ctx)?))
    }
}

macro_rules! collection_read {
    ($typ:ty, $add_func:ident, $generic:tt $(: $bound:ident $(+ $other:ident)*)? $(, $generics:tt $(: $bounds:ident $(+ $others:ident)*)?)*) => {
        impl<'b, $generic : SpRead $(+ $bound $(+ $other)*)?, $($generics : SpRead $(+ $bounds$(+ $others)*)?),*> SpRead for $typ {
            fn inner_from_reader<R: Read + ?Sized>(
                src: &mut R,
                ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError> {
                let mut dst = [0u8; size_of::<DefaultCountType>()];
                validate_reader_exact(ctx, &mut dst, src)?;
                unsafe { Self::inner_from_reader_unchecked(dst.as_mut_ptr(), src, ctx) }
            }
        
            unsafe fn inner_from_reader_unchecked<R: Read + ?Sized>(
                checked_bytes: *mut u8,
                src: &mut R,
                ctx: &mut SpCtx,
            ) -> Result<Self, crate::SpError>
            {
                let len = match ctx.len.take() {
                    None => <DefaultCountType>::inner_from_reader_unchecked(checked_bytes, src, ctx)? as usize,
                    Some(v) => v,
                };
                
                let mut r = <$typ>::new();
                // If any of the generics are variably sized
                if <$generic>::IS_VAR_SIZE $( || <$generics>::IS_VAR_SIZE)* {
                    // Add every item 1 by 1
                    for _i in 0..len {
                        r.$add_func(
                            <$generic>::inner_from_reader(src, ctx)?
                            $(,<$generics>::inner_from_reader(src, ctx)?)*
                        );
                    }
                } else {
                    let mut bytes = Vec::new();
                    // calculate total size required for these statically sized items
                    let sz_needed = len * (<$generic>::STATIC_SIZE $( + !<$generics>::STATIC_SIZE)*);
                    validate_reader(ctx, sz_needed, &mut bytes, src)?;
                    let mut ptr = bytes.as_mut_ptr();
                    for _i in 0..len {
                        r.$add_func(
                            {
                                let v = <$generic>::inner_from_reader_unchecked(ptr, src, ctx)?;
                                ptr = ptr.add(<$generic>::STATIC_SIZE);
                                v
                            }
                            $(,{
                                let v = <$generics>::inner_from_reader_unchecked(ptr, src, ctx)?;
                                ptr = ptr.add(<$generics>::STATIC_SIZE);
                                v
                            })*
                        );
                    }
                }
                Ok(r)
            }
        }
    };
}

collection_read!(Vec<T>, push, T);
collection_read!(VecDeque<T>, push_back, T);
collection_read!(LinkedList<T>, push_back, T);
collection_read!(HashSet<K>, insert, K: Eq + Hash);
collection_read!(BTreeSet<K>, insert, K: Ord);
collection_read!(HashMap<K,V>, insert, K: Eq + Hash, V);
collection_read!(BTreeMap<K,V>, insert, K: Ord, V);
collection_read!(BinaryHeap<T>, push, T: Ord);
