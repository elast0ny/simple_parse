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

impl<T: SpRead> SpRead for Vec<T> {
    fn inner_from_reader<'a, R: Read + ?Sized>(
        src: &mut R,
        ctx: &mut SpCtx,
        dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError> {
        // Get the number of elements we must read
        let len = match ctx.len.take() {
            None => {
                let mut tmp = MaybeUninit::uninit();
                *<DefaultCountType>::inner_from_reader(src, ctx, &mut tmp)? as usize
            }
            Some(v) => v,
        };

        // The maximum number of items we can pre-allocate to respect MAX_ALLOC_SIZE
        let max_alloc_item_num: usize = std::cmp::max(MAX_ALLOC_SIZE / size_of::<T>(), 1);

        let mut r = Vec::<MaybeUninit<T>>::new();

        if T::IS_SAFE_REPR {
            let mut items_left = len;
            while items_left > 0 {
                let num_items = std::cmp::min(max_alloc_item_num, items_left);
                // Make sure our allocation can accomodate an extra `num_items`
                let old_len = r.len();
                r.reserve(num_items);

                // Cast our allocation into &mut [u8]
                let next_free_ptr = unsafe { r.get_unchecked_mut(old_len) } as *mut _ as *mut u8;
                let dst_bytes = unsafe {
                    core::slice::from_raw_parts_mut(
                        next_free_ptr as *mut u8,
                        num_items * size_of::<T>(),
                    )
                };

                // Read `num_items` items into our allocation
                if let Err(e) = src.read_exact(dst_bytes) {
                    return Err(SpError::ReadFailed(e));
                }
                #[cfg(feature = "verbose")]
                ::log::debug!("  read({})", dst_bytes.len());

                ctx.cursor += dst_bytes.len();
                unsafe {
                    r.set_len(old_len + num_items);
                }

                // Validate every item's content
                for v in r.iter_mut().skip(old_len) {
                    unsafe { <T>::validate_contents(ctx, v)? };
                }

                items_left -= num_items;
            }
        } else {
            for i in 0..len {
                if i >= r.capacity() {
                    r.reserve(std::cmp::min(max_alloc_item_num, len - i));
                }
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

impl<T: SpRead> SpRead for VecDeque<T> {
    fn inner_from_reader<'a, R: Read + ?Sized>(
        src: &mut R,
        ctx: &mut SpCtx,
        dst: &'a mut MaybeUninit<Self>,
    ) -> Result<&'a mut Self, crate::SpError> {
        // Use the Vec<T> implementation
        let mut tmp = MaybeUninit::uninit();
        <Vec<T>>::inner_from_reader(src, ctx, &mut tmp)?;

        unsafe {
            dst.write(VecDeque::from(tmp.assume_init()));
            Ok(dst.assume_init_mut())
        }
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
collection_read!(LinkedList<T>, push_back, T);
collection_read!(HashSet<K>, insert, K: Eq + Hash);
collection_read!(BTreeSet<K>, insert, K: Ord);
collection_read!(HashMap<K,V>, insert, K: Eq + Hash, V);
collection_read!(BTreeMap<K,V>, insert, K: Ord, V);
collection_read!(BinaryHeap<T>, push, T: Ord);
