//! Recursive-descent + Pratt parser implementation.

use foxa_ast::{
    BinOp, Block, EnumItem, Expr, ExprKind, FieldDef, FieldInit, FnItem, Item, ItemKind, Lit,
    MatchArm, Module, Param, Pattern, PatternVariantKind, Stmt, StmtKind, StructItem, UnaryOp,
    VariantDef, VariantKind, Visibility,
};
use foxa_diagnostics::{Diagnostic, DiagnosticBag};
use foxa_lexer::{LiteralKind, Token, TokenKind};
use foxa_span::{FileId, Span};

/// Foxa parser over a pre-lexed token stream.
pub struct Parser<'a> {
    file_id: FileId,
    source: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: &'a mut DiagnosticBag,
}

impl<'a> Parser<'a> {
    /// Creates a parser from tokens produced by the lexer.
    #[must_use]
    pub fn new(
        file_id: FileId,
        source: &'a str,
        tokens: Vec<Token>,
        diagnostics: &'a mut DiagnosticBag,
    ) -> Self {
        Self {
            file_id,
            source,
            tokens,
            pos: 0,
            diagnostics,
        }
    }

    /// Parses an entire module (source file).
    pub fn parse_module(mut self) -> Module {
        let start = self.peek_span();
        let mut items = Vec::new();
        while !self.is_eof() {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                self.bump();
            }
        }
        let end = self.prev_span();
        Module {
            items,
            span: start.merge(end),
        }
    }

    fn parse_item(&mut self) -> Option<Item> {
        let vis = if self.at(TokenKind::Pub) {
            self.bump();
            Visibility::Public
        } else {
            Visibility::Private
        };

        if self.at(TokenKind::Fn) {
            let start = self.peek_span();
            let func = self.parse_fn()?;
            let span = start.merge(func.body.span);
            return Some(Item {
                kind: ItemKind::Fn(func),
                vis,
                span,
            });
        }

        if self.at(TokenKind::Struct) {
            let item = self.parse_struct()?;
            let span = item.span;
            return Some(Item {
                kind: ItemKind::Struct(item),
                vis,
                span,
            });
        }

        if self.at(TokenKind::Enum) {
            let item = self.parse_enum()?;
            let span = item.span;
            return Some(Item {
                kind: ItemKind::Enum(item),
                vis,
                span,
            });
        }

        let span = self.peek_span();
        self.diagnostics.push(
            Diagnostic::error("expected item")
                .with_code("E0100")
                .with_label(span, "here")
                .with_help("items start with `fn`, `struct`, `enum`, `use`, or `mod`"),
        );
        None
    }

    fn parse_struct(&mut self) -> Option<StructItem> {
        let start = self.peek_span();
        self.expect(TokenKind::Struct)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.is_eof() {
            fields.push(self.parse_field_def()?);
            if self.at(TokenKind::Comma) {
                self.bump();
            } else {
                break;
            }
        }
        let end = self.peek_span();
        self.expect(TokenKind::RBrace)?;
        Some(StructItem {
            name,
            fields,
            span: start.merge(end),
        })
    }

    fn parse_field_def(&mut self) -> Option<FieldDef> {
        let start = self.peek_span();
        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type_text()?;
        let end = self.prev_span();
        Some(FieldDef {
            name,
            ty,
            span: start.merge(end),
        })
    }

    fn parse_enum(&mut self) -> Option<EnumItem> {
        let start = self.peek_span();
        self.expect(TokenKind::Enum)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LBrace)?;
        let mut variants = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.is_eof() {
            variants.push(self.parse_variant()?);
            if self.at(TokenKind::Comma) {
                self.bump();
            } else {
                break;
            }
        }
        let end = self.peek_span();
        self.expect(TokenKind::RBrace)?;
        Some(EnumItem {
            name,
            variants,
            span: start.merge(end),
        })
    }

    fn parse_variant(&mut self) -> Option<VariantDef> {
        let start = self.peek_span();
        let name = self.expect_ident()?;
        let kind = if self.at(TokenKind::LParen) {
            self.bump();
            let mut tys = Vec::new();
            if !self.at(TokenKind::RParen) {
                loop {
                    tys.push(self.parse_type_text()?);
                    if self.at(TokenKind::Comma) {
                        self.bump();
                        if self.at(TokenKind::RParen) {
                            break;
                        }
                        continue;
                    }
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;
            VariantKind::Tuple(tys)
        } else if self.at(TokenKind::LBrace) {
            self.bump();
            let mut fields = Vec::new();
            while !self.at(TokenKind::RBrace) && !self.is_eof() {
                fields.push(self.parse_field_def()?);
                if self.at(TokenKind::Comma) {
                    self.bump();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RBrace)?;
            VariantKind::Struct(fields)
        } else {
            VariantKind::Unit
        };
        let end = self.prev_span();
        Some(VariantDef {
            name,
            kind,
            span: start.merge(end),
        })
    }

    fn parse_fn(&mut self) -> Option<FnItem> {
        self.expect(TokenKind::Fn)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.at(TokenKind::RParen) {
            loop {
                params.push(self.parse_param()?);
                if self.at(TokenKind::Comma) {
                    self.bump();
                    if self.at(TokenKind::RParen) {
                        break;
                    }
                    continue;
                }
                break;
            }
        }
        self.expect(TokenKind::RParen)?;

        let return_ty = if self.at(TokenKind::Arrow) {
            self.bump();
            Some(self.parse_type_text()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        Some(FnItem {
            name,
            params,
            return_ty,
            body,
        })
    }

    fn parse_param(&mut self) -> Option<Param> {
        let start = self.peek_span();
        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type_text()?;
        let end = self.prev_span();
        Some(Param {
            name,
            ty,
            span: start.merge(end),
        })
    }

    fn parse_type_text(&mut self) -> Option<String> {
        if self.at(TokenKind::LBracket) {
            self.bump();
            let inner = self.parse_type_text()?;
            self.expect(TokenKind::RBracket)?;
            return Some(format!("[{inner}]"));
        }
        if !self.at(TokenKind::Ident)
            && !self.at(TokenKind::SelfType)
            && !matches!(self.peek_kind(), TokenKind::LParen)
        {
            let span = self.peek_span();
            self.diagnostics.push(
                Diagnostic::error("expected type")
                    .with_code("E0101")
                    .with_label(span, "here"),
            );
            return None;
        }
        let first = self.lexeme().to_string();
        self.bump();
        let mut text = first;
        while self.at(TokenKind::ColonColon) {
            self.bump();
            let part = self.expect_ident()?;
            text.push_str("::");
            text.push_str(&part);
        }
        // Generic args: Option[T] / Result[T, E] / Vec[T]
        if self.at(TokenKind::LBracket) {
            self.bump();
            let mut args = Vec::new();
            if !self.at(TokenKind::RBracket) {
                loop {
                    args.push(self.parse_type_text()?);
                    if self.at(TokenKind::Comma) {
                        self.bump();
                        if self.at(TokenKind::RBracket) {
                            break;
                        }
                        continue;
                    }
                    break;
                }
            }
            self.expect(TokenKind::RBracket)?;
            text.push('[');
            text.push_str(&args.join(", "));
            text.push(']');
        }
        Some(text)
    }

    fn parse_block(&mut self) -> Option<Block> {
        let start = self.peek_span();
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        let mut trailing = None;

        while !self.at(TokenKind::RBrace) && !self.is_eof() {
            if self.is_stmt_keyword() {
                if let Some(stmt) = self.parse_stmt() {
                    stmts.push(stmt);
                } else {
                    self.bump();
                }
                continue;
            }

            if self.starts_expr() {
                let checkpoint = self.pos;
                if let Some(expr) = self.parse_expr(0) {
                    if self.at(TokenKind::Semi) {
                        let end = self.peek_span();
                        self.bump();
                        stmts.push(Stmt {
                            kind: StmtKind::Expr(expr.clone()),
                            span: expr.span.merge(end),
                        });
                        continue;
                    } else if self.at(TokenKind::RBrace) {
                        trailing = Some(Box::new(expr));
                        break;
                    } else if is_block_tailed(&expr) {
                        let span = expr.span;
                        stmts.push(Stmt {
                            kind: StmtKind::Expr(expr),
                            span,
                        });
                        continue;
                    } else {
                        let span = expr.span;
                        self.diagnostics.push(
                            Diagnostic::error("expected `;` after expression")
                                .with_code("E0102")
                                .with_label(span, "expression here")
                                .with_help("add `;` or use this as the block's trailing value"),
                        );
                        stmts.push(Stmt {
                            kind: StmtKind::Expr(expr),
                            span,
                        });
                        continue;
                    }
                } else {
                    self.pos = checkpoint;
                    self.bump();
                    continue;
                }
            }

            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                self.bump();
            }
        }

        let end = self.peek_span();
        self.expect(TokenKind::RBrace)?;
        Some(Block {
            stmts,
            expr: trailing,
            span: start.merge(end),
        })
    }

    fn is_stmt_keyword(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::Let
                | TokenKind::Return
                | TokenKind::While
                | TokenKind::For
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Semi
        )
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        let start = self.peek_span();

        if self.at(TokenKind::Semi) {
            let span = self.bump();
            return Some(Stmt {
                kind: StmtKind::Empty,
                span,
            });
        }

        if self.at(TokenKind::Break) {
            self.bump();
            let end = self.peek_span();
            self.expect(TokenKind::Semi)?;
            return Some(Stmt {
                kind: StmtKind::Break,
                span: start.merge(end),
            });
        }

        if self.at(TokenKind::Continue) {
            self.bump();
            let end = self.peek_span();
            self.expect(TokenKind::Semi)?;
            return Some(Stmt {
                kind: StmtKind::Continue,
                span: start.merge(end),
            });
        }

        if self.at(TokenKind::While) {
            self.bump();
            let cond = self.parse_expr(0)?;
            let body = self.parse_block()?;
            let span = start.merge(body.span);
            return Some(Stmt {
                kind: StmtKind::While { cond, body },
                span,
            });
        }

        if self.at(TokenKind::For) {
            self.bump();
            let name = self.expect_ident()?;
            self.expect(TokenKind::In)?;
            let iter = self.parse_expr(0)?;
            let body = self.parse_block()?;
            let span = start.merge(body.span);
            return Some(Stmt {
                kind: StmtKind::For { name, iter, body },
                span,
            });
        }

        if self.at(TokenKind::Let) {
            self.bump();
            let mutable = if self.at(TokenKind::Mut) {
                self.bump();
                true
            } else {
                false
            };
            let name = self.expect_ident()?;
            let ty = if self.at(TokenKind::Colon) {
                self.bump();
                Some(self.parse_type_text()?)
            } else {
                None
            };
            let init = if self.at(TokenKind::Eq) {
                self.bump();
                Some(self.parse_expr(0)?)
            } else {
                None
            };
            let end = self.peek_span();
            self.expect(TokenKind::Semi)?;
            return Some(Stmt {
                kind: StmtKind::Let {
                    mutable,
                    name,
                    ty,
                    init,
                },
                span: start.merge(end),
            });
        }

        if self.at(TokenKind::Return) {
            self.bump();
            let value = if self.at(TokenKind::Semi) {
                None
            } else {
                Some(self.parse_expr(0)?)
            };
            let end = self.peek_span();
            self.expect(TokenKind::Semi)?;
            return Some(Stmt {
                kind: StmtKind::Return(value),
                span: start.merge(end),
            });
        }

        let expr = self.parse_expr(0)?;
        let end = self.peek_span();
        self.expect(TokenKind::Semi)?;
        Some(Stmt {
            kind: StmtKind::Expr(expr),
            span: start.merge(end),
        })
    }

    fn starts_expr(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::Ident
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Literal(_)
                | TokenKind::LParen
                | TokenKind::LBrace
                | TokenKind::LBracket
                | TokenKind::Minus
                | TokenKind::Not
                | TokenKind::And
                | TokenKind::Star
                | TokenKind::If
                | TokenKind::Match
                | TokenKind::SelfValue
        )
    }

    /// Pratt parser for expressions.
    pub fn parse_expr(&mut self, min_bp: u8) -> Option<Expr> {
        let mut lhs = self.parse_prefix()?;

        loop {
            let op_kind = self.peek_kind();
            if let Some((l_bp, r_bp)) = infix_binding_power(op_kind) {
                if l_bp < min_bp {
                    break;
                }
                let op = token_to_binop(op_kind)?;
                self.bump();
                let rhs = self.parse_expr(r_bp)?;
                let span = lhs.span.merge(rhs.span);
                lhs = Expr {
                    kind: ExprKind::Binary {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    span,
                };
                continue;
            }

            // Call
            if self.at(TokenKind::LParen) {
                if 90 < min_bp {
                    break;
                }
                lhs = self.parse_call(lhs)?;
                continue;
            }

            // Field access
            if self.at(TokenKind::Dot) {
                if 95 < min_bp {
                    break;
                }
                self.bump();
                let field = self.expect_ident()?;
                let end = self.prev_span();
                let span = lhs.span.merge(end);
                lhs = Expr {
                    kind: ExprKind::Field {
                        base: Box::new(lhs),
                        field,
                    },
                    span,
                };
                continue;
            }

            break;
        }

        Some(lhs)
    }

    fn parse_prefix(&mut self) -> Option<Expr> {
        let start = self.peek_span();
        match self.peek_kind() {
            TokenKind::Literal(kind) => {
                let text = self.lexeme().to_string();
                let span = self.bump();
                let lit = match kind {
                    LiteralKind::Integer => Lit::Int(text),
                    LiteralKind::Float => Lit::Float(text),
                    LiteralKind::String => Lit::Str(strip_quotes(&text)),
                    LiteralKind::Char => Lit::Char(strip_char_quotes(&text)),
                };
                Some(Expr {
                    kind: ExprKind::Literal(lit),
                    span,
                })
            }
            TokenKind::True => {
                let span = self.bump();
                Some(Expr {
                    kind: ExprKind::Literal(Lit::Bool(true)),
                    span,
                })
            }
            TokenKind::False => {
                let span = self.bump();
                Some(Expr {
                    kind: ExprKind::Literal(Lit::Bool(false)),
                    span,
                })
            }
            TokenKind::Ident | TokenKind::SelfValue => {
                let name = self.lexeme().to_string();
                let name_span = self.bump();
                // Struct literal: Ident { field: expr, ... }
                // Disambiguate from block by requiring `ident:` after `{`
                if self.at(TokenKind::LBrace) && self.looks_like_struct_lit() {
                    return self.parse_struct_lit(name, start);
                }
                Some(Expr {
                    kind: ExprKind::Path(name),
                    span: name_span,
                })
            }
            TokenKind::LParen => {
                self.bump();
                let inner = self.parse_expr(0)?;
                let end = self.peek_span();
                self.expect(TokenKind::RParen)?;
                Some(Expr {
                    kind: ExprKind::Group(Box::new(inner)),
                    span: start.merge(end),
                })
            }
            TokenKind::LBracket => {
                self.bump();
                let mut elems = Vec::new();
                if !self.at(TokenKind::RBracket) {
                    loop {
                        elems.push(self.parse_expr(0)?);
                        if self.at(TokenKind::Comma) {
                            self.bump();
                            if self.at(TokenKind::RBracket) {
                                break;
                            }
                            continue;
                        }
                        break;
                    }
                }
                let end = self.peek_span();
                self.expect(TokenKind::RBracket)?;
                Some(Expr {
                    kind: ExprKind::Array(elems),
                    span: start.merge(end),
                })
            }
            TokenKind::LBrace => {
                let block = self.parse_block()?;
                let span = block.span;
                Some(Expr {
                    kind: ExprKind::Block(block),
                    span,
                })
            }
            TokenKind::If => self.parse_if(),
            TokenKind::Match => self.parse_match(),
            TokenKind::Minus => {
                self.bump();
                let expr = self.parse_expr(prefix_binding_power())?;
                let span = start.merge(expr.span);
                Some(Expr {
                    kind: ExprKind::Unary {
                        op: UnaryOp::Neg,
                        expr: Box::new(expr),
                    },
                    span,
                })
            }
            TokenKind::Not => {
                self.bump();
                let expr = self.parse_expr(prefix_binding_power())?;
                let span = start.merge(expr.span);
                Some(Expr {
                    kind: ExprKind::Unary {
                        op: UnaryOp::Not,
                        expr: Box::new(expr),
                    },
                    span,
                })
            }
            TokenKind::And => {
                self.bump();
                let expr = self.parse_expr(prefix_binding_power())?;
                let span = start.merge(expr.span);
                Some(Expr {
                    kind: ExprKind::Unary {
                        op: UnaryOp::Ref,
                        expr: Box::new(expr),
                    },
                    span,
                })
            }
            TokenKind::Star => {
                self.bump();
                let expr = self.parse_expr(prefix_binding_power())?;
                let span = start.merge(expr.span);
                Some(Expr {
                    kind: ExprKind::Unary {
                        op: UnaryOp::Deref,
                        expr: Box::new(expr),
                    },
                    span,
                })
            }
            _ => {
                let span = self.peek_span();
                self.diagnostics.push(
                    Diagnostic::error("expected expression")
                        .with_code("E0103")
                        .with_label(span, "here"),
                );
                None
            }
        }
    }

    fn looks_like_struct_lit(&self) -> bool {
        // Peek: `{` Ident `:`
        let mut i = self.pos;
        if self.tokens.get(i).map(|t| t.kind) != Some(TokenKind::LBrace) {
            return false;
        }
        i += 1;
        // empty struct lit `Name {}`
        if self.tokens.get(i).map(|t| t.kind) == Some(TokenKind::RBrace) {
            return true;
        }
        if self.tokens.get(i).map(|t| t.kind) != Some(TokenKind::Ident) {
            return false;
        }
        i += 1;
        self.tokens.get(i).map(|t| t.kind) == Some(TokenKind::Colon)
    }

    fn parse_struct_lit(&mut self, name: String, start: Span) -> Option<Expr> {
        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.is_eof() {
            let fstart = self.peek_span();
            let fname = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let value = self.parse_expr(0)?;
            let fspan = fstart.merge(value.span);
            fields.push(FieldInit {
                name: fname,
                value,
                span: fspan,
            });
            if self.at(TokenKind::Comma) {
                self.bump();
            } else {
                break;
            }
        }
        let end = self.peek_span();
        self.expect(TokenKind::RBrace)?;
        Some(Expr {
            kind: ExprKind::StructLit { name, fields },
            span: start.merge(end),
        })
    }

    fn parse_if(&mut self) -> Option<Expr> {
        let start = self.peek_span();
        self.expect(TokenKind::If)?;
        let cond = self.parse_expr(0)?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.at(TokenKind::Else) {
            self.bump();
            if self.at(TokenKind::If) {
                Some(Box::new(self.parse_if()?))
            } else {
                let block = self.parse_block()?;
                let span = block.span;
                Some(Box::new(Expr {
                    kind: ExprKind::Block(block),
                    span,
                }))
            }
        } else {
            None
        };
        let end = else_branch
            .as_ref()
            .map(|e| e.span)
            .unwrap_or(then_branch.span);
        Some(Expr {
            kind: ExprKind::If {
                cond: Box::new(cond),
                then_branch,
                else_branch,
            },
            span: start.merge(end),
        })
    }

    fn parse_match(&mut self) -> Option<Expr> {
        let start = self.peek_span();
        self.expect(TokenKind::Match)?;
        let scrutinee = self.parse_expr(0)?;
        self.expect(TokenKind::LBrace)?;
        let mut arms = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.is_eof() {
            let astart = self.peek_span();
            let pattern = self.parse_pattern()?;
            self.expect(TokenKind::FatArrow)?;
            let body = if self.at(TokenKind::LBrace) {
                let block = self.parse_block()?;
                let span = block.span;
                Expr {
                    kind: ExprKind::Block(block),
                    span,
                }
            } else {
                self.parse_expr(0)?
            };
            let aspan = astart.merge(body.span);
            arms.push(MatchArm {
                pattern,
                body,
                span: aspan,
            });
            if self.at(TokenKind::Comma) {
                self.bump();
            }
        }
        let end = self.peek_span();
        self.expect(TokenKind::RBrace)?;
        Some(Expr {
            kind: ExprKind::Match {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            span: start.merge(end),
        })
    }

    fn parse_pattern(&mut self) -> Option<Pattern> {
        if self.at(TokenKind::Ident) && self.lexeme() == "_" {
            self.bump();
            return Some(Pattern::Wildcard);
        }
        match self.peek_kind() {
            TokenKind::True => {
                self.bump();
                Some(Pattern::Lit(Lit::Bool(true)))
            }
            TokenKind::False => {
                self.bump();
                Some(Pattern::Lit(Lit::Bool(false)))
            }
            TokenKind::Literal(kind) => {
                let text = self.lexeme().to_string();
                self.bump();
                let lit = match kind {
                    LiteralKind::Integer => Lit::Int(text),
                    LiteralKind::Float => Lit::Float(text),
                    LiteralKind::String => Lit::Str(strip_quotes(&text)),
                    LiteralKind::Char => Lit::Char(strip_char_quotes(&text)),
                };
                Some(Pattern::Lit(lit))
            }
            TokenKind::Ident => {
                let name = self.lexeme().to_string();
                self.bump();
                let mut path = name.clone();
                while self.at(TokenKind::ColonColon) {
                    self.bump();
                    let part = self.expect_ident()?;
                    path.push_str("::");
                    path.push_str(&part);
                }
                if self.at(TokenKind::LParen) {
                    self.bump();
                    let mut pats = Vec::new();
                    if !self.at(TokenKind::RParen) {
                        loop {
                            pats.push(self.parse_pattern()?);
                            if self.at(TokenKind::Comma) {
                                self.bump();
                                if self.at(TokenKind::RParen) {
                                    break;
                                }
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    Some(Pattern::Variant {
                        path,
                        kind: PatternVariantKind::Tuple(pats),
                    })
                } else if self.at(TokenKind::LBrace) && self.looks_like_struct_lit() {
                    self.bump();
                    let mut fields = Vec::new();
                    while !self.at(TokenKind::RBrace) && !self.is_eof() {
                        let fname = self.expect_ident()?;
                        let pat = if self.at(TokenKind::Colon) {
                            self.bump();
                            self.parse_pattern()?
                        } else {
                            Pattern::Ident(fname.clone())
                        };
                        fields.push((fname, pat));
                        if self.at(TokenKind::Comma) {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    Some(Pattern::Variant {
                        path,
                        kind: PatternVariantKind::Struct(fields),
                    })
                } else if path.contains("::")
                    || path.chars().next().is_some_and(|c| c.is_uppercase())
                {
                    // Treat Capitalized / path as unit variant; lowercase as binding
                    if path.contains("::") || path.chars().next().is_some_and(|c| c.is_uppercase())
                    {
                        Some(Pattern::Variant {
                            path,
                            kind: PatternVariantKind::Unit,
                        })
                    } else {
                        Some(Pattern::Ident(path))
                    }
                } else {
                    Some(Pattern::Ident(path))
                }
            }
            _ => {
                let span = self.peek_span();
                self.diagnostics.push(
                    Diagnostic::error("expected pattern")
                        .with_code("E0106")
                        .with_label(span, "here"),
                );
                None
            }
        }
    }

    fn parse_call(&mut self, callee: Expr) -> Option<Expr> {
        self.expect(TokenKind::LParen)?;
        let mut args = Vec::new();
        if !self.at(TokenKind::RParen) {
            loop {
                args.push(self.parse_expr(0)?);
                if self.at(TokenKind::Comma) {
                    self.bump();
                    if self.at(TokenKind::RParen) {
                        break;
                    }
                    continue;
                }
                break;
            }
        }
        let end = self.peek_span();
        self.expect(TokenKind::RParen)?;
        let span = callee.span.merge(end);
        Some(Expr {
            kind: ExprKind::Call {
                callee: Box::new(callee),
                args,
            },
            span,
        })
    }

    fn is_eof(&self) -> bool {
        self.peek_kind() == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().expect("token stream has EOF"))
    }

    fn peek_kind(&self) -> TokenKind {
        self.peek().kind
    }

    fn peek_span(&self) -> Span {
        self.peek().span
    }

    fn prev_span(&self) -> Span {
        if self.pos == 0 {
            Span::at(self.file_id, 0)
        } else {
            self.tokens[self.pos - 1].span
        }
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek_kind() == kind
    }

    fn bump(&mut self) -> Span {
        let span = self.peek_span();
        if !self.is_eof() {
            self.pos += 1;
        }
        span
    }

    fn lexeme(&self) -> &str {
        let span = self.peek_span();
        &self.source[span.lo.as_usize()..span.hi.as_usize()]
    }

    fn expect(&mut self, kind: TokenKind) -> Option<Span> {
        if self.at(kind) {
            Some(self.bump())
        } else {
            let span = self.peek_span();
            self.diagnostics.push(
                Diagnostic::error(format!("expected {kind}, found {}", self.peek_kind()))
                    .with_code("E0104")
                    .with_label(span, "unexpected token"),
            );
            None
        }
    }

    fn expect_ident(&mut self) -> Option<String> {
        if self.at(TokenKind::Ident) {
            let name = self.lexeme().to_string();
            self.bump();
            Some(name)
        } else {
            let span = self.peek_span();
            self.diagnostics.push(
                Diagnostic::error(format!("expected identifier, found {}", self.peek_kind()))
                    .with_code("E0105")
                    .with_label(span, "here"),
            );
            None
        }
    }
}

fn strip_quotes(s: &str) -> String {
    let t = s.strip_prefix('"').unwrap_or(s);
    t.strip_suffix('"').unwrap_or(t).to_string()
}

fn is_block_tailed(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::If { .. } | ExprKind::Match { .. } | ExprKind::Block(_)
    )
}

fn strip_char_quotes(s: &str) -> String {
    let t = s.strip_prefix('\'').unwrap_or(s);
    t.strip_suffix('\'').unwrap_or(t).to_string()
}

fn prefix_binding_power() -> u8 {
    90
}

fn infix_binding_power(kind: TokenKind) -> Option<(u8, u8)> {
    Some(match kind {
        TokenKind::Eq => (10, 9),
        TokenKind::OrOr => (20, 21),
        TokenKind::AndAnd => (30, 31),
        TokenKind::EqEq | TokenKind::Ne => (40, 41),
        TokenKind::Lt | TokenKind::Le | TokenKind::Gt | TokenKind::Ge => (50, 51),
        TokenKind::Plus | TokenKind::Minus => (60, 61),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => (70, 71),
        _ => return None,
    })
}

fn token_to_binop(kind: TokenKind) -> Option<BinOp> {
    Some(match kind {
        TokenKind::Plus => BinOp::Add,
        TokenKind::Minus => BinOp::Sub,
        TokenKind::Star => BinOp::Mul,
        TokenKind::Slash => BinOp::Div,
        TokenKind::Percent => BinOp::Rem,
        TokenKind::EqEq => BinOp::Eq,
        TokenKind::Ne => BinOp::Ne,
        TokenKind::Lt => BinOp::Lt,
        TokenKind::Le => BinOp::Le,
        TokenKind::Gt => BinOp::Gt,
        TokenKind::Ge => BinOp::Ge,
        TokenKind::AndAnd => BinOp::And,
        TokenKind::OrOr => BinOp::Or,
        TokenKind::Eq => BinOp::Assign,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxa_lexer::Lexer;
    use foxa_span::SourceMap;

    fn parse(src: &str) -> (Module, DiagnosticBag) {
        let mut map = SourceMap::new();
        let id = map.add_file("t.foxa", src);
        let mut bag = DiagnosticBag::new();
        let tokens = Lexer::new(id, src, &mut bag).tokenize_all();
        let module = Parser::new(id, src, tokens, &mut bag).parse_module();
        (module, bag)
    }

    #[test]
    fn parse_hello_fn() {
        let (m, bag) = parse("fn main() {\n    print(\"hello\");\n}");
        assert!(!bag.has_errors(), "{:?}", bag.items());
        assert_eq!(m.items.len(), 1);
    }

    #[test]
    fn parse_struct_enum() {
        let (m, bag) = parse(
            r#"
            struct Point { x: Int, y: Int }
            enum Option { None, Some(Int) }
            fn main() {
                let p = Point { x: 1, y: 2 };
                let n = p.x;
            }
            "#,
        );
        assert!(!bag.has_errors(), "{:?}", bag.items());
        assert_eq!(m.items.len(), 3);
        assert!(matches!(m.items[0].kind, ItemKind::Struct(_)));
        assert!(matches!(m.items[1].kind, ItemKind::Enum(_)));
    }

    #[test]
    fn parse_while_for() {
        let (m, bag) = parse(
            r#"
            fn main() {
                let mut i = 0;
                while i < 3 {
                    i = i + 1;
                }
                for x in [1, 2, 3] {
                    print(x);
                }
            }
            "#,
        );
        assert!(!bag.has_errors(), "{:?}", bag.items());
        match &m.items[0].kind {
            ItemKind::Fn(f) => {
                assert!(f
                    .body
                    .stmts
                    .iter()
                    .any(|s| matches!(s.kind, StmtKind::While { .. })));
                assert!(f
                    .body
                    .stmts
                    .iter()
                    .any(|s| matches!(s.kind, StmtKind::For { .. })));
            }
            _ => panic!("expected fn"),
        }
    }

    #[test]
    fn parse_match() {
        let (m, bag) = parse(
            r#"
            fn main() {
                match Some(1) {
                    Some(x) => print(x),
                    None => print(0),
                }
            }
            "#,
        );
        assert!(!bag.has_errors(), "{:?}", bag.items());
        match &m.items[0].kind {
            ItemKind::Fn(f) => {
                let in_stmts = f.body.stmts.iter().any(|s| {
                    matches!(&s.kind, StmtKind::Expr(e) if matches!(e.kind, ExprKind::Match { .. }))
                });
                let in_tail = matches!(
                    f.body.expr.as_deref().map(|e| &e.kind),
                    Some(ExprKind::Match { .. })
                );
                assert!(in_stmts || in_tail);
            }
            _ => panic!("expected fn"),
        }
    }

    #[test]
    fn parse_let_and_expr() {
        let (m, bag) = parse("fn add(a: Int, b: Int) -> Int { let x = a + b; x }");
        assert!(!bag.has_errors(), "{:?}", bag.items());
        match &m.items[0].kind {
            ItemKind::Fn(f) => {
                assert_eq!(f.params.len(), 2);
                assert!(f.body.expr.is_some());
            }
            _ => panic!("expected fn"),
        }
    }

    #[test]
    fn precedence_mul_over_add() {
        let (m, bag) = parse("fn f() { 1 + 2 * 3 }");
        assert!(!bag.has_errors());
        match &m.items[0].kind {
            ItemKind::Fn(f) => {
                let expr = f.body.expr.as_ref().unwrap();
                match &expr.kind {
                    ExprKind::Binary {
                        op: BinOp::Add,
                        rhs,
                        ..
                    } => match &rhs.kind {
                        ExprKind::Binary { op: BinOp::Mul, .. } => {}
                        other => panic!("expected mul on rhs, got {other:?}"),
                    },
                    other => panic!("expected add, got {other:?}"),
                }
            }
            _ => panic!("expected fn"),
        }
    }
}
