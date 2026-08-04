//! AST walker that builds scopes and resolves names.

use crate::scope::{Scope, ScopeId};
use crate::symbol::{Symbol, SymbolId, SymbolKind};
use foxa_ast::{
    Block, EnumItem, Expr, ExprKind, FnItem, ItemKind, Module, Pattern, PatternVariantKind, Stmt,
    StmtKind, StructItem,
};
use foxa_diagnostics::{Diagnostic, DiagnosticBag};
use foxa_span::Span;
use std::collections::HashMap;

/// Result of name resolution for a module.
#[derive(Debug, Default)]
pub struct ResolveMap {
    symbols: Vec<Symbol>,
    scopes: Vec<Scope>,
    expr_refs: HashMap<SpanKey, SymbolId>,
    functions: HashMap<String, SymbolId>,
    /// Type name → symbol.
    types: HashMap<String, SymbolId>,
    /// Variant short name → symbol.
    variants: HashMap<String, SymbolId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SpanKey {
    file: u32,
    lo: u32,
    hi: u32,
}

impl From<Span> for SpanKey {
    fn from(span: Span) -> Self {
        Self {
            file: span.file_id.as_raw(),
            lo: span.lo.as_u32(),
            hi: span.hi.as_u32(),
        }
    }
}

impl ResolveMap {
    /// All symbols.
    #[must_use]
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// All scopes.
    #[must_use]
    pub fn scopes(&self) -> &[Scope] {
        &self.scopes
    }

    /// Looks up a symbol by ID.
    #[must_use]
    pub fn symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.as_raw() as usize)
    }

    /// Resolves a path expression span to a symbol.
    #[must_use]
    pub fn resolve_expr(&self, span: Span) -> Option<SymbolId> {
        self.expr_refs.get(&SpanKey::from(span)).copied()
    }

    /// Looks up a module-level function by name.
    #[must_use]
    pub fn lookup_function(&self, name: &str) -> Option<SymbolId> {
        self.functions.get(name).copied()
    }

    /// Looks up a type by name.
    #[must_use]
    pub fn lookup_type(&self, name: &str) -> Option<SymbolId> {
        self.types.get(name).copied()
    }

    /// Looks up an enum variant by short name.
    #[must_use]
    pub fn lookup_variant(&self, name: &str) -> Option<SymbolId> {
        self.variants.get(name).copied()
    }
}

/// Name resolver.
pub struct Resolver<'a> {
    diagnostics: &'a mut DiagnosticBag,
    map: ResolveMap,
    current_scope: ScopeId,
}

impl<'a> Resolver<'a> {
    /// Creates a resolver that writes diagnostics into `diagnostics`.
    pub fn new(diagnostics: &'a mut DiagnosticBag) -> Self {
        let mut map = ResolveMap::default();
        let root = Scope::new(ScopeId::from_raw(0), None);
        map.scopes.push(root);
        let mut resolver = Self {
            diagnostics,
            map,
            current_scope: ScopeId::from_raw(0),
        };
        resolver.install_builtins();
        resolver
    }

    /// Resolves an entire module and returns the resolve map.
    pub fn resolve(mut self, module: &Module) -> ResolveMap {
        for item in &module.items {
            match &item.kind {
                ItemKind::Fn(func) => self.declare_function(func, item.span),
                ItemKind::Struct(s) => self.declare_struct(s),
                ItemKind::Enum(e) => self.declare_enum(e),
                ItemKind::Error => {}
            }
        }
        for item in &module.items {
            if let ItemKind::Fn(func) = &item.kind {
                self.resolve_function(func);
            }
        }
        self.map
    }

    fn install_builtins(&mut self) {
        let span = Span::at(foxa_span::FileId::from_raw(0), 0);
        for name in [
            "print", "assert", "Int", "Float", "Bool", "String", "Char", "Unit", "Vec", "Option",
            "Result",
        ] {
            let id = self.alloc_symbol(name, SymbolKind::Builtin, span, false);
            self.scope_mut(ScopeId::from_raw(0)).define(name, id);
            if matches!(name, "print" | "assert") {
                self.map.functions.insert(name.to_string(), id);
            }
            if matches!(
                name,
                "Vec" | "Option" | "Result" | "Int" | "Float" | "Bool" | "String" | "Char" | "Unit"
            ) {
                self.map.types.insert(name.to_string(), id);
            }
        }
        for (name, parent) in [
            ("None", "Option"),
            ("Some", "Option"),
            ("Ok", "Result"),
            ("Err", "Result"),
        ] {
            let _ = parent;
            let id = self.alloc_symbol(name, SymbolKind::Variant, span, false);
            self.scope_mut(ScopeId::from_raw(0)).define(name, id);
            self.map.variants.insert(name.to_string(), id);
        }
    }

