mod cli;
mod languages;
mod parser;
mod renderer;
mod types;
mod walker;

use clap::Parser;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let cli = cli::Cli::parse();
    let start = Instant::now();

    // 1. Discover files
    let root = fs::canonicalize(&cli.path).unwrap_or_else(|_| cli.path.clone());
    let files = walker::discover_files(&root, &cli.exclude);

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
    let output = renderer::render(&root, &file_map);

    // 6. Write or print
    if cli.stdout {
        print!("{}", output);
    } else {
        if let Some(parent) = cli.output.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&cli.output, &output).unwrap_or_else(|e| {
            eprintln!("Error writing to {}: {}", cli.output.display(), e);
            std::process::exit(1);
        });
        let file_count = file_map.len();
        let item_count: usize = file_map.values().map(|v| v.len()).sum();
        eprintln!(
            "Wrote {} ({} files, {} items) in {:.0}ms",
            cli.output.display(),
            file_count,
            item_count,
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
}
