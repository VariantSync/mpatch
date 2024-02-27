use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
    vec::IntoIter,
};

use crate::{Error, ErrorKind};

/// A VersionDiff represents a diff between two versions of a project or parts of a projects.
/// A VersionDiff comprises one or more FileDiffs which in turn represent diffs for individual
/// files.
#[derive(Debug, Clone)]
pub struct VersionDiff {
    file_diffs: Vec<FileDiff>,
}

impl VersionDiff {
    /// Reads a diff file and tries to parse it into a VersionDiff.
    ///
    /// # Error
    /// This function returns an error if the file cannot be read or if the file's content cannot
    /// be parsed into a VersionDiff.
    pub fn read<P: AsRef<Path>>(path: P) -> Result<VersionDiff, Error> {
        let content = std::fs::read_to_string(path)?;
        VersionDiff::try_from(content)
    }

    /// Returns a reference to the slice of FileDiffs in this VersionDiff.
    pub fn file_diffs(&self) -> &[FileDiff] {
        self.file_diffs.as_slice()
    }

    /// Returns the number of FileDiffs in this VersionDiff.
    pub fn len(&self) -> usize {
        self.file_diffs.len()
    }

    /// Returns true if this VersionDiff contains no FileDiffs, otherwise returns false.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl IntoIterator for VersionDiff {
    type Item = FileDiff;

    type IntoIter = IntoIter<FileDiff>;

    fn into_iter(self) -> Self::IntoIter {
        self.file_diffs.into_iter()
    }
}

impl Display for VersionDiff {
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

impl TryFrom<String> for VersionDiff {
    type Error = crate::Error;

    fn try_from(content: String) -> Result<Self, Self::Error> {
        let mut file_diffs = vec![];

        let mut file_diff_content = vec![];
        for line in content.lines() {
            // Collect lines until the next FileDiff header
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

/// A FileDiff represents a diff between two versions of a file.
/// Each FileDiff contains a DiffCommand (i.e., its header line), a source and a target file, and
/// one or more hunks.
/// Hunks contain grouped changes to lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    diff_command: DiffCommand,
    source_file_header: SourceFileHeader,
    target_file_header: TargetFileHeader,
    hunks: Vec<Hunk>,
}

impl Display for FileDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.diff_command)?;
        write!(
            f,
            "\n--- {}\t{}",
            self.source_file_header.path, self.source_file_header.timestamp
        )?;
        write!(
            f,
            "\n+++ {}\t{}",
            self.target_file_header.path, self.target_file_header.timestamp
        )?;
        for hunk in &self.hunks {
            // no writeln because Hunks have newline characters themselves
            write!(f, "\n{hunk}")?;
        }
        Ok(())
    }
}

impl FileDiff {
    /// Returns the header of this FileDiff (i.e., the DiffCommand used to generate it).
    pub fn diff_command(&self) -> &DiffCommand {
        &self.diff_command
    }

    /// Returns the source file header of the diff operation (i.e., the information about
    /// the file assumed to be the older version).
    pub fn source_file_header(&self) -> &SourceFileHeader {
        &self.source_file_header
    }

    /// Returns the target file header of the diff operation (i.e., the information file
    /// assumed to be the newer version).
    pub fn target_file_header(&self) -> &TargetFileHeader {
        &self.target_file_header
    }

    /// Returns a reference to the hunks contained in the FileDiff.
    pub fn hunks(&self) -> &[Hunk] {
        &self.hunks
    }

    /// Collects all changes in this FileDiff and returns an iterator over their references.
    ///
    /// # Returns
    /// Returns a ChangedLines iterator that iterates all HunkLine instances containing changes.
    ///
    pub fn changes(&self) -> ChangedLines {
        let changes: Vec<&HunkLine> = self
            .hunks()
            .iter()
            .flat_map(|h| h.lines.iter())
            .filter(|l| l.line_type == LineType::Add || l.line_type == LineType::Remove)
            // reverse the order so that changes can be easily popped from the vec
            .rev()
            .collect();
        ChangedLines { changes }
    }

    /// Collects and takes owenership of all changes in this FileDiff and returns and iterator over
    /// them. This method consumes the FileDiff.
    ///
    /// # Returns
    /// Returns an IntoChangedLines iterator that iterates all HunkLine instances containing changes.
    ///
    pub fn into_changes(self) -> IntoChangedLines {
        let changes: Vec<HunkLine> = self
            .hunks
            .into_iter()
            .flat_map(|h| h.lines)
            .filter(|l| l.line_type == LineType::Add || l.line_type == LineType::Remove)
            // reverse the order so that changes can be easily popped from the vec
            .rev()
            .collect();
        IntoChangedLines { changes }
    }

