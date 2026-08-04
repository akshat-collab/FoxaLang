//! Type checking walk.

use crate::ty::Ty;
use foxa_ast::{
    BinOp, Block, EnumItem, Expr, ExprKind, FnItem, ItemKind, Lit, MatchArm, Module, Pattern,
    PatternVariantKind, Stmt, StmtKind, StructItem, UnaryOp, VariantKind,
};
use foxa_diagnostics::{Diagnostic, DiagnosticBag};
use foxa_resolve::{ResolveMap, SymbolId, SymbolKind};
use foxa_span::Span;
use std::collections::HashMap;

/// Maps symbols and expression spans to types.
#[derive(Debug, Default)]
pub struct TypeMap {
    symbols: HashMap<SymbolId, Ty>,
    exprs: HashMap<SpanKey, Ty>,
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

impl TypeMap {
    /// Type of a symbol, if known.
    #[must_use]
    pub fn symbol_ty(&self, id: SymbolId) -> Option<&Ty> {
        self.symbols.get(&id)
    }

    /// Type of an expression span, if known.
    #[must_use]
    pub fn expr_ty(&self, span: Span) -> Option<&Ty> {
        self.exprs.get(&SpanKey::from(span))
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum VariantInfo {
    Unit,
    Tuple(Vec<Ty>),
    Struct(HashMap<String, Ty>),
}

#[derive(Debug, Clone)]
struct StructInfo {
    fields: HashMap<String, Ty>,
}

#[derive(Debug, Clone)]
struct EnumInfo {
    variants: HashMap<String, VariantInfo>,
}

/// Monomorphic type checker.
pub struct TypeChecker<'a> {
    resolve: &'a ResolveMap,
    diagnostics: &'a mut DiagnosticBag,
    types: TypeMap,
    expected_return: Ty,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    /// variant name -> owning enum name
    variant_owner: HashMap<String, String>,
}

impl<'a> TypeChecker<'a> {
    /// Creates a type checker over a resolved module.
    pub fn new(resolve: &'a ResolveMap, diagnostics: &'a mut DiagnosticBag) -> Self {
        let mut checker = Self {
            resolve,
            diagnostics,
            types: TypeMap::default(),
            expected_return: Ty::Unit,
            structs: HashMap::new(),
            enums: HashMap::new(),
            variant_owner: HashMap::new(),
        };
        checker.install_builtin_types();
        checker
    }

    /// Type-checks the module and returns the type map.
    pub fn check(mut self, module: &Module) -> TypeMap {
        for item in &module.items {
            match &item.kind {
                ItemKind::Struct(s) => self.register_struct(s),
                ItemKind::Enum(e) => self.register_enum(e),
                ItemKind::Fn(func) => self.declare_fn_sig(func),
                ItemKind::Error => {}
            }
        }
        for item in &module.items {
            if let ItemKind::Fn(func) = &item.kind {
                self.check_fn(func);
            }
        }
        self.types
    }

    fn install_builtin_types(&mut self) {
        if let Some(id) = self.resolve.lookup_function("print") {
            self.types.symbols.insert(
                id,
                Ty::Fn {
                    params: vec![Ty::String],
                    ret: Box::new(Ty::Unit),
                },
            );
        }
        if let Some(id) = self.resolve.lookup_function("assert") {
            self.types.symbols.insert(
                id,
                Ty::Fn {
                    params: vec![Ty::Bool],
                    ret: Box::new(Ty::Unit),
                },
            );
        }
        // Builtin Option / Result variant owners
        self.variant_owner.insert("None".into(), "Option".into());
        self.variant_owner.insert("Some".into(), "Option".into());
        self.variant_owner.insert("Ok".into(), "Result".into());
        self.variant_owner.insert("Err".into(), "Result".into());
        self.enums.insert(
            "Option".into(),
            EnumInfo {
                variants: HashMap::from([
                    ("None".into(), VariantInfo::Unit),
                    (
                        "Some".into(),
                        VariantInfo::Tuple(vec![Ty::Named("_".into())]),
                    ),
                ]),
            },
        );
        self.enums.insert(
            "Result".into(),
            EnumInfo {
                variants: HashMap::from([
                    ("Ok".into(), VariantInfo::Tuple(vec![Ty::Named("_".into())])),
                    (
                        "Err".into(),
                        VariantInfo::Tuple(vec![Ty::Named("_".into())]),
                    ),
                ]),
            },
        );
    }

