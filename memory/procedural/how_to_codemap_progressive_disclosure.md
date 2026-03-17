---
summary: "Implementation plan for adding L1/L2 progressive disclosure to CodeMapper"
created: 2026-03-17
updated: 2026-03-17
memory_type: procedural
tags: [code, codemap, feature, progressive-disclosure]
---

# CodeMapper Progressive Disclosure (L1/L2)

## Goal
Generate two output files:
- **CODEMAP-L1.md**: Names only, WITH structural nesting (methods under parent type)
- **CODEMAP-L2.md**: Full current output (signatures, params, return types, line numbers)

Both always generated together.

## Architecture
- Add `parent_type: Option<String>` to `FunctionSignature` (default `None`)
- Each extractor sets `parent_type` when extracting methods inside class/struct/impl bodies
- Renderer does all grouping centrally — no language-specific rendering logic

## Implementation Order

### Step 1: `src/types.rs`
Add `pub parent_type: Option<String>` to `FunctionSignature` struct.

### Step 2: Update extractors (12 files)
Each extractor needs parent_type passed when pushing methods inside type bodies:

**Pattern for all:**
```rust
// Before:
items.push(Extractable::Function(sig));
// After:
items.push(Extractable::Function(FunctionSignature { parent_type: Some(parent_name.clone()), ..sig }));
```

**Files and locations:**
- `python.rs` — `extract_class_methods()` line ~71
- `rust_lang.rs` — `extract_impl()` line ~121, `extract_trait_methods()` line ~138
- `java.rs` — `extract_class_body()` line ~67
- `javascript.rs` — `extract_class_methods()` line ~70
- `typescript.rs` — `extract_class_methods()` line ~117
- `go_lang.rs` — `extract_method()` line ~19 (extract receiver type name)
- `csharp.rs` — `extract_members()` line ~62
- `cpp.rs` — No method extraction currently, skip
- `dart.rs` — `extract_class_members()` lines ~81, ~86
- `php.rs` — `extract_class_members()` line ~90
- `ruby.rs` — `extract_class_methods()` lines ~99, `extract_body_methods()` lines ~117, ~122
- `swift.rs` — `extract_class_declaration()` needs to pass type name to `extract_body_functions()`
- `kotlin.rs` — `extract_class_declaration()` needs to pass type name to `extract_class_body()`

### Step 3: `src/renderer.rs`
Rewrite to:
- Group items by `parent_type` (methods nested under their type, standalone functions separate)
- Generate L1 (names only) and L2 (full signatures) output strings
- Return both as a struct

### Step 4: `src/main.rs`
Update to write two files (`CODEMAP-L1.md`, `CODEMAP-L2.md`) instead of one.

### Step 5: `src/cli.rs`
Update `--output` default or derive L1/L2 paths from it.

### Step 6: Build and test
`cargo build` and run on CodeMapper's own source.

## Key Discoveries
- Every extractor pushes to a flat `Vec<Extractable>` — parent type name is in scope but not passed through
- The change per extractor is tiny: ~1-3 lines
- Extractors that DON'T need changes: bash, lua, r_lang, elixir, haskell, ocaml, scala, zig, julia, c_lang
