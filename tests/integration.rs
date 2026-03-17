use codemap::parser;
use codemap::renderer;
use codemap::types::{Extractable, TypeKind};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a fixture file and return extracted items.
fn parse_fixture(filename: &str) -> Vec<Extractable> {
    let path = Path::new("tests/fixtures").join(filename);
    let source = fs::read_to_string(&path).expect("fixture file missing");
    parser::extract_from_file(&path, &source).expect("language not supported")
}

/// Collect all function names from extracted items.
fn function_names(items: &[Extractable]) -> Vec<&str> {
    items
        .iter()
        .filter_map(|i| match i {
            Extractable::Function(sig) => Some(sig.name.as_str()),
            _ => None,
        })
        .collect()
}

/// Collect all type names from extracted items.
fn type_names(items: &[Extractable]) -> Vec<&str> {
    items
        .iter()
        .filter_map(|i| match i {
            Extractable::Type(t) => Some(t.name.as_str()),
            _ => None,
        })
        .collect()
}

/// Check that a function with the given name exists.
fn has_function(items: &[Extractable], name: &str) -> bool {
    function_names(items).contains(&name)
}

/// Check that a type with the given name exists.
fn has_type(items: &[Extractable], name: &str) -> bool {
    type_names(items).contains(&name)
}

// ---------------------------------------------------------------------------
// Unit tests: parser dispatch
// ---------------------------------------------------------------------------

#[test]
fn test_language_for_extension_known() {
    assert!(parser::language_for_extension("rs").is_some());
    assert!(parser::language_for_extension("py").is_some());
    assert!(parser::language_for_extension("js").is_some());
    assert!(parser::language_for_extension("ts").is_some());
    assert!(parser::language_for_extension("tsx").is_some());
    assert!(parser::language_for_extension("go").is_some());
    assert!(parser::language_for_extension("sh").is_some());
    assert!(parser::language_for_extension("c").is_some());
    assert!(parser::language_for_extension("cpp").is_some());
    assert!(parser::language_for_extension("java").is_some());
    assert!(parser::language_for_extension("cs").is_some());
    assert!(parser::language_for_extension("rb").is_some());
    assert!(parser::language_for_extension("php").is_some());
    assert!(parser::language_for_extension("swift").is_some());
    assert!(parser::language_for_extension("kt").is_some());
    assert!(parser::language_for_extension("scala").is_some());
    assert!(parser::language_for_extension("lua").is_some());
    assert!(parser::language_for_extension("zig").is_some());
    assert!(parser::language_for_extension("ex").is_some());
    assert!(parser::language_for_extension("hs").is_some());
    assert!(parser::language_for_extension("ml").is_some());
    assert!(parser::language_for_extension("dart").is_some());
    assert!(parser::language_for_extension("r").is_some());
    assert!(parser::language_for_extension("jl").is_some());
}

#[test]
fn test_language_for_extension_unknown() {
    assert!(parser::language_for_extension("xyz").is_none());
    assert!(parser::language_for_extension("md").is_none());
    assert!(parser::language_for_extension("").is_none());
}

#[test]
fn test_is_code_extension() {
    assert!(parser::is_code_extension("rs"));
    assert!(parser::is_code_extension("py"));
    assert!(parser::is_code_extension("go"));
    assert!(!parser::is_code_extension("md"));
    assert!(!parser::is_code_extension("txt"));
    assert!(!parser::is_code_extension("json"));
}

// ---------------------------------------------------------------------------
// Unit tests: renderer
// ---------------------------------------------------------------------------