    fn declare_function(&mut self, func: &FnItem, span: Span) {
        if let Some(prev) = self.lookup_in_chain(&func.name) {
            let prev_sym = self.map.symbol(prev).cloned();
            self.diagnostics.push(
                Diagnostic::error(format!("duplicate definition of `{}`", func.name))
                    .with_code("E0200")
                    .with_label(span, "redefined here"),
            );
            if let Some(prev_sym) = prev_sym {
                self.diagnostics.push(
                    Diagnostic::new(foxa_diagnostics::Severity::Note, "previous definition")
                        .with_label(prev_sym.span, "first defined here"),
                );
            }
            return;
        }
        let id = self.alloc_symbol(&func.name, SymbolKind::Function, span, false);
        let scope = self.current_scope;
        self.scope_mut(scope).define(&func.name, id);
        self.map.functions.insert(func.name.clone(), id);
    }

    fn declare_struct(&mut self, s: &StructItem) {
        let id = self.alloc_symbol(&s.name, SymbolKind::Struct, s.span, false);
        self.scope_mut(self.current_scope).define(&s.name, id);
        self.map.types.insert(s.name.clone(), id);
    }

    fn declare_enum(&mut self, e: &EnumItem) {
        let id = self.alloc_symbol(&e.name, SymbolKind::Enum, e.span, false);
        self.scope_mut(self.current_scope).define(&e.name, id);
        self.map.types.insert(e.name.clone(), id);
        for v in &e.variants {
            let vid = self.alloc_symbol(&v.name, SymbolKind::Variant, v.span, false);
            self.scope_mut(self.current_scope).define(&v.name, vid);
            self.map.variants.insert(v.name.clone(), vid);
        }
    }

    fn resolve_function(&mut self, func: &FnItem) {
        let parent = self.current_scope;
        let fn_scope = self.push_scope(parent);
        self.current_scope = fn_scope;

        for param in &func.params {
            if self.scope_mut(fn_scope).lookup_local(&param.name).is_some() {
                self.diagnostics.push(
                    Diagnostic::error(format!("duplicate parameter `{}`", param.name))
                        .with_code("E0201")
                        .with_label(param.span, "redefined here"),
                );
            } else {
                let id = self.alloc_symbol(&param.name, SymbolKind::Param, param.span, false);
                self.scope_mut(fn_scope).define(&param.name, id);
            }
        }

        self.resolve_block(&func.body);
        self.current_scope = parent;
    }

