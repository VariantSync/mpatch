use std::{fmt::Display, fs, path::Path, vec};

use crate::{matching::Matching, Error, FileArtifact, FileDiff};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePatch {
    changes: Vec<Change>,
    change_type: FileChangeType,
}

impl FilePatch {
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

        let mut changes = Vec::with_capacity(self.changes.len());
        let mut rejected_changes = vec![];
        for mut change in self.changes {
            let target_line_number = match change.change_type {
                LineChangeType::Add => target_matching
                    .target_index_fuzzy(change.line_number)
                    .map(|match_id| match_id.unwrap_or(0)),
                LineChangeType::Remove => target_matching
                    .target_index(change.line_number)
                    .expect("the source line was never matched"),
            };
            if let Some(target_line_number) = target_line_number {
                change.line_number = target_line_number;
                changes.push(change);
            } else {
                rejected_changes.push(change);
            }
        }
        AlignedPatch {
            changes,
            rejected_changes,
            target: target_matching.into_target(),
            change_type: self.change_type,
        }
    }

    pub fn align_to_multiple_targets(&self, target_matchings: Vec<Matching>) -> Vec<AlignedPatch> {
        let mut aligned_patches = Vec::with_capacity(target_matchings.len());
        for matching in target_matchings.into_iter() {
            aligned_patches.push(self.clone().align_to_target(matching));
        }
        aligned_patches
    }

    pub fn changes(&self) -> &[Change] {
        self.changes.as_ref()
    }
}

impl From<FileDiff> for FilePatch {
    fn from(file_diff: FileDiff) -> Self {
        let mut changes = vec![];

        let first_hunk = file_diff.hunks().first().expect("no hunk in diff");
        let file_change_type = if first_hunk.source_location().hunk_start() == 0 {
            FileChangeType::Create
        } else if first_hunk.target_location().hunk_start() == 0 {
            FileChangeType::Remove
        } else {
            FileChangeType::Modify
        };

        for line in file_diff.into_changes() {
            let line_number;
            let change_type;
            match line.line_type() {
                crate::diffs::LineType::Add => {
                    change_type = LineChangeType::Add;
                    line_number = line.source_line().change_location();
                }
                crate::diffs::LineType::Remove => {
                    change_type = LineChangeType::Remove;
                    line_number = line.source_line().real_location();
                }
                _ => panic!("a change must always be an Add or Remove"),
            }

            changes.push(Change {
                line: line.into_text(),
                change_type,
                line_number,
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

        let mut line_number = 1;
        let mut patched_lines = vec![];
        'lines_loop: for line in lines {
            while match changes.peek() {
                Some(change) => change.line_number == line_number,
                None => false,
            } {
                let change = changes.next().expect("there should be a change to extract");
                match change.change_type {
                    LineChangeType::Add => {
                        // add this line to the vector of patched lines
                        patched_lines.push(change.line);
                        line_number += 1;
                    }
                    LineChangeType::Remove => {
                        // remove this line by skipping it
                        assert_eq!(line, change.line);
                        continue 'lines_loop;
                    }
                }
            }
            // once all changes for this line_number have been applied, we can add the next
            // unchanged line
            patched_lines.push(line);
            line_number += 1;
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
            FileChangeType::Remove => write!(f, "Create"),
            FileChangeType::Modify => write!(f, "Create"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::CommitDiff;

    use super::{Change, FilePatch, LineChangeType};

    #[test]
    fn patch_from_diff() {
        let file_diff = CommitDiff::read("tests/diffs/simple.diff").unwrap();
        let file_diff = file_diff.file_diffs().first().unwrap().clone();

        let expected_changes = [
            Change {
                line: "REMOVED".to_string(),
                change_type: LineChangeType::Remove,
                line_number: 4,
            },
            Change {
                line: "ADDED".to_string(),
                change_type: LineChangeType::Add,
                line_number: 4,
            },
            Change {
                line: "REMOVED".to_string(),
                change_type: LineChangeType::Remove,
                line_number: 26,
            },
            Change {
                line: "ADDED".to_string(),
                change_type: LineChangeType::Add,
                line_number: 26,
            },
        ];

        let patch = FilePatch::from(file_diff);

        for (change, expected_change) in patch.changes.into_iter().zip(expected_changes.into_iter())
        {
            assert_eq!(change, expected_change);
        }
    }
}
