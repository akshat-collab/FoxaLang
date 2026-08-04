//! Interpreter errors.

use thiserror::Error;

/// Runtime failure while interpreting.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InterpError {
    /// Program has no `main` function.
    #[error("no `main` function found")]
    NoMain,
    /// Unbound variable (should be caught by resolver).
    #[error("undefined variable `{0}`")]
    Undefined(String),
    /// Invalid operation on values.
    #[error("runtime type error: {0}")]
    TypeError(String),
    /// Explicit panic / failed assert.
    #[error("assertion failed")]
    AssertFailed,
    /// `return` outside a function (internal).
    #[error("internal: return outside function")]
    ReturnOutside,
    /// Argument count mismatch.
    #[error("wrong number of arguments: expected {expected}, got {got}")]
    Arity {
        /// Expected count.
        expected: usize,
        /// Actual count.
        got: usize,
    },
    /// Generic message.
    #[error("{0}")]
    Message(String),
}
