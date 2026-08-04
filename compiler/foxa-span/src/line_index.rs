//! Line-index cache for converting byte offsets to line/column.

/// Maps byte offsets to 1-based line numbers using a sorted newline table.
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offset of the start of each line (line 1 starts at 0).
    line_starts: Vec<u32>,
}

impl LineIndex {
    /// Builds a line index by scanning `source` for newline bytes.
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0u32];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push((i + 1) as u32);
            }
        }
        Self { line_starts }
    }

    /// Returns the 1-based line number for `pos`.
    #[must_use]
    pub fn line_of(&self, pos: u32) -> u32 {
        match self.line_starts.binary_search(&pos) {
            Ok(idx) => (idx as u32) + 1,
            Err(idx) => idx as u32,
        }
    }

    /// Returns the byte offset where the given 1-based line starts.
    #[must_use]
    pub fn line_start(&self, line: u32) -> Option<u32> {
        if line == 0 {
            return None;
        }
        self.line_starts.get((line - 1) as usize).copied()
    }

    /// Returns 1-based column for `pos` (counting Unicode scalar values).
    #[must_use]
    pub fn column_of(&self, source: &str, pos: u32) -> u32 {
        let line = self.line_of(pos);
        let start = self.line_start(line).unwrap_or(0) as usize;
        let end = (pos as usize).min(source.len());
        if start > end {
            return 1;
        }
        let col = source[start..end].chars().count() as u32;
        col + 1
    }

    /// Number of lines in the source (at least 1 for empty input).
    #[must_use]
    pub fn line_count(&self) -> u32 {
        self.line_starts.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_has_one_line() {
        let idx = LineIndex::new("");
        assert_eq!(idx.line_count(), 1);
        assert_eq!(idx.line_of(0), 1);
    }

    #[test]
    fn multiline_indexing() {
        let src = "abc\ndef\ng";
        let idx = LineIndex::new(src);
        assert_eq!(idx.line_of(0), 1);
        assert_eq!(idx.line_of(3), 1); // '\n'
        assert_eq!(idx.line_of(4), 2); // 'd'
        assert_eq!(idx.line_of(8), 3); // 'g'
        assert_eq!(idx.column_of(src, 5), 2); // 'e'
    }
}
