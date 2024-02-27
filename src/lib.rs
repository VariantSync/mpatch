// TODO: Feature traces and target configuration are part of the input!
// TODO: Handle git diffs as well; they have differences e.g., /dev/null, permission change

pub mod diffs;
pub mod error;
pub mod io;
pub mod matching;
pub mod patch;

pub use diffs::FileDiff;
pub use diffs::Hunk;
pub use diffs::VersionDiff;
pub use error::Error;
pub use error::ErrorKind;
pub use io::FileArtifact;
pub use matching::LCSMatcher;
pub use matching::Matcher;
pub use patch::apply_all;
