use crate::{SpError, SpRead, SpWrite};

macro_rules! ImplSpTraits {
    ($typ:ty) => {
        impl<'b> SpRead<'b> for $typ {
            fn inner_from_bytes(
                input: &'b [u8],
                is_input_le: bool,
                _count: Option<usize>,
            ) -> Result<(&'b [u8], Self), crate::SpError>
            where
                Self: Sized,
            {
                let input = input.as_ref();
                if input.len() < std::mem::size_of::<$typ>() {
                    return Err(SpError::NotEnoughBytes);
                }
                let (typ_bytes, rest) = input.as_ref().split_at(std::mem::size_of::<$typ>());
                // Safe because we checked the size above
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

            fn from_bytes(input: &'b [u8]) -> Result<(&'b [u8], Self), crate::SpError>
            where
                Self: Sized,
            {
                Self::inner_from_bytes(input, true, None)
            }
        }

        impl SpWrite for $typ {
            fn inner_to_bytes(
                &mut self,
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

            fn to_bytes(&mut self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
                self.inner_to_bytes(true, dst)
            }
        }

        impl<'b> SpRead<'b> for &$typ {
            fn inner_from_bytes(
                input: &'b [u8],
                _is_input_le: bool,
                _count: Option<usize>,
            ) -> Result<(&'b [u8], Self), crate::SpError>
            where
                Self: 'b + Sized,
            {
                let input = input.as_ref();
                if input.len() < std::mem::size_of::<$typ>() {
                    return Err(SpError::NotEnoughBytes);
                }
                let value = unsafe{&*(input.as_ptr() as *const $typ)};
                let (_, rest) = input.as_ref().split_at(std::mem::size_of::<$typ>());
                
                Ok((rest, value))
            }

            fn from_bytes(input: &'b [u8]) -> Result<(&'b [u8], Self), crate::SpError>
            where
                Self: 'b + Sized,
            {
                Self::inner_from_bytes(input, true, None)
            }
        }

        impl SpWrite for &$typ {
            fn inner_to_bytes(
                &mut self,
                is_output_le: bool,
                dst: &mut Vec<u8>,
            ) -> Result<usize, crate::SpError> {
                let mut v = **self;
                v.inner_to_bytes(is_output_le, dst)
            }

            fn to_bytes(&mut self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
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

impl<'b, T: SpRead<'b>> SpRead<'b> for *mut T {
    fn inner_from_bytes(
        input: &'b [u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&'b [u8], Self), crate::SpError>
    where
        Self: 'b + Sized,
    {
        let (rest, res) = usize::inner_from_bytes(input, is_input_le, count)?;
        Ok((rest, res as *mut T))
    }

    fn from_bytes(input: &'b [u8]) -> Result<(&'b [u8], Self), crate::SpError>
    where
        Self: 'b + Sized,
    {
        Self::inner_from_bytes(input, true, None)
    }
}

impl<T: SpWrite> SpWrite for *mut T {
    fn inner_to_bytes(
        &mut self,
        is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError> {
        let mut val = *self as usize;
        val.inner_to_bytes(is_output_le, dst)
    }
    fn to_bytes(&mut self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}