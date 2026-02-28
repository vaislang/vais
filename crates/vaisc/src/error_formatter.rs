//! Unified error formatting utilities for Vais compiler
//!
//! This module provides a centralized interface for formatting errors with source context.
//! It consolidates formatting logic from TypeError, ParseError, and other error types.

use std::path::{Path, PathBuf};
use vais_ast::Span;
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
}
