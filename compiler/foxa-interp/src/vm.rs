//! Tree-walk evaluation engine.

use crate::env::Environment;
use crate::error::InterpError;
use crate::value::Value;
use foxa_ast::{
    BinOp, Block, Expr, ExprKind, FnItem, ItemKind, Lit, Module, Pattern, PatternVariantKind, Stmt,
    StmtKind, UnaryOp,
};
use std::collections::HashMap;
use std::io::{self, Write};

/// Control-flow signal.
#[derive(Debug)]
enum Flow {
    Value(Value),
    Return(Value),
    Break,
    Continue,
}

/// Tree-walk interpreter.
pub struct Interpreter<'a> {
    module: &'a Module,
    functions: HashMap<String, &'a FnItem>,
    env: Environment,
    stdout: Box<dyn Write + 'a>,
}

impl<'a> Interpreter<'a> {
    /// Creates an interpreter for a type-checked module.
    #[must_use]
    pub fn new(module: &'a Module) -> Self {
        Self::with_stdout(module, Box::new(io::stdout()))
    }

    /// Creates an interpreter that writes `print` output to `stdout`.
    #[must_use]
    pub fn with_stdout(module: &'a Module, stdout: Box<dyn Write + 'a>) -> Self {
        let mut functions = HashMap::new();
        for item in &module.items {
            if let ItemKind::Fn(func) = &item.kind {
                functions.insert(func.name.clone(), func);
            }
        }
        Self {
            module,
            functions,
            env: Environment::new(),
            stdout,
        }
    }

    /// Runs the `main` function.
    pub fn run_main(&mut self) -> Result<Value, InterpError> {
        let _ = self.module;
        if !self.functions.contains_key("main") {
            return Err(InterpError::NoMain);
        }
        self.call_named("main", &[])
    }

    /// Calls a named function with arguments.
    pub fn call_named(&mut self, name: &str, args: &[Value]) -> Result<Value, InterpError> {
        match name {
            "print" | "show" => return self.builtin_print(args),
            "assert" => return self.builtin_assert(args),
            "Some" if args.len() == 1 => {
                return Ok(Value::Enum {
                    type_name: "Option".into(),
                    variant: "Some".into(),
                    fields: vec![args[0].clone()],
                    named: HashMap::new(),
                });
            }
            "Ok" if args.len() == 1 => {
                return Ok(Value::Enum {
                    type_name: "Result".into(),
                    variant: "Ok".into(),
                    fields: vec![args[0].clone()],
                    named: HashMap::new(),
                });
            }
            "Err" if args.len() == 1 => {
                return Ok(Value::Enum {
                    type_name: "Result".into(),
                    variant: "Err".into(),
                    fields: vec![args[0].clone()],
                    named: HashMap::new(),
                });
            }
            _ => {}
        }
        // User enum tuple variants: treat unknown names with args as Enum
        if !self.functions.contains_key(name) && !args.is_empty() {
            return Ok(Value::Enum {
                type_name: String::new(),
                variant: name.to_string(),
                fields: args.to_vec(),
                named: HashMap::new(),
            });
        }
        let func = self
            .functions
            .get(name)
            .copied()
            .ok_or_else(|| InterpError::Undefined(name.to_string()))?;
        self.call_fn(func, args)
    }

    fn call_fn(&mut self, func: &FnItem, args: &[Value]) -> Result<Value, InterpError> {
        if func.params.len() != args.len() {
            return Err(InterpError::Arity {
                expected: func.params.len(),
                got: args.len(),
            });
        }
        self.env.push();
        for (param, arg) in func.params.iter().zip(args.iter()) {
            self.env.define(&param.name, arg.clone());
        }
        let result = match self.eval_block(&func.body)? {
            Flow::Value(v) | Flow::Return(v) => Ok(v),
            Flow::Break | Flow::Continue => Err(InterpError::Message(
                "break/continue outside of loop".into(),
            )),
        };
        self.env.pop();
        result
    }