    /// Generates and returns the full header of this FileDiff containing the DiffCommand, the
    /// information about the source file, and the information about the target file.
    pub fn header(&self) -> String {
        format!(
            "{}\n--- {}\t{}\n+++ {}\t{}",
            self.diff_command,
            self.source_file_header.path,
            self.source_file_header.timestamp,
            self.target_file_header.path,
            self.target_file_header.timestamp
        )
    }
}

/// Iterator over references of HunkLines constituting line changes.
pub struct ChangedLines<'a> {
    // In all current intatiations of ChangedLines, the changes are provided in reverse order to
    // allow for pop operations while maintaining the original order of the changes.
    changes: Vec<&'a HunkLine>,
}

impl<'a> Iterator for ChangedLines<'a> {
    type Item = &'a HunkLine;

    fn next(&mut self) -> Option<Self::Item> {
        self.changes.pop()
    }
}

/// Iterator over owned instances of HunkLines constituting line changes.
pub struct IntoChangedLines {
    // In all current intatiations of IntoChangedLines, the changes are provided in reverse order to
    // allow for pop operations while maintaining the original order of the changes.
    changes: Vec<HunkLine>,
}

impl Iterator for IntoChangedLines {
    type Item = HunkLine;

    fn next(&mut self) -> Option<Self::Item> {
        self.changes.pop()
    }
}

impl TryFrom<Vec<String>> for FileDiff {
    type Error = Error;

    fn try_from(lines: Vec<String>) -> Result<Self, Self::Error> {
        let mut lines = lines.into_iter();

        // Parse the diff command
        let diff_command = lines.next().unwrap();
        if !diff_command.starts_with("diff ") {
            return Err(Error::new(
                &format!("invalid file diff start: {diff_command}"),
                ErrorKind::DiffParseError,
            ));
        }
        let diff_command = DiffCommand(diff_command);

        // Parse the source and target file headers
        let source_file = SourceFileHeader::try_from(lines.next().unwrap())?;
        let target_file = TargetFileHeader::try_from(lines.next().unwrap())?;

        // Parse the hunks
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

        Ok(FileDiff {
            diff_command,
            source_file_header: source_file,
            target_file_header: target_file,
            hunks,
        })
    }
}

/// A DiffCommand holds the exact call to diff used to create a FileDiff (e.g., "diff -Naur ...").
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DiffCommand(pub String);

impl Display for DiffCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A Hunk consists of a source location, a target location, and one or more HunkLines.
/// The locations describe the start and length of the changed text by line number.
/// The source location specifies the location before the changes (i.e., the state in the source
/// file).
/// The target location specifies the location after the changes (i.e., the state in the target
/// file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    source_location: HunkLocation,
    target_location: HunkLocation,
    lines: Vec<HunkLine>,
}

impl Hunk {
    /// Parses the location line of the hunk into two HunkLocation instances, one for the source
    /// and one for the target.
    /// A location type has the form "@@ -SOURCE_START,SOURCE_LENGTH +TARGET_START,TARGET_LENGTH @@"
    ///
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

    /// Returns the source location of this Hunk.
    pub fn source_location(&self) -> HunkLocation {
        self.source_location
    }

    /// Returns the target location of this Hunk.
    pub fn target_location(&self) -> HunkLocation {
        self.target_location
    }

