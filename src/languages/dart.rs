use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "function_signature" | "method_signature" => {
                if let Some(sig) = extract_function(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "class_declaration" | "class_definition" => {
                if let Some((t, type_name)) = extract_named(source, child, TypeKind::Class) {
                    items.push(Extractable::Type(t));
                    extract_class_members(source, child, &mut items, &type_name);
                }
            }
            "mixin_declaration" => {
                if let Some((t, type_name)) = extract_named(source, child, TypeKind::Trait) {
                    items.push(Extractable::Type(t));
                    extract_class_members(source, child, &mut items, &type_name);
                }
            }
            "enum_declaration" => {
                if let Some((t, type_name)) = extract_named(source, child, TypeKind::Enum) {
                    items.push(Extractable::Type(t));
                    // Enums can have methods too in Dart
                    extract_class_members(source, child, &mut items, &type_name);
                }
            }
            "type_alias" => {
                if let Some((t, type_name)) = extract_named(source, child, TypeKind::TypeAlias) {
                    items.push(Extractable::Type(t));
                    // Type aliases don't typically have methods, but we'll keep the pattern consistent
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

    let params_node = child_by_kind(node, "formal_parameter_list")?;
    let params = node_text(params_node, source).to_string();

    let return_type = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "type_identifier" || c.kind() == "void_type")
        .map(|t| node_text(t, source).to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
        parent_type: None,
    })
}

fn extract_named(source: &str, node: Node, kind: TypeKind) -> Option<(NamedType, String)> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some((
        NamedType {
            name: name.clone(),
            kind,
        },
        name,
    ))
}

fn extract_class_members(
    source: &str,
    class_node: Node,
    items: &mut Vec<Extractable>,
    parent_type: &str,
) {
    let mut cursor = class_node.walk();
    for child in class_node.children(&mut cursor) {
        if child.kind() == "class_body" {
            let mut bc = child.walk();
            for member in child.children(&mut bc) {
                match member.kind() {
                    "method_signature" | "function_signature" => {
                        if let Some(sig) = extract_function(source, member) {
                            items.push(Extractable::Function(FunctionSignature {
                                parent_type: Some(parent_type.to_string()),
                                ..sig
                            }));
                        }
                    }
                    "constructor_signature" => {
                        if let Some(sig) = extract_constructor(source, member, parent_type) {
                            items.push(Extractable::Function(sig));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn extract_constructor(source: &str, node: Node, parent_type: &str) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params = child_by_kind(node, "formal_parameter_list")
        .map(|p| node_text(p, source).to_string())
        .unwrap_or_else(|| "()".to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type: None,
        line: node.start_position().row as u32 + 1,
        parent_type: Some(parent_type.to_string()),
    })
}
