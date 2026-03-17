# Plan: Add File Tree to L1/L2 Output
_Add a compact file tree section at the top of both L1 and L2 outputs, showing project structure with config files collapsed to summaries._

---

# Checklist
- [x] Step 1: Add `--tree-depth` CLI flag to `cli.rs`
- [x] Step 2: Create `src/tree.rs` with tree building and rendering logic
- [x] Step 3: Register `tree` module in `src/lib.rs`
- [x] Step 4: Update `renderer::render()` to accept and prepend tree string
- [x] Step 5: Wire tree building into `main.rs` pipeline
- [x] Step 6: Build and test

---

## Context

CodeMapper is a Rust CLI that scans codebases and produces L1 (names only) and L2 (full signatures) markdown indexes. Currently, outputs jump straight into per-file sections with no structural overview. The goal is to add a file tree at the top of both outputs.

Key existing code:
- `src/cli.rs` — clap CLI definition, `Cli` struct with fields for path, output, stdout, exclude, languages
- `src/main.rs` — pipeline: discover files → filter by lang → parse → sort → render → write
- `src/walker.rs` — `discover_files()` returns `Vec<PathBuf>` of all discovered files
- `src/parser.rs` — `is_code_extension()` returns true for code langs (rs, py, ts, etc.), `language_for_extension()` handles all supported extensions including config (json, yaml, html, css)
- `src/renderer.rs` — `render(root, files)` returns `(String, String)` for L1 and L2
- `src/lib.rs` — module declarations

The tree must be built from ALL discovered files (before language filtering), so config files are visible in the tree even though they aren't individually parsed.

---

## Prerequisites
- Rust toolchain installed (`cargo build` works)
- Working directory: `C:\Users\rx\001_Code\101_DevArea\CodeMapper`

---

## Scope Boundaries
- DO NOT touch `src/languages/` — no language extractor changes
- DO NOT touch `src/walker.rs` — discovery logic unchanged
- DO NOT change the parsing pipeline — tree is purely a rendering addition
- File tree is separate from `--languages` filtering — tree shows ALL discovered files

---

## Steps

### Step 1: Add `--tree-depth` CLI flag to `cli.rs`

Open `src/cli.rs`. Add a new field to the `Cli` struct after the `languages` field:

```rust
/// Maximum directory depth for the file tree (0 = unlimited)
#[arg(long = "tree-depth", default_value = "5", global = true)]
pub tree_depth: usize,
```