    fn resolve_block(&mut self, block: &Block) {
        let parent = self.current_scope;
        let block_scope = self.push_scope(parent);
        self.current_scope = block_scope;

        for stmt in &block.stmts {
            self.resolve_stmt(stmt);
        }
        if let Some(expr) = &block.expr {
            self.resolve_expr(expr);
        }

        self.current_scope = parent;
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let {
                mutable,
                name,
                init,
                ..
            } => {
                if let Some(init) = init {
                    self.resolve_expr(init);
                }
                let scope = self.current_scope;
                if self.scope_mut(scope).lookup_local(name).is_some() {
                    self.diagnostics.push(
                        Diagnostic::error(format!("duplicate binding `{name}`"))
                            .with_code("E0202")
                            .with_label(stmt.span, "redefined in this scope"),
                    );
                } else {
                    let id = self.alloc_symbol(name, SymbolKind::Local, stmt.span, *mutable);
                    self.scope_mut(scope).define(name, id);
                }
            }
            StmtKind::While { cond, body } => {
                self.resolve_expr(cond);
                self.resolve_block(body);
            }
            StmtKind::For { name, iter, body } => {
                self.resolve_expr(iter);
                let parent = self.current_scope;
                let loop_scope = self.push_scope(parent);
                self.current_scope = loop_scope;
                let id = self.alloc_symbol(name, SymbolKind::Local, stmt.span, false);
                self.scope_mut(loop_scope).define(name, id);
                self.resolve_block(body);
                self.current_scope = parent;
            }
            StmtKind::Expr(expr) | StmtKind::Return(Some(expr)) => self.resolve_expr(expr),
            StmtKind::Return(None) | StmtKind::Empty | StmtKind::Break | StmtKind::Continue => {}
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(_) | ExprKind::Error => {}
            ExprKind::Path(name) => {
                if let Some(id) = self.lookup_in_chain(name) {
                    self.map.expr_refs.insert(SpanKey::from(expr.span), id);
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(format!("cannot find value `{name}` in this scope"))
                            .with_code("E0203")
                            .with_label(expr.span, "not found in this scope")
                            .with_help("check spelling or declare the name before use"),
                    );
                }
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                self.resolve_expr(lhs);
                self.resolve_expr(rhs);
            }
            ExprKind::Unary { expr: inner, .. } | ExprKind::Group(inner) => {
                self.resolve_expr(inner);
            }
            ExprKind::Call { callee, args } => {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            ExprKind::Field { base, .. } => self.resolve_expr(base),
            ExprKind::StructLit { fields, .. } => {
                for f in fields {
                    self.resolve_expr(&f.value);
                }
            }
            ExprKind::Block(block) => self.resolve_block(block),
            ExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(cond);
                self.resolve_block(then_branch);
                if let Some(els) = else_branch {
                    self.resolve_expr(els);
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                self.resolve_expr(scrutinee);
                for arm in arms {
                    let parent = self.current_scope;
                    let arm_scope = self.push_scope(parent);
                    self.current_scope = arm_scope;
                    self.resolve_pattern(&arm.pattern, arm.span);
                    self.resolve_expr(&arm.body);
                    self.current_scope = parent;
                }
            }
            ExprKind::Array(elems) => {
                for e in elems {
                    self.resolve_expr(e);
                }
            }
        }
    }

    fn resolve_pattern(&mut self, pattern: &Pattern, span: Span) {
        match pattern {
            Pattern::Wildcard | Pattern::Lit(_) => {}
            Pattern::Ident(name) => {
                let id = self.alloc_symbol(name, SymbolKind::Local, span, false);
                self.scope_mut(self.current_scope).define(name, id);
            }
            Pattern::Variant { kind, .. } => match kind {
                PatternVariantKind::Unit => {}
                PatternVariantKind::Tuple(pats) => {
                    for p in pats {
                        self.resolve_pattern(p, span);
                    }
                }
                PatternVariantKind::Struct(fields) => {
                    for (_, p) in fields {
                        self.resolve_pattern(p, span);
                    }
                }
            },
        }
    }

    fn lookup_in_chain(&self, name: &str) -> Option<SymbolId> {
        let mut current = Some(self.current_scope);
        while let Some(scope_id) = current {
            let scope = &self.map.scopes[scope_id.as_raw() as usize];
            if let Some(id) = scope.lookup_local(name) {
                return Some(id);
            }
            current = scope.parent;
        }
        None
    }

    fn alloc_symbol(
        &mut self,
        name: impl Into<String>,
        kind: SymbolKind,
        span: Span,
        mutable: bool,
    ) -> SymbolId {
        let id = SymbolId::from_raw(self.map.symbols.len() as u32);
        self.map
            .symbols
            .push(Symbol::new(id, name, kind, span, mutable));
        id
    }

    fn push_scope(&mut self, parent: ScopeId) -> ScopeId {
        let id = ScopeId::from_raw(self.map.scopes.len() as u32);
        self.map.scopes.push(Scope::new(id, Some(parent)));
        id
    }

    fn scope_mut(&mut self, id: ScopeId) -> &mut Scope {
        &mut self.map.scopes[id.as_raw() as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxa_lexer::Lexer;
    use foxa_parser::Parser;
    use foxa_span::SourceMap;

    fn resolve_src(src: &str) -> (ResolveMap, DiagnosticBag) {
        let mut map = SourceMap::new();
        let file = map.add_file("t.foxa", src);
        let mut bag = DiagnosticBag::new();
        let tokens = Lexer::new(file, src, &mut bag).tokenize_all();
        let module = Parser::new(file, src, tokens, &mut bag).parse_module();
        let resolved = Resolver::new(&mut bag).resolve(&module);
        (resolved, bag)
    }

    #[test]
    fn resolves_local() {
        let (resolved, bag) = resolve_src("fn main() { let x = 1; print(x); }");
        assert!(!bag.has_errors(), "{:?}", bag.items());
        assert!(resolved.lookup_function("main").is_some());
    }

    #[test]
    fn unknown_name_errors() {
        let (_resolved, bag) = resolve_src("fn main() { print(missing); }");
        assert!(bag.has_errors());
    }

    #[test]
    fn duplicate_function_errors() {
        let (_resolved, bag) = resolve_src("fn f() {}\nfn f() {}");
        assert!(bag.has_errors());
    }

    #[test]
    fn params_visible_in_body() {
        let (_resolved, bag) = resolve_src("fn add(a: Int, b: Int) -> Int { a + b }");
        assert!(!bag.has_errors(), "{:?}", bag.items());
    }

    #[test]
    fn resolves_struct_and_while() {
        let (_resolved, bag) = resolve_src(
            r#"
            struct Point { x: Int, y: Int }
            fn main() {
                let p = Point { x: 1, y: 2 };
                let mut i = 0;
                while i < p.x {
                    i = i + 1;
                }
            }
            "#,
        );
        assert!(!bag.has_errors(), "{:?}", bag.items());
    }
}
