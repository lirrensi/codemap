use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use std::collections::HashMap;
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();

    // First pass: collect all @spec typespecs (name -> return_type)
    let specs = collect_specs(source, root);

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        match child.kind() {
            "call" => {
                if let Some(extractable) = extract_call(source, child, &specs) {
                    items.push(extractable);
                }
                extract_module_functions(source, child, &mut items, &specs);
            }
            _ => {}
        }
    }

    items
}

/// Collect all @spec declarations mapping function name -> return type string
fn collect_specs(source: &str, root: Node) -> HashMap<String, String> {
    let mut specs = HashMap::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        collect_specs_from_node(source, child, &mut specs);
    }
    specs
}

fn collect_specs_from_node(source: &str, node: Node, specs: &mut HashMap<String, String>) {
    if node.kind() == "unary_operator" {
        // @spec foo(...) :: ReturnType
        // unary_operator > @ + call (identifier "spec" + arguments)
        if let Some(spec_type) = extract_spec(source, node) {
            specs.insert(spec_type.0, spec_type.1);
        }
    }
    // Recurse into children (do_blocks, nested modules, etc.)
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_specs_from_node(source, child, specs);
    }
}

/// Extract (function_name, return_type) from a @spec unary_operator node
fn extract_spec(source: &str, node: Node) -> Option<(String, String)> {
    // Structure: unary_operator > call (identifier "spec" + arguments)
    let call = child_by_kind(node, "call")?;
    let ident = child_by_kind(call, "identifier")?;
    if node_text(ident, source) != "spec" {
        return None;
    }

    let args = child_by_kind(call, "arguments")?;
    // arguments contains a binary_operator: foo(params) :: ReturnType
    let binop = child_by_kind(args, "binary_operator")?;
    if node_text(child_by_kind(binop, "::")?, source) != "::" {
        return None;
    }

    // Left side: call (function name + type params)
    // Right side: return type
    let left = binop.child(0)?;
    let name = if left.kind() == "call" {
        let name_node = child_by_kind(left, "identifier")?;
        node_text(name_node, source).to_string()
    } else {
        node_text(left, source).to_string()
    };

    // Return type is everything after ::
    let mut cursor = binop.walk();
    let children: Vec<Node> = binop.children(&mut cursor).collect();
    let mut found_colon = false;
    let mut return_type_parts = Vec::new();
    for child in &children {
        if child.kind() == "::" {
            found_colon = true;
            continue;
        }
        if found_colon {
            return_type_parts.push(node_text(*child, source));
        }
    }

    let return_type = if return_type_parts.is_empty() {
        return None;
    } else {
        return_type_parts.join("")
    };

    Some((name, return_type))
}

fn extract_call(source: &str, node: Node, specs: &HashMap<String, String>) -> Option<Extractable> {
    let target = child_by_kind(node, "identifier")?;
    let target_text = node_text(target, source);

    match target_text {
        "def" | "defp" => {
            let args = child_by_kind(node, "arguments")?;
            let mut ac = args.walk();
            for arg in args.children(&mut ac) {
                if arg.kind() == "call" {
                    let name_node = child_by_kind(arg, "identifier")?;
                    let name = node_text(name_node, source).to_string();
                    let params = child_by_kind(arg, "arguments")
                        .map(|p| node_text(p, source).to_string())
                        .unwrap_or_else(|| "()".to_string());

                    // Look up return type from @spec
                    let return_type = specs.get(&name).cloned();

                    return Some(Extractable::Function(FunctionSignature {
                        name,
                        params,
                        return_type,
                        line: node.start_position().row as u32 + 1,
                    }));
                }
            }
            None
        }
        "defmodule" => {
            let args = child_by_kind(node, "arguments")?;
            let mut ac = args.walk();
            for arg in args.children(&mut ac) {
                if arg.kind() == "alias" {
                    let name = node_text(arg, source).to_string();
                    return Some(Extractable::Type(NamedType {
                        name,
                        kind: TypeKind::Module,
                    }));
                }
            }
            None
        }
        _ => None,
    }
}

fn extract_module_functions(
    source: &str,
    node: Node,
    items: &mut Vec<Extractable>,
    specs: &HashMap<String, String>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "do_block" {
            let mut dc = child.walk();
            for block_child in child.children(&mut dc) {
                if block_child.kind() == "call" {
                    if let Some(extractable) = extract_call(source, block_child, specs) {
                        items.push(extractable);
                    }
                }
            }
        }
    }
}
