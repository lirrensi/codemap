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
