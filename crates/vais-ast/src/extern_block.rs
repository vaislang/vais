//! Extern block and extern function declarations

use crate::infrastructure::{Attribute, Spanned};
use crate::ast_types::Type;
use crate::function::Param;

/// Extern block: `N "C" { declarations }`
#[derive(Debug, Clone, PartialEq)]
pub struct ExternBlock {
    /// ABI string (e.g., "C", "Rust")
    pub abi: String,
    /// Extern function declarations
    pub functions: Vec<ExternFunction>,
}

/// Extern function declaration in extern block
#[derive(Debug, Clone, PartialEq)]
pub struct ExternFunction {
    pub name: Spanned<String>,
    pub params: Vec<Param>,
    pub ret_type: Option<Spanned<Type>>,
    pub is_vararg: bool,
    /// Attributes like `#[wasm_import("env", "js_alert")]`
    pub attributes: Vec<Attribute>,
}
