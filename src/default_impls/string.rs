use std::convert::TryInto;
use std::ffi::{CStr, CString};
use std::io::{Cursor, Read, Write};

use crate::{SpError, SpRead, SpReadRaw, SpReadRawMut, SpWrite};

/// &str from bytes
impl<'b> SpReadRaw<'b> for &'b str {
    fn inner_from_slice(
        src: &mut Cursor<&'b [u8]>,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        // Get slice
        let slice = <&[u8]>::inner_from_slice(src, is_input_le, count)?;

        let val = match std::str::from_utf8(slice) {
            Ok(v) => v,
            Err(_) => return Err(SpError::InvalidBytes),
        };

        Ok(val)
    }
    fn from_slice(src: &mut Cursor<&'b [u8]>) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        <Self>::inner_from_slice(src, true, None)
    }
}
/// &str from mut bytes
impl<'b> SpReadRawMut<'b> for &'b str {
    fn inner_from_mut_slice(
        src: &mut Cursor<&'b mut [u8]>,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        // Get slice
        let slice = <&[u8]>::inner_from_mut_slice(src, is_input_le, count)?;

        let val = match std::str::from_utf8(slice) {
            Ok(v) => v,
            Err(_) => return Err(SpError::InvalidBytes),
        };

        Ok(val)
    }
    fn from_mut_slice(src: &mut Cursor<&'b mut [u8]>) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        <Self>::inner_from_mut_slice(src, true, None)
    }
}
/// Write &str into writer
impl SpWrite for &str {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        is_output_le: bool,
        prepend_count: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.as_bytes().inner_to_writer(is_output_le, prepend_count, dst)
    }
    fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_writer(true, true, dst)
    }
}

/// String from Reader
impl SpRead for String {
    fn inner_from_reader<R: Read + ?Sized>(
        src: &mut R,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        // Read string contents as a Vec<u8>
        let vec = <Vec<u8>>::inner_from_reader(src, is_input_le, count)?;

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
    fn from_reader<R: Read + ?Sized>(src: &mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        Self::inner_from_reader(src, true, None)
    }
}
/// String from bytes
impl<'b> SpReadRaw<'b> for String {
    fn inner_from_slice(
        src: &mut Cursor<&'b [u8]>,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        // Get &str
        let s = <&str>::inner_from_slice(src, is_input_le, count)?;
        Ok(s.to_string())
    }
    fn from_slice(src: &mut Cursor<&'b [u8]>) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        <Self>::inner_from_slice(src, true, None)
    }
}
/// String from mut bytes
impl<'b> SpReadRawMut<'b> for String {
    fn inner_from_mut_slice(
        src: &mut Cursor<&'b mut [u8]>,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        // Get &str
        let s = <&str>::inner_from_mut_slice(src, is_input_le, count)?;
        Ok(s.to_string())
    }
    fn from_mut_slice(src: &mut Cursor<&'b mut [u8]>) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        <Self>::inner_from_mut_slice(src, true, None)
    }
}
/// Write String to writer
impl SpWrite for String {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        _is_output_le: bool,
        prepend_count: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.as_str().inner_to_writer(true, prepend_count, dst)
    }
    fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_writer(true, true, dst)
    }
}
/// Write &String to writer
impl SpWrite for &String {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        _is_output_le: bool,
        prepend_count: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.as_str().inner_to_writer(true, prepend_count, dst)
    }
    fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_writer(true, true, dst)
    }
}
/// Write &mut String to writer
impl SpWrite for &mut String {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        _is_output_le: bool,
        prepend_count: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.as_str().inner_to_writer(true, prepend_count, dst)
    }
    fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_writer(true, true, dst)
    }
}

macro_rules! readraw_cstr {
    ($src:expr) => {{
        let bytes = $src.get_ref();
        let start_idx = $src.position().try_into().unwrap();
        let mut idx = start_idx;

        // Look for the next \0
        let mut cstr_bytes = None;
        while idx < bytes.len() {
            if *unsafe { bytes.get_unchecked(idx) } == 0 {
                // Create slice that includes \0
                cstr_bytes = Some(unsafe {
                    std::slice::from_raw_parts(bytes.as_ptr().add(start_idx), (idx - start_idx) + 1)
                });
                break;
            }
            idx += 1;
        }

        match cstr_bytes {
            Some(bytes) => {
                $src.set_position((start_idx + bytes.len()) as u64);
                Ok(unsafe { CStr::from_bytes_with_nul_unchecked(bytes) })
            }
            None => {
                // Ran out of bytes before finding \0
                Err(SpError::NotEnoughSpace)
            }
        }
    }};
}

/// &CStr from bytes
impl<'b> SpReadRaw<'b> for &'b CStr {
    fn inner_from_slice(
        src: &mut Cursor<&'b [u8]>,
        _is_input_le: bool,
        _count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        readraw_cstr!(src)
    }
    fn from_slice(src: &mut Cursor<&'b [u8]>) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        <Self>::inner_from_slice(src, true, None)
    }
}
/// &CStr from mut bytes
impl<'b> SpReadRawMut<'b> for &'b CStr {
    fn inner_from_mut_slice(
        src: &mut Cursor<&'b mut [u8]>,
        _is_input_le: bool,
        _count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        readraw_cstr!(src)
    }
    fn from_mut_slice(src: &mut Cursor<&'b mut [u8]>) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        <Self>::inner_from_mut_slice(src, true, None)
    }
}
/// Write &CStr to writer
impl SpWrite for &CStr {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        _is_output_le: bool,
        _prepend_count: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        let contents = self.to_bytes_with_nul();

        // Copy the string bytes
        match dst.write(contents) {
            Ok(v) => Ok(v),
            Err(_) => Err(SpError::NotEnoughSpace),
        }
    }
    fn to_writer<'a, W: Write + ?Sized>(&self, dst: &'a mut W) -> Result<usize, crate::SpError> {
        self.inner_to_writer(true, true, dst)
    }
}

/// Read CString from reader
impl SpRead for CString {
    fn inner_from_reader<R: Read + ?Sized>(
        src: &mut R,
        is_input_le: bool,
        _count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        // We dont know the size yet
        let mut vec = Vec::new();

        // Consume bytes until we hit null
        loop {
            let val = <u8>::inner_from_reader(src, is_input_le, None)?;
            vec.push(val);
            if val == 0 {
                break;
            }
        }

        Ok(unsafe { CString::from_vec_unchecked(vec) })
    }
    /// Convert arbitrary bytes to Self
    fn from_reader<R: Read + ?Sized>(src: &mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized,
    {
        Self::inner_from_reader(src, true, None)
    }
}
/// Write CString to writer
impl SpWrite for CString {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        _is_output_le: bool,
        prepend_count: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.as_c_str().inner_to_writer(true, prepend_count, dst)
    }
    fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_writer(true, true, dst)
    }
}
/// Write &CString to writer
impl SpWrite for &CString {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        _is_output_le: bool,
        _prepend_count: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.as_c_str().inner_to_writer(true, true, dst)
    }
    fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_writer(true, true, dst)
    }
}
/// Write &mut CString to writer
impl SpWrite for &mut CString {
    fn inner_to_writer<W: Write + ?Sized>(
        &self,
        _is_output_le: bool,
        _prepend_count: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        self.as_c_str().inner_to_writer(true, true, dst)
    }
    fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_writer(true, true, dst)
    }
}