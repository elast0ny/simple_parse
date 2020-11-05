/// Implements SpRead for $typ
/// 
/// $typ
///     Type that the trait will be implemented on
/// $reader($parse_func:ident, $src:expr, $is_input_le:expr, $count:expr)
///     Macro that fills the body of the inner_from_slice() trait
///     $reader == inner_from_slice
/// $generics
///     Optionnal generics that the $typ relies on
#[macro_use]
macro_rules! impl_SpRead {

    ($typ:ty, $reader:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)*)*) => {
        /// $typ from reader
        impl<$($generics : SpRead $(+ $bound$(+ $other)*)*),*> SpRead for $typ {
            fn inner_from_reader<R: Read + ?Sized> (
                src: &mut R,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized {
                    $reader!(inner_from_reader, src, is_input_le, count)
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
/// $reader($parse_func:ident, $src:expr, $is_input_le:expr, $count:expr)
///     Macro that fills the body of the inner_from_slice() trait
///     $reader == inner_from_slice
/// $generics
///     Optionnal generics that the $typ relies on
#[macro_use]
macro_rules! impl_SpReadRaw {
    ($typ:ty, $reader:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)*)*) => {
        /// $typ from bytes
        impl<'b, $($generics : SpReadRaw<'b> $(+ $bound$(+ $other)*)*),*> SpReadRaw<'b> for $typ {
            fn inner_from_slice(
                src: &mut Cursor<&'b [u8]>,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized {
                    $reader!(inner_from_slice, src, is_input_le, count)
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
/// $reader($parse_func:ident, $src:expr, $is_input_le:expr, $count:expr)
///     Macro that fills the body of the inner_from_mut_slice() trait
///     $parse_func == inner_from_mut_slice
/// $generics
///     Optionnal generics that the $typ relies on
#[macro_use]
macro_rules! impl_SpReadRawMut {
    ($typ:ty, $reader:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)*)*) => {
        /// $typ from mutable bytes
        impl<'b, $($generics : SpReadRawMut<'b> $(+ $bound$(+ $other)*)*),*> SpReadRawMut<'b> for $typ {
            fn inner_from_mut_slice(
                src: &mut Cursor<&'b mut [u8]>,
                is_input_le: bool,
                count: Option<usize>,
            ) -> Result<Self, crate::SpError>
            where
                Self: Sized {
                    $reader!(inner_from_mut_slice, src, is_input_le, count)
                }
            fn from_mut_slice(src: &mut Cursor<&'b mut [u8]>) -> Result<Self, crate::SpError>
            where
                Self: Sized {
                    <Self>::inner_from_mut_slice(src, true, None)
                }
        }
    }
}

/// Implements SpWrite for $typ, &$typ and &mut $typ
/// 
/// $typ
///     Type that the trait will be implemented on
/// $writer($self:ident, $is_output_le:ident, $dst: ident)
///     Macro that fills the body of the inner_to_writer() trait
/// $generics
///     Optionnal generics that the $typ relies on
#[macro_use]
macro_rules! impl_SpWrite {
    ($typ:ty, $writer:ident $(, $generics:tt $(: $bound:ident $(+ $other:ident)*)*)*) => {
        /// Write Self into writer
        impl<$($generics : SpWrite $(+ $bound$(+ $other)*)*),*> SpWrite for $typ {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                $writer!(self, is_output_le, dst)
            }
            fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
                self.inner_to_writer(true, dst)
            }
        }
        /// Write &Self into writer
        impl<$($generics : SpWrite $(+ $bound$(+ $other)*)*),*> SpWrite for & $typ {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                (**self).inner_to_writer(is_output_le, dst)
            }
            fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
                self.inner_to_writer(true, dst)
            }
        }
        /// Write &mut Self into writer
        impl<$($generics : SpWrite $(+ $bound$(+ $other)*)*),*> SpWrite for &mut $typ {
            fn inner_to_writer<W: Write + ?Sized>(
                &self,
                is_output_le: bool,
                dst: &mut W,
            ) -> Result<usize, crate::SpError> {
                (**self).inner_to_writer(is_output_le, dst)
            }
            fn to_writer<W: Write + ?Sized>(&self, dst: &mut W) -> Result<usize, crate::SpError> {
                self.inner_to_writer(true, dst)
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
