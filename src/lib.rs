//! mpatch is a tool for patching files based on a matching between the source and target of the
//! patch. Its usage is similar to Unix patch as it requires a diff file as input that specifies
//! the changes which have been determined between two versions of the source variant. Currently,
//! it is assumed that the diff has been calculated using Unix diff with the recommended list of arguments
//! `-Naur`.
//!
//! The library can be used to calculate matchings between two source files, or to apply patches
//! read from a file or provided as text.
//!
//! # Examples
//! ## Patching target variants
//! You can patch a target variant using the `apply_all` function. See its documentation for more
//! information on the paramters.
//! ```
//! use std::path::PathBuf;
//! use std::str::FromStr;
//! let source_dir = PathBuf::from("tests/samples/source_variant/version-0");
//! let target_dir = PathBuf::from("tests/samples/target_variant/version-0");
//! let patch_file = PathBuf::from("tests/samples/patch.diff");
//! let rejects_file = None;
//! let strip = 1;
//! let dryrun = true;
//! let matcher = mpatch::LCSMatcher;
//!
//! if let Err(error) = mpatch::apply_all(
//!     source_dir,
//!     target_dir,
//!     patch_file,
//!     rejects_file,
//!     strip,
//!     dryrun,
//!     matcher,
//! ) {
//!     eprintln!("{}", error);
//! }
//! ```

// TODO: Feature traces and target configuration are part of the input!
// TODO: Handle git diffs as well; they have differences e.g., /dev/null, permission change
// TODO: Handle certain edge cases in which code is added at then end of the file (the existing
// last line should not be pushed down)

/// Module for types that implement reading and parsing diff files.
pub mod diffs;
/// Module for error types.
pub mod error;
mod io;
/// Module for aligning patches
#[doc(inline)]
pub use patch::alignment;
/// Module for applying patches
#[doc(inline)]
pub use patch::application;
/// Module for filtering patches
#[doc(inline)]
pub use patch::filtering;
/// Module for matching two files.
#[doc(inline)]
pub use patch::matching;
/// Module for types and functions that represent patches and patch application.
pub mod patch;

#[doc(inline)]
pub use diffs::FileDiff;
#[doc(inline)]
pub use diffs::VersionDiff;
#[doc(inline)]
pub use error::Error;
#[doc(inline)]
pub use error::ErrorKind;
#[doc(inline)]
pub use io::FileArtifact;
#[doc(inline)]
pub use matching::LCSMatcher;
#[doc(inline)]
pub use matching::Matcher;
#[doc(inline)]
pub use matching::Matching;
#[doc(inline)]
pub use patch::apply_all;
#[doc(inline)]
pub use patch::AlignedPatch;
#[doc(inline)]
pub use patch::FilePatch;
#[doc(inline)]
pub use patch::PatchOutcome;
