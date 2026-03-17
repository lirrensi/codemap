# CodeMapper — Product Specification

## Overview

CodeMapper is a CLI tool that scans a codebase and generates AI-optimized markdown indexes of function signatures and type definitions. It is designed to produce compact, structured summaries of a project's code surface — suitable for feeding into LLMs or for quick human reference.

The tool parses source files using tree-sitter grammars, extracts meaningful items (functions, methods, structs, classes, enums, etc.), and writes them into markdown files organized by file path.

## CLI Usage

```
codemap [OPTIONS] [PATH]
```

### Arguments

| Argument | Default | Description |
|---|---|---|
| `PATH` | `.` | Root directory to scan |

### Options

| Flag | Default | Description |
|---|---|---|
| `-o, --output <FILE>` | `./docs/CODEMAP.md` | Output file path. Two files are generated from this: `<stem>.L1.<ext>` and `<stem>.L2.<ext>` |
| `--stdout` | `false` | Print L1 output to stdout instead of writing files |
| `--exclude <PATTERN>` | (none) | Glob pattern to exclude. Repeatable. Applied in addition to `.gitignore` rules |
| `--languages <LIST>` | (all) | Comma-separated list of file extensions to include (e.g. `rs,py,go`) |
| `--tree-depth <N>` | `5` | Max directory depth for the file tree. `0` = unlimited (show everything) |

### Examples

```bash
# Scan current directory, write to ./docs/CODEMAP.L1.md and CODEMAP.L2.md
codemap

# Scan a specific project
codemap /path/to/project

# Custom output location
codemap -o ./output/index.md
# Produces: ./output/index.L1.md and ./output/index.L2.md

# Only Rust and Python files
codemap --languages rs,py

# Exclude test directories
codemap --exclude "**/tests/**" --exclude "**/fixtures/**"

# Print to stdout
codemap --stdout
```

## Output Format

CodeMapper produces two files from a single run. Both include a **file tree** at the top, followed by file-by-file extraction results.

### File Tree

Both L1 and L2 outputs begin with a tree view of the project structure. This gives the LLM (or reader) immediate context about how the codebase is organized before diving into signatures.

**Rendering rules:**

- Code files (recognized languages) are listed individually by name
- Data/config files (JSON, YAML, HTML, CSS) within a directory are collapsed to a summary line: `*.json (4 files)`
- Directories are always shown, even if they contain no code files (so `config/`, `templates/` etc. are visible)
- Tree uses standard box-drawing characters: `├──`, `└──`, `│`
- Depth is capped at `--tree-depth` (default: 5). Set to `0` for unlimited.
- Hidden files and gitignored paths are excluded (same as the rest of the tool)
- `--exclude` patterns are respected
- Only directories that contain at least one parseable file (code or config) are included

**Example tree:**

```
## File Tree

src/
├── main.rs
├── lib.rs
├── cli.rs
├── types.rs
├── parser.rs
├── renderer.rs
├── walker.rs
└── languages/
    ├── mod.rs
    ├── rust_lang.rs
    ├── python.rs
    └── ... (20 more)
config/
└── *.json (4 files)
templates/
└── *.html (2 files)
tests/
├── integration.rs
└── fixtures/
    └── ... (12 more)
```

### L1 — Names Only

Compact view. Lists type names and function names with line numbers. Methods are nested under their parent type.

```markdown
# CODEMAP (Level 1 - Names Only)
_generated: 2026-03-17T12:00:00Z_

## File Tree

src/
├── main.rs
├── lib.rs
└── config/
    └── *.json (3 files)

## src/main.rs
- `Config` (struct)
- `main` :1
  In `Config`:
    - `new` :5
    - `validate` :12
```

### L2 — Full Signatures

Detailed view. Includes full parameter lists and return types.

```markdown
# CODEMAP (Level 2 - Full Signatures)
_generated: 2026-03-17T12:00:00Z_

## File Tree

src/
├── main.rs
├── lib.rs
└── config/
    └── *.json (3 files)

## src/main.rs
- `Config` (struct)
  - `main()` :1
  In `Config`:
    - `new(name: String) -> Config` :5
    - `validate(&self) -> bool` :12
```

### Output Rules

- Files with no extractable items are omitted from output
- File paths are relative to the scanned root
- Paths use forward slashes regardless of OS
- Items within a file are listed in source order
- Files are sorted alphabetically by path
- A UTC timestamp is included in the header

## Supported Languages

Code languages (extract functions and types):

| Language | Extensions |
|---|---|
| Bash | `.sh`, `.bash` |
| C | `.c`, `.h` |
| C# | `.cs` |
| C++ | `.cpp`, `.cc`, `.cxx`, `.hpp` |
| Dart | `.dart` |
| Elixir | `.ex`, `.exs` |
| Go | `.go` |
| Haskell | `.hs` |
| Java | `.java` |
| JavaScript | `.js`, `.mjs`, `.cjs` |
| Julia | `.jl` |
| Kotlin | `.kt`, `.kts` |
| Lua | `.lua` |
| OCaml | `.ml`, `.mli` |
| PHP | `.php` |
| Python | `.py` |
| R | `.r`, `.R` |
| Ruby | `.rb` |
| Rust | `.rs` |
| Scala | `.scala` |
| Swift | `.swift` |
| TypeScript | `.ts`, `.tsx` |
| Zig | `.zig` |

Data/config languages (parsed but may yield limited items):

| Language | Extensions |
|---|---|
| CSS | `.css`, `.scss` |
| HTML | `.html`, `.htm` |
| JSON | `.json` |
| YAML | `.yml`, `.yaml` |

## Extracted Item Types

### Functions

Extracted for all supported code languages. Includes:
- Name
- Parameter list (as written in source)
- Return type (if present in source)
- Line number (1-based)
- Parent type name (if the function is a method)

### Types

Extracted when the language supports them. Recognized kinds:

| Kind | Typical Languages |
|---|---|
| `struct` | Rust, Go, C, C++, Swift, Julia |
| `enum` | Rust, C, C++, Java, C#, TypeScript, Swift, Scala, Dart, PHP |
| `class` | Python, Java, C#, C++, Ruby, Swift, Kotlin, Dart |
| `interface` | Go, Java, C#, TypeScript, Kotlin, PHP |
| `trait` | Rust, Scala, PHP |
| `type` | Rust (type alias), OCaml, Haskell |
| `module` | Ruby, Elixir |

## Behavior Guarantees

### File Discovery

- Respects `.gitignore`, `.git/info/exclude`, and global gitignore
- Hidden files (dotfiles) are included unless excluded by gitignore
- `--exclude` patterns are applied as glob matches against full paths
- Only files with recognized extensions are parsed; others are silently skipped

### Performance

- File parsing runs in parallel using all available CPU cores
- File walking uses parallel directory traversal
- Large codebases are handled without loading all files into memory simultaneously

### Determinism

- Output is deterministic for the same input: files are sorted alphabetically, items appear in source order
- The timestamp in the header reflects generation time and will differ between runs

### Error Handling

- Files that cannot be read (permissions, encoding) are silently skipped
- Files that fail to parse are silently skipped
- The tool exits with code 0 on success, 1 on write failure
