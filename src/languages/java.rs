use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        if child.kind() == "class_declaration" {
            if let Some(t) = extract_class(source, child) {
                items.push(Extractable::Type(t));
            }
            extract_class_body(source, child, &mut items);
        } else if child.kind() == "interface_declaration" {
            if let Some(t) = extract_interface(source, child) {
                items.push(Extractable::Type(t));
            }
        } else if child.kind() == "enum_declaration" {
            if let Some(t) = extract_enum(source, child) {
                items.push(Extractable::Type(t));
            }
        }
    }

    items
}

fn extract_class(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Class,
    })
}

fn extract_interface(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Interface,
    })
}

fn extract_enum(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Enum,
    })
}

fn extract_class_body(source: &str, class_node: Node, items: &mut Vec<Extractable>) {
    let mut cursor = class_node.walk();
    for child in class_node.children(&mut cursor) {
        if child.kind() == "class_body" {
            let mut body_cursor = child.walk();
            for member in child.children(&mut body_cursor) {
                if member.kind() == "method_declaration"
                    || member.kind() == "constructor_declaration"
                {
                    if let Some(sig) = extract_method(source, member) {
                        items.push(Extractable::Function(sig));
                    }
                }
            }
        }
    }
}

fn extract_method(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(node, "formal_parameters")?;
    let params = node_text(params_node, source).to_string();

    let return_type = node
        .children(&mut node.walk())
        .find(|c| {
            matches!(
                c.kind(),
                "type_identifier"
                    | "void_type"
                    | "generic_type"
                    | "integral_type"
                    | "floating_point_type"
                    | "boolean_type"
            )
        })
        .map(|t| node_text(t, source).to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
    })
}
