//! Lexical analyzer for Foxa.
//!
//! # Purpose
//!
//! Converts UTF-8 source text into a stream of [`Token`]s with precise spans.
//! The lexer is lossless for significant tokens and reports lexical errors
//! into a [`foxa_diagnostics::DiagnosticBag`] without panicking.
//!
//! # Architecture
//!
//! - [`TokenKind`] — token classification
//! - [`Token`] — kind + span + optional literal payload
//! - [`Lexer`] — stateful scanner over a source file
//!
//! # Example
//!
//! ```
//! use foxa_diagnostics::DiagnosticBag;
//! use foxa_lexer::Lexer;
//! use foxa_span::SourceMap;
//!
//! let mut map = SourceMap::new();
//! let file = map.add_file("t.foxa", "fn main() {}");
//! let source = map.get(file).unwrap().source.clone();
//! let mut bag = DiagnosticBag::new();
//! let tokens = Lexer::new(file, &source, &mut bag).tokenize_all();
//! assert!(!bag.has_errors());
//! assert!(!tokens.is_empty());
//! ```

#![deny(missing_docs)]

mod cursor;
mod token;

pub use cursor::Lexer;
pub use token::{LiteralKind, Token, TokenKind};
