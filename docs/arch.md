# CodeMapper — Architecture

## Tech Stack

| Component | Choice | Purpose |
|---|---|---|
| Language | Rust (edition 2024) | Core implementation |
| CLI parsing | `clap` 4 (derive) | Command-line argument handling |
| Parsing | `tree-sitter` 0.25 | Source code parsing (per-language grammars) |
| Parallelism | `rayon` 1.10 | Parallel file parsing |
| File walking | `ignore` 0.4 | Gitignore-aware directory traversal |
| Glob matching | `glob` 0.3 | Exclude pattern matching |
| Timestamps | `chrono` 0.4 | UTC timestamp in output headers |

## Module Structure

```
src/
  main.rs          Entry point: dispatches subcommands, runs scan pipeline
  cli.rs           CLI argument definitions and subcommand enum (clap)
  lib.rs           Public module exports
  types.rs         Core data types (Extractable, FunctionSignature, NamedType)
  parser.rs        Language dispatch and tree-sitter parsing
  renderer.rs      Markdown output generation (L1 and L2)
  tree.rs          File tree construction with config collapsing
  walker.rs        File discovery with gitignore support
  setup.rs         Non-interactive setup (pre-commit config + gitignore)
  onboard.rs       Interactive onboarding wizard (pre-commit + gitignore + AGENTS.md)
  languages/       Per-language extractors
    mod.rs         Shared helpers (node_text, child_by_kind, children_by_kind)
    rust_lang.rs
    python.rs
    javascript.rs
    typescript.rs
    go_lang.rs
    bash.rs
    c_lang.rs
    cpp.rs
    java.rs
    csharp.rs
    ruby.rs
    php.rs
    swift.rs
    kotlin.rs
    scala.rs
    lua.rs
    zig.rs
    elixir.rs
    haskell.rs
    ocaml.rs
    dart.rs
    r_lang.rs
    julia.rs
```

## Pipeline

The tool runs a linear pipeline in `main.rs`:

```
1. Discover files    walker::discover_files()
2. Build tree        tree::build_tree() → markdown string
3. Filter by lang    extension check against --languages
4. Parse in parallel parser::extract_from_file() via rayon
5. Sort              BTreeMap by file path
6. Render            renderer::render() → (L1, L2)
7. Write             fs::write() or print to stdout
```

## Module Details

### `cli.rs`

Defines the `Cli` struct using clap derive. Scan options (path, output, exclude, languages, tree-depth, stdout) are global fields. The `Commands` enum defines three subcommands:

- `Scan` — runs the scan pipeline (same as default/no subcommand)
- `Setup` — non-interactive setup via `setup.rs`
- `Onboard` — interactive setup via `onboard.rs`

### `types.rs`

Core data model shared across all modules:

- **`Extractable`** — enum of `Function(FunctionSignature)` or `Type(NamedType)`
- **`FunctionSignature`** — name, params (raw string), return_type (optional), line (1-based), parent_type (optional)
- **`NamedType`** — name + `TypeKind`
- **`TypeKind`** — enum: Struct, Enum, Trait, Class, Interface, TypeAlias, Module

### `walker.rs`

Wraps the `ignore` crate's `WalkBuilder`. Configures:
- Gitignore respect (local, global, exclude)
- Hidden files included
- Thread count from `available_parallelism()`
- Post-walk filtering for `--exclude` glob patterns

Returns `Vec<PathBuf>` of discovered files.

### `parser.rs`

Two responsibilities:

**Language dispatch** — `language_for_path()` maps file extensions to tree-sitter `Language` objects. `language_for_extension()` is the core match statement.

**Extraction** — `extract_from_file()` parses source into a tree-sitter `Tree`, then dispatches to the appropriate language extractor in `languages/`. Returns `Option<Vec<Extractable>>` — `None` for unsupported languages, empty vec for files with no extractable items.

### `tree.rs`

Builds a compact file tree string from discovered paths. Two entry points:

