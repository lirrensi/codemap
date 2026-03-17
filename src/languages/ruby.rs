use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "method" => {
                if let Some(sig) = extract_method(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "class" => {
                if let Some((t, class_name)) = extract_class(source, child) {
                    items.push(Extractable::Type(t));
                    extract_class_methods(source, child, &mut items, &class_name);
                }
            }
            "module" => {
                if let Some((t, module_name)) = extract_module(source, child) {
                    items.push(Extractable::Type(t));
                    extract_class_methods(source, child, &mut items, &module_name);
                }
            }
            "singleton_method" => {
                if let Some(sig) = extract_singleton_method(source, child) {
                    // Singleton methods belong to the class/module they're defined in
                    // For simplicity, we'll attach them to the enclosing class/module
                    // In a more sophisticated implementation, we'd track the current scope
                    items.push(Extractable::Function(sig));
                }
            }
            _ => {}
        }
    }

    items
}

fn extract_method(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params = child_by_kind(node, "method_parameters")
        .or_else(|| child_by_kind(node, "parameters"))
        .map(|p| node_text(p, source).to_string())
        .unwrap_or_else(|| "()".to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type: None,
        line: node.start_position().row as u32 + 1,
        parent_type: None,
    })
}

fn extract_class(source: &str, node: Node) -> Option<(NamedType, String)> {
    let name_node = child_by_kind(node, "constant")?;
    let name = node_text(name_node, source).to_string();
    Some((
        NamedType {
            name: name.clone(),
            kind: TypeKind::Class,
        },
        name,
    ))
}

fn extract_module(source: &str, node: Node) -> Option<(NamedType, String)> {
    let name_node = child_by_kind(node, "constant")?;
    let name = node_text(name_node, source).to_string();
    Some((
        NamedType {
            name: name.clone(),
            kind: TypeKind::Module,
        },
        name,
    ))
}

fn extract_singleton_method(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = format!("self.{}", node_text(name_node, source));

    let params = child_by_kind(node, "method_parameters")
        .or_else(|| child_by_kind(node, "parameters"))
        .map(|p| node_text(p, source).to_string())
        .unwrap_or_else(|| "()".to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type: None,
        line: node.start_position().row as u32 + 1,
        parent_type: None,
    })
}

fn extract_class_methods(
    source: &str,
    class_node: Node,
    items: &mut Vec<Extractable>,
    parent_type: &str,
) {
    let mut cursor = class_node.walk();
    for child in class_node.children(&mut cursor) {
        match child.kind() {
            "method" | "singleton_method" => {
                if let Some(sig) = extract_method(source, child) {
                    items.push(Extractable::Function(FunctionSignature {
                        parent_type: Some(parent_type.to_string()),
                        ..sig
                    }));
                }
            }
            "body_statement" => {
                // Methods are nested inside body_statement
                extract_body_methods(source, child, items, parent_type);
            }
            _ => {}
        }
    }
}

fn extract_body_methods(
    source: &str,
    body_node: Node,
    items: &mut Vec<Extractable>,
    parent_type: &str,
) {
    let mut cursor = body_node.walk();
    for child in body_node.children(&mut cursor) {
        match child.kind() {
            "method" => {
                if let Some(sig) = extract_method(source, child) {
                    items.push(Extractable::Function(FunctionSignature {
                        parent_type: Some(parent_type.to_string()),
                        ..sig
                    }));
                }
            }
            "singleton_method" => {
                if let Some(sig) = extract_singleton_method(source, child) {
                    items.push(Extractable::Function(FunctionSignature {
                        parent_type: Some(parent_type.to_string()),
                        ..sig
                    }));
                }
            }
            _ => {}
        }
    }
}
