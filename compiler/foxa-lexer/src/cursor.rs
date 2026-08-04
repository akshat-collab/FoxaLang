//! Lexer cursor and scanning logic.

use crate::token::{keyword, LiteralKind, Token, TokenKind};
use foxa_diagnostics::{Diagnostic, DiagnosticBag};
use foxa_span::{FileId, Span};

/// Stateful lexical scanner over a single source file.
pub struct Lexer<'a> {
    file_id: FileId,
    source: &'a str,
    pos: usize,
    diagnostics: &'a mut DiagnosticBag,
}

impl<'a> Lexer<'a> {
    /// Creates a lexer for `source` belonging to `file_id`.
    pub fn new(file_id: FileId, source: &'a str, diagnostics: &'a mut DiagnosticBag) -> Self {
        Self {
            file_id,
            source,
            pos: 0,
            diagnostics,
        }
    }

    /// Lexes the entire input into a token vector (including a trailing EOF).
    pub fn tokenize_all(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok.is_eof();
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }

    /// Advances and returns the next token.
    pub fn next_token(&mut self) -> Token {
        self.skip_trivia();
        let start = self.pos as u32;

        let Some(ch) = self.peek_char() else {
            return Token::new(TokenKind::Eof, Span::at(self.file_id, start));
        };

        match ch {
            // Identifiers / keywords
            c if is_ident_start(c) => self.lex_ident(start),

            // Numbers
            '0'..='9' => self.lex_number(start),

            // Strings
            '"' => self.lex_string(start),

            // Chars
            '\'' => self.lex_char(start),

            // Punctuation / operators
            '(' => self.bump_simple(TokenKind::LParen, start),
            ')' => self.bump_simple(TokenKind::RParen, start),
            '{' => self.bump_simple(TokenKind::LBrace, start),
            '}' => self.bump_simple(TokenKind::RBrace, start),
            '[' => self.bump_simple(TokenKind::LBracket, start),
            ']' => self.bump_simple(TokenKind::RBracket, start),
            ',' => self.bump_simple(TokenKind::Comma, start),
            ';' => self.bump_simple(TokenKind::Semi, start),
            '@' => self.bump_simple(TokenKind::At, start),
            '#' => self.bump_simple(TokenKind::Pound, start),
            '?' => self.bump_simple(TokenKind::Question, start),
            '%' => self.bump_simple(TokenKind::Percent, start),
            '^' => self.bump_simple(TokenKind::Caret, start),

            '.' => self.bump_simple(TokenKind::Dot, start),

            ':' => {
                self.bump_char();
                if self.peek_char() == Some(':') {
                    self.bump_char();
                    Token::new(
                        TokenKind::ColonColon,
                        Span::new(self.file_id, start, self.pos as u32),
                    )
                } else {
                    Token::new(
                        TokenKind::Colon,
                        Span::new(self.file_id, start, self.pos as u32),
                    )
                }
            }

            '+' => self.lex_assign_op(TokenKind::Plus, TokenKind::PlusEq, start),
            '*' => self.lex_assign_op(TokenKind::Star, TokenKind::StarEq, start),
            '/' => self.lex_assign_op(TokenKind::Slash, TokenKind::SlashEq, start),

            '-' => {
                self.bump_char();
                match self.peek_char() {
                    Some('>') => {
                        self.bump_char();
                        Token::new(
                            TokenKind::Arrow,
                            Span::new(self.file_id, start, self.pos as u32),
                        )
                    }
                    Some('=') => {
                        self.bump_char();
                        Token::new(
                            TokenKind::MinusEq,
                            Span::new(self.file_id, start, self.pos as u32),
                        )
                    }
                    _ => Token::new(
                        TokenKind::Minus,
                        Span::new(self.file_id, start, self.pos as u32),
                    ),
                }
            }

            '=' => {
                self.bump_char();
                match self.peek_char() {
                    Some('=') => {
                        self.bump_char();
                        Token::new(
                            TokenKind::EqEq,
                            Span::new(self.file_id, start, self.pos as u32),
                        )
                    }
                    Some('>') => {
                        self.bump_char();
                        Token::new(
                            TokenKind::FatArrow,
                            Span::new(self.file_id, start, self.pos as u32),
                        )
                    }
                    _ => Token::new(
                        TokenKind::Eq,
                        Span::new(self.file_id, start, self.pos as u32),
                    ),
                }
            }

            '!' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    Token::new(
                        TokenKind::Ne,
                        Span::new(self.file_id, start, self.pos as u32),
                    )
                } else {
                    Token::new(
                        TokenKind::Not,
                        Span::new(self.file_id, start, self.pos as u32),
                    )
                }
            }

