use crate::Error;

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

pub enum HunkLine {
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
    #[test]
    fn PLACEHOLDER() {
        todo!();
    }

    #[test]
    fn PLACEHOLDER() {
        todo!();
    }

    #[test]
    fn PLACEHOLDER() {
        todo!();
    }

    #[test]
    fn PLACEHOLDER() {
        todo!();
    }

    #[test]
    fn PLACEHOLDER() {
        todo!();
    }

    #[test]
    fn PLACEHOLDER() {
        todo!();
    }
}
