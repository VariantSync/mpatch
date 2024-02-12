use similar::TextDiff;

use crate::FileArtifact;

pub trait Matcher {
    fn match_files(&mut self, source: FileArtifact, target: FileArtifact) -> Matching;
}

pub struct Matching {
    source: FileArtifact,
    target: FileArtifact,
    source_to_target: Vec<MatchId>,
    target_to_source: Vec<MatchId>,
}

pub type MatchId = Option<usize>;

impl Matching {
    pub fn target_index(&self, source_index: usize) -> Option<MatchId> {
        // To represent line numbers in files we offset the index by '1'
        // A negative offset is applied to the input index (e.g., line 1 is stored at index 0)
        // A positive offset is applied to the retrieved counterpart index (e.g., the counterpart
        // of line 1 is also line 1, which is stored as a 0).
        self.source_to_target
            .get(source_index - 1)
            .copied()
            .map(|v| v.map(|v| v + 1))
    }

    pub fn source_index(&self, target_index: usize) -> Option<MatchId> {
        self.target_to_source
            .get(target_index - 1)
            .copied()
            .map(|v| v.map(|v| v + 1))
    }

    pub fn source(&self) -> &FileArtifact {
        &self.source
    }

    pub fn target(&self) -> &FileArtifact {
        &self.target
    }

    pub fn into_source(self) -> FileArtifact {
        self.source
    }

    pub fn into_target(self) -> FileArtifact {
        self.target
    }

    pub(crate) fn target_index_fuzzy(&self, line_number: usize) -> Option<MatchId> {
        let mut line_number = line_number;

        // Search for the closest context line above the change; i.e., key and value must both be
        // Some(...)
        while line_number > 0 && self.target_index(line_number).flatten().is_none() {
            line_number -= 1;
        }

        if line_number == 0 {
            // Line numbers start at '1', so there is no valid target index for '0'
            None
        } else {
            let target_line = self.target_index(line_number);
            // The result must be Some(...) in all cases
            assert!(target_line.is_some());
            target_line
        }
    }
}

pub struct LCSMatcher;

impl LCSMatcher {
    pub fn new() -> Self {
        LCSMatcher
    }
}

impl Default for LCSMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Matcher for LCSMatcher {
    fn match_files(&mut self, left: FileArtifact, right: FileArtifact) -> Matching {
        let left_text = left.to_string();
        let right_text = right.to_string();
        let text_diff = TextDiff::from_lines(&left_text, &right_text);
        let mut left_to_right = Vec::with_capacity(left.len());
        let mut right_to_left = Vec::with_capacity(right.len());

        for c in text_diff.iter_all_changes() {
            if c.old_index().is_some() {
                assert_eq!(c.old_index().unwrap(), left_to_right.len());
                left_to_right.push(c.new_index());
            }
            if c.new_index().is_some() {
                assert_eq!(c.new_index().unwrap(), right_to_left.len());
                right_to_left.push(c.old_index());
            }
        }
        Matching {
            source: left,
            target: right,
            source_to_target: left_to_right,
            target_to_source: right_to_left,
        }
    }
}

#[cfg(test)]
mod tests {}
