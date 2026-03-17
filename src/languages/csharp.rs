use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "class_declaration" => {
                if let Some((t, type_name)) = extract_named(source, child, TypeKind::Class) {
                    items.push(Extractable::Type(t));
                    extract_members(source, child, &mut items, &type_name);
                }
            }
            "interface_declaration" => {
                if let Some((t, type_name)) = extract_named(source, child, TypeKind::Interface) {
                    items.push(Extractable::Type(t));
                    extract_members(source, child, &mut items, &type_name);
                }
            }
            "struct_declaration" => {
                if let Some((t, type_name)) = extract_named(source, child, TypeKind::Struct) {
                    items.push(Extractable::Type(t));
                    extract_members(source, child, &mut items, &type_name);
                }
            }
            "enum_declaration" => {
                if let Some((t, type_name)) = extract_named(source, child, TypeKind::Enum) {
                    items.push(Extractable::Type(t));
                    // Enums can have methods too in C#
                    extract_members(source, child, &mut items, &type_name);
                }
            }
            "record_declaration" => {
                if let Some((t, type_name)) = extract_named(source, child, TypeKind::Struct) {
                    items.push(Extractable::Type(t));
                    // Records can have methods too
                    extract_members(source, child, &mut items, &type_name);
                }
            }
            _ => {}
        }
    }

    items
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

fn extract_members(source: &str, parent: Node, items: &mut Vec<Extractable>, parent_type: &str) {
    let mut cursor = parent.walk();
    for child in parent.children(&mut cursor) {
        if child.kind() == "declaration_list" {
            let mut dc = child.walk();
            for member in child.children(&mut dc) {
                if member.kind() == "method_declaration"
                    || member.kind() == "constructor_declaration"
                {
                    if let Some(sig) = extract_method(source, member) {
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

fn extract_method(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(node, "parameter_list")?;
    let params = node_text(params_node, source).to_string();

    let return_type = node
        .children(&mut node.walk())
        .find(|c| matches!(c.kind(), "predefined_type" | "identifier" | "generic_name"))
        .map(|t| node_text(t, source).to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
        parent_type: None,
    })
}
