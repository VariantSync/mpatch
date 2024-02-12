use mpatch::{
    patch::{AlignedPatch, FilePatch},
    CommitDiff, FileArtifact, LCSMatcher, Matcher,
};

// TODO: Test multi-alignment
// TODO: Test file creation
// TODO: Test file removal
// TODO: Test file renaming
// TODO: Test file permission change
// TODO: Test patch application to entire directory
// TODO: Test missing target files
// TODO: Test rejects file writing

const INVARIANT_SOURCE: &str = "tests/samples/source_variant/version-0/invariant.c";
const INVARIANT_TARGET: &str = "tests/samples/target_variant/version-0/invariant.c";
const INVARIANT_DIFF: &str = "tests/diffs/invariant.diff";
const EXPECTED_INVARIANT_PATCH: &str = "tests/expected_patches/invariant.diff";
const EXPECTED_INVARIANT_RESULT: &str = "tests/samples/target_variant/version-1/invariant.c";

const ADDITIVE_SOURCE: &str = "tests/samples/source_variant/version-0/additive.c";
const ADDITIVE_TARGET: &str = "tests/samples/target_variant/version-0/additive.c";
const ADDITIVE_DIFF: &str = "tests/diffs/additive.diff";
const EXPECTED_ADDITIVE_PATCH: &str = "tests/expected_patches/additive.diff";
const EXPECTED_ADDITIVE_RESULT: &str = "tests/samples/target_variant/version-1/additive.c";

const SUBSTRACTIVE_SOURCE: &str = "tests/samples/source_variant/version-0/substractive.c";
const SUBSTRACTIVE_TARGET: &str = "tests/samples/target_variant/version-0/substractive.c";
const SUBSTRACTIVE_DIFF: &str = "tests/diffs/substractive.diff";
const EXPECTED_SUBSTRACTIVE_PATCH: &str = "tests/expected_patches/substractive.diff";
const EXPECTED_SUBSTRACTIVE_RESULT: &str = "tests/samples/target_variant/version-1/substractive.c";

const MIXED_SOURCE: &str = "tests/samples/source_variant/version-0/mixed.c";
const MIXED_TARGET: &str = "tests/samples/target_variant/version-0/mixed.c";
const MIXED_DIFF: &str = "tests/diffs/mixed.diff";
const EXPECTED_MIXED_PATCH: &str = "tests/expected_patches/mixed.diff";
const EXPECTED_MIXED_RESULT: &str = "tests/samples/target_variant/version-1/mixed.c";

const NON_EXISTANT_SOURCE: &str = "tests/samples/source_variant/version-0/remove_non_existant.c";
const NON_EXISTANT_TARGET: &str = "tests/samples/target_variant/version-0/remove_non_existant.c";
const NON_EXISTANT_DIFF: &str = "tests/diffs/remove_non_existant.diff";
const EXPECTED_NON_EXISTANT_PATCH: &str = "tests/expected_patches/remove_non_existant.diff";
const EXPECTED_NON_EXISTANT_RESULT: &str =
    "tests/samples/target_variant/version-1/remove_non_existant.c";

fn read_patch(path: &str) -> FilePatch {
    let diff = CommitDiff::read(path).unwrap();
    FilePatch::from(diff.file_diffs().first().unwrap().clone())
}

#[test]
fn invariant_alignment() {
    run_alignment_test(
        INVARIANT_SOURCE,
        INVARIANT_TARGET,
        INVARIANT_DIFF,
        EXPECTED_INVARIANT_PATCH,
    );
}

#[test]
fn additive_alignment() {
    run_alignment_test(
        ADDITIVE_SOURCE,
        ADDITIVE_TARGET,
        ADDITIVE_DIFF,
        EXPECTED_ADDITIVE_PATCH,
    );
}

#[test]
fn substractive_alignment() {
    run_alignment_test(
        SUBSTRACTIVE_SOURCE,
        SUBSTRACTIVE_TARGET,
        SUBSTRACTIVE_DIFF,
        EXPECTED_SUBSTRACTIVE_PATCH,
    );
}

#[test]
fn non_existant_alignment() {
    run_alignment_test(
        NON_EXISTANT_SOURCE,
        NON_EXISTANT_TARGET,
        NON_EXISTANT_DIFF,
        EXPECTED_NON_EXISTANT_PATCH,
    );
}

#[test]
fn mixed_alignment() {
    run_alignment_test(MIXED_SOURCE, MIXED_TARGET, MIXED_DIFF, EXPECTED_MIXED_PATCH);
}

fn run_alignment_test(source: &str, target: &str, diff: &str, expected_patch: &str) {
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

#[test]
fn apply_invariant() {
    let aligned_patch = get_aligned_patch(INVARIANT_SOURCE, INVARIANT_TARGET, INVARIANT_DIFF);
    run_application_test(aligned_patch, EXPECTED_INVARIANT_RESULT, 0);
}

#[test]
fn apply_additive() {
    let aligned_patch = get_aligned_patch(ADDITIVE_SOURCE, ADDITIVE_TARGET, ADDITIVE_DIFF);
    run_application_test(aligned_patch, EXPECTED_ADDITIVE_RESULT, 0);
}

#[test]
fn apply_substractive() {
    let aligned_patch =
        get_aligned_patch(SUBSTRACTIVE_SOURCE, SUBSTRACTIVE_TARGET, SUBSTRACTIVE_DIFF);
    run_application_test(aligned_patch, EXPECTED_SUBSTRACTIVE_RESULT, 0);
}

#[test]
fn apply_mixed() {
    let aligned_patch = get_aligned_patch(MIXED_SOURCE, MIXED_TARGET, MIXED_DIFF);
    run_application_test(aligned_patch, EXPECTED_MIXED_RESULT, 0);
}

#[test]
fn apply_non_existant() {
    let aligned_patch =
        get_aligned_patch(NON_EXISTANT_SOURCE, NON_EXISTANT_TARGET, NON_EXISTANT_DIFF);
    run_application_test(aligned_patch, EXPECTED_NON_EXISTANT_RESULT, 1);
}

fn get_aligned_patch(source: &str, target: &str, diff: &str) -> AlignedPatch {
    let source = FileArtifact::read(source).unwrap();
    let target = FileArtifact::read(target).unwrap();

    let mut matcher = LCSMatcher;
    let matching = matcher.match_files(source, target);

    let patch = read_patch(diff);
    patch.align_to_target(matching)
}

fn run_application_test(
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
