use similar::TextDiff;

use crate::FileArtifact;

/// A trait for defining a common interface for matchers that match lines between two files.
///
/// Matchers are used by mpatch to determine the alignment for a patch. This means that mpatch
/// decides where to apply a patch based on the matching provided by a matcher.
///
/// ## How to implement
/// Ideally, a matcher implementation uses a dedicated matching algorithm to determine the
/// differences and commonalities between two files (e.g., LCS).
///
/// A naive implementation of a matcher could iterate over the lines in both files, matching lines
/// if they have the same content.
/// ```
/// struct NaiveMatcher;
///
/// impl Matcher for NaiveMatcher {
///     fn match_files(&mut self, source: FileArtifact, target: FileArtifact) -> Matching {
///         // Initialze the vectors holding the match ids
///         let mut source_to_target = Vec::with_capacity(source.len());
///         let mut target_to_source = Vec::with_capacity(target.len());
///
///         // Add an entry for each line in the source and target file. Each line must have an entry in
///         // the vector at position `line_number-1`.
///         // The match for a line is stored as line number of its counterpart in the other file
///         // without -1 offset.
///         // This means that if the first line of both files matches the entries of the vectors look
///         // as follows:
///         // source_to_target[0] == Some(1)
///         // target_to_source[0] == Some(1)
///         //
///         // Note that the getter methods of the Matching struct abstract this implementation detail:
///         // matching.target_index(1) == Some(1)
///         // matching.source_index(1) == Some(1)
///         for (line_number, (source_line, target_line)) in
///             source.lines().iter().zip(target.lines()).enumerate()
///         {
///             if source_line == target_line {
///                 source_to_target.push(Some(line_number));
///                 target_to_source.push(Some(line_number));
///             } else {
///                 source_to_target.push(None);
///                 target_to_source.push(None);
///             }
///         }
///         Matching::new(source, target, source_to_target, target_to_source)
///     }
/// }
///
/// // Now we can use the matcher!
///
/// // Initialze some simple FileArtifacts
/// let file_a = FileArtifact::from_lines(
///     PathBuf::from_str("file_a").unwrap(),
///     vec!["SAME LINE".to_string(), "DIFFERENT LINE".to_string()],
/// );
/// let file_b = FileArtifact::from_lines(
///     PathBuf::from_str("file_b").unwrap(),
///     vec!["SAME LINE".to_string(), "DIFFERENT    LINE".to_string()],
/// );
///
/// // Call the matcher
/// let mut matcher = NaiveMatcher;
/// let matching = matcher.match_files(file_a, file_b);
///
/// // The first line matches
/// assert_eq!(matching.target_index(1).unwrap(), Some(1));
/// assert_eq!(matching.source_index(1).unwrap(), Some(1));
///
/// // The second line does not match; there is no matching for source and target
/// assert_eq!(matching.target_index(2).unwrap(), None);
/// assert_eq!(matching.source_index(2).unwrap(), None);
///
/// // There is no matching for a third line, because there was no third line in both files
/// assert!(matching.target_index(3).is_none());
/// assert!(matching.source_index(3).is_none());
/// ```
pub trait Matcher {
    /// Determines the matching between the two fiven files. The matching takes ownership of the
    /// files to ensure that they are not changed by some other code, which would invalidate the
    /// matching, and to allow for easy access to lines depending on a match id.
    ///
    /// # Examples
    /// The following is an example of a naive implementation that matches lines if they have the
    /// same line number and content.
    /// ```
    ///fn match_files(&mut self, source: FileArtifact, target: FileArtifact) -> Matching {
    ///    // Initialze the vectors holding the match ids
    ///    let mut source_to_target = Vec::with_capacity(source.len());
    ///    let mut target_to_source = Vec::with_capacity(target.len());
    ///
    ///    // Add an entry for each line in the source and target file. Each line must have an entry in
    ///    // the vector at position `line_number-1`.
    ///    // The match for a line is stored as line number of its counterpart in the other file
    ///    // without -1 offset.
    ///    // This means that if the first line of both files matches the entries of the vectors look
    ///    // as follows:
    ///    // source_to_target[0] == Some(1)
    ///    // target_to_source[0] == Some(1)
    ///    //
    ///    // Note that the getter methods of the Matching struct abstract this implementation detail:
    ///    // matching.target_index(1) == Some(1)
    ///    // matching.source_index(1) == Some(1)
    ///    for (line_number, (source_line, target_line)) in
    ///        source.lines().iter().zip(target.lines()).enumerate()
    ///    {
    ///        if source_line == target_line {
    ///            source_to_target.push(Some(line_number));
    ///            target_to_source.push(Some(line_number));
    ///        } else {
    ///            source_to_target.push(None);
    ///            target_to_source.push(None);
    ///        }
    ///    }
    ///    Matching::new(source, target, source_to_target, target_to_source)
    ///}
    fn match_files(&mut self, source: FileArtifact, target: FileArtifact) -> Matching;
}

/// A matching holds the information about lines that have been matched between a source and a
/// target file. To this end, the matching controls two vectors of match ids: one with matchings
/// for the lines in the source file, and one with matchings for lines in the target file.
/// This allows for quick access to the matches by line number.
///
/// Furthermore, a matching owns the instances of the FileArtifacts that have been matched. This
/// ensures that the matched FileArtifacts are not altered. Note that this does not prevent the
/// actual file being modified on disk.
pub struct Matching {
    source: FileArtifact,
    target: FileArtifact,
    source_to_target: Vec<MatchId>,
    target_to_source: Vec<MatchId>,
}

