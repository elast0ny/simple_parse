use crate::{SpError, SpRead, SpWrite};

impl<'b> SpRead<'b> for String {
    fn inner_from_bytes(
        input: &[u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&[u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        let mut rest = input;
        
        // Get the number of bytes for the string
        let res = <usize>::inner_from_bytes(rest, is_input_le, count)?;
        rest = res.0;
        let num_bytes = res.1;

        // Make sure theres enough data
        if num_bytes > rest.len() {
            return Err(SpError::NotEnoughBytes);
        }

        // Parse the bytes as utf8
        let res = match std::str::from_utf8(&rest[..num_bytes]) {
            Ok(s) => s.to_string(),
            Err(_e) => {
                return Err(SpError::InvalidBytes);
            }
        };
        rest = &rest[num_bytes..];

        Ok((rest, res))
    }

    fn from_bytes(input: &[u8]) -> Result<(&[u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        Self::inner_from_bytes(input, true, None)
    }
}

impl SpWrite for String {
    fn inner_to_bytes(
        &mut self,
        is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError> {
        
        // Write the number of bytes
        let mut total_sz = (self.len()).inner_to_bytes(is_output_le, dst)?;

        // Copy the string bytes
        dst.extend_from_slice(self.as_bytes());
        total_sz += self.len();

        Ok(total_sz)
    }
    fn to_bytes(&mut self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }

}


impl<'b> SpRead<'b> for &'b str {
    fn inner_from_bytes (
        input: &'b [u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&[u8], Self), crate::SpError>
    where
        Self: 'b + Sized,
    {
        let mut rest = input;
        
        // Get the number of bytes for the string
        let res = <usize>::inner_from_bytes(rest, is_input_le, count)?;
        rest = res.0;
        let num_bytes = res.1;

        // Make sure theres enough data
        if num_bytes > rest.len() {
            return Err(SpError::NotEnoughBytes);
        }

        // Parse the bytes as utf8
        let val = match std::str::from_utf8(&rest[..num_bytes]) {
            Ok(s) => s,
            Err(_e) => {
                return Err(SpError::InvalidBytes);
            }
        };
        rest = &rest[num_bytes..];

        Ok((rest, val))
    }

    fn from_bytes(input: &'b [u8]) -> Result<(&'b [u8], Self), crate::SpError>
    where
        Self: 'b + Sized,
    {
        Self::inner_from_bytes(input, true, None)
    }
}

impl SpWrite for &str {
    fn inner_to_bytes(
        &mut self,
        is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError> {
        
        // Write the number of bytes
        let mut total_sz = (self.len()).inner_to_bytes(is_output_le, dst)?;

        // Copy the string bytes
        dst.extend_from_slice(self.as_bytes());
        total_sz += self.len();

        Ok(total_sz)
    }
    fn to_bytes(&mut self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}