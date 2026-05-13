//! Lightweight type resolution for LSP features
//!
//! Resolves expression types from AST without full type checking.
//! Used for type-aware completion, hover, and inlay hints.
//!
//! # Submodules
//!
//! - `helpers`: AST type conversion and formatting functions
//! - `context`: TypeContext construction from AST modules
//! - `inference`: Expression type inference and dot-completion

mod context;
mod helpers;
mod inference;

#[cfg(test)]
mod tests;

use helpers::{ast_type_to_lsp, format_type, parse_type_string};
use std::collections::HashMap;
use vais_ast::{Expr, FunctionBody, IfElse, Item, Module, Spanned, Stmt, Type};

/// Resolved type information for LSP purposes
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LspType {
    /// Named type (struct, enum, trait, type alias)
    Named(String),
    /// Primitive type
    Primitive(String),
    /// Array type
    Array(Box<LspType>),
    /// Tuple type
    Tuple(Vec<LspType>),
    /// Optional type
    Optional(Box<LspType>),
    /// Result type
    Result(Box<LspType>, Box<LspType>),
    /// Function type
    Function {
        params: Vec<LspType>,
        ret: Box<LspType>,
    },
    /// Range type
    Range,
    /// Unit type
    Unit,
    /// Unknown (could not resolve)
    Unknown,
}

impl LspType {
    pub(crate) fn display_name(&self) -> String {
        match self {
            LspType::Named(name) => name.clone(),
            LspType::Primitive(name) => name.clone(),
            LspType::Array(inner) => format!("[{}]", inner.display_name()),
            LspType::Tuple(types) => {
                let inner: Vec<String> = types.iter().map(|t| t.display_name()).collect();
                format!("({})", inner.join(", "))
            }
            LspType::Optional(inner) => format!("Option<{}>", inner.display_name()),
            LspType::Result(ok, err) => {
                format!("Result<{}, {}>", ok.display_name(), err.display_name())
            }
            LspType::Function { params, ret } => {
                let params_str: Vec<String> = params.iter().map(|p| p.display_name()).collect();
                format!("fn({}) -> {}", params_str.join(", "), ret.display_name())
            }
            LspType::Range => "Range".to_string(),
            LspType::Unit => "()".to_string(),
            LspType::Unknown => "_".to_string(),
        }
    }
}

/// Struct field information
#[derive(Debug, Clone)]
pub(crate) struct FieldInfo {
    pub(crate) name: String,
    pub(crate) ty: LspType,
    pub(crate) type_display: String,
}

/// Method information
#[derive(Debug, Clone)]
pub(crate) struct MethodInfo {
    pub(crate) name: String,
    pub(crate) params: Vec<(String, String)>, // (name, type_display)
    pub(crate) ret_type: Option<String>,
    pub(crate) from_trait: Option<String>,
}

/// Lightweight type context built from AST
pub(crate) struct TypeContext {
    /// Struct name -> fields
    pub(crate) structs: HashMap<String, Vec<FieldInfo>>,
    /// Type name -> methods (from impl blocks)
    pub(crate) type_methods: HashMap<String, Vec<MethodInfo>>,
    /// Trait name -> methods
    pub(crate) trait_methods: HashMap<String, Vec<MethodInfo>>,
    /// Type name -> list of trait names it implements
    pub(crate) type_traits: HashMap<String, Vec<String>>,
    /// Enum name -> variant names
    pub(crate) enum_variants: HashMap<String, Vec<String>>,
    /// Function name -> return type
    pub(crate) function_returns: HashMap<String, LspType>,
    /// Variable name -> type (scope-local, from let bindings)
    pub(crate) variable_types: HashMap<String, LspType>,
}

/// Completion entry for type-aware suggestions
#[derive(Debug, Clone)]
pub(crate) struct CompletionEntry {
    pub(crate) label: String,
    pub(crate) kind: CompletionKind,
    pub(crate) detail: String,
    pub(crate) insert_text: String,
    pub(crate) from_trait: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum CompletionKind {
    Field,
    Method,
}
