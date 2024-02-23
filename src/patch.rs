use std::{fmt::Display, fs, path::Path, vec};

use crate::{matching::Matching, Error, FileArtifact, FileDiff};

/// A file patch contains a vector of changes for a specific file from a FileDiff.
/// A file patch also has a change type that describes whether the file is created, removed, or
/// modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePatch {
    changes: Vec<Change>,
    change_type: FileChangeType,
}

impl FilePatch {
    /// Consumes and aligns this patch to a specific target file based on a matching.
    /// The source file in the matching must also be the source file of the FileDiff from which
    /// this FilePatch has been created. This means that it is the version of the source file
    /// before the changes in this patch have been applied to it.
    /// The target file is automatically read from the given matching.
    ///
    /// ## Returns
    /// Returns an aligned patch. In an aligned patch, all changes have been mapped to the best
    /// possible location in the target file. Changes removing a line are mapped to the exact line
    /// that has been removed from the source file. If no such line is found, the change is
    /// rejected and stored as a reject of the aligned patch.
    /// Changes adding a line are mapped to the closest matching location in the target file, which
    /// is determined by considering the matches of the lines in the source file that come before
    /// the added line.
    pub fn align_to_target(self, target_matching: Matching) -> AlignedPatch {
        if self.change_type == FileChangeType::Create {
            // Files that are to be created are aligned by definition
            return AlignedPatch {
                changes: self.changes,
                rejected_changes: vec![],
                target: target_matching.into_target(),
                change_type: self.change_type,
            };
        }

        // Align all changes
        let mut changes = Vec::with_capacity(self.changes.len());
        let mut rejected_changes = vec![];
        for mut change in self.changes {
            // Determine the best target line for each change
            let target_line_number = match change.change_type {
                LineChangeType::Add => target_matching
                    .target_index_fuzzy(change.line_number)
                    // Adds without a match are mapped to line 0 (i.e., prepend line)
                    .or(Some(0)),
                LineChangeType::Remove => {
                    // Removals without a match are automatically rejected
                    target_matching.target_index(change.line_number).flatten()
                }
            };
            if let Some(target_line_number) = target_line_number {
                // Align the change, if a suitable location has been found
                change.line_number = target_line_number;
                changes.push(change);
            } else {
                // Otherwise, reject the change
                rejected_changes.push(change);
            }
        }

        // During the alignment it is possible that changes switch their order because code chunks
        // might have been switched in the target file. This causes issues when applying changes,
        // because the change application assumes that the changes are ordered by line number.
        // Therefore, we sort all changes to ensure that they are applied in the correct order.
        changes.sort();

        AlignedPatch {
            changes,
            rejected_changes,
            target: target_matching.into_target(),
            change_type: self.change_type,
        }
    }

    /// Clones the patch for each given matching and aligns it to the corresponding target of each
    /// matching.
    /// The source file in each matching must also be the source file of the FileDiff from which
    /// this FilePatch has been created. This means that it is the version of the source file
    /// before the changes in this patch have been applied to it.
    /// The target file is automatically read from the given matching.
    ///
    /// ## Returns
    /// Returns a vector of aligned patches, one for each matching. In an aligned patch, all changes
    /// have been mapped to the best possible location in the target file. Changes removing a line
    /// are mapped to the exact line that has been removed from the source file. If no such line is
    /// found, the change is rejected and stored as a reject of the aligned patch.
    /// Changes adding a line are mapped to the closest matching location in the target file, which
    /// is determined by considering the matches of the lines in the source file that come before
    /// the added line.
    pub fn align_to_multiple_targets(&self, target_matchings: Vec<Matching>) -> Vec<AlignedPatch> {
        let mut aligned_patches = Vec::with_capacity(target_matchings.len());
        for matching in target_matchings.into_iter() {
            aligned_patches.push(self.clone().align_to_target(matching));
        }
        aligned_patches
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignedPatch {
    changes: Vec<Change>,
    rejected_changes: Vec<Change>,
    target: FileArtifact,
    change_type: FileChangeType,
}

impl AlignedPatch {
    pub fn changes(&self) -> &[Change] {
        self.changes.as_ref()
    }

    pub fn target(&self) -> &FileArtifact {
        &self.target
    }

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

pub struct PatchOutcome {
    patched_file: FileArtifact,
    rejected_changes: Vec<Change>,
    change_type: FileChangeType,
}

impl PatchOutcome {
    pub fn patched_file(&self) -> &FileArtifact {
        &self.patched_file
    }

    pub fn rejected_changes(&self) -> &[Change] {
        &self.rejected_changes
    }

    pub fn change_type(&self) -> FileChangeType {
        self.change_type
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Change {
    line: String,
    change_type: LineChangeType,
    line_number: usize,
    change_id: usize,
}

impl Change {
    pub fn line(&self) -> &str {
        &self.line
    }

    pub fn change_type(&self) -> LineChangeType {
        self.change_type
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }

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
    use crate::VersionDiff;

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
}
