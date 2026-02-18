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
}
