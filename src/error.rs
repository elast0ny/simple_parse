use std::error;
use std::fmt;

/// Possible errors when reading/writing
#[derive(Debug)]
pub enum SpError {
    /// Could not read bytes from reader
    ReadFailed(std::io::Error),
    /// The data we attempted to decode did not contain a valid enum variant
    UnknownEnumVariant,
    /// There is not enough space to write T into the writer or to read T from the reader
    NotEnoughSpace,
    /// An annotated `len` field's type is too small to fit the number of elements
    CountFieldOverflow,
    /// The data contained enough bytes but the contents were invalid
    InvalidBytes,
    /// A Rust reference cannot be created as the data is misaligned
    BadAlignment,
}
impl fmt::Display for SpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SpError::ReadFailed(e) => write!(f, "Failed to read more bytes : {e}"),
            SpError::UnknownEnumVariant => write!(f, "Encountered invalid enum variant ID"),
            SpError::NotEnoughSpace => {
                write!(f, "Not enough bytes in the buffer to parse wanted type")
            }
            SpError::CountFieldOverflow => write!(
                f,
                "The `len` field's type is too small for the number of items !"
            ),
            SpError::InvalidBytes => write!(f, "Failed to parse the bytes into the wanted type"),
            SpError::BadAlignment => write!(f, "Input bytes are misaligned"),
        }
    }
}
impl error::Error for SpError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            SpError::ReadFailed(e) => Some(e),
            _ => None,
        }
    }
}
