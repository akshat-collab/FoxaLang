//! Statement and block AST nodes.

use crate::Expr;
use foxa_span::Span;

/// A sequence of statements with an optional trailing expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    /// Statements inside the block.
    pub stmts: Vec<Stmt>,
    /// Optional trailing expression (block value).
    pub expr: Option<Box<Expr>>,
    /// Source span covering `{ ... }`.
    pub span: Span,
}

/// Statement node.
#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    /// Statement kind.
    pub kind: StmtKind,
    /// Source span.
    pub span: Span,
}

/// Statement variants.
#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    /// Local binding: `let mut? name = expr;`
    Let {
        /// Whether the binding is mutable.
        mutable: bool,
        /// Binding name.
        name: String,
        /// Optional type annotation text (parsed later).
        ty: Option<String>,
        /// Initializer.
        init: Option<Expr>,
    },
    /// `while cond { ... }`
    While {
        /// Loop condition.
        cond: Expr,
        /// Loop body.
        body: Block,
    },
    /// `for name in iter { ... }`
    For {
        /// Loop variable name.
        name: String,
        /// Iterable expression.
        iter: Expr,
        /// Loop body.
        body: Block,
    },
    /// Expression statement.
    Expr(Expr),
    /// `return expr?;`
    Return(Option<Expr>),
    /// `break;`
    Break,
    /// `continue;`
    Continue,
    /// Empty `;`
    Empty,
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxa_span::FileId;

    #[test]
    fn empty_block() {
        let b = Block {
            stmts: vec![],
            expr: None,
            span: Span::new(FileId::from_raw(0), 0, 2),
        };
        assert!(b.stmts.is_empty());
    }
}
