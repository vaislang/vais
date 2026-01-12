//! AOEL Type Check Errors

use aoel_lexer::Span;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    Mismatch {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String, span: Span },

    #[error("Undefined function: {name}")]
    UndefinedFunction { name: String, span: Span },

    #[error("Argument count mismatch: expected {expected}, found {found}")]
    ArgumentCount {
        expected: usize,
        found: usize,
        span: Span,
    },

    #[error("Cannot infer type of recursive function without annotation")]
    RecursiveInference { span: Span },

    #[error("Infinite type: {0}")]
    InfiniteType(String),

    #[error("Cannot apply {op} to {ty}")]
    InvalidOperator { op: String, ty: String, span: Span },

    #[error("Cannot index {base} with {index}")]
    InvalidIndex {
        base: String,
        index: String,
        span: Span,
    },

    #[error("Cannot access field '{field}' on type {ty}")]
    InvalidField {
        field: String,
        ty: String,
        span: Span,
    },

    #[error("Not a function: {ty}")]
    NotAFunction { ty: String, span: Span },

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type TypeResult<T> = Result<T, TypeError>;
