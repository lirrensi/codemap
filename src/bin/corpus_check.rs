use codemap::parser;
use codemap::types::Extractable;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Language-to-repo mapping
// ---------------------------------------------------------------------------

struct LangRepo {
    name: &'static str,
    repo: &'static str,
    exts: &'static [&'static str],
}

const LANG_REPOS: &[LangRepo] = &[
    LangRepo {
        name: "rust",
        repo: "https://github.com/tree-sitter/tree-sitter-rust.git",
        exts: &["rs"],
    },
    LangRepo {
        name: "python",
        repo: "https://github.com/tree-sitter/tree-sitter-python.git",
        exts: &["py"],
    },
    LangRepo {
        name: "javascript",
        repo: "https://github.com/tree-sitter/tree-sitter-javascript.git",
        exts: &["js"],
    },
    LangRepo {
        name: "typescript",
        repo: "https://github.com/tree-sitter/tree-sitter-typescript.git",
        exts: &["ts"],
    },
    LangRepo {
        name: "go",
        repo: "https://github.com/tree-sitter/tree-sitter-go.git",
        exts: &["go"],
    },
    LangRepo {
        name: "c",
        repo: "https://github.com/tree-sitter/tree-sitter-c.git",
        exts: &["c", "h"],
    },
    LangRepo {
        name: "cpp",
        repo: "https://github.com/tree-sitter/tree-sitter-cpp.git",
        exts: &["cpp", "cc", "cxx", "hpp"],
    },
    LangRepo {
        name: "java",
        repo: "https://github.com/tree-sitter/tree-sitter-java.git",
        exts: &["java"],
    },
    LangRepo {
        name: "ruby",
        repo: "https://github.com/tree-sitter/tree-sitter-ruby.git",
        exts: &["rb"],
    },
    LangRepo {
        name: "haskell",
        repo: "https://github.com/tree-sitter/tree-sitter-haskell.git",
        exts: &["hs"],
    },
    LangRepo {
        name: "ocaml",
        repo: "https://github.com/tree-sitter/tree-sitter-ocaml.git",
        exts: &["ml"],
    },
    LangRepo {
        name: "zig",
        repo: "https://github.com/tree-sitter/tree-sitter-zig.git",
        exts: &["zig"],
    },
];

// ---------------------------------------------------------------------------
// Corpus parsing
// ---------------------------------------------------------------------------

/// A single test case extracted from a tree-sitter corpus file.
struct CorpusSnippet {
    name: String,
    source: String,
}

/// Parse a tree-sitter corpus .txt file into individual test snippets.
///
/// Format:
/// ```text
/// ================================================================================
/// test_name
/// ================================================================================
///
/// source code here
///
/// --------------------------------------------------------------------------------
///
/// (expected tree)
/// ================================================================================
/// ```
fn parse_corpus_file(content: &str) -> Vec<CorpusSnippet> {
    let mut snippets = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Look for a separator line (20+ '=' chars)
        if !is_eq_separator(lines[i]) {
            i += 1;
            continue;
        }

        // Next line should be the test name
        i += 1;
        if i >= lines.len() {
            break;
        }
        let test_name = lines[i].trim().to_string();
        if test_name.is_empty() {
            continue;
        }

        // Skip the closing '=' separator
        i += 1;
        if i >= lines.len() || !is_eq_separator(lines[i]) {
            continue;
        }
        i += 1;

        // Collect source lines until we hit the "---..." dash separator
        let mut source_lines = Vec::new();
        while i < lines.len() {
            if is_dash_separator(lines[i]) {
                break;
            }
            // Also stop if we hit the next test block
            if is_eq_separator(lines[i]) {
                break;
            }
            source_lines.push(lines[i]);
            i += 1;
        }

        if !source_lines.is_empty() {
            snippets.push(CorpusSnippet {
                name: test_name,
                source: source_lines.join("\n"),
            });
        }
    }

    snippets
}

fn is_eq_separator(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 20 && trimmed.chars().all(|c| c == '=')
}

fn is_dash_separator(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 3 && trimmed.chars().all(|c| c == '-')
}

// ---------------------------------------------------------------------------
// Git operations
// ---------------------------------------------------------------------------

