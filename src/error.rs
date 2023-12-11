#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
    kind: ErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    DiffParseError,
}
