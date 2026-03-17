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
            "method_declaration" => {
                if let Some(sig) = extract_method(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "type_declaration" => {
                extract_type_declaration(source, child, &mut items);
            }
            _ => {}
        }
    }

    items
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(node, "parameter_list")?;
    let params = node_text(params_node, source).to_string();

    let return_type = node
        .children(&mut node.walk())
        .filter(|c| c.kind() == "parameter_list")
        .nth(1) // second parameter_list is return type
        .map(|rt| node_text(rt, source).to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
        parent_type: None,
    })
}

fn extract_method(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "field_identifier")?;
    let name = node_text(name_node, source).to_string();

    // Get receiver (first parameter list)
    let receiver_node = child_by_kind(node, "parameter_list");
    let receiver_text = receiver_node.map(|p| node_text(p, source).to_string());

    // Extract type name from receiver like "(t *MyType)" -> "*MyType" or "(t MyType)" -> "MyType"
    let parent_type = receiver_text.clone().and_then(|text| {
        // Remove parentheses and split by space to get type
        let inner = text.trim_start_matches('(').trim_end_matches(')');
        let parts: Vec<&str> = inner.split_whitespace().collect();
        // Skip the parameter name (first part), take the type (second part)
        if parts.len() >= 2 {
            Some(parts[1].to_string())
        } else if !parts.is_empty() {
            // Handle case like "(MyType)" where there's no parameter name
            Some(parts[0].to_string())
        } else {
            None
        }
    });

    let params_node = node
        .children(&mut node.walk())
        .filter(|c| c.kind() == "parameter_list")
        .nth(1); // second param list is actual params

    let params = params_node
        .map(|p| node_text(p, source).to_string())
        .unwrap_or_else(|| "()".to_string());

    let return_type = node
        .children(&mut node.walk())
        .filter(|c| c.kind() == "parameter_list")
        .nth(2)
        .map(|rt| node_text(rt, source).to_string());

    // Prepend receiver to params
    let full_params = match receiver_text {
        Some(r) => format!("{}{}", r, params),
        None => params,
    };

    Some(FunctionSignature {
        name,
        params: full_params,
        return_type,
        line: node.start_position().row as u32 + 1,
        parent_type,
    })
}

fn extract_type_declaration(source: &str, node: Node, items: &mut Vec<Extractable>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "type_spec" {
            let name =
                child_by_kind(child, "type_identifier").map(|n| node_text(n, source).to_string());

            if let Some(name) = name {
                // Determine the type kind
                let mut tc = child.walk();
                for c in child.children(&mut tc) {
                    let kind = match c.kind() {
                        "struct_type" => Some(TypeKind::Struct),
                        "interface_type" => Some(TypeKind::Interface),
                        _ => None,
                    };
                    if let Some(kind) = kind {
                        items.push(Extractable::Type(NamedType {
                            name: name.clone(),
                            kind,
                        }));
                        break;
                    }
                }
            }
        }
    }
}
