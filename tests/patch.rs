// TODO: Where is the alignment calculated? In a matching? On a change? Someplace else?
use mpatch::{CommitDiff, FileArtifact, LCSMatcher, Matcher};

const INVARIANT_SOURCE_V0: &str = "tests/samples/source_variant/version-0/invariant.c";
const INVARIANT_SOURCE_V1: &str = "tests/samples/source_variant/version-1/invariant.c";
const INVARIANT_TARGET_V0: &str = "tests/samples/target_variant/version-0/invariant.c";
const DIFF_FILE: &str = "tests/samples/source_variant/patch.diff";

#[test]
fn alignment_unchanged() {
    let mut patch = CommitDiff::read(DIFF_FILE).unwrap();
    let source_v0 = FileArtifact::read(INVARIANT_SOURCE_V0).unwrap();
    let target_v0 = FileArtifact::read(INVARIANT_TARGET_V0).unwrap();

    let mut matcher = LCSMatcher;
    let matching = matcher.match_files(&source_v0, &target_v0);
}

#[test]
fn additive_alignment() {
    unimplemented!();
}

#[test]
fn substractive_alignment() {
    unimplemented!();
}

#[test]
fn mixed_alignment() {
    unimplemented!();
}
