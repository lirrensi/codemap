pub mod bash;
pub mod c_lang;
pub mod cpp;
pub mod csharp;
pub mod dart;
pub mod elixir;
pub mod go_lang;
pub mod haskell;
pub mod java;
pub mod javascript;
pub mod julia;
pub mod kotlin;
pub mod lua;
pub mod ocaml;
pub mod php;
pub mod python;
pub mod r_lang;
pub mod ruby;
pub mod rust_lang;
pub mod scala;
pub mod swift;
pub mod typescript;
pub mod zig;

use tree_sitter::Node;

/// Helper: get the text of a node from source bytes.
pub fn node_text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

/// Helper: find a child node by kind.
pub fn child_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == kind {
            return Some(child);
        }
    }
    None
}

/// Helper: find all children of a specific kind.
#[allow(dead_code)]
pub fn children_by_kind<'a>(node: Node<'a>, kind: &str) -> Vec<Node<'a>> {
    let mut result = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == kind {
            result.push(child);
        }
    }
    result
}
