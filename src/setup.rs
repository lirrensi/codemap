use std::fs;
use std::path::{Path, PathBuf};

const PRE_COMMIT_REPO: &str = "https://github.com/yourorg/codemap";

const CODEMAP_GITIGNORE: &str = "docs/CODEMAP.*.md";

const NEW_CONFIG: &str = r#"repos:
  - repo: https://github.com/yourorg/codemap
    rev: v0.1.0
    hooks:
      - id: codemap
        name: Update codebase index
        pass_filenames: false
"#;

const HOOK_ENTRY: &str = r#"
  - repo: https://github.com/yourorg/codemap
    rev: v0.1.0
    hooks:
      - id: codemap
        name: Update codebase index
        pass_filenames: false
"#;

fn find_repo_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join(".git").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn setup_gitignore(repo_root: &Path) -> bool {
    let gitignore_path = repo_root.join(".gitignore");

    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path).unwrap_or_default();
        if content.contains("CODEMAP") {
            return false; // already there
        }
        // Append to existing .gitignore
        let updated = format!("{}\n{}\n", content.trim_end(), CODEMAP_GITIGNORE);
        fs::write(&gitignore_path, updated).expect("Failed to update .gitignore");
        true
    } else {
        // Create new .gitignore
        fs::write(&gitignore_path, format!("{}\n", CODEMAP_GITIGNORE))
            .expect("Failed to create .gitignore");
        true
    }
}

pub fn run_setup() -> i32 {
    let repo_root = match find_repo_root() {
        Some(root) => root,
        None => {
            eprintln!("Error: not a git repository. Run 'git init' or cd into a repo.");
            return 1;
        }
    };

    let config_path = repo_root.join(".pre-commit-config.yaml");

    // 1. Pre-commit hook
    if config_path.exists() {
        let content = fs::read_to_string(&config_path).unwrap_or_default();
        if content.contains("codemap") || content.contains(PRE_COMMIT_REPO) {
            eprintln!("  Pre-commit: already configured");
        } else {
            let updated = format!("{}\n{}", content.trim_end(), HOOK_ENTRY);
            fs::write(&config_path, updated).expect("Failed to update .pre-commit-config.yaml");
            eprintln!("  Updated: .pre-commit-config.yaml");
        }
    } else {
        fs::write(&config_path, NEW_CONFIG).expect("Failed to create .pre-commit-config.yaml");
        eprintln!("  Created: .pre-commit-config.yaml");
    }

    // 2. Gitignore
    if setup_gitignore(&repo_root) {
        eprintln!("  Added to .gitignore: {}", CODEMAP_GITIGNORE);
    } else {
        eprintln!("  .gitignore: already has CODEMAP entry");
    }

    eprintln!();
    eprintln!("  Next step:");
    eprintln!("    pre-commit install");
    eprintln!();
    eprintln!("  Output files are local-only. Each developer gets their own.");
    eprintln!("  To customize: edit .pre-commit-config.yaml");
    eprintln!("  To commit the index: remove the CODEMAP line from .gitignore");

    0
}
