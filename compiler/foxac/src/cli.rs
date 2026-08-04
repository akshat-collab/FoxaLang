//! CLI argument definitions.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Foxa language toolchain.
#[derive(Debug, Parser)]
#[command(name = "foxa")]
#[command(about = "The Foxa programming language toolchain", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available toolchain commands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create a new Foxa project.
    New {
        /// Project name.
        name: String,
        /// Create a library instead of a binary.
        #[arg(long)]
        lib: bool,
    },
    /// Build the current package (codegen not yet available).
    Build {
        /// Build in release mode.
        #[arg(long)]
        release: bool,
    },
    /// Compile and run a Foxa file or package.
    Run {
        /// Optional path to a `.foxa` file.
        path: Option<PathBuf>,
    },
    /// Run package tests.
    Test,
    /// Format Foxa sources.
    Fmt {
        /// Check formatting without writing.
        #[arg(long)]
        check: bool,
    },
    /// Lint Foxa sources.
    Lint,
    /// Generate documentation.
    Doc,
    /// Type-check / parse-check without producing binaries.
    Check {
        /// Path to a `.foxa` file.
        path: PathBuf,
    },
    /// Lex a source file and print tokens (compiler debugging).
    Lex {
        /// Path to a `.foxa` file.
        path: PathBuf,
        /// Emit JSON instead of text.
        #[arg(long)]
        json: bool,
    },
    /// Parse a source file and print a summary (compiler debugging).
    Parse {
        /// Path to a `.foxa` file.
        path: PathBuf,
    },
    /// Lower to MIR and print a summary.
    Mir {
        /// Path to a `.foxa` file.
        path: PathBuf,
    },
    /// JIT-compile Int functions with Cranelift and execute `name(a,b)` demo.
    Jit {
        /// Path to a `.foxa` file containing an `add(a: Int, b: Int) -> Int`.
        path: PathBuf,
        /// Left operand.
        #[arg(long, default_value_t = 20)]
        a: i64,
        /// Right operand.
        #[arg(long, default_value_t = 22)]
        b: i64,
        /// Function name to call.
        #[arg(long, default_value = "add")]
        func: String,
    },
}
