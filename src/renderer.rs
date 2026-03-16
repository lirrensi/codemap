use crate::types::Extractable;
use chrono::Utc;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Render the CODEMAP.md from a map of file paths to their extracted items.
pub fn render(root: &Path, files: &BTreeMap<PathBuf, Vec<Extractable>>) -> String {
    let mut output = String::new();

    output.push_str("# CODEMAP\n");
    output.push_str(&format!(
        "_generated: {}_\n\n",
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
    ));

    for (path, items) in files {
        if items.is_empty() {
            continue;
        }

        let relative = path.strip_prefix(root).unwrap_or(path);
        let relative_str = relative.to_string_lossy().replace('\\', "/");

        output.push_str(&format!("## {}\n", relative_str));

        for item in items {
            match item {
                Extractable::Function(sig) => {
                    let return_type = match &sig.return_type {
                        Some(rt) => format!(" -> {}", rt),
                        None => String::new(),
                    };
                    output.push_str(&format!(
                        "- `{}`{} :{}\n",
                        format!("{}{}{}", sig.name, sig.params, return_type),
                        "", // already included above
                        sig.line
                    ));
                }
                Extractable::Type(t) => {
                    output.push_str(&format!("- `{}` ({})\n", t.name, t.kind));
                }
            }
        }

        output.push('\n');
    }

    output
}
