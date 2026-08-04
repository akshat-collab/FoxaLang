//! Command implementations.

use crate::cli::{Cli, Commands};
use anyhow::{bail, Context, Result};
use foxa_codegen::compile_module;
use foxa_diagnostics::{DiagnosticBag, Emitter};
use foxa_interp::Interpreter;
use foxa_lexer::{Lexer, TokenKind};
use foxa_mir::lower_module;
use foxa_parser::Parser;
use foxa_resolve::Resolver;
use foxa_span::SourceMap;
use foxa_types::TypeChecker;
use std::fs;
use std::path::Path;

/// Dispatches CLI commands.
pub fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::New { name, lib } => cmd_new(&name, lib),
        Commands::Build { release } => cmd_build(release),
        Commands::Run { path } => cmd_run(path.as_deref()),
        Commands::Test => {
            println!("foxa test: test runner will be available in a later phase");
            Ok(())
        }
        Commands::Fmt { check } => {
            let mode = if check { "check" } else { "write" };
            println!("foxa fmt ({mode}): formatter will be available in a later phase");
            Ok(())
        }
        Commands::Lint => {
            println!("foxa lint: linter will be available in a later phase");
            Ok(())
        }
        Commands::Doc => {
            println!("foxa doc: documentation generator will be available in a later phase");
            Ok(())
        }
        Commands::Check { path } => cmd_check(&path),
        Commands::Lex { path, json } => cmd_lex(&path, json),
        Commands::Parse { path } => cmd_parse(&path),
        Commands::Mir { path } => cmd_mir(&path),
        Commands::Jit { path, a, b, func } => cmd_jit(&path, &func, a, b),
    }
}

fn cmd_new(name: &str, lib: bool) -> Result<()> {
    if name.is_empty() || name.contains('/') || name.contains('\\') {
        bail!("invalid project name `{name}`");
    }
    let root = Path::new(name);
    if root.exists() {
        bail!("directory `{name}` already exists");
    }
    fs::create_dir_all(root.join("src"))?;
    let manifest = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2026"

[dependencies]
"#
    );
    fs::write(root.join("Foxa.toml"), manifest)?;
    let entry = if lib {
        "/// Library crate root.\npub fn answer() -> Int {\n    42\n}\n"
    } else {
        "fn main() {\n    print(\"Hello, Foxa!\");\n}\n"
    };
    let entry_name = if lib { "lib.foxa" } else { "main.foxa" };
    fs::write(root.join("src").join(entry_name), entry)?;
    println!(
        "Created {} project `{name}`",
        if lib { "library" } else { "binary" }
    );
    Ok(())
}

fn cmd_build(release: bool) -> Result<()> {
    let profile = if release { "release" } else { "dev" };
    println!(
        "foxa build ({profile}): native codegen is not available yet; use `foxa run` to execute via the interpreter"
    );
    Ok(())
}

fn cmd_run(path: Option<&Path>) -> Result<()> {
    let Some(path) = path else {
        println!("foxa run: specify a file, e.g. `foxa run src/main.foxa`");
        return Ok(());
    };
    let (module, mut bag, map) = load_and_parse(path)?;
    if !bag.has_errors() {
        let resolved = Resolver::new(&mut bag).resolve(&module);
        if !bag.has_errors() {
            TypeChecker::new(&resolved, &mut bag).check(&module);
        }
    }
    emit_diagnostics(&map, &bag)?;
    if bag.has_errors() {
        bail!("run failed with {} error(s)", bag.error_count());
    }
    let mut interp = Interpreter::new(&module);
    interp
        .run_main()
        .map_err(|e| anyhow::anyhow!("runtime error: {e}"))?;
    Ok(())
}

fn cmd_check(path: &Path) -> Result<()> {
    let (module, mut bag, map) = load_and_parse(path)?;
    if !bag.has_errors() {
        let resolved = Resolver::new(&mut bag).resolve(&module);
        if !bag.has_errors() {
            TypeChecker::new(&resolved, &mut bag).check(&module);
        }
    }
    emit_diagnostics(&map, &bag)?;
    if bag.has_errors() {
        bail!("check failed with {} error(s)", bag.error_count());
    }
    println!(
        "ok — {} item(s) type-checked in {}",
        module.items.len(),
        path.display()
    );
    Ok(())
}

