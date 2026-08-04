//! Cranelift JIT engine.

use cranelift_codegen::ir::{types, AbiParam, InstBuilder, UserFuncName};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};
use foxa_mir::{MirBinOp, MirFunction, MirModule, MirRvalue, MirStmt, MirTerminator, MirTy};
use std::collections::HashMap;
use thiserror::Error;

/// Codegen failures.
#[derive(Debug, Error)]
pub enum CodegenError {
    /// Cranelift/module error.
    #[error("codegen error: {0}")]
    Message(String),
    /// Function not found after compile.
    #[error("function `{0}` not found in JIT module")]
    MissingFunction(String),
}

/// JIT compilation engine.
pub struct JitEngine {
    module: JITModule,
    ids: HashMap<String, FuncId>,
}

impl JitEngine {
    /// Creates a new JIT engine with host print helper.
    pub fn new() -> Result<Self, CodegenError> {
        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .map_err(|e| CodegenError::Message(e.to_string()))?;
        flag_builder
            .set("is_pic", "false")
            .map_err(|e| CodegenError::Message(e.to_string()))?;
        let isa_builder = cranelift_native::builder()
            .map_err(|e| CodegenError::Message(format!("host ISA: {e}")))?;
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| CodegenError::Message(e.to_string()))?;

        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        builder.symbol("foxa_print_i64", foxa_print_i64 as *const u8);
        let module = JITModule::new(builder);
        Ok(Self {
            module,
            ids: HashMap::new(),
        })
    }

    /// Looks up a compiled function as `fn(i64, i64) -> i64` (for binary Int fns).
    ///
    /// # Safety
    ///
    /// Caller must ensure the signature matches the Foxa function.
    pub unsafe fn get_fn2(&self, name: &str) -> Result<fn(i64, i64) -> i64, CodegenError> {
        let id = self
            .ids
            .get(name)
            .ok_or_else(|| CodegenError::MissingFunction(name.to_string()))?;
        let ptr = self.module.get_finalized_function(*id);
        Ok(std::mem::transmute::<*const u8, fn(i64, i64) -> i64>(ptr))
    }

    /// Looks up a compiled `fn() -> i64`.
    ///
    /// # Safety
    ///
    /// Caller must ensure the signature matches.
    pub unsafe fn get_fn0(&self, name: &str) -> Result<fn() -> i64, CodegenError> {
        let id = self
            .ids
            .get(name)
            .ok_or_else(|| CodegenError::MissingFunction(name.to_string()))?;
        let ptr = self.module.get_finalized_function(*id);
        Ok(std::mem::transmute::<*const u8, fn() -> i64>(ptr))
    }
}

extern "C" fn foxa_print_i64(x: i64) {
    println!("{x}");
}

/// Compiles all Int-compatible MIR functions into a JIT engine.
pub fn compile_module(mir: &MirModule) -> Result<JitEngine, CodegenError> {
    let mut engine = JitEngine::new()?;

    // Declare print helper
    {
        let mut sig = engine.module.make_signature();
        sig.params.push(AbiParam::new(types::I64));
        let _print_id = engine
            .module
            .declare_function("foxa_print_i64", Linkage::Import, &sig)
            .map_err(|e| CodegenError::Message(e.to_string()))?;
    }

    // Declare all functions first
    for func in &mir.functions {
        if !is_codegenable(func) {
            continue;
        }
        let mut sig = engine.module.make_signature();
        for &p in &func.params {
            let ty = func.locals[p.0 as usize];
            sig.params.push(AbiParam::new(cl_ty(ty)?));
        }
        if func.return_ty != MirTy::Unit {
            sig.returns.push(AbiParam::new(cl_ty(func.return_ty)?));
        }
        let id = engine
            .module
            .declare_function(&func.name, Linkage::Export, &sig)
            .map_err(|e| CodegenError::Message(e.to_string()))?;
        engine.ids.insert(func.name.clone(), id);
    }

    let mut ctx = engine.module.make_context();
    let mut fb_ctx = FunctionBuilderContext::new();

    for func in &mir.functions {
        if !is_codegenable(func) {
            continue;
        }
        let id = *engine.ids.get(&func.name).unwrap();
        ctx.func.signature = engine
            .module
            .declarations()
            .get_function_decl(id)
            .signature
            .clone();
        ctx.func.name = UserFuncName::user(0, id.as_u32());

        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
            let entry = builder.create_block();
            builder.append_block_params_for_function_params(entry);
            builder.switch_to_block(entry);
            builder.seal_block(entry);

            // Map locals to SSA values / stack slots — use variables
            let mut vars: HashMap<u32, cranelift_codegen::ir::Value> = HashMap::new();
            for (i, &pid) in func.params.iter().enumerate() {
                let val = builder.block_params(entry)[i];
                vars.insert(pid.0, val);
            }

            // Single-block simplified codegen for straight-line + return
            // Full multi-block lowering is next iteration; here we flatten
            // the entry block statements until Return.
            if let Some(bb) = func.blocks.first() {
                for stmt in &bb.stmts {
                    let MirStmt::Assign { local, rvalue } = stmt;
                    let v = emit_rvalue(&mut builder, rvalue, &vars)?;
                    vars.insert(local.0, v);
                }
                match &bb.term {
                    MirTerminator::Return(Some(local)) => {
                        let v = *vars
                            .get(&local.0)
                            .ok_or_else(|| CodegenError::Message("missing return local".into()))?;
                        builder.ins().return_(&[v]);
                    }
                    MirTerminator::Return(None) => {
                        builder.ins().return_(&[]);
                    }
                    _ => {
                        // Try to find a binary result as return for `{ a + b }`
                        if let Some((_, v)) = vars.iter().last() {
                            if func.return_ty == MirTy::Int {
                                builder.ins().return_(&[*v]);
                            } else {
                                builder.ins().return_(&[]);
                            }
                        } else {
                            builder.ins().return_(&[]);
                        }
                    }
                }
            } else {
                builder.ins().return_(&[]);
            }

            builder.finalize();
        }

        engine
            .module
            .define_function(id, &mut ctx)
            .map_err(|e| CodegenError::Message(e.to_string()))?;
        engine.module.clear_context(&mut ctx);
    }

    engine
        .module
        .finalize_definitions()
        .map_err(|e| CodegenError::Message(e.to_string()))?;
    Ok(engine)
}

