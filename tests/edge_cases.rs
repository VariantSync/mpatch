use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Once,
};

use mpatch::{
    filtering::KeepAllFilter, patch::PatchPaths, Error, FileArtifact, LCSMatcher, VersionDiff,
};

const RESULT_DIR: &str = "tests/edge_cases/target_variant/version-1";
const SOURCE_DIR: &str = "tests/edge_cases/source_variant/version-0";
const TARGET_DIR: &str = "tests/edge_cases/target_variant/version-0";

const BINARY_RESULT_DIR: &str = "tests/binary/target_variant/version-1";
const BINARY_SOURCE_DIR: &str = "tests/binary/source_variant/version-0";
const BINARY_TARGET_DIR: &str = "tests/binary/target_variant/version-1";

const ADDED_FILE_DIFF: &str = "tests/edge_cases/diffs/added_file.diff";
const ADDED_FILE_ACTUAL_RESULT: &str = "tests/edge_cases/target_variant/version-1/added_file.c";
const ADDED_FILE_EXPECTED_RESULT: &str = "tests/edge_cases/source_variant/version-1/added_file.c";

const MISSING_TARGET_DIFF: &str = "tests/edge_cases/diffs/missing_target.diff";
const MISSING_TARGET_ACTUAL_RESULT: &str =
    "tests/edge_cases/target_variant/version-1/missing_target.c";

const REMOVED_FILE_DIFF: &str = "tests/edge_cases/diffs/removed_file.diff";
const REMOVED_ACTUAL_RESULT: &str = "tests/edge_cases/target_variant/version-1/removed_file.c";
const REMOVED_FILE_EXPECTED_RESULT: &str =
    "tests/edge_cases/source_variant/version-1/removed_file.c";

const RENAMED_FILE_DIFF: &str = "tests/edge_cases/diffs/renamed_file.diff";
const RENAMED_ACTUAL_RESULT: &str = "tests/edge_cases/target_variant/version-1/file_renamed.c";
const RENAMED_FILE_EXPECTED_RESULT: &str =
    "tests/edge_cases/source_variant/version-1/file_renamed.c";

const BINARY_FILE_DIFF: &str = "tests/binary/diffs/binary.diff";
const BINARY_FILE_ACTUAL_RESULT: &str = "tests/binary/target_variant/version-1/file_renamed.c";

static INIT_EDGE: Once = Once::new();
static INIT_BINARY: Once = Once::new();

fn prepare_result_dir() {
    INIT_EDGE.call_once(|| {
        fs::create_dir_all(RESULT_DIR).unwrap();
        for file in fs::read_dir(TARGET_DIR).unwrap() {
            let file = file.unwrap();
            let mut target_file = PathBuf::from_str(RESULT_DIR).unwrap();
            target_file.push(file.path().file_name().unwrap());
            fs::copy(file.path(), target_file).unwrap();
        }
    });
    INIT_BINARY.call_once(|| {
        fs::create_dir_all(BINARY_RESULT_DIR).unwrap();
        for file in fs::read_dir(BINARY_TARGET_DIR).unwrap() {
            let file = file.unwrap();
            let mut target_file = PathBuf::from_str(BINARY_RESULT_DIR).unwrap();
            target_file.push(file.path().file_name().unwrap());
            fs::copy(file.path(), target_file).unwrap();
        }
    });
}

#[test]
fn added_file() -> Result<(), Error> {
    prepare_result_dir();
    let _cleaner = FileCleaner(ADDED_FILE_ACTUAL_RESULT);
    let patch_paths = PatchPaths::new(
        as_path(SOURCE_DIR),
        as_path(RESULT_DIR),
        as_path(ADDED_FILE_DIFF),
        None,
    );
    mpatch::apply_all(patch_paths, 1, false, LCSMatcher, KeepAllFilter)?;
    compare_actual_and_expected(ADDED_FILE_ACTUAL_RESULT, ADDED_FILE_EXPECTED_RESULT)?;
    Ok(())
}

#[test]
fn removed_file() -> Result<(), Error> {
    prepare_result_dir();
    let _cleaner = FileCleaner(REMOVED_ACTUAL_RESULT);
    let patch_paths = PatchPaths::new(
        as_path(SOURCE_DIR),
        as_path(RESULT_DIR),
        as_path(REMOVED_FILE_DIFF),
        None,
    );
    mpatch::apply_all(patch_paths, 1, false, LCSMatcher, KeepAllFilter)?;
    compare_actual_and_expected(REMOVED_ACTUAL_RESULT, REMOVED_FILE_EXPECTED_RESULT)?;
    Ok(())
}

#[test]
fn missing_target() -> Result<(), Error> {
    prepare_result_dir();
    let _cleaner = FileCleaner(MISSING_TARGET_ACTUAL_RESULT);
    let patch_paths = PatchPaths::new(
        as_path(SOURCE_DIR),
        as_path(RESULT_DIR),
        as_path(MISSING_TARGET_DIFF),
        None,
    );
    mpatch::apply_all(patch_paths, 1, false, LCSMatcher, KeepAllFilter)?;
    assert!(!Path::exists(&PathBuf::from(MISSING_TARGET_ACTUAL_RESULT)));
    Ok(())
}

#[test]
fn renamed_file() -> Result<(), Error> {
    prepare_result_dir();
    let _cleaner = FileCleaner(RENAMED_ACTUAL_RESULT);
    let patch_paths = PatchPaths::new(
        as_path(SOURCE_DIR),
        as_path(RESULT_DIR),
        as_path(RENAMED_FILE_DIFF),
        None,
    );
    mpatch::apply_all(patch_paths, 1, false, LCSMatcher, KeepAllFilter)?;
    compare_actual_and_expected(RENAMED_ACTUAL_RESULT, RENAMED_FILE_EXPECTED_RESULT)?;
    Ok(())
}

#[test]
fn binary_file() {
    prepare_result_dir();
    let _cleaner = FileCleaner(BINARY_FILE_ACTUAL_RESULT);
    let patch_paths = PatchPaths::new(
        as_path(BINARY_SOURCE_DIR),
        as_path(BINARY_TARGET_DIR),
        as_path(BINARY_FILE_DIFF),
        None,
    );
    if let Err(error) = mpatch::apply_all(patch_paths, 1, false, LCSMatcher, KeepAllFilter) {
        assert_eq!(error.message(), "stream did not contain valid UTF-8");
    } else {
        panic!("binary file patching should not yet be allowed");
    }
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
            assert_eq!(expected, actual, "lines {i} differ")
        }
    } else {
        assert!(actual.is_err());
    }

    Ok(())
}

fn as_path(p: &str) -> PathBuf {
    PathBuf::from(p)
}

struct FileCleaner<'a>(&'a str);

impl<'a> Drop for FileCleaner<'a> {
    fn drop(&mut self) {
        if Path::exists(&PathBuf::from(self.0)) {
            fs::remove_file(self.0).unwrap()
        }
    }
}

#[test]
fn crlf() {
    const DIFF_FILE: &str = "tests/weird_edge_cases/diffs/crlf.diff";
    let diff = VersionDiff::read(DIFF_FILE).unwrap();
    let file_diffs = diff.file_diffs();
    assert_eq!(1, file_diffs.len());
}

#[test]
fn mixed() {
    const DIFF_FILE: &str = "tests/weird_edge_cases/diffs/mixed.diff";
    VersionDiff::read(DIFF_FILE).unwrap();
}
