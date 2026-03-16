use super::{child_by_kind, node_text};
use crate::types::{Extractable, FunctionSignature, NamedType, TypeKind};
use tree_sitter::Node;

pub fn extract(source: &str, tree: &tree_sitter::Tree) -> Vec<Extractable> {
    let mut items = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "value_definition" => {
                if let Some(sig) = extract_value(source, child) {
                    items.push(Extractable::Function(sig));
                }
            }
            "type_definition" => {
                if let Some(t) = extract_type_def(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            "module_definition" => {
                if let Some(t) = extract_module(source, child) {
                    items.push(Extractable::Type(t));
                }
            }
            _ => {}
        }
    }

    items
}

fn extract_value(source: &str, node: Node) -> Option<FunctionSignature> {
    // value_definition > let_binding > value_name
    let binding = child_by_kind(node, "let_binding")?;
    let name_node = child_by_kind(binding, "value_name")?;
    let name = node_text(name_node, source).to_string();

    // Extract params from parameter children of let_binding
    let params: Vec<&str> = binding
        .children(&mut binding.walk())
        .filter(|c| c.kind() == "parameter")
        .map(|p| node_text(p, source))
        .collect();

    let params_str = if params.is_empty() {
        "()".to_string()
    } else {
        format!("({})", params.join(", "))
    };

    // Extract return type from the `: type` annotation after params
    let return_type = extract_return_type(source, binding);

    Some(FunctionSignature {
        name,
        params: params_str,
        return_type,
        line: node.start_position().row as u32 + 1,
    })
}

/// Extract return type from let_binding.
/// In OCaml, the return type annotation appears after the params as `: type`.
/// The AST structure is: ... parameter parameter : type_constructor_path =
fn extract_return_type(source: &str, binding: Node) -> Option<String> {
    let mut cursor = binding.walk();
    let children: Vec<Node> = binding.children(&mut cursor).collect();

    // Find the ":" token
    for i in 0..children.len() {
        if children[i].kind() == ":" {
            // The return type is the next sibling after ":"
            if i + 1 < children.len() {
                let type_node = children[i + 1];
                // Make sure it's a type node, not "=" or something else
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
        "type_constructor_path"
            | "type_constructor"
            | "type_variable"
            | "polymorphic_variant_type"
            | "function_type"
            | "tuple_type"
            | "package_type"
            | "object_type"
            | "class_type"
            | "constructor_path"
    )
}

fn extract_type_def(source: &str, node: Node) -> Option<NamedType> {
    // type_definition > type_binding > type_constructor
    let binding = child_by_kind(node, "type_binding")?;
    let name_node = child_by_kind(binding, "type_constructor")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::TypeAlias,
    })
}

fn extract_module(source: &str, node: Node) -> Option<NamedType> {
    let name_node = child_by_kind(node, "module_name")?;
    let name = node_text(name_node, source).to_string();
    Some(NamedType {
        name,
        kind: TypeKind::Module,
    })
}
