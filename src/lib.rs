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

/// Applies all file patches that are found in the diff file. This function also requires a path to
/// the directories of the source and target variants for the patch application, because it tries
/// to match the artifacts of the source variant with the artifacts of the target variant. The
/// calculated matching is then used to determine the best locations for a specific change to a
/// file.
///
/// This function creates new files for files that are created by a patch, and it deletes files for
/// file deletions in a patch, regardless of whether the lines in the patch match the lines in the
/// file completelty.
///
/// ## Parameters
///
/// ### source_dir_path
/// Specifies the path to the root directory of the source variant of the diff. The source variant
/// is the version of the program that was modified by the changes that are now to be applied.
///
/// ### target_dir_path
/// Specifies the path to the root directory of the target variant that is to be patched.
///
/// ### patch_file_path
/// Specifies the path to the diff that describes all changes that are to be applied. This diff
/// should usually have been determined between two versions of the source variant.
///
/// ### rejects_file_path
/// You may optionally provide a path to a rejects file to which all rejected changes are written.
/// If no path is provided, the rejects are printed to stdout.
///
/// ### strip
/// You can also define a path strip `s`. The strip is used to remove the leading `s` elements from
/// a path (e.g., the path `hello/world/directory` becomes `world/directory` for a strip of `s=1`).
/// Providing a strip is usually necessary because diffs are created based on two version of a
/// source variant that are located in different directories (e.g., `source-A/...` and
/// `source-B/...`). This top-level directory should be cut from the path for the patch being
/// applied successfully.  
///
/// ### dryrun
/// You should also specify whether the patch application should be made persistant (i.e., patched
/// files are saved), or if this is only a dryrun. In case of a dryrun, the patch application is
/// only simulated, printing all rejects to stdout without file changes.
///
/// ### matcher
/// Lastly, this function requires a matcher that is used to calculate the matching between source
/// and target variant. See `mpatch::matching` for more information.
///
// TODO: It would be great to track differences during file removal as rejects
// TODO: Improve interface of this function (e.g., make it smaller or at least more versatile)
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
        source_file_path.push(PathBuf::strip_and_clone(
            &file_diff.source_file().path(),
            strip,
        ));

        let mut target_file_path = target_dir_path.clone();
        target_file_path.push(PathBuf::strip_and_clone(
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
