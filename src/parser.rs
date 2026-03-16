use crate::types::Extractable;
use std::path::Path;
use tree_sitter::{Language, Parser, Tree};

/// Get the tree-sitter language for a given file extension.
pub fn language_for_path(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?;
    language_for_extension(ext)
}

/// Get the tree-sitter language for a given file extension string.
pub fn language_for_extension(ext: &str) -> Option<Language> {
    match ext {
        "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
        "py" => Some(tree_sitter_python::LANGUAGE.into()),
        "js" | "mjs" | "cjs" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "ts" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "sh" | "bash" => Some(tree_sitter_bash::LANGUAGE.into()),
        "c" | "h" => Some(tree_sitter_c::LANGUAGE.into()),
        "cpp" | "cc" | "cxx" | "hpp" => Some(tree_sitter_cpp::LANGUAGE.into()),
        "java" => Some(tree_sitter_java::LANGUAGE.into()),
        "cs" => Some(tree_sitter_c_sharp::LANGUAGE.into()),
        "rb" => Some(tree_sitter_ruby::LANGUAGE.into()),
        "php" => Some(tree_sitter_php::LANGUAGE_PHP.into()),
        "swift" => Some(tree_sitter_swift::LANGUAGE.into()),
        "kt" | "kts" => Some(tree_sitter_kotlin_ng::LANGUAGE.into()),
        "scala" => Some(tree_sitter_scala::LANGUAGE.into()),
        "lua" => Some(tree_sitter_lua::LANGUAGE.into()),
        "zig" => Some(tree_sitter_zig::LANGUAGE.into()),
        "ex" | "exs" => Some(tree_sitter_elixir::LANGUAGE.into()),
        "hs" => Some(tree_sitter_haskell::LANGUAGE.into()),
        "ml" => Some(tree_sitter_ocaml::LANGUAGE_OCAML.into()),
        "mli" => Some(tree_sitter_ocaml::LANGUAGE_OCAML_INTERFACE.into()),
        "dart" => Some(tree_sitter_dart::LANGUAGE.into()),
        "r" | "R" => Some(tree_sitter_r::LANGUAGE.into()),
        "jl" => Some(tree_sitter_julia::LANGUAGE.into()),
        "yml" | "yaml" => Some(tree_sitter_yaml::LANGUAGE.into()),
        "json" => Some(tree_sitter_json::LANGUAGE.into()),
        "html" | "htm" => Some(tree_sitter_html::LANGUAGE.into()),
        "css" | "scss" => Some(tree_sitter_css::LANGUAGE.into()),
        _ => None,
    }
}

/// Check if a file extension is a supported code language (not markup/config).
#[allow(dead_code)]
pub fn is_code_extension(ext: &str) -> bool {
    matches!(
        ext,
        "rs" | "py"
            | "js"
            | "mjs"
            | "cjs"
            | "ts"
            | "tsx"
            | "go"
            | "sh"
            | "bash"
            | "c"
            | "h"
            | "cpp"
            | "cc"
            | "cxx"
            | "hpp"
            | "java"
            | "cs"
            | "rb"
            | "php"
            | "swift"
            | "scala"
            | "lua"
            | "zig"
            | "ex"
            | "exs"
            | "hs"
            | "ml"
            | "mli"
            | "dart"
            | "r"
            | "R"
            | "jl"
    )
}

/// Parse source code into a tree-sitter Tree.
pub fn parse_source(language: Language, source: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;
    parser.parse(source, None)
}

/// Extract all signatures from a file. Returns None if language is unsupported.
pub fn extract_from_file(path: &Path, source: &str) -> Option<Vec<Extractable>> {
    let language = language_for_path(path)?;
    let tree = parse_source(language, source)?;
    let ext = path.extension()?.to_str()?;

    let items = match ext {
        "rs" => crate::languages::rust_lang::extract(source, &tree),
        "py" => crate::languages::python::extract(source, &tree),
        "js" | "mjs" | "cjs" => crate::languages::javascript::extract(source, &tree),
        "ts" | "tsx" => crate::languages::typescript::extract(source, &tree),
        "go" => crate::languages::go_lang::extract(source, &tree),
        "sh" | "bash" => crate::languages::bash::extract(source, &tree),
        "c" | "h" => crate::languages::c_lang::extract(source, &tree),
        "cpp" | "cc" | "cxx" | "hpp" => crate::languages::cpp::extract(source, &tree),
        "java" => crate::languages::java::extract(source, &tree),
        "cs" => crate::languages::csharp::extract(source, &tree),
        "rb" => crate::languages::ruby::extract(source, &tree),
        "php" => crate::languages::php::extract(source, &tree),
        "swift" => crate::languages::swift::extract(source, &tree),
        "kt" | "kts" => crate::languages::kotlin::extract(source, &tree),
        "scala" => crate::languages::scala::extract(source, &tree),
        "lua" => crate::languages::lua::extract(source, &tree),
        "zig" => crate::languages::zig::extract(source, &tree),
        "ex" | "exs" => crate::languages::elixir::extract(source, &tree),
        "hs" => crate::languages::haskell::extract(source, &tree),
        "ml" | "mli" => crate::languages::ocaml::extract(source, &tree),
        "dart" => crate::languages::dart::extract(source, &tree),
        "r" | "R" => crate::languages::r_lang::extract(source, &tree),
        "jl" => crate::languages::julia::extract(source, &tree),
        _ => Vec::new(),
    };

    Some(items)
}
