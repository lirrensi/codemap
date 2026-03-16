use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "create_function_statement" => {
                if let Some(sig) = extract_create_function(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "create_procedure_statement" => {
                if let Some(sig) = extract_create_procedure(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "create_table_statement" => {
                if let Some(t) = extract_create_table(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "create_view_statement" => {
                if let Some(t) = extract_create_view(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "create_type_statement" => {
                if let Some(t) = extract_create_type(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            _ => {}
        }
    }

    items
}

fn extract_create_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "identifier" || c.kind() == "object_reference")?;
    let name = node_text(name_node, source).to_string();

    let params = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "parameter_list" || c.kind() == "create_function_parameters")
        .map(|p| node_text(p, source).to_string())
        .unwrap_or_else(|| "()".to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type: None,
        line: node.start_position().row as u32 + 1,
    })
}

fn extract_create_procedure(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "identifier" || c.kind() == "object_reference")?;
    let name = node_text(name_node, source).to_string();

    let params = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "parameter_list" || c.kind() == "create_function_parameters")
        .map(|p| node_text(p, source).to_string())
        .unwrap_or_else(|| "()".to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type: None,
        line: node.start_position().row as u32 + 1,
    })
}

fn extract_create_table(source: &str, node: Node) -> Option<NamedType> {
    let name_node = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "object_reference" || c.kind() == "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Struct,
    })
}

fn extract_create_view(source: &str, node: Node) -> Option<NamedType> {
    let name_node = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "object_reference" || c.kind() == "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}

fn extract_create_type(source: &str, node: Node) -> Option<NamedType> {
    let name_node = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "identifier" || c.kind() == "object_reference")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}
