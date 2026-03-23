use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "codemap",
    about = "AI-optimized codebase index generator",
    long_about = "Scans a codebase and generates AI-optimized markdown indexes\n\
                  of function signatures and type definitions.\n\n\
                  Run without a subcommand to scan:\n  codemap [OPTIONS] [PATH]\n\n\
                  Subcommands:\n  codemap          Run the scan (same as default)\n  codemap setup    Non-interactive pre-commit + gitignore setup\n  codemap onboard  Interactive project setup wizard"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    // --- Scan options (used when no subcommand or with 'codemap' subcommand) ---
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
    /// Run the scan — generates CODEMAP.L1.md and CODEMAP.L2.md
    ///
    /// Same behavior as running codemap without a subcommand.
    /// Use this when you want to be explicit in scripts.
    Scan,

    /// Non-interactive setup: pre-commit hook + .gitignore
    ///
    /// Detects if a pre-commit config already exists:
    ///   - If yes: inserts the codemap hook entry (skips if already configured)
    ///   - If no: creates a new .pre-commit-config.yaml
    ///
    /// After setup, run: pre-commit install
    Setup,

    /// Interactive project onboarding wizard
    ///
    /// Walks you through three setup steps:
    ///   1. Pre-commit hook to auto-run codemap on commit
    ///   2. Add docs/codemap to .gitignore
    ///   3. Create/update AGENTS.md with codemap reference
    ///
    /// Each step asks for confirmation before making changes.
    Onboard,
}
