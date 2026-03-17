use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use std::collections::HashMap;
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();

    // First pass: collect all type signatures (for return type lookup)
    // In Haskell, signatures are siblings of function implementations:
    //   add :: Int -> Int -> Int   <- signature node
    //   add x y = x + y           <- function node
    let signatures = collect_signatures(source, root);

    // Root is "haskell" with children "header" and "declarations"
    // Actual declarations live inside the "declarations" node
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "declarations" {
            extract_declarations(source, child, &mut items, &signatures);
        } else {
            extract_declaration_node(source, child, &mut items, &signatures);
        }
    }

    items
}

/// Collect all signature nodes and map function name -> return type
fn collect_signatures(source: &str, root: Node) -> HashMap<String, String> {
    let mut sigs = HashMap::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        if child.kind() == "declarations" {
            let mut dc = child.walk();
            for decl in child.children(&mut dc) {
                if decl.kind() == "signature"
                    && let Some((name, ret_type)) = extract_signature_info(source, decl)
                {
                    sigs.insert(name, ret_type);
                }
            }
        } else if child.kind() == "signature"
            && let Some((name, ret_type)) = extract_signature_info(source, child)
        {
            sigs.insert(name, ret_type);
        }
    }
    sigs
}

/// Extract name and return type from a signature node.
/// A signature looks like: `add :: Int -> Int -> Int`
/// The return type is the LAST type in the `->` chain.
fn extract_signature_info(source: &str, node: Node) -> Option<(String, String)> {
    let name_node = child_by_kind(node, "variable")?;
    let name = node_text(name_node, source).to_string();

    // The type is in a "function" child node
    let type_node = child_by_kind(node, "function")?;
    let full_type = node_text(type_node, source).to_string();

    // The return type is the last part after the last "->"
    let return_type = full_type
        .rsplit("->")
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or(full_type);

    Some((name, return_type))
}

fn extract_declarations(
    source: &str,
    node: tree_sitter::Node,
    items: &mut Vec<Extractable>,
    signatures: &HashMap<String, String>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_declaration_node(source, child, items, signatures);
    }
}

fn extract_declaration_node(
    source: &str,
    node: tree_sitter::Node,
    items: &mut Vec<Extractable>,
    signatures: &HashMap<String, String>,
) {
    match node.kind() {
        "function" => {
            if let Some(sig) = extract_function(source, node, signatures) {
                items.push(Extractable::Function(sig));
            }
        }
        "data_type" => {
            if let Some(t) = extract_data_type(source, node) {
                items.push(Extractable::Type(t));
            }
        }
        "newtype" => {
            if let Some(t) = extract_newtype(source, node) {
                items.push(Extractable::Type(t));
            }
        }
        "type_synomym" => {
            if let Some(t) = extract_type_alias(source, node) {
                items.push(Extractable::Type(t));
            }
        }
        "class" => {
            if let Some(t) = extract_typeclass(source, node) {
                items.push(Extractable::Type(t));
            }
        }
        _ => {}
    }
}

fn extract_function(
    source: &str,
    node: Node,
    signatures: &HashMap<String, String>,
) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "variable")?;
    let name = node_text(name_node, source).to_string();

    // Extract params from patterns node
    let params = child_by_kind(node, "patterns")
        .map(|p| {
            let text = node_text(p, source);
            format!("({})", text)
        })
        .unwrap_or_else(|| "()".to_string());

    // Look up return type from signatures collected earlier
    let return_type = signatures.get(&name).cloned();

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
        parent_type: None,
    })
}

fn extract_data_type(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "name")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Enum,
    })
}

fn extract_newtype(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "name")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}

fn extract_type_alias(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "name")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}

fn extract_typeclass(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "name")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Trait,
    })
}
