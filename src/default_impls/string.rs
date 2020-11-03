use crate::{SpError, SpRead, SpWrite};

use std::ffi::{CStr, CString};

impl<'a> SpRead<'a> for String {
    fn inner_from_bytes(
        input: &'a [u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&'a [u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        let mut rest = input;

        // Get the number of bytes for the string
        let res = <u64>::inner_from_bytes(rest, is_input_le, count)?;
        rest = res.0;
        let num_bytes = res.1;

        // Make sure theres enough data
        if num_bytes as usize > rest.len() {
            return Err(SpError::NotEnoughBytes);
        }

        // Parse the bytes as utf8
        let res = match std::str::from_utf8(&rest[..num_bytes as usize]) {
            Ok(s) => s.to_string(),
            Err(_e) => {
                return Err(SpError::InvalidBytes);
            }
        };
        rest = &rest[num_bytes as usize..];

        Ok((rest, res))
    }

    fn from_bytes(input: &'a [u8]) -> Result<(&'a [u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        Self::inner_from_bytes(input, true, None)
    }
}

impl SpWrite for String {
    fn inner_to_bytes(
        &self,
        _is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError> {
        self.as_str().inner_to_bytes(true, dst)
    }
    fn to_bytes(&self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}

impl SpWrite for &str {
    fn inner_to_bytes(
        &self,
        is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError> {
        // Write string lenght as u64
        let mut total_sz = (self.len() as u64).inner_to_bytes(is_output_le, dst)?;

        // Copy the string bytes
        dst.extend_from_slice(self.as_bytes());
        total_sz += self.len();

        Ok(total_sz)
    }
    fn to_bytes(&self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}

impl<'a> SpRead<'a> for CString {
    fn inner_from_bytes(
        input: &'a [u8],
        _is_input_le: bool,
        _count: Option<usize>,
    ) -> Result<(&'a [u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        let mut rest = input;

        let mut null_byte_idx = None;
        for (idx, b) in rest.iter().enumerate() {
            if *b == 0x00 {
                null_byte_idx = Some(idx);
                break;
            }
        }

        let res = if let Some(idx) = null_byte_idx {
            rest = &rest[idx + 1..];
            CString::new(&rest[..idx]).unwrap()
        } else {
            // No null terminator found
            return Err(SpError::InvalidBytes);
        };

        Ok((rest, res))
    }

    fn from_bytes(input: &'a [u8]) -> Result<(&'a [u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        Self::inner_from_bytes(input, true, None)
    }
}

impl SpWrite for CString {
    fn inner_to_bytes(
        &self,
        _is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError> {
        self.as_c_str().inner_to_bytes(true, dst)
    }
    fn to_bytes(&self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}

impl SpWrite for &CStr {
    fn inner_to_bytes(
        &self,
        _is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError> {
        let contents = self.to_bytes_with_nul();

        // Copy the string bytes
        dst.extend_from_slice(contents);

        Ok(contents.len())
    }
    fn to_bytes(&self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}
