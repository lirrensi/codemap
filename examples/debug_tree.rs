//! Debug helper: parse a file and print the tree-sitter AST structure.
//! Run with: cargo run --example debug_tree -- <filepath>

use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: debug_tree <filepath>");
        std::process::exit(1);
    }

    let path = std::path::Path::new(&args[1]);
    let source = fs::read_to_string(path).expect("Failed to read file");

    let ext = path.extension().unwrap().to_str().unwrap();
    let language = match ext {
        "swift" => tree_sitter_swift::LANGUAGE.into(),
        "kt" | "kts" => tree_sitter_kotlin_ng::LANGUAGE.into(),
        "hs" => tree_sitter_haskell::LANGUAGE.into(),
        "ml" => tree_sitter_ocaml::LANGUAGE_OCAML.into(),
        "dart" => tree_sitter_dart::LANGUAGE.into(),
        "r" | "R" => tree_sitter_r::LANGUAGE.into(),
        "php" => tree_sitter_php::LANGUAGE_PHP.into(),
        "rb" => tree_sitter_ruby::LANGUAGE.into(),
        "ex" | "exs" => tree_sitter_elixir::LANGUAGE.into(),
        "jl" => tree_sitter_julia::LANGUAGE.into(),
        _ => {
            eprintln!("Unsupported extension: {}", ext);
            std::process::exit(1);
        }
    };

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&language)
        .expect("Failed to set language");
    let tree = parser.parse(&source, None).expect("Failed to parse");

    let root = tree.root_node();
    println!("Root kind: {}", root.kind());
    println!("Root child count: {}", root.child_count());
    print_tree(root, &source, 0);
}

fn print_tree(node: tree_sitter::Node, source: &str, indent: usize) {
    let prefix = "  ".repeat(indent);
    let text_preview: String = source[node.byte_range()].chars().take(60).collect();
    let text_preview = text_preview.replace('\n', "\\n");
    println!(
        "{}{} [{}-{}] {:?}",
        prefix,
        node.kind(),
        node.start_position().row + 1,
        node.end_position().row + 1,
        text_preview
    );

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_tree(child, source, indent + 1);
    }
}
