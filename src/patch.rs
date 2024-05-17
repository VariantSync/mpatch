pub mod align;

use std::{
    fmt::Display,
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
    vec,
};

use crate::{
    diffs::{FileDiff, VersionDiff},
    io::{print_rejects, write_rejects, FileArtifact, StrippedPath},
    patch::align::align_to_target,
    Error, Matcher,
};

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
        // Required for reject printing/writing
        let diff_header = file_diff.header();

        let mut source_file_path = source_dir_path.clone();
        source_file_path.push(PathBuf::strip_cloned(
            &file_diff.source_file_header().path_cloned(),
            strip,
        ));

        let mut target_file_path = target_dir_path.clone();
        target_file_path.push(PathBuf::strip_cloned(
            &file_diff.target_file_header().path_cloned(),
            strip,
        ));

        let source = FileArtifact::read_or_create_empty(source_file_path)?;
        let target = FileArtifact::read_or_create_empty(target_file_path)?;

        let matching = matcher.match_files(source, target);
        let patch = FilePatch::from(file_diff);
        let aligned_patch = align_to_target(patch, matching);

        let patch_outcome = aligned_patch.apply(dryrun)?;

        let (actual_result, rejects, change_type) = (
            patch_outcome.patched_file(),
            patch_outcome.rejected_changes(),
            patch_outcome.change_type(),
        );

        // print the result
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

    /// Consumes and applies this patch to the target file artifact.
    /// This function differentiates between the three different FileChangeTypes: Create, Remove,
    /// and Modify.
    ///
    /// In case of Create, a new file is created and the entire content of the patch
    /// added to it. The patch fails if the file already exists.
    ///
    /// In case of Remove, the file and its entire content is removed, even if the file has more content
    /// than specified in the patch. The patch is rejected if the file does not exist.
    ///
    /// In case of Modify, the changes in the patch are applied in order. The patch is rejected if
    /// the file does not exist.
    ///
    /// If dryrun is set to true, the changes are not saved to the file. This is useful when
    /// looking for rejects without wanting to modify the target file.
    ///
    /// ## Error
    /// Returns an Error if the necessary file operations cannot be performed.
    pub fn apply(mut self, dryrun: bool) -> Result<PatchOutcome, Error> {
        // Check file existance; it must not exist when it is to be created and it must exist
        // when it is to be modified or removed
        let reject_patch = if self.change_type == FileChangeType::Create {
            Path::exists(self.target.path())
        } else {
            !Path::exists(self.target.path())
        };
        if reject_patch {
            self.reject_all();
            return Ok(PatchOutcome {
                patched_file: self.target,
                rejected_changes: self.rejected_changes,
                change_type: self.change_type,
            });
        }
        match self.change_type {
            FileChangeType::Create => self.apply_file_creation(dryrun),
            FileChangeType::Remove => self.apply_file_removal(dryrun),
            FileChangeType::Modify => self.apply_file_modification(dryrun),
        }
    }

    /// Rejects all changes in this patch.
    fn reject_all(&mut self) {
        let mut rejects = vec![];
        while let Some(change) = self.changes.pop() {
            rejects.push(change);
        }
        while let Some(reject) = self.rejected_changes.pop() {
            rejects.push(reject);
        }
        rejects.sort_by(|a, b| a.line_number.cmp(&b.line_number));
        self.changes = vec![];
        self.rejected_changes = rejects;
    }

    /// Applies a modification patch.
    fn apply_file_modification(self, dryrun: bool) -> Result<PatchOutcome, Error> {
        let ((path, lines), mut changes) = (
            (self.target.into_path_and_lines()),
            self.changes.into_iter().peekable(),
        );

        // The number of the currently processed line in the target file (before modification)
        // The line number is used to identify the edit locations that were previously determined
        // during the alignment.
        // We start at 0 to account for line insertions before the first line
        let mut target_line_number = 1;
        let mut patched_lines = vec![];
        'lines_loop: for line in lines {
            while changes.peek().map_or(false, |c| match c.change_type {
                // Adds are anchored to the context line above (i.e., lower than target_line_number)
                LineChangeType::Add => c.line_number <= target_line_number,
                // Removes are anchored to actual line being removed (i.e. the line being currently
                // processed which has line number 'target_line_number'
                LineChangeType::Remove => c.line_number == target_line_number,
            }) {
                let change = changes.next().expect("there should be a change to extract");
                match change.change_type {
                    LineChangeType::Add => {
                        // add this line to the vector of patched lines
                        patched_lines.push(change.line);
                    }
                    LineChangeType::Remove => {
                        // remove this line by skipping it
                        assert_eq!(
                            line, change.line,
                            "unexpected line difference in line {target_line_number}"
                        );
                        target_line_number += 1;
                        continue 'lines_loop;
                    }
                }
            }

            // once all changes for this line_number have been applied, we can add the next
            // unchanged line
            patched_lines.push(line);
            target_line_number += 1;
        }

        // Apply the remaining changes
        for change in changes {
            match change.change_type {
                LineChangeType::Add => {
                    // add this line to the vector of patched lines
                    patched_lines.push(change.line);
                }
                LineChangeType::Remove => {
                    eprint!("{}: {change}", change.line_number);
                    panic!("there were unprocessed changes in the patch");
                }
            }
        }

        let patched_file = FileArtifact::from_lines(path, patched_lines);

        if !dryrun {
            patched_file.write()?;
        }

        Ok(PatchOutcome {
            patched_file,
            rejected_changes: self.rejected_changes,
            change_type: self.change_type,
        })
    }

    /// Applies the creation of a new file.
    fn apply_file_creation(self, dryrun: bool) -> Result<PatchOutcome, Error> {
        let (path, lines) = (
            self.target.path().to_path_buf(),
            self.changes.into_iter().map(|c| c.line).collect(),
        );

        if !dryrun {
            // Create all parent directories
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        let patched_file = FileArtifact::from_lines(path, lines);
        if !dryrun {
            patched_file.write()?;
        }

        Ok(PatchOutcome {
            patched_file,
            rejected_changes: self.rejected_changes,
            change_type: self.change_type,
        })
    }

    /// Applies the removal of an existing file.
    fn apply_file_removal(self, dryrun: bool) -> Result<PatchOutcome, Error> {
        // there are no lines in the removed file
        let path = self.target.path().to_path_buf();

        if !dryrun {
            fs::remove_file(&path)?;
        }

        Ok(PatchOutcome {
            patched_file: FileArtifact::from_lines(path, vec![]),
            rejected_changes: self.rejected_changes,
            change_type: self.change_type,
        })
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
    use std::{cmp::Ordering, path::PathBuf};

    use crate::{diffs::VersionDiff, AlignedPatch, FileArtifact};

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
    fn reject_all() {
        let file_diff = VersionDiff::read("tests/diffs/simple.diff").unwrap();
        let file_diff = file_diff.file_diffs().first().unwrap().clone();
        let patch = FilePatch::from(file_diff);
        let mut patch = AlignedPatch {
            changes: patch.changes,
            rejected_changes: vec![Change {
                line: "additional reject".to_string(),
                change_type: LineChangeType::Add,
                line_number: 99,
                change_id: 4,
            }],
            target: FileArtifact::new(PathBuf::from("empty")),
            change_type: super::FileChangeType::Modify,
        };

        patch.reject_all();
        assert_eq!(5, patch.rejected_changes.len());
    }

    #[test]
    fn add_lines_at_end() {
        let artifact = FileArtifact::from_lines(
            PathBuf::from("tests/samples/target_variant/version-0/main.c"),
            vec!["first line".to_string()],
        );
        let changes = vec![
            Change {
                line: "second line".to_string(),
                change_type: LineChangeType::Add,
                line_number: 2,
                change_id: 0,
            },
            Change {
                line: "third line".to_string(),
                change_type: LineChangeType::Add,
                line_number: 2,
                change_id: 1,
            },
        ];

        let patch = AlignedPatch {
            changes,
            rejected_changes: vec![],
            target: artifact,
            change_type: super::FileChangeType::Modify,
        };

        let patch_outcome = patch.apply(true).unwrap();
        assert!(patch_outcome.rejected_changes().is_empty());

        let patched_file = patch_outcome.patched_file();
        assert_eq!(3, patched_file.len());
        assert_eq!("first line", patched_file.lines()[0]);
        assert_eq!("second line", patched_file.lines()[1]);
        assert_eq!("third line", patched_file.lines()[2]);
    }

    #[test]
    #[should_panic(expected = "there were unprocessed changes")]
    fn try_to_remove_lines_after_end() {
        let artifact = FileArtifact::from_lines(
            PathBuf::from("tests/samples/target_variant/version-0/main.c"),
            vec!["first line".to_string()],
        );
        let changes = vec![Change {
            line: "second line".to_string(),
            change_type: LineChangeType::Remove,
            line_number: 2,
            change_id: 0,
        }];

        let patch = AlignedPatch {
            changes,
            rejected_changes: vec![],
            target: artifact,
            change_type: super::FileChangeType::Modify,
        };

        patch.apply(true).unwrap();
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
