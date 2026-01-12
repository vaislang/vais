//! Codegen Errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Invalid IR: {0}")]
    InvalidIR(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type CodegenResult<T> = Result<T, CodegenError>;
