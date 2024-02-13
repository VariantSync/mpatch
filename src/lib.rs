// TODO: Recognize file creation
// TODO: Recognize file deletion
// TODO: Feature traces and target configuration are part of the input!
// TODO: Handle git diffs as well; they have differences e.g., /dev/null, permission change

pub mod diffs;
pub mod error;
pub mod files;
pub mod matching;
pub mod patch;

pub use diffs::CommitDiff;
pub use diffs::FileDiff;
pub use diffs::Hunk;
pub use error::Error;
pub use error::ErrorKind;
pub use files::FileArtifact;
pub use matching::LCSMatcher;
pub use matching::Matcher;
