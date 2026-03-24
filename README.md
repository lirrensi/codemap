# CodeMapper

Generate AI-optimized indexes of your codebase. Functions, types, signatures — extracted and organized into clean markdown, ready for LLMs or quick human reference.

```
codemap ./my-project
```

Produces two files:
- **CODEMAP.L1.md** — Line-first names only (compact)
- **CODEMAP.L2.md** — Line-first full signatures (detailed)

Each file starts with a **file tree header** — a visual map of the project structure. Right after that is a short read-me block explaining the `line | item` format. Code files are listed individually, config files collapsed into summaries (e.g., `*.json (4 files)`). Read this first to know where everything lives.

Supports **25+ languages** out of the box. Parses in parallel. Respects `.gitignore`.

---

## Why this exists

When an AI agent (or a new developer) lands on a codebase, the first thing it does is read files. Lots of them. Most of those files are irrelevant — config, boilerplate, glue code. The useful signal is buried in noise: *which file has the auth logic? where's the database schema? what does this function actually return?*

CodeMapper gives you that signal up front. One small markdown file with every function name, every type, every signature, with the line number first. You read the map first, then go directly to the 3 files that matter instead of skimming 30.

This works for:
- **AI agents** — Feed `CODEMAP.L2.md` into Claude, GPT, Cursor, or Copilot instead of pasting entire files. The model gets the full structure of your codebase in a fraction of the tokens.
- **Human developers** — Onboarding, code review, navigation. Find what you need without grepping blind.
- **Documentation** — Auto-generated, always up-to-date function and type reference.

The trade-off is explicit: **a small amount of structured context** (the map) against **a large amount of unstructured noise** (reading everything). The map doesn't have docstrings or implementation details. But it's *tiny* — and that's the point.

There are two modes for this reason:
- **L1 (concise)** — line-first names only. Skim the whole codebase in seconds.
- **L2 (rich)** — line-first full parameter lists and return types. Use `grep`/`rg` to find exactly the function you need, then read that file.

Read the small file. Find the needle. Read the real file. Done.

---

## Install

### One-line install

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/lirrensi/codemap/main/scripts/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/lirrensi/codemap/main/scripts/install.ps1 | iex
```

The script detects your OS and architecture, downloads the latest release, and installs to `/usr/local/bin` (Unix) or `%LOCALAPPDATA%\codemap\bin` (Windows).

Override install directory:
```bash
# Unix
CODEMAP_INSTALL_DIR=~/bin curl -fsSL ... | sh

# Windows
$env:CODEMAP_INSTALL_DIR = "C:\Tools"
irm ... | iex
```

### Prerequisites

CodeMapper uses [tree-sitter](https://tree-sitter.github.io/tree-sitter/) for parsing. Tree-sitter grammars compile C code, so you need a **C compiler**:

| Platform | What you need |
|---|---|
| **Windows** | [MinGW-w64](https://www.mingw-w64.org/) or MSVC (via Visual Studio Build Tools) |
| **macOS** | Xcode Command Line Tools (`xcode-select --install`) |
| **Linux** | `gcc` or `clang` (usually pre-installed) |

### From source

```bash
git clone https://github.com/lirrensi/codemap.git
cd codemap
cargo build --release
```

The binary is at `target/release/codemap` (or `codemap.exe` on Windows).

### Pre-built binaries

Download the latest release for your platform from the [Releases page](https://github.com/lirrensi/codemap/releases).

### Verify it works

```bash
codemap --help
```

---

## Usage

### Basic

```bash
# Scan current directory
codemap

# Scan a specific project
codemap /path/to/project

# Print to stdout (no files written)
codemap --stdout
```

### Options

| Flag | Default | Description |
|---|---|---|
| `-o, --output <FILE>` | `./docs/CODEMAP.md` | Output path. Generates `<stem>.L1.<ext>` and `<stem>.L2.<ext>` |
| `--stdout` | `false` | Print L1 output to stdout instead of writing files |
| `--exclude <PATTERN>` | — | Glob pattern to exclude. Repeatable. |
| `--languages <LIST>` | all | Comma-separated extensions to include (e.g. `rs,py,go`) |
| `--tree-depth <N>` | `5` | Max directory depth in the file tree header. `0` = unlimited |

### Examples

```bash
# Only Rust and Python files
codemap --languages rs,py

# Exclude test directories
codemap --exclude "**/tests/**" --exclude "**/fixtures/**"

# Custom output location
codemap -o ./output/index.md
# Produces: ./output/index.L1.md and ./output/index.L2.md
```

---

## Output

### File tree header

Every generated file starts with a visual tree of the project structure:

```
├── src/
│   ├── main.rs
│   ├── parser.rs
│   ├── renderer.rs
│   └── languages/
│       ├── rust.rs
│       ├── python.rs
│       └── go.rs
├── tests/
│   └── integration.rs
├── *.toml (2 files)
└── *.md (1 file)
```

Code files are listed individually. Config and data files (JSON, YAML, TOML, etc.) are collapsed into summaries so the tree stays readable. This gives you the lay of the land before diving into the signatures below.

### How to Read This

```text
## path/to/file.ext (127 lines)
34 | function_name(function_parameters) -> return_type
```

- `path/to/file.ext` is the file being indexed
- `127 lines` is the total number of lines in that file
- `34` is the 1-based line number where the item starts
- `|` separates the line number from the extracted item
- The item text is the name in L1, or the full signature in L2

### L1 — Names Only

Compact. Line-first function names, type names, and nested methods.

```markdown
# CODEMAP (Level 1 - Names Only)
_generated: 2026-03-17T12:00:00Z_

