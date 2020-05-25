mod error;
pub use error::*;

pub use simple_parse_derive::*;
pub trait SpRead {
    fn inner_from_bytes(
        input: &[u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&[u8], Self), crate::SpError>
    where
        Self: Sized;

    /// Convert arbitrary bytes to Self
    fn from_bytes(input: &[u8]) -> Result<(&[u8], Self), crate::SpError>
    where
        Self: Sized;
}

pub trait SpWrite {
    fn inner_to_bytes(&mut self, is_output_le: bool, dst: &mut Vec<u8>) -> Result<(), crate::SpError>;

    /// Convert the current contents of the struct to bytes.
    /// This function potentially changes the content of self and
    /// can fail.
    fn to_bytes(&mut self, dst: &mut Vec<u8>) -> Result<(), crate::SpError>;
}

macro_rules! ImplSpTraits {
    ($typ:ty) => {
        impl SpRead for $typ {
            fn inner_from_bytes(
                input: &[u8],
                is_input_le: bool,
                _count: Option<usize>,
            ) -> Result<(&[u8], Self), crate::SpError>
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

            fn from_bytes(input: &[u8]) -> Result<(&[u8], Self), crate::SpError>
            where
                Self: Sized,
            {
                Self::inner_from_bytes(input, true, None)
            }
        }

        impl SpWrite for $typ {
            fn inner_to_bytes(&mut self, is_output_le: bool, dst: &mut Vec<u8>) -> Result<(), crate::SpError> {
                let value = if is_output_le {
                    self.to_le_bytes()
                } else {
                    self.to_be_bytes()
                };

                dst.extend(value.as_ref());
                Ok(())
            }

            fn to_bytes(&mut self, dst: &mut Vec<u8>) -> Result<(), crate::SpError> {
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

impl<T: SpRead> SpRead for Vec<T> {
    fn inner_from_bytes(
        input: &[u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&[u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        let mut rest = input;
        let num_items = match count {
            None => panic!("Called Vec<T>::from_byte() but no count field specified for the Vec ! Did you forget to annotate the Vec with #[sp(count=\"<field>\")]"),
            Some(c) => c,
        };

        let mut res = Vec::with_capacity(num_items);

        for _ in 0..num_items {
            let r = <T>::inner_from_bytes(rest, is_input_le, None)?;
            rest = r.0;

            res.push(r.1);
        }

        Ok((rest, res))
    }

    fn from_bytes(input: &[u8]) -> Result<(&[u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        Self::inner_from_bytes(input, true, None)
    }
}

impl<T: SpWrite> SpWrite for Vec<T> {
    fn inner_to_bytes(&mut self, is_output_le: bool, dst: &mut Vec<u8>) -> Result<(), crate::SpError> {
        for tmp in self.iter_mut() {
            tmp.inner_to_bytes(is_output_le, dst)?;
        }
        Ok(())
    }
    fn to_bytes(&mut self, dst: &mut Vec<u8>) -> Result<(), crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}