    /// Returns a reference to the HunkLines of this Hunk.
    pub fn lines(&self) -> &[HunkLine] {
        &self.lines
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

        // Parse the source and target location
        let (source_location, target_location) =
            Hunk::parse_location_line(&lines.next().unwrap()).unwrap();

        // Parse the hunk lines
        let mut hunk_lines = vec![];
        let mut source_id = source_location.hunk_start;
        let mut target_id = target_location.hunk_start;
        for line in lines {
            let line_type = LineType::determine_type(&line)?;
            let source_line;
            let target_line;
            match line_type {
                LineType::Context => {
                    // Context lines exist in source and target
                    source_line = LineLocation::RealLocation(source_id);
                    source_id += 1;
                    target_line = LineLocation::RealLocation(target_id);
                    target_id += 1;
                }
                LineType::Add => {
                    // Added lines only exist in the target
                    source_line = LineLocation::ChangeLocation(source_id);
                    target_line = LineLocation::RealLocation(target_id);
                    target_id += 1;
                }
                LineType::Remove => {
                    // Removed lines only exist in the source
                    source_line = LineLocation::RealLocation(source_id);
                    source_id += 1;
                    target_line = LineLocation::ChangeLocation(target_id);
                }
                LineType::EOF => {
                    // EOF describe missing newline characters at the end of the file.
                    source_line = LineLocation::None;
                    target_line = LineLocation::None;
                }
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

/// A HunkLocation defines the location of a Hunk by its line number and length in lines.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct HunkLocation {
    hunk_start: usize,
    hunk_length: usize,
}

impl HunkLocation {
    /// Returns the start line number of this hunk. The first line has line number '1'.
    pub fn hunk_start(&self) -> usize {
        self.hunk_start
    }

    /// Returns the length of this hunk in lines (i.e., the number of HunkLines).
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

        if numbers.len() == 1 {
            // Sometimes, the location is only given by the location, but not with a length (i.e.,
            // if there is only one line.
            numbers.push(1);
        }

        Ok(HunkLocation {
            hunk_start: numbers[0],
            hunk_length: numbers[1],
        })
    }
}

/// A HunkLine contains the information about a single line in a Hunk.
/// A HunkLine stores the text of the line, its location in the source file, its location in the
/// target file, and its LineType.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HunkLine {
    line: String,
    source_line: LineLocation,
    target_line: LineLocation,
    line_type: LineType,
}

/// A LineLocation can be a RealLocation (i.e., the line actually exists in the file at that
/// location), a ChangeLocation (i.e., the line will be added or has been removed from this
/// location), or None (i.e., the line is not actually part of source or target).
/// The latter is the case for EOF markings in a hunk.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LineLocation {
    RealLocation(usize),
    ChangeLocation(usize),
    None,
}

impl LineLocation {
    /// Unwraps this LineLocation instance into its RealLocation value.
    ///
    /// # Panics
    /// This method panics if the LineLocation is not a RealLocation variant.
    pub fn real_location(&self) -> usize {
        if let LineLocation::RealLocation(value) = self {
            *value
        } else {
            panic!("not a RealLocation variant");
        }
    }

    /// Unwraps this LineLocation instance into its ChangeLocation value.
    ///
    /// # Panics
    /// This method panics if the LineLocation is not a ChangeLocation variant.
    pub fn change_location(&self) -> usize {
        if let LineLocation::ChangeLocation(value) = self {
            *value
        } else {
            panic!("not a ChangeLocation variant");
        }
    }
}

impl HunkLine {
    /// Returns the content (i.e., the text) of this line.
    pub fn content(&self) -> &str {
        &self.line
    }

    /// Returns the line type of this line.
    pub fn line_type(&self) -> LineType {
        self.line_type
    }

    /// Constructs a new HunkLine from the given locations, type, and text.
    pub fn new(
        source_line: LineLocation,
        target_line: LineLocation,
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

    /// Returns the line's location in the source file.
    pub fn source_line(&self) -> LineLocation {
        self.source_line
    }

    /// Returns the line's location in the target file.
    pub fn target_line(&self) -> LineLocation {
        self.target_line
    }

    /// Returns the content of the hunk line after the meta-symbol that defines the change type.
    pub fn into_original_text(mut self) -> String {
        self.line.split_off(1)
    }
}

impl Display for HunkLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.line)
    }
}

/// Defines the type of a HunkLine (i.e., Context, Add, Remove, EOF).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum LineType {
    /// A context line in a diff that starts with a space ' ' as first character and represents an
    /// unchanged line.
    Context,
    /// A line that has been added to the target file (i.e., it does not exist in the source file).
    Add,
    /// A line that has been removed from the target file (i.e., it only exists in the source
    /// file).
    Remove,
    /// An EOF metaline (i.e., "\No newline at end of file").
    EOF,
}

impl LineType {
    /// Determines the LineType of the given line.
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

/// A source file header holds the path to the source file and the timestamp of when it was read for
/// diffing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFileHeader {
    path: String,
    // TODO: Use actual time value
    timestamp: String,
}

impl SourceFileHeader {
    /// Returns the path to the source file as &str.
    pub fn path_str(&self) -> &str {
        &self.path
    }

    /// Returns the path to the source file as owned PathBuf.
    pub fn path(&self) -> PathBuf {
        PathBuf::from_str(&self.path).expect("paths must be UTF-8 encoded")
    }

