//! Debug helper: parse a file and print the tree-sitter AST structure.
//! Run with: cargo run --example debug_tree -- <filepath>
//! Supports ALL languages that codemap supports.

use codemap::parser;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: debug_tree <filepath>");
        eprintln!();
        eprintln!("Supported extensions:");
        eprintln!("  .rs .py .js .mjs .cjs .ts .tsx .go .sh .bash");
        eprintln!("  .c .h .cpp .cc .cxx .hpp .java .cs .rb .php");
        eprintln!("  .swift .kt .kts .scala .lua .zig .ex .exs .hs");
        eprintln!("  .ml .mli .dart .r .R .jl .yml .yaml .json");
        eprintln!("  .html .htm .css .scss");
        std::process::exit(1);
    }

    let path = std::path::Path::new(&args[1]);
    let source = fs::read_to_string(path).expect("Failed to read file");

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("?");
    let language = match parser::language_for_path(path) {
        Some(lang) => lang,
        None => {
            eprintln!("Unsupported extension: {}", ext);
            std::process::exit(1);
        }
    };

    let tree = parser::parse_source(language, &source).expect("Failed to parse");

    let root = tree.root_node();
    println!("=== File: {} ===", path.display());
    println!("Root kind: {}", root.kind());
    println!("Root child count: {}", root.child_count());
    println!();
    print_tree(root, &source, 0);

    // Also show what codemap extracts
    println!();
    println!("=== Codemap extraction ===");
    let items = parser::extract_from_file(path, &source).unwrap_or_default();
    if items.is_empty() {
        println!("(nothing extracted)");
    } else {
        for item in &items {
            println!("  {:?}", item);
        }
    }
}

fn print_tree(node: tree_sitter::Node, source: &str, indent: usize) {
    let prefix = "  ".repeat(indent);
    let text: String = source[node.byte_range()].chars().take(80).collect();
    let text = text.replace('\n', "\\n").replace('\r', "");
    println!(
        "{}{} [L{}-L{}] {:?}",
        prefix,
        node.kind(),
        node.start_position().row + 1,
        node.end_position().row + 1,
        text
    );

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_tree(child, source, indent + 1);
    }
}
