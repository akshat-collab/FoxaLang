# Foxa Roadmap

## Phase 0 — Foundation ✅

- [x] Cargo workspace and crate layout
- [x] Design documentation
- [x] CLI skeleton (`foxa`)
- [x] CI workflow

## Phase 1 — Frontend ✅ (current)

- [x] `foxa-span` — source map, spans, line index
- [x] `foxa-diagnostics` — structured errors + emitter
- [x] `foxa-lexer` — full token set for v0.1
- [x] `foxa-ast` — minimal AST
- [x] `foxa-parser` — items, stmts, Pratt expressions
- [x] `foxa check` / `lex` / `parse`

## Phase 2 — Semantic analysis ✅

- [x] Symbol tables and scopes (`foxa-resolve`)
- [x] Name resolution with builtins
- [x] Type checker (monomorphic primitives + functions)
- [ ] Basic `Option` / `Result` in std prelude

## Phase 3 — Executable ✅

- [x] Tree-walk interpreter (`foxa run`)
- [x] Core builtins: print, assert, Option, Result, Vec literals, String

## Phase 4 — Language surface ✅

- [x] `while` / `for` / `break` / `continue`
- [x] `struct` / `enum` + field access + struct literals
- [x] `match` with variant patterns
- [x] Array literals `[...]`

## Phase 5 — MIR + native ✅ (initial)

- [x] MIR lowering (`foxa mir`)
- [x] Cranelift JIT for Int functions (`foxa jit`)
- [ ] Multi-block Cranelift + object emission
- [ ] LLVM AOT backend

## Phase 6 — Package manager ✅ (initial)

- [x] `Foxa.toml` parse / init / add
- [x] Offline + path dependency resolve (`foxa-pkg resolve`)
- [ ] Registry fetch / publish

## Phase 7 — Runtime & concurrency

- [ ] Allocator
- [ ] Async runtime
- [ ] Thread scheduler

## Phase 8 — Full standard library

Collections methods, I/O, networking, serialization, crypto, testing, etc.

## Phase 9 — Tooling

- [ ] Formatter & linter
- [ ] LSP
- [ ] Documentation generator

