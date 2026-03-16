use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "function_definition" => {
                if let Some(sig) = extract_function(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "class_definition" => {
                if let Some(t) = extract_class(source, child) {
                    items.push(Extractable::Type(t));
                    // Also extract methods from the class body
                    extract_class_methods(source, child, &mut items);
                }
            }
            _ => {}
        }
    }

    items
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(node, "parameters")?;
    let params = node_text(params_node, source).to_string();

    // Check for return type annotation
    let return_type = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "type")
        .map(|t| node_text(t, source).to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
    })
}

fn extract_class(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Class,
    })
}

fn extract_class_methods(source: &str, class_node: Node, items: &mut Vec<Extractable>) {
    // Find the block (class body)
    let mut cursor = class_node.walk();
    for child in class_node.children(&mut cursor) {
        if child.kind() == "block" {
            let mut block_cursor = child.walk();
            for stmt in child.children(&mut block_cursor) {
                if stmt.kind() == "function_definition" {
                    if let Some(sig) = extract_function(source, stmt) {
                        // Keep params as-is (self/cls is already there from tree-sitter)
                        items.push(Extractable::Function(sig));
                    }
                }
            }
        }
    }
}
