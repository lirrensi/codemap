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
                // Kotlin uses class_declaration for class, data class, enum class, interface
                extract_class_declaration(source, child, &mut items);
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

fn extract_class_declaration(source: &str, node: Node, items: &mut Vec<Extractable>) {
    let mut cursor = node.walk();
    let mut kind = TypeKind::Class;
    let mut has_interface = false;
    let mut has_enum = false;
    let mut parent_type_name: Option<String> = None;

    for child in node.children(&mut cursor) {
        match child.kind() {
            "modifiers" => {
                // Check for data modifier
                let mut mc = child.walk();
                for m in child.children(&mut mc) {
                    if m.kind() == "class_modifier" {
                        let mut cc = m.walk();
                        for c in m.children(&mut cc) {
                            if node_text(c, source) == "enum" {
                                has_enum = true;
                            }
                        }
                    }
                }
            }
            "interface" => {
                has_interface = true;
            }
            "enum" => {
                has_enum = true;
            }
            "identifier" | "simple_identifier" => {
                let name = node_text(child, source).to_string();
                if has_interface {
                    kind = TypeKind::Interface;
                } else if has_enum {
                    kind = TypeKind::Enum;
                }
                items.push(Extractable::Type(NamedType {
                    name: name.clone(),
                    kind: kind.clone(),
                }));
                parent_type_name = Some(name.clone());
            }
            "class_body" => {
                extract_class_body(source, child, items, parent_type_name.clone());
            }
            _ => {}
        }
    }
}

fn extract_class_body(
    source: &str,
    body_node: Node,
    items: &mut Vec<Extractable>,
    parent_type_name: Option<String>,
) {
    let mut cursor = body_node.walk();
    for child in body_node.children(&mut cursor) {
        if child.kind() == "function_declaration"
            && let Some(mut sig) = extract_function(source, child)
        {
            sig.parent_type = parent_type_name.clone();
            items.push(Extractable::Function(sig));
        }
    }
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node =
        child_by_kind(node, "simple_identifier").or_else(|| child_by_kind(node, "identifier"))?;
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
        parent_type: None,
    })
}

fn extract_named(source: &str, node: Node, kind: TypeKind) -> Option<NamedType> {
    let name_node =
        child_by_kind(node, "simple_identifier").or_else(|| child_by_kind(node, "identifier"))?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType { name, kind })
}
