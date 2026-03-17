use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature};

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        // R grammar uses "binary_operator" for <- assignments
        if child.kind() == "binary_operator"
            || child.kind() == "assignment"
            || child.kind() == "left_assignment"
            || child.kind() == "equals_assignment"
        {
            // name <- function(...) { ... }
            let mut ac = child.walk();
            for c in child.children(&mut ac) {
                if c.kind() == "function_definition" {
                    // Get the name from the left side of assignment (first child)
                    let name = child.child(0).and_then(|n| {
                        if n.kind() == "identifier" {
                            Some(node_text(n, source).to_string())
                        } else {
                            None
                        }
                    });

                    if let Some(name) = name {
                        let params = child_by_kind(c, "parameters")
                            .map(|p| node_text(p, source).to_string())
                            .unwrap_or_else(|| "()".to_string());

                        items.push(Extractable::Function(FunctionSignature {
                            name,
                            params,
                            return_type: None,
                            line: child.start_position().row as u32 + 1,
                            parent_type: None,
                        }));
                    }
                }
            }
        }
    }

    items
}
