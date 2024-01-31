use std::{collections::BTreeMap, fmt::Display, fs};

#[derive(Debug)]
pub struct FileArtifact {
    path: String,
    lines: BTreeMap<usize, String>,
}

impl FileArtifact {
    /// Read the content of the file under path and create a new FileArtifact from it.
    pub fn read(path: &str) -> Result<FileArtifact, std::io::Error> {
        let file_content = fs::read_to_string(path)?;
        Ok(FileArtifact::parse_content(path, file_content))
    }

    /// Write the content of this FileArtifact to the file under the given path. The file is
    /// created if it does not exist. This method will overwrite existing files.
    pub fn write_to(&self, path: &str) -> Result<(), std::io::Error> {
        fs::write(path, self.to_string())
    }

    /// Write the content of this FileArtifact back to the file from which it was loaded. This is meant
    /// to be used in cases where the content has been modified.
    pub fn write(&self) -> Result<(), std::io::Error> {
        fs::write(&self.path, self.to_string())
    }

    /// Individual function that can be called in unit tests without requiring a test file
    fn parse_content(path: &str, file_content: String) -> Self {
        let mut lines = BTreeMap::new();
        for (line_number, line) in file_content
            .lines()
            .map(|l| l.to_string())
            .enumerate()
            .map(|(id, l)| (id + 1, l))
        {
            lines.insert(line_number, line);
        }
        FileArtifact {
            path: path.to_string(),
            lines,
        }
    }
}

impl Display for FileArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lines = self.lines.values();
        // print the first line without newline character
        if let Some(line) = lines.next() {
            write!(f, "{line}")?;
        }
        for line in lines {
            write!(f, "\n{line}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::FileArtifact;

    #[test]
    // Assure that the content of a file is not manipulated by pure read and write operations
    fn read_write_equality() {
        let test_content = r"hello
        oh beautiful
        world!

        "
        .to_string();

        let artifact = FileArtifact::parse_content("UNUSED PATH", test_content.clone());

        assert_eq!(test_content, artifact.to_string());
    }
}
