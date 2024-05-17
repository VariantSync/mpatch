use crate::{FilePatch, Matching};

use super::{Change, LineChangeType};

pub trait Filter {
    fn apply_filter(&mut self, patch: FilePatch, matching: &Matching) -> FilePatch;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DistanceFilter(usize);

impl DistanceFilter {
    pub fn new(max_distance: usize) -> DistanceFilter {
        DistanceFilter(max_distance)
    }

    fn keep_change(&self, change: &Change, matching: &Matching) -> bool {
        if change.change_type == LineChangeType::Remove {
            // Removes are filteres by the alignment in any case
            return true;
        }
        // Determine the best target line for each change
        let (_, match_offset) = matching.target_index_fuzzy(change.line_number);
        match_offset.0 < self.0
    }
}

impl Filter for DistanceFilter {
    fn apply_filter(&mut self, patch: FilePatch, matching: &Matching) -> FilePatch {
        FilePatch {
            change_type: patch.change_type,
            changes: patch
                .changes
                .into_iter()
                .filter(|c| self.keep_change(c, matching))
                .collect(),
        }
    }
}