    fn register_struct(&mut self, s: &StructItem) {
        let mut fields = HashMap::new();
        for f in &s.fields {
            fields.insert(f.name.clone(), Ty::from_name(&f.ty));
        }
        self.structs.insert(s.name.clone(), StructInfo { fields });
    }

    fn register_enum(&mut self, e: &EnumItem) {
        let mut variants = HashMap::new();
        for v in &e.variants {
            let info = match &v.kind {
                VariantKind::Unit => VariantInfo::Unit,
                VariantKind::Tuple(tys) => {
                    VariantInfo::Tuple(tys.iter().map(|t| Ty::from_name(t)).collect())
                }
                VariantKind::Struct(fields) => {
                    let mut map = HashMap::new();
                    for f in fields {
                        map.insert(f.name.clone(), Ty::from_name(&f.ty));
                    }
                    VariantInfo::Struct(map)
                }
            };
            variants.insert(v.name.clone(), info);
            self.variant_owner.insert(v.name.clone(), e.name.clone());
        }
        self.enums.insert(e.name.clone(), EnumInfo { variants });
    }

    fn declare_fn_sig(&mut self, func: &FnItem) {
        let params: Vec<Ty> = func.params.iter().map(|p| Ty::from_name(&p.ty)).collect();
        let ret = func
            .return_ty
            .as_deref()
            .map(Ty::from_name)
            .unwrap_or(Ty::Unit);
        let fn_ty = Ty::Fn {
            params: params.clone(),
            ret: Box::new(ret),
        };
        if let Some(id) = self.resolve.lookup_function(&func.name) {
            self.types.symbols.insert(id, fn_ty);
            for (param, ty) in func.params.iter().zip(params) {
                if let Some(sym) = self.resolve.symbols().iter().find(|s| {
                    s.kind == SymbolKind::Param && s.name == param.name && s.span == param.span
                }) {
                    self.types.symbols.insert(sym.id, ty);
                }
            }
        }
    }

    fn check_fn(&mut self, func: &FnItem) {
        let ret = func
            .return_ty
            .as_deref()
            .map(Ty::from_name)
            .unwrap_or(Ty::Unit);
        self.expected_return = ret.clone();
        let block_ty = self.check_block(&func.body);
        if !block_ty.is_error()
            && block_ty != ret
            && ret != Ty::Unit
            && (func.body.expr.is_some() || !matches!(block_ty, Ty::Unit))
        {
            self.diagnostics.push(
                Diagnostic::error(format!(
                    "function `{}` returns `{ret}` but body has type `{block_ty}`",
                    func.name
                ))
                .with_code("E0300")
                .with_label(func.body.span, format!("expected `{ret}`")),
            );
        }
        if func.body.expr.is_none() && ret != Ty::Unit {
            let has_return = func
                .body
                .stmts
                .iter()
                .any(|s| matches!(s.kind, StmtKind::Return(_)));
            if !has_return {
                self.diagnostics.push(
                    Diagnostic::error(format!(
                        "function `{}` must return `{ret}` but has no return value",
                        func.name
                    ))
                    .with_code("E0301")
                    .with_label(func.body.span, "missing return value"),
                );
            }
        }
    }

