use mpatch::{
    patch::{AlignedPatch, FilePatch},
    CommitDiff, FileArtifact, LCSMatcher, Matcher,
};

pub fn get_aligned_patch(source: &str, target: &str, diff: &str) -> AlignedPatch {
    let source = FileArtifact::read(source).unwrap();
    let target = FileArtifact::read(target).unwrap();

    let mut matcher = LCSMatcher;
    let matching = matcher.match_files(source, target);

    let patch = read_patch(diff);
    patch.align_to_target(matching)
}

pub fn run_alignment_test(source: &str, target: &str, diff: &str, expected_patch: &str) {
    let source = FileArtifact::read(source).unwrap();
    let target = FileArtifact::read(target).unwrap();

    let mut matcher = LCSMatcher;
    let matching = matcher.match_files(source, target);

    let patch = read_patch(diff);
    let expected_patch = read_patch(expected_patch);
    let aligned_patch = patch.align_to_target(matching);

    for (expected, aligned) in expected_patch
        .changes()
        .iter()
        .zip(aligned_patch.changes().iter())
    {
        assert_eq!(expected, aligned);
    }
}

pub fn run_application_test(
    aligned_patch: AlignedPatch,
    expected_result: &str,
    expected_rejects_count: usize,
) {
    let expected_result = FileArtifact::read(expected_result).unwrap();

    let actual_result = aligned_patch.apply();
    let (actual_result, rejects) = (
        actual_result.patched_file(),
        actual_result.rejected_changes(),
    );

    assert_eq!(expected_result.lines().len(), actual_result.lines().len());
    assert_eq!(rejects.len(), expected_rejects_count);
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
    let diff = CommitDiff::read(path).unwrap();
    FilePatch::from(diff.file_diffs().first().unwrap().clone())
}
