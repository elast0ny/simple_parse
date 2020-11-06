/// Writes [T] into dst
#[macro_use]
macro_rules! slice_to_writer {
    ($self:ident $(as $as_typ:ty)?, $is_output_le:ident, $prepend_count:ident, $dst: ident) => {{
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

/// Implements SpRead for $typ
/// 
/// $typ
///     Type that the trait will be implemented on
/// $reader($typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr)
///     Macro that fills the body of the inner_from_slice() trait
///     $reader == inner_from_slice
/// $generics
///     Optionnal generics that the $typ relies on
#[macro_use]
macro_rules! impl_SpRead {
    ($typ:ty, $inner_typ:ty, $reader:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)*)*) => {
        impl<$($generics : SpRead $(+ $bound$(+ $other)*)*),*> SpRead for $typ {
            fn inner_from_reader<R: Read + ?Sized> (
                src: &mut R,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized {
                    $reader!($typ, $inner_typ, inner_from_reader, src, is_input_le, count)
                }

            fn from_reader<R: Read + ?Sized>(_src: &mut R) -> Result<Self, crate::SpError>
            where
                Self: Sized {
                    panic!("SpRead::inner_from_reader() must be used for collections to specify an item count");
                }
        }
    }
}

/// Implements SpReadRaw for $typ
/// 
/// $typ
///     Type that the trait will be implemented on
/// $reader($typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr)
///     Macro that fills the body of the inner_from_slice() trait
///     $reader == inner_from_slice
/// $generics
///     Optionnal generics that the $typ relies on
#[macro_use]
macro_rules! impl_SpReadRaw {
    ($typ:ty, $inner_typ:ty, $reader:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)*)*) => {
        impl<'b, $($generics : SpReadRaw<'b> $(+ $bound$(+ $other)*)*),*> SpReadRaw<'b> for $typ {
            fn inner_from_slice(
                src: &mut Cursor<&'b [u8]>,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized {
                    $reader!($typ, $inner_typ, inner_from_slice, src, is_input_le, count)
                }
            fn from_slice(src: &mut Cursor<&'b [u8]>) -> Result<Self, crate::SpError>
            where
                Self: Sized {
                    <Self>::inner_from_slice(src, true, None)
                }
        }
    }
}

/// Implements SpReadRawMut for $typ
/// 
/// $typ
///     Type that the trait will be implemented on
/// $reader($typ:ty, $parse_func:ident, $src:expr, $is_input_le:expr, $count:expr)
///     Macro that fills the body of the inner_from_mut_slice() trait
///     $parse_func == inner_from_mut_slice
/// $generics
///     Optionnal generics that the $typ relies on
#[macro_use]
macro_rules! impl_SpReadRawMut {
    ($typ:ty, $inner_typ:ty, $reader:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)*)*) => {
        impl<'b, $($generics : SpReadRawMut<'b> $(+ $bound$(+ $other)*)*),*> SpReadRawMut<'b> for $typ {
            fn inner_from_mut_slice(
                src: &mut Cursor<&'b mut [u8]>,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized {
                    $reader!($typ, $inner_typ, inner_from_mut_slice, src, is_input_le, count)
                }
            fn from_mut_slice(src: &mut Cursor<&'b mut [u8]>) -> Result<Self, crate::SpError>
            where
                Self: Sized {
                    <Self>::inner_from_mut_slice(src, true, None)
                }
        }
    }
}

/// Implements SpWrite for $typ
/// 
/// $typ
///     Type that the trait will be implemented on
/// $writer($self:ident $(as $as_typ:ty)?, $is_output_le:ident, $prepend_count:ident, $dst: ident)
///     Macro that fills the body of the inner_to_writer() trait
/// $generics
///     Optionnal generics that the $typ relies on
#[macro_use]
macro_rules! impl_SpWrite {
    ($typ:ty $(as $inner_typ:ty)?, $writer:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)*)*) => {
        /// Write Self into writer
        impl<$($generics : SpWrite $(+ $bound$(+ $other)*)*),*> SpWrite for $typ {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                $writer!(self $(as $inner_typ)?, is_output_le, prepend_count, dst)
            }
            fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
                self.inner_to_writer(true, true, dst)
            }
        }
        impl<$($generics : SpWrite $(+ $bound$(+ $other)*)*),*> SpWrite for [$typ] {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                slice_to_writer!(self, is_output_le, prepend_count, dst)
            }
            fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
                self.inner_to_writer(true, true, dst)
            }
        }
        impl<$($generics : SpWrite $(+ $bound$(+ $other)*)*),*> SpWrite for &[$typ] {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                slice_to_writer!(self, is_output_le, prepend_count, dst)
            }
            fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
                self.inner_to_writer(true, true, dst)
            }
        }
        impl<$($generics : SpWrite $(+ $bound$(+ $other)*)*),*> SpWrite for &mut [$typ] {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                prepend_count: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                slice_to_writer!(self, is_output_le, prepend_count, dst)
            }
            fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
                self.inner_to_writer(true, true, dst)
            }
        }
    }
}

mod primitive;
pub use primitive::*;

mod string;
pub use string::*;

mod collections;
pub use collections::*;
