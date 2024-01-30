use mpatch::{diff::LineType, CommitDiff, FileDiff};

const DIFF_FILE: &str = "tests/diffs/base_patch.diff";

fn load_diffs() -> Vec<FileDiff> {
    let diff = CommitDiff::read(DIFF_FILE).unwrap();
    let file_diffs = diff.file_diffs();
    assert_eq!(3, file_diffs.len());
    diff.file_diffs().to_vec()
}

#[test]
fn parse_header() {
    let file_diffs = load_diffs();
    let diff = file_diffs.first().unwrap();
    assert_eq!(
        diff.diff_command().0,
        "diff -Naur version-A/single.txt version-B/single.txt"
    );
    let diff = file_diffs.get(1).unwrap();
    assert_eq!(
        diff.diff_command().0,
        "diff -Naur version-A/double_end.txt version-B/double_end.txt"
    );
    let diff = file_diffs.get(2).unwrap();
    assert_eq!(
        diff.diff_command().0,
        "diff -Naur version-A/long.txt version-B/long.txt"
    );
}

#[test]
fn parse_old_file_name() {
    let file_diffs = load_diffs();
    let diff = file_diffs.first().unwrap();
    assert_eq!(diff.source_file().path(), "version-A/single.txt");
    let diff = file_diffs.get(1).unwrap();
    assert_eq!(diff.source_file().path(), "version-A/double_end.txt");
    let diff = file_diffs.get(2).unwrap();
    assert_eq!(diff.source_file().path(), "version-A/long.txt");
}

#[test]
fn parse_new_file_name() {
    let file_diffs = load_diffs();
    let diff = file_diffs.first().unwrap();
    assert_eq!(diff.target_file().path(), "version-B/single.txt");
    let diff = file_diffs.get(1).unwrap();
    assert_eq!(diff.target_file().path(), "version-B/double_end.txt");
    let diff = file_diffs.get(2).unwrap();
    assert_eq!(diff.target_file().path(), "version-B/long.txt");
}

#[test]
fn parse_time() {
    let file_diffs = load_diffs();
    let diff = file_diffs.first().unwrap();
    assert_eq!(
        diff.source_file().timestamp(),
        "2023-11-03 16:26:28.701847364 +0100"
    );
    let diff = file_diffs.get(1).unwrap();
    assert_eq!(
        diff.source_file().timestamp(),
        "2023-11-03 16:39:35.953263076 +0100"
    );
    let diff = file_diffs.get(2).unwrap();
    assert_eq!(
        diff.source_file().timestamp(),
        "2023-11-03 16:26:28.701847364 +0100"
    );
}

#[test]
fn parse_hunk_header() {
    let file_diffs = load_diffs();
    let diff = file_diffs.first().unwrap();
    let hunk = diff.hunks().first().unwrap();
    assert_eq!(hunk.source_location().hunk_start(), 0);
    assert_eq!(hunk.source_location().hunk_length(), 0);
    assert_eq!(hunk.target_location().hunk_start(), 1);
    assert_eq!(hunk.target_location().hunk_length(), 1);

    let diff = file_diffs.get(1).unwrap();
    let hunk = diff.hunks().first().unwrap();
    assert_eq!(hunk.source_location().hunk_start(), 1);
    assert_eq!(hunk.source_location().hunk_length(), 4);
    assert_eq!(hunk.target_location().hunk_start(), 1);
    assert_eq!(hunk.target_location().hunk_length(), 3);

    let diff = file_diffs.get(2).unwrap();
    let hunk = diff.hunks().get(1).unwrap();
    assert_eq!(hunk.source_location().hunk_start(), 23);
    assert_eq!(hunk.source_location().hunk_length(), 7);
    assert_eq!(hunk.target_location().hunk_start(), 23);
    assert_eq!(hunk.target_location().hunk_length(), 7);
}

#[test]
fn parse_line_type() {
    let file_diffs = load_diffs();
    let diff = file_diffs.first().unwrap();
    let hunk = diff.hunks().first().unwrap();
    assert_eq!(hunk.lines().first().unwrap().line_type(), LineType::Add);
    assert_eq!(hunk.lines().get(1).unwrap().line_type(), LineType::EOF);

    let diff = file_diffs.get(1).unwrap();
    let hunk = diff.hunks().first().unwrap();
    assert_eq!(hunk.lines().get(3).unwrap().line_type(), LineType::Remove);
    assert_eq!(hunk.lines().get(4).unwrap().line_type(), LineType::EOF);
    assert_eq!(hunk.lines().get(5).unwrap().line_type(), LineType::Add);
    assert_eq!(hunk.lines().get(6).unwrap().line_type(), LineType::EOF);
}

#[test]
fn parse_line_location() {
    todo!();
}

#[test]
fn unparse_commit_diff() {
    let diff = CommitDiff::read(DIFF_FILE).unwrap();
    let diff_text = std::fs::read_to_string(DIFF_FILE).unwrap();
    assert_eq!(diff.to_string(), diff_text);
}

#[test]
fn unparse_file_diffs() {
    let file_diffs = load_diffs();

    let diff = file_diffs.first().unwrap();
    let text = r"
diff -Naur version-A/single.txt version-B/single.txt
--- version-A/single.txt	2023-11-03 16:26:28.701847364 +0100
+++ version-B/single.txt	2023-11-03 16:26:37.168563729 +0100
@@ -0,0 +1 @@
+ADDED
\ No newline at end of file
    "
    .trim()
    .to_string();
    assert_eq!(diff.to_string(), text);

    let diff = file_diffs.get(1).unwrap();
    let text = r"
diff -Naur version-A/double_end.txt version-B/double_end.txt
--- version-A/double_end.txt	2023-11-03 16:39:35.953263076 +0100
+++ version-B/double_end.txt	2023-11-03 16:40:12.500153951 +0100
@@ -1,4 +1,3 @@
 Line A
 Line B
-Line C
-Line D
\ No newline at end of file
+Line C
\ No newline at end of file
    "
    .trim()
    .to_string();
    assert_eq!(diff.to_string(), text);

    let diff = file_diffs.get(2).unwrap();
    let text = r"
diff -Naur version-A/long.txt version-B/long.txt
--- version-A/long.txt	2023-11-03 16:26:28.701847364 +0100
+++ version-B/long.txt	2023-11-03 16:26:37.168563729 +0100
@@ -1,7 +1,7 @@
 context 1
 context 2
 context 3
-REMOVED
+ADDED
 context 4
 context 5
 context 6
@@ -23,7 +23,7 @@
 context 1
 context 2
 context 3
-REMOVED
+ADDED
 context 4
 context 5
 context 6
    "
    .trim()
    .to_string();
    assert_eq!(diff.to_string(), text);
}
