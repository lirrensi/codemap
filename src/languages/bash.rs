use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        if child.kind() == "function_definition" {
            if let Some(sig) = extract_function(source, child) {
                items.push(Extractable::Function(sig));
            }
        }
    }

    items
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "word")?;
    let name = node_text(name_node, source).to_string();

    Some(FunctionSignature {
        name,
        params: "()".to_string(), // Bash functions don't have typed params
        return_type: None,
        line: node.start_position().row as u32 + 1,
        parent_type: None,
    })
}