fn clone_repo(url: &str, dest: &Path) -> bool {
    if dest.exists() {
        eprintln!("  already cloned, skipping");
        return true;
    }
    eprintln!("  cloning {} ...", url);
    let status = Command::new("git")
        .args(["clone", "--depth", "1", url, &dest.to_string_lossy()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match status {
        Ok(s) if s.success() => true,
        _ => {
            eprintln!("  FAILED to clone");
            false
        }
    }
}

// ---------------------------------------------------------------------------
// File discovery
// ---------------------------------------------------------------------------

fn find_corpus_files(repo_dir: &Path) -> Vec<PathBuf> {
    let corpus_dir = repo_dir.join("test").join("corpus");
    if !corpus_dir.exists() {
        return Vec::new();
    }
    collect_files_with_ext(&corpus_dir, &["txt"])
}

fn find_source_files(dir: &Path, exts: &[&str]) -> Vec<PathBuf> {
    if !dir.exists() {
        return Vec::new();
    }
    let mut files = Vec::new();
    collect_recursive(dir, exts, &mut files);
    files.sort();
    files
}

fn collect_files_with_ext(dir: &Path, exts: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_recursive(dir, exts, &mut files);
    files.sort();
    files
}

fn collect_recursive(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip common non-source directories
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && matches!(
                    name,
                    "node_modules"
                        | "target"
                        | ".git"
                        | "vendor"
                        | "__pycache__"
                        | ".tox"
                        | "dist"
                        | "build"
                )
            {
                continue;
            }
            collect_recursive(&path, exts, out);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str())
            && exts.iter().any(|e| e.eq_ignore_ascii_case(ext))
        {
            out.push(path);
        }
    }
}

// ---------------------------------------------------------------------------
// Extraction runner
// ---------------------------------------------------------------------------

struct SnippetResult {
    name: String,
    source: String,
    items: Vec<Extractable>,
}

struct SourceFileResult {
    path: PathBuf,
    items: Vec<Extractable>,
}

fn run_on_snippet(name: &str, source: &str, ext: &str) -> Option<SnippetResult> {
    let fake_path = PathBuf::from(format!("snippet.{}", ext));
    let items = parser::extract_from_file(&fake_path, source).unwrap_or_default();
    Some(SnippetResult {
        name: name.to_string(),
        source: source.to_string(),
        items,
    })
}

fn run_on_source_file(path: &Path) -> Option<SourceFileResult> {
    let source = fs::read_to_string(path).ok()?;
    let items = parser::extract_from_file(path, &source).unwrap_or_default();
    Some(SourceFileResult {
        path: path.to_path_buf(),
        items,
    })
}

// ---------------------------------------------------------------------------
// Report generation
// ---------------------------------------------------------------------------

fn render_item(item: &Extractable) -> String {
    match item {
        Extractable::Function(sig) => {
            let ret = sig
                .return_type
                .as_ref()
                .map(|r| format!(" -> {}", r))
                .unwrap_or_default();
            format!("`{}{}{}` :{}", sig.name, sig.params, ret, sig.line)
        }
        Extractable::Type(t) => format!("`{}` ({})", t.name, t.kind),
    }
}

fn generate_corpus_section(lang: &LangRepo, results: &[SnippetResult]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "## {} — corpus snippets ({} tested)\n\n",
        lang.name,
        results.len()
    ));

    for (idx, result) in results.iter().enumerate() {
        out.push_str(&format!("### {}. {}\n", idx + 1, result.name));
        out.push_str(&format!("```{}\n{}\n```\n", lang.exts[0], result.source));

        if result.items.is_empty() {
            out.push_str("**→ (nothing extracted)**\n\n");
        } else {
            out.push_str("**→**\n");
            for item in &result.items {
                out.push_str(&format!("- {}\n", render_item(item)));
            }
            out.push('\n');
        }
    }
    out
}

