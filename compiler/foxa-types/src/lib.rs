//! Type checker for Foxa.
//!
//! # Purpose
//!
//! Assigns types to expressions and verifies that operations, calls, and
//! bindings are well-typed. Phase 2 covers monomorphic typing with a fixed
//! set of primitive types and function signatures.
//!
//! # Architecture
//!
//! - [`Ty`] — type representation
//! - [`TypeEnv`] — maps symbols to types
//! - [`TypeChecker`] — walks AST using a [`foxa_resolve::ResolveMap`]
//!
//! # Example
//!
//! ```
//! use foxa_diagnostics::DiagnosticBag;
//! use foxa_lexer::Lexer;
//! use foxa_parser::Parser;
//! use foxa_resolve::Resolver;
//! use foxa_span::SourceMap;
//! use foxa_types::TypeChecker;
//!
//! let src = "fn main() { let x = 1; }";
//! let mut map = SourceMap::new();
//! let file = map.add_file("t.foxa", src);
//! let mut bag = DiagnosticBag::new();
//! let tokens = Lexer::new(file, src, &mut bag).tokenize_all();
//! let module = Parser::new(file, src, tokens, &mut bag).parse_module();
//! let resolved = Resolver::new(&mut bag).resolve(&module);
//! TypeChecker::new(&resolved, &mut bag).check(&module);
//! assert!(!bag.has_errors());
//! ```

#![deny(missing_docs)]

mod checker;
mod ty;

pub use checker::{TypeChecker, TypeMap};
pub use ty::Ty;
