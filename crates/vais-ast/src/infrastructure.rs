//! Source location infrastructure: Span, Spanned, Attribute

use crate::expressions::Expr;

/// Source location information for error reporting and diagnostics.
///
/// Spans track the start and end positions of AST nodes in the source code,
/// enabling precise error messages and IDE features like go-to-definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Byte offset of the start of this span in the source file
    pub start: usize,
    /// Byte offset of the end of this span in the source file
    pub end: usize,
}

impl Span {
    /// Creates a new span from start and end positions.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Merges two spans into a single span covering both ranges.
    ///
    /// The resulting span starts at the minimum start position and
    /// ends at the maximum end position of the two input spans.
    pub fn merge(self, other: Span) -> Span {
        Span {
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
