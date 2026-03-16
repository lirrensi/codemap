use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Recursively discover source files, respecting .gitignore and custom excludes.
pub fn discover_files(root: &Path, excludes: &[String]) -> Vec<PathBuf> {
    let mut builder = WalkBuilder::new(root);
    builder
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .hidden(false) // include dotfiles but respect gitignore
        .threads(num_cpus());

    for pattern in excludes {
        builder.add_custom_ignore_filename(pattern);
        // Also filter post-walk since custom ignore filenames are for files like .ignore
    }

    builder
        .build()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            // Skip directories
            if !path.is_file() {
                return None;
            }

            // Apply exclude glob patterns
            for pattern in excludes {
                if let Ok(glob) = glob::Pattern::new(pattern) {
                    if glob.matches_path(path) {
                        return None;
                    }
                }
            }

            Some(path.to_path_buf())
        })
        .collect()
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}
