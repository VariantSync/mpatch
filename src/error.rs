use std::fmt::Display;

/// Error is the main error type of this crate and used in all high-level instances of Result<...>
/// return values. Each error contains a message and an ErrorKind instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
    kind: ErrorKind,
}

impl Error {
    /// Creates a new Error instance with the given message and kind. The message is cloned into a
    /// String instance.
    ///
    /// # Examples
    /// ```
    ///    use mpatch::{Error, ErrorKind};
    ///
    ///    let error = Error::new("an error ocurred", ErrorKind::DiffParseError);
    ///    assert_eq!("an error ocurred", error.message());
    ///    assert_eq!(ErrorKind::DiffParseError, *error.kind());
    /// ```
    ///
    pub fn new(message: &str, kind: ErrorKind) -> Error {
        Error {
            message: message.to_string(),
            kind,
        }
    }

    /// Returns the message of this error
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the error kind of this error
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error {
            message: value.to_string(),
            kind: ErrorKind::IOError,
        }
    }
}

/// An ErrorKinds classifies which type of error has occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// A DiffParseError may occur while parsing a diff (i.e., a patch file)
    DiffParseError,
    /// An IOError may occur while reading or writing files from disk
    IOError,
    /// A PatchError may occur while applying a patch
    PatchError,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::DiffParseError => write!(f, "DiffParseError"),
            ErrorKind::IOError => write!(f, "IOError"),
            ErrorKind::PatchError => write!(f, "PatchError"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, ErrorKind};

    #[test]
    fn error_creation() {
        let error = Error::new("an error ocurred", ErrorKind::DiffParseError);
        assert_eq!("an error ocurred", error.message());
        assert_eq!(ErrorKind::DiffParseError, *error.kind());
    }

    #[test]
    fn error_printing() {
        let error = Error::new("error to print", ErrorKind::IOError);
        assert_eq!("IOError: error to print", error.to_string());
    }

    #[test]
    fn error_kind_printing() {
        assert_eq!("DiffParseError", &ErrorKind::DiffParseError.to_string());
        assert_eq!("IOError", &ErrorKind::IOError.to_string());
        assert_eq!("PatchError", &ErrorKind::PatchError.to_string());
    }
}
