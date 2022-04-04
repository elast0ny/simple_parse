use core::hash::Hash;
use std::{collections::*, ffi::CString, mem::size_of};

use crate::*;

impl SpRead for String {
    fn inner_from_reader<'a, R: Read + ?Sized>(
        src: &mut R,
        ctx: &mut SpCtx,
        dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError> {
        // Read the string as a Vec<u8>
        let mut tmp = MaybeUninit::uninit();
        <Vec<u8>>::inner_from_reader(src, ctx, &mut tmp)?;
        let bytes = unsafe { tmp.assume_init() };

        // Validate UTF8
        match String::from_utf8(bytes) {
            Ok(v) => dst.write(v),
            Err(_) => return Err(SpError::InvalidBytes),
        };

        let v = unsafe { dst.assume_init_mut() };

        #[cfg(feature = "verbose")]
        if v.len() > 32 {
            ::log::debug!("'{}'...", &v[..v.char_indices().skip(32).next().unwrap().0]);
        } else {
            ::log::debug!("'{}'", &v);
        }

        Ok(v)
    }
}

impl SpRead for CString {
    fn inner_from_reader<'a, R: Read + ?Sized>(
        src: &mut R,
        ctx: &mut SpCtx,
        dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError> {
        // Read one byte at a time adding them to bytes until we hit a null terminator
        let mut bytes = Vec::new();
        let mut tmp = [0u8];
        loop {
            if let Err(e) = src.read_exact(&mut tmp) {
                return Err(SpError::ReadFailed(e));
            }

            #[cfg(feature = "verbose")]
            ::log::debug!("  read(1)");

            if tmp[0] == 0x00 {
                break;
            }
            bytes.push(tmp[0]);
        }

        ctx.cursor += bytes.len();

        unsafe {
            dst.write(CString::from_vec_unchecked(bytes));

            Ok(dst.assume_init_mut())
        }
    }
}

impl<T: SpRead> SpRead for Option<T> {
    fn inner_from_reader<'a, R: Read + ?Sized>(
        src: &mut R,
        ctx: &mut SpCtx,
        dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError> {
        // Read the u8 which indicates None or Some
        let mut tmp = MaybeUninit::uninit();
        let v = <u8>::inner_from_reader(src, ctx, &mut tmp)?;

        // Initialize the value
        if *v == 0 {
            dst.write(None);
        } else {
            let mut tmp = MaybeUninit::uninit();
            <T>::inner_from_reader(src, ctx, &mut tmp)?;
            dst.write(Some(unsafe { tmp.assume_init() }));
        }

        unsafe { Ok(dst.assume_init_mut()) }
    }
}

macro_rules! collection_read {
    ($typ:ty, $add_func:ident, $generic:tt $(: $bound:ident $(+ $other:ident)*)? $(, $generics:tt $(: $bounds:ident $(+ $others:ident)*)?)*) => {
        impl<'b, $generic : SpRead $(+ $bound $(+ $other)*)? $(, $generics : SpRead $(+ $bounds$(+ $others)*)?)*> SpRead for $typ  {
            fn inner_from_reader<'a, R: Read + ?Sized>(
                src: &mut R,
                ctx: &mut SpCtx,
                dst: &'a mut MaybeUninit<Self>,
            ) -> Result<&'a mut Self, crate::SpError>
            {
                // Get the number of elements we must read
                let len = match ctx.len.take() {
                    None => {
                        let mut tmp = MaybeUninit::uninit();
                        *<DefaultCountType>::inner_from_reader(src, ctx, &mut tmp)? as usize
                    },
                    Some(v) => v,
                };

                let mut r = <$typ>::new();
                for _i in 0..len {
                    r.$add_func(
                        {
                            let mut v = MaybeUninit::<$generic>::uninit();
                            <$generic>::inner_from_reader(src, ctx, &mut v)?;
                            unsafe { v.assume_init() }
                        }
                        $(,
                            {
                                let mut v = MaybeUninit::<$generics>::uninit();
                                <$generics>::inner_from_reader(src, ctx, &mut v)?;
                                unsafe { v.assume_init() }
                            }
                        )*
                    );
                }

                dst.write(r);
                Ok(unsafe{ dst.assume_init_mut()})
            }
        }
    };
}

// TODO : Generalize this optimized Vec<T> impl to all collections

impl<T: SpRead + Sized> SpRead for Vec<T> {
    fn inner_from_reader<'a, R: Read + ?Sized>(
        src: &mut R,
        ctx: &mut SpCtx,
        dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError> {
        // Get the number of elements we must read
        let mut len = match ctx.len.take() {
            None => {
                let mut tmp = MaybeUninit::uninit();
                *<DefaultCountType>::inner_from_reader(src, ctx, &mut tmp)? as usize
            }
            Some(v) => v,
        };

        let mut r: Vec<MaybeUninit<T>> = Vec::new();
        if T::IS_SAFE_REPR {
            while len > 0 {
                let item_delta = std::cmp::min(MAX_ALLOC_SIZE / size_of::<T>(), len);

                // Make enough space for an extra `item_delta` items
                r.reserve(item_delta);
                let cur_len = r.len();

                unsafe {
                    // Pointer to the next free item
                    let next_free_ptr = r.as_mut_ptr().add(cur_len);

                    // Read the content for the next `item_delta` items
                    let dst_bytes = core::slice::from_raw_parts_mut(
                        next_free_ptr as *mut u8,
                        item_delta * size_of::<T>(),
                    );
                    if let Err(e) = src.read_exact(dst_bytes) {
                        return Err(SpError::ReadFailed(e));
                    }
                    #[cfg(feature = "verbose")]
                    ::log::debug!("  read({})", dst_bytes.len());

                    ctx.cursor += dst_bytes.len();
                    r.set_len(cur_len + item_delta);

                    // Validate every item's content
                    for v in &mut r[cur_len..] {
                        <T>::validate_contents(ctx, v)?;
                    }
                }
                len -= item_delta;
            }
        } else {
            for _i in 0..len {
                r.push({
                    let mut v = MaybeUninit::uninit();
                    <T>::inner_from_reader(src, ctx, &mut v)?;
                    v
                });
            }
        }

        unsafe {
            dst.write(core::mem::transmute(r));
            Ok(dst.assume_init_mut())
        }
    }
}

//collection_read!(Vec<T>, push, T);
collection_read!(VecDeque<T>, push_back, T);
collection_read!(LinkedList<T>, push_back, T);
collection_read!(HashSet<K>, insert, K: Eq + Hash);
collection_read!(BTreeSet<K>, insert, K: Ord);
collection_read!(HashMap<K,V>, insert, K: Eq + Hash, V);
collection_read!(BTreeMap<K,V>, insert, K: Ord, V);
collection_read!(BinaryHeap<T>, push, T: Ord);
