use std::fmt::Display;

use crate::{Error, ErrorKind};

#[derive(Debug, Clone)]
pub struct CommitDiff {
    file_diffs: Vec<FileDiff>,
}

impl CommitDiff {
    pub fn read(path: &str) -> Result<CommitDiff, Error> {
        let content = std::fs::read_to_string(path);
        let content = content.expect("was not able to load diff");
        CommitDiff::try_from(content)
    }

    pub fn file_diffs(&self) -> &[FileDiff] {
        self.file_diffs.as_slice()
    }

    pub fn len(&self) -> usize {
        self.file_diffs.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Display for CommitDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut multiple = false;
        for file_diff in &self.file_diffs {
            if multiple {
                writeln!(f)?;
            }
            write!(f, "{file_diff}")?;
            multiple = true;
        }
        Ok(())
    }
}

impl TryFrom<String> for CommitDiff {
    type Error = crate::Error;

    fn try_from(content: String) -> Result<Self, Self::Error> {
        let mut file_diff_content = vec![];
        let mut file_diffs = vec![];

        for line in content.lines() {
            // Colltect lines until the next FileDiff header
            if line.starts_with("diff ") {
                if !file_diff_content.is_empty() {
                    file_diffs.push(FileDiff::try_from(file_diff_content)?);
                }
                file_diff_content = vec![];
            }
            file_diff_content.push(line.to_string());
        }

        // push the last FileDiff
        if !file_diff_content.is_empty() {
            file_diffs.push(FileDiff::try_from(file_diff_content)?);
        }
        if file_diffs.is_empty() {
            Err(Error::new(
                "the given diff is empty: {content}",
                ErrorKind::DiffParseError,
            ))
        } else {
            Ok(Self { file_diffs })
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    diff_command: DiffCommand,
    source_file: SourceFile,
    target_file: TargetFile,
    hunks: Vec<Hunk>,
}

impl Display for FileDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.diff_command)?;
        write!(
            f,
            "\n--- {}\t{}",
            self.source_file.path, self.source_file.timestamp
        )?;
        write!(
            f,
            "\n+++ {}\t{}",
            self.target_file.path, self.target_file.timestamp
        )?;
        for hunk in &self.hunks {
            // no writeln because Hunks have newline characters themselves
            write!(f, "\n{hunk}")?;
        }
        Ok(())
    }
}

impl FileDiff {
    pub fn diff_command(&self) -> &DiffCommand {
        &self.diff_command
    }

    pub fn source_file(&self) -> &SourceFile {
        &self.source_file
    }

    pub fn target_file(&self) -> &TargetFile {
        &self.target_file
    }

    pub fn hunks(&self) -> &[Hunk] {
        self.hunks.as_ref()
    }

    pub fn changes(&self) -> Changes {
        let changes: Vec<&HunkLine> = self
            .hunks()
            .iter()
            .flat_map(|h| h.lines.iter())
            .filter(|l| l.line_type == LineType::Add || l.line_type == LineType::Remove)
            // reverse the order so that changes can be easily popped from the vec
            .rev()
            .collect();
        Changes { changes }
    }

    pub fn into_changes(self) -> IntoChanges {
        let changes: Vec<HunkLine> = self
            .hunks
            .into_iter()
            .flat_map(|h| h.lines)
            .filter(|l| l.line_type == LineType::Add || l.line_type == LineType::Remove)
            // reverse the order so that changes can be easily popped from the vec
            .rev()
            .collect();
        IntoChanges { changes }
    }
}

pub struct Changes<'a> {
    // changes in reverse order
    // the order is reversed to allow pop operations
    changes: Vec<&'a HunkLine>,
}

impl<'a> Iterator for Changes<'a> {
    type Item = &'a HunkLine;

    fn next(&mut self) -> Option<Self::Item> {
        self.changes.pop()
    }
}

pub struct IntoChanges {
    // changes in reverse order
    // the order is reversed to allow pop operations
    changes: Vec<HunkLine>,
}

impl Iterator for IntoChanges {
    type Item = HunkLine;

    fn next(&mut self) -> Option<Self::Item> {
        self.changes.pop()
    }
}

impl TryFrom<Vec<String>> for FileDiff {
    type Error = Error;

