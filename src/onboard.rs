use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const CODEMAP_GITIGNORE: &str = "docs/CODEMAP.*.md";

const NEW_PRE_COMMIT_CONFIG: &str = r#"repos:
  - repo: local
    hooks:
      - id: codemap
        name: Update codebase index
        entry: codemap
        language: system
        pass_filenames: false
"#;

const HOOK_ENTRY: &str = r#"
  - repo: local
    hooks:
      - id: codemap
        name: Update codebase index
        entry: codemap
        language: system
        pass_filenames: false
"#;

const DEFAULT_AGENTS_SECTION: &str = "## Code Map\n\
`docs/` contains the codebase map\n\
- `docs/CODEMAP.L1.md` — Compact index (names and line numbers)\n\
- `docs/CODEMAP.L2.md` — Detailed index (full signatures with parameters and return types)\n\
\n\
You may read entire L1 file as a quick reference.\n\
L2 file may be big - prefer using `rg` or partial reads to refer to specific files/modules.\n";

// ─── Helpers ────────────────────────────────────────────────────────────────

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

fn prompt_yes_no(question: &str, default_yes: bool) -> bool {
    let suffix = if default_yes { " [Y/n]: " } else { " [y/N]: " };
    loop {
        eprint!("{}{}", question, suffix);
        io::stderr().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return false;
        }
        let input = input.trim().to_lowercase();

        if input.is_empty() {
            return default_yes;
        }
        if input == "y" || input == "yes" {
            return true;
        }
        if input == "n" || input == "no" {
            return false;
        }
        eprintln!("  Please enter 'y' or 'n'.");
    }
}

/// Three-way prompt: accept suggested / write your own / skip.
/// Returns: Some(text) to use, or None to skip.
fn prompt_accept_custom_skip(question: &str, suggested: &str) -> Option<String> {
    eprintln!("{}", question);
    eprintln!("  [a] Accept suggested text");
    eprintln!("  [e] Edit — write your own");
    eprintln!("  [s] Skip");
    loop {
        eprint!("  Choice [a/e/s]: ");
        io::stderr().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return None;
        }
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "a" | "accept" | "" => return Some(suggested.to_string()),
            "e" | "edit" | "write" => {
                eprintln!("  Enter your text (empty line to finish):");
                let mut lines = Vec::new();
                loop {
                    eprint!("  > ");
                    io::stderr().flush().ok();
                    let mut line = String::new();
                    if io::stdin().read_line(&mut line).is_err() {
                        break;
                    }
                    if line.trim().is_empty() {
                        break;
                    }
                    lines.push(line);
                }
                if lines.is_empty() {
                    eprintln!("  No text entered, skipping.");
                    return None;
                }
                return Some(lines.join(""));
            }
            "s" | "skip" | "no" => return None,
            _ => eprintln!("  Please enter 'a', 'e', or 's'."),
        }
    }
}

fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

// ─── Step 1: Pre-commit hook ────────────────────────────────────────────────

fn setup_pre_commit(repo_root: &Path) -> bool {
    let config_path = repo_root.join(".pre-commit-config.yaml");

    if config_path.exists() {
        let content = fs::read_to_string(&config_path).unwrap_or_default();
        if content.contains("codemap") {
            eprintln!("  ✔ Pre-commit hook already configured.");
            return false;
        }
        let updated = format!("{}\n{}", content.trim_end(), HOOK_ENTRY);
        fs::write(&config_path, &updated).expect("Failed to update .pre-commit-config.yaml");
        eprintln!("  ✔ Updated .pre-commit-config.yaml with codemap hook.");
    } else {
        fs::write(&config_path, NEW_PRE_COMMIT_CONFIG)
            .expect("Failed to create .pre-commit-config.yaml");
        eprintln!("  ✔ Created .pre-commit-config.yaml with codemap hook.");
    }
    true
}

fn install_pre_commit_tool() -> bool {
    eprintln!("  Installing pre-commit via pip...");
    let status = Command::new("pip").args(["install", "pre-commit"]).status();

    match status {
        Ok(s) if s.success() => {
            eprintln!("  ✔ pre-commit installed successfully.");
            true
        }
        _ => {
            eprintln!("  ✘ Failed to install pre-commit.");
            eprintln!("    Try manually: pip install pre-commit");
            false
        }
    }
}

fn run_pre_commit_install(repo_root: &Path) {
    let status = Command::new("pre-commit")
        .arg("install")
        .current_dir(repo_root)
        .status();

    match status {
        Ok(s) if s.success() => {
            eprintln!("  ✔ pre-commit hook installed and active.");
        }
        _ => {
            eprintln!("  ✘ Failed to run 'pre-commit install'.");
            eprintln!("    Run manually: pre-commit install");
        }
    }
}

// ─── Step 2: Gitignore ──────────────────────────────────────────────────────

fn setup_gitignore(repo_root: &Path) -> bool {
    let gitignore_path = repo_root.join(".gitignore");

    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path).unwrap_or_default();
        if content.contains(CODEMAP_GITIGNORE) {
            eprintln!("  ✔ .gitignore already contains '{}'.", CODEMAP_GITIGNORE);
            return false;
        }
        let updated = format!("{}\n{}\n", content.trim_end(), CODEMAP_GITIGNORE);
        fs::write(&gitignore_path, &updated).expect("Failed to update .gitignore");
        eprintln!("  ✔ Added '{}' to .gitignore.", CODEMAP_GITIGNORE);
    } else {
        fs::write(&gitignore_path, format!("{}\n", CODEMAP_GITIGNORE))
            .expect("Failed to create .gitignore");
        eprintln!("  ✔ Created .gitignore with '{}'.", CODEMAP_GITIGNORE);
    }
    true
}

