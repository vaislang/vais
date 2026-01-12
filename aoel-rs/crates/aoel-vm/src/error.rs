//! Runtime error types

use thiserror::Error;
use aoel_ir::ValueType;

/// Runtime error type
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Type error: expected {expected}, got {actual}")]
    TypeError { expected: ValueType, actual: ValueType },

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Undefined input: {0}")]
    UndefinedInput(String),

    #[error("Undefined node: {0}")]
    UndefinedNode(String),

    #[error("Unknown builtin function: {0}")]
    UnknownBuiltin(String),

    #[error("Invalid argument count: expected {expected}, got {actual}")]
    InvalidArgCount { expected: usize, actual: usize },

    #[error("Index out of bounds: {index} for length {length}")]
    IndexOutOfBounds { index: i64, length: usize },

    #[error("Invalid field access: {field} on type {value_type}")]
    InvalidFieldAccess { field: String, value_type: ValueType },

    #[error("Halt: {0}")]
    Halt(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for runtime operations
pub type RuntimeResult<T> = Result<T, RuntimeError>;