    fn try_from(lines: Vec<String>) -> Result<Self, Self::Error> {
        let mut lines = lines.into_iter();
        let diff_command = lines.next().unwrap();
        if !diff_command.starts_with("diff ") {
            return Err(Error::new(
                &format!("invalid file diff start: {diff_command}"),
                ErrorKind::DiffParseError,
            ));
        }
        let source_file = SourceFile::try_from(lines.next().unwrap())?;
        let target_file = TargetFile::try_from(lines.next().unwrap())?;
        let mut hunk_lines = vec![];
        let mut hunks = vec![];
        for line in lines {
            if line.starts_with("@@ ") {
                if !hunk_lines.is_empty() {
                    hunks.push(Hunk::try_from(hunk_lines)?);
                }
                hunk_lines = vec![];
            }
            hunk_lines.push(line);
        }
        // push the last hunk
        if !hunk_lines.is_empty() {
            hunks.push(Hunk::try_from(hunk_lines)?);
        }
        let diff_command = DiffCommand(diff_command);
        Ok(FileDiff {
            diff_command,
            source_file,
            target_file,
            hunks,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DiffCommand(pub String);

impl Display for DiffCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Hunk {
    source_location: HunkLocation,
    target_location: HunkLocation,
    lines: Vec<HunkLine>,
}

impl Hunk {
    fn parse_location_line(line: &str) -> Result<(HunkLocation, HunkLocation), Error> {
        if !line.starts_with("@@ ") || !line.ends_with(" @@") {
            return Err(Error::new(
                &format!("invalid hunk location: {line}"),
                ErrorKind::DiffParseError,
            ));
        }
        let mut hunk_locations: [Option<HunkLocation>; 2] = [None, None];

        for (id, location) in line
            .split_whitespace()
            // Skip the leading "@@ "
            .skip(1)
            // Ignore the trailing " @@"
            .take(2)
            .enumerate()
        {
            hunk_locations[id] = Some(HunkLocation::try_from(location)?);
        }

        Ok((hunk_locations[0].unwrap(), hunk_locations[1].unwrap()))
    }

    pub fn source_location(&self) -> HunkLocation {
        self.source_location
    }

    pub fn target_location(&self) -> HunkLocation {
        self.target_location
    }

    pub fn lines(&self) -> &[HunkLine] {
        self.lines.as_ref()
    }
}

impl Display for Hunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "@@ -{} +{} @@",
            self.source_location, self.target_location
        )?;
        for line in &self.lines {
            write!(f, "\n{line}")?;
        }
        Ok(())
    }
}

impl TryFrom<Vec<String>> for Hunk {
    type Error = Error;

