use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "function_item" => {
                if let Some(sig) = extract_function(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "struct_item" => {
                if let Some(t) = extract_named_type(source, child, TypeKind::Struct) {
                    items.push(Extractable::Type(t));
                }
            }
            "enum_item" => {
                if let Some(t) = extract_named_type(source, child, TypeKind::Enum) {
                    items.push(Extractable::Type(t));
                }
            }
            "trait_item" => {
                if let Some(t) = extract_named_type(source, child, TypeKind::Trait) {
                    items.push(Extractable::Type(t));
                }
            }
            "type_item" => {
                if let Some(t) = extract_named_type(source, child, TypeKind::TypeAlias) {
                    items.push(Extractable::Type(t));
                }
            }
            "impl_item" => {
                extract_impl(source, child, &mut items);
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

    let return_type = child_by_kind(node, "return_type").map(|rt| {
        let text = node_text(rt, source);
        // Strip the leading "->"
        text.trim_start_matches("->").trim().to_string()
    });

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
    })
}

fn extract_named_type(source: &str, node: Node, kind: TypeKind) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType { name, kind })
}

fn extract_impl(source: &str, node: Node, items: &mut Vec<Extractable>) {
    // Extract methods from impl blocks
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "declaration_list" {
            let mut method_cursor = child.walk();
            for method_child in child.children(&mut method_cursor) {
                if method_child.kind() == "function_item" {
                    if let Some(mut sig) = extract_function(source, method_child) {
                        // Prefix with `self` hint for methods
                        if has_self_param(source, method_child) {
                            sig.params = insert_self(&sig.params);
                        }
                        items.push(Extractable::Function(sig));
                    }
                }
            }
        }
    }
}

fn has_self_param(source: &str, node: Node) -> bool {
    if let Some(params) = child_by_kind(node, "parameters") {
        let text = node_text(params, source);
        return text.contains("self");
    }
    false
}

fn insert_self(params: &str) -> String {
    // If params is "()", make it "(self)"
    // If params is "(x: i32)", make it "(self, x: i32)"
    let inner = params.trim_matches(|c| c == '(' || c == ')');
    if inner.trim().is_empty() {
        "(self)".to_string()
    } else {
        format!("(self, {})", inner.trim())
    }
}
