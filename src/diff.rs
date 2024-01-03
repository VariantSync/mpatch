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
        for line in content.lines() {
            // TODO:
            if line.starts_with("diff ") {}
            file_diff_content.push(line)
        }
        // Parse the collected lines to a FileDiff
        // Repeat until all lines have been processed

        todo!()
    }
}

pub struct FileDiff {
    source_file: SourceFile,
    target_file: TargetFile,
    hunks: Vec<Hunk>,
}

impl FileDiff {
    pub fn text(&self) -> &str {
        todo!();
    }
}

pub struct Hunk {
    source_location: HunkLocation,
    target_location: HunkLocation,
    lines: Vec<HunkLine>,
}

pub struct HunkLocation {
    hunk_start: usize,
    hunk_length: usize,
}

#[derive(Debug)]
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
        if content.as_str() == "\\No newline at end of file" {
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
                _ => return Err(Error::new("invalid hunk line", ErrorKind::DiffParseError)),
            }
        } else {
            return Err(Error::new("invalid hunk line", ErrorKind::DiffParseError));
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

pub struct TargetFile {
    path: String,
    // TODO: Use actual time value
    timestamp: String,
}

#[cfg(test)]
mod tests {
    use crate::diff::LineType;

    use super::HunkLine;

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
        let line = "\\No newline at end of file";
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
}