            '<' => {
                self.bump_char();
                match self.peek_char() {
                    Some('=') => {
                        self.bump_char();
                        Token::new(
                            TokenKind::Le,
                            Span::new(self.file_id, start, self.pos as u32),
                        )
                    }
                    Some('<') => {
                        self.bump_char();
                        Token::new(
                            TokenKind::Shl,
                            Span::new(self.file_id, start, self.pos as u32),
                        )
                    }
                    _ => Token::new(
                        TokenKind::Lt,
                        Span::new(self.file_id, start, self.pos as u32),
                    ),
                }
            }

            '>' => {
                self.bump_char();
                match self.peek_char() {
                    Some('=') => {
                        self.bump_char();
                        Token::new(
                            TokenKind::Ge,
                            Span::new(self.file_id, start, self.pos as u32),
                        )
                    }
                    Some('>') => {
                        self.bump_char();
                        Token::new(
                            TokenKind::Shr,
                            Span::new(self.file_id, start, self.pos as u32),
                        )
                    }
                    _ => Token::new(
                        TokenKind::Gt,
                        Span::new(self.file_id, start, self.pos as u32),
                    ),
                }
            }

            '&' => {
                self.bump_char();
                if self.peek_char() == Some('&') {
                    self.bump_char();
                    Token::new(
                        TokenKind::AndAnd,
                        Span::new(self.file_id, start, self.pos as u32),
                    )
                } else {
                    Token::new(
                        TokenKind::And,
                        Span::new(self.file_id, start, self.pos as u32),
                    )
                }
            }

            '|' => {
                self.bump_char();
                if self.peek_char() == Some('|') {
                    self.bump_char();
                    Token::new(
                        TokenKind::OrOr,
                        Span::new(self.file_id, start, self.pos as u32),
                    )
                } else {
                    Token::new(
                        TokenKind::Or,
                        Span::new(self.file_id, start, self.pos as u32),
                    )
                }
            }

