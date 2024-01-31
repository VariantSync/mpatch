use similar::TextDiff;

use crate::FileArtifact;

pub trait Matcher {
    fn match_files<'a>(left: &'a FileArtifact, right: &'a FileArtifact) -> Matching<'a>;
}

pub struct Matching<'a> {
    left: &'a FileArtifact,
    right: &'a FileArtifact,
    left_to_right: Vec<MatchId>,
    right_to_left: Vec<MatchId>,
}

pub type MatchId = Option<usize>;

impl<'a> Matching<'a> {
    pub fn right_index_for(&self, left_index: usize) -> Option<MatchId> {
        // To represent line numbers in files we offset the index by '1'
        // A negative offset is applied to the input index (e.g., line 1 is stored at index 0)
        // A positive offset is applied to the retrieved counterpart index (e.g., the counterpart
        // of line 1 is also line 1, which is stored as a 0).
        self.left_to_right
            .get(left_index - 1)
            .copied()
            .map(|v| v.map(|v| v + 1))
    }

    pub fn left_index_for(&self, right_index: usize) -> Option<MatchId> {
        self.right_to_left
            .get(right_index - 1)
            .copied()
            .map(|v| v.map(|v| v + 1))
    }
}

pub struct LCSMatcher {}

impl LCSMatcher {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for LCSMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Matcher for LCSMatcher {
    fn match_files<'a>(left: &'a FileArtifact, right: &'a FileArtifact) -> Matching<'a> {
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
            left,
            right,
            left_to_right,
            right_to_left,
        }
    }
}

#[cfg(test)]
mod tests {}
