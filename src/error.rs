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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    DiffParseError,
}
