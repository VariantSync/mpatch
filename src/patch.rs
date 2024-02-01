use crate::{matching::Matching, FileArtifact, FileDiff};

pub struct Patch {
    changes: Vec<Change>,
}

impl Patch {
    pub fn align_to_target(self, target: FileArtifact, matching: &Matching) -> AlignedPatch {
        todo!();
    }

    pub fn align_to_multiple_targets(
        &self,
        targets: Vec<(FileArtifact, &Matching)>,
    ) -> Vec<AlignedPatch> {
        todo!();
    }
}

impl From<FileDiff> for Patch {
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

        Patch { changes }
    }
}

pub struct AlignedPatch {
    changes: Vec<Change>,
    target: FileArtifact,
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
    use crate::{CommitDiff, FileDiff};

    use super::{Change, ChangeType, Patch};

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

        let patch = Patch::from(file_diff);

        for (change, expected_change) in patch.changes.into_iter().zip(expected_changes.into_iter())
        {
            assert_eq!(change, expected_change);
        }
    }
}