    fn check_block(&mut self, block: &Block) -> Ty {
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
        if let Some(expr) = &block.expr {
            self.check_expr(expr)
        } else {
            Ty::Unit
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let { name, ty, init, .. } => {
                let annotated = ty.as_deref().map(Ty::from_name);
                let init_ty = init.as_ref().map(|e| self.check_expr(e));
                let final_ty = match (annotated, init_ty) {
                    (Some(a), Some(i)) => {
                        if !i.is_error() && !types_compatible(&a, &i) {
                            self.diagnostics.push(
                                Diagnostic::error(format!(
                                    "mismatched types: expected `{a}`, found `{i}`"
                                ))
                                .with_code("E0302")
                                .with_label(stmt.span, "type mismatch"),
                            );
                            Ty::Error
                        } else {
                            a
                        }
                    }
                    (Some(a), None) => a,
                    (None, Some(i)) => i,
                    (None, None) => {
                        self.diagnostics.push(
                            Diagnostic::error(format!(
                                "cannot infer type for `{name}`; add a type annotation or initializer"
                            ))
                            .with_code("E0303")
                            .with_label(stmt.span, "needs type"),
                        );
                        Ty::Error
                    }
                };
                if let Some(sym) =
                    self.resolve.symbols().iter().find(|s| {
                        s.kind == SymbolKind::Local && s.name == *name && s.span == stmt.span
                    })
                {
                    self.types.symbols.insert(sym.id, final_ty);
                }
            }
            StmtKind::While { cond, body } => {
                let cond_ty = self.check_expr(cond);
                if !cond_ty.is_error() && cond_ty != Ty::Bool {
                    self.diagnostics.push(
                        Diagnostic::error(format!(
                            "while condition must be `Bool`, found `{cond_ty}`"
                        ))
                        .with_code("E0315")
                        .with_label(cond.span, "expected `Bool`"),
                    );
                }
                let _ = self.check_block(body);
            }
            StmtKind::For { name, iter, body } => {
                let iter_ty = self.check_expr(iter);
                let elem_ty = match &iter_ty {
                    Ty::Vec(inner) => *inner.clone(),
                    Ty::Error => Ty::Error,
                    other => {
                        self.diagnostics.push(
                            Diagnostic::error(format!("`for` requires `Vec[T]`, found `{other}`"))
                                .with_code("E0316")
                                .with_label(iter.span, "not iterable"),
                        );
                        Ty::Error
                    }
                };
                if let Some(sym) =
                    self.resolve.symbols().iter().find(|s| {
                        s.kind == SymbolKind::Local && s.name == *name && s.span == stmt.span
                    })
                {
                    self.types.symbols.insert(sym.id, elem_ty);
                }
                let _ = self.check_block(body);
            }
            StmtKind::Expr(expr) => {
                let _ = self.check_expr(expr);
            }
            StmtKind::Return(value) => {
                let got = value
                    .as_ref()
                    .map(|e| self.check_expr(e))
                    .unwrap_or(Ty::Unit);
                let expected = self.expected_return.clone();
                if !got.is_error() && !types_compatible(&expected, &got) {
                    self.diagnostics.push(
                        Diagnostic::error(format!(
                            "mismatched return type: expected `{expected}`, found `{got}`"
                        ))
                        .with_code("E0304")
                        .with_label(stmt.span, "return type mismatch"),
                    );
                }
            }
            StmtKind::Empty | StmtKind::Break | StmtKind::Continue => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Ty {
        let ty = match &expr.kind {
            ExprKind::Literal(lit) => match lit {
                Lit::Int(_) => Ty::Int,
                Lit::Float(_) => Ty::Float,
                Lit::Str(_) => Ty::String,
                Lit::Char(_) => Ty::Char,
                Lit::Bool(_) => Ty::Bool,
            },
            ExprKind::Path(name) => {
                if let Some(id) = self.resolve.resolve_expr(expr.span) {
                    if let Some(sym) = self.resolve.symbol(id) {
                        if sym.kind == SymbolKind::Variant {
                            self.variant_type(name)
                        } else {
                            self.types
                                .symbols
                                .get(&id)
                                .cloned()
                                .unwrap_or_else(|| self.variant_type(name))
                        }
                    } else {
                        Ty::Error
                    }
                } else {
                    Ty::Error
                }
            }
            ExprKind::Binary { op, lhs, rhs } => self.check_binary(*op, lhs, rhs, expr.span),
            ExprKind::Unary { op, expr: inner } => self.check_unary(*op, inner),
            ExprKind::Call { callee, args } => self.check_call(callee, args, expr.span),
            ExprKind::Field { base, field } => {
                let base_ty = self.check_expr(base);
                self.check_field(&base_ty, field, expr.span)
            }
            ExprKind::StructLit { name, fields } => {
                for f in fields {
                    let _ = self.check_expr(&f.value);
                }
                if let Some(info) = self.structs.get(name).cloned() {
                    for f in fields {
                        match info.fields.get(&f.name) {
                            Some(expected) => {
                                let got = self
                                    .types
                                    .expr_ty(f.value.span)
                                    .cloned()
                                    .unwrap_or(Ty::Error);
                                if !got.is_error() && !types_compatible(expected, &got) {
                                    self.diagnostics.push(
                                        Diagnostic::error(format!(
                                            "field `{}`: expected `{expected}`, found `{got}`",
                                            f.name
                                        ))
                                        .with_code("E0317")
                                        .with_label(f.span, "wrong field type"),
                                    );
                                }
                            }
                            None => {
                                self.diagnostics.push(
                                    Diagnostic::error(format!(
                                        "struct `{name}` has no field `{}`",
                                        f.name
                                    ))
                                    .with_code("E0318")
                                    .with_label(f.span, "unknown field"),
                                );
                            }
                        }
                    }
                    Ty::Named(name.clone())
                } else if let Some(owner) = self.variant_owner.get(name).cloned() {
                    Ty::Named(owner)
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(format!("unknown struct or variant `{name}`"))
                            .with_code("E0319")
                            .with_label(expr.span, "not found"),
                    );
                    Ty::Error
                }
            }
            ExprKind::Group(inner) => self.check_expr(inner),
            ExprKind::Block(block) => self.check_block(block),
            ExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_ty = self.check_expr(cond);
                if !cond_ty.is_error() && cond_ty != Ty::Bool {
                    self.diagnostics.push(
                        Diagnostic::error(format!("condition must be `Bool`, found `{cond_ty}`"))
                            .with_code("E0305")
                            .with_label(cond.span, "expected `Bool`"),
                    );
                }
                let then_ty = self.check_block(then_branch);
                if let Some(els) = else_branch {
                    let else_ty = self.check_expr(els);
                    if !then_ty.is_error() && !else_ty.is_error() && then_ty != else_ty {
                        self.diagnostics.push(
                            Diagnostic::error(format!(
                                "`if` branches have incompatible types `{then_ty}` and `{else_ty}`"
                            ))
                            .with_code("E0306")
                            .with_label(expr.span, "type mismatch between branches"),
                        );
                        Ty::Error
                    } else {
                        then_ty
                    }
                } else {
                    Ty::Unit
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                let scrut_ty = self.check_expr(scrutinee);
                let mut result: Option<Ty> = None;
                for arm in arms {
                    self.bind_pattern_types(&arm.pattern, &scrut_ty, arm);
                    let arm_ty = self.check_expr(&arm.body);
                    match &result {
                        None => result = Some(arm_ty),
                        Some(prev) if !prev.is_error() && !arm_ty.is_error() && prev != &arm_ty => {
                            self.diagnostics.push(
                                Diagnostic::error(format!(
                                    "`match` arms have incompatible types `{prev}` and `{arm_ty}`"
                                ))
                                .with_code("E0320")
                                .with_label(arm.span, "type mismatch"),
                            );
                            result = Some(Ty::Error);
                        }
                        _ => {}
                    }
                }
                result.unwrap_or(Ty::Unit)
            }
            ExprKind::Array(elems) => {
                if elems.is_empty() {
                    Ty::Vec(Box::new(Ty::Named("_".into())))
                } else {
                    let first = self.check_expr(&elems[0]);
                    for e in &elems[1..] {
                        let t = self.check_expr(e);
                        if !t.is_error() && !first.is_error() && t != first {
                            self.diagnostics.push(
                                Diagnostic::error(format!(
                                    "array elements must have the same type, found `{first}` and `{t}`"
                                ))
                                .with_code("E0321")
                                .with_label(e.span, "mismatched element type"),
                            );
                        }
                    }
                    Ty::Vec(Box::new(first))
                }
            }
            ExprKind::Error => Ty::Error,
        };
        self.types
            .exprs
            .insert(SpanKey::from(expr.span), ty.clone());
        ty
    }

    fn variant_type(&self, name: &str) -> Ty {
        match self.variant_owner.get(name).map(|s| s.as_str()) {
            Some("Option") => Ty::Option(Box::new(Ty::Named("_".into()))),
            Some("Result") => Ty::Result {
                ok: Box::new(Ty::Named("_".into())),
                err: Box::new(Ty::Named("_".into())),
            },
            Some(owner) => Ty::Named(owner.to_string()),
            None => Ty::Named(name.to_string()),
        }
    }

    fn check_field(&mut self, base_ty: &Ty, field: &str, span: Span) -> Ty {
        match base_ty {
            Ty::Named(name) => {
                if let Some(info) = self.structs.get(name) {
                    if let Some(ty) = info.fields.get(field) {
                        return ty.clone();
                    }
                }
                self.diagnostics.push(
                    Diagnostic::error(format!("no field `{field}` on type `{name}`"))
                        .with_code("E0322")
                        .with_label(span, "unknown field"),
                );
                Ty::Error
            }
            Ty::Error => Ty::Error,
            other => {
                self.diagnostics.push(
                    Diagnostic::error(format!("type `{other}` has no fields"))
                        .with_code("E0323")
                        .with_label(span, "not a struct"),
                );
                Ty::Error
            }
        }
    }

    fn bind_pattern_types(&mut self, pattern: &Pattern, scrut_ty: &Ty, arm: &MatchArm) {
        match pattern {
            Pattern::Wildcard | Pattern::Lit(_) => {}
            Pattern::Ident(name) => {
                if let Some(sym) =
                    self.resolve.symbols().iter().find(|s| {
                        s.kind == SymbolKind::Local && s.name == *name && s.span == arm.span
                    })
                {
                    self.types.symbols.insert(sym.id, scrut_ty.clone());
                }
            }
            Pattern::Variant { path, kind } => {
                let short = path.rsplit("::").next().unwrap_or(path);
                match kind {
                    PatternVariantKind::Unit => {}
                    PatternVariantKind::Tuple(pats) => {
                        let payload = match (short, scrut_ty) {
                            ("Some", Ty::Option(inner)) => vec![*inner.clone()],
                            ("Ok", Ty::Result { ok, .. }) => vec![*ok.clone()],
                            ("Err", Ty::Result { err, .. }) => vec![*err.clone()],
                            _ => {
                                if let Some(owner) = self.variant_owner.get(short) {
                                    if let Some(EnumInfo { variants }) = self.enums.get(owner) {
                                        if let Some(VariantInfo::Tuple(tys)) = variants.get(short) {
                                            tys.clone()
                                        } else {
                                            vec![]
                                        }
                                    } else {
                                        vec![]
                                    }
                                } else {
                                    vec![]
                                }
                            }
                        };
                        for (pat, ty) in pats.iter().zip(payload.iter()) {
                            self.bind_pattern_types(pat, ty, arm);
                        }
                    }
                    PatternVariantKind::Struct(fields) => {
                        for (fname, pat) in fields {
                            let fty = if let Ty::Named(n) = scrut_ty {
                                self.structs
                                    .get(n)
                                    .and_then(|s| s.fields.get(fname))
                                    .cloned()
                                    .unwrap_or(Ty::Error)
                            } else {
                                Ty::Error
                            };
                            self.bind_pattern_types(pat, &fty, arm);
                        }
                    }
                }
            }
        }
    }

    fn check_binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, span: Span) -> Ty {
        let left = self.check_expr(lhs);
        let right = self.check_expr(rhs);
        if left.is_error() || right.is_error() {
            return Ty::Error;
        }
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                if left == right && matches!(left, Ty::Int | Ty::Float) {
                    left
                } else if op == BinOp::Add && left == Ty::String && right == Ty::String {
                    Ty::String
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(format!(
                            "cannot apply `{op:?}` to `{left}` and `{right}`"
                        ))
                        .with_code("E0307")
                        .with_label(span, "invalid operands"),
                    );
                    Ty::Error
                }
            }
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                if types_compatible(&left, &right) {
                    Ty::Bool
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(format!("cannot compare `{left}` and `{right}`"))
                            .with_code("E0308")
                            .with_label(span, "type mismatch"),
                    );
                    Ty::Error
                }
            }
            BinOp::And | BinOp::Or => {
                if left == Ty::Bool && right == Ty::Bool {
                    Ty::Bool
                } else {
                    self.diagnostics.push(
                        Diagnostic::error("logical operators require `Bool` operands")
                            .with_code("E0309")
                            .with_label(span, "expected `Bool`"),
                    );
                    Ty::Error
                }
            }
            BinOp::Assign => {
                if types_compatible(&left, &right) {
                    Ty::Unit
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(format!("cannot assign `{right}` to `{left}`"))
                            .with_code("E0310")
                            .with_label(span, "type mismatch"),
                    );
                    Ty::Error
                }
            }
        }
    }

    fn check_unary(&mut self, op: UnaryOp, inner: &Expr) -> Ty {
        let ty = self.check_expr(inner);
        if ty.is_error() {
            return Ty::Error;
        }
        match op {
            UnaryOp::Neg if matches!(ty, Ty::Int | Ty::Float) => ty,
            UnaryOp::Not if ty == Ty::Bool => Ty::Bool,
            UnaryOp::Ref | UnaryOp::Deref => ty,
            _ => {
                self.diagnostics.push(
                    Diagnostic::error(format!("cannot apply `{op:?}` to `{ty}`"))
                        .with_code("E0311")
                        .with_label(inner.span, "invalid operand"),
                );
                Ty::Error
            }
        }
    }

    fn check_call(&mut self, callee: &Expr, args: &[Expr], span: Span) -> Ty {
        let arg_tys: Vec<Ty> = args.iter().map(|a| self.check_expr(a)).collect();

        if let ExprKind::Path(name) = &callee.kind {
            if (name == "print" || name == "show") && arg_tys.len() == 1 {
                let _ = self.check_expr(callee);
                return Ty::Unit;
            }
            // Variant constructors
            match name.as_str() {
                "Some" if arg_tys.len() == 1 => {
                    let _ = self.check_expr(callee);
                    return Ty::Option(Box::new(arg_tys[0].clone()));
                }
                "Ok" if arg_tys.len() == 1 => {
                    let _ = self.check_expr(callee);
                    return Ty::Result {
                        ok: Box::new(arg_tys[0].clone()),
                        err: Box::new(Ty::Named("_".into())),
                    };
                }
                "Err" if arg_tys.len() == 1 => {
                    let _ = self.check_expr(callee);
                    return Ty::Result {
                        ok: Box::new(Ty::Named("_".into())),
                        err: Box::new(arg_tys[0].clone()),
                    };
                }
                _ => {
                    if let Some(owner) = self.variant_owner.get(name).cloned() {
                        if let Some(info) = self.enums.get(&owner) {
                            if let Some(VariantInfo::Tuple(params)) = info.variants.get(name) {
                                let params = params.clone();
                                let _ = self.check_expr(callee);
                                if params.len() == arg_tys.len()
                                    || params.iter().any(|p| matches!(p, Ty::Named(n) if n == "_"))
                                {
                                    return Ty::Named(owner);
                                }
                            }
                        }
                    }
                }
            }
        }

        let callee_ty = self.check_expr(callee);
        match callee_ty {
            Ty::Fn { params, ret } => {
                if params.len() != arg_tys.len() {
                    self.diagnostics.push(
                        Diagnostic::error(format!(
                            "this function takes {} argument(s) but {} were supplied",
                            params.len(),
                            arg_tys.len()
                        ))
                        .with_code("E0312")
                        .with_label(span, "wrong argument count"),
                    );
                    return Ty::Error;
                }
                for (i, (expected, got)) in params.iter().zip(arg_tys.iter()).enumerate() {
                    if !got.is_error() && !types_compatible(expected, got) {
                        self.diagnostics.push(
                            Diagnostic::error(format!(
                                "argument {i}: expected `{expected}`, found `{got}`"
                            ))
                            .with_code("E0313")
                            .with_label(args[i].span, "wrong type"),
                        );
                    }
                }
                *ret
            }
            Ty::Error => Ty::Error,
            other => {
                self.diagnostics.push(
                    Diagnostic::error(format!("expected function, found `{other}`"))
                        .with_code("E0314")
                        .with_label(callee.span, "not a function"),
                );
                Ty::Error
            }
        }
    }
}