    fn try_from(lines: Vec<String>) -> Result<Self, Self::Error> {
        let mut lines = lines.into_iter();
        let (source_location, target_location) =
            Hunk::parse_location_line(&lines.next().unwrap()).unwrap();
        let mut hunk_lines = vec![];

        let mut source_id = source_location.hunk_start;
        let mut target_id = target_location.hunk_start;
        for line in lines {
            let line_type = LineType::determine_type(&line)?;
            let mut source_line = None;
            let mut target_line = None;
            match line_type {
                LineType::Context => {
                    source_line = Some(source_id);
                    source_id += 1;
                    target_line = Some(target_id);
                    target_id += 1;
                }
                LineType::Add => {
                    target_line = Some(target_id);
                    target_id += 1;
                }

                LineType::Remove => {
                    source_line = Some(source_id);
                    source_id += 1;
                }
                LineType::EOF => (),
            }
            // Set the location of the line
            hunk_lines.push(HunkLine::new(source_line, target_line, line_type, line)?);
        }
        Ok(Hunk {
            source_location,
            target_location,
            lines: hunk_lines,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct HunkLocation {
    hunk_start: usize,
    hunk_length: usize,
}

impl HunkLocation {
    pub fn hunk_start(&self) -> usize {
        self.hunk_start
    }

    pub fn hunk_length(&self) -> usize {
        self.hunk_length
    }
}

impl Display for HunkLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.hunk_start == 1 && self.hunk_length == 1 {
            // handle this weird edge case in unix diff
            // if the location and size both are '1', the location text is abbreviated to just '1'
            write!(f, "1")
        } else {
            write!(f, "{},{}", self.hunk_start, self.hunk_length,)
        }
    }
}

impl TryFrom<&str> for HunkLocation {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let error = || {
            Err(Error::new(
                &format!("invalid hunk location: {value}"),
                ErrorKind::DiffParseError,
            ))
        };
        if value.is_empty() {
            return error();
        }
        if value.chars().nth(0).unwrap() != '-' && value.chars().nth(0).unwrap() != '+' {
            return error();
        }

        let mut numbers = vec![];
        for number in value[1..].split(',') {
            match number.parse::<usize>() {
                Ok(number) => numbers.push(number),
                Err(_) => return error(),
            }
        }

        // TODO: verify that handling the location specifiers like this is correct
        if numbers.len() == 1 {
            // Sometimes, the location is only given by the location, but not with a length (i.e.,
            // if there is only one line.
            numbers.push(numbers[0]);
        }

        Ok(HunkLocation {
            hunk_start: numbers[0],
            hunk_length: numbers[1],
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HunkLine {
    line: String,
    source_line: Option<usize>,
    target_line: Option<usize>,
    line_type: LineType,
}

impl HunkLine {
    pub fn content(&self) -> &str {
        self.line.as_ref()
    }

    pub fn line_type(&self) -> LineType {
        self.line_type
    }

    pub fn new(
        source_line: Option<usize>,
        target_line: Option<usize>,
        line_type: LineType,
        line: String,
    ) -> Result<HunkLine, Error> {
        Ok(HunkLine {
            line,
            source_line,
            target_line,
            line_type,
        })
    }

    pub fn source_line(&self) -> Option<usize> {
        self.source_line
    }

    pub fn target_line(&self) -> Option<usize> {
        self.target_line
    }
}

impl Display for HunkLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.line)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum LineType {
    Context,
    Add,
    Remove,
    EOF,
}

impl LineType {
    fn determine_type(line: &str) -> Result<LineType, Error> {
        if line == "\\ No newline at end of file" {
            return Ok(LineType::EOF);
        }
        if let Some(marker) = line.chars().nth(0) {
            match marker {
                '+' => Ok(LineType::Add),
                '-' => Ok(LineType::Remove),
                ' ' => Ok(LineType::Context),
                _ => Err(Error::new(
                    &format!("invalid hunk line: {line}"),
                    ErrorKind::DiffParseError,
                )),
            }
        } else {
            Err(Error::new(
                &format!("invalid hunk line: {line}"),
                ErrorKind::DiffParseError,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    path: String,
    // TODO: Use actual time value
    timestamp: String,
}

impl SourceFile {
    pub fn path(&self) -> &str {
        self.path.as_ref()
    }

    pub fn timestamp(&self) -> &str {
        self.timestamp.as_ref()
    }
}

impl TryFrom<String> for SourceFile {
    type Error = Error;

    fn try_from(line: String) -> Result<Self, Self::Error> {
        if !line.starts_with("--- ") {
            return Err(Error::new(
                "invalid format: does not start with '--- '",
                ErrorKind::DiffParseError,
            ));
        }
        let (path, timestamp) = parse_file_line(line)?;
        Ok(Self { path, timestamp })
    }
}

impl TryFrom<&str> for SourceFile {
    type Error = Error;

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        Self::try_from(line.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct TargetFile {
    path: String,
    // TODO: Use actual time value
    timestamp: String,
}

impl TargetFile {
    pub fn path(&self) -> &str {
        self.path.as_ref()
    }

    pub fn timestamp(&self) -> &str {
        self.timestamp.as_ref()
    }
}

impl TryFrom<String> for TargetFile {
    type Error = Error;

    fn try_from(line: String) -> Result<Self, Self::Error> {
        if !line.starts_with("+++ ") {
            return Err(Error::new(
                "invalid format: does not start with '--- '",
                ErrorKind::DiffParseError,
            ));
        }
        let (path, timestamp) = parse_file_line(line)?;
        Ok(Self { path, timestamp })
    }
}

impl TryFrom<&str> for TargetFile {
    type Error = Error;

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        Self::try_from(line.to_string())
    }
}

fn parse_file_line(input: String) -> Result<(String, String), Error> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() != 5 {
        return Err(Error::new(
            "invalid format: incorrect number of elements",
            ErrorKind::DiffParseError,
        ));
    }

    let path = parts[1].to_string();
    let timestamp = format!("{} {} {}", parts[2], parts[3], parts[4]);

    Ok((path, timestamp))
}

#[cfg(test)]
mod tests {
    use crate::{
        diffs::{LineType, TargetFile},
        FileDiff, Hunk,
    };

    use super::{HunkLine, SourceFile};

    fn check_line_parsing(line: &str, expected_type: LineType) {
        let line_type = LineType::determine_type(line).unwrap();
        assert_eq!(line_type, expected_type);
    }

    #[test]
    fn parse_context_line() {
        let line = " unchanged code";
        check_line_parsing(line, LineType::Context);
    }

    #[test]
    fn parse_add_line() {
        let line = "+added code";
        check_line_parsing(line, LineType::Add);
    }

    #[test]
    fn parse_remove_line() {
        let line = "-removed code";
        check_line_parsing(line, LineType::Remove);
    }

    #[test]
    fn parse_eof_line() {
        let line = "\\ No newline at end of file";
        check_line_parsing(line, LineType::EOF);
    }

    #[test]
    fn recognize_invalid_line() {
        let line = "Not a valid format";
        assert!(LineType::determine_type(line).is_err());
    }

    #[test]
    fn recognize_invalid_line_eof() {
        let line = "\\Not a valid line";
        assert!(LineType::determine_type(line).is_err());
    }

    #[test]
    fn recognize_invalid_empty_line() {
        let line = "";
        assert!(LineType::determine_type(line).is_err());
    }

    #[test]
    fn parse_valid_location_line() {
        let location_line = "@@ -1,7 +1,7 @@";
        let (source_location, target_location) = Hunk::parse_location_line(location_line).unwrap();
        assert_eq!(source_location.hunk_start, 1);
        assert_eq!(source_location.hunk_length, 7);
        assert_eq!(target_location.hunk_start, 1);
        assert_eq!(source_location.hunk_length, 7);
    }

    #[test]
    fn recognize_invalid_location_line_start() {
        let location_line = "@ -1,7 +1,7 @@";
        assert!(Hunk::parse_location_line(location_line).is_err());
    }

    #[test]
    fn recognize_invalid_location_line_end() {
        let location_line = "@@ -1,7 +1,7 @";
        assert!(Hunk::parse_location_line(location_line).is_err());
    }

    #[test]
    fn recognize_invalid_location_line_number() {
        let location_line = "@@ -1,7 1,7 @@";
        assert!(Hunk::parse_location_line(location_line).is_err());
    }

    #[test]
    fn recognize_invalid_location_line_comma() {
        let location_line = "@@ -1,7 +1;7 @@";
        assert!(Hunk::parse_location_line(location_line).is_err());
    }

    #[test]
    fn parse_valid_source_file() {
        let line = "--- version-A/double_end.txt	2023-11-03 16:39:35.953263076 +0100";
        let source = SourceFile::try_from(line).unwrap();
        assert_eq!("version-A/double_end.txt", source.path);
        assert_eq!("2023-11-03 16:39:35.953263076 +0100", source.timestamp);
    }

    #[test]
    fn parse_valid_target_file() {
        let line = "+++ version-B/double_end.txt	2023-11-03 16:40:12.500153951 +0100";
        let source = TargetFile::try_from(line).unwrap();
        assert_eq!("version-B/double_end.txt", source.path);
        assert_eq!("2023-11-03 16:40:12.500153951 +0100", source.timestamp);
    }

    #[test]
    fn recognize_invalid_source_file() {
        let line = "+++ version-A/double_end.txt	2023-11-03 16:39:35.953263076 +0100";
        assert!(SourceFile::try_from(line).is_err());
    }

    #[test]
    fn recognize_invalid_target_file() {
        let line = "--- version-A/double_end.txt	2023-11-03 16:39:35.953263076 +0100";
        assert!(TargetFile::try_from(line).is_err());
    }

    #[test]
    fn parse_valid_hunk() {
        let input = "@@ -1,7 +2,5 @@
                     context 1
                     context 2
                     context 3
                    -REMOVED
                    +ADDED
                     context 4
                     context 5
                     context 6
                    ";
        let input = prepare_diff_vec(input);
        let hunk = Hunk::try_from(input.clone()).unwrap();
        assert_eq!(hunk.source_location.hunk_start, 1);
        assert_eq!(hunk.source_location.hunk_length, 7);
        assert_eq!(hunk.target_location.hunk_start, 2);
        assert_eq!(hunk.target_location.hunk_length, 5);

        let expected_lines = [
            HunkLine::new(
                Some(1),
                Some(2),
                LineType::Context,
                " context 1".to_string(),
            )
            .unwrap(),
            HunkLine::new(
                Some(2),
                Some(3),
                LineType::Context,
                " context 2".to_string(),
            )
            .unwrap(),
            HunkLine::new(
                Some(3),
                Some(4),
                LineType::Context,
                " context 3".to_string(),
            )
            .unwrap(),
            HunkLine::new(Some(4), None, LineType::Remove, "-REMOVED".to_string()).unwrap(),
            HunkLine::new(None, Some(5), LineType::Add, "+ADDED".to_string()).unwrap(),
            HunkLine::new(
                Some(5),
                Some(6),
                LineType::Context,
                " context 4".to_string(),
            )
            .unwrap(),
            HunkLine::new(
                Some(6),
                Some(7),
                LineType::Context,
                " context 5".to_string(),
            )
            .unwrap(),
            HunkLine::new(
                Some(7),
                Some(8),
                LineType::Context,
                " context 6".to_string(),
            )
            .unwrap(),
        ];

        for (id, line) in expected_lines.into_iter().enumerate() {
            assert_eq!(hunk.lines.get(id), Some(&line));
        }
    }

    #[test]
    fn parse_valid_hunk_with_eofs() {
        let input = "@@ -1,4 +1,3 @@
                     Line A
                     Line B
                    -Line C
                    -Line D
                    \\ No newline at end of file
                    +Line C
                    \\ No newline at end of file
                    ";
        let input = prepare_diff_vec(input);
        let hunk = Hunk::try_from(input.clone()).unwrap();
        assert_eq!(hunk.source_location.hunk_start, 1);
        assert_eq!(hunk.source_location.hunk_length, 4);
        assert_eq!(hunk.target_location.hunk_start, 1);
        assert_eq!(hunk.target_location.hunk_length, 3);

        let expected_types = [
            LineType::Context,
            LineType::Context,
            LineType::Remove,
            LineType::Remove,
            LineType::EOF,
            LineType::Add,
            LineType::EOF,
        ];

        for (id, line_type) in expected_types.into_iter().enumerate() {
            assert_eq!(hunk.lines.get(id).unwrap().line_type(), line_type);
        }
    }

    #[test]
    fn parse_file_diff_with_multiple_hunks() {
        let content = "diff -Naur version-A/long.txt version-B/long.txt
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
                       ";
        let mut content = prepare_diff_vec(content);
        content[0] = content[0].trim().to_string();
        let file_diff = FileDiff::try_from(content.clone()).unwrap();
        assert_eq!(file_diff.diff_command.0, content[0]);
        assert_eq!(file_diff.source_file.path, "version-A/long.txt".to_string());
        assert_eq!(file_diff.target_file.path, "version-B/long.txt".to_string());
        assert_eq!(
            file_diff.source_file.timestamp,
            "2023-11-03 16:26:28.701847364 +0100".to_string()
        );
        assert_eq!(
            file_diff.target_file.timestamp,
            "2023-11-03 16:26:37.168563729 +0100".to_string()
        );
        assert_eq!(file_diff.hunks.len(), 2);
    }

    #[inline(always)]
    fn prepare_diff_vec(input: &str) -> Vec<String> {
        input
            .lines()
            .map(|s| s.trim())
            .map(|s| {
                // Add back the space for context lines
                if s.starts_with(|c| c != '-' && c != '+' && c != '\\' && c != '@') {
                    format!(" {s}")
                } else {
                    s.to_string()
                }
            })
            .filter(|s| !s.is_empty())
            .collect()
    }

    #[test]
    fn identify_line_locations() {
        let input = "@@ -4,7 +10,5 @@
                     context 1
                     context 2
                     context 3
                    -REMOVED
                    +ADDED
                     context 4
                     context 5
                     context 6
                    ";
        let input = prepare_diff_vec(input);
        let hunk = Hunk::try_from(input.clone()).unwrap();

        let offset_old = 3;
        let offset_new = 9;

        let expected_lines = [
            (Some(1), Some(1)),
            (Some(2), Some(2)),
            (Some(3), Some(3)),
            (Some(4), None),
            (None, Some(4)),
            (Some(5), Some(5)),
            (Some(6), Some(6)),
            (Some(7), Some(7)),
        ];

        for (i, line) in hunk.lines.iter().enumerate() {
            let (old_id, new_id) = expected_lines[i];
            println!("{line:?}");
            assert_eq!(line.source_line, old_id.map(|v| v + offset_old));
            assert_eq!(line.target_line, new_id.map(|v| v + offset_new));
        }
    }
}
