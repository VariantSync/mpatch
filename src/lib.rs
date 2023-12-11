// TODO: Parse diff into internal format
// TODO: Parse source file into internal format
// TODO: Align patch diff with variant diff
// TODO: Apply changes to target file
// TODO: Load diff from disk
// TODO: Load target file from disk
// TODO: Save target file to disk
// TODO: Recognize file creation
// TODO: Recognize file deletion
// TODO: Write rejects to terminal
// TODO: Write rejects to file

pub mod diff;
pub mod error;

pub use diff::Diff;
pub use diff::FileDiff;
pub use diff::Hunk;
pub use error::Error;
pub use error::ErrorKind;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
