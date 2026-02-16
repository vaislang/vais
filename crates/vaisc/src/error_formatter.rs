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

/// Trait for types that can be formatted as errors with context.
/// Reserved for extensible error formatting system.
#[allow(dead_code)]
pub trait FormattableError {
    /// Format the error with the given context
    fn format_with_context(&self, context: &ErrorFormatContext) -> String;

    /// Get the error code as a string
    fn error_code(&self) -> &str;

    /// Get the error title (usually localized)
    fn error_title(&self) -> String;

    /// Get the error message
    fn error_message(&self) -> String;

    /// Get optional help text
    fn error_help(&self) -> Option<String> {
        None
    }

    /// Get the span for the error
    fn error_span(&self) -> Option<Span>;
}

impl FormattableError for TypeError {
    fn format_with_context(&self, context: &ErrorFormatContext) -> String {
        let reporter = context.reporter();
        let span = self.span();
        let title = self.localized_title();
        let message = self.localized_message();
        let help = self.localized_help();

        reporter.format_error(
            self.error_code(),
            &title,
            span,
            &message,
            help.as_deref(),
            &self.secondary_spans(),
        )
    }

    fn error_code(&self) -> &str {
        <TypeError>::error_code(self)
    }

    fn error_title(&self) -> String {
        self.localized_title()
    }

    fn error_message(&self) -> String {
        self.localized_message()
    }

    fn error_help(&self) -> Option<String> {
        self.localized_help()
    }

    fn error_span(&self) -> Option<Span> {
        self.span()
    }
}

impl FormattableError for ParseError {
    fn format_with_context(&self, context: &ErrorFormatContext) -> String {
        let reporter = context.reporter();
        let span = self.span().map(|s| Span::new(s.start, s.end));
        let title = self.localized_title();
        let message = self.localized_message();

        reporter.format_error(self.error_code(), &title, span, &message, None, &[])
    }

    fn error_code(&self) -> &str {
        <ParseError>::error_code(self)
    }

    fn error_title(&self) -> String {
        self.localized_title()
    }

    fn error_message(&self) -> String {
        self.localized_message()
    }

    fn error_span(&self) -> Option<Span> {
        self.span().map(|s| Span::new(s.start, s.end))
    }
}

/// Format a type error with source context (localized)
pub fn format_type_error(error: &TypeError, source: &str, path: &Path) -> String {
    let context = ErrorFormatContext::new(source.to_string(), path.to_path_buf());
    error.format_with_context(&context)
}

/// Format a parse error with source context (localized)
pub fn format_parse_error(error: &ParseError, source: &str, path: &Path) -> String {
    let context = ErrorFormatContext::new(source.to_string(), path.to_path_buf());
    error.format_with_context(&context)
}

/// Format any error implementing FormattableError.
/// Generic entry point for extensible error formatting.
#[allow(dead_code)]
pub fn format_error<E: FormattableError>(error: &E, source: &str, path: &Path) -> String {
    let context = ErrorFormatContext::new(source.to_string(), path.to_path_buf());
    error.format_with_context(&context)
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
}
