//! Unified error formatting utilities for Vais compiler
//!
//! This module provides a centralized interface for formatting errors with source context.
//! It consolidates formatting logic from TypeError, ParseError, and other error types.

use std::path::{Path, PathBuf};
use vais_ast::Span;
use vais_codegen::{CodegenError, SpannedCodegenError};
use vais_parser::ParseError;
use vais_types::{error_report::ErrorReporter, TypeError};

/// Error formatting context containing source information
pub struct ErrorFormatContext {
    pub source: String,
    pub path: PathBuf,
}

impl ErrorFormatContext {
    /// Create a new error formatting context
    pub fn new(source: String, path: PathBuf) -> Self {
        Self { source, path }
    }

    /// Get the filename for display purposes
    pub fn filename(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    /// Get the reporter for this context
    pub fn reporter(&self) -> ErrorReporter<'_> {
        ErrorReporter::new(&self.source).with_filename(self.filename())
    }
}

/// Format a type error with source context (localized)
pub fn format_type_error(error: &TypeError, source: &str, path: &Path) -> String {
    let context = ErrorFormatContext::new(source.to_string(), path.to_path_buf());
    let reporter = context.reporter();
    let span = error.span();
    let title = error.localized_title();
    let message = error.localized_message();
    let help = error.localized_help();

    reporter.format_error(
        error.error_code(),
        &title,
        span,
        &message,
        help.as_deref(),
        &error.secondary_spans(),
    )
}

/// Format a codegen error with source context.
///
/// If the error has a span, renders it with source underlines via `ErrorReporter`.
/// Otherwise, falls back to a simple `note:` style message.
pub fn format_codegen_error(error: &CodegenError, source: &str, path: &Path) -> String {
    let context = ErrorFormatContext::new(source.to_string(), path.to_path_buf());
    let reporter = context.reporter();

    reporter.format_error(
        error.error_code(),
        error.title(),
        None,
        &error.to_string(),
        error.help().as_deref(),
        &[],
    )
}

/// Format a spanned codegen error with source context.
///
/// Uses the span carried by [`SpannedCodegenError`] to render a precise
/// source-location diagnostic.
#[allow(dead_code)]
pub fn format_spanned_codegen_error(
    error: &SpannedCodegenError,
    source: &str,
    path: &Path,
) -> String {
    let context = ErrorFormatContext::new(source.to_string(), path.to_path_buf());
    let reporter = context.reporter();

    reporter.format_error(
        error.error_code(),
        error.title(),
        error.span,
        &error.to_string(),
        error.help().as_deref(),
        &[],
    )
}

