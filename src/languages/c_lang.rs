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
            "struct_specifier" => {
                if let Some(t) = extract_struct(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "enum_specifier" => {
                if let Some(t) = extract_enum(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "type_definition" => {
                if let Some(t) = extract_typedef(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            _ => {}
        }
    }

    items
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    // In C, function_definition has a declarator child with the function_declarator
    let declarator = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "function_declarator")?;

    let name_node = child_by_kind(declarator, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(declarator, "parameter_list")?;
    let params = node_text(params_node, source).to_string();

    // Get return type from the declaration specifiers before the declarator
    let return_type = node
        .children(&mut node.walk())
        .find(|c| {
            c.kind() == "primitive_type"
                || c.kind() == "type_identifier"
                || c.kind() == "sized_type_specifier"
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

fn extract_struct(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Struct,
    })
}

fn extract_enum(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Enum,
    })
}

fn extract_typedef(source: &str, node: Node) -> Option<NamedType> {
    let name_node = node
        .children(&mut node.walk())
        .filter(|c| c.kind() == "type_identifier")
        .last()?; // The last type_identifier is the new name
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}
