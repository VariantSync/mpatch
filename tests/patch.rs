// TODO: Where is the alignment calculated? In a matching? On a change? Someplace else?
use mpatch::{
    patch::{AlignedPatch, Patch},
    CommitDiff, FileArtifact, LCSMatcher, Matcher,
};

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

fn read_patch(path: &str) -> Patch {
    let diff = CommitDiff::read(path).unwrap();
    Patch::from(diff.file_diffs().first().unwrap().clone())
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
fn mixed_alignment() {
    run_alignment_test(MIXED_SOURCE, MIXED_TARGET, MIXED_DIFF, EXPECTED_MIXED_PATCH);
}

fn run_alignment_test(source: &str, target: &str, diff: &str, expected_patch: &str) {
    let source = FileArtifact::read(source).unwrap();
    let target = FileArtifact::read(target).unwrap();

    let mut matcher = LCSMatcher;
    let matching = matcher.match_files(&source, &target);

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
    run_application_test(aligned_patch, EXPECTED_INVARIANT_RESULT);
}

#[test]
fn apply_additive() {
    let aligned_patch = get_aligned_patch(ADDITIVE_SOURCE, ADDITIVE_TARGET, ADDITIVE_DIFF);
    run_application_test(aligned_patch, EXPECTED_ADDITIVE_RESULT);
}

#[test]
fn apply_substractive() {
    let aligned_patch =
        get_aligned_patch(SUBSTRACTIVE_SOURCE, SUBSTRACTIVE_TARGET, SUBSTRACTIVE_DIFF);
    run_application_test(aligned_patch, EXPECTED_SUBSTRACTIVE_RESULT);
}

#[test]
fn apply_mixed() {
    let aligned_patch = get_aligned_patch(MIXED_SOURCE, MIXED_TARGET, MIXED_DIFF);
    run_application_test(aligned_patch, EXPECTED_MIXED_RESULT);
}

fn get_aligned_patch(source: &str, target: &str, diff: &str) -> AlignedPatch {
    let source = FileArtifact::read(source).unwrap();
    let target = FileArtifact::read(target).unwrap();

    let mut matcher = LCSMatcher;
    let matching = matcher.match_files(&source, &target);

    let patch = read_patch(diff);
    patch.align_to_target(matching)
}

fn run_application_test(aligned_patch: AlignedPatch, expected_result: &str) {
    let expected_result = FileArtifact::read(expected_result).unwrap();

    let actual_result = aligned_patch.apply();
    assert_eq!(expected_result.lines().len(), actual_result.lines().len());
    for (expected, actual) in expected_result
        .lines()
        .iter()
        .zip(actual_result.lines().iter())
    {
        println!("exp: {expected}\nact: {actual}");
        assert_eq!(expected, actual);
    }
}
