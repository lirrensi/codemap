use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "codemap",
    about = "AI-optimized codebase index generator",
    long_about = "Scans a codebase and generates AI-optimized markdown indexes\n\
                  of function signatures and type definitions.\n\n\
                  Run without a subcommand to scan:\n  codemap [OPTIONS] [PATH]\n\n\
                  Run 'setup' to install the pre-commit hook:\n  codemap setup"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    // --- Scan options (used when no subcommand) ---
    /// Root directory to scan
    #[arg(default_value = ".", global = true)]
    pub path: PathBuf,

    /// Output file path
    #[arg(short, long, default_value = "./docs/CODEMAP.md", global = true)]
    pub output: PathBuf,

    /// Print to stdout instead of writing files
    #[arg(long, global = true)]
    pub stdout: bool,

    /// Glob patterns to exclude (repeatable)
    #[arg(long = "exclude", action = clap::ArgAction::Append, global = true)]
    pub exclude: Vec<String>,

    /// Comma-separated list of languages to include (default: all supported)
    #[arg(long, global = true)]
    pub languages: Option<String>,

    /// Maximum directory depth for the file tree (0 = unlimited)
    #[arg(long = "tree-depth", default_value = "5", global = true)]
    pub tree_depth: usize,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add codemap to .pre-commit-config.yaml
    ///
    /// Detects if a pre-commit config already exists:
    ///   - If yes: inserts the codemap hook entry (skips if already configured)
    ///   - If no: creates a new .pre-commit-config.yaml
    ///
    /// After setup, run: pre-commit install
    Setup,
}
