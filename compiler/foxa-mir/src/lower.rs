//! AST → MIR lowering (Int/Bool subset with if/return/calls).

use crate::ir::{
    BasicBlock, BlockId, LocalId, MirBinOp, MirFunction, MirModule, MirRvalue, MirStmt,
    MirTerminator, MirTy,
};
use foxa_ast::{BinOp, Block, Expr, ExprKind, FnItem, ItemKind, Lit, Module, StmtKind};
use std::collections::HashMap;

/// Lowers a Foxa module to MIR.
///
/// Supports: Int/Bool locals, arithmetic/compare, if/else, return, direct calls,
/// and simple lets. Unsupported constructs become Opaque locals / Unreachable.
#[must_use]
pub fn lower_module(module: &Module) -> MirModule {
    let mut mir = MirModule::default();
    for item in &module.items {
        if let ItemKind::Fn(func) = &item.kind {
            mir.functions.push(lower_function(func));
        }
    }
    mir
}

fn lower_function(func: &FnItem) -> MirFunction {
    let mut cx = LowerCx::new();
    let mut params = Vec::new();
    for p in &func.params {
        let ty = mir_ty_from_name(&p.ty);
        let id = cx.alloc_local(ty);
        cx.names.insert(p.name.clone(), id);
        params.push(id);
    }
    let ret_ty = func
        .return_ty
        .as_deref()
        .map(mir_ty_from_name)
        .unwrap_or(MirTy::Unit);

    let entry = cx.new_block();
    let exit = cx.lower_block(func.body.clone(), entry);
    if matches!(cx.blocks[exit.0 as usize].term, MirTerminator::Unreachable) {
        if let Some(tail) = cx.names.get("__tail").copied() {
            cx.blocks[exit.0 as usize].term = MirTerminator::Return(Some(tail));
        } else {
            cx.blocks[exit.0 as usize].term = MirTerminator::Return(None);
        }
    }

    MirFunction {
        name: func.name.clone(),
        params,
        locals: cx.locals,
        blocks: cx.blocks,
        return_ty: ret_ty,
    }
}

struct LowerCx {
    locals: Vec<MirTy>,
    blocks: Vec<BasicBlock>,
    names: HashMap<String, LocalId>,
}

impl LowerCx {
    fn new() -> Self {
        Self {
            locals: Vec::new(),
            blocks: Vec::new(),
            names: HashMap::new(),
        }
    }

    fn alloc_local(&mut self, ty: MirTy) -> LocalId {
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(ty);
        id
    }

    fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len() as u32);
        self.blocks.push(BasicBlock::default());
        id
    }

    fn lower_block(&mut self, block: Block, mut current: BlockId) -> BlockId {
        for stmt in &block.stmts {
            current = self.lower_stmt(stmt, current);
        }
        if let Some(expr) = &block.expr {
            let (val, b) = self.lower_expr(expr, current);
            // Store as implicit return candidate — caller may set terminator
            let tmp = self.alloc_local(MirTy::Opaque);
            self.blocks[b.0 as usize].stmts.push(MirStmt::Assign {
                local: tmp,
                rvalue: MirRvalue::Use(val),
            });
            // Keep value in a well-known local by rebinding — for return use val
            let _ = tmp;
            // Replace: set names["__tail"] = val for return helper
            self.names.insert("__tail".into(), val);
            return b;
        }
        current
    }

    fn lower_stmt(&mut self, stmt: &foxa_ast::Stmt, current: BlockId) -> BlockId {
        match &stmt.kind {
            StmtKind::Let { name, init, ty, .. } => {
                let ty = ty.as_deref().map(mir_ty_from_name).unwrap_or(MirTy::Opaque);
                let local = self.alloc_local(ty);
                self.names.insert(name.clone(), local);
                if let Some(init) = init {
                    let (val, b) = self.lower_expr(init, current);
                    self.blocks[b.0 as usize].stmts.push(MirStmt::Assign {
                        local,
                        rvalue: MirRvalue::Use(val),
                    });
                    return b;
                }
                current
            }
            StmtKind::Return(value) => {
                if let Some(expr) = value {
                    let (val, b) = self.lower_expr(expr, current);
                    self.blocks[b.0 as usize].term = MirTerminator::Return(Some(val));
                    // Continue in a fresh dead block
                    self.new_block()
                } else {
                    self.blocks[current.0 as usize].term = MirTerminator::Return(None);
                    self.new_block()
                }
            }
            StmtKind::Expr(expr) => {
                let (_v, b) = self.lower_expr(expr, current);
                b
            }
            StmtKind::While { cond, body } => {
                let header = self.new_block();
                let body_bb = self.new_block();
                let exit = self.new_block();
                self.blocks[current.0 as usize].term = MirTerminator::Goto(header);
                let (cval, hb) = self.lower_expr(cond, header);
                // If cond lowering created new blocks, terminate the last
                self.blocks[hb.0 as usize].term = MirTerminator::Switch {
                    cond: cval,
                    true_block: body_bb,
                    false_block: exit,
                };
                let after_body = self.lower_block(body.clone(), body_bb);
                if matches!(
                    self.blocks[after_body.0 as usize].term,
                    MirTerminator::Unreachable
                ) {
                    self.blocks[after_body.0 as usize].term = MirTerminator::Goto(header);
                }
                exit
            }
            StmtKind::Empty | StmtKind::Break | StmtKind::Continue | StmtKind::For { .. } => {
                current
            }
        }
    }

    fn lower_expr(&mut self, expr: &Expr, current: BlockId) -> (LocalId, BlockId) {
        match &expr.kind {
            ExprKind::Literal(Lit::Int(s)) => {
                let n = s.replace('_', "").parse::<i64>().unwrap_or(0);
                let local = self.alloc_local(MirTy::Int);
                self.blocks[current.0 as usize].stmts.push(MirStmt::Assign {
                    local,
                    rvalue: MirRvalue::ConstInt(n),
                });
                (local, current)
            }
            ExprKind::Literal(Lit::Bool(b)) => {
                let local = self.alloc_local(MirTy::Bool);
                self.blocks[current.0 as usize].stmts.push(MirStmt::Assign {
                    local,
                    rvalue: MirRvalue::ConstBool(*b),
                });
                (local, current)
            }
            ExprKind::Path(name) => {
                if let Some(id) = self.names.get(name).copied() {
                    (id, current)
                } else {
                    let local = self.alloc_local(MirTy::Opaque);
                    (local, current)
                }
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let (l, b1) = self.lower_expr(lhs, current);
                let (r, b2) = self.lower_expr(rhs, b1);
                let ty = match op {
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        MirTy::Bool
                    }
                    _ => MirTy::Int,
                };
                let dest = self.alloc_local(ty);
                if let Some(mop) = binop(*op) {
                    self.blocks[b2.0 as usize].stmts.push(MirStmt::Assign {
                        local: dest,
                        rvalue: MirRvalue::Binary {
                            op: mop,
                            lhs: l,
                            rhs: r,
                        },
                    });
                }
                (dest, b2)
            }
            ExprKind::Call { callee, args } => {
                let name = match &callee.kind {
                    ExprKind::Path(n) => n.clone(),
                    _ => "<unknown>".into(),
                };
                let mut arg_ids = Vec::new();
                let mut b = current;
                for a in args {
                    let (id, nb) = self.lower_expr(a, b);
                    arg_ids.push(id);
                    b = nb;
                }
                let dest = self.alloc_local(MirTy::Int);
                self.blocks[b.0 as usize].stmts.push(MirStmt::Assign {
                    local: dest,
                    rvalue: MirRvalue::Call {
                        callee: name,
                        args: arg_ids,
                    },
                });
                (dest, b)
            }
            ExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let (cval, after_cond) = self.lower_expr(cond, current);
                let then_bb = self.new_block();
                let else_bb = self.new_block();
                let join = self.new_block();
                self.blocks[after_cond.0 as usize].term = MirTerminator::Switch {
                    cond: cval,
                    true_block: then_bb,
                    false_block: else_bb,
                };
                let then_end = self.lower_block(then_branch.clone(), then_bb);
                let then_val = self
                    .names
                    .get("__tail")
                    .copied()
                    .unwrap_or_else(|| self.alloc_local(MirTy::Unit));
                if matches!(
                    self.blocks[then_end.0 as usize].term,
                    MirTerminator::Unreachable
                ) {
                    self.blocks[then_end.0 as usize].term = MirTerminator::Goto(join);
                }
                let else_val = if let Some(els) = else_branch {
                    let (v, end) = self.lower_expr(els, else_bb);
                    if matches!(self.blocks[end.0 as usize].term, MirTerminator::Unreachable) {
                        self.blocks[end.0 as usize].term = MirTerminator::Goto(join);
                    }
                    v
                } else {
                    if matches!(
                        self.blocks[else_bb.0 as usize].term,
                        MirTerminator::Unreachable
                    ) {
                        self.blocks[else_bb.0 as usize].term = MirTerminator::Goto(join);
                    }
                    self.alloc_local(MirTy::Unit)
                };
                // Phi-like: pick then_val into result (simplified — uses then)
                let result = self.alloc_local(MirTy::Int);
                self.blocks[join.0 as usize].stmts.push(MirStmt::Assign {
                    local: result,
                    rvalue: MirRvalue::Use(then_val),
                });
                let _ = else_val;
                (result, join)
            }
            ExprKind::Group(inner) => self.lower_expr(inner, current),
            ExprKind::Block(block) => {
                let end = self.lower_block(block.clone(), current);
                let val = self
                    .names
                    .get("__tail")
                    .copied()
                    .unwrap_or_else(|| self.alloc_local(MirTy::Unit));
                (val, end)
            }
            _ => {
                let local = self.alloc_local(MirTy::Opaque);
                (local, current)
            }
        }
    }
}

