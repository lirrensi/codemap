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
                // Swift uses class_declaration for struct, class, enum
                // The first keyword child determines the actual type
                extract_class_declaration(source, child, &mut items);
            }
            "protocol_declaration" => {
                if let Some(t) = extract_named(source, child, TypeKind::Interface) {
                    items.push(Extractable::Type(t));
                }
            }
            "typealias_declaration" => {
                if let Some(t) = extract_typealias(source, child) {
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
    let mut kind = TypeKind::Class; // default
    let mut parent_type_name: Option<String> = None;

    for child in node.children(&mut cursor) {
        match child.kind() {
            "struct" => kind = TypeKind::Struct,
            "class" => kind = TypeKind::Class,
            "enum" => kind = TypeKind::Enum,
            "type_identifier" => {
                let name = node_text(child, source).to_string();
                items.push(Extractable::Type(NamedType {
                    name: name.clone(),
                    kind: kind.clone(),
                }));
                parent_type_name = Some(name);
            }
            "class_body" | "enum_class_body" => {
                // Extract methods from inside the body
                extract_body_functions(source, child, items, parent_type_name.clone());
            }
            _ => {}
        }
    }
}

fn extract_body_functions(
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
    let name_node = child_by_kind(node, "simple_identifier")?;
    let name = node_text(name_node, source).to_string();

    // Swift params are direct children: ( param, param, ... )
    // Collect text from first ( to last ) or use parameter_clause if present
    let params = if let Some(params_node) = child_by_kind(node, "parameter_clause") {
        node_text(params_node, source).to_string()
    } else {
        // Collect params manually from ( to )
        let mut cursor = node.walk();
        let mut collecting = false;
        let mut depth = 0;
        let mut param_start = None;
        let mut param_end = None;
        for child in node.children(&mut cursor) {
            let text = node_text(child, source);
            if text == "(" && !collecting {
                collecting = true;
                param_start = Some(child.start_byte());
                depth = 1;
            } else if collecting {
                if text == "(" {
                    depth += 1;
                } else if text == ")" {
                    depth -= 1;
                    if depth == 0 {
                        param_end = Some(child.end_byte());
                        break;
                    }
                }
            }
        }
        if let (Some(start), Some(end)) = (param_start, param_end) {
            source[start..end].to_string()
        } else {
            "()".to_string()
        }
    };

    let return_type = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "type_annotation")
        .map(|t| {
            node_text(t, source)
                .trim_start_matches("->")
                .trim()
                .to_string()
        })
        .or_else(|| {
            // Also check for direct user_type after ->
            let mut cursor = node.walk();
            let mut found_arrow = false;
            for child in node.children(&mut cursor) {
                if node_text(child, source) == "->" {
                    found_arrow = true;
                } else if found_arrow && child.kind() == "user_type" {
                    return Some(node_text(child, source).to_string());
                }
            }
            None
        });

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
        parent_type: None,
    })
}

fn extract_named(source: &str, node: Node, kind: TypeKind) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType { name, kind })
}

fn extract_typealias(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}