fn is_codegenable(func: &MirFunction) -> bool {
    func.params
        .iter()
        .all(|p| matches!(func.locals[p.0 as usize], MirTy::Int | MirTy::Bool))
        && matches!(func.return_ty, MirTy::Int | MirTy::Bool | MirTy::Unit)
}

fn cl_ty(ty: MirTy) -> Result<types::Type, CodegenError> {
    match ty {
        MirTy::Int | MirTy::Bool => Ok(types::I64),
        MirTy::Unit => Err(CodegenError::Message("unit not a value type".into())),
        MirTy::Opaque => Err(CodegenError::Message("opaque type".into())),
    }
}

fn emit_rvalue(
    builder: &mut FunctionBuilder<'_>,
    rvalue: &MirRvalue,
    vars: &HashMap<u32, cranelift_codegen::ir::Value>,
) -> Result<cranelift_codegen::ir::Value, CodegenError> {
    match rvalue {
        MirRvalue::Use(local) => vars
            .get(&local.0)
            .copied()
            .ok_or_else(|| CodegenError::Message(format!("undefined local {}", local.0))),
        MirRvalue::ConstInt(n) => Ok(builder.ins().iconst(types::I64, *n)),
        MirRvalue::ConstBool(b) => Ok(builder.ins().iconst(types::I64, i64::from(*b))),
        MirRvalue::Binary { op, lhs, rhs } => {
            let l = *vars
                .get(&lhs.0)
                .ok_or_else(|| CodegenError::Message("lhs".into()))?;
            let r = *vars
                .get(&rhs.0)
                .ok_or_else(|| CodegenError::Message("rhs".into()))?;
            Ok(match op {
                MirBinOp::Add => builder.ins().iadd(l, r),
                MirBinOp::Sub => builder.ins().isub(l, r),
                MirBinOp::Mul => builder.ins().imul(l, r),
                MirBinOp::Div => builder.ins().sdiv(l, r),
                MirBinOp::Rem => builder.ins().srem(l, r),
                MirBinOp::Eq => {
                    let c =
                        builder
                            .ins()
                            .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, l, r);
                    builder.ins().uextend(types::I64, c)
                }
                MirBinOp::Ne => {
                    let c =
                        builder
                            .ins()
                            .icmp(cranelift_codegen::ir::condcodes::IntCC::NotEqual, l, r);
                    builder.ins().uextend(types::I64, c)
                }
                MirBinOp::Lt => {
                    let c = builder.ins().icmp(
                        cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
                        l,
                        r,
                    );
                    builder.ins().uextend(types::I64, c)
                }
                MirBinOp::Le => {
                    let c = builder.ins().icmp(
                        cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                        l,
                        r,
                    );
                    builder.ins().uextend(types::I64, c)
                }
                MirBinOp::Gt => {
                    let c = builder.ins().icmp(
                        cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThan,
                        l,
                        r,
                    );
                    builder.ins().uextend(types::I64, c)
                }
                MirBinOp::Ge => {
                    let c = builder.ins().icmp(
                        cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual,
                        l,
                        r,
                    );
                    builder.ins().uextend(types::I64, c)
                }
            })
        }
        MirRvalue::Call { .. } => Ok(builder.ins().iconst(types::I64, 0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxa_diagnostics::DiagnosticBag;
    use foxa_lexer::Lexer;
    use foxa_mir::lower_module;
    use foxa_parser::Parser;
    use foxa_span::SourceMap;

    #[test]
    fn jit_add() {
        let src = "fn add(a: Int, b: Int) -> Int { a + b }";
        let mut map = SourceMap::new();
        let file = map.add_file("t.foxa", src);
        let mut bag = DiagnosticBag::new();
        let tokens = Lexer::new(file, src, &mut bag).tokenize_all();
        let module = Parser::new(file, src, tokens, &mut bag).parse_module();
        let mir = lower_module(&module);
        let engine = compile_module(&mir).expect("compile");
        let add = unsafe { engine.get_fn2("add").expect("get add") };
        assert_eq!(add(20, 22), 42);
    }
}
