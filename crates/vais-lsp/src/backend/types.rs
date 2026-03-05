//! Backend utility types and data structures

use tower_lsp::lsp_types::*;
use vais_ast::Span;

/// Call graph entry representing function call relationships
#[derive(Debug, Clone)]
pub(crate) struct CallGraphEntry {
    /// Caller function name
    pub(crate) caller: String,
    /// Caller span
    pub(crate) caller_span: Span,
    /// Callee function name
    pub(crate) callee: String,
    /// Call site span
    pub(crate) call_span: Span,
}

/// Symbol definition information
#[derive(Debug, Clone)]
pub(crate) struct SymbolDef {
    pub(crate) name: String,
    pub(crate) kind: SymbolKind,
    pub(crate) span: Span,
}

/// Symbol reference information
#[derive(Debug, Clone)]
pub(crate) struct SymbolRef {
    pub(crate) name: String,
    pub(crate) span: Span,
}

/// Cached symbol information for a document
#[derive(Debug, Clone)]
pub(crate) struct SymbolCache {
    /// Document version this cache is valid for
    pub(crate) version: i32,
    /// All symbol definitions in the document
    pub(crate) definitions: Vec<SymbolDef>,
    /// All symbol references in the document
    pub(crate) references: Vec<SymbolRef>,
    /// Call graph entries for call hierarchy
    pub(crate) call_graph: Vec<CallGraphEntry>,
}

/// Inlay hint information
#[derive(Debug, Clone)]
pub(crate) struct InlayHintInfo {
    /// Position in the source
    pub(crate) position: usize,
    /// Hint label
    pub(crate) label: String,
    /// Hint kind (Type, Parameter)
    pub(crate) kind: InlayHintKind,
}

/// Folding range information
#[derive(Debug, Clone)]
pub(crate) struct FoldingRangeInfo {
    /// Start line (0-indexed)
    pub(crate) start_line: u32,
    /// End line (0-indexed)
    pub(crate) end_line: u32,
    /// Kind of folding range
    pub(crate) kind: Option<FoldingRangeKind>,
}
