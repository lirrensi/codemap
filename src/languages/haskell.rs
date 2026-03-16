use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();

    // Root is "haskell" with children "header" and "declarations"
    // Actual declarations live inside the "declarations" node
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "declarations" {
            extract_declarations(source, child, &mut items);
        } else {
            extract_declaration_node(source, child, &mut items);
        }
    }

    items
}

fn extract_declarations(source: &str, node: tree_sitter::Node, items: &mut Vec<Extractable>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_declaration_node(source, child, items);
    }
}

fn extract_declaration_node(source: &str, node: tree_sitter::Node, items: &mut Vec<Extractable>) {
    match node.kind() {
        "function" => {
            if let Some(sig) = extract_function(source, node) {
                items.push(Extractable::Function(sig));
            }
        }
        "data_type" => {
            if let Some(t) = extract_data_type(source, node) {
                items.push(Extractable::Type(t));
            }
        }
        "newtype" => {
            if let Some(t) = extract_newtype(source, node) {
                items.push(Extractable::Type(t));
            }
        }
        "type_synomym" => {
            if let Some(t) = extract_type_alias(source, node) {
                items.push(Extractable::Type(t));
            }
        }
        "class" => {
            if let Some(t) = extract_typeclass(source, node) {
                items.push(Extractable::Type(t));
            }
        }
        _ => {}
    }
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "variable")?;
    let name = node_text(name_node, source).to_string();

    // Extract params from patterns node
    let params = child_by_kind(node, "patterns")
        .map(|p| {
            let text = node_text(p, source);
            format!("({})", text)
        })
        .unwrap_or_else(|| "()".to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type: None,
        line: node.start_position().row as u32 + 1,
    })
}

fn extract_data_type(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "name")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Enum,
    })
}

fn extract_newtype(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "name")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}

fn extract_type_alias(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "name")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}

fn extract_typeclass(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "name")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Trait,
    })
}
