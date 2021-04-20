use core::fmt::{self, Display, Formatter};

#[cfg(feature = "std")]
use std::error::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Possible errors of `EntityTag`.
pub enum EntityTagError {
    MissingStartingDoubleQuote,
    MissingClosingDoubleQuote,
    InvalidTag,
}

impl Display for EntityTagError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            EntityTagError::MissingStartingDoubleQuote => {
                f.write_str("the opaque tag misses the starting double quote")
            }
            EntityTagError::MissingClosingDoubleQuote => {
                f.write_str("the opaque tag misses the closing double quote")
            }
            EntityTagError::InvalidTag => f.write_str("invalid tag"),
        }
    }
}

#[cfg(feature = "std")]
impl Error for EntityTagError {}
