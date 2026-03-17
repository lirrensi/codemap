use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "function_item" => {
                if let Some(mut sig) = extract_function(source, child) {
                    sig.parent_type = None;
                    items.push(Extractable::Function(sig));
                }
            }
            "struct_item" => {
                if let Some((t, _type_name)) = extract_named_type(source, child, TypeKind::Struct) {
                    items.push(Extractable::Type(t));
                }
            }
            "enum_item" => {
                if let Some((t, _type_name)) = extract_named_type(source, child, TypeKind::Enum) {
                    items.push(Extractable::Type(t));
                }
            }
            "trait_item" => {
                if let Some((t, type_name)) = extract_named_type(source, child, TypeKind::Trait) {
                    items.push(Extractable::Type(t));
                    // Also extract trait method signatures
                    extract_trait_methods(source, child, &mut items, &type_name);
                }
            }
            "type_item" => {
                if let Some((t, _type_name)) =
                    extract_named_type(source, child, TypeKind::TypeAlias)
                {
                    items.push(Extractable::Type(t));
                }
            }
            "impl_item" => {
                // For impl blocks, we don't have a specific type name from the impl itself
                // The type being implemented is in the impl's type specification
                // For now, we'll pass None since we don't extract the implemented type name here
                extract_impl(source, child, &mut items, None);
            }
            _ => {}
        }
    }

    items
}

fn extract_function(source: &str, node: Node) -> Option<FunctionSignature> {
    let name_node = child_by_kind(node, "identifier")?;
    let name = node_text(name_node, source).to_string();

    let params_node = child_by_kind(node, "parameters")?;
    let params = node_text(params_node, source).to_string();

    // Return type is NOT a single child node - it's a "->" token followed by a type node.
    // Walk siblings after "parameters" to find "->" then grab the next type node.
    let return_type = extract_return_type(source, node);

    Some(FunctionSignature {
        name,
        params,
        return_type,
        line: node.start_position().row as u32 + 1,
        parent_type: None,
    })
}

fn extract_return_type(source: &str, node: Node) -> Option<String> {
    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();

    // Find the "->" token
    for i in 0..children.len() {
        if children[i].kind() == "->" {
            // The return type is the next sibling (skip over "->")
            if i + 1 < children.len() {
                let type_node = children[i + 1];
                // Make sure it's actually a type node, not something else
                if is_type_node(type_node.kind()) {
                    return Some(node_text(type_node, source).to_string());
                }
            }
        }
    }
    None
}

fn is_type_node(kind: &str) -> bool {
    matches!(
        kind,
        "type_identifier"
            | "primitive_type"
            | "generic_type"
            | "reference_type"
            | "tuple_type"
            | "array_type"
            | "unit_type"
            | "function_type"
            | "macro_invocation"  // for things like Result<...>
            | "scoped_type_identifier"
            | "never_type"
    )
}

fn extract_named_type(source: &str, node: Node, kind: TypeKind) -> Option<(NamedType, String)> {
    let name_node = child_by_kind(node, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    Some((
        NamedType {
            name: name.clone(),
            kind,
        },
        name,
    ))
}

fn extract_impl(
    source: &str,
    node: Node,
    items: &mut Vec<Extractable>,
    parent_type: Option<String>,
) {
    // Extract methods from impl blocks
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "declaration_list" {
            let mut method_cursor = child.walk();
            for method_child in child.children(&mut method_cursor) {
                if method_child.kind() == "function_item"
                    && let Some(sig) = extract_function(source, method_child)
                {
                    // Keep params as-is from tree-sitter (self/&self is already there)
                    items.push(Extractable::Function(FunctionSignature {
                        parent_type: parent_type.clone(),
                        ..sig
                    }));
                }
            }
        }
    }
}

fn extract_trait_methods(
    source: &str,
    node: Node,
    items: &mut Vec<Extractable>,
    parent_type: &str,
) {
    // Extract method signatures from trait declarations
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "declaration_list" {
            let mut method_cursor = child.walk();
            for method_child in child.children(&mut method_cursor) {
                if method_child.kind() == "function_signature_item"
                    && let Some(sig) = extract_function(source, method_child)
                {
                    items.push(Extractable::Function(FunctionSignature {
                        parent_type: Some(parent_type.to_string()),
                        ..sig
                    }));
                }
            }
        }
    }
}
