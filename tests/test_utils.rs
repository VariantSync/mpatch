use mpatch::{
    alignment::align_patch_to_target, application::apply_patch, patch::Change, AlignedPatch,
    FileArtifact, FilePatch, LCSMatcher, Matcher, VersionDiff,
};

pub fn run_alignment_test(source: &str, target: &str, diff: &str, expected_patch: &str) {
    let source = FileArtifact::read(source).unwrap();
    let target = FileArtifact::read(target).unwrap();

    let mut matcher = LCSMatcher;
    let matching = matcher.match_files(source, target);

    let patch = read_patch(diff);
    let expected_patch = read_patch(expected_patch);
    let aligned_patch = align_patch_to_target(patch, matching);

    for (expected, aligned) in expected_patch
        .changes()
        .iter()
        .zip(aligned_patch.changes().iter())
    {
        assert_change_equality(expected, aligned);
    }
}

pub fn assert_change_equality(c1: &Change, c2: &Change) {
    assert_eq!(c1.change_type(), c2.change_type());
    assert_eq!(c1.line(), c2.line());
    assert_eq!(c1.line_number(), c2.line_number());
}

pub fn run_application_test(
    aligned_patch: AlignedPatch,
    expected_result: &str,
    expected_rejects_count: usize,
) {
    let expected_result = FileArtifact::read(expected_result).unwrap();

    let actual_result = apply_patch(aligned_patch, true).unwrap();
    let (actual_result, rejects) = (
        actual_result.patched_file(),
        actual_result.rejected_changes(),
    );

    assert_eq!(expected_result.lines().len(), actual_result.lines().len());
    assert_eq!(rejects.len(), expected_rejects_count);

    if !rejects.is_empty() {
        println!("Found rejects:");
        for reject in rejects {
            println!("{}: {reject}", reject.change_id());
        }
        println!();
    }

    for (expected, actual) in expected_result
        .lines()
        .iter()
        .zip(actual_result.lines().iter())
    {
        println!("exp: {expected}\nact: {actual}");
        assert_eq!(expected, actual);
    }
}

pub fn read_patch(path: &str) -> FilePatch {
    let diff = VersionDiff::read(path)
        .unwrap()
        .file_diffs()
        .first()
        .unwrap()
        .clone();
    println!("read patch:");
    println!("{}", diff.header());
    FilePatch::from(diff)
}

pub fn get_aligned_patch(source: &str, target: &str, diff: &str) -> AlignedPatch {
    let source = FileArtifact::read(source).unwrap();
    let target = FileArtifact::read(target).unwrap();

    let mut matcher = LCSMatcher;
    let matching = matcher.match_files(source, target);

    let patch = read_patch(diff);
    align_patch_to_target(patch, matching)
}
