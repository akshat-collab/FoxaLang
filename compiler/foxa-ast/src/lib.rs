//! Abstract syntax tree for Foxa.
//!
//! # Purpose
//!
//! Defines the concrete syntax tree produced by the parser. Nodes carry
//! [`foxa_span::Span`]s for diagnostics and IDE features.
//!
//! # Architecture
//!
//! Covers modules, functions, structs, enums, statements (including loops),
//! and expressions (including field access, struct literals, and match).
//!
//! # Example
//!
//! ```
//! use foxa_ast::{Expr, ExprKind, Lit};
//! use foxa_span::{FileId, Span};
//!
//! let span = Span::new(FileId::from_raw(0), 0, 2);
//! let expr = Expr {
//!     kind: ExprKind::Literal(Lit::Bool(true)),
//!     span,
//! };
//! assert!(matches!(expr.kind, ExprKind::Literal(Lit::Bool(true))));
//! ```

#![deny(missing_docs)]

mod expr;
mod item;
mod stmt;

pub use expr::{
    BinOp, Expr, ExprKind, FieldInit, Lit, MatchArm, Pattern, PatternVariantKind, UnaryOp,
};
pub use item::{
    EnumItem, FieldDef, FnItem, Item, ItemKind, Module, Param, StructItem, VariantDef, VariantKind,
    Visibility,
};
pub use stmt::{Block, Stmt, StmtKind};
