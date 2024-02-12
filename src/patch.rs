use crate::{matching::Matching, FileArtifact, FileDiff};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePatch {
    changes: Vec<Change>,
}

impl FilePatch {
    pub fn align_to_target(self, target_matching: Matching) -> AlignedPatch {
        let mut changes = Vec::with_capacity(self.changes.len());
        let mut rejected_changes = vec![];
        for mut change in self.changes {
            let target_line_number = match change.change_type {
                ChangeType::Add => target_matching
                    .target_index_fuzzy(change.line_number)
                    .map(|match_id| match_id.unwrap_or(0)),
                ChangeType::Remove => target_matching
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
        for line in file_diff.into_changes() {
            let line_number;
            let change_type;
            match line.line_type() {
                crate::diffs::LineType::Add => {
                    change_type = ChangeType::Add;
                    line_number = line.source_line().change_location();
                }
                crate::diffs::LineType::Remove => {
                    change_type = ChangeType::Remove;
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

        FilePatch { changes }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignedPatch {
    changes: Vec<Change>,
    rejected_changes: Vec<Change>,
    target: FileArtifact,
}

impl AlignedPatch {
    pub fn changes(&self) -> &[Change] {
        self.changes.as_ref()
    }

    pub fn target(&self) -> &FileArtifact {
        &self.target
    }

    pub fn apply(self) -> PatchOutcome {
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
                    ChangeType::Add => {
                        // add this line to the vector of patched lines
                        patched_lines.push(change.line);
                        line_number += 1;
                    }
                    ChangeType::Remove => {
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

        PatchOutcome {
            patched_file: FileArtifact::from_lines(path, patched_lines),
            rejected_changes: self.rejected_changes,
        }
    }
}

pub struct PatchOutcome {
    patched_file: FileArtifact,
    rejected_changes: Vec<Change>,
}

impl PatchOutcome {
    pub fn patched_file(&self) -> &FileArtifact {
        &self.patched_file
    }

    pub fn rejected_changes(&self) -> &[Change] {
        self.rejected_changes.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Change {
    line: String,
    change_type: ChangeType,
    line_number: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChangeType {
    Add,
    Remove,
}

#[cfg(test)]
mod tests {
    use crate::CommitDiff;

    use super::{Change, ChangeType, FilePatch};

    #[test]
    fn patch_from_diff() {
        let file_diff = CommitDiff::read("tests/diffs/simple.diff").unwrap();
        let file_diff = file_diff.file_diffs().first().unwrap().clone();

        let expected_changes = [
            Change {
                line: "REMOVED".to_string(),
                change_type: ChangeType::Remove,
                line_number: 4,
            },
            Change {
                line: "ADDED".to_string(),
                change_type: ChangeType::Add,
                line_number: 4,
            },
            Change {
                line: "REMOVED".to_string(),
                change_type: ChangeType::Remove,
                line_number: 26,
            },
            Change {
                line: "ADDED".to_string(),
                change_type: ChangeType::Add,
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