fn types_compatible(a: &Ty, b: &Ty) -> bool {
    if a == b {
        return true;
    }
    match (a, b) {
        (Ty::Named(n), _) | (_, Ty::Named(n)) if n == "_" => true,
        (Ty::Option(x), Ty::Option(y)) => types_compatible(x, y),
        (Ty::Vec(x), Ty::Vec(y)) => types_compatible(x, y),
        (Ty::Result { ok: a, err: e }, Ty::Result { ok: b, err: f }) => {
            types_compatible(a, b) && types_compatible(e, f)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxa_lexer::Lexer;
    use foxa_parser::Parser;
    use foxa_resolve::Resolver;
    use foxa_span::SourceMap;

    fn check_src(src: &str) -> DiagnosticBag {
        let mut map = SourceMap::new();
        let file = map.add_file("t.foxa", src);
        let mut bag = DiagnosticBag::new();
        let tokens = Lexer::new(file, src, &mut bag).tokenize_all();
        let module = Parser::new(file, src, tokens, &mut bag).parse_module();
        let resolved = Resolver::new(&mut bag).resolve(&module);
        if bag.has_errors() {
            return bag;
        }
        TypeChecker::new(&resolved, &mut bag).check(&module);
        bag
    }

    #[test]
    fn accepts_well_typed_add() {
        let bag = check_src("fn add(a: Int, b: Int) -> Int { a + b }");
        assert!(!bag.has_errors(), "{:?}", bag.items());
    }

    #[test]
    fn rejects_type_mismatch() {
        let bag = check_src("fn f() -> Int { true }");
        assert!(bag.has_errors());
    }

    #[test]
    fn rejects_bad_operands() {
        let bag = check_src("fn f() { let x = true + 1; }");
        assert!(bag.has_errors());
    }

    #[test]
    fn hello_typechecks() {
        let bag = check_src("fn main() { print(\"Hello, Foxa!\"); }");
        assert!(!bag.has_errors(), "{:?}", bag.items());
    }

    #[test]
    fn struct_and_while_typecheck() {
        let bag = check_src(
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

    #[test]
    fn option_match_typechecks() {
        let bag = check_src(
            r#"
            fn main() {
                let o = Some(1);
                match o {
                    Some(x) => print(x),
                    None => print(0),
                }
            }
            "#,
        );
        assert!(!bag.has_errors(), "{:?}", bag.items());
    }
}
