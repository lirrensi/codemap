use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "function_declaration" => {
                if let Some(sig) = extract_function(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "class_declaration" => {
                if let Some(t) = extract_named(source, child, TypeKind::Class) {
                    items.push(Extractable::Type(t));
                }
            }
            "object_declaration" => {
                if let Some(t) = extract_named(source, child, TypeKind::Struct) {
                    items.push(Extractable::Type(t));
                }
            }
            "interface_declaration" => {
                if let Some(t) = extract_named(source, child, TypeKind::Interface) {
                    items.push(Extractable::Type(t));
                }
            }
            "type_alias" => {
                if let Some(t) = extract_named(source, child, TypeKind::TypeAlias) {
                    items.push(Extractable::Type(t));
                }
            }
            "enum_class" => {
                if let Some(t) = extract_named(source, child, TypeKind::Enum) {
                    items.push(Extractable::Type(t));
                }
            }
            _ => {}
        }
    }

    items
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "simple_identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(node, "function_value_parameters")?;
    let params = node_text(params_node, source).to_string();

    let return_type = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "user_type" || c.kind() == "nullable_type")
        .map(|t| node_text(t, source).to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
    })
}

fn extract_named(source: &str, node: Node, kind: TypeKind) -> Option<NamedType> {
    let name_node = child_by_kind(node, "simple_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType { name, kind })
}
