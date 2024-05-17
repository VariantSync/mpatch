use crate::{AlignedPatch, FilePatch, Matching};

use super::{FileChangeType, LineChangeType};

/// Consumes and aligns the patch to a specific target file based on a matching.
/// The source file in the matching must also be the source file of the FileDiff from which
/// the FilePatch has been created. This means that it is the version of the source file
/// before the changes in the patch have been applied to it.
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
pub fn align_to_target(patch: FilePatch, target_matching: Matching) -> AlignedPatch {
    if patch.change_type == FileChangeType::Create {
        // Files that are to be created are aligned by definition
        return AlignedPatch {
            changes: patch.changes,
            rejected_changes: vec![],
            target: target_matching.into_target(),
            change_type: patch.change_type,
        };
    }

    // Align all changes
    let mut changes = Vec::with_capacity(patch.changes.len());
    let mut rejected_changes = vec![];
    for mut change in patch.changes {
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
        change_type: patch.change_type,
    }
}

/// Clones the patch for each given matching and aligns it to the corresponding target of each
/// matching.
/// The source file in each matching must also be the source file of the FileDiff from which
/// the FilePatch has been created. This means that it is the version of the source file
/// before the changes in the patch have been applied to it.
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
pub fn align_to_multiple_targets(
    patch: &FilePatch,
    target_matchings: Vec<Matching>,
) -> Vec<AlignedPatch> {
    let mut aligned_patches = Vec::with_capacity(target_matchings.len());
    for matching in target_matchings.into_iter() {
        aligned_patches.push(align_to_target(patch.clone(), matching));
    }
    aligned_patches
}
