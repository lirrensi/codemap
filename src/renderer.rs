use crate::types::{Extractable, FunctionSignature, NamedType};
use chrono::Utc;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/// Render the CODEMAP.md from a map of file paths to their extracted items.
pub fn render(
    root: &Path,
    files: &BTreeMap<PathBuf, Vec<Extractable>>,
    tree: &str,
) -> (String, String) {
    let mut l1_output = String::new(); // Level 1: names only with nesting
    let mut l2_output = String::new(); // Level 2: full signatures

    l1_output.push_str("# CODEMAP (Level 1 - Names Only)\n");
    l2_output.push_str("# CODEMAP (Level 2 - Full Signatures)\n");

    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    l1_output.push_str(&format!("_generated: {}_\n\n", timestamp));
    l2_output.push_str(&format!("_generated: {}_\n\n", timestamp));

    // File tree section — wrapped in a code block for valid markdown rendering
    l1_output.push_str("## File Tree\n\n");
    l1_output.push_str("```text\n");
    l1_output.push_str(tree);
    l1_output.push_str("```\n\n");

    l2_output.push_str("## File Tree\n\n");
    l2_output.push_str("```text\n");
    l2_output.push_str(tree);
    l2_output.push_str("```\n\n");

    for (path, items) in files {
        if items.is_empty() {
            continue;
        }

        let relative = path.strip_prefix(root).unwrap_or(path);
        let relative_str = relative.to_string_lossy().replace('\\', "/");

        l1_output.push_str(&format!("## {}\n", relative_str));
        l2_output.push_str(&format!("## {}\n", relative_str));

        // Group functions by parent_type for Level 1 rendering
        let mut functions_by_parent: HashMap<Option<String>, Vec<&FunctionSignature>> =
            HashMap::new();
        let mut types: Vec<&NamedType> = Vec::new();

        // Separate items into functions and types
        for item in items {
            match item {
                Extractable::Function(sig) => {
                    functions_by_parent
                        .entry(sig.parent_type.clone())
                        .or_default()
                        .push(sig);
                }
                Extractable::Type(t) => {
                    types.push(t);
                }
            }
        }

        // Render types (same for both levels)
        for t in types {
            l1_output.push_str(&format!("- `{}` ({})\n", t.name, t.kind));
            l2_output.push_str(&format!("- `{}` ({})\n", t.name, t.kind));
        }

        // Render functions for Level 1 (names only with nesting)
        for (parent_type, functions) in &functions_by_parent {
            if let Some(parent) = parent_type {
                l1_output.push_str(&format!("  In `{}`:\n", parent));
                for sig in functions {
                    l1_output.push_str(&format!("    - `{}` :{}\n", sig.name, sig.line));
                }
            } else {
                // Top-level functions
                for sig in functions {
                    l1_output.push_str(&format!("- `{}` :{}\n", sig.name, sig.line));
                }
            }
        }

        // Render functions for Level 2 (full signatures)
        for (parent_type, functions) in &functions_by_parent {
            if let Some(parent) = parent_type {
                l2_output.push_str(&format!("  In `{}`:\n", parent));
                for sig in functions {
                    let return_type = match &sig.return_type {
                        Some(rt) => format!(" -> {}", rt),
                        None => String::new(),
                    };
                    l2_output.push_str(&format!(
                        "    - `{}{}{}` :{}\n",
                        sig.name, sig.params, return_type, sig.line
                    ));
                }
            } else {
                // Top-level functions
                for sig in functions {
                    let return_type = match &sig.return_type {
                        Some(rt) => format!(" -> {}", rt),
                        None => String::new(),
                    };
                    l2_output.push_str(&format!(
                        "  - `{}{}{}` :{}\n",
                        sig.name, sig.params, return_type, sig.line
                    ));
                }
            }
        }

        l1_output.push('\n');
        l2_output.push('\n');
    }

    (l1_output, l2_output)
}