## File Tree
├── src/
│   └── main.rs
...

## src/main.rs (42 lines)
- `Config` (struct)
1 | `main`
  In `Config`:
    5 | `new`
    12 | `validate`
```

### L2 — Full Signatures

Detailed. Full parameter lists and return types.

```markdown
# CODEMAP (Level 2 - Full Signatures)
_generated: 2026-03-17T12:00:00Z_

## File Tree
├── src/
│   └── main.rs
...

## src/main.rs (42 lines)
- `Config` (struct)
1 | `main()`
  In `Config`:
    5 | `new(name: String) -> Config`
    12 | `validate(&self) -> bool`
```

---

## Supported Languages

### Code languages (functions + types)

Bash, C, C#, C++, Dart, Elixir, Go, Haskell, Java, JavaScript, Julia, Kotlin, Lua, OCaml, PHP, Python, R, Ruby, Rust, Scala, Swift, TypeScript, Zig

### Data/config languages (limited extraction)

CSS, HTML, JSON, YAML

---

## Integration

CodeMapper can set itself up in your repo across three areas of developer experience:

1. **Pre-commit hook** — regenerates the code index automatically before every commit
2. **Gitignore** — keeps generated files out of version control (each developer gets their own)
3. **AI agent setup** — tells tools like Cursor, Copilot, and Claude where to find the index

### Quick setup (non-interactive)

```bash
codemap setup
```

Sets up **pre-commit hook** and **gitignore** in one command. No prompts — it just does it.

- Creates or updates `.pre-commit-config.yaml` with the codemap hook
- Adds `docs/CODEMAP.*.md` to `.gitignore` (or creates one)

Running it twice is safe — it skips anything already configured.

### Interactive onboarding

```bash
codemap onboard
```

A guided wizard that walks you through all **three areas**, step by step:

```
  Step 1/3: Pre-commit hook
  Set up pre-commit hook? [Y/n]:
```

- Detects if `pre-commit` is installed; offers to install it via `pip` if missing
- Offers to run `pre-commit install` to activate the hook immediately

```
  Step 2/3: .gitignore
  Add 'docs/CODEMAP.*.md' to .gitignore? [Y/n]:
```

- Explains why gitignoring is recommended (regenerated files add noise to diffs)

```
  Step 3/3: AGENTS.md
  Suggested text to append:
  ┌──────────────────────────────────────────────────────────┐
  │ ## Code Map                                             │
  │ ...                                                      │
  └──────────────────────────────────────────────────────────┘
  What would you like to do?
  [a] Accept suggested text
  [e] Edit — write your own
  [s] Skip
```

- Creates or updates `AGENTS.md` with a reference to the generated code maps
- AI tools (Cursor, Copilot, Claude) read this file to understand your project faster
- You can accept the default text, write your own, or skip

Each step asks for confirmation before making changes. Safe to run on an existing repo.

### How it works

```
1. You write code
2. git commit -m "add feature"
3. pre-commit runs codemap BEFORE the commit
4. CODEMAP files regenerate locally
5. Files are gitignored — NOT committed
6. Your commit is just your code. Clean.
```

Output files are **local-only**. Each developer gets their own fresh copy on every commit. No repo bloat, no merge conflicts.

### Per-developer setup

Each developer installs once, then it's automatic:

```bash
pip install pre-commit   # or: brew install pre-commit
cargo install codemap
pre-commit install       # hooks up to this repo
```

### Want to commit the index anyway?

Just remove the `docs/CODEMAP.*.md` line from `.gitignore`. Useful for public repos where the index is useful for visitors.

### To remove codemap from your repo

1. Remove the codemap entry from `.pre-commit-config.yaml`
2. Remove the `docs/CODEMAP.*.md` line from `.gitignore`
3. Remove the codemap section from `AGENTS.md` (if you used `onboard`)
4. Delete `docs/CODEMAP.L1.md` and `docs/CODEMAP.L2.md` if they exist

No trace left in the repo.

### Manual setup

Add to `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: codemap
        name: Update codebase index
        entry: codemap
        language: system
        pass_filenames: false
```

And to `.gitignore`:

```
docs/CODEMAP.*.md
```

---

## Use cases

- **AI agent context**: Feed `CODEMAP.L2.md` into Claude, GPT, Cursor, or Copilot. The model gets the full structure of your codebase without reading every file. Use `codemap onboard` to set up `AGENTS.md` so your tools find it automatically.
- **Onboarding**: New team members get a bird's-eye view of the codebase in one file.
- **Code review**: Quickly see what changed structurally between versions.
- **Navigation**: Find exactly which file has what you need without grepping blind.
- **Documentation**: Auto-generated, always up-to-date function/type reference.

---

## Behavior

- Respects `.gitignore`, `.git/info/exclude`, and global gitignore
- Hidden files (dotfiles) are included unless excluded by gitignore
- Parallel parsing across all CPU cores
- Deterministic output (sorted by file path, items in source order)
- Silently skips files that can't be read or parsed
- Exits with code 0 on success, 1 on write failure

---

## License

See [LICENSE](LICENSE).
