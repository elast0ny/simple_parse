use std::error;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum SpError {
    UnknownEnumVariant(usize),
    NotEnoughSpace,
    CountFieldOverflow,
    InvalidBytes,
}
impl fmt::Display for SpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SpError::UnknownEnumVariant(ref id) => {
                write!(f, "Encountered unknown enum variant ID : {}", id)
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
