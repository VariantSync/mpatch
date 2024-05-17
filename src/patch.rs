pub mod alignment;
pub mod application;
pub mod filtering;
pub mod matching;

use std::{fmt::Display, fs::File, io::BufWriter, path::PathBuf, vec};

use crate::{
    diffs::{FileDiff, VersionDiff},
    io::{print_rejects, write_rejects, FileArtifact, StrippedPath},
    patch::{alignment::align_to_target, application::apply_patch},
    Error, Matcher,
};

use self::filtering::Filter;

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
    patch_paths: PatchPaths,
    strip: usize,
    dryrun: bool,
    mut matcher: impl Matcher,
    mut filter: impl Filter,
) -> Result<(), Error> {
    let diff = VersionDiff::read(patch_paths.patch_file_path)?;

    // We only create a rejects file if there are rejects
    let mut rejects_file: Option<BufWriter<File>> = None;

    for file_diff in diff {
        // Required for reject printing/writing
        let diff_header = file_diff.header();

        let mut source_file_path = patch_paths.source_dir_path.clone();
        source_file_path.push(PathBuf::strip_cloned(
            &file_diff.source_file_header().path_cloned(),
            strip,
        ));

        let mut target_file_path = patch_paths.target_dir_path.clone();
        target_file_path.push(PathBuf::strip_cloned(
            &file_diff.target_file_header().path_cloned(),
            strip,
        ));

        let source = FileArtifact::read_or_create_empty(source_file_path)?;
        let target = FileArtifact::read_or_create_empty(target_file_path)?;

        let matching = matcher.match_files(source, target);
        let patch = FilePatch::from(file_diff);
        let filtered_patch = filter.apply_filter(patch, &matching);
        let aligned_patch = align_to_target(filtered_patch, matching);

        let patch_outcome = apply_patch(aligned_patch, dryrun)?;

        let (actual_result, rejects, change_type) = (
            patch_outcome.patched_file(),
            patch_outcome.rejected_changes(),
            patch_outcome.change_type(),
        );

        // print the result
        println!("--------------------------------------------------------");
        println!("{change_type} {}", actual_result.path().to_string_lossy());

        if !rejects.is_empty() {
            match &patch_paths.rejects_file_path {
                Some(path) => write_rejects(diff_header, rejects, &mut rejects_file, path)?,
                None => {
                    print_rejects(diff_header, rejects);
                }
            }
        }
    }

    Ok(())
}

pub struct PatchPaths {
    source_dir_path: PathBuf,
    target_dir_path: PathBuf,
    patch_file_path: PathBuf,
    rejects_file_path: Option<PathBuf>,
}

impl PatchPaths {
    pub fn new(
        source_dir_path: PathBuf,
        target_dir_path: PathBuf,
        patch_file_path: PathBuf,
        rejects_file_path: Option<PathBuf>,
    ) -> PatchPaths {
        PatchPaths {
            source_dir_path,
            target_dir_path,
            patch_file_path,
            rejects_file_path,
        }
    }
}

/// A file patch contains a vector of changes for a specific file from a FileDiff.
/// A file patch also has a change type that describes whether the file is created, removed, or
/// modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePatch {
    changes: Vec<Change>,
    change_type: FileChangeType,
}

impl FilePatch {
    /// Returns a reference to the changes in this patch.
    pub fn changes(&self) -> &[Change] {
        &self.changes
    }
}

impl From<FileDiff> for FilePatch {
    fn from(file_diff: FileDiff) -> Self {
        let mut changes = vec![];

        // Determine the change type of this patch by looking at the first hunk
        let first_hunk = file_diff.hunks().first().expect("no hunk in diff");
        // A hunk start of '0' indicates that the file does not exist for source or target
        let file_change_type = if first_hunk.source_location().hunk_start() == 0 {
            FileChangeType::Create
        } else if first_hunk.target_location().hunk_start() == 0 {
            FileChangeType::Remove
        } else {
            FileChangeType::Modify
        };

        // Extract all changes from the file diff
        for (change_id, line) in file_diff.into_changes().enumerate() {
            let line_number;
            let change_type;
            match line.line_type() {
                crate::diffs::LineType::Add => {
                    change_type = LineChangeType::Add;
                    // Lines that are added do not exist in the source file yet; therefore, they only
                    // have a change location, but no real location
                    line_number = line.source_line().change_location();
                }
                crate::diffs::LineType::Remove => {
                    change_type = LineChangeType::Remove;
                    // Lines that are removed must exist in the source file and must thus have a
                    // real location.
                    line_number = line.source_line().real_location();
                }
                _ => panic!("a change must always be an Add or Remove"),
            }

            changes.push(Change {
                line: line.into_original_text(),
                change_type,
                line_number,
                change_id,
            });
        }

        FilePatch {
            changes,
            change_type: file_change_type,
        }
    }
}

/// An aligned patch contains a vector of changes that were aligned for a specific target file.
/// The patch holds ownership of the target FileArtifact and changes it during patch application.
/// Applying the patch consumes it to prohibit mutliple applications of the same patch to the same
/// file. An aligned patch also has a change type that describes whether the file is created,
/// removed, or modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignedPatch {
    changes: Vec<Change>,
    rejected_changes: Vec<Change>,
    target: FileArtifact,
    change_type: FileChangeType,
}

impl AlignedPatch {
    /// Returns a reference to the aligned changes of this patch.
    pub fn changes(&self) -> &[Change] {
        self.changes.as_ref()
    }

    /// Returns a reference to the target file artifact of this patch.
    pub fn target(&self) -> &FileArtifact {
        &self.target
    }
}

impl Display for AlignedPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.change_type,
            self.target.path().to_string_lossy()
        )
    }
}

