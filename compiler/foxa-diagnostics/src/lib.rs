//! Structured diagnostics for the Foxa compiler.
//!
//! # Purpose
//!
//! Provides a unified model for errors, warnings, and notes with source
//! spans, labels, and actionable help text — inspired by rustc diagnostics.
//!
//! # Architecture
//!
//! - [`Severity`] — error / warning / note / help
//! - [`Diagnostic`] — primary message plus labeled spans
//! - [`DiagnosticBag`] — accumulates diagnostics for a compilation
//! - [`Emitter`] — renders diagnostics to a writer (terminal or JSON)
//!
//! # Example
//!
//! ```
//! use foxa_diagnostics::{Diagnostic, DiagnosticBag, Severity};
//! use foxa_span::{FileId, Span};
//!
//! let mut bag = DiagnosticBag::new();
//! let span = Span::new(FileId::from_raw(0), 0, 1);
//! bag.push(
//!     Diagnostic::new(Severity::Error, "unexpected token")
//!         .with_label(span, "here")
//!         .with_help("expected `;` or `}`"),
//! );
//! assert!(bag.has_errors());
//! ```

#![deny(missing_docs)]

mod diagnostic;
mod emitter;

pub use diagnostic::{Diagnostic, DiagnosticBag, Label, Severity};
pub use emitter::{Emitter, RenderStyle};