#[test]
fn test_renderer_output_format() {
    let mut files = BTreeMap::new();
    let items = vec![
        Extractable::Type(codemap::types::NamedType {
            name: "Config".into(),
            kind: TypeKind::Struct,
        }),
        Extractable::Function(codemap::types::FunctionSignature {
            name: "add".into(),
            params: "(a: i32, b: i32)".into(),
            return_type: Some("i32".into()),
            line: 10,
            parent_type: None,
        }),
    ];
    files.insert(PathBuf::from("/root/src/main.rs"), items);

    let (l1_output, l2_output) = renderer::render(Path::new("/root"), &files, "");
    assert!(l2_output.contains("_generated:"));
    assert!(l2_output.contains("## src/main.rs"));
    assert!(l2_output.contains("`Config` (struct)"));
    assert!(l2_output.contains("`add(a: i32, b: i32) -> i32` :10"));
}

#[test]
fn test_renderer_skips_empty_files() {
    let mut files = BTreeMap::new();
    files.insert(PathBuf::from("/root/empty.rs"), vec![]);
    files.insert(
        PathBuf::from("/root/main.rs"),
        vec![Extractable::Function(codemap::types::FunctionSignature {
            name: "main".into(),
            params: "()".into(),
            return_type: None,
            line: 1,
            parent_type: None,
        })],
    );

    let (_, l2_output) = renderer::render(Path::new("/root"), &files, "");
    assert!(l2_output.contains("## main.rs"));
    assert!(!l2_output.contains("empty.rs"));
}

// ---------------------------------------------------------------------------
// Unit tests: types
// ---------------------------------------------------------------------------

#[test]
fn test_type_kind_display() {
    assert_eq!(TypeKind::Struct.to_string(), "struct");
    assert_eq!(TypeKind::Enum.to_string(), "enum");
    assert_eq!(TypeKind::Trait.to_string(), "trait");
    assert_eq!(TypeKind::Class.to_string(), "class");
    assert_eq!(TypeKind::Interface.to_string(), "interface");
    assert_eq!(TypeKind::TypeAlias.to_string(), "type");
    assert_eq!(TypeKind::Module.to_string(), "module");
}

// ---------------------------------------------------------------------------
// End-to-end tests: each language
// ---------------------------------------------------------------------------

