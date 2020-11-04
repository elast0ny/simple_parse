use std::ffi::{CStr, CString};
use std::io::{Read, Write};

use crate::{SpError, SpRead, SpWrite};

// String, &str
impl SpRead for String {
    fn inner_from_bytes<R: Read + ?Sized> (
        src: &mut R,
        is_input_le: bool,
        _count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized {

            // Get the number of bytes for the string
            let num_bytes = <u64>::inner_from_bytes(src, is_input_le, None)?;

            // Read string contents as a Vec<u8>
            let vec = <Vec<u8>>::inner_from_bytes(src, is_input_le, Some(num_bytes as _))?;
            
            // Parse the bytes as utf8
            let val = match String::from_utf8(vec) {
                Ok(s) => s,
                Err(_e) => {
                    return Err(SpError::InvalidBytes);
                }
            };

            Ok(val)
        }

    /// Convert arbitrary bytes to Self
    fn from_bytes<R: Read + ?Sized>(src: &mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized {
            Self::inner_from_bytes(src, true, None)
        }
}
impl SpWrite for String {
    fn inner_to_bytes<W: Write + ?Sized>(
        &self,
        _is_output_le: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.as_str().inner_to_bytes(true, dst)
    }
    fn to_bytes<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}
impl SpWrite for &str {
    fn inner_to_bytes<W: Write + ?Sized>(
        &self,
        is_output_le: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        // Write string lenght as u64
        let total_sz = (self.len() as u64).inner_to_bytes(is_output_le, dst)?;

        match dst.write(self.as_bytes()) {
            Ok(v) => Ok(total_sz + v),
            Err(_) => Err(SpError::NotEnoughSpace),
        }
    }
    fn to_bytes<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}


impl SpRead for CString {
    fn inner_from_bytes<R: Read + ?Sized> (
        src: &mut R,
        is_input_le: bool,
        _count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized {

            // We dont know the size yet
            let mut vec = Vec::new();

            // Consume bytes until we hit null
            loop {
                let val = <u8>::inner_from_bytes(src, is_input_le, None)?;
                vec.push(val);
                if val == 0 {
                    break;
                }
            }

            Ok(
                unsafe {
                    CString::from_vec_unchecked(vec)
                }
            )
        }

    /// Convert arbitrary bytes to Self
    fn from_bytes<R: Read + ?Sized>(src: &mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized {
            Self::inner_from_bytes(src, true, None)
        }
}
impl SpWrite for CString {
    fn inner_to_bytes<W: Write + ?Sized>(
        &self,
        _is_output_le: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.as_c_str().inner_to_bytes(true, dst)
    }
    fn to_bytes<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}
impl SpWrite for &CStr {
    fn inner_to_bytes<W: Write + ?Sized>(
        &self,
        _is_output_le: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        let contents = self.to_bytes_with_nul();

        // Copy the string bytes
        match dst.write(contents) {
            Ok(v) => Ok(v),
            Err(_) => Err(SpError::NotEnoughSpace),
        }
    }
    fn to_bytes<'a, W: Write + ?Sized>(&self, dst: &'a mut W) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}