fn cmd_lex(path: &Path, json: bool) -> Result<()> {
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut map = SourceMap::new();
    let file = map.add_file(path, source.clone());
    let mut bag = DiagnosticBag::new();
    let tokens = Lexer::new(file, &source, &mut bag).tokenize_all();
    emit_diagnostics(&map, &bag)?;

    if json {
        println!("[");
        for (i, tok) in tokens.iter().enumerate() {
            let lexeme = map.snippet(tok.span).unwrap_or("");
            let comma = if i + 1 == tokens.len() { "" } else { "," };
            println!(
                "  {{\"kind\": \"{:?}\", \"lexeme\": {:?}, \"span\": \"{}\"}}{comma}",
                tok.kind, lexeme, tok.span
            );
        }
        println!("]");
    } else {
        for tok in &tokens {
            if tok.kind == TokenKind::Eof {
                println!("{:<20} <eof>", format!("{:?}", tok.kind));
                continue;
            }
            let lexeme = map.snippet(tok.span).unwrap_or("");
            println!("{:<20} {:?}", format!("{:?}", tok.kind), lexeme);
        }
    }

    if bag.has_errors() {
        bail!("lex failed with {} error(s)", bag.error_count());
    }
    Ok(())
}

fn cmd_parse(path: &Path) -> Result<()> {
    let (module, bag, map) = load_and_parse(path)?;
    emit_diagnostics(&map, &bag)?;
    for (i, item) in module.items.iter().enumerate() {
        match &item.kind {
            foxa_ast::ItemKind::Fn(f) => {
                println!(
                    "item[{i}]: fn {}({} params) -> {:?}",
                    f.name,
                    f.params.len(),
                    f.return_ty
                );
            }
            foxa_ast::ItemKind::Struct(s) => {
                println!("item[{i}]: struct {} ({} fields)", s.name, s.fields.len());
            }
            foxa_ast::ItemKind::Enum(e) => {
                println!("item[{i}]: enum {} ({} variants)", e.name, e.variants.len());
            }
            foxa_ast::ItemKind::Error => println!("item[{i}]: <error>"),
        }
    }
    if bag.has_errors() {
        bail!("parse failed with {} error(s)", bag.error_count());
    }
    Ok(())
}

fn cmd_mir(path: &Path) -> Result<()> {
    let (module, mut bag, map) = load_and_parse(path)?;
    if !bag.has_errors() {
        let resolved = Resolver::new(&mut bag).resolve(&module);
        if !bag.has_errors() {
            TypeChecker::new(&resolved, &mut bag).check(&module);
        }
    }
    emit_diagnostics(&map, &bag)?;
    if bag.has_errors() {
        bail!("mir failed with {} error(s)", bag.error_count());
    }
    let mir = lower_module(&module);
    for f in &mir.functions {
        println!(
            "fn {} ({} params, {} locals, {} blocks) -> {}",
            f.name,
            f.params.len(),
            f.locals.len(),
            f.blocks.len(),
            f.return_ty
        );
    }
    Ok(())
}

fn cmd_jit(path: &Path, func: &str, a: i64, b: i64) -> Result<()> {
    let (module, mut bag, map) = load_and_parse(path)?;
    if !bag.has_errors() {
        let resolved = Resolver::new(&mut bag).resolve(&module);
        if !bag.has_errors() {
            TypeChecker::new(&resolved, &mut bag).check(&module);
        }
    }
    emit_diagnostics(&map, &bag)?;
    if bag.has_errors() {
        bail!("jit failed with {} error(s)", bag.error_count());
    }
    let mir = lower_module(&module);
    let engine = compile_module(&mir).map_err(|e| anyhow::anyhow!("{e}"))?;
    let f = unsafe { engine.get_fn2(func) }.map_err(|e| anyhow::anyhow!("{e}"))?;
    let result = f(a, b);
    println!("{result}");
    Ok(())
}

fn load_and_parse(path: &Path) -> Result<(foxa_ast::Module, DiagnosticBag, SourceMap)> {
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut map = SourceMap::new();
    let file = map.add_file(path, source.clone());
    let mut bag = DiagnosticBag::new();
    let tokens = Lexer::new(file, &source, &mut bag).tokenize_all();
    let module = Parser::new(file, &source, tokens, &mut bag).parse_module();
    Ok((module, bag, map))
}

fn emit_diagnostics(map: &SourceMap, bag: &DiagnosticBag) -> Result<()> {
    if bag.items().is_empty() {
        return Ok(());
    }
    let emitter = Emitter::new(map).with_color(true);
    let mut stderr = std::io::stderr().lock();
    emitter.emit_all(bag, &mut stderr)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    #[test]
    fn check_example_source() {
        let dir = std::env::temp_dir().join(format!("foxa-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("hello.foxa");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "fn main() {{ print(\"hi\"); }}").unwrap();
        cmd_check(&path).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn new_project_creates_files() {
        let name = format!("foxa_proj_{}", std::process::id());
        let root = PathBuf::from(&name);
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        cmd_new(&name, false).unwrap();
        assert!(root.join("Foxa.toml").exists());
        assert!(root.join("src/main.foxa").exists());
        fs::remove_dir_all(&root).unwrap();
    }
}
