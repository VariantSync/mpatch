// TODO: Where is the alignment calculated? In a matching? On a change? Someplace else?
use mpatch::{patch::Patch, CommitDiff, FileArtifact, LCSMatcher, Matcher};

const INVARIANT_SOURCE: &str = "tests/samples/source_variant/version-0/invariant.c";
const INVARIANT_TARGET: &str = "tests/samples/target_variant/version-0/invariant.c";
const INVARIANT_DIFF: &str = "tests/diffs/invariant.diff";
const EXPECTED_INVARIANT_PATCH: &str = "tests/expected_patches/invariant.diff";

const ADDITIVE_SOURCE: &str = "tests/samples/source_variant/version-0/additive.c";
const ADDITIVE_TARGET: &str = "tests/samples/target_variant/version-0/additive.c";
const ADDITIVE_DIFF: &str = "tests/diffs/additive.diff";
const EXPECTED_ADDITIVE_PATCH: &str = "tests/expected_patches/additive.diff";

const SUBSTRACTIVE_SOURCE: &str = "tests/samples/source_variant/version-0/substractive.c";
const SUBSTRACTIVE_TARGET: &str = "tests/samples/target_variant/version-0/substractive.c";
const SUBSTRACTIVE_DIFF: &str = "tests/diffs/substractive.diff";
const EXPECTED_SUBSTRACTIVE_PATCH: &str = "tests/expected_patches/substractive.diff";

fn read_patch(path: &str) -> Patch {
    let diff = CommitDiff::read(path).unwrap();
    Patch::from(diff.file_diffs().first().unwrap().clone())
}

#[test]
fn alignment_unchanged() {
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
    unimplemented!();
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
