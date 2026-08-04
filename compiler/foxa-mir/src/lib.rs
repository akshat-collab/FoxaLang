//! Mid-level IR for Foxa.
//!
//! # Purpose
//!
//! MIR is an SSA-friendly representation between the typed AST and backends
//! (interpreter reference, Cranelift, LLVM). Each function is a graph of
//! basic blocks with typed locals.
//!
//! # Architecture
//!
//! - [`MirModule`] — collection of functions
//! - [`MirFunction`] — parameters, locals, blocks
//! - [`BasicBlock`] — statements + terminator
//! - [`lower_module`] — AST → MIR lowering

#![deny(missing_docs)]

mod ir;
mod lower;

pub use ir::{
    BasicBlock, BlockId, LocalId, MirBinOp, MirFunction, MirModule, MirRvalue, MirStmt,
    MirTerminator, MirTy,
};
pub use lower::lower_module;
