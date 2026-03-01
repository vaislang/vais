//! Code generation error types
//!
//! Defines error types for code generation failures.
//!
//! # Span support
//!
//! `CodegenError` can be paired with source location information via
//! [`SpannedCodegenError`].  The internal codegen pipeline continues to use
//! `CodegenResult<T> = Result<T, CodegenError>` without any changes.
//! Span attachment happens at the boundary (e.g., in the expression
//! dispatcher) using [`CodegenError::with_span`] or the [`WithSpan`]
//! extension trait.

use thiserror::Error;
use vais_ast::Span;

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

    /// Attach a source span to this error, producing a [`SpannedCodegenError`].
    pub fn with_span(self, span: Span) -> SpannedCodegenError {
        SpannedCodegenError {
            error: self,
            span: Some(span),
        }
    }

    /// Get the error message (the inner string payload)
    pub fn message(&self) -> &str {
        match self {
            CodegenError::UndefinedVar(s)
            | CodegenError::UndefinedFunction(s)
            | CodegenError::TypeError(s)
            | CodegenError::LlvmError(s)
            | CodegenError::Unsupported(s)
            | CodegenError::RecursionLimitExceeded(s)
            | CodegenError::InternalError(s) => s,
        }
    }

    /// Get the error title (without the inner payload)
    pub fn title(&self) -> &'static str {
        match self {
            CodegenError::UndefinedVar(_) => "Undefined variable",
            CodegenError::UndefinedFunction(_) => "Undefined function",
            CodegenError::TypeError(_) => "Type error",
            CodegenError::LlvmError(_) => "LLVM error",
            CodegenError::Unsupported(_) => "Unsupported feature",
            CodegenError::RecursionLimitExceeded(_) => "Recursion depth limit exceeded",
            CodegenError::InternalError(_) => "Internal compiler error",
        }
    }
}

/// Result type for code generation operations
pub type CodegenResult<T> = Result<T, CodegenError>;

/// A [`CodegenError`] paired with an optional source [`Span`].
///
/// Used at the boundary between codegen and the compiler driver to
/// provide source-location diagnostics.  Internal codegen functions
/// continue to return plain `CodegenResult<T>`.
#[derive(Debug, Error)]
#[error("{error}")]
pub struct SpannedCodegenError {
    /// The underlying codegen error
    pub error: CodegenError,
    /// Optional source location
    pub span: Option<Span>,
}

impl SpannedCodegenError {
    /// Create a spanned error with a known span.
    pub fn new(error: CodegenError, span: Span) -> Self {
        Self {
            error,
            span: Some(span),
        }
    }

    /// Create a spanned error without span (for backward compatibility).
    pub fn without_span(error: CodegenError) -> Self {
        Self { error, span: None }
    }

    /// Delegate to the underlying error code.
    pub fn error_code(&self) -> &str {
        self.error.error_code()
    }

    /// Delegate to the underlying help message.
    pub fn help(&self) -> Option<String> {
        self.error.help()
    }

    /// Delegate to the underlying title.
    pub fn title(&self) -> &'static str {
        self.error.title()
    }

    /// Delegate to the underlying message.
    pub fn message(&self) -> &str {
        self.error.message()
    }
}

impl From<CodegenError> for SpannedCodegenError {
    fn from(error: CodegenError) -> Self {
        SpannedCodegenError { error, span: None }
    }
}

/// Extension trait for attaching span information to a `CodegenResult`.
///
/// # Example
/// ```ignore
/// use vais_codegen::{CodegenError, CodegenResult, WithSpan};
/// use vais_ast::Span;
///
/// fn example(span: Span) -> CodegenResult<()> {
///     Err(CodegenError::UndefinedVar("x".into())).with_span(span)
/// }
/// ```
pub trait WithSpan<T> {
    /// Attach a span to the error inside this `Result`, converting it to
    /// `Result<T, SpannedCodegenError>`.
    fn with_span(self, span: Span) -> Result<T, SpannedCodegenError>;
}

impl<T> WithSpan<T> for CodegenResult<T> {
    fn with_span(self, span: Span) -> Result<T, SpannedCodegenError> {
        self.map_err(|e| e.with_span(span))
    }
}

