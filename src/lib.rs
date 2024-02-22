// TODO: Feature traces and target configuration are part of the input!
// TODO: Handle git diffs as well; they have differences e.g., /dev/null, permission change

pub mod diffs;
pub mod error;
pub mod io;
pub mod matching;
pub mod patch;

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

pub use diffs::FileDiff;
pub use diffs::Hunk;
pub use diffs::VersionDiff;
pub use error::Error;
pub use error::ErrorKind;
pub use io::FileArtifact;
pub use matching::LCSMatcher;
pub use matching::Matcher;
use patch::FilePatch;

use crate::io::print_rejects;
use crate::io::read_or_create_empty;
use crate::io::write_rejects;
use crate::io::StrippedPath;

pub fn apply_all(
    source_dir_path: PathBuf,
    target_dir_path: PathBuf,
    patch_file_path: PathBuf,
    rejects_file_path: Option<PathBuf>,
    strip: usize,
    dryrun: bool,
    mut matcher: impl Matcher,
) -> Result<(), Error> {
    let diff = VersionDiff::read(patch_file_path)?;

    // We only create a rejects file if there are rejects
    let mut rejects_file: Option<BufWriter<File>> = None;

    for file_diff in diff {
        let diff_header = file_diff.header();
        let mut source_file_path = source_dir_path.clone();
        source_file_path.push(PathBuf::from_stripped(
            &file_diff.source_file().path(),
            strip,
        ));

        let mut target_file_path = target_dir_path.clone();
        target_file_path.push(PathBuf::from_stripped(
            &file_diff.target_file().path(),
            strip,
        ));

        let source = read_or_create_empty(source_file_path)?;
        let target = read_or_create_empty(target_file_path)?;

        let matching = matcher.match_files(source, target);
        let patch = FilePatch::from(file_diff);
        let aligned_patch = patch.align_to_target(matching);

        let patch_outcome = aligned_patch.apply(dryrun)?;
        let (actual_result, rejects, change_type) = (
            patch_outcome.patched_file(),
            patch_outcome.rejected_changes(),
            patch_outcome.change_type(),
        );

        // print the result of a dryrun
        println!("--------------------------------------------------------");
        println!("{change_type} {}", actual_result.path().to_string_lossy());

        if !rejects.is_empty() {
            match &rejects_file_path {
                Some(path) => write_rejects(diff_header, rejects, &mut rejects_file, path)?,
                None => {
                    print_rejects(diff_header, rejects);
                }
            }
        }
    }

    Ok(())
}
