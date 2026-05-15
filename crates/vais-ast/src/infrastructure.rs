//! Source location infrastructure: Span, Spanned, Attribute

use crate::expressions::Expr;

/// Source location information for error reporting and diagnostics.
///
/// Spans track the start and end positions of AST nodes in the source code,
/// enabling precise error messages and IDE features like go-to-definition.
///
/// ## Phase 17.H1: cross-module uniqueness via `file_id`
///
/// `file_id` is a per-source-file identifier assigned by the parser/driver.
/// Two expressions in *different* source files can have the same
/// `(start, end)` byte range without clashing once both carry distinct
/// `file_id` values.
///
/// TC's `expr_types` map is keyed by the `Span` triple, so prior to this
/// field a `body_size` expression in `constants.vais` could get its type
/// silently replaced by a span-colliding expression elsewhere (observed in
/// Phase 16 session 8 — "TC span bleed"). `file_id: 0` remains valid as a
/// "synthetic / unknown source" sentinel so callers that don't yet thread
/// file identity (macros, codegen-constructed spans) continue to compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Span {
    /// Identifier of the source file (0 = synthetic / unknown).
    pub file_id: u32,
    /// Byte offset of the start of this span in the source file
    pub start: usize,
    /// Byte offset of the end of this span in the source file
    pub end: usize,
}

impl Span {
    /// Creates a new span from start and end positions, with `file_id = 0`
    /// (synthetic / unknown source). Prefer [`Span::with_file`] whenever the
    /// source file is known — keeping `file_id = 0` reintroduces the
    /// cross-module span-collision hazard that motivated this field.
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            file_id: 0,
            start,
            end,
        }
    }

    /// Creates a new span with an explicit source `file_id`.
    pub fn with_file(file_id: u32, start: usize, end: usize) -> Self {
        Self {
            file_id,
            start,
            end,
        }
    }

    /// Merges two spans into a single span covering both ranges.
    ///
    /// The resulting span starts at the minimum start position and
    /// ends at the maximum end position of the two input spans. If the
    /// two inputs disagree on `file_id`, the left operand's id wins —
    /// cross-file merge is only legal for synthetic spans and the caller
    /// is responsible for keeping merged spans within one file.
    pub fn merge(self, other: Span) -> Span {
        Span {
            file_id: if self.file_id != 0 {
                self.file_id
            } else {
                other.file_id
            },
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// AST node wrapper that includes source location information.
///
/// This generic wrapper associates any AST node type with its source span,
/// enabling error reporting and tooling features throughout the compiler pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    /// The actual AST node
    pub node: T,
    /// Source location of this node
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Creates a new spanned node with the given value and location.
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

/// Attribute for conditional compilation and metadata annotations.
///
/// Attributes like `#[cfg(test)]`, `#\[inline\]`, `#[repr(C)]`, etc. provide metadata
/// and control compilation behavior.
///
/// Contract attributes (`requires`, `ensures`, `invariant`) can contain full expressions
/// that are stored in the `expr` field for formal verification.
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    /// Attribute name (e.g., "cfg", "inline", "repr", "requires", "ensures")
    pub name: String,
    /// Attribute arguments (e.g., cfg(test) -> ["test"], repr(C) -> ["C"])
    /// For contract attributes, this contains the original expression string
    pub args: Vec<String>,
    /// Contract expression for requires/ensures/invariant attributes
    /// Contains the parsed AST expression for formal verification
    pub expr: Option<Box<Spanned<Expr>>>,
}
