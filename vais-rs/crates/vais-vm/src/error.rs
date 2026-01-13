//! Vais VM Runtime Errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    #[error("Index out of bounds: {index} (length: {length})")]
    IndexOutOfBounds { index: i64, length: usize },

    #[error("Invalid field access: {field}")]
    InvalidFieldAccess { field: String },

    #[error("Maximum recursion depth exceeded")]
    MaxRecursionDepth,

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("FFI error: {0}")]
    FfiError(String),
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;
