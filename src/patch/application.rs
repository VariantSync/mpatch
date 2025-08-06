use std::{fs, path::Path};

use crate::{AlignedPatch, Error, FileArtifact, PatchOutcome};

use super::{FileChangeType, LineChangeType};

/// Consumes and applies the patch to the target file artifact.
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
pub fn apply_patch(mut patch: AlignedPatch, dryrun: bool) -> Result<PatchOutcome, Error> {
    // Check file existance; it must not exist when it is to be created and it must exist
    // when it is to be modified or removed
    let reject_patch = if patch.change_type == FileChangeType::Create {
        Path::exists(patch.target.path())
    } else {
        !Path::exists(patch.target.path())
    };
    if reject_patch {
        reject_all(&mut patch);
        return Ok(PatchOutcome {
            patched_file: patch.target,
            rejected_changes: patch.rejected_changes,
            change_type: patch.change_type,
        });
    }
    match patch.change_type {
        FileChangeType::Create => apply_file_creation(patch, dryrun),
        FileChangeType::Remove => apply_file_removal(patch, dryrun),
        FileChangeType::Modify => apply_file_modification(patch, dryrun),
    }
}

/// Rejects all changes in the patch.
fn reject_all(patch: &mut AlignedPatch) {
    let mut rejects = vec![];
    while let Some(change) = patch.changes.pop() {
        rejects.push(change);
    }
    while let Some(reject) = patch.rejected_changes.pop() {
        rejects.push(reject);
    }
    rejects.sort_by(|a, b| a.line_number.cmp(&b.line_number));
    patch.changes = vec![];
    patch.rejected_changes = rejects;
}

/// Applies a modification patch.
fn apply_file_modification(patch: AlignedPatch, dryrun: bool) -> Result<PatchOutcome, Error> {
    let ((path, lines), mut changes) = (
        (patch.target.into_path_and_lines()),
        patch.changes.into_iter().peekable(),
    );

    // The number of the currently processed line in the target file (before modification)
    // The line number is used to identify the edit locations that were previously determined
    // during the alignment.
    // We start at 0 to account for line insertions before the first line
    let mut target_line_number = 1;
    let mut patched_lines = vec![];
    'lines_loop: for line in lines {
        while changes.peek().map_or_else(
            || false,
            |c| match c.change_type {
                // Adds are anchored to the context line above (i.e., lower than target_line_number)
                LineChangeType::Add => c.line_number <= target_line_number,
                // Removes are anchored to actual line being removed (i.e. the line being currently
                // processed which has line number 'target_line_number'
                LineChangeType::Remove => c.line_number == target_line_number,
            },
        ) {
            let change = changes.next().expect("there should be a change to extract");
            match change.change_type {
                LineChangeType::Add => {
                    // add this line to the vector of patched lines
                    patched_lines.push(change.line);
                }
                LineChangeType::Remove => {
                    // remove this line by skipping it
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
        rejected_changes: patch.rejected_changes,
        change_type: patch.change_type,
    })
}

/// Applies the creation of a new file.
fn apply_file_creation(patch: AlignedPatch, dryrun: bool) -> Result<PatchOutcome, Error> {
    let (path, lines) = (
        patch.target.path().to_path_buf(),
        patch.changes.into_iter().map(|c| c.line).collect(),
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
        rejected_changes: patch.rejected_changes,
        change_type: patch.change_type,
    })
}

/// Applies the removal of an existing file.
fn apply_file_removal(patch: AlignedPatch, dryrun: bool) -> Result<PatchOutcome, Error> {
    // there are no lines in the removed file
    let path = patch.target.path().to_path_buf();

    if !dryrun {
        fs::remove_file(&path)?;
    }

    Ok(PatchOutcome {
        patched_file: FileArtifact::from_lines(path, vec![]),
        rejected_changes: patch.rejected_changes,
        change_type: patch.change_type,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        patch::{Change, LineChangeType},
        AlignedPatch, FileArtifact, FilePatch, VersionDiff,
    };

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

        super::reject_all(&mut patch);
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

        let patch_outcome = super::apply_patch(patch, true).unwrap();
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

        super::apply_patch(patch, true).unwrap();
    }
}
