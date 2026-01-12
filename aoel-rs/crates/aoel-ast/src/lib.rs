//! AOEL Abstract Syntax Tree
//!
//! Defines all AST node types for the AOEL language.

mod types;
mod expr;
mod stmt;
mod unit;
mod visitor;

pub use types::*;
pub use expr::*;
pub use stmt::*;
pub use unit::*;
pub use visitor::*;

use aoel_lexer::Span;
use serde::{Deserialize, Serialize};

/// Common trait for all AST nodes
pub trait AstNode {
    /// Get the source span of this node
    fn span(&self) -> Span;
}

/// Identifier with its source location
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl Ident {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

impl AstNode for Ident {
    fn span(&self) -> Span {
        self.span
    }
}

/// Qualified name (e.g., `examples.hello_world`)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualifiedName {
    pub parts: Vec<Ident>,
    pub span: Span,
}

impl QualifiedName {
    pub fn new(parts: Vec<Ident>, span: Span) -> Self {
        Self { parts, span }
    }

    pub fn full_name(&self) -> String {
        self.parts
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }
}

impl AstNode for QualifiedName {
    fn span(&self) -> Span {
        self.span
    }
}

/// External reference (e.g., `@db.users`)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalRef {
    pub path: String,
    pub span: Span,
}

impl ExternalRef {
    pub fn new(path: impl Into<String>, span: Span) -> Self {
        Self {
            path: path.into(),
            span,
        }
    }
}

impl AstNode for ExternalRef {
    fn span(&self) -> Span {
        self.span
    }
}
