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
                if let Some(t) = extract_class(source, child) {
                    items.push(Extractable::Type(t));
                    extract_class_methods(source, child, &mut items);
                }
            }
            "interface_declaration" => {
                if let Some(t) = extract_interface(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "type_alias_declaration" => {
                if let Some(t) = extract_type_alias(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "enum_declaration" => {
                if let Some(t) = extract_enum(source, child) {
                    items.push(Extractable::Type(t));
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
    })
}

fn extract_class(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Class,
    })
}

fn extract_interface(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Interface,
    })
}

fn extract_type_alias(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
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

fn extract_class_methods(source: &str, class_node: Node, items: &mut Vec<Extractable>) {
    let mut cursor = class_node.walk();
    for child in class_node.children(&mut cursor) {
        if child.kind() == "class_body" {
            let mut body_cursor = child.walk();
            for member in child.children(&mut body_cursor) {
                if member.kind() == "method_definition" {
                    if let Some(sig) = extract_method(source, member) {
                        items.push(Extractable::Function(sig));
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
                        }));
                        return;
                    }
                }
            }
        }
    }
}
