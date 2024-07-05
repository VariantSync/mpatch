use std::{fmt::Display, fs};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::{patch::Change, Error};

/// Prints the given rejects with print!
pub fn print_rejects(diff_header: String, rejects: &[Change]) {
    println!("{diff_header}");
    for reject in rejects {
        print!("{}: {}", reject.change_id(), reject);
    }
}

/// Writes the given diff header and the rejects of the diff to the specified file.
pub fn write_rejects<P: AsRef<Path>>(
    diff_header: String,
    rejects: &[Change],
    rejects_file: &mut Option<BufWriter<File>>,
    path: P,
) -> Result<(), Error> {
    // Create the rejects file on demand
    let file_writer = rejects_file.get_or_insert_with(|| {
        BufWriter::new(File::create_new(&path).expect("was not able to create rejects file"))
    });
    file_writer.write_fmt(format_args!("{}\n", diff_header))?;
    for reject in rejects {
        file_writer.write_fmt(format_args!("{}: {}", reject.change_id(), reject))?
    }
    file_writer.flush()?;
    Ok(())
}

/// Represents a file that can be patched. Each file artifact tracks the path to the file on disk
/// and the content of the file in lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileArtifact {
    path: PathBuf,
    lines: Vec<String>,
}

impl FileArtifact {
    /// Creates a new empty file artifact with the given path.
    pub fn new(path: PathBuf) -> FileArtifact {
        FileArtifact {
            path,
            lines: vec![],
        }
    }

    /// Creates a new file artifact with the given path and lines.
    pub fn from_lines(path: PathBuf, lines: Vec<String>) -> FileArtifact {
        FileArtifact { path, lines }
    }

    /// Reads the content of the file under path and creates a new FileArtifact from it.
    pub fn read<P: AsRef<Path>>(path: P) -> Result<FileArtifact, Error> {
        let content = fs::read_to_string(&path)?;
        Ok(FileArtifact::parse_content(path, content))
    }

    /// Reads the contents of a file as file artifact or creates an empty FileArtifact instance
    /// if no corresponding file exists. This function does not create new files on disk, only
    /// representations in memory.
    pub fn read_or_create_empty(pathbuf: PathBuf) -> Result<FileArtifact, Error> {
        Ok(if Path::exists(&pathbuf) {
            FileArtifact::read(&pathbuf)?
        } else {
            FileArtifact::new(pathbuf)
        })
    }

    /// Writes the content of this FileArtifact back to the file from which it was loaded. This is meant
    /// to be used in cases where the content has been modified.
    pub fn write(&self) -> Result<(), std::io::Error> {
        fs::write(&self.path, self.to_string())
    }

    /// Returns the number of lines in this file artifact.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Returns true if this file artifact has no lines; otherwise, returns false.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Creates a new file artifact from the given path and content.
    fn parse_content<P: AsRef<Path>>(path: P, file_content: String) -> Self {
        let mut lines = vec![];
        for line in file_content.lines().map(|l| l.to_string()) {
            lines.push(line);
        }
        FileArtifact {
            path: path.as_ref().to_path_buf(),
            lines,
        }
    }

    /// Returns a reference to the lines of this file artifact.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Consumes this file artifact and returns its lines.
    pub fn into_lines(self) -> Vec<String> {
        self.lines
    }

    /// Destructures this file artifact into its fields.
    pub fn into_path_and_lines(self) -> (PathBuf, Vec<String>) {
        (self.path, self.lines)
    }

    /// Returns a reference to the path of this file artifact.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Display for FileArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lines = self.lines.iter();
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

/// A helper trait for adding stripping functionality to paths represented by PathBuf.
/// Stripping a path means that the first n parts of the path are removed.
/// For instance if the path `mpatch/src/io.rs` is stripped by `2` the result is `io.rs`.
pub trait StrippedPath {
    /// Skips the first `strip` parts of the path and then clones the remaining parts into a
    /// new PathBuf that is returned.
    /// For instance if the path `mpatch/src/io.rs` is stripped by `2` the result is `io.rs`.
    fn strip_cloned(&self, strip: usize) -> PathBuf;
}

impl StrippedPath for PathBuf {
    fn strip_cloned(&self, strip: usize) -> PathBuf {
        self.iter().skip(strip).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::{FileArtifact, StrippedPath};

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
        assert!(!artifact.is_empty());
        assert_eq!(5, artifact.len());
    }

    #[test]
    fn path_strip_single() {
        let path = PathBuf::from_str("hello/world").unwrap();
        assert_eq!(path.strip_cloned(1).to_str().unwrap(), "world");
        assert_eq!(path.strip_cloned(2).to_str().unwrap(), "");
    }

    #[test]
    fn path_strip_multiple() {
        let path = PathBuf::from_str("hello/world/you//are/beautiful").unwrap();
        assert_eq!(path.strip_cloned(2).to_str().unwrap(), "you/are/beautiful");
        assert_eq!(path.strip_cloned(5).to_str().unwrap(), "");
    }

    #[test]
    fn from_stripped() {
        let path = PathBuf::from_str("hello/world").unwrap();
        let stripped = PathBuf::strip_cloned(&path, 1);
        assert_eq!(stripped.to_str().unwrap(), "world");
        let stripped = PathBuf::strip_cloned(&path, 2);
        assert_eq!(stripped.to_str().unwrap(), "");
    }
}