            _ => {
                let bad = ch;
                self.bump_char();
                let span = Span::new(self.file_id, start, self.pos as u32);
                self.diagnostics.push(
                    Diagnostic::error(format!("unexpected character `{bad}`"))
                        .with_code("E0001")
                        .with_label(span, "invalid character here")
                        .with_help("remove this character or check for encoding issues"),
                );
                // Skip and continue — return a zero-width placeholder by
                // recursively fetching the next real token.
                self.next_token()
            }
        }
    }

    fn bump_simple(&mut self, kind: TokenKind, start: u32) -> Token {
        self.bump_char();
        Token::new(kind, Span::new(self.file_id, start, self.pos as u32))
    }

    fn lex_assign_op(&mut self, plain: TokenKind, assign: TokenKind, start: u32) -> Token {
        self.bump_char();
        if self.peek_char() == Some('=') {
            self.bump_char();
            Token::new(assign, Span::new(self.file_id, start, self.pos as u32))
        } else {
            Token::new(plain, Span::new(self.file_id, start, self.pos as u32))
        }
    }

    fn lex_ident(&mut self, start: u32) -> Token {
        while matches!(self.peek_char(), Some(c) if is_ident_continue(c)) {
            self.bump_char();
        }
        let span = Span::new(self.file_id, start, self.pos as u32);
        let text = &self.source[start as usize..self.pos];
        let kind = keyword(text).unwrap_or(TokenKind::Ident);
        Token::new(kind, span)
    }

    fn lex_number(&mut self, start: u32) -> Token {
        // Hex / binary / octal
        if self.peek_char() == Some('0') {
            let next = self.peek_char_at(1);
            if matches!(next, Some('x' | 'X' | 'b' | 'B' | 'o' | 'O')) {
                self.bump_char(); // 0
                self.bump_char(); // x/b/o
                let mut digits = 0;
                while matches!(self.peek_char(), Some(c) if c.is_ascii_hexdigit() || c == '_') {
                    if self.peek_char() != Some('_') {
                        digits += 1;
                    }
                    self.bump_char();
                }
                let span = Span::new(self.file_id, start, self.pos as u32);
                if digits == 0 {
                    self.diagnostics.push(
                        Diagnostic::error("numeric literal has no digits")
                            .with_code("E0002")
                            .with_label(span, "here"),
                    );
                }
                return Token::new(TokenKind::Literal(LiteralKind::Integer), span);
            }
        }

        // Decimal integer / float
        while matches!(self.peek_char(), Some(c) if c.is_ascii_digit() || c == '_') {
            self.bump_char();
        }

        let mut is_float = false;
        if self.peek_char() == Some('.')
            && matches!(self.peek_char_at(1), Some(c) if c.is_ascii_digit())
        {
            is_float = true;
            self.bump_char(); // .
            while matches!(self.peek_char(), Some(c) if c.is_ascii_digit() || c == '_') {
                self.bump_char();
            }
        }

        // Exponent
        if matches!(self.peek_char(), Some('e' | 'E')) {
            is_float = true;
            self.bump_char();
            if matches!(self.peek_char(), Some('+' | '-')) {
                self.bump_char();
            }
            let mut digits = 0;
            while matches!(self.peek_char(), Some(c) if c.is_ascii_digit() || c == '_') {
                if self.peek_char() != Some('_') {
                    digits += 1;
                }
                self.bump_char();
            }
            if digits == 0 {
                let span = Span::new(self.file_id, start, self.pos as u32);
                self.diagnostics.push(
                    Diagnostic::error("exponent has no digits")
                        .with_code("E0003")
                        .with_label(span, "here"),
                );
            }
        }

        let span = Span::new(self.file_id, start, self.pos as u32);
        let kind = if is_float {
            TokenKind::Literal(LiteralKind::Float)
        } else {
            TokenKind::Literal(LiteralKind::Integer)
        };
        Token::new(kind, span)
    }

    fn lex_string(&mut self, start: u32) -> Token {
        self.bump_char(); // opening "
        let mut closed = false;
        while let Some(ch) = self.peek_char() {
            match ch {
                '"' => {
                    self.bump_char();
                    closed = true;
                    break;
                }
                '\\' => {
                    self.bump_char();
                    if self.peek_char().is_some() {
                        self.bump_char();
                    }
                }
                '\n' => break,
                _ => {
                    self.bump_char();
                }
            }
        }
        let span = Span::new(self.file_id, start, self.pos as u32);
        if !closed {
            self.diagnostics.push(
                Diagnostic::error("unterminated string literal")
                    .with_code("E0004")
                    .with_label(span, "string starts here")
                    .with_help("add a closing `\"`"),
            );
        }
        Token::new(TokenKind::Literal(LiteralKind::String), span)
    }

    fn lex_char(&mut self, start: u32) -> Token {
        self.bump_char(); // opening '
        let mut closed = false;
        if self.peek_char() == Some('\\') {
            self.bump_char();
            if self.peek_char().is_some() {
                self.bump_char();
            }
        } else if self.peek_char().is_some() {
            self.bump_char();
        }
        if self.peek_char() == Some('\'') {
            self.bump_char();
            closed = true;
        }
        let span = Span::new(self.file_id, start, self.pos as u32);
        if !closed {
            self.diagnostics.push(
                Diagnostic::error("unterminated character literal")
                    .with_code("E0005")
                    .with_label(span, "here")
                    .with_help("character literals look like `'x'`"),
            );
        }
        Token::new(TokenKind::Literal(LiteralKind::Char), span)
    }

    fn skip_trivia(&mut self) {
        loop {
            match self.peek_char() {
                Some(c) if c.is_whitespace() => {
                    self.bump_char();
                }
                Some('/') if self.peek_char_at(1) == Some('/') => {
                    // line comment
                    while let Some(c) = self.peek_char() {
                        self.bump_char();
                        if c == '\n' {
                            break;
                        }
                    }
                }
                Some('/') if self.peek_char_at(1) == Some('*') => {
                    self.bump_char();
                    self.bump_char();
                    let start = (self.pos as u32).saturating_sub(2);
                    let mut closed = false;
                    while let Some(c) = self.peek_char() {
                        if c == '*' && self.peek_char_at(1) == Some('/') {
                            self.bump_char();
                            self.bump_char();
                            closed = true;
                            break;
                        }
                        self.bump_char();
                    }
                    if !closed {
                        let span = Span::new(self.file_id, start, self.pos as u32);
                        self.diagnostics.push(
                            Diagnostic::error("unterminated block comment")
                                .with_code("E0006")
                                .with_label(span, "comment starts here")
                                .with_help("add `*/` to close the comment"),
                        );
                    }
                }
                _ => break,
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_char_at(&self, char_offset: usize) -> Option<char> {
        self.source[self.pos..].chars().nth(char_offset)
    }

    fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_ascii_alphabetic() || (!c.is_ascii() && c.is_alphabetic())
}

fn is_ident_continue(c: char) -> bool {
    c == '_' || c.is_ascii_alphanumeric() || (!c.is_ascii() && c.is_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxa_span::SourceMap;

    fn lex(src: &str) -> (Vec<Token>, DiagnosticBag) {
        let mut map = SourceMap::new();
        let id = map.add_file("t.foxa", src);
        let mut bag = DiagnosticBag::new();
        let tokens = Lexer::new(id, src, &mut bag).tokenize_all();
        (tokens, bag)
    }

    fn kinds(src: &str) -> Vec<TokenKind> {
        let (tokens, bag) = lex(src);
        assert!(!bag.has_errors(), "unexpected errors for {src:?}: {bag:?}");
        tokens.into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn empty_input() {
        assert_eq!(kinds(""), vec![TokenKind::Eof]);
    }

    #[test]
    fn function_skeleton() {
        assert_eq!(
            kinds("fn main() {}"),
            vec![
                TokenKind::Fn,
                TokenKind::Ident,
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn keywords_and_idents() {
        let k = kinds("let mut x = true;");
        assert_eq!(
            k,
            vec![
                TokenKind::Let,
                TokenKind::Mut,
                TokenKind::Ident,
                TokenKind::Eq,
                TokenKind::True,
                TokenKind::Semi,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn operators() {
        assert_eq!(
            kinds("a == b != c <= d >= e && f || g -> h => i"),
            vec![
                TokenKind::Ident,
                TokenKind::EqEq,
                TokenKind::Ident,
                TokenKind::Ne,
                TokenKind::Ident,
                TokenKind::Le,
                TokenKind::Ident,
                TokenKind::Ge,
                TokenKind::Ident,
                TokenKind::AndAnd,
                TokenKind::Ident,
                TokenKind::OrOr,
                TokenKind::Ident,
                TokenKind::Arrow,
                TokenKind::Ident,
                TokenKind::FatArrow,
                TokenKind::Ident,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn numbers() {
        let k = kinds("42 0xFF 0b101 3.14 1e10 1.5e-2");
        assert_eq!(
            k,
            vec![
                TokenKind::Literal(LiteralKind::Integer),
                TokenKind::Literal(LiteralKind::Integer),
                TokenKind::Literal(LiteralKind::Integer),
                TokenKind::Literal(LiteralKind::Float),
                TokenKind::Literal(LiteralKind::Float),
                TokenKind::Literal(LiteralKind::Float),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn strings_and_chars() {
        let k = kinds(r#""hello" 'x' '\n'"#);
        assert_eq!(
            k,
            vec![
                TokenKind::Literal(LiteralKind::String),
                TokenKind::Literal(LiteralKind::Char),
                TokenKind::Literal(LiteralKind::Char),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn comments_skipped() {
        assert_eq!(
            kinds("a // comment\nb /* block */ c"),
            vec![
                TokenKind::Ident,
                TokenKind::Ident,
                TokenKind::Ident,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn unterminated_string_errors() {
        let (_tokens, bag) = lex("\"oops");
        assert!(bag.has_errors());
        assert!(bag.items()[0].code.as_deref() == Some("E0004"));
    }

    #[test]
    fn unexpected_character_errors() {
        let (_tokens, bag) = lex("let $ x");
        assert!(bag.has_errors());
        assert!(bag.items()[0].code.as_deref() == Some("E0001"));
    }

    #[test]
    fn spans_cover_lexemes() {
        let (tokens, bag) = lex("fn");
        assert!(!bag.has_errors());
        assert_eq!(tokens[0].span.len(), 2);
    }
}
