use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};

use std::cmp::Eq;
use std::hash::Hash;
use std::convert::TryInto;

use crate::{SpRead, SpReadRaw, SpReadRawMut, SpWrite};
use std::io::Cursor;

/* Vec */

/// From reader
macro_rules! vec_read {
    ($parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let num_items = match $count {
            Some(c) => c,
            None => {
                // Read prepended size
                <u64>::$parse_func($src, true, None)?.try_into().unwrap()
            }
        };

        if num_items == 0 {
            return Ok(Vec::new());
        }

        let mut val: Vec<T> = Vec::with_capacity(num_items);

        for _ in 0..num_items {
            val.push(<T>::$parse_func($src, $is_input_le, None)?)
        }

        Ok(val)
    }};
}

/// Into writer
macro_rules! vec_SpWrite {
    ($self:ident, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        let mut total_sz = 0;
        // Write size as u64
        if $prepend_count {
            // Use default settings for inner types
            total_sz += ($self.len() as u64).inner_to_writer(true, true, $dst)?;
        }

        for val in $self.iter() {
            total_sz += val.inner_to_writer($is_output_le, $prepend_count, $dst)?;
        }
        
        Ok(total_sz)
    }};
}
impl_SpRead!(Vec<T>, vec_read, T);
impl_SpReadRaw!(Vec<T>, vec_read, T);
impl_SpReadRawMut!(Vec<T>, vec_read, T);
impl_SpWrite!(Vec<T>, vec_SpWrite, T);


/* HashSet */
/// From reader
macro_rules! hashset_read {
    ($parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let num_items = match $count {
            None => panic!("HashSet must be annotated with #[sp(count=\"field_name\")]"),
            Some(c) => c,
        };

        let mut val = HashSet::with_capacity(num_items);

        for _ in 0..num_items {
            val.insert(<T>::$parse_func($src, $is_input_le, None)?);
        }

        Ok(val)
    }};
}
/// Into writer
macro_rules! hashset_SpWrite {
    ($self:ident, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        let mut total_sz = 0;
        let _ = $is_output_le;
        // Write size as u64
        if $prepend_count {
            // Use default settings for inner types
            total_sz += ($self.len() as u64).inner_to_writer(true, true, $dst)?;
        }

        for tmp in $self.iter() {
            total_sz += tmp.inner_to_writer($is_output_le, $prepend_count, $dst)?;
        }
        Ok(total_sz)
    }};
}
impl_SpRead!(HashSet<T>, hashset_read, T: Eq + Hash);
impl_SpReadRaw!(HashSet<T>, hashset_read, T: Eq + Hash);
impl_SpReadRawMut!(HashSet<T>, hashset_read, T: Eq + Hash);
impl_SpWrite!(HashSet<T>, hashset_SpWrite, T);

/* HashMap */
/// From reader
macro_rules! hashmap_read {
    ($parse_func:ident, $src:expr, $is_input_le:expr, $count:expr) => {{
        let num_items = match $count {
            None => panic!("HashMap must be annotated with #[sp(count=\"field_name\")]"),
            Some(c) => c,
        };

        let mut val = HashMap::with_capacity(num_items);

        for _ in 0..num_items {
            val.insert(
                <K>::$parse_func($src, $is_input_le, None)?,
                <V>::$parse_func($src, $is_input_le, None)?,
            );
        }

        Ok(val)
    }};
}
/// Into writer
macro_rules! hashmap_SpWrite {
    ($self:ident, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
        let mut total_sz = 0;
        let _ = $is_output_le;
        // Write size as u64
        if $prepend_count {
            // Use default settings for inner types
            total_sz += ($self.len() as u64).inner_to_writer(true, true, $dst)?;
        }

        for (k, v) in $self.iter() {
            total_sz += k.inner_to_writer($is_output_le, $prepend_count, $dst)?;
            total_sz += v.inner_to_writer($is_output_le, $prepend_count, $dst)?;
        }
        Ok(total_sz)
    }};
}
impl_SpRead!(HashMap<K,V>, hashmap_read, K: Eq + Hash, V);
impl_SpReadRaw!(HashMap<K,V>, hashmap_read, K: Eq + Hash, V);
impl_SpReadRawMut!(HashMap<K,V>, hashmap_read, K: Eq + Hash, V);
impl_SpWrite!(HashMap<K,V>, hashmap_SpWrite, K: Eq + Hash, V);