//! Tree-walk interpreter for Foxa.
//!
//! # Purpose
//!
//! Executes type-checked Foxa programs without native codegen. Used by
//! `foxa run` until LLVM/Cranelift backends are ready, and as a reference
//! semantics oracle for later backends.
//!
//! # Architecture
//!
//! - [`Value`] — runtime values
//! - [`Environment`] — nested binding environments
//! - [`Interpreter`] — evaluates modules, calling `main` by default
//!
//! # Example
//!
//! ```
//! use foxa_diagnostics::DiagnosticBag;
//! use foxa_interp::Interpreter;
//! use foxa_lexer::Lexer;
//! use foxa_parser::Parser;
//! use foxa_resolve::Resolver;
//! use foxa_span::SourceMap;
//! use foxa_types::TypeChecker;
//!
//! let src = "fn main() { print(\"hi\"); }";
//! let mut map = SourceMap::new();
//! let file = map.add_file("t.foxa", src);
//! let mut bag = DiagnosticBag::new();
//! let tokens = Lexer::new(file, src, &mut bag).tokenize_all();
//! let module = Parser::new(file, src, tokens, &mut bag).parse_module();
//! let resolved = Resolver::new(&mut bag).resolve(&module);
//! TypeChecker::new(&resolved, &mut bag).check(&module);
//! assert!(!bag.has_errors());
//! Interpreter::new(&module).run_main().unwrap();
//! ```

#![deny(missing_docs)]

mod env;
mod error;
mod value;
mod vm;

pub use error::InterpError;
pub use value::Value;
pub use vm::Interpreter;
