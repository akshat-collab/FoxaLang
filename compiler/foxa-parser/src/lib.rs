//! Pratt + recursive-descent parser for Foxa.
//!
//! # Purpose
//!
//! Turns a token stream into an [`foxa_ast::Module`]. Declarations use
//! recursive descent; expressions use Pratt (precedence climbing).
//!
//! # Architecture
//!
//! - [`Parser`] — owns tokens, diagnostics, and parse position
//! - Statement/item parsers are recursive descent
//! - [`Parser::parse_expr`] uses binding powers for operators
//!
//! # Example
//!
//! ```
//! use foxa_diagnostics::DiagnosticBag;
//! use foxa_lexer::Lexer;
//! use foxa_parser::Parser;
//! use foxa_span::SourceMap;
//!
//! let src = "fn main() { let x = 1; }";
//! let mut map = SourceMap::new();
//! let file = map.add_file("t.foxa", src);
//! let mut bag = DiagnosticBag::new();
//! let tokens = Lexer::new(file, src, &mut bag).tokenize_all();
//! let module = Parser::new(file, src, tokens, &mut bag).parse_module();
//! assert!(!bag.has_errors());
//! assert_eq!(module.items.len(), 1);
//! ```

#![deny(missing_docs)]

mod parser;

pub use parser::Parser;