#[test]
fn e2e_rust() {
    let items = parse_fixture("sample.rs");
    assert!(has_type(&items, "Config"), "should find Config struct");
    assert!(has_type(&items, "Status"), "should find Status enum");
    assert!(
        has_type(&items, "Renderable"),
        "should find Renderable trait"
    );
    assert!(has_type(&items, "Pair"), "should find Pair type alias");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(has_function(&items, "new"), "should find new method");
    assert!(
        items.len() >= 7,
        "expected at least 7 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_python() {
    let items = parse_fixture("sample.py");
    assert!(
        has_type(&items, "Calculator"),
        "should find Calculator class"
    );
    assert!(has_type(&items, "Vector"), "should find Vector class");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(
        has_function(&items, "_private_helper"),
        "should find private helper"
    );
    assert!(
        items.len() >= 6,
        "expected at least 6 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_javascript() {
    let items = parse_fixture("sample.js");
    assert!(
        has_type(&items, "EventEmitter"),
        "should find EventEmitter class"
    );
    assert!(has_function(&items, "add"), "should find add function");
    assert!(
        has_function(&items, "multiply"),
        "should find multiply arrow"
    );
    assert!(
        items.len() >= 4,
        "expected at least 4 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_typescript() {
    let items = parse_fixture("sample.ts");
    assert!(has_type(&items, "Point"), "should find Point interface");
    assert!(has_type(&items, "Direction"), "should find Direction enum");
    assert!(has_type(&items, "Vector2D"), "should find Vector2D class");
    assert!(
        has_function(&items, "distance"),
        "should find distance function"
    );
    assert!(
        items.len() >= 7,
        "expected at least 7 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_tsx() {
    let items = parse_fixture("sample.tsx");
    assert!(has_type(&items, "Props"), "should find Props interface");
    assert!(
        has_function(&items, "Button"),
        "should find Button function"
    );
    assert!(
        items.len() >= 3,
        "expected at least 3 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_go() {
    let items = parse_fixture("sample.go");
    assert!(has_type(&items, "Config"), "should find Config struct");
    assert!(has_type(&items, "Reader"), "should find Reader interface");
    assert!(has_function(&items, "NewConfig"), "should find NewConfig");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(
        items.len() >= 5,
        "expected at least 5 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_bash() {
    let items = parse_fixture("sample.sh");
    assert!(has_function(&items, "greet"), "should find greet");
    assert!(has_function(&items, "add"), "should find add");
    assert!(has_function(&items, "deploy"), "should find deploy");
    assert_eq!(items.len(), 3);
}

#[test]
fn e2e_c() {
    let items = parse_fixture("sample.c");
    assert!(has_type(&items, "Point"), "should find Point struct");
    assert!(has_type(&items, "Color"), "should find Color enum");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(
        has_function(&items, "print_point"),
        "should find print_point"
    );
    assert!(
        items.len() >= 5,
        "expected at least 5 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_cpp() {
    let items = parse_fixture("sample.cpp");
    assert!(has_type(&items, "Point"), "should find Point struct");
    assert!(has_type(&items, "Vector"), "should find Vector class");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(
        items.len() >= 4,
        "expected at least 4 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_java() {
    let items = parse_fixture("sample.java");
    assert!(
        has_type(&items, "Calculator"),
        "should find Calculator class"
    );
    assert!(
        has_type(&items, "Drawable"),
        "should find Drawable interface"
    );
    assert!(has_type(&items, "Status"), "should find Status enum");
    assert!(has_function(&items, "add"), "should find add method");
    assert!(has_function(&items, "multiply"), "should find multiply");
    assert!(
        items.len() >= 7,
        "expected at least 7 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_csharp() {
    let items = parse_fixture("sample.cs");
    assert!(
        has_type(&items, "Calculator"),
        "should find Calculator class"
    );
    assert!(
        has_type(&items, "IDrawable"),
        "should find IDrawable interface"
    );
    assert!(has_type(&items, "Status"), "should find Status enum");
    assert!(has_function(&items, "Add"), "should find Add method");
    assert!(
        items.len() >= 7,
        "expected at least 7 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_ruby() {
    let items = parse_fixture("sample.rb");
    assert!(
        has_type(&items, "Calculator"),
        "should find Calculator class"
    );
    assert!(
        has_type(&items, "MathHelper"),
        "should find MathHelper module"
    );
    assert!(has_function(&items, "initialize"), "should find initialize");
    assert!(has_function(&items, "add"), "should find add method");
    assert!(
        items.len() >= 6,
        "expected at least 6 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_php() {
    let items = parse_fixture("sample.php");
    assert!(
        has_type(&items, "Calculator"),
        "should find Calculator class"
    );
    assert!(
        has_type(&items, "Drawable"),
        "should find Drawable interface"
    );
    assert!(has_type(&items, "Loggable"), "should find Loggable trait");
    assert!(has_type(&items, "Status"), "should find Status enum");
    assert!(has_function(&items, "add"), "should find add method");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(
        items.len() >= 8,
        "expected at least 8 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_swift() {
    let items = parse_fixture("sample.swift");
    assert!(has_type(&items, "Point"), "should find Point struct");
    assert!(has_type(&items, "Vector"), "should find Vector class");
    assert!(
        has_type(&items, "Renderable"),
        "should find Renderable protocol"
    );
    assert!(has_function(&items, "add"), "should find add function");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(
        has_function(&items, "magnitude"),
        "should find magnitude method"
    );
    assert!(
        items.len() >= 8,
        "expected at least 8 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_kotlin() {
    let items = parse_fixture("sample.kt");
    assert!(
        has_type(&items, "Calculator"),
        "should find Calculator class"
    );
    assert!(
        has_type(&items, "Renderable"),
        "should find Renderable interface"
    );
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(
        items.len() >= 6,
        "expected at least 6 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_scala() {
    let items = parse_fixture("sample.scala");
    assert!(
        has_type(&items, "Renderable"),
        "should find Renderable trait"
    );
    assert!(
        has_type(&items, "Calculator"),
        "should find Calculator class"
    );
    assert!(has_type(&items, "Color"), "should find Color enum");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(
        items.len() >= 7,
        "expected at least 7 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_lua() {
    let items = parse_fixture("sample.lua");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(
        items.len() >= 2,
        "expected at least 2 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_zig() {
    let items = parse_fixture("sample.zig");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(has_function(&items, "magnitude"), "should find magnitude");
    assert!(
        items.len() >= 3,
        "expected at least 3 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_elixir() {
    let items = parse_fixture("sample.ex");
    assert!(
        has_type(&items, "Calculator"),
        "should find Calculator module"
    );
    assert!(has_type(&items, "Greeter"), "should find Greeter module");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(
        items.len() >= 5,
        "expected at least 5 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_haskell() {
    let items = parse_fixture("sample.hs");
    assert!(has_type(&items, "Point"), "should find Point data type");
    assert!(
        has_type(&items, "Renderable"),
        "should find Renderable typeclass"
    );
    assert!(has_function(&items, "add"), "should find add function");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(
        items.len() >= 6,
        "expected at least 6 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_ocaml() {
    let items = parse_fixture("sample.ml");
    assert!(has_type(&items, "point"), "should find point type");
    assert!(has_type(&items, "direction"), "should find direction type");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(
        items.len() >= 5,
        "expected at least 5 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_dart() {
    let items = parse_fixture("sample.dart");
    assert!(has_type(&items, "Point"), "should find Point class");
    assert!(has_type(&items, "Direction"), "should find Direction enum");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(
        items.len() >= 5,
        "expected at least 5 items, got {}",
        items.len()
    );
}

#[test]
fn e2e_r() {
    let items = parse_fixture("sample.R");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(
        has_function(&items, "multiply"),
        "should find multiply function"
    );
    assert!(
        has_function(&items, "calculate_mean"),
        "should find calculate_mean"
    );
    assert_eq!(items.len(), 4);
}

#[test]
fn e2e_julia() {
    let items = parse_fixture("sample.jl");
    assert!(has_type(&items, "Point"), "should find Point struct");
    assert!(has_type(&items, "Shape"), "should find Shape abstract type");
    assert!(has_function(&items, "greet"), "should find greet function");
    assert!(has_function(&items, "add"), "should find add function");
    assert!(
        items.len() >= 6,
        "expected at least 6 items, got {}",
        items.len()
    );
}

// ---------------------------------------------------------------------------
// End-to-end: line numbers present for functions
// ---------------------------------------------------------------------------

#[test]
fn e2e_function_line_numbers() {
    let items = parse_fixture("sample.rs");
    for item in &items {
        if let Extractable::Function(sig) = item {
            assert!(sig.line > 0, "line number should be > 0 for {}", sig.name);
        }
    }
}

// ---------------------------------------------------------------------------
// End-to-end: renderer produces valid output for all fixtures
// ---------------------------------------------------------------------------

#[test]
fn e2e_renderer_all_fixtures() {
    let fixtures_dir = Path::new("tests/fixtures");
    let mut files = BTreeMap::new();

    for entry in fs::read_dir(fixtures_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let source = fs::read_to_string(&path).unwrap();
            if let Some(items) = parser::extract_from_file(&path, &source) {
                if !items.is_empty() {
                    files.insert(path, items);
                }
            }
        }
    }

    assert!(!files.is_empty(), "should have parsed at least one fixture");

    let (_, output) = renderer::render(fixtures_dir, &files, "");
    assert!(output.contains("# CODEMAP"), "should have header");
    assert!(output.contains("_generated:"), "should have timestamp");

    // Every file in the map should appear in output
    for path in files.keys() {
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(output.contains(&*name), "output should contain {}", name);
    }
}
