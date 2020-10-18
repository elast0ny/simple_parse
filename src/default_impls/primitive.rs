use crate::{SpError, SpRead, SpWrite};

macro_rules! ImplSpTraits {
    ($typ:ty) => {
        impl<'a> SpRead<'a> for $typ {
            fn inner_from_bytes(
                input: &'a [u8],
                is_input_le: bool,
                _count: Option<usize>,
            ) -> Result<(&'a [u8], Self), crate::SpError>
            where
                Self: Sized,
            {
                if input.len() < std::mem::size_of::<$typ>() {
                    return Err(SpError::NotEnoughBytes);
                }
                let (typ_bytes, rest) = input.split_at(std::mem::size_of::<$typ>());
                let typ_bytes = unsafe { &*(typ_bytes.as_ptr() as *const $typ) };
                let value = if is_input_le {
                    if cfg!(target_endian = "big") {
                        typ_bytes.swap_bytes()
                    } else {
                        *typ_bytes
                    }
                } else {
                    if cfg!(target_endian = "little") {
                        typ_bytes.swap_bytes()
                    } else {
                        *typ_bytes
                    }
                };
                Ok((rest, value))
            }

            fn from_bytes(input: &'a [u8]) -> Result<(&'a [u8], Self), crate::SpError>
            where
                Self: Sized,
            {
                Self::inner_from_bytes(input, true, None)
            }
        }

        impl SpWrite for $typ {
            fn inner_to_bytes(
                &self,
                is_output_le: bool,
                dst: &mut Vec<u8>,
            ) -> Result<usize, crate::SpError> {
                let value = if is_output_le {
                    self.to_le_bytes()
                } else {
                    self.to_be_bytes()
                };
                let bytes = value.as_ref();
                dst.extend(bytes);
                Ok(bytes.len())
            }

            fn to_bytes(&self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
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

impl<'a, T: SpRead<'a>> SpRead<'a> for *mut T {
    fn inner_from_bytes(
        input: &'a[u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&'a[u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        let (rest, res) = usize::inner_from_bytes(input, is_input_le, count)?;
        Ok((rest, res as *mut T))
    }

    fn from_bytes(input: &'a[u8]) -> Result<(&'a[u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        Self::inner_from_bytes(input, true, None)
    }
}

impl<T: SpWrite> SpWrite for *mut T {
    fn inner_to_bytes(
        &self,
        is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError> {
        let val = *self as usize;
        val.inner_to_bytes(is_output_le, dst)
    }
    fn to_bytes(&self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}
