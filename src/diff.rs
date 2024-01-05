use std::str::Lines;

use crate::{Error, ErrorKind};

pub struct Diff {
    file_diffs: Vec<FileDiff>,
}

impl Diff {
    pub fn read(path: &str) -> Result<Diff, Error> {
        let content = std::fs::read_to_string(path);
        let content = content.expect("was not able to load diff");
        Diff::try_from(content)
    }

    pub fn file_diffs(&self) -> &[FileDiff] {
        todo!();
    }

    pub fn len(&self) -> usize {
        self.file_diffs.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl TryFrom<String> for Diff {
    type Error = crate::Error;

    fn try_from(content: String) -> Result<Self, Self::Error> {
        // Colltect lines until the next FileDiff header
        let mut file_diff_content = vec![];
        let mut lines = content.lines();

        // TODO: Move to FileDiff parsing
        // TODO: Eeach diff consists of multiple file diff that start with 'diff ...', these should
        // be parsed until there is no more filediff
        if let Some(line) = lines.next() {
            if !line.starts_with("diff ") {
                return Err(Error::new(
                    "invalid format: does not start with 'diff '",
                    ErrorKind::DiffParseError,
                ));
            }
        }
        for line in content.lines() {
            // TODO: collect lines for a FileDiff
            file_diff_content.push(line)
        }
        // Parse the collected lines to a FileDiff
        // Repeat until all lines have been processed

        todo!()
    }
}

pub struct FileDiff {
    diff_command: DiffCommand,
    source_file: SourceFile,
    target_file: TargetFile,
    hunks: Vec<Hunk>,
}

impl FileDiff {
    pub fn text(&self) -> &str {
        todo!();
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

pub struct DiffCommand(pub String);

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
}

impl TryFrom<Vec<String>> for Hunk {
    type Error = Error;

    fn try_from(lines: Vec<String>) -> Result<Self, Self::Error> {
        let mut lines = lines.into_iter();
        let (source_location, target_location) =
            Hunk::parse_location_line(&lines.next().unwrap()).unwrap();
        let mut hunk_lines = vec![];
        for line in lines {
            hunk_lines.push(HunkLine::try_from(line)?);
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
    fn new(hunk_start: usize, hunk_length: usize) -> HunkLocation {
        HunkLocation {
            hunk_start,
            hunk_length,
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
        for number in value[1..].split(",") {
            match usize::from_str_radix(number, 10) {
                Ok(number) => numbers.push(number),
                Err(_) => return error(),
            }
        }

        Ok(HunkLocation {
            hunk_start: numbers[0],
            hunk_length: numbers[1],
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct HunkLine {
    content: String,
    line_type: LineType,
}

impl TryFrom<&str> for HunkLine {
    type Error = Error;

    fn try_from(content: &str) -> Result<Self, Self::Error> {
        HunkLine::try_from(content.to_string())
    }
}

impl TryFrom<String> for HunkLine {
    type Error = Error;

    fn try_from(content: String) -> Result<Self, Self::Error> {
        if content.as_str() == "\\ No newline at end of file" {
            return Ok(HunkLine {
                content,
                line_type: LineType::EOF,
            });
        }
        let line_type = if let Some(marker) = content.chars().nth(0) {
            match marker {
                '+' => LineType::Add,
                '-' => LineType::Remove,
                ' ' => LineType::Context,
                _ => {
                    return Err(Error::new(
                        &format!("invalid hunk line: {content}"),
                        ErrorKind::DiffParseError,
                    ))
                }
            }
        } else {
            return Err(Error::new(
                &format!("invalid hunk line: {content}"),
                ErrorKind::DiffParseError,
            ));
        };
        Ok(HunkLine { content, line_type })
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum LineType {
    Context,
    Add,
    Remove,
    EOF,
}

pub struct SourceFile {
    path: String,
    // TODO: Use actual time value
    timestamp: String,
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

pub struct TargetFile {
    path: String,
    // TODO: Use actual time value
    timestamp: String,
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
    // add code here
}

impl TryFrom<&str> for TargetFile {
    type Error = Error;

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        Self::try_from(line.to_string())
    }
    // add code here
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
        diff::{DiffCommand, LineType, TargetFile},
        FileDiff, Hunk,
    };

    use super::{HunkLine, SourceFile};

    fn check_line_parsing(line: &str, expected_type: LineType) {
        let hunk_line = HunkLine::try_from(line).unwrap();
        assert_eq!(hunk_line.content, line);
        assert_eq!(hunk_line.line_type, expected_type);
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
        assert!(HunkLine::try_from(line).is_err());
    }

    #[test]
    fn recognize_invalid_line_eof() {
        let line = "\\Not a valid line";
        assert!(HunkLine::try_from(line).is_err());
    }

    #[test]
    fn recognize_invalid_empty_line() {
        let line = "";
        assert!(HunkLine::try_from(line).is_err());
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

        for (id, line) in input.into_iter().skip(1).enumerate() {
            let line = HunkLine::try_from(line).unwrap();
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

        for (id, line) in input.into_iter().skip(1).enumerate() {
            let line = HunkLine::try_from(line).unwrap();
            assert_eq!(hunk.lines.get(id), Some(&line));
        }
    }

    // TODO: Test FileDiff parsing
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
}
