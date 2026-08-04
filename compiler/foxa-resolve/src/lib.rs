//! Name resolution for Foxa.
//!
//! # Purpose
//!
//! Builds scoped symbol tables and resolves identifier references to
//! definitions. Detects duplicate definitions and unknown names.
//!
//! # Architecture
//!
//! - [`SymbolId`] / [`Symbol`] — definitions (functions, locals, params)
//! - [`Scope`] — nested lexical scopes with parent links
//! - [`ResolveMap`] — maps expression spans to resolved symbols
//! - [`Resolver`] — walks the AST and populates the map
//!
//! # Example
//!
//! ```
//! use foxa_diagnostics::DiagnosticBag;
//! use foxa_lexer::Lexer;
//! use foxa_parser::Parser;
//! use foxa_resolve::Resolver;
//! use foxa_span::SourceMap;
//!
//! let src = "fn main() { let x = 1; }";
//! let mut map = SourceMap::new();
//! let file = map.add_file("t.foxa", src);
//! let mut bag = DiagnosticBag::new();
//! let tokens = Lexer::new(file, src, &mut bag).tokenize_all();
//! let module = Parser::new(file, src, tokens, &mut bag).parse_module();
//! let resolved = Resolver::new(&mut bag).resolve(&module);
//! assert!(!bag.has_errors());
//! assert!(resolved.symbols().len() >= 2); // main + x
//! ```

#![deny(missing_docs)]

mod resolver;
mod scope;
mod symbol;

pub use resolver::{ResolveMap, Resolver};
pub use scope::{Scope, ScopeId};
pub use symbol::{Symbol, SymbolId, SymbolKind};
