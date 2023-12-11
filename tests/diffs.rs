use mpatch::Diff;

const DIFF_FILE: &str = "tests/diffs/base_patch.diff";

#[test]
fn parse_header() {
    let diff = Diff::read(DIFF_FILE).unwrap();
    let file_diffs = diff.file_diffs();
    assert_eq!(3, file_diffs.len());
    let first_diff = file_diffs.first().unwrap();
    assert_eq!(
        first_diff.text(),
        "diff -Naur version-A/single.txt version-B/single.txt"
    );
}

#[test]
fn parse_old_file_name() {
    todo!();
}

#[test]
fn parse_new_file_name() {
    todo!();
}

#[test]
fn parse_time() {
    todo!();
}

#[test]
fn parse_hunk_header() {
    todo!();
}

#[test]
fn parse_all_hunks() {
    todo!();
}

#[test]
fn parse_line_type() {
    todo!();
}

#[test]
fn parse_line_location() {
    todo!();
}

#[test]
fn unparse_diff() {
    todo!();
}