    /// Returns the text of the timestamp of the time when this file was diffed.
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }
}

impl TryFrom<String> for SourceFileHeader {
    type Error = Error;

    fn try_from(line: String) -> Result<Self, Self::Error> {
        if !line.starts_with("--- ") {
            return Err(Error::new(
                "invalid format: line does not start with '--- '",
                ErrorKind::DiffParseError,
            ));
        }
        let (path, timestamp) = split_file_metainfo(line)?;
        Ok(Self { path, timestamp })
    }
}

impl TryFrom<&str> for SourceFileHeader {
    type Error = Error;

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        Self::try_from(line.to_string())
    }
}

/// A target file header holds the path to the target file and the timestamp of when it was read for
/// diffing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetFileHeader {
    path: String,
    // TODO: Use actual time value
    timestamp: String,
}

impl TargetFileHeader {
    /// Returns the path to the target file as &str.
    pub fn path_str(&self) -> &str {
        &self.path
    }

    /// Returns the path to the target file as owned PathBuf.
    pub fn path(&self) -> PathBuf {
        PathBuf::from_str(&self.path).expect("paths must be UTF-8 encoded")
    }

    /// Returns the text of the timestamp of the time when this file was diffed.
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }
}

impl TryFrom<String> for TargetFileHeader {
    type Error = Error;

    fn try_from(line: String) -> Result<Self, Self::Error> {
        if !line.starts_with("+++ ") {
            return Err(Error::new(
                "invalid format: line does not start with '+++ '",
                ErrorKind::DiffParseError,
            ));
        }
        let (path, timestamp) = split_file_metainfo(line)?;
        Ok(Self { path, timestamp })
    }
}

impl TryFrom<&str> for TargetFileHeader {
    type Error = Error;

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        Self::try_from(line.to_string())
    }
}

/// Splits the lines specifying the meta-information about the source and target files into file
/// path and timestamp.
fn split_file_metainfo(input: String) -> Result<(String, String), Error> {
    let parts: Vec<&str> = input.split_whitespace().collect();

    let path_id = 1;
    let path = parts[path_id].to_string();

    let mut timestamp = String::new();
    let timestamp_start = 2;
    for (i, part) in parts.into_iter().skip(timestamp_start).enumerate() {
        if i > 0 {
            // Add whitespace before each added part after the first one
            timestamp.push(' ');
        }
        timestamp.push_str(part);
    }

    Ok((path, timestamp))
}

#[cfg(test)]
mod tests {
    use crate::{
        diffs::{FileDiff, Hunk, LineType, TargetFileHeader, VersionDiff},
        ErrorKind,
    };