- **`build_tree(paths: &[PathBuf], root: &Path, max_depth: usize) -> String`** — takes all discovered file paths (from walker, before language filtering), builds a directory tree, and returns a markdown-formatted tree string.

**Collapsing logic:**
- Files with code extensions (per `parser::is_code_extension()`) are listed individually
- Files with data/config extensions (json, yaml, yml, html, htm, css, scss) within the same directory are collapsed to `*.ext (N files)`
- Mixed directories show both: code files individually + config summaries
- Directories containing no parseable files at all are pruned
- `max_depth` of `0` means unlimited; otherwise directories deeper than `max_depth` are truncated with `... (N more)`

**Data structure:** Internal `TreeNode` enum with `Dir(BTreeMap<String, TreeNode>)` and `File` variants. Built by splitting each path into components and inserting into the tree, then rendered recursively with box-drawing characters (`├──`, `└──`, `│`).

### `renderer.rs`

Takes the sorted `BTreeMap<PathBuf, Vec<Extractable>>` and a pre-built tree string, and produces two markdown strings:

- **L1** — Names only. Header + file tree + per-file sections. Functions show `name :line`. Methods are indented under `In \`ParentType\`:`.
- **L2** — Full signatures. Same structure. Functions show `name(params) -> returnType :line`. Same nesting.

Both include a header with generation timestamp and the file tree section. Empty files are skipped.

### `setup.rs`

Non-interactive project setup. `run_setup() -> i32` does two things:

1. **Pre-commit config** — creates `.pre-commit-config.yaml` or appends the codemap hook entry if the file exists. Skips if codemap is already configured.
2. **Gitignore** — adds `docs/CODEMAP.*.md` to `.gitignore` (or creates one). Skips if already present.

Finds the git repo root by walking parent directories from `cwd`.

### `onboard.rs`

Interactive onboarding wizard. `run_onboard() -> i32` walks through three steps, each with a yes/no prompt:

1. **Pre-commit hook** — same config logic as `setup.rs`, plus:
   - Checks if `pre-commit` binary exists via `command_exists()` (runs `pre-commit --version`)
   - If missing: offers `pip install pre-commit`
   - After config creation: offers to run `pre-commit install` to activate immediately

2. **Gitignore** — adds `docs/CODEMAP.*.md`. Displays explanation that the codemap regenerates every commit with timestamps, so committing it bloats history with derivative artifacts.

3. **AGENTS.md** — shows a preview of the suggested text, then offers a three-way choice via `prompt_accept_custom_skip()`:
   - Accept suggested text
   - Write your own (multi-line stdin until empty line)
   - Skip

Interactive input uses raw stdin/stderr reads. No external dependencies beyond `std::process::Command` for running pip and pre-commit.

### `languages/` — Per-Language Extractors

Each file exports a single function:

```rust
pub fn extract(source: &str, tree: &Tree) -> Vec<Extractable>
```

The extractor walks the tree-sitter AST using cursor-based traversal, identifies function/method definitions and type definitions, and constructs `Extractable` items.

**Shared helpers** in `languages/mod.rs`:
- `node_text(node, source)` — extract source text for a node
- `child_by_kind(node, kind)` — find first child of a given kind
- `children_by_kind(node, kind)` — find all children of a given kind

## Adding a New Language

1. Add the tree-sitter grammar crate to `Cargo.toml`
2. Add the extension-to-language mapping in `parser.rs` (`language_for_extension`)
3. Add the extension to the `is_code_extension` match in `parser.rs`
4. Add the dispatch arm in `parser.rs` (`extract_from_file`)
5. Create `src/languages/<name>.rs` with an `extract()` function
6. Register the module in `src/languages/mod.rs`
7. Add a test fixture in `tests/fixtures/`
8. Add an e2e test in `tests/integration.rs`

## Build & Test

```bash
make build      # cargo build
make release    # cargo build --release
make test       # cargo test
make clean      # cargo clean
```

The Makefile sets `CC` to a MinGW GCC path for Windows builds (tree-sitter grammars require a C compiler).
