//! Source file registry and snippet extraction.

use crate::line_index::LineIndex;
use crate::span::{BytePos, FileId, Location, Span};
use std::path::{Path, PathBuf};

/// A single source file registered in a [`SourceMap`].
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Stable file identifier.
    pub id: FileId,
    /// Display path (may be virtual, e.g. `<stdin>`).
    pub path: PathBuf,
    /// Full UTF-8 source text.
    pub source: String,
    line_index: LineIndex,
}

impl SourceFile {
    /// Creates a source file. Prefer [`SourceMap::add_file`].
    #[must_use]
    pub fn new(id: FileId, path: impl Into<PathBuf>, source: impl Into<String>) -> Self {
        let source = source.into();
        let line_index = LineIndex::new(&source);
        Self {
            id,
            path: path.into(),
            source,
            line_index,
        }
    }

    /// Converts a byte position to a 1-based line/column location.
    #[must_use]
    pub fn lookup(&self, pos: BytePos) -> Location {
        let p = pos.as_u32();
        Location {
            file_id: self.id,
            line: self.line_index.line_of(p),
            column: self.line_index.column_of(&self.source, p),
        }
    }

    /// Returns the source text covered by `span`, if in range.
    #[must_use]
    pub fn snippet(&self, span: Span) -> Option<&str> {
        if span.file_id != self.id {
            return None;
        }
        let lo = span.lo.as_usize();
        let hi = span.hi.as_usize();
        self.source.get(lo..hi)
    }

    /// Returns the path as a display string.
    #[must_use]
    pub fn path_str(&self) -> String {
        self.path.display().to_string()
    }
}

/// Registry of source files for one compilation session.
#[derive(Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    /// Creates an empty source map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a source file and returns its [`FileId`].
    pub fn add_file(&mut self, path: impl Into<PathBuf>, source: impl Into<String>) -> FileId {
        let id = FileId::from_raw(self.files.len() as u32);
        self.files.push(SourceFile::new(id, path, source));
        id
    }

    /// Returns a source file by ID.
    #[must_use]
    pub fn get(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(id.as_raw() as usize)
    }

    /// Returns all registered files.
    #[must_use]
    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    /// Looks up a location for a byte position.
    #[must_use]
    pub fn lookup(&self, file_id: FileId, pos: BytePos) -> Option<Location> {
        self.get(file_id).map(|f| f.lookup(pos))
    }

    /// Returns the snippet for a span.
    #[must_use]
    pub fn snippet(&self, span: Span) -> Option<&str> {
        self.get(span.file_id)?.snippet(span)
    }

    /// Formats `file:line:column` for the start of a span.
    #[must_use]
    pub fn format_span_start(&self, span: Span) -> Option<String> {
        let file = self.get(span.file_id)?;
        let loc = file.lookup(span.lo);
        Some(format!("{}:{}:{}", file.path_str(), loc.line, loc.column))
    }

    /// Finds a file by path (exact match).
    #[must_use]
    pub fn find_by_path(&self, path: &Path) -> Option<&SourceFile> {
        self.files.iter().find(|f| f.path == path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_snippet() {
        let mut map = SourceMap::new();
        let id = map.add_file("main.foxa", "fn main() {}\n");
        let span = Span::new(id, 0, 2);
        assert_eq!(map.snippet(span), Some("fn"));
        assert_eq!(
            map.format_span_start(span).as_deref(),
            Some("main.foxa:1:1")
        );
    }

    #[test]
    fn lookup_multiline() {
        let mut map = SourceMap::new();
        let id = map.add_file("t.foxa", "a\nbc");
        let loc = map.lookup(id, BytePos::from_u32(2)).unwrap();
        assert_eq!(loc.line, 2);
        assert_eq!(loc.column, 1);
    }
}
