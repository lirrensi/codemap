mod cli;

use clap::Parser;
use cli::Commands;
use codemap::{onboard, parser, renderer, setup, tree, types, walker};
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let cli = cli::Cli::parse();

    // Dispatch subcommand
    if let Some(ref cmd) = cli.command {
        match cmd {
            Commands::Scan => {
                run_scan(&cli);
                return;
            }
            Commands::Setup => {
                std::process::exit(setup::run_setup());
            }
            Commands::Onboard => {
                std::process::exit(onboard::run_onboard());
            }
        }
    }

    // Default: scan mode (same as `codemap scan`)
    run_scan(&cli);
}

fn run_scan(cli: &cli::Cli) {
    let start = Instant::now();

    // 1. Discover files
    let root = fs::canonicalize(&cli.path).unwrap_or_else(|_| cli.path.clone());
    let files = walker::discover_files(&root, &cli.exclude);

    // Build file tree from all discovered files (before language filtering)
    let tree_output = tree::build_tree(&files, &root, cli.tree_depth);

    // 2. Filter by language if specified
    let files: Vec<PathBuf> = if let Some(ref langs) = cli.languages {
        let allowed: Vec<&str> = langs.split(',').map(|s| s.trim()).collect();
        files
            .into_iter()
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|ext| allowed.iter().any(|l| l.eq_ignore_ascii_case(ext)))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        files
    };

    // 3. Parse all files in parallel
    let results: Vec<(PathBuf, Vec<types::Extractable>)> = files
        .par_iter()
        .filter_map(|path| {
            let source = fs::read_to_string(path).ok()?;
            let items = parser::extract_from_file(path, &source)?;
            if items.is_empty() {
                None
            } else {
                Some((path.clone(), items))
            }
        })
        .collect();

    // 4. Sort by file path and build the map
    let mut file_map = BTreeMap::new();
    for (path, items) in results {
        file_map.insert(path, items);
    }

    // 5. Render
    let (l1_output, l2_output) = renderer::render(&root, &file_map, &tree_output);

    // 6. Write or print
    if cli.stdout {
        print!("{}", l1_output);
    } else {
        // Write L1 file
        if let Some(parent) = cli.output.parent() {
            fs::create_dir_all(parent).ok();
        }
        let l1_path = cli.output.with_file_name(format!(
            "{}.L1.{}",
            cli.output.file_stem().unwrap_or_default().to_string_lossy(),
            cli.output.extension().unwrap_or_default().to_string_lossy()
        ));
        fs::write(&l1_path, &l1_output).unwrap_or_else(|e| {
            eprintln!("Error writing to {}: {}", l1_path.display(), e);
            std::process::exit(1);
        });

        // Write L2 file
        let l2_path = cli.output.with_file_name(format!(
            "{}.L2.{}",
            cli.output.file_stem().unwrap_or_default().to_string_lossy(),
            cli.output.extension().unwrap_or_default().to_string_lossy()
        ));
        fs::write(&l2_path, &l2_output).unwrap_or_else(|e| {
            eprintln!("Error writing to {}: {}", l2_path.display(), e);
            std::process::exit(1);
        });

        let file_count = file_map.len();
        let item_count: usize = file_map.values().map(|v| v.len()).sum();
        eprintln!(
            "Wrote {} and {} ({} files, {} items) in {:.0}ms",
            l1_path.display(),
            l2_path.display(),
            file_count,
            item_count,
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
}