// ─── Step 3: AGENTS.md ──────────────────────────────────────────────────────

fn setup_agents_md(repo_root: &Path, text: &str) -> bool {
    let agents_path = repo_root.join("AGENTS.md");

    if agents_path.exists() {
        let content = fs::read_to_string(&agents_path).unwrap_or_default();
        if content.contains("docs/CODEMAP") {
            eprintln!("  ✔ AGENTS.md already references the codemap.");
            return false;
        }
        let updated = format!("{}\n\n{}", content.trim_end(), text.trim_start());
        fs::write(&agents_path, &updated).expect("Failed to update AGENTS.md");
        eprintln!("  ✔ Added section to AGENTS.md.");
    } else {
        fs::write(&agents_path, text.trim_start()).expect("Failed to create AGENTS.md");
        eprintln!("  ✔ Created AGENTS.md.");
    }
    true
}

// ─── Main entry point ───────────────────────────────────────────────────────

pub fn run_onboard() -> i32 {
    let repo_root = match find_repo_root() {
        Some(root) => root,
        None => {
            eprintln!("Error: not a git repository. Run 'git init' or cd into a repo.");
            return 1;
        }
    };

    let precommit_installed = command_exists("pre-commit");

    eprintln!();
    eprintln!("  CodeMapper — Project Onboarding");
    eprintln!("  ================================");
    eprintln!("  Repo: {}", repo_root.display());
    eprintln!(
        "  pre-commit: {}",
        if precommit_installed {
            "found"
        } else {
            "not found"
        }
    );
    eprintln!();

    // ── Step 1: Pre-commit hook ─────────────────────────────────────────
    eprintln!("  Step 1/3: Pre-commit hook");
    eprintln!("  This registers codemap in .pre-commit-config.yaml so the");
    eprintln!("  code index regenerates automatically on every commit.");
    eprintln!();

    if prompt_yes_no("  Set up pre-commit hook?", true) {
        let created = setup_pre_commit(&repo_root);

        if created {
            // Offer to install pre-commit tool if missing
            if !precommit_installed {
                eprintln!();
                eprintln!("  The 'pre-commit' tool is not installed on your system.");
                eprintln!("  It's required to actually run the hooks.");
                eprintln!();
                if prompt_yes_no("  Install pre-commit via pip now?", true) {
                    if install_pre_commit_tool() {
                        eprintln!();
                        if prompt_yes_no("  Run 'pre-commit install' to activate?", true) {
                            run_pre_commit_install(&repo_root);
                        }
                    }
                } else {
                    eprintln!("  Install later: pip install pre-commit");
                    eprintln!("  Then run:      pre-commit install");
                }
            } else {
                // pre-commit is installed, offer to activate
                eprintln!();
                if prompt_yes_no("  Run 'pre-commit install' to activate the hook now?", true) {
                    run_pre_commit_install(&repo_root);
                } else {
                    eprintln!("  Run later: pre-commit install");
                }
            }
        }
    } else {
        eprintln!("  — Skipped.");
    }
    eprintln!();

    // ── Step 2: Gitignore ───────────────────────────────────────────────
    eprintln!("  Step 2/3: .gitignore");
    eprintln!("  The codemap regenerates on every commit. Each run produces");
    eprintln!("  slightly different output (timestamps, minor ordering).");
    eprintln!("  Committing these files adds noise to every diff and bloats");
    eprintln!("  git history with derivative artifacts that are cheap to");
    eprintln!("  regenerate. It is recommended to gitignore them.");
    eprintln!();

    if prompt_yes_no("  Add 'docs/CODEMAP.*.md' to .gitignore?", true) {
        setup_gitignore(&repo_root);
    } else {
        eprintln!("  — Skipped. Note: codemap files will appear in git diffs.");
    }
    eprintln!();

    // ── Step 3: AGENTS.md ───────────────────────────────────────────────
    eprintln!("  Step 3/3: AGENTS.md");
    eprintln!("  AGENTS.md tells AI agents (and developers) where to find");
    eprintln!("  the codebase map. This helps tools like Cursor, Copilot,");
    eprintln!("  and Claude understand your project faster.");
    eprintln!();
    eprintln!("  Suggested text to append:");
    eprintln!("  ┌──────────────────────────────────────────────────────────┐");
    for line in DEFAULT_AGENTS_SECTION.lines() {
        eprintln!("  │ {:<58} │", line);
    }
    eprintln!("  └──────────────────────────────────────────────────────────┘");
    eprintln!();

    let agents_text =
        prompt_accept_custom_skip("  What would you like to do?", DEFAULT_AGENTS_SECTION);

    match agents_text {
        Some(text) => {
            setup_agents_md(&repo_root, &text);
        }
        None => {
            eprintln!("  — Skipped.");
        }
    }
    eprintln!();

    // ── Summary ─────────────────────────────────────────────────────────
    eprintln!("  Onboarding complete.");
    eprintln!("  Next step: run 'codemap' to generate the initial code index.");
    eprintln!();

    0
}