/// A MatchId is simply and Option<usize> where the usize is a line number in the interval [1,n].
pub type MatchId = Option<usize>;

impl Matching {
    /// Creates a new Matching from the given source and target files and match id vectors.  
    /// Each line in the source and target must have an entry in the corresponding d vector at position `line_number-1`.
    /// The match for a line is stored as line number of its counterpart in the other file without -1 offset.
    /// This means that if the first line of both files matches, the entries of the vectors look as follows:
    /// source_to_target[0] == Some(1)
    /// target_to_source[0] == Some(1)
    ///
    /// Note that the getter methods of the Matching struct abstract this implementation detail:
    /// matching.target_index(1) == Some(1)
    /// matching.source_index(1) == Some(1)
    pub fn new(
        source: FileArtifact,
        target: FileArtifact,
        source_to_target: Vec<MatchId>,
        target_to_source: Vec<MatchId>,
    ) -> Matching {
        Matching {
            source,
            target,
            source_to_target,
            target_to_source,
        }
    }

    /// Returns the match in the target file for a line number of the source file.
    ///
    /// ## Input
    /// source_index: specifies the line number of a line in the source file for which the match
    /// should be retrieved.
    ///
    /// ## Output
    /// Returns None if the source line has not been processed by the matcher. Returns
    /// Some(MatchId) if the source line has been processed. The returned MatchId is Some if there
    /// is a match in the target file; otherwise, it is None.
    pub fn target_index(&self, source_index: usize) -> Option<MatchId> {
        // To represent line numbers in files we offset the index by '1'
        // A negative offset is applied to the input index (e.g., line 1 is stored at index 0)
        // A positive offset is applied to the retrieved counterpart index (e.g., the counterpart
        // of line 1 is also line 1, which is stored as a 0).
        self.source_to_target
            .get(source_index - 1)
            .copied()
            .map(|v| v.map(|v| v + 1))
    }

    /// Returns the match in the source file for a line number of the target file.
    ///
    /// ## Input
    /// target_index: specifies the line number of a line in the target file for which the match
    /// should be retrieved.
    ///
    /// ## Output
    /// Returns None if the target line has not been processed by the matcher. Returns
    /// Some(MatchId) if the target line has been processed. The returned MatchId is Some if there
    /// is a match in the source file; otherwise, it is None.
    pub fn source_index(&self, target_index: usize) -> Option<MatchId> {
        self.target_to_source
            .get(target_index - 1)
            .copied()
            .map(|v| v.map(|v| v + 1))
    }

    /// Returns a reference to the source file instance.
    pub fn source(&self) -> &FileArtifact {
        &self.source
    }

    /// Returns a reference to the target file instance.
    pub fn target(&self) -> &FileArtifact {
        &self.target
    }

    /// Consumes this matching and returns ownership of the source file.
    pub fn into_source(self) -> FileArtifact {
        self.source
    }

    /// Consumes this matching and returns ownership of the target file.
    pub fn into_target(self) -> FileArtifact {
        self.target
    }

    /// Searches for closest line above the given source line that has a match in the target file.
    /// This means considers the source lines above the given line number until a line with a match
    /// in the target file is found. It then returns the match id of the corresponding target line.
    /// If the given line number has a match itself, this match is returned.
    ///
    /// ## Input
    /// source_index: specifies the line number of a line in the source file for which the fuzzy match
    /// should be retrieved.
    ///
    /// ## Output
    /// Returns None if there is no matched line at or above the given line number. Returns
    /// Some(usize) with the target line number if a match has been found.
    pub(crate) fn target_index_fuzzy(&self, line_number: usize) -> MatchId {
        let mut line_number = line_number;

        // Search for the closest context line above the change; i.e., key and value must both be
        // Some(...)
        while line_number > 0 && self.target_index(line_number).flatten().is_none() {
            line_number -= 1;
        }

        if line_number == 0 {
            // Line numbers start at '1', so there is no valid target index for '0'
            None
        } else {
            let target_line = self.target_index(line_number);
            // The result must be Some(...) in all cases
            target_line.unwrap()
        }
    }
}

/// A simple matcher using the `similar` crate which offers implementations of the LCS algorithm.
pub struct LCSMatcher;

impl LCSMatcher {
    /// Creates a new LCSMatcher
    pub fn new() -> Self {
        LCSMatcher
    }
}

impl Default for LCSMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Matcher for LCSMatcher {
    fn match_files(&mut self, left: FileArtifact, right: FileArtifact) -> Matching {
        let left_text = left.to_string();
        let right_text = right.to_string();
        let text_diff = TextDiff::from_lines(&left_text, &right_text);
        let mut left_to_right = Vec::with_capacity(left.len());
        let mut right_to_left = Vec::with_capacity(right.len());

        let mut last_line = None;
        for c in text_diff.iter_all_changes() {
            if c.old_index().is_some() {
                assert_eq!(c.old_index().unwrap(), left_to_right.len());
                left_to_right.push(c.new_index());
            }
            if c.new_index().is_some() {
                assert_eq!(c.new_index().unwrap(), right_to_left.len());
                right_to_left.push(c.old_index());
            }
            last_line.replace(c);
        }

        // Handle newlines at EOF, by creating an additional matching for the next line
        if let Some(last_line) = last_line {
            if !last_line.missing_newline() {
                if last_line.old_index().is_some() {
                    left_to_right.push(last_line.new_index().map(|i| i + 1));
                }
                if last_line.new_index().is_some() {
                    right_to_left.push(last_line.old_index().map(|i| i + 1));
                }
            }
        }

        Matching::new(left, right, left_to_right, right_to_left)
    }
}

#[cfg(test)]
mod tests {}
