use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "codemap", about = "AI-optimized codebase index generator")]
pub struct Cli {
    /// Root directory to scan
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output file path
    #[arg(short, long, default_value = "./docs/CODEMAP.md")]
    pub output: PathBuf,

    /// Print to stdout instead of writing a file
    #[arg(long)]
    pub stdout: bool,

    /// Glob patterns to exclude (repeatable)
    #[arg(long = "exclude", action = clap::ArgAction::Append)]
    pub exclude: Vec<String>,

    /// Comma-separated list of languages to include (default: all supported)
    #[arg(long)]
    pub languages: Option<String>,
}
