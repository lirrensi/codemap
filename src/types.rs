/// Core types shared across all modules.

/// A single extractable item from a source file.
#[derive(Debug, Clone)]
pub enum Extractable {
    Function(FunctionSignature),
    Type(NamedType),
}

/// A function or method signature.
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub params: String,
    pub return_type: Option<String>,
    pub line: u32, // 1-based
    pub parent_type: Option<String>,
}

/// A named type (struct, enum, trait, class, interface, etc.)
#[derive(Debug, Clone)]
pub struct NamedType {
    pub name: String,
    pub kind: TypeKind,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Struct,
    Enum,
    Trait,
    Class,
    Interface,
    TypeAlias,
    Module,
}

impl std::fmt::Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::Struct => write!(f, "struct"),
            TypeKind::Enum => write!(f, "enum"),
            TypeKind::Trait => write!(f, "trait"),
            TypeKind::Class => write!(f, "class"),
            TypeKind::Interface => write!(f, "interface"),
            TypeKind::TypeAlias => write!(f, "type"),
            TypeKind::Module => write!(f, "module"),
        }
    }
}
