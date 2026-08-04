# Foxa

**Foxa** is a modern systems programming language designed for performance,
safety, and excellent developer experience.

## Playground (frontend)

Colab-style React app with online compiler, Learn, ML Lab, Help, and Feedback:

```bash
cd frontend
npm install
npm run dev
```

**Netlify:** connect this repo — `netlify.toml` builds `frontend/` and publishes `dist` with SPA redirects. See [frontend/README.md](frontend/README.md#deploy-on-netlify).

## Native toolchain

> Status: **active development** — `foxa show` / `foxa run` interpret programs;
> `foxa fn` scaffolds functions into `.foxa` files; `foxa jit` compiles Int
> functions via Cranelift; `foxa-pkg` resolves `Foxa.toml` dependencies.

```bash
# Build the toolchain
cargo build -p foxac

# Show program output (compile + run main)
cargo run -p foxac -- show examples/hello.foxa

# Scaffold a Foxa function into a .foxa file
cargo run -p foxac -- fn greet --params "name: String" --ret String --file examples/greet.foxa

# Check a source file (lex + parse + resolve + typecheck)
cargo run -p foxac -- check examples/hello.foxa

# Run via the interpreter
cargo run -p foxac -- run examples/hello.foxa
cargo run -p foxac -- run examples/features.foxa

# JIT an Int function with Cranelift
cargo run -p foxac -- jit examples/jit_add.foxa

# Package resolve
cargo run -p foxa-pkg -- resolve --manifest Foxa.toml
```

## Example

```foxa
fn main() {
    show("Hello, Foxa!");
}

fn add(a: Int, b: Int) -> Int {
    a + b
}
```

Create functions with Foxa `fn` (CLI: `foxa fn ...`). Runnable files need `fn main()`.

## Design pillars

| Pillar | Approach |
|--------|----------|
| Safety | Ownership for unique values + ARC for shared heap (no tracing GC in v1) |
| Performance | LLVM release backend; fast interpreter/Cranelift for debug |
| Concurrency | Structured concurrency + async/await |
| Errors | `Result` + `?`; panics only for bugs |
| Tooling | First-class CLI, formatter, linter, LSP, package manager |

## Repository layout

```
frontend/          # React playground (compiler, Learn, ML Lab, Help, Feedback)
compiler/          # Rust compiler crates (foxa-span … foxac)
runtime/           # Runtime (allocator, async, threads) — planned
std/               # Standard library — planned
tools/             # pkg, fmt, lint, doc, lsp — planned
docs/design/       # Architecture and language design
examples/          # Sample programs
tests/             # Integration / golden tests
```

## Documentation

- [Language overview](docs/design/language-overview.md)
- [Compiler architecture](docs/design/compiler-architecture.md)
- [Grammar sketch](docs/design/grammar.md)
- [Roadmap](docs/design/roadmap.md)
- [Frontend playground](frontend/README.md)

## Development

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

## License

Apache-2.0
# FoxaLang
