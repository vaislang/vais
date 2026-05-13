//! WIT interface, resource, and world definitions

use super::types::{WitFunction, WitParam, WitResult, WitTypeDefinition};

/// WIT resource method
#[derive(Debug, Clone)]
pub struct WitResourceMethod {
    pub name: String,
    pub kind: WitMethodKind,
    pub params: Vec<WitParam>,
    pub results: Option<WitResult>,
    pub docs: Option<String>,
}

/// WIT resource method kind
#[derive(Debug, Clone, PartialEq)]
pub enum WitMethodKind {
    Constructor,
    Static,
    Method,
}

/// WIT resource definition
#[derive(Debug, Clone)]
pub struct WitResource {
    pub name: String,
    pub methods: Vec<WitResourceMethod>,
    pub docs: Option<String>,
}

/// WIT interface definition
#[derive(Debug, Clone)]
pub struct WitInterface {
    pub name: String,
    pub types: Vec<WitTypeDefinition>,
    pub functions: Vec<WitFunction>,
    pub resources: Vec<WitResource>,
    pub docs: Option<String>,
}

/// WIT world import
#[derive(Debug, Clone)]
pub struct WitImport {
    pub name: String,
    pub item: WitImportItem,
}

/// WIT world import item
#[derive(Debug, Clone)]
pub enum WitImportItem {
    Interface(String),
    Function(WitFunction),
}

/// WIT world export
#[derive(Debug, Clone)]
pub struct WitExport {
    pub name: String,
    pub item: WitExportItem,
}

/// WIT world export item
#[derive(Debug, Clone)]
pub enum WitExportItem {
    Interface(String),
    Function(WitFunction),
}

/// WIT world definition (top-level component interface)
#[derive(Debug, Clone)]
pub struct WitWorld {
    pub name: String,
    pub imports: Vec<WitImport>,
    pub exports: Vec<WitExport>,
    pub docs: Option<String>,
}
