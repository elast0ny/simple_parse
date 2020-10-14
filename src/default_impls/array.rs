use crate::{SpRead, SpWrite};

impl<'b, T: SpRead<'b>> SpRead<'b> for Vec<T> {
    fn inner_from_bytes(
        input: &'b [u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&'b [u8], Self), crate::SpError>
    where
        Self: 'b + Sized,
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

    fn from_bytes(input: &'b [u8]) -> Result<(&'b [u8], Self), crate::SpError>
    where
        Self: 'b + Sized,
    {
        Self::inner_from_bytes(input, true, None)
    }
}

impl<T: SpWrite> SpWrite for Vec<T> {
    fn inner_to_bytes(
        &self,
        is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError> {
        let mut total_sz = 0;
        for tmp in self.iter() {
            total_sz += tmp.inner_to_bytes(is_output_le, dst)?;
        }
        Ok(total_sz)
    }
    fn to_bytes(&self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}

impl<'b, T: 'b + SpRead<'b>> SpRead<'b> for &[T] {
    fn inner_from_bytes(
        input: &'b [u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&'b [u8], Self), crate::SpError>
    where
        Self: 'b + Sized
    {
        let mut rest = input;
        let num_items = match count {
            None => panic!("Called &[T]::from_byte() but no count field specified for the slice ! Did you forget to annotate the slice with #[sp(count=\"<field>\")]"),
            Some(c) => c,
        };

        let res = unsafe {std::slice::from_raw_parts(input.as_ptr() as *const T, num_items)};

        // Make sure every T is valid
        for _ in 0..num_items {
            rest = <T>::inner_from_bytes(rest, is_input_le, None)?.0;
        }

        Ok((rest, res))
    }

    fn from_bytes(input: &'b [u8]) -> Result<(&'b [u8], Self), crate::SpError>
    where
        Self: 'b + Sized,
    {
        Self::inner_from_bytes(input, true, None)
    }
}

impl<T: SpWrite> SpWrite for &[T] {
    fn inner_to_bytes(
        &self,
        is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError> {
        let mut total_sz = 0;
        for tmp in self.iter() {
            total_sz += tmp.inner_to_bytes(is_output_le, dst)?;
        }
        Ok(total_sz)
    }
    fn to_bytes(&self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}