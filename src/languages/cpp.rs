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
            "struct_specifier" | "class_specifier" => {
                if let Some(t) = extract_class_or_struct(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "enum_specifier" => {
                if let Some(t) = extract_enum(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "namespace_definition" => {
                extract_namespace(source, child, &mut items);
            }
            "declaration" => {
                // Could be a function declaration (prototype) or typedef
                if let Some(sig) = extract_function_declaration(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            _ => {}
        }
    }

    items
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let declarator = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "function_declarator")?;

    let name_node = child_by_kind(declarator, "identifier")
        .or_else(|| child_by_kind(declarator, "field_identifier"))?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(declarator, "parameter_list")?;
    let params = node_text(params_node, source).to_string();

    let return_type = node
        .children(&mut node.walk())
        .find(|c| {
            matches!(
                c.kind(),
                "primitive_type"
                    | "type_identifier"
                    | "sized_type_specifier"
                    | "qualified_identifier"
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

fn extract_function_declaration(source: &str, node: Node) -> Option<FunctionSignature> {
    let declarator = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "function_declarator")?;

    let name_node = child_by_kind(declarator, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(declarator, "parameter_list")?;
    let params = node_text(params_node, source).to_string();

    let return_type = node
        .children(&mut node.walk())
        .find(|c| {
            matches!(
                c.kind(),
                "primitive_type" | "type_identifier" | "sized_type_specifier"
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

fn extract_class_or_struct(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    let kind = if node.kind() == "class_specifier" {
        TypeKind::Class
    } else {
        TypeKind::Struct
    };
    Some(NamedType { name, kind })
}

fn extract_enum(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Enum,
    })
}

fn extract_namespace(source: &str, node: Node, items: &mut Vec<Extractable>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "function_definition" {
            if let Some(sig) = extract_function(source, child) {
                items.push(Extractable::Function(sig));
            }
        }
    }
}
