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
            "class_declaration" => {
                if let Some((t, class_name)) = extract_class(source, child) {
                    items.push(Extractable::Type(t));
                    extract_class_members(source, child, &mut items, &class_name);
                }
            }
            "interface_declaration" => {
                if let Some((t, interface_name)) = extract_named(source, child, TypeKind::Interface)
                {
                    items.push(Extractable::Type(t));
                    // Interfaces can have methods too in PHP
                    extract_class_members(source, child, &mut items, &interface_name);
                }
            }
            "trait_declaration" => {
                if let Some((t, trait_name)) = extract_named(source, child, TypeKind::Trait) {
                    items.push(Extractable::Type(t));
                    // Traits can have methods too
                    extract_class_members(source, child, &mut items, &trait_name);
                }
            }
            "enum_declaration" => {
                if let Some((t, enum_name)) = extract_named(source, child, TypeKind::Enum) {
                    items.push(Extractable::Type(t));
                    // Enums can have methods too in PHP
                    extract_class_members(source, child, &mut items, &enum_name);
                }
            }
            _ => {}
        }
    }
    items
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "name")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(node, "formal_parameters")?;
    let params = node_text(params_node, source).to_string();

    let return_type = node
        .children(&mut node.walk())
        .find(|c| {
            c.kind() == "primitive_type" || c.kind() == "named_type" || c.kind() == "union_type"
        })
        .map(|t| node_text(t, source).to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
        parent_type: None,
    })
}

fn extract_class(source: &str, node: Node) -> Option<(NamedType, String)> {
    let name_node = child_by_kind(node, "name")?;
    let name = node_text(name_node, source).to_string();
    Some((
        NamedType {
            name: name.clone(),
            kind: TypeKind::Class,
        },
        name,
    ))
}

fn extract_named(source: &str, node: Node, kind: TypeKind) -> Option<(NamedType, String)> {
    let name_node = child_by_kind(node, "name")?;
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
        if child.kind() == "declaration_list" {
            let mut dc = child.walk();
            for member in child.children(&mut dc) {
                if member.kind() == "method_declaration" {
                    if let Some(sig) = extract_function(source, member) {
                        items.push(Extractable::Function(FunctionSignature {
                            parent_type: Some(parent_type.to_string()),
                            ..sig
                        }));
                    }
                }
            }
        }
    }
}
