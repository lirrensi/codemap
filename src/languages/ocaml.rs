use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "value_definition" => {
                if let Some(sig) = extract_value(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "type_definition" => {
                if let Some(t) = extract_type_def(source, child) {
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

fn extract_value(source: &str, node: Node) -> Option<FunctionSignature> {
    // value_definition > let_binding > value_name
    let binding = child_by_kind(node, "let_binding")?;
    let name_node = child_by_kind(binding, "value_name")?;
    let name = node_text(name_node, source).to_string();

    // Extract params from parameter children of let_binding
    let params: Vec<&str> = binding
        .children(&mut binding.walk())
        .filter(|c| c.kind() == "parameter")
        .map(|p| node_text(p, source))
        .collect();

    let params_str = if params.is_empty() {
        "()".to_string()
    } else {
        format!("({})", params.join(", "))
    };

    Some(FunctionSignature {
        name,
        params: params_str,
        return_type: None,
        line: node.start_position().row as u32 + 1,
    })
}

fn extract_type_def(source: &str, node: Node) -> Option<NamedType> {
    // type_definition > type_binding > type_constructor
    let binding = child_by_kind(node, "type_binding")?;
    let name_node = child_by_kind(binding, "type_constructor")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}

fn extract_module(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "module_name")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Module,
    })
}
