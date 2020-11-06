use std::error;
use std::fmt;

/// Possible errors when decoding/encoding
#[derive(Debug, Clone, Copy)]
pub enum SpError {
    /// The data we attempted to decode did not contain a valid enum variant
    UnknownEnumVariant,
    /// The is not enough space to decode into T or to write T into the writer
    NotEnoughSpace,
    /// An annotated count field's type is too small to fit the number of elements
    CountFieldOverflow,
    /// The data contained enough bytes but the format was wrong
    InvalidBytes,
}
impl fmt::Display for SpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SpError::UnknownEnumVariant => {
                write!(f, "Encountered invalid enum variant ID")
            }
            SpError::NotEnoughSpace => {
                write!(f, "Not enough bytes in the buffer to parse wanted type")
            }
            SpError::CountFieldOverflow => write!(
                f,
                "The count field's type is too small for the number of items !"
            ),
            SpError::InvalidBytes => write!(f, "Failed to parse the bytes into the wanted type"),
        }
    }
}
impl error::Error for SpError {
    fn cause(&self) -> Option<&dyn error::Error> {
        Some(self)
    }
}
