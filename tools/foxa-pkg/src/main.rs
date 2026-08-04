//! Foxa package manager (`foxa-pkg`).
//!
//! Parses `Foxa.toml` and resolves dependency constraints (offline + path deps).

mod manifest;
mod resolve;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "foxa-pkg")]
#[command(about = "Foxa package manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a Foxa.toml in the current directory.
    Init {
        /// Package name.
        #[arg(long)]
        name: Option<String>,
    },
    /// Resolve dependencies declared in Foxa.toml.
    Resolve {
        /// Path to Foxa.toml.
        #[arg(long, default_value = "Foxa.toml")]
        manifest: PathBuf,
    },
    /// Add a dependency constraint to Foxa.toml.
    Add {
        /// Dependency spec `name@version` or `name`.
        spec: String,
        /// Path to Foxa.toml.
        #[arg(long, default_value = "Foxa.toml")]
        manifest: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Init { name } => {
            let name = name.unwrap_or_else(|| "app".into());
            manifest::write_default(&PathBuf::from("Foxa.toml"), &name)?;
            println!("Wrote Foxa.toml for package `{name}`");
            Ok(())
        }
        Commands::Resolve { manifest } => {
            let m = manifest::load(&manifest)?;
            let graph = resolve::resolve(&m)?;
            println!("Resolved {} package(s):", graph.len());
            for pkg in &graph {
                println!("  {} {}", pkg.name, pkg.version);
            }
            Ok(())
        }
        Commands::Add { spec, manifest } => {
            manifest::add_dependency(&manifest, &spec)?;
            println!("Added `{spec}` to {}", manifest.display());
            Ok(())
        }
    }
}
