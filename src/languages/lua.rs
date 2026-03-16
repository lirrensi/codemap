use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        if child.kind() == "function_declaration" || child.kind() == "function_definition" {
            if let Some(sig) = extract_function(source, child) {
                items.push(Extractable::Function(sig));
            }
        } else if child.kind() == "variable_declaration" {
            // local function name() ... end
            let mut vc = child.walk();
            for c in child.children(&mut vc) {
                if c.kind() == "function_definition" {
                    if let Some(name) =
                        child_by_kind(child, "identifier").map(|n| node_text(n, source).to_string())
                    {
                        let params = child_by_kind(c, "parameters")
                            .map(|p| node_text(p, source).to_string())
                            .unwrap_or_else(|| "()".to_string());
                        items.push(Extractable::Function(FunctionSignature {
                            name,
                            params,
                            return_type: None,
                            line: child.start_position().row as u32 + 1,
                        }));
                    }
                }
            }
        }
    }

    items
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params = child_by_kind(node, "parameters")
        .map(|p| node_text(p, source).to_string())
        .unwrap_or_else(|| "()".to_string());

    Some(FunctionSignature {
        name,
        params,
        return_type: None,
        line: node.start_position().row as u32 + 1,
    })
}
