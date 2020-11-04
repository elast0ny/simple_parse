use crate::{SpError, SpRead, SpWrite};
use std::io::{Read, Write};

macro_rules! ImplSpTraits {
    ($typ:ty) => {
        impl SpRead for $typ {
            fn inner_from_bytes<'a, R: Read + ?Sized>(
                src: &'a mut R,
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

            fn from_bytes<'a, R: Read + ?Sized>(src: &'a mut R) -> Result<Self, crate::SpError>
            where
                Self: Sized,
            {
                Self::inner_from_bytes(src, true, None)
            }
        }

        impl SpWrite for $typ {
            fn inner_to_bytes<'a, W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                dst: &'a mut W,
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

            fn to_bytes<'a, W: Write + ?Sized>(&self, dst: &'a mut W) -> Result<usize, crate::SpError> {
                self.inner_to_bytes(true, dst)
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

impl<T: SpRead> SpRead for *mut T {
    fn inner_from_bytes<R: Read + ?Sized>(
        src: &mut R,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        let val = usize::inner_from_bytes(src, is_input_le, count)?;
        Ok(val as *mut T)
    }

    fn from_bytes<R: Read + ?Sized>(src: &mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        Self::inner_from_bytes(src, true, None)
    }
}

impl<T: SpWrite> SpWrite for *mut T {
    fn inner_to_bytes<'a, W: Write + ?Sized>(
        &self,
        is_output_le: bool,
        dst: &'a mut W,
    ) -> Result<usize, crate::SpError> {
        let val = *self as usize;
        val.inner_to_bytes(is_output_le, dst)
    }
    fn to_bytes<'a, W: Write + ?Sized>(&self, dst: &'a mut W) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}

impl SpRead for bool {
    fn inner_from_bytes<R: Read + ?Sized>(
        src: &mut R,
        is_input_le: bool,
        _count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        let val = u8::inner_from_bytes(src, is_input_le, _count)?;
        Ok(val != 0)
    }

    fn from_bytes<R: Read + ?Sized>(src: &mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        Self::inner_from_bytes(src, true, None)
    }
}

impl SpWrite for bool {
    fn inner_to_bytes<W: Write + ?Sized>(
        &self,
        is_output_le: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        let val = if *self { 1u8 } else { 0u8 };
        val.inner_to_bytes(is_output_le, dst)
    }
    fn to_bytes<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}
