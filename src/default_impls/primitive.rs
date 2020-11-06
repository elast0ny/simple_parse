use crate::{SpError, SpRead, SpReadRaw, SpReadRawMut, SpWrite};
use std::convert::TryInto;
use std::io::{Cursor, Read, Write};

// Implements :
//      Self        SpRead
//      Self        SpReadRaw
//      &Self       SpReadRaw
//      Self        SpReadRawMut
//      &Self       SpReadRawMut
//      &mut Self   SpReadRawMut
//      Self        SpWrite
//      &Self       SpWrite
//      &mut Self   SpWrite
macro_rules! ImplSpTraits {
    ($typ:ty) => {
        // Self from reader
        impl SpRead for $typ {
            fn inner_from_reader<R: Read + ?Sized>(
                src: &mut R,
                is_input_le: bool,
                _count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                // Create dst
                let mut val_dst = <$typ>::default();
                let dst = unsafe {
                    std::slice::from_raw_parts_mut(
                        (&mut val_dst) as *mut $typ as *mut u8,
                        std::mem::size_of::<$typ>(),
                    )
                };

                // Read into dst
                if src.read(dst).is_err() {
                    return Err(SpError::NotEnoughSpace);
                }

                // Convert endianness if needed
                let val = if is_input_le {
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
            }
            fn from_reader<R: Read + ?Sized>(src: &mut R) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                Self::inner_from_reader(src, true, None)
            }
        }
        // Self from bytes
        impl<'b> SpReadRaw<'b> for $typ {
            fn inner_from_slice(
                src: &mut Cursor<&'b [u8]>,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                let val_ref = <&Self>::inner_from_slice(src, is_input_le, count)?;
                
                let val = if is_input_le {
                    val_ref.to_le()
                } else {
                    val_ref.to_be()
                };

                // Get reference and deref it to get the value
                Ok(val)
            }
            fn from_slice(src: &mut Cursor<&'b [u8]>) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                <Self>::inner_from_slice(src, true, None)
            }
        }
        // Reference to Self from bytes
        impl<'b> SpReadRaw<'b> for &'b $typ {
            fn inner_from_slice(
                src: &mut Cursor<&'b [u8]>,
                _is_input_le: bool,
                _count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                let idx = src.position();
                // Size check
                let bytes = src.get_ref();
                if idx + std::mem::size_of::<$typ>() as u64 > bytes.len() as u64 {
                    return Err(SpError::NotEnoughSpace);
                }
                // Cast to reference
                let val = unsafe { &*(bytes.as_ptr().add(idx.try_into().unwrap()) as *const $typ) };
                // Move cursor forward
                src.set_position(idx + std::mem::size_of::<$typ>() as u64);
                Ok(val)
            }
            fn from_slice(src: &mut Cursor<&'b [u8]>) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                <Self>::inner_from_slice(src, true, None)
            }
        }
        // Self from mut bytes
        impl<'b> SpReadRawMut<'b> for $typ {
            fn inner_from_mut_slice(
                src: &mut Cursor<&'b mut [u8]>,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                // Get reference and deref it to get the value
                Ok(*(<&mut Self>::inner_from_mut_slice(src, is_input_le, count)?))
            }
            fn from_mut_slice(src: &mut Cursor<&'b mut [u8]>) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                <Self>::inner_from_mut_slice(src, true, None)
            }
        }
        // Mutatble reference to Self from mut bytes
        impl<'b> SpReadRawMut<'b> for &'b mut $typ {
            fn inner_from_mut_slice(
                src: &mut Cursor<&'b mut [u8]>,
                _is_input_le: bool,
                _count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                let idx = src.position();
                // Size check
                let bytes = src.get_ref();
                if idx + std::mem::size_of::<$typ>() as u64 > bytes.len() as u64 {
                    return Err(SpError::NotEnoughSpace);
                }
                // Cast to reference
                let val =
                    unsafe { &mut *(bytes.as_ptr().add(idx.try_into().unwrap()) as *mut $typ) };
                // Move cursor forward
                src.set_position(idx + std::mem::size_of::<$typ>() as u64);
                Ok(val)
            }
            fn from_mut_slice(src: &mut Cursor<&'b mut [u8]>) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                <Self>::inner_from_mut_slice(src, true, None)
            }
        }
        // Reference to Self from mut bytes
        impl<'b> SpReadRawMut<'b> for &'b $typ {
            fn inner_from_mut_slice(
                src: &mut Cursor<&'b mut [u8]>,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                Ok(<&mut $typ>::inner_from_mut_slice(src, is_input_le, count)?)
            }
            fn from_mut_slice(src: &mut Cursor<&'b mut [u8]>) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                <Self>::inner_from_mut_slice(src, true, None)
            }
        }
        // Write Self into writer
        impl SpWrite for $typ {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                _prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                let value = if is_output_le {
                    self.to_le_bytes()
                } else {
                    self.to_be_bytes()
                };
                let bytes = value.as_ref();
                match dst.write(bytes) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(SpError::NotEnoughSpace),
                }
            }

            fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
                self.inner_to_writer(true, true, dst)
            }
        }
        // Write &Self into writer
        impl SpWrite for &$typ {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                (**self).inner_to_writer(is_output_le, prepend_count, dst)
            }

            fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
                self.inner_to_writer(true, true, dst)
            }
        }
        // Write &mut Self into writer
        impl SpWrite for &mut $typ {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                (**self).inner_to_writer(is_output_le, prepend_count, dst)
            }

            fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
                self.inner_to_writer(true, true, dst)
            }
        }
    };
}

