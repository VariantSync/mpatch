// TODO: Align patch diff with variant diff
// TODO: Apply changes to target file
// TODO: Load target file from disk
// TODO: Save target file to disk
// TODO: Recognize file creation
// TODO: Recognize file deletion
// TODO: Write rejects to terminal
// TODO: Write rejects to file
// TODO: Feature traces and target configuration are part of the input!
// TODO: The matcher should be exchangeable (e.g., use a trait?)
// TODO: The matcher could use the LCS algorithm for starters; we only parse diffs to get the
// required changes, but the matcher itself is not based on diff but on LCS; look for an LCS crate
// on crates.io

pub mod diffs;
pub mod error;
pub mod files;
pub mod matching;

pub use diffs::CommitDiff;
pub use diffs::FileDiff;
pub use diffs::Hunk;
pub use error::Error;
pub use error::ErrorKind;
pub use files::FileArtifact;
pub use matching::LCSMatcher;
pub use matching::Matcher;
