// TODO: Recognize file creation
// TODO: Recognize file deletion
// TODO: Feature traces and target configuration are part of the input!
// TODO: Handle git diffs as well; they have differences e.g., /dev/null, permission change

pub mod diffs;
pub mod error;
pub mod files;
pub mod matching;
pub mod patch;

use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

pub use diffs::CommitDiff;
pub use diffs::FileDiff;
pub use diffs::Hunk;
pub use error::Error;
pub use error::ErrorKind;
pub use files::FileArtifact;
use files::StrippedPath;
pub use matching::LCSMatcher;
pub use matching::Matcher;
use patch::FilePatch;

pub fn apply_all(
    source_dir: PathBuf,
    target_dir: PathBuf,
    patch_file: &str,
    rejects_file: Option<&str>,
    strip: usize,
    dryrun: bool,
    mut matcher: impl Matcher,
) -> Result<(), Error> {
    let diff = CommitDiff::read(patch_file).unwrap();

    let mut rejects_file = rejects_file
        .map(|rf| BufWriter::new(File::create_new(rf).expect("rejects file already exists!")));

    for file_diff in diff {
        let mut source_file_path = source_dir.clone();
        source_file_path.push(PathBuf::from_stripped(
            &file_diff.source_file().path(),
            strip,
        ));

        let mut target_file_path = target_dir.clone();
        target_file_path.push(PathBuf::from_stripped(
            &file_diff.target_file().path(),
            strip,
        ));

        let source = FileArtifact::read(
            source_file_path
                .to_str()
                .expect("the source directory is not a valid UTF-8 path"),
        )
        .unwrap();
        let target = FileArtifact::read(
            target_file_path
                .to_str()
                .expect("the target directory is not a valid UTF-8 path"),
        )
        .unwrap();

        let matching = matcher.match_files(source, target);
        let patch = FilePatch::from(file_diff);
        let aligned_patch = patch.align_to_target(matching);
        let actual_result = aligned_patch.apply();
        let (actual_result, rejects) = (
            actual_result.patched_file(),
            actual_result.rejected_changes(),
        );

        if !dryrun {
            if let Err(e) = actual_result.write() {
                return Err(Error::new(&e.to_string(), ErrorKind::IOError));
            }
        }

        match &mut rejects_file {
            Some(rejects_file) => {
                for reject in rejects {
                    if let Err(e) = rejects_file.write_fmt(format_args!("{}", reject)) {
                        return Err(Error::new(&e.to_string(), ErrorKind::IOError));
                    }
                }
            }
            None => {
                for reject in rejects {
                    println!("{}", reject);
                }
            }
        }
    }

    Ok(())
}
