use crate::parser::is_code_extension;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A node in the file tree.
enum TreeNode {
    Dir(BTreeMap<String, TreeNode>),
    File,
    ConfigGroup(BTreeMap<String, usize>),
}

/// Build a compact file tree string from all discovered paths.
///
/// Code files are listed individually. Config files (non-code extensions)
/// are collapsed into summary lines like `*.json (4 files)`.
///
/// `max_depth` of 0 means unlimited.
pub fn build_tree(paths: &[PathBuf], root: &Path, max_depth: usize) -> String {
    let mut tree_root: BTreeMap<String, TreeNode> = BTreeMap::new();

    for path in paths {
        let relative = match path.strip_prefix(root) {
            Ok(r) => r,
            Err(_) => path.as_path(),
        };

        let components: Vec<&str> = relative
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        if components.is_empty() {
            continue;
        }

        // Skip hidden files/dirs (any component starting with '.')
        if components.iter().any(|c| c.starts_with('.')) {
            continue;
        }

        // Navigate/create directory nodes
        insert_path(&mut tree_root, &components, path);
    }

    // Render the tree
    let mut output = String::new();
    render_nodes(&tree_root, "", max_depth, 0, &mut output);
    output
}

/// Insert a file path's components into the tree.
fn insert_path(root: &mut BTreeMap<String, TreeNode>, components: &[&str], path: &Path) {
    let mut current = root;
    let dirs = &components[..components.len() - 1];

    for dir_name in dirs {
        let node = current
            .entry(dir_name.to_string())
            .or_insert_with(|| TreeNode::Dir(BTreeMap::new()));

        match node {
            TreeNode::Dir(children) => {
                current = children;
            }
            _ => return, // File blocking directory path — skip
        }
    }

    let filename = *components.last().unwrap();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext.is_empty() || is_code_extension(&ext) {
        // List code files and extensionless files individually
        current
            .entry(filename.to_string())
            .or_insert_with(|| TreeNode::File);
    } else {
        let config_key = format!("*.{ext}");
        match current.get_mut(&config_key) {
            Some(TreeNode::ConfigGroup(map)) => {
                *map.entry(ext).or_insert(0) += 1;
            }
            _ => {
                let mut map = BTreeMap::new();
                map.insert(ext, 1);
                current.insert(config_key, TreeNode::ConfigGroup(map));
            }
        }
    }
}

/// Recursively render tree nodes with box-drawing prefixes.
fn render_nodes(
    nodes: &BTreeMap<String, TreeNode>,
    prefix: &str,
    max_depth: usize,
    depth: usize,
    out: &mut String,
) {
    if max_depth > 0 && depth >= max_depth {
        if !nodes.is_empty() {
            out.push_str(prefix);
            out.push_str("... (");
            out.push_str(&nodes.len().to_string());
            out.push_str(" more)\n");
        }
        return;
    }

    let entries: Vec<_> = nodes.iter().collect();
    let len = entries.len();

    for (i, (name, node)) in entries.iter().enumerate() {
        let is_last = i == len - 1;
        let connector = if is_last { "└── " } else { "├── " };

        match node {
            TreeNode::Dir(children) => {
                out.push_str(prefix);
                out.push_str(connector);
                out.push_str(name);
                out.push_str("/\n");

                let child_prefix = if is_last {
                    format!("{prefix}    ")
                } else {
                    format!("{prefix}│   ")
                };
                render_nodes(children, &child_prefix, max_depth, depth + 1, out);
            }
            TreeNode::File => {
                out.push_str(prefix);
                out.push_str(connector);
                out.push_str(name);
                out.push('\n');
            }
            TreeNode::ConfigGroup(ext_map) => {
                out.push_str(prefix);
                out.push_str(connector);
                let summary: Vec<String> = ext_map
                    .iter()
                    .map(|(ext, count)| {
                        if *count == 1 {
                            format!("*.{ext} (1 file)")
                        } else {
                            format!("*.{ext} ({count} files)")
                        }
                    })
                    .collect();
                out.push_str(&summary.join(", "));
                out.push('\n');
            }
        }
    }
}
