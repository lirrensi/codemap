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
            "short_function_definition" => {
                if let Some(sig) = extract_short_function(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "struct_definition" => {
                if let Some(t) = extract_struct(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "abstract_definition" => {
                if let Some(t) = extract_abstract(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "module_definition" => {
                if let Some(t) = extract_module(source, child) {
                    items.push(Extractable::Type(t));
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

    let params_node = child_by_kind(node, "parameter_list")?;
    let params = node_text(params_node, source).to_string();

    let return_type = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "return_type")
        .map(|t| {
            node_text(t, source)
                .trim_start_matches("::")
                .trim()
                .to_string()
        });

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
    })
}

fn extract_short_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(node, "parameter_list")?;
    let params = node_text(params_node, source).to_string();

    Some(FunctionSignature {
        name,
        params,
        return_type: None,
        line: node.start_position().row as u32 + 1,
    })
}

fn extract_struct(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Struct,
    })
}

fn extract_abstract(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}

fn extract_module(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Module,
    })
}
