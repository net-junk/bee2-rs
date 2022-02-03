//! Error types.

/// Invalid length of input data(key, IV, annotation) error.
#[derive(Clone, Debug)]
pub struct InvalidLength;

impl core::fmt::Display for InvalidLength {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "InvalidLength")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidLength {}

/// Invalid order of commands error.
#[derive(Copy, Clone, Debug)]
pub struct InvalidCommand;

impl core::fmt::Display for InvalidCommand {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.write_str("InvalidCommand;")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidCommand {}

/// Invalid length of input data(key, IV, annotation) error.
#[derive(Clone, Debug)]
pub struct IncorrectTag;

impl core::fmt::Display for IncorrectTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "IncorrectTag")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for IncorrectTag {}

/// General Error for bee2-rs
#[derive(Clone, Debug)]
pub enum Error {
    InvalidLength(InvalidLength),
    InvalidCommand(InvalidCommand),
    IncorrectTag(IncorrectTag),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        match *self {
            Error::InvalidCommand(ref err) => write!(f, "{}", err),
            Error::InvalidLength(ref err) => write!(f, "{}", err),
            Error::IncorrectTag(ref err) => write!(f, "{}", err),
        }
    }
}

impl From<InvalidLength> for Error {
    fn from(other: InvalidLength) -> Self {
        Error::InvalidLength(other)
    }
}

impl From<InvalidCommand> for Error {
    fn from(other: InvalidCommand) -> Self {
        Error::InvalidCommand(other)
    }
}

impl From<IncorrectTag> for Error {
    fn from(other: IncorrectTag) -> Self {
        Error::IncorrectTag(other)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
