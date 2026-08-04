//! Expression AST nodes.

use foxa_span::Span;

/// A literal value in source.
#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    /// Integer as source text (parsed later during type checking).
    Int(String),
    /// Float as source text.
    Float(String),
    /// String contents (escapes not yet unescaped).
    Str(String),
    /// Character contents.
    Char(String),
    /// Boolean.
    Bool(bool),
}

/// Binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
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
    /// `&&`
    And,
    /// `||`
    Or,
    /// `=`
    Assign,
}

/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    /// `-`
    Neg,
    /// `!`
    Not,
    /// `&`
    Ref,
    /// `*`
    Deref,
}

/// Expression node.
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    /// Expression kind.
    pub kind: ExprKind,
    /// Source span.
    pub span: Span,
}

/// Field initializer in a struct literal.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldInit {
    /// Field name.
    pub name: String,
    /// Value expression.
    pub value: Expr,
    /// Source span.
    pub span: Span,
}

/// A `match` arm.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    /// Pattern.
    pub pattern: Pattern,
    /// Arm body.
    pub body: Expr,
    /// Source span.
    pub span: Span,
}

/// Pattern for `match` / future destructuring.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Wildcard `_`
    Wildcard,
    /// Binding `name`
    Ident(String),
    /// Literal pattern.
    Lit(Lit),
    /// Enum/path variant: `None`, `Some(x)`, `Point { x, y }`
    Variant {
        /// Path text (e.g. `Some` or `Option::Some`).
        path: String,
        /// Nested bindings / fields.
        kind: PatternVariantKind,
    },
}

/// Nested shape of a variant pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum PatternVariantKind {
    /// Unit: `None`
    Unit,
    /// Tuple: `Some(x)`
    Tuple(Vec<Pattern>),
    /// Struct: `Ok { value }`
    Struct(Vec<(String, Pattern)>),
}

/// Expression variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    /// Literal.
    Literal(Lit),
    /// Identifier reference.
    Path(String),
    /// Binary operation.
    Binary {
        /// Operator.
        op: BinOp,
        /// Left-hand side.
        lhs: Box<Expr>,
        /// Right-hand side.
        rhs: Box<Expr>,
    },
    /// Unary operation.
    Unary {
        /// Operator.
        op: UnaryOp,
        /// Operand.
        expr: Box<Expr>,
    },
    /// Function / method call.
    Call {
        /// Callee expression.
        callee: Box<Expr>,
        /// Arguments.
        args: Vec<Expr>,
    },
    /// Field access: `expr.field`
    Field {
        /// Base expression.
        base: Box<Expr>,
        /// Field name.
        field: String,
    },
    /// Struct / enum-struct literal: `Name { field: value, ... }`
    StructLit {
        /// Type / variant path.
        name: String,
        /// Field initializers.
        fields: Vec<FieldInit>,
    },
    /// Parenthesized group.
    Group(Box<Expr>),
    /// Block used as expression.
    Block(crate::Block),
    /// `if` expression.
    If {
        /// Condition.
        cond: Box<Expr>,
        /// Then branch.
        then_branch: crate::Block,
        /// Optional else branch.
        else_branch: Option<Box<Expr>>,
    },
    /// `match` expression.
    Match {
        /// Scrutinee.
        scrutinee: Box<Expr>,
        /// Arms.
        arms: Vec<MatchArm>,
    },
    /// Array / vec literal sugar: `[a, b, c]`
    Array(Vec<Expr>),
    /// Placeholder for incomplete parse recovery.
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxa_span::FileId;

    #[test]
    fn lit_bool() {
        let e = Expr {
            kind: ExprKind::Literal(Lit::Bool(true)),
            span: Span::new(FileId::from_raw(0), 0, 4),
        };
        assert!(matches!(e.kind, ExprKind::Literal(Lit::Bool(true))));
    }
}