/// Format a parse error with source context (localized)
pub fn format_parse_error(error: &ParseError, source: &str, path: &Path) -> String {
    let context = ErrorFormatContext::new(source.to_string(), path.to_path_buf());
    let reporter = context.reporter();
    let span = error.span().map(|s| Span::new(s.start, s.end));
    let title = error.localized_title();
    let message = error.localized_message();

    reporter.format_error(error.error_code(), &title, span, &message, None, &[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_format_context_creation() {
        let source = "let x = 5;".to_string();
        let path = PathBuf::from("test.vais");
        let context = ErrorFormatContext::new(source, path);

        assert_eq!(context.filename(), "test.vais");
        assert!(!context.source.is_empty());
    }

    #[test]
    fn test_error_format_context_filename_unknown() {
        let source = "let x = 5;".to_string();
        let path = PathBuf::from("");
        let context = ErrorFormatContext::new(source, path);

        assert_eq!(context.filename(), "unknown");
    }

    #[test]
    fn test_error_format_context_nested_path() {
        let source = "F main() {}".to_string();
        let path = PathBuf::from("/home/user/project/src/lib.vais");
        let context = ErrorFormatContext::new(source, path);

        assert_eq!(context.filename(), "lib.vais");
    }

    #[test]
    fn test_error_format_context_reporter() {
        let source = "F main() { R 1 }".to_string();
        let path = PathBuf::from("test.vais");
        let context = ErrorFormatContext::new(source, path);

        // Just verify reporter can be created without panic
        let _reporter = context.reporter();
    }

    #[test]
    fn test_error_format_context_empty_source() {
        let source = String::new();
        let path = PathBuf::from("empty.vais");
        let context = ErrorFormatContext::new(source, path);

        assert!(context.source.is_empty());
        assert_eq!(context.filename(), "empty.vais");
    }

    #[test]
    fn test_error_format_context_path_with_extension() {
        let source = "code".to_string();
        let path = PathBuf::from("my_module.vais");
        let context = ErrorFormatContext::new(source, path);
        assert_eq!(context.filename(), "my_module.vais");
    }

    #[test]
    fn test_error_format_context_path_directory_only() {
        let source = "code".to_string();
        let path = PathBuf::from("/tmp/");
        let context = ErrorFormatContext::new(source, path);
        // file_name() of "/tmp/" would be None or "tmp" depending on trailing slash
        // Just ensure it doesn't panic
        let _name = context.filename();
    }

    #[test]
    fn test_error_format_context_preserves_source() {
        let source = "F main() { R 42 }\nF helper() { R 1 }".to_string();
        let path = PathBuf::from("test.vais");
        let context = ErrorFormatContext::new(source.clone(), path);
        assert_eq!(context.source, source);
    }

    #[test]
    fn test_error_format_context_preserves_path() {
        let source = "code".to_string();
        let path = PathBuf::from("/my/project/src/main.vais");
        let context = ErrorFormatContext::new(source, path.clone());
        assert_eq!(context.path, path);
    }

    #[test]
    fn test_error_format_context_multiline_source() {
        let source = "F main() {\n    R 42\n}\n".to_string();
        let path = PathBuf::from("test.vais");
        let context = ErrorFormatContext::new(source, path);
        assert!(context.source.contains("R 42"));
    }

    #[test]
    fn test_error_format_context_unicode_filename() {
        let source = "code".to_string();
        let path = PathBuf::from("unicode_파일.vais");
        let context = ErrorFormatContext::new(source, path);
        assert_eq!(context.filename(), "unicode_파일.vais");
    }

    // ========== Codegen error formatting ==========

    #[test]
    fn test_format_codegen_error_undefined_var() {
        let err = CodegenError::UndefinedVar("x".to_string());
        let source = "F main() { R x }";
        let path = PathBuf::from("test.vais");
        let output = format_codegen_error(&err, source, &path);
        assert!(output.contains("C001"));
        assert!(output.contains("Undefined variable"));
    }

    #[test]
    fn test_format_codegen_error_undefined_function() {
        let err = CodegenError::UndefinedFunction("foo".to_string());
        let source = "F main() { foo() }";
        let path = PathBuf::from("test.vais");
        let output = format_codegen_error(&err, source, &path);
        assert!(output.contains("C002"));
        assert!(output.contains("Undefined function"));
    }

    #[test]
    fn test_format_codegen_error_type_error() {
        let err = CodegenError::TypeError("mismatch".to_string());
        let source = "F main() { R 1 + true }";
        let path = PathBuf::from("test.vais");
        let output = format_codegen_error(&err, source, &path);
        assert!(output.contains("C003"));
        assert!(output.contains("Type error"));
    }

    #[test]
    fn test_format_spanned_codegen_error_with_span() {
        let err = SpannedCodegenError::new(
            CodegenError::UndefinedVar("x".to_string()),
            Span::new(14, 15), // points to "x" in "F main() { R x }"
        );
        let source = "F main() { R x }";
        let path = PathBuf::from("test.vais");
        let output = format_spanned_codegen_error(&err, source, &path);
        assert!(output.contains("C001"));
        assert!(output.contains("Undefined variable"));
        assert!(output.contains("test.vais:1:15")); // line 1, column 15
    }

    #[test]
    fn test_format_spanned_codegen_error_without_span() {
        let err = SpannedCodegenError::without_span(CodegenError::LlvmError("oops".to_string()));
        let source = "F main() { R 0 }";
        let path = PathBuf::from("test.vais");
        let output = format_spanned_codegen_error(&err, source, &path);
        assert!(output.contains("C004"));
        assert!(output.contains("LLVM error"));
    }

    #[test]
    fn test_format_codegen_error_with_help() {
        let err = CodegenError::Unsupported("async generators".to_string());
        let source = "F main() { }";
        let path = PathBuf::from("test.vais");
        let output = format_codegen_error(&err, source, &path);
        assert!(output.contains("C005"));
        assert!(output.contains("help:"));
    }

    #[test]
    fn test_format_codegen_error_ice() {
        let err = CodegenError::InternalError("unresolved generic".to_string());
        let source = "F main() { }";
        let path = PathBuf::from("test.vais");
        let output = format_codegen_error(&err, source, &path);
        assert!(output.contains("C007"));
        assert!(output.contains("compiler bug"));
    }
}
