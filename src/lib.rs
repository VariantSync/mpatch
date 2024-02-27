// TODO: Feature traces and target configuration are part of the input!
// TODO: Handle git diffs as well; they have differences e.g., /dev/null, permission change

pub mod diffs;
pub mod error;
mod io;
pub mod matching;
pub mod patch;

pub use diffs::FileDiff;
pub use diffs::VersionDiff;
pub use error::Error;
pub use error::ErrorKind;
pub use io::FileArtifact;
pub use matching::LCSMatcher;
pub use matching::Matcher;
pub use patch::apply_all;
pub use patch::AlignedPatch;
pub use patch::FilePatch;
pub use patch::PatchOutcome;