fn generate_source_section(label: &str, root: &Path, results: &[SourceFileResult]) -> String {
    let mut out = String::new();
    out.push_str(&format!("## {} ({} files)\n\n", label, results.len()));

    for result in results {
        let rel = result
            .path
            .strip_prefix(root)
            .unwrap_or(&result.path)
            .to_string_lossy()
            .replace('\\', "/");
        out.push_str(&format!("### {}\n", rel));

        if result.items.is_empty() {
            out.push_str("**→ (nothing extracted)**\n\n");
        } else {
            out.push_str("**→**\n");
            for item in &result.items {
                out.push_str(&format!("- {}\n", render_item(item)));
            }
            out.push('\n');
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn print_usage() {
    eprintln!("Usage: corpus_check [OPTIONS] [LANGUAGE]");
    eprintln!();
    eprintln!("Run codemap against tree-sitter corpus files and/or real source code.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --source <path>   Also test against source files in this directory");
    eprintln!("  --max <n>         Max snippets per language (default: 10)");
    eprintln!("  --max-files <n>   Max source files to test (default: 5)");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  LANGUAGE          Optional: only test this language (e.g. 'rust', 'python')");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  corpus_check                  # Test all languages");
    eprintln!("  corpus_check rust             # Test only Rust corpus");
    eprintln!("  corpus_check --source ../curl # Test corpus + real source in ../curl");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return;
    }

    // Parse args
    let mut filter_lang: Option<&str> = None;
    let mut source_dir: Option<PathBuf> = None;
    let mut max_snippets: usize = 10;
    let mut max_source_files: usize = 5;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--source" => {
                i += 1;
                source_dir = args.get(i).map(PathBuf::from);
            }
            "--max" => {
                i += 1;
                if let Some(n) = args.get(i).and_then(|s| s.parse().ok()) {
                    max_snippets = n;
                }
            }
            "--max-files" => {
                i += 1;
                if let Some(n) = args.get(i).and_then(|s| s.parse().ok()) {
                    max_source_files = n;
                }
            }
            other if !other.starts_with('-') => {
                filter_lang = Some(other);
            }
            _ => {}
        }
        i += 1;
    }

    let temp_dir = std::env::temp_dir().join("codemap_corpus_check");
    fs::create_dir_all(&temp_dir).ok();

    let mut full_report = String::new();
    full_report.push_str("# Corpus Quality Check\n\n");
    full_report.push_str(&format!(
        "_generated: {}_\n\n",
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
    ));

    let mut total_snippets = 0;
    let mut total_sources = 0;
    let mut total_items = 0;
    let mut total_empty_snippets = 0;

    for lang in LANG_REPOS {
        if let Some(f) = filter_lang
            && !lang.name.eq_ignore_ascii_case(f)
        {
            continue;
        }

        eprintln!("\n=== {} ===", lang.name);
        let repo_dir = temp_dir.join(lang.name);

        // Clone
        if !clone_repo(lang.repo, &repo_dir) {
            continue;
        }

        // --- Corpus snippets (distribute across files) ---
        let corpus_files = find_corpus_files(&repo_dir);
        eprintln!("  found {} corpus files", corpus_files.len());

        let snippets_per_file = if corpus_files.is_empty() {
            0
        } else {
            (max_snippets / corpus_files.len()).max(1)
        };

        let mut snippet_results = Vec::new();
        for cf in &corpus_files {
            if snippet_results.len() >= max_snippets {
                break;
            }
            let content = match fs::read_to_string(cf) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let snippets = parse_corpus_file(&content);
            let file_name = cf.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
            let mut taken_from_file = 0;
            for snippet in snippets {
                if snippet_results.len() >= max_snippets {
                    break;
                }
                if taken_from_file >= snippets_per_file {
                    break;
                }
                if let Some(result) = run_on_snippet(&snippet.name, &snippet.source, lang.exts[0]) {
                    if result.items.is_empty() {
                        total_empty_snippets += 1;
                    }
                    total_items += result.items.len();
                    snippet_results.push(result);
                    taken_from_file += 1;
                }
            }
            if taken_from_file > 0 {
                eprintln!("    {} → {} snippets", file_name, taken_from_file);
            }
        }

        total_snippets += snippet_results.len();

        eprintln!(
            "  snippets: {} ({} with items)",
            snippet_results.len(),
            snippet_results
                .iter()
                .filter(|r| !r.items.is_empty())
                .count(),
        );

        full_report.push_str(&generate_corpus_section(lang, &snippet_results));
    }

    // --- Real source files (if --source provided) ---
    if let Some(ref src_dir) = source_dir {
        let canonical = fs::canonicalize(src_dir).unwrap_or_else(|_| src_dir.clone());
        eprintln!("\n=== source: {} ===", canonical.display());

        // Collect all supported extensions
        let all_exts: Vec<&str> = LANG_REPOS
            .iter()
            .flat_map(|l| l.exts.iter().copied())
            .collect();
        let source_files = find_source_files(&canonical, &all_exts);
        eprintln!("  found {} source files", source_files.len());

        let mut source_results = Vec::new();
        for sf in source_files
            .iter()
            .take(max_source_files * LANG_REPOS.len())
        {
            if let Some(result) = run_on_source_file(sf) {
                total_items += result.items.len();
                source_results.push(result);
            }
        }

        total_sources += source_results.len();

        full_report.push_str(&generate_source_section(
            &format!("Real source — {}", canonical.display()),
            &canonical,
            &source_results,
        ));
    }

    // Summary
    let summary = format!(
        "## Summary\n\n\
         - Languages tested: {}\n\
         - Corpus snippets: {}\n\
         - Real source files: {}\n\
         - Total items extracted: {}\n\
         - Snippets with nothing extracted: {}\n",
        LANG_REPOS.len(),
        total_snippets,
        total_sources,
        total_items,
        total_empty_snippets,
    );

    full_report.push_str(&summary);

    // Write report
    let output_path = PathBuf::from("docs/CORPUS_CHECK.md");
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&output_path, &full_report).unwrap_or_else(|e| {
        eprintln!("Error writing report: {}", e);
        std::process::exit(1);
    });

    eprintln!(
        "\nDone. Report written to {} ({} snippets, {} source files, {} items)",
        output_path.display(),
        total_snippets,
        total_sources,
        total_items
    );
}
