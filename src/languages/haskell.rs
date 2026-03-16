use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "function" => {
                if let Some(sig) = extract_function(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "data_type" => {
                if let Some(t) = extract_data_type(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "newtype" => {
                if let Some(t) = extract_newtype(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "type_alias" => {
                if let Some(t) = extract_type_alias(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "class" => {
                if let Some(t) = extract_typeclass(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            _ => {}
        }
    }

    items
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "variable")?;
    let name = node_text(name_node, source).to_string();

    Some(FunctionSignature {
        name,
        params: "()".to_string(), // Haskell params are complex, simplified
        return_type: None,
        line: node.start_position().row as u32 + 1,
    })
}

fn extract_data_type(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Enum,
    })
}

fn extract_newtype(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}

fn extract_type_alias(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}

fn extract_typeclass(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Trait,
    })
}
