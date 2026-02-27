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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_error_code() {
        let err = CodegenError::InternalError("test ICE".to_string());
        assert_eq!(err.error_code(), "C007");
    }

    #[test]
    fn test_internal_error_help() {
        let err = CodegenError::InternalError("test ICE".to_string());
        let help = err.help();
        assert!(help.is_some());
        assert!(help.unwrap().contains("compiler bug"));
    }

    #[test]
    fn test_all_error_codes_unique() {
        let errors = vec![
            CodegenError::UndefinedVar("x".into()),
            CodegenError::UndefinedFunction("f".into()),
            CodegenError::TypeError("t".into()),
            CodegenError::LlvmError("l".into()),
            CodegenError::Unsupported("u".into()),
            CodegenError::RecursionLimitExceeded("r".into()),
            CodegenError::InternalError("i".into()),
        ];
        let codes: Vec<String> = errors.iter().map(|e| e.error_code().to_string()).collect();
        let unique: std::collections::HashSet<String> = codes.iter().cloned().collect();
        assert_eq!(codes.len(), unique.len(), "All error codes must be unique");
    }

    // ========== Error Display ==========

    #[test]
    fn test_error_display() {
        assert_eq!(
            CodegenError::UndefinedVar("x".to_string()).to_string(),
            "Undefined variable: x"
        );
        assert_eq!(
            CodegenError::UndefinedFunction("foo".to_string()).to_string(),
            "Undefined function: foo"
        );
        assert_eq!(
            CodegenError::TypeError("mismatch".to_string()).to_string(),
            "Type error: mismatch"
        );
        assert_eq!(
            CodegenError::LlvmError("segfault".to_string()).to_string(),
            "LLVM error: segfault"
        );
        assert_eq!(
            CodegenError::Unsupported("async".to_string()).to_string(),
            "Unsupported feature: async"
        );
        assert_eq!(
            CodegenError::RecursionLimitExceeded("deep".to_string()).to_string(),
            "Recursion depth limit exceeded: deep"
        );
        assert_eq!(
            CodegenError::InternalError("ICE".to_string()).to_string(),
            "ICE: ICE"
        );
    }

    // ========== Error Codes ==========

    #[test]
    fn test_error_code_values() {
        assert_eq!(CodegenError::UndefinedVar("".into()).error_code(), "C001");
        assert_eq!(
            CodegenError::UndefinedFunction("".into()).error_code(),
            "C002"
        );
        assert_eq!(CodegenError::TypeError("".into()).error_code(), "C003");
        assert_eq!(CodegenError::LlvmError("".into()).error_code(), "C004");
        assert_eq!(CodegenError::Unsupported("".into()).error_code(), "C005");
        assert_eq!(
            CodegenError::RecursionLimitExceeded("".into()).error_code(),
            "C006"
        );
        assert_eq!(CodegenError::InternalError("".into()).error_code(), "C007");
    }

    // ========== Help messages ==========

    #[test]
    fn test_help_undefined_var() {
        let err = CodegenError::UndefinedVar("x".to_string());
        let help = err.help().unwrap();
        assert!(help.contains("defined before use"));
    }

    #[test]
    fn test_help_undefined_var_with_suggestion() {
        let err = CodegenError::UndefinedVar("Did you mean 'y'?".to_string());
        assert!(err.help().is_none());
    }

    #[test]
    fn test_help_undefined_function() {
        let err = CodegenError::UndefinedFunction("foo".to_string());
        let help = err.help().unwrap();
        assert!(help.contains("defined before calling"));
    }

    #[test]
    fn test_help_undefined_function_with_suggestion() {
        let err = CodegenError::UndefinedFunction("Did you mean 'bar'?".to_string());
        assert!(err.help().is_none());
    }

    #[test]
    fn test_help_type_error() {
        let err = CodegenError::TypeError("mismatch".to_string());
        let help = err.help().unwrap();
        assert!(help.contains("compatible types"));
    }

    #[test]
    fn test_help_unsupported() {
        let err = CodegenError::Unsupported("async generators".to_string());
        let help = err.help().unwrap();
        assert!(help.contains("async generators"));
    }

    #[test]
    fn test_help_recursion_limit() {
        let err = CodegenError::RecursionLimitExceeded("deep nesting".to_string());
        let help = err.help().unwrap();
        assert!(help.contains("reducing nesting"));
    }

    #[test]
    fn test_help_llvm_error() {
        let err = CodegenError::LlvmError("crash".to_string());
        assert!(err.help().is_none());
    }
}
