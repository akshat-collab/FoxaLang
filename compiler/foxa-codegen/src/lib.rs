//! Cranelift codegen backend for Foxa MIR.
//!
//! # Purpose
//!
//! Compiles Int/Bool MIR functions to native machine code via Cranelift JIT.
//! This is the path toward AOT object emission and LLVM parity.
//!
//! # Architecture
//!
//! - [`JitEngine`] — owns a Cranelift JIT module
//! - [`compile_module`] — lowers each MIR function and defines it in the JIT
//! - Host helpers: `foxa_print_i64` for `print` of integers

#![deny(missing_docs)]

mod jit;

pub use jit::{compile_module, CodegenError, JitEngine};