✅ Success: `Cli` struct has `tree_depth: usize` field. `cargo build` compiles without errors.
❌ If failed: Check for typos in the attribute macro. Verify `usize` is imported (it's a primitive, no import needed).

### Step 2: Create `src/tree.rs` with tree building and rendering logic

Create new file `src/tree.rs`. Implement:

**Data structure:**
```rust
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

enum TreeNode {
    Dir(BTreeMap<String, TreeNode>),
    File,        // code file — listed individually
    ConfigGroup(BTreeMap<String, usize>),  // ext -> count, e.g. {"json": 4, "yaml": 2}
}
```

**Public function:**
```rust
pub fn build_tree(paths: &[PathBuf], root: &Path, max_depth: usize) -> String
```

Logic:
1. For each path in `paths`, strip `root` prefix, get relative path components
2. Walk components to build tree. For each file's final component:
   - Get extension via `path.extension().to_str()`
   - If extension matches `is_code_extension()` → insert as `TreeNode::File` with filename as key
   - Otherwise → increment count in parent dir's `ConfigGroup` for that extension
3. Render recursively:
   - `Dir` entries: show `dirname/` then children with `├──`/`└──` prefix, increasing indent by 4 spaces
   - `File` entries: show filename (e.g., `main.rs`)
   - `ConfigGroup` entries: show `*.json (4 files)`, `*.yaml (2 files)` — one line per extension
   - If `max_depth > 0` and current depth >= `max_depth`: show `... (N more)` instead of recursing
4. Use `crate::parser::is_code_extension` to distinguish code from config files

The rendering prefix logic:
- Track which entries are last in their parent's sorted keys (for `└──` vs `├──`)
- Use `│` prefix for non-last siblings at each indent level

✅ Success: `src/tree.rs` exists with `build_tree()` function. Module compiles.
❌ If failed: Check BTreeMap iteration order (alphabetical by key). Verify `is_code_extension` is imported from `crate::parser`.

### Step 3: Register `tree` module in `src/lib.rs`

Open `src/lib.rs`. Add line:
```rust
pub mod tree;
```

Place it alphabetically (after `setup`, before `types`):
```rust
pub mod languages;
pub mod parser;
pub mod renderer;
pub mod setup;
pub mod tree;
pub mod types;
pub mod walker;
```

✅ Success: `cargo build` compiles. Module is accessible as `codemap::tree`.
❌ If failed: Verify the module declaration is `pub mod tree;` (not `mod tree;`).

### Step 4: Update `renderer::render()` to accept and prepend tree string

Open `src/renderer.rs`.

Change the function signature from:
```rust
pub fn render(root: &Path, files: &BTreeMap<PathBuf, Vec<Extractable>>) -> (String, String)
```
to:
```rust
pub fn render(root: &Path, files: &BTreeMap<PathBuf, Vec<Extractable>>, tree: &str) -> (String, String)
```

After the timestamp lines (after line 16, which reads `l2_output.push_str(&format!("_generated: {}_\n\n", timestamp));`), insert the tree section in both outputs:

```rust
// File tree section
l1_output.push_str("## File Tree\n\n");
l1_output.push_str(tree);
l1_output.push_str("\n\n");

l2_output.push_str("## File Tree\n\n");
l2_output.push_str(tree);
l2_output.push_str("\n\n");
```

The `tree` string already ends with a newline from `build_tree()`, so we add one more `\n` for spacing before the file sections begin.

✅ Success: `cargo build` compiles. Both L1 and L2 outputs start with `## File Tree` section.
❌ If failed: Verify the `tree` parameter is `&str` not `String`. Check the newlines produce correct markdown spacing.

### Step 5: Wire tree building into `main.rs` pipeline

Open `src/main.rs`.

1. Add `tree` to the import line:
```rust
use codemap::{parser, renderer, setup, tree, types, walker};
```

2. In `run_scan()`, between step 1 (discover files) and step 2 (filter by language), add tree building. Insert after line 33 (`let files = walker::discover_files(&root, &cli.exclude);`):

```rust
// Build file tree from all discovered files (before language filtering)
let tree_output = tree::build_tree(&files, &root, cli.tree_depth);
```

3. Update the renderer call (line 72) to pass the tree:
```rust
let (l1_output, l2_output) = renderer::render(&root, &file_map, &tree_output);
```

✅ Success: `cargo build` compiles. Running `cargo run -- . --stdout` shows file tree at top of L1 output.
❌ If failed: Verify `cli.tree_depth` is `usize`. Check that `build_tree` is called before the language filter so all files are in the tree.

### Step 6: Build and test

1. Run `cargo build` — must compile with zero errors
2. Run `cargo test` — all existing tests must pass
3. Run `cargo run -- . --stdout` and verify:
   - Output starts with `# CODEMAP (Level 1 - Names Only)`
   - Second section is `## File Tree`
   - Tree shows code files individually (e.g., `main.rs`, `cli.rs`)
   - Tree shows config collapsed (e.g., `*.json (N files)` if any JSON exists)
   - Tree respects depth limit (default 5)
4. Run `cargo run -- . --tree-depth 0 --stdout` — tree shows all levels
5. Run `cargo run -- . --tree-depth 2 --stdout` — tree truncated at depth 2 with `...`

✅ Success: All tests pass. Manual inspection confirms tree appears correctly in output.
❌ If failed: If compilation error, read the error message — likely a type mismatch in the new `tree` parameter. If test failure, check if existing tests call `renderer::render()` and need the new `tree` parameter added.

---

## Verification

End-to-end verification:
1. `cargo build` — clean compile
2. `cargo test` — all pass (no regressions)
3. `cargo run -- . --stdout` — L1 output starts with file tree, shows `src/main.rs`, `src/cli.rs`, etc. individually, and config files collapsed
4. `cargo run -- . --tree-depth 0 -o /tmp/test.md` — both `/tmp/test/test.L1.md` and `/tmp/test/test.L2.md` contain the tree section
5. Compare tree content to actual directory structure — tree matches what's on disk

---

## Rollback

If the feature cannot be completed:
1. Delete `src/tree.rs`
2. Revert `src/lib.rs` (remove `pub mod tree;`)
3. Revert `src/cli.rs` (remove `tree_depth` field)
4. Revert `src/renderer.rs` (remove `tree` parameter, remove tree section in output)
5. Revert `src/main.rs` (remove tree import, remove `build_tree` call, revert `render` call)
6. Run `cargo build` to verify clean revert
