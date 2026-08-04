//! Byte positions, file IDs, and spans.

use std::fmt;

/// Opaque identifier for a source file in a [`crate::SourceMap`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(u32);

impl FileId {
    /// Creates a file ID from a raw index. Prefer [`crate::SourceMap::add_file`].
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw numeric ID.
    #[must_use]
    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileId({})", self.0)
    }
}

/// Zero-based UTF-8 byte offset within a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BytePos(u32);

impl BytePos {
    /// Creates a byte position from a raw offset.
    #[must_use]
    pub const fn from_u32(pos: u32) -> Self {
        Self(pos)
    }

    /// Returns the raw offset.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Returns the offset as `usize`.
    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for BytePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Half-open byte range `[lo, hi)` within a single source file.
///
/// Spans are `Copy` and intentionally tiny so they can be attached to every
/// AST node without allocation pressure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// File containing this span.
    pub file_id: FileId,
    /// Inclusive start byte offset.
    pub lo: BytePos,
    /// Exclusive end byte offset.
    pub hi: BytePos,
}

impl Span {
    /// Creates a span from file ID and byte offsets.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `lo > hi`.
    #[must_use]
    pub fn new(file_id: FileId, lo: u32, hi: u32) -> Self {
        debug_assert!(lo <= hi, "span lo ({lo}) must be <= hi ({hi})");
        Self {
            file_id,
            lo: BytePos::from_u32(lo),
            hi: BytePos::from_u32(hi),
        }
    }

    /// Creates a zero-width span at `pos`.
    #[must_use]
    pub fn at(file_id: FileId, pos: u32) -> Self {
        Self::new(file_id, pos, pos)
    }

    /// Returns the length in bytes.
    #[must_use]
    pub fn len(self) -> u32 {
        self.hi.as_u32().saturating_sub(self.lo.as_u32())
    }

    /// Returns `true` if the span has zero length.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.lo == self.hi
    }

    /// Merges two spans from the same file into their covering range.
    ///
    /// # Panics
    ///
    /// Panics if the spans belong to different files.
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        assert_eq!(
            self.file_id, other.file_id,
            "cannot merge spans from different files"
        );
        Self::new(
            self.file_id,
            self.lo.as_u32().min(other.lo.as_u32()),
            self.hi.as_u32().max(other.hi.as_u32()),
        )
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}-{}",
            self.file_id.as_raw(),
            self.lo.as_u32(),
            self.hi.as_u32()
        )
    }
}

/// Human-readable line/column location (1-based lines and columns).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    /// File containing this location.
    pub file_id: FileId,
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number (Unicode scalar values, approximately).
    pub column: u32,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_len_and_empty() {
        let id = FileId::from_raw(0);
        let s = Span::new(id, 10, 15);
        assert_eq!(s.len(), 5);
        assert!(!s.is_empty());
        assert!(Span::at(id, 10).is_empty());
    }

    #[test]
    fn span_merge() {
        let id = FileId::from_raw(0);
        let a = Span::new(id, 0, 5);
        let b = Span::new(id, 3, 10);
        assert_eq!(a.merge(b), Span::new(id, 0, 10));
    }

    #[test]
    #[should_panic(expected = "different files")]
    fn span_merge_different_files_panics() {
        let a = Span::new(FileId::from_raw(0), 0, 1);
        let b = Span::new(FileId::from_raw(1), 0, 1);
        let _ = a.merge(b);
    }
}
