//! FILE: src/renderer.rs
//! PURPOSE: Render the markdown code maps with line-first entries and read-the-map guidance.
//! OWNS: L1/L2 markdown output formatting for generated codemap files.
//! EXPORTS: render() - builds the rendered L1 and L2 documents.
//! DOCS: README.md, docs/product.md, docs/arch.md

use crate::types::{Extractable, FunctionSignature, NamedType};
use chrono::Utc;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/// Render the CODEMAP.md from a map of file paths to their extracted items.
pub fn render(
    root: &Path,
    files: &BTreeMap<PathBuf, Vec<Extractable>>,
    line_counts: &BTreeMap<PathBuf, usize>,
    tree: &str,
) -> (String, String) {
    let mut l1_output = String::new(); // Level 1: names only with nesting
    let mut l2_output = String::new(); // Level 2: full signatures

    l1_output.push_str("# CODEMAP (Level 1 - Names Only)\n");
    l2_output.push_str("# CODEMAP (Level 2 - Full Signatures)\n");

    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    l1_output.push_str(&format!("_generated: {}_\n\n", timestamp));
    l2_output.push_str(&format!("_generated: {}_\n\n", timestamp));

    // File tree section
    l1_output.push_str("## File Tree\n\n");
    l1_output.push_str(tree);
    l1_output.push('\n');

    l2_output.push_str("## File Tree\n\n");
    l2_output.push_str(tree);
    l2_output.push('\n');

    append_reading_guide(&mut l1_output);
    append_reading_guide(&mut l2_output);

    for (path, items) in files {
        if items.is_empty() {
            continue;
        }

        let relative = path.strip_prefix(root).unwrap_or(path);
        let relative_str = relative.to_string_lossy().replace('\\', "/");
        let line_count = line_counts.get(path).copied().unwrap_or(0);

        let line_label = line_count_label(line_count);
        l1_output.push_str(&format!("## {} ({})\n", relative_str, line_label));
        l2_output.push_str(&format!("## {} ({})\n", relative_str, line_label));

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
                    l1_output.push_str(&format!("    {} | `{}`\n", sig.line, sig.name));
                }
            } else {
                // Top-level functions
                for sig in functions {
                    l1_output.push_str(&format!("{} | `{}`\n", sig.line, sig.name));
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
                        "    {} | `{}{}{}`\n",
                        sig.line, sig.name, sig.params, return_type
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
                        "{} | `{}{}{}`\n",
                        sig.line, sig.name, sig.params, return_type
                    ));
                }
            }
        }

        l1_output.push('\n');
        l2_output.push('\n');
    }

    (l1_output, l2_output)
}

fn append_reading_guide(output: &mut String) {
    output.push_str("## How to Read This\n\n");
    output.push_str("```text\n");
    output.push_str("## path/to/file.ext (127 lines)\n");
    output.push_str("34 | function_name(function_parameters) -> return_type\n");
    output.push_str("```\n\n");
    output.push_str("- `path/to/file.ext` is the file being indexed\n");
    output.push_str("- `127 lines` is the total number of lines in that file\n");
    output.push_str("- `34` is the 1-based line number where the item starts\n");
    output.push_str("- `|` separates the line number from the extracted item\n");
    output.push_str("- The item text is the name in L1, or the full signature in L2\n\n");
}

fn line_count_label(line_count: usize) -> String {
    match line_count {
        1 => "1 line".to_string(),
        n => format!("{} lines", n),
    }
}