    use super::{HunkLine, SourceFileHeader};
    use super::{
        HunkLocation,
        LineLocation::{ChangeLocation, RealLocation},
    };

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
        let source = SourceFileHeader::try_from(line).unwrap();
        assert_eq!("version-A/double_end.txt", source.path);
        assert_eq!("2023-11-03 16:39:35.953263076 +0100", source.timestamp);
    }

    #[test]
    fn parse_valid_target_file() {
        let line = "+++ version-B/double_end.txt	2023-11-03 16:40:12.500153951 +0100";
        let source = TargetFileHeader::try_from(line).unwrap();
        assert_eq!("version-B/double_end.txt", source.path);
        assert_eq!("2023-11-03 16:40:12.500153951 +0100", source.timestamp);
    }

    #[test]
    fn recognize_invalid_source_file() {
        let line = "+++ version-A/double_end.txt	2023-11-03 16:39:35.953263076 +0100";
        assert!(SourceFileHeader::try_from(line).is_err());
    }

    #[test]
    fn recognize_invalid_target_file() {
        let line = "--- version-A/double_end.txt	2023-11-03 16:39:35.953263076 +0100";
        assert!(TargetFileHeader::try_from(line).is_err());
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
                RealLocation(1),
                RealLocation(2),
                LineType::Context,
                " context 1".to_string(),
            )
            .unwrap(),
            HunkLine::new(
                RealLocation(2),
                RealLocation(3),
                LineType::Context,
                " context 2".to_string(),
            )
            .unwrap(),
            HunkLine::new(
                RealLocation(3),
                RealLocation(4),
                LineType::Context,
                " context 3".to_string(),
            )
            .unwrap(),
            HunkLine::new(
                RealLocation(4),
                ChangeLocation(5),
                LineType::Remove,
                "-REMOVED".to_string(),
            )
            .unwrap(),
            HunkLine::new(
                ChangeLocation(5),
                RealLocation(5),
                LineType::Add,
                "+ADDED".to_string(),
            )
            .unwrap(),
            HunkLine::new(
                RealLocation(5),
                RealLocation(6),
                LineType::Context,
                " context 4".to_string(),
            )
            .unwrap(),
            HunkLine::new(
                RealLocation(6),
                RealLocation(7),
                LineType::Context,
                " context 5".to_string(),
            )
            .unwrap(),
            HunkLine::new(
                RealLocation(7),
                RealLocation(8),
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
        assert_eq!(
            file_diff.source_file_header.path,
            "version-A/long.txt".to_string()
        );
        assert_eq!(
            file_diff.target_file_header.path,
            "version-B/long.txt".to_string()
        );
        assert_eq!(
            file_diff.source_file_header.timestamp,
            "2023-11-03 16:26:28.701847364 +0100".to_string()
        );
        assert_eq!(
            file_diff.target_file_header.timestamp,
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
            (RealLocation(1), RealLocation(1)),
            (RealLocation(2), RealLocation(2)),
            (RealLocation(3), RealLocation(3)),
            (RealLocation(4), ChangeLocation(4)),
            (ChangeLocation(5), RealLocation(4)),
            (RealLocation(5), RealLocation(5)),
            (RealLocation(6), RealLocation(6)),
            (RealLocation(7), RealLocation(7)),
        ];

        for (i, line) in hunk.lines.iter().enumerate() {
            let (mut old_id, mut new_id) = expected_lines[i];
            println!("{line:?}");
            match old_id {
                RealLocation(v) => old_id = RealLocation(v + offset_old),
                ChangeLocation(v) => old_id = ChangeLocation(v + offset_old),
                crate::diffs::LineLocation::None => (),
            }
            match new_id {
                RealLocation(v) => new_id = RealLocation(v + offset_new),
                ChangeLocation(v) => new_id = ChangeLocation(v + offset_new),
                crate::diffs::LineLocation::None => (),
            }
            assert_eq!(line.source_line, old_id);
            assert_eq!(line.target_line, new_id);
        }
    }

    #[test]
    fn correctly_parse_version_diff() {
        let content = "
diff -Naur version-A/A.txt version-B/A.txt
--- version-A/A.txt	2023-11-03 16:26:28.701847364 +0100
+++ version-B/A.txt	2023-11-03 16:26:37.168563729 +0100
@@ -1,7 +1,7 @@
 context 1
 context 2
 context 3
-REMOVED
+ADDED
 context 4
 context 5
 context 6
diff -Naur version-A/B.txt version-B/B.txt
--- version-A/B.txt	2023-11-03 16:26:28.701847364 +0100
+++ version-B/B.txt	2023-11-03 16:26:37.168563729 +0100
@@ -1,7 +1,7 @@
 context 1
 context 2
 context 3
-REMOVED
+ADDED
 context 4
 context 5
 context 6";
        let version_diff = VersionDiff::try_from(content.trim_start().to_string()).unwrap();
        assert!(!version_diff.is_empty());
        assert_eq!(2, version_diff.len());
    }

    #[test]
    fn empty_diff() {
        let content = "";
        let result = VersionDiff::try_from(content.trim_start().to_string());
        let result = result.unwrap_err();
        assert_eq!(ErrorKind::DiffParseError, *result.kind());
        assert!(result.message().starts_with("the given diff is empty"));
    }

    #[test]
    fn invalid_file_diff_start() {
        let content = "
di -Naur version-A/B.txt version-B/B.txt
--- version-A/B.txt	2023-11-03 16:26:28.701847364 +0100
+++ version-B/B.txt	2023-11-03 16:26:37.168563729 +0100
@@ -1,7 +1,7 @@
 context 1
 context 2
 context 3
-REMOVED
+ADDED
 context 4
 context 5
 context 6";
        let mut content = prepare_diff_vec(content);
        content[0] = content[0].trim().to_string();
        let result = FileDiff::try_from(content.clone());

        let result = result.unwrap_err();
        assert_eq!(ErrorKind::DiffParseError, *result.kind());
        assert!(result.message().starts_with("invalid file diff start"));
    }

    #[test]
    fn invalid_empty_hunk_location() {
        let content = "";
        let result = HunkLocation::try_from(content).unwrap_err();
        assert_eq!(ErrorKind::DiffParseError, *result.kind());
        assert!(result.message().starts_with("invalid hunk location: "));
    }
}
