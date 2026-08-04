//! Source locations for the Foxa compiler.
//!
//! # Purpose
//!
//! Every diagnostic, AST node, and IR instruction carries a [`Span`] that
//! points into a source file registered in a [`SourceMap`]. Spans are cheap
//! (`Copy`) and stable across incremental recompiles of unchanged files.
//!
//! # Architecture
//!
//! - [`FileId`] — opaque handle into the source map
//! - [`BytePos`] — zero-based UTF-8 byte offset
//! - [`Span`] — half-open range `[lo, hi)` within one file
//! - [`SourceFile`] — file path, contents, and line-index cache
//! - [`SourceMap`] — registry of all source files for a compilation
//!
//! # Example
//!
//! ```
//! use foxa_span::{SourceMap, Span};
//!
//! let mut map = SourceMap::new();
//! let file = map.add_file("main.foxa", "fn main() {}\n");
//! let span = Span::new(file, 0, 2); // "fn"
//! assert_eq!(map.snippet(span), Some("fn"));
//! ```

#![deny(missing_docs)]

mod line_index;
mod source_map;
mod span;

pub use line_index::LineIndex;
pub use source_map::{SourceFile, SourceMap};
pub use span::{BytePos, FileId, Location, Span};
