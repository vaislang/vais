//! Code generation error types
//!
//! Defines error types for code generation failures.

use thiserror::Error;

/// Error type for code generation failures.
///
/// Represents various kinds of errors that can occur during LLVM IR generation,
/// including undefined references, type mismatches, and unsupported features.
#[derive(Debug, Error)]
pub enum CodegenError {
    /// Reference to an undefined variable
    #[error("Undefined variable: {0}")]
    UndefinedVar(String),

    /// Call to an undefined function
    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    /// Type-related error during code generation
    #[error("Type error: {0}")]
    TypeError(String),

    /// LLVM-specific error
    #[error("LLVM error: {0}")]
    LlvmError(String),

    /// Feature not yet implemented in code generation
    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    /// Recursion depth limit exceeded (infinite type recursion)
    #[error("Recursion depth limit exceeded: {0}")]
    RecursionLimitExceeded(String),

    /// Internal compiler error: a type that should have been resolved before codegen
    /// (e.g., generic, associated type, ImplTrait) was not resolved.
    #[error("ICE: {0}")]
    InternalError(String),
}

impl CodegenError {
    /// Get the error code for this codegen error
    pub fn error_code(&self) -> &str {
        match self {
            CodegenError::UndefinedVar(_) => "C001",
            CodegenError::UndefinedFunction(_) => "C002",
            CodegenError::TypeError(_) => "C003",
            CodegenError::LlvmError(_) => "C004",
            CodegenError::Unsupported(_) => "C005",
            CodegenError::RecursionLimitExceeded(_) => "C006",
            CodegenError::InternalError(_) => "C007",
        }
    }

    /// Get a help message for this error
    pub fn help(&self) -> Option<String> {
        match self {
            CodegenError::UndefinedVar(msg) => {
                if msg.contains("Did you mean") {
                    None // suggestion already embedded
                } else {
                    Some("check that the variable is defined before use".to_string())
                }
            }
            CodegenError::UndefinedFunction(msg) => {
                if msg.contains("Did you mean") {
                    None
                } else {
                    Some("check that the function is defined before calling it".to_string())
                }
            }
            CodegenError::TypeError(_) => {
                Some("ensure all operands have compatible types".to_string())
            }
            CodegenError::Unsupported(feature) => Some(format!(
                "'{}' is not yet implemented in code generation",
                feature
            )),
            CodegenError::RecursionLimitExceeded(_) => {
                Some("consider reducing nesting depth or refactoring recursive types".to_string())
            }
            CodegenError::LlvmError(_) => None,
            CodegenError::InternalError(_) => {
                Some("this is likely a compiler bug — please report it".to_string())
            }
        }
    }
}

/// Result type for code generation operations
pub type CodegenResult<T> = Result<T, CodegenError>;
