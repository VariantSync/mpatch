use std::{fs, path::PathBuf};

use mpatch::{Error, FileArtifact, LCSMatcher};

const RESULT_DIR: &str = "tests/edge_cases/target_variant/version-1";
const SOURCE_DIR: &str = "tests/edge_cases/source_variant/version-0";
const TARGET_DIR: &str = "tests/edge_cases/target_variant/version-0";

const ADDED_FILE_DIFF: &str = "tests/edge_cases/diffs/added_file.diff";
const ADDED_FILE_ACTUAL_RESULT: &str = "tests/edge_cases/target_variant/version-1/added_file.c";
const ADDED_FILE_EXPECTED_RESULT: &str = "tests/edge_cases/source_variant/version-1/added_file.c";

const MISSING_TARGET_DIFF: &str = "tests/edge_cases/diffs/missing_target.diff";
const MISSING_TARGET_ACTUAL_RESULT: &str =
    "tests/edge_cases/target_variant/version-1/missing_target.c";
const MISSING_TARGET_EXPECTED_RESULT: &str =
    "tests/edge_cases/source_variant/version-1/missing_target.c";

const REMOVED_FILE_DIFF: &str = "tests/edge_cases/diffs/removed_file.diff";
const REMOVED_ACTUAL_RESULT: &str = "tests/edge_cases/target_variant/version-1/removed_file.c";
const REMOVED_FILE_EXPECTED_RESULT: &str =
    "tests/edge_cases/source_variant/version-1/removed_file.c";

const RENAMED_FILE_DIFF: &str = "tests/edge_cases/diffs/renamed_file.diff";
const RENAMED_ACTUAL_RESULT: &str = "tests/edge_cases/target_variant/version-1/file_renamed.c";
const RENAMED_FILE_EXPECTED_RESULT: &str =
    "tests/edge_cases/source_variant/version-1/file_renamed.c";

fn clean_result_dir(file_path: &str) {
    fs::remove_file(file_path).unwrap();
    fs::read_dir(RESULT_DIR).unwrap();
}

#[test]
fn added_file() -> Result<(), Error> {
    mpatch::apply_all(
        as_path(SOURCE_DIR),
        as_path(TARGET_DIR),
        as_path(ADDED_FILE_DIFF),
        None,
        1,
        false,
        LCSMatcher,
    )?;
    compare_actual_and_expected(ADDED_FILE_ACTUAL_RESULT, ADDED_FILE_EXPECTED_RESULT)?;
    clean_result_dir(ADDED_FILE_ACTUAL_RESULT);
    Ok(())
}

#[test]
fn removed_file() -> Result<(), Error> {
    mpatch::apply_all(
        as_path(SOURCE_DIR),
        as_path(TARGET_DIR),
        as_path(REMOVED_FILE_DIFF),
        None,
        1,
        false,
        LCSMatcher,
    )?;
    compare_actual_and_expected(REMOVED_ACTUAL_RESULT, REMOVED_FILE_EXPECTED_RESULT)?;
    clean_result_dir(REMOVED_ACTUAL_RESULT);
    Ok(())
}

#[test]
fn missing_target() -> Result<(), Error> {
    mpatch::apply_all(
        as_path(SOURCE_DIR),
        as_path(TARGET_DIR),
        as_path(MISSING_TARGET_DIFF),
        None,
        1,
        false,
        LCSMatcher,
    )?;
    compare_actual_and_expected(MISSING_TARGET_ACTUAL_RESULT, MISSING_TARGET_EXPECTED_RESULT)?;
    clean_result_dir(MISSING_TARGET_ACTUAL_RESULT);
    Ok(())
}

#[test]
fn renamed_file() -> Result<(), Error> {
    mpatch::apply_all(
        as_path(SOURCE_DIR),
        as_path(TARGET_DIR),
        as_path(RENAMED_FILE_DIFF),
        None,
        1,
        false,
        LCSMatcher,
    )?;
    compare_actual_and_expected(RENAMED_ACTUAL_RESULT, RENAMED_FILE_EXPECTED_RESULT)?;
    clean_result_dir(RENAMED_ACTUAL_RESULT);
    Ok(())
}

fn compare_actual_and_expected(path_actual: &str, path_expected: &str) -> Result<(), Error> {
    let expected = FileArtifact::read(path_expected);
    let actual = FileArtifact::read(path_actual);

    if let Ok(expected) = expected {
        let actual = actual.unwrap();
        assert_eq!(expected.len(), actual.len());
        for (i, (expected, actual)) in expected
            .into_lines()
            .into_iter()
            .zip(actual.into_lines().into_iter())
            .enumerate()
        {
            assert_eq!(expected, actual, "lines {} differ", i)
        }
    } else {
        assert!(actual.is_err());
    }

    Ok(())
}

fn as_path(p: &str) -> PathBuf {
    PathBuf::from(p)
}
