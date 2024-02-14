use std::fs;

use mpatch::{
    diffs::{ChangedLines, LineLocation, LineType},
    CommitDiff, FileDiff,
};

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
    assert_eq!(diff.source_file().path_str(), "version-A/single.txt");
    let diff = file_diffs.get(1).unwrap();
    assert_eq!(diff.source_file().path_str(), "version-A/double_end.txt");
    let diff = file_diffs.get(2).unwrap();
    assert_eq!(diff.source_file().path_str(), "version-A/long.txt");
}

#[test]
fn parse_new_file_name() {
    let file_diffs = load_diffs();
    let diff = file_diffs.first().unwrap();
    assert_eq!(diff.target_file().path_str(), "version-B/single.txt");
    let diff = file_diffs.get(1).unwrap();
    assert_eq!(diff.target_file().path_str(), "version-B/double_end.txt");
    let diff = file_diffs.get(2).unwrap();
    assert_eq!(diff.target_file().path_str(), "version-B/long.txt");
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
fn unparse_commit_diff() {
    let diff = CommitDiff::read(DIFF_FILE).unwrap();
    let diff_text = fs::read_to_string(DIFF_FILE).unwrap();

    assert_eq!(diff.to_string(), diff_text.trim_end());
}

#[test]
fn unparse_file_diffs() {
    let file_diffs = load_diffs();

    let diff = file_diffs.first().unwrap();
    let text = r"diff -Naur version-A/single.txt version-B/single.txt
--- version-A/single.txt	2023-11-03 16:26:28.701847364 +0100
+++ version-B/single.txt	2023-11-03 16:26:37.168563729 +0100
@@ -0,0 +1 @@
+ADDED
\ No newline at end of file"
        .trim()
        .to_string();

    assert_eq!(diff.hunks().first().unwrap().lines().len(), 2);
    assert_eq!(diff.to_string(), text);

    let diff = file_diffs.get(1).unwrap();
    assert_eq!(diff.hunks().first().unwrap().lines().len(), 7);
    let text = r"diff -Naur version-A/double_end.txt version-B/double_end.txt
--- version-A/double_end.txt	2023-11-03 16:39:35.953263076 +0100
+++ version-B/double_end.txt	2023-11-03 16:40:12.500153951 +0100
@@ -1,4 +1,3 @@
 Line A
 Line B
-Line C
-Line D
\ No newline at end of file
+Line C
\ No newline at end of file"
        .trim()
        .to_string();
    assert_eq!(diff.to_string(), text);

    let diff = file_diffs.get(2).unwrap();
    assert_eq!(diff.hunks().get(1).unwrap().lines().len(), 8);
    let text = r"diff -Naur version-A/long.txt version-B/long.txt
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
 context 6"
        .trim()
        .to_string();
    assert_eq!(diff.to_string(), text);
}

#[test]
fn retrieve_changes_per_file() {
    let diff = CommitDiff::read(DIFF_FILE).unwrap();
    let file_diff = &diff.file_diffs()[0];
    let changes = file_diff.changes();
    assert_eq!((1, 0), count_changes(changes));

    let file_diff = &diff.file_diffs()[1];
    let changes = file_diff.changes();
    assert_eq!((1, 2), count_changes(changes));

    let file_diff = &diff.file_diffs()[2];
    let changes = file_diff.changes();
    assert_eq!((2, 2), count_changes(changes));
}

fn count_changes(changes: ChangedLines) -> (usize, usize) {
    let mut add_count = 0;
    let mut remove_count = 0;
    for change in changes {
        match change.line_type() {
            LineType::Add => add_count += 1,
            LineType::Remove => remove_count += 1,
            _ => panic!("Not a change!"),
        }
    }
    (add_count, remove_count)
}

use mpatch::diffs::LineLocation::{ChangeLocation, RealLocation};

#[test]
fn locate_changes_per_file() {
    let diff = CommitDiff::read(DIFF_FILE).unwrap();

    let file_diff = &diff.file_diffs()[0];
    let changes = file_diff.changes();
    let mut locations = change_locations(changes);
    assert_eq!(
        (ChangeLocation(1), RealLocation(1)),
        locations.pop().unwrap()
    );

    let file_diff = &diff.file_diffs()[1];
    let changes = file_diff.changes();
    let mut locations = change_locations(changes);
    locations.reverse();
    assert_eq!(
        (RealLocation(3), ChangeLocation(3)),
        locations.pop().unwrap()
    );
    assert_eq!(
        (RealLocation(4), ChangeLocation(3)),
        locations.pop().unwrap()
    );
    assert_eq!(
        (ChangeLocation(3), RealLocation(3)),
        locations.pop().unwrap()
    );

    let file_diff = &diff.file_diffs()[2];
    let changes = file_diff.changes();
    let mut locations = change_locations(changes);
    locations.reverse();
    assert_eq!(
        (RealLocation(4), ChangeLocation(4)),
        locations.pop().unwrap()
    );
    assert_eq!(
        (ChangeLocation(4), RealLocation(4)),
        locations.pop().unwrap()
    );
    assert_eq!(
        (RealLocation(26), ChangeLocation(26)),
        locations.pop().unwrap()
    );
    assert_eq!(
        (ChangeLocation(26), RealLocation(26)),
        locations.pop().unwrap()
    );
}

fn change_locations(changes: ChangedLines) -> Vec<(LineLocation, LineLocation)> {
    let mut locations = vec![];
    for change in changes {
        locations.push((change.source_line(), change.target_line()));
    }
    locations
}
