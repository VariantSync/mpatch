pub mod test_utils;

use test_utils::{get_aligned_patch, run_alignment_test, run_application_test};

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

const APPENDING_SOURCE: &str = "tests/samples/source_variant/version-0/appending.c";
const APPENDING_TARGET: &str = "tests/samples/target_variant/version-0/appending.c";
const APPENDING_DIFF: &str = "tests/diffs/appending.diff";
const EXPECTED_APPENDING_PATCH: &str = "tests/expected_patches/appending.diff";
const EXPECTED_APPENDING_RESULT: &str = "tests/samples/target_variant/version-1/appending.c";

const PREPENDING_SOURCE: &str = "tests/samples/source_variant/version-0/prepending.c";
const PREPENDING_TARGET: &str = "tests/samples/target_variant/version-0/prepending.c";
const PREPENDING_DIFF: &str = "tests/diffs/prepending.diff";
const EXPECTED_PREPENDING_PATCH: &str = "tests/expected_patches/prepending.diff";
const EXPECTED_PREPENDING_RESULT: &str = "tests/samples/target_variant/version-1/prepending.c";

const ANCHOR_BELOW_SOURCE: &str = "tests/samples/source_variant/version-0/anchor_below.c";
const ANCHOR_BELOW_TARGET: &str = "tests/samples/target_variant/version-0/anchor_below.c";
const ANCHOR_BELOW_DIFF: &str = "tests/diffs/anchor_below.diff";
const EXPECTED_ANCHOR_BELOW_PATCH: &str = "tests/expected_patches/anchor_below.diff";
const EXPECTED_ANCHOR_BELOW_RESULT: &str = "tests/samples/target_variant/version-1/anchor_below.c";

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

#[test]
fn appending_alignment() {
    run_alignment_test(
        APPENDING_SOURCE,
        APPENDING_TARGET,
        APPENDING_DIFF,
        EXPECTED_APPENDING_PATCH,
    );
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

#[test]
fn apply_appending() {
    let aligned_patch = get_aligned_patch(APPENDING_SOURCE, APPENDING_TARGET, APPENDING_DIFF);
    run_application_test(aligned_patch, EXPECTED_APPENDING_RESULT, 0);
}

#[test]
fn prepending_alignment() {
    run_alignment_test(
        PREPENDING_SOURCE,
        PREPENDING_TARGET,
        PREPENDING_DIFF,
        EXPECTED_PREPENDING_PATCH,
    );
}

#[test]
fn apply_prepending() {
    let aligned_patch = get_aligned_patch(PREPENDING_SOURCE, PREPENDING_TARGET, PREPENDING_DIFF);
    run_application_test(aligned_patch, EXPECTED_PREPENDING_RESULT, 0);
}

#[test]
fn anchor_below_alignment() {
    run_alignment_test(
        ANCHOR_BELOW_SOURCE,
        ANCHOR_BELOW_TARGET,
        ANCHOR_BELOW_DIFF,
        EXPECTED_ANCHOR_BELOW_PATCH,
    );
}

#[test]
fn apply_anchor_below() {
    let aligned_patch =
        get_aligned_patch(ANCHOR_BELOW_SOURCE, ANCHOR_BELOW_TARGET, ANCHOR_BELOW_DIFF);
    run_application_test(aligned_patch, EXPECTED_ANCHOR_BELOW_RESULT, 0);
}
