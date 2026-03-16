use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "call" => {
                // def, defp, defmodule, defstruct, etc.
                if let Some(extractable) = extract_call(source, child) {
                    items.push(extractable);
                }
                // Also extract functions from inside defmodule do_blocks
                extract_module_functions(source, child, &mut items);
            }
            _ => {}
        }
    }

    items
}

fn extract_call(source: &str, node: Node) -> Option<Extractable> {
    let target = child_by_kind(node, "identifier")?;
    let target_text = node_text(target, source);

    match target_text {
        "def" | "defp" => {
            // Find the call node (function name + params)
            let args = child_by_kind(node, "arguments")?;
            let mut ac = args.walk();
            for arg in args.children(&mut ac) {
                if arg.kind() == "call" {
                    let name_node = child_by_kind(arg, "identifier")?;
                    let name = node_text(name_node, source).to_string();
                    let params = child_by_kind(arg, "arguments")
                        .map(|p| node_text(p, source).to_string())
                        .unwrap_or_else(|| "()".to_string());
                    return Some(Extractable::Function(FunctionSignature {
                        name,
                        params,
                        return_type: None,
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

/// Extract functions from inside a defmodule's do_block
pub fn extract_module_functions(source: &str, node: Node, items: &mut Vec<Extractable>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "do_block" {
            let mut dc = child.walk();
            for block_child in child.children(&mut dc) {
                if block_child.kind() == "call" {
                    if let Some(extractable) = extract_call(source, block_child) {
                        items.push(extractable);
                    }
                }
            }
        }
    }
}
