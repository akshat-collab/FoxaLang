//! Token kinds and payloads.

use foxa_span::Span;
use std::fmt;

/// Classification of a lexical token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Identifiers & keywords
    /// User identifier or keyword-as-identifier after `@`.
    Ident,
    /// `fn`
    Fn,
    /// `let`
    Let,
    /// `mut`
    Mut,
    /// `const`
    Const,
    /// `struct`
    Struct,
    /// `enum`
    Enum,
    /// `impl`
    Impl,
    /// `trait`
    Trait,
    /// `if`
    If,
    /// `else`
    Else,
    /// `while`
    While,
    /// `for`
    For,
    /// `loop`
    Loop,
    /// `match`
    Match,
    /// `return`
    Return,
    /// `break`
    Break,
    /// `continue`
    Continue,
    /// `true`
    True,
    /// `false`
    False,
    /// `pub`
    Pub,
    /// `use`
    Use,
    /// `mod`
    Mod,
    /// `as`
    As,
    /// `in`
    In,
    /// `where`
    Where,
    /// `type`
    Type,
    /// `self` (value)
    SelfValue,
    /// `Self` (type)
    SelfType,
    /// `async`
    Async,
    /// `await`
    Await,
    /// `unsafe`
    Unsafe,

    // Literals
    /// Integer, float, string, char, or bool literal.
    Literal(LiteralKind),

    // Punctuation
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `:`
    Colon,
    /// `::`
    ColonColon,
    /// `;`
    Semi,
    /// `->`
    Arrow,
    /// `=>`
    FatArrow,
    /// `@`
    At,
    /// `#`
    Pound,
    /// `?`
    Question,

    // Operators
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `=`
    Eq,
    /// `==`
    EqEq,
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
    AndAnd,
    /// `||`
    OrOr,
    /// `!`
    Not,
    /// `&`
    And,
    /// `|`
    Or,
    /// `^`
    Caret,
    /// `<<`
    Shl,
    /// `>>`
    Shr,
    /// `+=`
    PlusEq,
    /// `-=`
    MinusEq,
    /// `*=`
    StarEq,
    /// `/=`
    SlashEq,

    /// End of file.
    Eof,
}

/// Sub-kind of a literal token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LiteralKind {
    /// Integer literal (decimal, hex, binary, octal).
    Integer,
    /// Floating-point literal.
    Float,
    /// String literal `"..."`.
    String,
    /// Character literal `'x'`.
    Char,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Ident => "identifier",
            Self::Fn => "`fn`",
            Self::Let => "`let`",
            Self::Mut => "`mut`",
            Self::Const => "`const`",
            Self::Struct => "`struct`",
            Self::Enum => "`enum`",
            Self::Impl => "`impl`",
            Self::Trait => "`trait`",
            Self::If => "`if`",
            Self::Else => "`else`",
            Self::While => "`while`",
            Self::For => "`for`",
            Self::Loop => "`loop`",
            Self::Match => "`match`",
            Self::Return => "`return`",
            Self::Break => "`break`",
            Self::Continue => "`continue`",
            Self::True => "`true`",
            Self::False => "`false`",
            Self::Pub => "`pub`",
            Self::Use => "`use`",
            Self::Mod => "`mod`",
            Self::As => "`as`",
            Self::In => "`in`",
            Self::Where => "`where`",
            Self::Type => "`type`",
            Self::SelfValue => "`self`",
            Self::SelfType => "`Self`",
            Self::Async => "`async`",
            Self::Await => "`await`",
            Self::Unsafe => "`unsafe`",
            Self::Literal(LiteralKind::Integer) => "integer literal",
            Self::Literal(LiteralKind::Float) => "float literal",
            Self::Literal(LiteralKind::String) => "string literal",
            Self::Literal(LiteralKind::Char) => "char literal",
            Self::LParen => "`(`",
            Self::RParen => "`)`",
            Self::LBrace => "`{`",
            Self::RBrace => "`}`",
            Self::LBracket => "`[`",
            Self::RBracket => "`]`",
            Self::Comma => "`,`",
            Self::Dot => "`.`",
            Self::Colon => "`:`",
            Self::ColonColon => "`::`",
            Self::Semi => "`;`",
            Self::Arrow => "`->`",
            Self::FatArrow => "`=>`",
            Self::At => "`@`",
            Self::Pound => "`#`",
            Self::Question => "`?`",
            Self::Plus => "`+`",
            Self::Minus => "`-`",
            Self::Star => "`*`",
            Self::Slash => "`/`",
            Self::Percent => "`%`",
            Self::Eq => "`=`",
            Self::EqEq => "`==`",
            Self::Ne => "`!=`",
            Self::Lt => "`<`",
            Self::Le => "`<=`",
            Self::Gt => "`>`",
            Self::Ge => "`>=`",
            Self::AndAnd => "`&&`",
            Self::OrOr => "`||`",
            Self::Not => "`!`",
            Self::And => "`&`",
            Self::Or => "`|`",
            Self::Caret => "`^`",
            Self::Shl => "`<<`",
            Self::Shr => "`>>`",
            Self::PlusEq => "`+=`",
            Self::MinusEq => "`-=`",
            Self::StarEq => "`*=`",
            Self::SlashEq => "`/=`",
            Self::Eof => "end of file",
        };
        write!(f, "{s}")
    }
}

/// A single lexed token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Token classification.
    pub kind: TokenKind,
    /// Source span covering the token lexeme.
    pub span: Span,
}

impl Token {
    /// Creates a token.
    #[must_use]
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Returns `true` if this is EOF.
    #[must_use]
    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }
}

/// Maps an identifier string to a keyword token, if any.
#[must_use]
pub fn keyword(ident: &str) -> Option<TokenKind> {
    Some(match ident {
        "fn" => TokenKind::Fn,
        "let" => TokenKind::Let,
        "mut" => TokenKind::Mut,
        "const" => TokenKind::Const,
        "struct" => TokenKind::Struct,
        "enum" => TokenKind::Enum,
        "impl" => TokenKind::Impl,
        "trait" => TokenKind::Trait,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "for" => TokenKind::For,
        "loop" => TokenKind::Loop,
        "match" => TokenKind::Match,
        "return" => TokenKind::Return,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "pub" => TokenKind::Pub,
        "use" => TokenKind::Use,
        "mod" => TokenKind::Mod,
        "as" => TokenKind::As,
        "in" => TokenKind::In,
        "where" => TokenKind::Where,
        "type" => TokenKind::Type,
        "self" => TokenKind::SelfValue,
        "Self" => TokenKind::SelfType,
        "async" => TokenKind::Async,
        "await" => TokenKind::Await,
        "unsafe" => TokenKind::Unsafe,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_recognized() {
        assert_eq!(keyword("fn"), Some(TokenKind::Fn));
        assert_eq!(keyword("Self"), Some(TokenKind::SelfType));
        assert_eq!(keyword("foo"), None);
    }
}