fn mir_ty_from_name(name: &str) -> MirTy {
    match name {
        "Int" => MirTy::Int,
        "Bool" => MirTy::Bool,
        "Unit" | "()" => MirTy::Unit,
        _ => MirTy::Opaque,
    }
}

fn binop(op: BinOp) -> Option<MirBinOp> {
    Some(match op {
        BinOp::Add => MirBinOp::Add,
        BinOp::Sub => MirBinOp::Sub,
        BinOp::Mul => MirBinOp::Mul,
        BinOp::Div => MirBinOp::Div,
        BinOp::Rem => MirBinOp::Rem,
        BinOp::Eq => MirBinOp::Eq,
        BinOp::Ne => MirBinOp::Ne,
        BinOp::Lt => MirBinOp::Lt,
        BinOp::Le => MirBinOp::Le,
        BinOp::Gt => MirBinOp::Gt,
        BinOp::Ge => MirBinOp::Ge,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxa_diagnostics::DiagnosticBag;
    use foxa_lexer::Lexer;
    use foxa_parser::Parser;
    use foxa_span::SourceMap;

    #[test]
    fn lowers_add() {
        let src = "fn add(a: Int, b: Int) -> Int { a + b }";
        let mut map = SourceMap::new();
        let file = map.add_file("t.foxa", src);
        let mut bag = DiagnosticBag::new();
        let tokens = Lexer::new(file, src, &mut bag).tokenize_all();
        let module = Parser::new(file, src, tokens, &mut bag).parse_module();
        let mir = lower_module(&module);
        assert_eq!(mir.functions.len(), 1);
        assert_eq!(mir.functions[0].name, "add");
        assert_eq!(mir.functions[0].params.len(), 2);
        assert!(!mir.functions[0].blocks.is_empty());
    }
}
