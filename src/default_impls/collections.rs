use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::alloc::{alloc, Layout};

use crate::{SpRead, SpWrite};

// Vec
impl<T: SpRead> SpRead for Vec<T> {
    fn inner_from_bytes<R: Read + ?Sized> (
        src: &mut R,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized {
            let num_items = match count {
                None => panic!("Called Vec<T>::from_byte() but no count field specified for the Vec ! Did you forget to annotate the Vec with #[sp(count=\"<field>\")]"),
                Some(c) => c,
            };

            if num_items == 0 {
                return Ok(Vec::new());
            }
            
            // Pre-alloc uninitialized memory for speed
            let mut val: Vec<T> = unsafe {
                let bytes = alloc(Layout::array::<T>(num_items).unwrap()) as *mut T;
                Vec::from_raw_parts(bytes, 0, num_items)
            };

            for _ in 0..num_items {
                val.push(<T>::inner_from_bytes(src, is_input_le, None)?)
            } 

            Ok(val)
        }

    /// Convert arbitrary bytes to Self
    fn from_bytes<R: Read + ?Sized>(_src: &mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized {
            panic!("SpRead::inner_from_bytes() must be used for collections to specify an item count");
        }
}
impl<T: SpWrite> SpWrite for Vec<T> {
    fn inner_to_bytes<W: Write + ?Sized>(
        &self,
        is_output_le: bool,
        dst: &mut W,
    ) -> Result<usize, crate::SpError> {
        let mut total_sz = 0;
        for tmp in self.iter() {
            total_sz += tmp.inner_to_bytes(is_output_le, dst)?;
        }
        Ok(total_sz)
    }

    fn to_bytes<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}

// HashSet
impl<T: SpRead + std::hash::Hash + std::cmp::Eq> SpRead for HashSet<T> {
    fn inner_from_bytes<'a, R: Read + ?Sized> (
        src: &'a mut R,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized {
            let num_items = match count {
                None => panic!("Called HashSet<T>::from_byte() but no count field specified ! Did you forget to annotate with #[sp(count=\"<field>\")] ?"),
                Some(c) => c,
            };
    
            let mut val = HashSet::with_capacity(num_items);
    
            for _ in 0..num_items {
                val.insert(<T>::inner_from_bytes(src, is_input_le, None)?);
            }
    
            Ok(val)
        }

    /// Convert arbitrary bytes to Self
    fn from_bytes<'a, R: Read + ?Sized>(_src: &'a mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized {
            panic!("SpRead::inner_from_bytes() must be used for collections to specify an item count");
        }
}
impl<T: SpWrite> SpWrite for HashSet<T> {
    fn inner_to_bytes<'a, W: Write + ?Sized>(
        &self,
        is_output_le: bool,
        dst: &'a mut W,
    ) -> Result<usize, crate::SpError> {
        let mut total_sz = 0;
        for tmp in self.iter() {
            total_sz += tmp.inner_to_bytes(is_output_le, dst)?;
        }
        Ok(total_sz)
    }

    fn to_bytes<'a, W: Write + ?Sized>(&self, dst: &'a mut W) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}

// HashMap
impl<K: SpRead + std::hash::Hash + std::cmp::Eq, V: SpRead + std::hash::Hash + std::cmp::Eq> SpRead for HashMap<K,V> {
    fn inner_from_bytes<'a, R: Read + ?Sized> (
        src: &'a mut R,
        is_input_le: bool,
        count: Option<usize>,
    ) -> Result<Self, crate::SpError>
    where
        Self: Sized {
            let num_items = match count {
                None => panic!("Called HashSet<T>::from_byte() but no count field specified ! Did you forget to annotate with #[sp(count=\"<field>\")] ?"),
                Some(c) => c,
            };
    
            let mut val = HashMap::with_capacity(num_items);
    
            for _ in 0..num_items {
                val.insert(<K>::inner_from_bytes(src, is_input_le, None)?,<V>::inner_from_bytes(src, is_input_le, None)?);
            }
    
            Ok(val)
        }

    /// Convert arbitrary bytes to Self
    fn from_bytes<'a, R: Read + ?Sized>(_src: &'a mut R) -> Result<Self, crate::SpError>
    where
        Self: Sized {
            panic!("SpRead::inner_from_bytes() must be used for collections to specify an item count");
        }
}
impl<K: SpWrite + std::hash::Hash + std::cmp::Eq, V: SpWrite + std::hash::Hash + std::cmp::Eq> SpWrite for HashMap<K,V> {
    fn inner_to_bytes<'a, W: Write + ?Sized>(
        &self,
        is_output_le: bool,
        dst: &'a mut W,
    ) -> Result<usize, crate::SpError> {
        let mut total_sz = 0;
        for (k,v) in self.iter() {
            total_sz += k.inner_to_bytes(is_output_le, dst)?;
            total_sz += v.inner_to_bytes(is_output_le, dst)?;
        }
        Ok(total_sz)
    }

    fn to_bytes<'a, W: Write + ?Sized>(&self, dst: &'a mut W) -> Result<usize, crate::SpError> {
        self.inner_to_bytes(true, dst)
    }
}