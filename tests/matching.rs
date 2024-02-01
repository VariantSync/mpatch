use mpatch::{FileArtifact, LCSMatcher, Matcher};

const SOURCE_FILE_PATH: &str = "tests/samples/source_variant/version-0/main.c";
const TARGET_FILE_PATH: &str = "tests/samples/target_variant/version-0/main.c";

#[test]
fn file_matches_itself() {
    let file_instance_a = FileArtifact::read(SOURCE_FILE_PATH).unwrap();
    let file_instance_b = FileArtifact::read(SOURCE_FILE_PATH).unwrap();

    let mut matcher = LCSMatcher::default();
    let matching = matcher.match_files(&file_instance_a, &file_instance_b);
    for index in 1..file_instance_a.len() {
        assert_eq!(
            matching.left_index_for(index),
            matching.right_index_for(index)
        )
    }
}

#[test]
fn left_to_right_found() {
    let file_instance_a = FileArtifact::read(SOURCE_FILE_PATH).unwrap();
    let file_instance_b = FileArtifact::read(TARGET_FILE_PATH).unwrap();
    let left_to_right_expected = [
        (1, 1),
        (2, 3),
        (3, 4),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 8),
        (8, 9),
        (9, 10),
        (10, 16),
        (11, 17),
        (12, 18),
        (13, 19),
        (14, 20),
        (15, 21),
        (16, 22),
        (17, 23),
        (18, 24),
        (19, 25),
        (20, 26),
        (21, 27),
        (22, 28),
        (23, 29),
        (24, 30),
        (25, 31),
        (26, 32),
        (27, 33),
        (28, 34),
    ];

    let matching = LCSMatcher::match_files(&file_instance_a, &file_instance_b);
    for (left, right) in left_to_right_expected {
        assert_eq!(matching.right_index_for(left).unwrap(), Some(right));
    }
}

#[test]
fn right_to_left_found() {
    let file_instance_a = FileArtifact::read(SOURCE_FILE_PATH).unwrap();
    let file_instance_b = FileArtifact::read(TARGET_FILE_PATH).unwrap();
    let right_to_left_expected = [
        (1, Some(1)),
        (2, None),
        (3, Some(2)),
        (4, Some(3)),
        (5, Some(4)),
        (6, Some(5)),
        (7, Some(6)),
        (8, Some(7)),
        (9, Some(8)),
        (10, Some(9)),
        (11, None),
        (12, None),
        (13, None),
        (14, None),
        (15, None),
        (16, Some(10)),
        (17, Some(11)),
        (18, Some(12)),
        (19, Some(13)),
        (20, Some(14)),
        (21, Some(15)),
        (22, Some(16)),
        (23, Some(17)),
        (24, Some(18)),
        (25, Some(19)),
        (26, Some(20)),
        (27, Some(21)),
        (28, Some(22)),
    ];

    let matching = LCSMatcher::match_files(&file_instance_a, &file_instance_b);
    for (right, left) in right_to_left_expected {
        assert_eq!(matching.left_index_for(right).unwrap(), left);
    }
}
