# Compiler Architecture

## Pipeline

```
Source text
    │
    ▼
┌─────────┐
│  Lexer  │  foxa-lexer — tokens + spans
└────┬────┘
     ▼
┌─────────┐
│ Parser  │  foxa-parser — Pratt exprs + recursive-descent items
└────┬────┘
     ▼
┌─────────┐
│   AST   │  foxa-ast
└────┬────┘
     ▼
┌──────────────────┐
│ Name resolution  │  foxa-resolve — scopes, symbols, builtins
└────────┬─────────┘
         ▼
┌──────────────────┐
│  Type checker    │  foxa-types — monomorphic (generics later)
└────────┬─────────┘
         ▼
┌──────────────────┐
│   Interpreter    │  foxa-interp — tree-walk (MIR/codegen next)
└──────────────────┘
         │
         ▼ (planned)
┌──────────────────┐
│ Borrow / ARC     │
└────────┬─────────┘
         ▼
┌──────────────────┐
│       MIR        │  — SSA-friendly mid IR
└────────┬─────────┘
         ▼
┌──────────────────┐
│   Optimizer      │
└────────┬─────────┘
         ▼
┌──────────────────┐
│    Codegen       │  — Cranelift → LLVM
└──────────────────┘
```

## Crates

| Crate | Role |
|-------|------|
| `foxa-span` | File IDs, byte spans, source map, line index |
| `foxa-diagnostics` | Errors/warnings with labels and help text |
| `foxa-lexer` | Tokenizer |
| `foxa-ast` | Syntax tree |
| `foxa-parser` | Parser |
| `foxa-resolve` | Scopes, symbol table, name resolution |
| `foxa-types` | Monomorphic type checker |
| `foxa-interp` | Tree-walk interpreter (`foxa run`) |
| `foxac` | CLI driver (`foxa`) |

Cross-cutting rules:

- **No global mutable state** — compilation state is passed explicitly.
- **Spans everywhere** — every AST node and diagnostic carries a span.
- **Error recovery** — lexer/parser continue after errors to report more issues.
- **SOLID boundaries** — each crate has a single responsibility and documented public API.

## Diagnostics

Diagnostics use a rustc-inspired model:

- Severity: error / warning / note / help
- Primary + secondary labels with spans
- Optional codes (`E0001`, …)
- Rendered via `Emitter` against a `SourceMap`

## Incremental compilation (planned)

Will use content-hashed source fingerprints and a query-style dependency graph
(similar to rustc/`salsa`) so unchanged modules are not re-typechecked.
