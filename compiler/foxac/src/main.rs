//! Foxa compiler driver (`foxa` CLI).
//!
//! # Purpose
//!
//! Entry point for the Foxa toolchain: lex, parse, (later) typecheck, and
//! compile Foxa programs. Subcommands mirror the planned product surface:
//! `new`, `build`, `run`, `test`, `fmt`, `lint`, `doc`, `check`.
//!
//! # Usage
//!
//! ```text
//! foxa --version
//! foxa show examples/hello.foxa
//! foxa fn greet --params "name: String" --ret String
//! foxa check examples/hello.foxa
//! foxa lex examples/hello.foxa
//! foxa parse examples/hello.foxa
//! ```

#![deny(missing_docs)]

mod cli;
mod commands;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();
    if let Err(err) = commands::execute(cli) {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}
