pub mod test_utils;
use mpatch::{
    filtering::{DistanceFilter, Filter},
    FileArtifact, LCSMatcher, Matcher,
};
use test_utils::{assert_change_equality, read_patch};

const SOURCE: &str = "tests/filter/samples/source_variant/version-0/main.c";
const TARGET: &str = "tests/filter/samples/target_variant/version-0/main.c";
const DIFF: &str = "tests/filter/diffs/main.diff";
const EXPECTED_PATCH_0: &str = "tests/filter/expected_patches/distance_0.diff";
const EXPECTED_PATCH_1: &str = "tests/filter/expected_patches/distance_1.diff";
const EXPECTED_PATCH_3: &str = "tests/filter/expected_patches/distance_3.diff";
const EXPECTED_PATCH_10: &str = "tests/filter/expected_patches/distance_10.diff";

#[test]
fn distance_0() {
    let mut filter = DistanceFilter::new(0);
    run_filter_test(&mut filter, SOURCE, TARGET, DIFF, EXPECTED_PATCH_0);
}

#[test]
fn distance_1() {
    let mut filter = DistanceFilter::new(1);
    run_filter_test(&mut filter, SOURCE, TARGET, DIFF, EXPECTED_PATCH_1);
}

#[test]
fn distance_3() {
    let mut filter = DistanceFilter::new(2);
    run_filter_test(&mut filter, SOURCE, TARGET, DIFF, EXPECTED_PATCH_3);
}

#[test]
fn distance_10() {
    let mut filter = DistanceFilter::new(10);
    run_filter_test(&mut filter, SOURCE, TARGET, DIFF, EXPECTED_PATCH_10);
}

pub fn run_filter_test(
    filter: &mut impl Filter,
    source: &str,
    target: &str,
    diff: &str,
    expected_patch: &str,
) {
    let source = FileArtifact::read(source).unwrap();
    let target = FileArtifact::read(target).unwrap();

    let mut matcher = LCSMatcher;
    let matching = matcher.match_files(source, target);

    let patch = read_patch(diff);
    let expected_patch = read_patch(expected_patch);

    let filtered_patch = filter.apply_filter(patch, &matching);

    for (expected, aligned) in expected_patch
        .changes()
        .iter()
        .zip(filtered_patch.changes().iter())
    {
        assert_change_equality(expected, aligned);
    }
}
