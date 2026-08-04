//! MIR data structures.

use std::fmt;

/// Module-level MIR.
#[derive(Debug, Clone, Default)]
pub struct MirModule {
    /// Functions in the module.
    pub functions: Vec<MirFunction>,
}

/// A lowered function.
#[derive(Debug, Clone)]
pub struct MirFunction {
    /// Function name.
    pub name: String,
    /// Parameter locals (first N locals).
    pub params: Vec<LocalId>,
    /// All locals including params.
    pub locals: Vec<MirTy>,
    /// Basic blocks (entry is block 0).
    pub blocks: Vec<BasicBlock>,
    /// Return type.
    pub return_ty: MirTy,
}

/// Local variable identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

/// Basic block identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

/// MIR types (subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirTy {
    /// Unit.
    Unit,
    /// Bool.
    Bool,
    /// Signed 64-bit int.
    Int,
    /// Opaque / unsupported in codegen yet.
    Opaque,
}

/// A basic block.
#[derive(Debug, Clone, Default)]
pub struct BasicBlock {
    /// Straight-line statements.
    pub stmts: Vec<MirStmt>,
    /// Block exit.
    pub term: MirTerminator,
}

/// MIR statement.
#[derive(Debug, Clone)]
pub enum MirStmt {
    /// `local = rvalue`
    Assign {
        /// Destination.
        local: LocalId,
        /// Source.
        rvalue: MirRvalue,
    },
}

/// Right-hand side of an assignment.
#[derive(Debug, Clone)]
pub enum MirRvalue {
    /// Use a local.
    Use(LocalId),
    /// Integer constant.
    ConstInt(i64),
    /// Boolean constant.
    ConstBool(bool),
    /// Binary operation.
    Binary {
        /// Operator.
        op: MirBinOp,
        /// Left.
        lhs: LocalId,
        /// Right.
        rhs: LocalId,
    },
    /// Call a function; result in this rvalue.
    Call {
        /// Callee name.
        callee: String,
        /// Arguments.
        args: Vec<LocalId>,
    },
}

/// Binary operators in MIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirBinOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Rem,
    /// `==`
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
}

/// Block terminator.
#[derive(Debug, Clone, Default)]
pub enum MirTerminator {
    /// Return a local (or unit if None).
    Return(Option<LocalId>),
    /// Unconditional jump.
    Goto(BlockId),
    /// Conditional branch.
    Switch {
        /// Condition local (Bool).
        cond: LocalId,
        /// Taken when true.
        true_block: BlockId,
        /// Taken when false.
        false_block: BlockId,
    },
    /// Unreachable / incomplete.
    #[default]
    Unreachable,
}

impl fmt::Display for MirTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "Unit"),
            Self::Bool => write!(f, "Bool"),
            Self::Int => write!(f, "Int"),
            Self::Opaque => write!(f, "Opaque"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_ids() {
        assert_eq!(LocalId(0), LocalId(0));
    }
}
