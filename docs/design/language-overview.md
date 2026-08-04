# Foxa Language Overview

## Goals

Foxa aims to compete with Rust, Go, Swift, Kotlin, and C# on architecture and
developer experience:

- **Memory-safe by default** with an unsafe escape hatch
- **Predictable performance** suitable for systems and services
- **Fast feedback** via incremental compilation and rich diagnostics
- **Batteries-included toolchain** (pkg, fmt, lint, doc, LSP, REPL)

## Memory model (v1)

Hybrid approach:

1. **Unique ownership** — values move by default; no implicit deep copies of
   heap structures.
2. **ARC (`shared T`)** — explicitly shared, thread-safe reference counting.
3. **Borrowing** — shared (`&T`) and exclusive (`&mut T`) references with
   lexical lifetimes initially; full borrow checker refinements in later phases.
4. **No tracing GC in v1** — keeps latency predictable and systems credibility.

## Type system

- Static, nominative typing with local inference
- Algebraic data types: `struct`, `enum` with payloads
- Generics + traits (interfaces) with where-clauses
- Null safety via `Option` / `Result` — no null
- Explicit annotations required at public API boundaries

## Error handling

- Fallible operations return `Result[T, E]`
- `?` propagates errors
- `panic` / `unreachable` for programmer bugs only

## Concurrency

- OS threads via `std.thread`
- Async/await with structured task groups (nursery pattern)
- Send/Sync-like auto-traits derived from type structure

## Package & modules

- Packages declared in `Foxa.toml`
- Modules map to files / directories (`mod`, `use`)
- Semver dependency resolution via `foxa` package manager

## Syntax sketch

```foxa
pub fn greet(name: String) -> String {
    "Hello, " + name
}

fn main() {
    let mut count = 0;
    while count < 3 {
        print(greet("Foxa"));
        count += 1;
    }
}
```

See [grammar.md](grammar.md) for the formal sketch.