    fn eval_block(&mut self, block: &Block) -> Result<Flow, InterpError> {
        self.env.push();
        for stmt in &block.stmts {
            match self.eval_stmt(stmt)? {
                Flow::Return(v) => {
                    self.env.pop();
                    return Ok(Flow::Return(v));
                }
                Flow::Break => {
                    self.env.pop();
                    return Ok(Flow::Break);
                }
                Flow::Continue => {
                    self.env.pop();
                    return Ok(Flow::Continue);
                }
                Flow::Value(_) => {}
            }
        }
        let result = if let Some(expr) = &block.expr {
            self.eval_expr(expr)?
        } else {
            Flow::Value(Value::Unit)
        };
        self.env.pop();
        Ok(result)
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Flow, InterpError> {
        match &stmt.kind {
            StmtKind::Let { name, init, .. } => {
                let value = match init {
                    Some(e) => match self.eval_expr(e)? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => return Ok(Flow::Break),
                        Flow::Continue => return Ok(Flow::Continue),
                        Flow::Value(v) => v,
                    },
                    None => Value::Unit,
                };
                self.env.define(name, value);
                Ok(Flow::Value(Value::Unit))
            }
            StmtKind::While { cond, body } => loop {
                let c = match self.eval_expr(cond)? {
                    Flow::Value(v) => v,
                    Flow::Return(v) => return Ok(Flow::Return(v)),
                    Flow::Break => return Ok(Flow::Break),
                    Flow::Continue => return Ok(Flow::Continue),
                };
                if !c.is_truthy() {
                    return Ok(Flow::Value(Value::Unit));
                }
                match self.eval_block(body)? {
                    Flow::Return(v) => return Ok(Flow::Return(v)),
                    Flow::Break => return Ok(Flow::Value(Value::Unit)),
                    Flow::Continue | Flow::Value(_) => {}
                }
            },
            StmtKind::For { name, iter, body } => {
                let iterable = match self.eval_expr(iter)? {
                    Flow::Value(v) => v,
                    Flow::Return(v) => return Ok(Flow::Return(v)),
                    Flow::Break => return Ok(Flow::Break),
                    Flow::Continue => return Ok(Flow::Continue),
                };
                let items = match iterable {
                    Value::Vec(v) => v,
                    other => {
                        return Err(InterpError::TypeError(format!(
                            "for expects Vec, got {other:?}"
                        )));
                    }
                };
                for item in items {
                    self.env.push();
                    self.env.define(name, item);
                    let flow = self.eval_block(body)?;
                    self.env.pop();
                    match flow {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => break,
                        Flow::Continue | Flow::Value(_) => {}
                    }
                }
                Ok(Flow::Value(Value::Unit))
            }
            StmtKind::Expr(expr) => self.eval_expr(expr),
            StmtKind::Return(value) => {
                let v = match value {
                    Some(e) => match self.eval_expr(e)? {
                        Flow::Return(v) | Flow::Value(v) => v,
                        Flow::Break | Flow::Continue => {
                            return Err(InterpError::Message("invalid flow in return".into()));
                        }
                    },
                    None => Value::Unit,
                };
                Ok(Flow::Return(v))
            }
            StmtKind::Break => Ok(Flow::Break),
            StmtKind::Continue => Ok(Flow::Continue),
            StmtKind::Empty => Ok(Flow::Value(Value::Unit)),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Flow, InterpError> {
        let value = match &expr.kind {
            ExprKind::Literal(lit) => self.eval_lit(lit)?,
            ExprKind::Path(name) => {
                if let Ok(v) = self.env.get(name) {
                    v
                } else if name == "None" {
                    Value::Enum {
                        type_name: "Option".into(),
                        variant: "None".into(),
                        fields: vec![],
                        named: HashMap::new(),
                    }
                } else if self.functions.contains_key(name)
                    || name == "print"
                    || name == "show"
                    || name == "assert"
                    || name == "Some"
                    || name == "Ok"
                    || name == "Err"
                {
                    return Err(InterpError::Message(format!(
                        "cannot use function `{name}` as a value yet"
                    )));
                } else {
                    // Unit enum variant
                    Value::Enum {
                        type_name: String::new(),
                        variant: name.clone(),
                        fields: vec![],
                        named: HashMap::new(),
                    }
                }
            }
            ExprKind::Binary { op, lhs, rhs } => {
                if *op == BinOp::Assign {
                    let right = match self.eval_expr(rhs)? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => return Ok(Flow::Break),
                        Flow::Continue => return Ok(Flow::Continue),
                        Flow::Value(v) => v,
                    };
                    if let ExprKind::Path(name) = &lhs.kind {
                        self.env.assign(name, right)?;
                        Value::Unit
                    } else {
                        return Err(InterpError::TypeError(
                            "assignment target must be a variable".into(),
                        ));
                    }
                } else if matches!(op, BinOp::And | BinOp::Or) {
                    self.eval_logic(*op, lhs, rhs)?
                } else {
                    let left = match self.eval_expr(lhs)? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => return Ok(Flow::Break),
                        Flow::Continue => return Ok(Flow::Continue),
                        Flow::Value(v) => v,
                    };
                    let right = match self.eval_expr(rhs)? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => return Ok(Flow::Break),
                        Flow::Continue => return Ok(Flow::Continue),
                        Flow::Value(v) => v,
                    };
                    self.eval_binop(*op, &left, &right)?
                }
            }
            ExprKind::Unary { op, expr: inner } => {
                let v = match self.eval_expr(inner)? {
                    Flow::Return(v) => return Ok(Flow::Return(v)),
                    Flow::Break => return Ok(Flow::Break),
                    Flow::Continue => return Ok(Flow::Continue),
                    Flow::Value(v) => v,
                };
                self.eval_unary(*op, &v)?
            }
            ExprKind::Call { callee, args } => {
                let name = match &callee.kind {
                    ExprKind::Path(n) => n.clone(),
                    _ => {
                        return Err(InterpError::Message(
                            "only direct function calls are supported".into(),
                        ));
                    }
                };
                let mut arg_vals = Vec::with_capacity(args.len());
                for arg in args {
                    match self.eval_expr(arg)? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => return Ok(Flow::Break),
                        Flow::Continue => return Ok(Flow::Continue),
                        Flow::Value(v) => arg_vals.push(v),
                    }
                }
                self.call_named(&name, &arg_vals)?
            }
            ExprKind::Field { base, field } => {
                let base_v = match self.eval_expr(base)? {
                    Flow::Return(v) => return Ok(Flow::Return(v)),
                    Flow::Break => return Ok(Flow::Break),
                    Flow::Continue => return Ok(Flow::Continue),
                    Flow::Value(v) => v,
                };
                base_v
                    .get_field(field)
                    .cloned()
                    .ok_or_else(|| InterpError::TypeError(format!("no field `{field}`")))?
            }
            ExprKind::StructLit { name, fields } => {
                let mut map = HashMap::new();
                for f in fields {
                    let v = match self.eval_expr(&f.value)? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => return Ok(Flow::Break),
                        Flow::Continue => return Ok(Flow::Continue),
                        Flow::Value(v) => v,
                    };
                    map.insert(f.name.clone(), v);
                }
                Value::Struct {
                    name: name.clone(),
                    fields: map,
                }
            }
            ExprKind::Group(inner) => match self.eval_expr(inner)? {
                Flow::Return(v) => return Ok(Flow::Return(v)),
                Flow::Break => return Ok(Flow::Break),
                Flow::Continue => return Ok(Flow::Continue),
                Flow::Value(v) => v,
            },
            ExprKind::Block(block) => match self.eval_block(block)? {
                Flow::Return(v) => return Ok(Flow::Return(v)),
                Flow::Break => return Ok(Flow::Break),
                Flow::Continue => return Ok(Flow::Continue),
                Flow::Value(v) => v,
            },
            ExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let c = match self.eval_expr(cond)? {
                    Flow::Return(v) => return Ok(Flow::Return(v)),
                    Flow::Break => return Ok(Flow::Break),
                    Flow::Continue => return Ok(Flow::Continue),
                    Flow::Value(v) => v,
                };
                if c.is_truthy() {
                    match self.eval_block(then_branch)? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => return Ok(Flow::Break),
                        Flow::Continue => return Ok(Flow::Continue),
                        Flow::Value(v) => v,
                    }
                } else if let Some(els) = else_branch {
                    match self.eval_expr(els)? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => return Ok(Flow::Break),
                        Flow::Continue => return Ok(Flow::Continue),
                        Flow::Value(v) => v,
                    }
                } else {
                    Value::Unit
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                let scrut = match self.eval_expr(scrutinee)? {
                    Flow::Return(v) => return Ok(Flow::Return(v)),
                    Flow::Break => return Ok(Flow::Break),
                    Flow::Continue => return Ok(Flow::Continue),
                    Flow::Value(v) => v,
                };
                for arm in arms {
                    if let Some(bindings) = match_pattern(&arm.pattern, &scrut) {
                        self.env.push();
                        for (k, v) in bindings {
                            self.env.define(k, v);
                        }
                        let flow = self.eval_expr(&arm.body)?;
                        self.env.pop();
                        return Ok(flow);
                    }
                }
                return Err(InterpError::Message("non-exhaustive match".into()));
            }
            ExprKind::Array(elems) => {
                let mut out = Vec::with_capacity(elems.len());
                for e in elems {
                    match self.eval_expr(e)? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => return Ok(Flow::Break),
                        Flow::Continue => return Ok(Flow::Continue),
                        Flow::Value(v) => out.push(v),
                    }
                }
                Value::Vec(out)
            }
            ExprKind::Error => {
                return Err(InterpError::Message("evaluating error expression".into()));
            }
        };
        Ok(Flow::Value(value))
    }

    fn eval_lit(&self, lit: &Lit) -> Result<Value, InterpError> {
        Ok(match lit {
            Lit::Int(s) => {
                let cleaned: String = s.chars().filter(|c| *c != '_').collect();
                let n = if let Some(rest) = cleaned
                    .strip_prefix("0x")
                    .or_else(|| cleaned.strip_prefix("0X"))
                {
                    i64::from_str_radix(rest, 16)
                } else if let Some(rest) = cleaned
                    .strip_prefix("0b")
                    .or_else(|| cleaned.strip_prefix("0B"))
                {
                    i64::from_str_radix(rest, 2)
                } else if let Some(rest) = cleaned
                    .strip_prefix("0o")
                    .or_else(|| cleaned.strip_prefix("0O"))
                {
                    i64::from_str_radix(rest, 8)
                } else {
                    cleaned.parse::<i64>()
                }
                .map_err(|_| InterpError::Message(format!("invalid integer `{s}`")))?;
                Value::Int(n)
            }
            Lit::Float(s) => {
                let cleaned: String = s.chars().filter(|c| *c != '_').collect();
                let n = cleaned
                    .parse::<f64>()
                    .map_err(|_| InterpError::Message(format!("invalid float `{s}`")))?;
                Value::Float(n)
            }
            Lit::Str(s) => Value::String(unescape_string(s)),
            Lit::Char(s) => Value::Char(unescape_char(s)?),
            Lit::Bool(b) => Value::Bool(*b),
        })
    }

    fn eval_logic(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr) -> Result<Value, InterpError> {
        let left = match self.eval_expr(lhs)? {
            Flow::Value(v) => v,
            _ => return Err(InterpError::Message("unexpected control flow".into())),
        };
        match op {
            BinOp::And => {
                if !left.is_truthy() {
                    Ok(Value::Bool(false))
                } else {
                    let right = match self.eval_expr(rhs)? {
                        Flow::Value(v) => v,
                        _ => return Err(InterpError::Message("unexpected control flow".into())),
                    };
                    Ok(Value::Bool(right.is_truthy()))
                }
            }
            BinOp::Or => {
                if left.is_truthy() {
                    Ok(Value::Bool(true))
                } else {
                    let right = match self.eval_expr(rhs)? {
                        Flow::Value(v) => v,
                        _ => return Err(InterpError::Message("unexpected control flow".into())),
                    };
                    Ok(Value::Bool(right.is_truthy()))
                }
            }
            _ => unreachable!(),
        }
    }

    fn eval_binop(&self, op: BinOp, left: &Value, right: &Value) -> Result<Value, InterpError> {
        match (op, left, right) {
            (BinOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (BinOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (BinOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (BinOp::Div, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err(InterpError::Message("division by zero".into()));
                }
                Ok(Value::Int(a / b))
            }
            (BinOp::Rem, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err(InterpError::Message("division by zero".into()));
                }
                Ok(Value::Int(a % b))
            }
            (BinOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (BinOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (BinOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (BinOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (BinOp::Add, Value::String(a), Value::String(b)) => {
                Ok(Value::String(format!("{a}{b}")))
            }
            (BinOp::Eq, a, b) => Ok(Value::Bool(a == b)),
            (BinOp::Ne, a, b) => Ok(Value::Bool(a != b)),
            (BinOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (BinOp::Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (BinOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (BinOp::Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(InterpError::TypeError(format!(
                "cannot apply {op:?} to {left:?} and {right:?}"
            ))),
        }
    }

    fn eval_unary(&self, op: UnaryOp, v: &Value) -> Result<Value, InterpError> {
        match (op, v) {
            (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
            (UnaryOp::Neg, Value::Float(n)) => Ok(Value::Float(-n)),
            (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            _ => Err(InterpError::TypeError(format!(
                "cannot apply {op:?} to {v:?}"
            ))),
        }
    }

    fn builtin_print(&mut self, args: &[Value]) -> Result<Value, InterpError> {
        if args.len() != 1 {
            return Err(InterpError::Arity {
                expected: 1,
                got: args.len(),
            });
        }
        writeln!(self.stdout, "{}", args[0])
            .map_err(|e| InterpError::Message(format!("io error: {e}")))?;
        Ok(Value::Unit)
    }

    fn builtin_assert(&mut self, args: &[Value]) -> Result<Value, InterpError> {
        if args.len() != 1 {
            return Err(InterpError::Arity {
                expected: 1,
                got: args.len(),
            });
        }
        if args[0].is_truthy() {
            Ok(Value::Unit)
        } else {
            Err(InterpError::AssertFailed)
        }
    }
}

fn match_pattern(pattern: &Pattern, value: &Value) -> Option<HashMap<String, Value>> {
    let mut bindings = HashMap::new();
    if match_pattern_into(pattern, value, &mut bindings) {
        Some(bindings)
    } else {
        None
    }
}

fn match_pattern_into(
    pattern: &Pattern,
    value: &Value,
    bindings: &mut HashMap<String, Value>,
) -> bool {
    match pattern {
        Pattern::Wildcard => true,
        Pattern::Ident(name) => {
            bindings.insert(name.clone(), value.clone());
            true
        }
        Pattern::Lit(lit) => {
            let lit_v = match lit {
                Lit::Int(s) => Value::Int(s.replace('_', "").parse().unwrap_or(0)),
                Lit::Float(s) => Value::Float(s.replace('_', "").parse().unwrap_or(0.0)),
                Lit::Str(s) => Value::String(s.clone()),
                Lit::Char(s) => Value::Char(s.chars().next().unwrap_or('\0')),
                Lit::Bool(b) => Value::Bool(*b),
            };
            &lit_v == value
        }
        Pattern::Variant { path, kind } => {
            let short = path.rsplit("::").next().unwrap_or(path);
            match value {
                Value::Enum {
                    variant,
                    fields,
                    named,
                    ..
                } => {
                    if variant != short {
                        return false;
                    }
                    match kind {
                        PatternVariantKind::Unit => fields.is_empty() && named.is_empty(),
                        PatternVariantKind::Tuple(pats) => {
                            if pats.len() != fields.len() {
                                return false;
                            }
                            for (p, v) in pats.iter().zip(fields.iter()) {
                                if !match_pattern_into(p, v, bindings) {
                                    return false;
                                }
                            }
                            true
                        }
                        PatternVariantKind::Struct(fpats) => {
                            for (fname, p) in fpats {
                                let Some(v) = named.get(fname) else {
                                    return false;
                                };
                                if !match_pattern_into(p, v, bindings) {
                                    return false;
                                }
                            }
                            true
                        }
                    }
                }
                _ => false,
            }
        }
    }
}

fn unescape_string(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn unescape_char(s: &str) -> Result<char, InterpError> {
    let mut chars = s.chars();
    match chars.next() {
        Some('\\') => match chars.next() {
            Some('n') => Ok('\n'),
            Some('t') => Ok('\t'),
            Some('r') => Ok('\r'),
            Some('\\') => Ok('\\'),
            Some('\'') => Ok('\''),
            Some(c) => Ok(c),
            None => Err(InterpError::Message("empty char escape".into())),
        },
        Some(c) => Ok(c),
        None => Err(InterpError::Message("empty char literal".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxa_diagnostics::DiagnosticBag;
    use foxa_lexer::Lexer;
    use foxa_parser::Parser;
    use foxa_resolve::Resolver;
    use foxa_span::SourceMap;
    use foxa_types::TypeChecker;

    fn run(src: &str) -> Result<String, InterpError> {
        let mut map = SourceMap::new();
        let file = map.add_file("t.foxa", src);
        let mut bag = DiagnosticBag::new();
        let tokens = Lexer::new(file, src, &mut bag).tokenize_all();
        let module = Parser::new(file, src, tokens, &mut bag).parse_module();
        let resolved = Resolver::new(&mut bag).resolve(&module);
        TypeChecker::new(&resolved, &mut bag).check(&module);
        assert!(!bag.has_errors(), "{:?}", bag.items());
        let mut buf = Vec::new();
        {
            let mut interp = Interpreter::with_stdout(&module, Box::new(&mut buf));
            interp.run_main()?;
        }
        Ok(String::from_utf8(buf).unwrap())
    }

    #[test]
    fn runs_hello() {
        let out = run(r#"fn main() { print("Hello, Foxa!"); }"#).unwrap();
        assert_eq!(out, "Hello, Foxa!\n");
    }

    #[test]
    fn runs_arithmetic() {
        let out = run(r#"
            fn main() { print(add(20, 22)); }
            fn add(a: Int, b: Int) -> Int { a + b }
            "#)
        .unwrap();
        assert_eq!(out, "42\n");
    }

    #[test]
    fn runs_if() {
        let out = run(r#"
            fn main() {
                if true { print("yes"); } else { print("no"); }
            }
            "#)
        .unwrap();
        assert_eq!(out, "yes\n");
    }

    #[test]
    fn runs_while() {
        let out = run(r#"
            fn main() {
                let mut i = 0;
                while i < 3 {
                    print(i);
                    i = i + 1;
                }
            }
            "#)
        .unwrap();
        assert_eq!(out, "0\n1\n2\n");
    }

    #[test]
    fn runs_for_and_struct() {
        let out = run(r#"
            struct Point { x: Int, y: Int }
            fn main() {
                let p = Point { x: 10, y: 20 };
                print(p.x);
                for n in [1, 2] {
                    print(n);
                }
            }
            "#)
        .unwrap();
        assert_eq!(out, "10\n1\n2\n");
    }

    #[test]
    fn runs_match_option() {
        let out = run(r#"
            fn main() {
                match Some(7) {
                    Some(x) => print(x),
                    None => print(0),
                }
            }
            "#)
        .unwrap();
        assert_eq!(out, "7\n");
    }

    #[test]
    fn runs_break() {
        let out = run(r#"
            fn main() {
                let mut i = 0;
                while true {
                    if i == 2 {
                        break;
                    }
                    print(i);
                    i = i + 1;
                }
            }
            "#)
        .unwrap();
        assert_eq!(out, "0\n1\n");
    }
}
