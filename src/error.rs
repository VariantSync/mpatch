pub struct Error {
    message: String,
    kind: ErrorKind,
}

pub enum ErrorKind {
    DiffParseError,
}