impl<T> WithSpan<T> for Result<T, SpannedCodegenError> {
    fn with_span(self, span: Span) -> Result<T, SpannedCodegenError> {
        self.map_err(|mut e| {
            if e.span.is_none() {
                e.span = Some(span);
            }
            e
        })
    }
}

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

    // ========== Span support ==========

    #[test]
    fn test_with_span() {
        let err = CodegenError::UndefinedVar("x".to_string());
        let spanned = err.with_span(Span::new(10, 11));
        assert_eq!(spanned.span, Some(Span::new(10, 11)));
        assert_eq!(spanned.error_code(), "C001");
        assert_eq!(spanned.title(), "Undefined variable");
        assert_eq!(spanned.message(), "x");
    }

    #[test]
    fn test_spanned_error_display() {
        let err = CodegenError::TypeError("mismatch".to_string());
        let spanned = SpannedCodegenError::new(err, Span::new(5, 20));
        assert_eq!(spanned.to_string(), "Type error: mismatch");
    }

    #[test]
    fn test_spanned_error_without_span() {
        let err = CodegenError::LlvmError("oops".to_string());
        let spanned = SpannedCodegenError::without_span(err);
        assert!(spanned.span.is_none());
        assert_eq!(spanned.error_code(), "C004");
    }

    #[test]
    fn test_from_codegen_error() {
        let err = CodegenError::Unsupported("feature".to_string());
        let spanned: SpannedCodegenError = err.into();
        assert!(spanned.span.is_none());
        assert_eq!(spanned.error_code(), "C005");
    }

    #[test]
    fn test_with_span_on_result() {
        let result: CodegenResult<()> = Err(CodegenError::UndefinedFunction("foo".to_string()));
        let spanned_result = result.with_span(Span::new(0, 3));
        let err = spanned_result.unwrap_err();
        assert_eq!(err.span, Some(Span::new(0, 3)));
    }

    #[test]
    fn test_with_span_preserves_existing_span() {
        let err = SpannedCodegenError::new(
            CodegenError::TypeError("t".to_string()),
            Span::new(10, 20),
        );
        let result: Result<(), SpannedCodegenError> = Err(err);
        let spanned_result = result.with_span(Span::new(0, 5));
        let err = spanned_result.unwrap_err();
        // Should preserve the original span (10, 20), not overwrite with (0, 5)
        assert_eq!(err.span, Some(Span::new(10, 20)));
    }

    #[test]
    fn test_with_span_on_ok_is_noop() {
        let result: CodegenResult<i32> = Ok(42);
        let spanned = result.with_span(Span::new(0, 1));
        assert_eq!(spanned.unwrap(), 42);
    }

    #[test]
    fn test_message_method() {
        assert_eq!(CodegenError::UndefinedVar("x".into()).message(), "x");
        assert_eq!(CodegenError::UndefinedFunction("f".into()).message(), "f");
        assert_eq!(CodegenError::TypeError("t".into()).message(), "t");
        assert_eq!(CodegenError::LlvmError("l".into()).message(), "l");
        assert_eq!(CodegenError::Unsupported("u".into()).message(), "u");
        assert_eq!(
            CodegenError::RecursionLimitExceeded("r".into()).message(),
            "r"
        );
        assert_eq!(CodegenError::InternalError("i".into()).message(), "i");
    }

    #[test]
    fn test_title_method() {
        assert_eq!(
            CodegenError::UndefinedVar("".into()).title(),
            "Undefined variable"
        );
        assert_eq!(
            CodegenError::UndefinedFunction("".into()).title(),
            "Undefined function"
        );
        assert_eq!(CodegenError::TypeError("".into()).title(), "Type error");
        assert_eq!(CodegenError::LlvmError("".into()).title(), "LLVM error");
        assert_eq!(
            CodegenError::Unsupported("".into()).title(),
            "Unsupported feature"
        );
        assert_eq!(
            CodegenError::RecursionLimitExceeded("".into()).title(),
            "Recursion depth limit exceeded"
        );
        assert_eq!(
            CodegenError::InternalError("".into()).title(),
            "Internal compiler error"
        );
    }
}
