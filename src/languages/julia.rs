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
                extract_module_functions(source, child, &mut items);
            }
            "assignment" => {
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
    // function_definition > signature > typed_expression
    // typed_expression contains: call_expression (name + params) :: return_type
    let sig_node = child_by_kind(node, "signature")?;
    extract_from_signature(source, sig_node, node)
}

fn extract_from_signature(source: &str, sig_node: Node, parent: Node) -> Option<FunctionSignature> {
    // Look for typed_expression which has call_expression + :: + return_type
    if let Some(typed) = child_by_kind(sig_node, "typed_expression") {
        return extract_from_typed_expression(source, typed, parent);
    }

    // Fallback: direct call_expression or identifier (no type annotation)
    let (name, params) = if let Some(call) = child_by_kind(sig_node, "call_expression") {
        let name_node = child_by_kind(call, "identifier")?;
        let name = node_text(name_node, source).to_string();
        let params = child_by_kind(call, "argument_list")
            .map(|p| node_text(p, source).to_string())
            .unwrap_or_else(|| "()".to_string());
        (name, params)
    } else {
        let name_node = child_by_kind(sig_node, "identifier")?;
        let name = node_text(name_node, source).to_string();
        (name, "()".to_string())
    };

    Some(FunctionSignature {
        name,
        params,
        return_type: None,
        line: parent.start_position().row as u32 + 1,
        parent_type: None,
    })
}

fn extract_from_typed_expression(
    source: &str,
    typed: Node,
    parent: Node,
) -> Option<FunctionSignature> {
    // typed_expression structure:
    //   call_expression (name + params)
    //   ::
    //   identifier (return type)
    let call = child_by_kind(typed, "call_expression")?;
    let name_node = child_by_kind(call, "identifier")?;
    let name = node_text(name_node, source).to_string();
    let params = child_by_kind(call, "argument_list")
        .map(|p| node_text(p, source).to_string())
        .unwrap_or_else(|| "()".to_string());

    // Return type is the node after "::" token
    let return_type = extract_julia_return_type(source, typed);

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: parent.start_position().row as u32 + 1,
        parent_type: None,
    })
}

/// Extract return type from typed_expression: find `::` then grab next sibling
fn extract_julia_return_type(source: &str, typed: Node) -> Option<String> {
    let mut cursor = typed.walk();
    let children: Vec<Node> = typed.children(&mut cursor).collect();

    for i in 0..children.len() {
        if children[i].kind() == "::" && i + 1 < children.len() {
            let type_node = children[i + 1];
            // Could be identifier, parametrized_type_expression, etc.
            return Some(node_text(type_node, source).to_string());
        }
    }
    None
}

fn extract_short_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(node, "parameter_list")?;
    let params = node_text(params_node, source).to_string();

    // Check for return_type child
    let return_type = child_by_kind(node, "return_type").map(|t| {
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
        parent_type: None,
    })
}

/// Extract short function from assignment: name(params)::Type = expr
fn extract_assignment_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let left = node.child(0)?;

    // Case 1: typed_expression wrapping a call_expression
    // typed_expression > call_expression (name + params) :: return_type
    if left.kind() == "typed_expression" {
        return extract_from_typed_expression(source, left, node);
    }

    // Case 2: plain call_expression (no return type)
    if left.kind() == "call_expression" {
        let name_node = child_by_kind(left, "identifier")?;
        let name = node_text(name_node, source).to_string();
        let params = child_by_kind(left, "argument_list")
            .map(|p| node_text(p, source).to_string())
            .unwrap_or_else(|| "()".to_string());
        return Some(FunctionSignature {
            name,
            params,
            return_type: None,
            line: node.start_position().row as u32 + 1,
            parent_type: None,
        });
    }
    None
}

fn extract_struct(source: &str, node: Node) -> Option<NamedType> {
    let head = child_by_kind(node, "type_head")?;
    let name_node = child_by_kind(head, "identifier")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Struct,
    })
}

fn extract_abstract(source: &str, node: Node) -> Option<NamedType> {
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
            "assignment" => {
                if let Some(sig) = extract_assignment_function(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            _ => {}
        }
    }
}
