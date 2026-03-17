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
                if let Some((t, class_name)) = extract_class(source, child) {
                    items.push(Extractable::Type(t));
                    extract_class_methods(source, child, &mut items, &class_name);
                }
            }
            "interface_declaration" => {
                if let Some((t, interface_name)) = extract_interface(source, child) {
                    items.push(Extractable::Type(t));
                    // Interfaces can have methods too
                    extract_class_methods(source, child, &mut items, &interface_name);
                }
            }
            "type_alias_declaration" => {
                if let Some((t, type_name)) = extract_type_alias(source, child) {
                    items.push(Extractable::Type(t));
                    // Type aliases don't typically have methods, but we'll keep the pattern consistent
                }
            }
            "enum_declaration" => {
                if let Some((t, enum_name)) = extract_enum(source, child) {
                    items.push(Extractable::Type(t));
                    // Enums can have methods too in TypeScript
                    extract_class_methods(source, child, &mut items, &enum_name);
                }
            }
            "lexical_declaration" | "variable_declaration" => {
                extract_arrow_or_function_var(source, child, &mut items);
            }
            _ => {}
        }
    }

    items
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(node, "formal_parameters")?;
    let params = node_text(params_node, source).to_string();

    let return_type = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "type_annotation")
        .map(|t| {
            node_text(t, source)
                .trim_start_matches(':')
                .trim()
                .to_string()
        });

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
        parent_type: None,
    })
}

fn extract_class(source: &str, node: Node) -> Option<(NamedType, String)> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some((
        NamedType {
            name: name.clone(),
            kind: TypeKind::Class,
        },
        name,
    ))
}

fn extract_interface(source: &str, node: Node) -> Option<(NamedType, String)> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some((
        NamedType {
            name: name.clone(),
            kind: TypeKind::Interface,
        },
        name,
    ))
}

fn extract_type_alias(source: &str, node: Node) -> Option<(NamedType, String)> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some((
        NamedType {
            name: name.clone(),
            kind: TypeKind::TypeAlias,
        },
        name,
    ))
}

fn extract_enum(source: &str, node: Node) -> Option<(NamedType, String)> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some((
        NamedType {
            name: name.clone(),
            kind: TypeKind::Enum,
        },
        name,
    ))
}

fn extract_class_methods(
    source: &str,
    class_node: Node,
    items: &mut Vec<Extractable>,
    parent_type: &str,
) {
    let mut cursor = class_node.walk();
    for child in class_node.children(&mut cursor) {
        if child.kind() == "class_body" {
            let mut body_cursor = child.walk();
            for member in child.children(&mut body_cursor) {
                if member.kind() == "method_definition" {
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
    let name_node = child_by_kind(node, "property_identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(node, "formal_parameters")?;
    let params = node_text(params_node, source).to_string();
    // TS methods use implicit `this`, not explicit self param - keep params as-is

    let return_type = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "type_annotation")
        .map(|t| {
            node_text(t, source)
                .trim_start_matches(':')
                .trim()
                .to_string()
        });

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
        parent_type: None,
    })
}

fn extract_arrow_or_function_var(source: &str, node: Node, items: &mut Vec<Extractable>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "variable_declarator" {
            let name = child_by_kind(child, "identifier").map(|n| node_text(n, source).to_string());
            if let Some(name) = name {
                let mut dc = child.walk();
                for c in child.children(&mut dc) {
                    if c.kind() == "arrow_function" || c.kind() == "function_expression" {
                        let params_node = child_by_kind(c, "formal_parameters");
                        let params = params_node
                            .map(|p| node_text(p, source).to_string())
                            .unwrap_or_else(|| "()".to_string());
                        let return_type = c
                            .children(&mut c.walk())
                            .find(|n| n.kind() == "type_annotation")
                            .map(|t| {
                                node_text(t, source)
                                    .trim_start_matches(':')
                                    .trim()
                                    .to_string()
                            });
                        items.push(Extractable::Function(FunctionSignature {
                            name,
                            params,
                            return_type,
                            line: node.start_position().row as u32 + 1,
                            parent_type: None,
                        }));
                        return;
                    }
                }
            }
        }
    }
}