ImplSpTraits!(u8);
ImplSpTraits!(u16);
ImplSpTraits!(u32);
ImplSpTraits!(u64);
ImplSpTraits!(u128);
ImplSpTraits!(usize);
ImplSpTraits!(i8);
ImplSpTraits!(i16);
ImplSpTraits!(i32);
ImplSpTraits!(i64);
ImplSpTraits!(i128);
ImplSpTraits!(isize);

/* bool */
macro_rules! bool_read {
    ($parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let val = u8::$parse_func($src, $is_input_le, $count)?;
        Ok(val != 0)
    }};
}
// Write bool into writer
macro_rules! bool_SpWrite {
    ($self:ident, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        let val = if *$self { 1u8 } else { 0u8 };
        val.inner_to_writer($is_output_le, $prepend_count, $dst)
    }};
}
impl_SpRead!(bool, bool_read);
impl_SpReadRaw!(bool, bool_read);
impl_SpReadRawMut!(bool, bool_read);
impl_SpWrite!(bool, bool_SpWrite);

/* Slices */
macro_rules! slice_SpReadRaw {
    ($parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        // Either count is passed to us or we use prepended value before contents
        let num_items = match $count {
            Some(v) => v as u64,
            None => {
                <u64>::$parse_func($src, $is_input_le, $count)?
            }
        };
        let sz_needed = num_items * std::mem::size_of::<T>() as u64;
        let idx = $src.position();
        // Size check
        let bytes = $src.get_ref();
        if idx + sz_needed > bytes.len() as u64 {
            return Err(SpError::NotEnoughSpace);
        }
        let val = unsafe {
            std::slice::from_raw_parts(
                bytes.as_ptr().add(idx.try_into().unwrap()) as *const T,
                num_items.try_into().unwrap(),
            )
        };
        $src.set_position(idx + sz_needed);
        Ok(val)
    }};
}
macro_rules! slice_SpReadRawMut {
    ($parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        // Either count is passed to us or we use prepended value before contents
        let num_items = match $count {
            Some(v) => v as u64,
            None => {
                <u64>::$parse_func($src, $is_input_le, $count)?
            }
        };
        let sz_needed = num_items * std::mem::size_of::<T>() as u64;
        let idx = $src.position();
        // Size check
        let bytes = $src.get_mut();
        if idx + sz_needed > bytes.len() as u64 {
            return Err(SpError::NotEnoughSpace);
        }
        let val = unsafe {
            std::slice::from_raw_parts_mut(
                bytes.as_mut_ptr().add(idx.try_into().unwrap()) as *mut T,
                num_items.try_into().unwrap(),
            )
        };
        $src.set_position(idx + sz_needed);
        Ok(val)
    }};
}
macro_rules! slice_SpWrite {
    ($self:ident, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        let mut total_sz = 0;
        // Write size as u64
        if $prepend_count {
            // Use default settings for inner types
            total_sz += ($self.len() as u64).inner_to_writer(true, true, $dst)?;
        }

        for val in $self.iter() {
            total_sz += val.inner_to_writer($is_output_le, $prepend_count, $dst)?;
        }
        
        Ok(total_sz)
    }};
}

impl_SpReadRaw!(&'b [T], slice_SpReadRaw, T);
impl_SpReadRawMut!(&'b [T], slice_SpReadRawMut, T);
impl_SpReadRawMut!(&'b mut [T], slice_SpReadRawMut, T);
impl_SpWrite!([T], slice_SpWrite, T);