/// A patch outcome contains all information about a performed patch application. It contains the
/// patched file artifact, which has already been written to disk if dryrun is disabled.
/// Furthermore, it contains a vector of all rejected changes and the change type of the applied
/// patch.
///
/// The outcomes for a dryrun of a patch and its real application are the same.  
/// TODO: Should the outcome really still contain the FileArtifact? This might suggest that it
/// should still be saved or edited.
pub struct PatchOutcome {
    patched_file: FileArtifact,
    rejected_changes: Vec<Change>,
    change_type: FileChangeType,
}

impl PatchOutcome {
    /// Returns a reference to the patched file artifact.
    pub fn patched_file(&self) -> &FileArtifact {
        &self.patched_file
    }

    /// Returns a reference to the rejected changes.
    pub fn rejected_changes(&self) -> &[Change] {
        &self.rejected_changes
    }

    /// Returns the change type of the applied patch.
    pub fn change_type(&self) -> FileChangeType {
        self.change_type
    }
}

/// A change represent a single line change (i.e., adding or removing a line of text).
/// Each change has a content, a change type, a line number, and a change id.
///
/// The change id is used to identify a change among all changes of a patch which was originally
/// created from a diff. Here, the changes in a diff are given ids from 0 to n-1.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Change {
    line: String,
    change_type: LineChangeType,
    line_number: usize,
    change_id: usize,
}

impl Change {
    /// Returns a reference to the content of this change.
    pub fn line(&self) -> &str {
        &self.line
    }

    /// Returns the change type.
    pub fn change_type(&self) -> LineChangeType {
        self.change_type
    }

    /// Returns the line number to which this change should be applied.
    pub fn line_number(&self) -> usize {
        self.line_number
    }

    /// Returns the id of the change with respect to the diff from which it was extracted.
    pub fn change_id(&self) -> usize {
        self.change_id
    }
}

impl PartialOrd for Change {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Change {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compare the line numbers to which the changes were matches
        let ordering = self.line_number().cmp(&other.line_number());
        // If they are equal, compare the change type
        let ordering = match ordering {
            std::cmp::Ordering::Equal => self.change_type.cmp(&other.change_type),
            ordering => return ordering,
        };
        // If they are equal as well, compare the change id
        match ordering {
            std::cmp::Ordering::Equal => self.change_id.cmp(&other.change_id),
            ordering => ordering,
        }
    }
}

impl Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.change_type {
            LineChangeType::Add => writeln!(f, "+{}", self.line),
            LineChangeType::Remove => writeln!(f, "-{}", self.line),
        }
    }
}

/// Enum representing the two possible change types for a line: Add and Remove.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LineChangeType {
    Add,
    Remove,
}

impl PartialOrd for LineChangeType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LineChangeType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Removes should always be applied before Adds
        match self {
            LineChangeType::Add => match other {
                LineChangeType::Add => std::cmp::Ordering::Equal,
                LineChangeType::Remove => std::cmp::Ordering::Greater,
            },
            LineChangeType::Remove => match other {
                LineChangeType::Add => std::cmp::Ordering::Less,
                LineChangeType::Remove => std::cmp::Ordering::Equal,
            },
        }
    }
}

/// Enum representing the three possible change types for a file: Create, Remove, and Modify.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileChangeType {
    Create,
    Remove,
    Modify,
}

impl Display for FileChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileChangeType::Create => write!(f, "Create"),
            FileChangeType::Remove => write!(f, "Remove"),
            FileChangeType::Modify => write!(f, "Modify"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::diffs::VersionDiff;

    use super::{Change, FilePatch, LineChangeType};

    #[test]
    fn patch_from_diff() {
        let file_diff = VersionDiff::read("tests/diffs/simple.diff").unwrap();
        let file_diff = file_diff.file_diffs().first().unwrap().clone();

        let expected_changes = [
            Change {
                line: "REMOVED".to_string(),
                change_type: LineChangeType::Remove,
                line_number: 4,
                change_id: 0,
            },
            Change {
                line: "ADDED".to_string(),
                change_type: LineChangeType::Add,
                line_number: 5,
                change_id: 1,
            },
            Change {
                line: "REMOVED".to_string(),
                change_type: LineChangeType::Remove,
                line_number: 26,
                change_id: 2,
            },
            Change {
                line: "ADDED".to_string(),
                change_type: LineChangeType::Add,
                line_number: 27,
                change_id: 3,
            },
        ];

        let patch = FilePatch::from(file_diff);

        for (change, expected_change) in patch.changes.into_iter().zip(expected_changes.into_iter())
        {
            assert_eq!(change, expected_change);
        }
    }

    #[test]
    fn order_changes_by_id_as_last_resort() {
        let mut changes = [
            Change {
                line: "second line".to_string(),
                change_type: LineChangeType::Add,
                line_number: 1,
                change_id: 1,
            },
            Change {
                line: "first line".to_string(),
                change_type: LineChangeType::Add,
                line_number: 1,
                change_id: 0,
            },
        ];

        changes.sort();

        assert_eq!(0, changes[0].change_id);
        assert_eq!(1, changes[1].change_id);
    }

    #[test]
    fn line_change_type_ordering() {
        assert_eq!(
            Ordering::Less,
            LineChangeType::Remove
                .partial_cmp(&LineChangeType::Add)
                .unwrap()
        );
        assert_eq!(
            Ordering::Equal,
            LineChangeType::Remove
                .partial_cmp(&LineChangeType::Remove)
                .unwrap()
        );
        assert_eq!(
            Ordering::Equal,
            LineChangeType::Add
                .partial_cmp(&LineChangeType::Add)
                .unwrap()
        );
        assert_eq!(
            Ordering::Greater,
            LineChangeType::Add
                .partial_cmp(&LineChangeType::Remove)
                .unwrap()
        );
    }
}
