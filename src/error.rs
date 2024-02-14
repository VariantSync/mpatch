use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
    kind: ErrorKind,
}

impl Error {
    pub fn new(message: &str, kind: ErrorKind) -> Error {
        Error {
            message: message.to_string(),
            kind,
        }
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", self.kind, self.message)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    DiffParseError,
    IOError,
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
