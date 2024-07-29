use crate::{FilePatch, Matching};

use super::{Change, FilteredPatch, LineChangeType};

pub trait Filter {
    fn apply_filter(&mut self, patch: FilePatch, matching: &Matching) -> FilteredPatch;
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
    fn apply_filter(&mut self, patch: FilePatch, matching: &Matching) -> FilteredPatch {
        let mut changes = vec![];
        let mut rejected_changes = vec![];

        patch.changes.into_iter().for_each(|c| {
            if self.keep_change(&c, matching) {
                changes.push(c);
            } else {
                rejected_changes.push(c);
            };
        });
        FilteredPatch {
            change_type: patch.change_type,
            changes,
            rejected_changes,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InsideMatchFilter(usize);

impl InsideMatchFilter {
    pub fn new(min_deepness: usize) -> InsideMatchFilter {
        InsideMatchFilter(min_deepness)
    }

    fn keep_change(&self, change: &Change, matching: &Matching) -> bool {
        let line_number = change.line_number;
        if change.change_type == LineChangeType::Remove {
            // Removes are filteres by the alignment in any case
            return matching.target_index(line_number).is_some();
        }

        // Check the deepness of the change in a match and if it is not deep enough, reject the
        // change
        for i in 0..self.0 {
            if matching.target_index(line_number - i).is_none() {
                return false;
            }

            if matching.target_index(line_number + i).is_none() {
                return false;
            }
        }
        true
    }
}

impl Filter for InsideMatchFilter {
    fn apply_filter(&mut self, patch: FilePatch, matching: &Matching) -> FilteredPatch {
        let mut changes = vec![];
        let mut rejected_changes = vec![];

        patch.changes.into_iter().for_each(|c| {
            if self.keep_change(&c, matching) {
                changes.push(c);
            } else {
                rejected_changes.push(c);
            };
        });
        FilteredPatch {
            change_type: patch.change_type,
            changes,
            rejected_changes,
        }
    }
}

#[derive(Debug)]
pub struct KeepAllFilter;

impl Filter for KeepAllFilter {
    fn apply_filter(&mut self, patch: FilePatch, _: &Matching) -> FilteredPatch {
        FilteredPatch {
            changes: patch.changes,
            change_type: patch.change_type,
            rejected_changes: vec![],
        }
    }
}
