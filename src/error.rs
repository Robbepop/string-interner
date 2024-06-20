use alloc::collections;
use core::fmt;

/// An error object returned from fallible methods of the string-interner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The interner already interns the maximum number of strings possible by the chosen symbol type.
    OutOfSymbols,
    /// An operation could not be completed, because it failed to allocate enough memory.
    OutOfMemory,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::OutOfSymbols => "no more symbols available",
            Error::OutOfMemory => "out of memory",
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl From<collections::TryReserveError> for Error {
    fn from(_: collections::TryReserveError) -> Self {
        Error::OutOfMemory
    }
}

impl From<hashbrown::TryReserveError> for Error {
    fn from(_: hashbrown::TryReserveError) -> Self {
        Error::OutOfMemory
    }
}

/// The type returned by fallible methods of the string-interner.
pub type Result<T> = core::result::Result<T, Error>;
