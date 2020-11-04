use std::collections::{HashMap, HashSet};
use crate::{SpRead, SpWrite};

impl<'a, T: SpRead<'a>> SpRead<'a> for Vec<T> {
    fn inner_from_bytes(
        input: &'a [u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&'a [u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        let num_items = match count {
            None => panic!("Called Vec<T>::from_byte() but no count field specified for the Vec ! Did you forget to annotate the Vec with #[sp(count=\"<field>\")]"),
            Some(c) => c,
        };

        let mut res = Vec::with_capacity(num_items);

        let mut rest = input;
        for _ in 0..num_items {
            let r = <T>::inner_from_bytes(rest, is_input_le, None)?;
            rest = r.0;

            res.push(r.1);
        }

        Ok((rest, res))
    }

    fn from_bytes(input: &'a [u8]) -> Result<(&'a [u8], Self), crate::SpError>
    where
        Self: Sized,
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


impl<'a, T: SpRead<'a> + std::hash::Hash + std::cmp::Eq> SpRead<'a> for HashSet<T> {
    fn inner_from_bytes(
        input: &'a [u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&'a [u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        let num_items = match count {
            None => panic!("Called HashSet<T>::from_byte() but no count field specified ! Did you forget to annotate with #[sp(count=\"<field>\")] ?"),
            Some(c) => c,
        };

        let mut res = HashSet::with_capacity(num_items);

        let mut rest = input;
        for _ in 0..num_items {
            let r = <T>::inner_from_bytes(rest, is_input_le, None)?;
            rest = r.0;

            res.insert(r.1);
        }

        Ok((rest, res))
    }

    fn from_bytes(input: &'a [u8]) -> Result<(&'a [u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        Self::inner_from_bytes(input, true, None)
    }
}

impl<T: SpWrite> SpWrite for HashSet<T> {
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


impl<'a, K: SpRead<'a> + std::hash::Hash + std::cmp::Eq, V: SpRead<'a> + std::hash::Hash + std::cmp::Eq> SpRead<'a> for HashMap<K,V> {
    fn inner_from_bytes(
        input: &'a [u8],
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<(&'a [u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        let num_items = match count {
            None => panic!("Called HashMap<K,V>::from_byte() but no count field specified ! Did you forget to annotate with #[sp(count=\"<field>\")] ?"),
            Some(c) => c,
        };

        let mut res = HashMap::with_capacity(num_items);

        let mut rest = input;
        for _ in 0..num_items {
            let k = <K>::inner_from_bytes(rest, is_input_le, None)?;
            rest = k.0;
            let v = <V>::inner_from_bytes(rest, is_input_le, None)?;
            rest = v.0;

            res.insert(k.1, v.1);
        }

        Ok((rest, res))
    }

    fn from_bytes(input: &'a [u8]) -> Result<(&'a [u8], Self), crate::SpError>
    where
        Self: Sized,
    {
        Self::inner_from_bytes(input, true, None)
    }
}

impl<K: SpWrite, V: SpWrite> SpWrite for HashMap<K,V> {
    fn inner_to_bytes(
        &self,
        is_output_le: bool,
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::SpError> {
        let mut total_sz = 0;
        for (k, v) in self.iter() {
            total_sz += k.inner_to_bytes(is_output_le, dst)?;
            total_sz += v.inner_to_bytes(is_output_le, dst)?;
        }
        Ok(total_sz)
    }
    fn to_bytes(&self, dst: &mut Vec<u8>) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}