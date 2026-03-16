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
            "short_function_definition" => {
                if let Some(sig) = extract_short_function(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "struct_definition" => {
                if let Some(t) = extract_struct(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "abstract_definition" => {
                if let Some(t) = extract_abstract(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "module_definition" => {
                if let Some(t) = extract_module(source, child) {
                    items.push(Extractable::Type(t));
                }
                // Also extract functions defined inside the module
                extract_module_functions(source, child, &mut items);
            }
            "assignment" => {
                // Short function: name(params) = expr
                if let Some(sig) = extract_assignment_function(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            _ => {}
        }
    }

    items
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    // function_definition > signature > [call_expression >] identifier + argument_list
    let sig_node = child_by_kind(node, "signature")?;

    // Name might be directly under signature, or inside a call_expression
    let name = if let Some(call) = child_by_kind(sig_node, "call_expression") {
        let name_node = child_by_kind(call, "identifier")?;
        node_text(name_node, source).to_string()
    } else {
        let name_node = child_by_kind(sig_node, "identifier")?;
        node_text(name_node, source).to_string()
    };

    // Params are in argument_list (not parameter_list)
    let params = child_by_kind(sig_node, "argument_list")
        .or_else(|| {
            child_by_kind(sig_node, "call_expression")
                .and_then(|c| child_by_kind(c, "argument_list"))
        })
        .map(|p| node_text(p, source).to_string())
        .unwrap_or_else(|| "()".to_string());

    let return_type = sig_node
        .children(&mut sig_node.walk())
        .find(|c| c.kind() == "return_type")
        .map(|t| {
            node_text(t, source)
                .trim_start_matches("::")
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

fn extract_short_function(source: &str, node: Node) -> Option<FunctionSignature> {
    // short_function_definition has identifier and parameter_list as direct children
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(node, "parameter_list")?;
    let params = node_text(params_node, source).to_string();

    Some(FunctionSignature {
        name,
        params,
        return_type: None,
        line: node.start_position().row as u32 + 1,
    })
}

/// Extract short function from assignment node: name(params) = expr
fn extract_assignment_function(source: &str, node: Node) -> Option<FunctionSignature> {
    // assignment > call_expression (left side) = expression (right side)
    let left = node.child(0)?;
    if left.kind() == "call_expression" {
        let name_node = child_by_kind(left, "identifier")?;
        let name = node_text(name_node, source).to_string();
        let params = child_by_kind(left, "argument_list")
            .map(|p| node_text(p, source).to_string())
            .unwrap_or_else(|| "()".to_string());
        Some(FunctionSignature {
            name,
            params,
            return_type: None,
            line: node.start_position().row as u32 + 1,
        })
    } else {
        None
    }
}

fn extract_struct(source: &str, node: Node) -> Option<NamedType> {
    // struct_definition > type_head > identifier
    let head = child_by_kind(node, "type_head")?;
    let name_node = child_by_kind(head, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Struct,
    })
}

fn extract_abstract(source: &str, node: Node) -> Option<NamedType> {
    // abstract_definition > type_head > identifier
    let head = child_by_kind(node, "type_head")?;
    let name_node = child_by_kind(head, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}

fn extract_module(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Module,
    })
}

fn extract_module_functions(source: &str, module_node: Node, items: &mut Vec<Extractable>) {
    let mut cursor = module_node.walk();
    for child in module_node.children(&mut cursor) {
        match child.kind() {
            "function_definition" => {
                if let Some(sig) = extract_function(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "short_function_definition" => {
                if let Some(sig) = extract_short_function(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            _ => {}
        }
    }
}
