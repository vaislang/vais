//! Error reporting utilities for the VAIS compiler
//!
//! Provides user-friendly error messages with source context, line numbers,
//! and visual indicators pointing to the error location.

use colored::Colorize;
use std::fmt;
use vais_ast::Span;

/// Error reporter that formats errors with source context
pub struct ErrorReporter<'a> {
    source: &'a str,
    filename: Option<&'a str>,
}

impl<'a> ErrorReporter<'a> {
    /// Create a new error reporter
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            filename: None,
        }
    }

    /// Set the filename for error messages
    pub fn with_filename(mut self, filename: &'a str) -> Self {
        self.filename = Some(filename);
        self
    }

    /// Format an error with source context
    pub fn format_error(
        &self,
        error_code: &str,
        title: &str,
        span: Option<Span>,
        message: &str,
        help: Option<&str>,
        secondary_spans: &[(Span, String)],
    ) -> String {
        let mut output = String::new();

        // Error header: error[E001]: Title
        output.push_str(&format!(
            "{}{}{}{} {}\n",
            "error".red().bold(),
            "[".bold(),
            error_code.yellow().bold(),
            "]".bold(),
            title.bold()
        ));

        // If we have a span, show source context
        if let Some(span) = span {
            if let Some(context) = self.get_source_context(span) {
                // Location: --> filename:line:column
                let location = if let Some(filename) = self.filename {
                    format!("{}:{}:{}", filename, context.line, context.column)
                } else {
                    format!("line {}:{}", context.line, context.column)
                };
                output.push_str(&format!("  {} {}\n", "-->".cyan().bold(), location));
                output.push_str(&format!("   {}\n", "|".cyan().bold()));

                // Show the line with line number
                output.push_str(&format!(
                    " {} {} {}\n",
                    format!("{:>3}", context.line).cyan().bold(),
                    "|".cyan().bold(),
                    context.line_text
                ));

                // Show underline and message
                output.push_str(&format!(
                    "   {} {}{} {}\n",
                    "|".cyan().bold(),
                    " ".repeat(context.column - 1),
                    "^".repeat(context.span_length.max(1)).red().bold(),
                    message.red()
                ));
            } else {
                // Fallback if we can't extract context
                output.push_str(&format!("  {} {}\n", "note:".cyan().bold(), message));
            }
        } else {
            // No span information
            output.push_str(&format!("  {} {}\n", "note:".cyan().bold(), message));
        }

        // Add help message if provided
        if let Some(help_text) = help {
            output.push_str(&format!(
                "   {} {}\n",
                "=".cyan().bold(),
                format!("help: {}", help_text).cyan()
            ));
        }

        // Add secondary spans
        for (sec_span, label) in secondary_spans {
            if let Some(context) = self.get_source_context(*sec_span) {
                output.push_str(&format!("   {} {}\n", "=".cyan().bold(), "note:".cyan()));
                let location = if let Some(filename) = self.filename {
                    format!("{}:{}:{}", filename, context.line, context.column)
                } else {
                    format!("line {}:{}", context.line, context.column)
                };
                output.push_str(&format!("  {} {}\n", "-->".cyan().bold(), location));
                output.push_str(&format!(
                    " {} {} {}\n",
                    format!("{:>3}", context.line).cyan().bold(),
                    "|".cyan().bold(),
                    context.line_text
                ));
                output.push_str(&format!(
                    "   {} {}{} {}\n",
                    "|".cyan().bold(),
                    " ".repeat(context.column - 1),
                    "^".repeat(context.span_length.max(1)).blue().bold(),
                    label.blue()
                ));
            }
        }

        output
    }

    /// Get source context for a span
    fn get_source_context(&self, span: Span) -> Option<SourceContext> {
        if span.start >= self.source.len() {
            return None;
        }

        // Find the line containing the span
        let mut line_num = 1;
        let mut line_start = 0;
        let mut current_pos = 0;

        for (idx, ch) in self.source.char_indices() {
            if idx >= span.start {
                break;
            }
            if ch == '\n' {
                line_num += 1;
                line_start = idx + 1;
            }
            current_pos = idx;
        }

        // Ensure we're at or past the span start
        if current_pos < span.start && line_start < span.start {
            // Move line_start forward if needed
            while line_start < span.start && line_start < self.source.len() {
                if self.source.as_bytes().get(line_start) == Some(&b'\n') {
                    line_start += 1;
                    break;
                }
                line_start += 1;
            }
        }

        // Find the end of the line
        let line_end = self.source[line_start..]
            .find('\n')
            .map(|pos| line_start + pos)
            .unwrap_or(self.source.len());

        // Extract the line text
        let line_text = self.source[line_start..line_end].to_string();

        // Calculate column (1-indexed)
        let column = span.start.saturating_sub(line_start) + 1;

        // Calculate span length (within this line)
        let span_length = if span.end <= line_end {
            span.end.saturating_sub(span.start)
        } else {
            line_end.saturating_sub(span.start)
        };

        Some(SourceContext {
            line: line_num,
            column,
            line_text,
            span_length,
        })
    }
}

/// Source context for error reporting
struct SourceContext {
    line: usize,
    column: usize,
    line_text: String,
    span_length: usize,
}

/// Helper trait for formatting errors with source context
pub trait DiagnosticError: fmt::Display {
    /// Get the error code (e.g., "E001")
    fn error_code(&self) -> &str;

    /// Get the error title
    fn title(&self) -> String {
        self.to_string()
    }

    /// Get the span if available
    fn span(&self) -> Option<Span> {
        None
    }

    /// Get a help message if applicable
    fn help(&self) -> Option<String> {
        None
    }

    /// Get the localized title
    fn localized_title(&self) -> String {
        self.title()
    }

    /// Get the localized message
    fn localized_message(&self) -> String {
        self.to_string()
    }

    /// Get the localized help message
    fn localized_help(&self) -> Option<String> {
        self.help()
    }

    /// Format the error with source context
    fn format_with_source(&self, source: &str, filename: Option<&str>) -> String {
        let reporter = ErrorReporter::new(source);
        let reporter = if let Some(f) = filename {
            reporter.with_filename(f)
        } else {
            reporter
        };

        reporter.format_error(
            self.error_code(),
            &self.title(),
            self.span(),
            &self.to_string(),
            self.help().as_deref(),
            &[],
        )
    }

    /// Format the error with source context using localized messages
    fn format_localized(&self, source: &str, filename: Option<&str>) -> String {
        let reporter = ErrorReporter::new(source);
        let reporter = if let Some(f) = filename {
            reporter.with_filename(f)
        } else {
            reporter
        };

        reporter.format_error(
            self.error_code(),
            &self.localized_title(),
            self.span(),
            &self.localized_message(),
            self.localized_help().as_deref(),
            &[],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_reporter() {
        let source = "F add(a: i64, b: i64) -> i64 = a + b\nF main() = add(1, \"hello\")";
        let reporter = ErrorReporter::new(source).with_filename("test.vais");

        let span = Span::new(44, 51); // "hello"
        let output = reporter.format_error(
            "E001",
            "Type mismatch",
            Some(span),
            "expected i64, found Str",
            Some("consider converting the string to a number"),
            &[],
        );

        println!("{}", output);
        assert!(output.contains("error"));
        assert!(output.contains("E001"));
        assert!(output.contains("Type mismatch"));
    }

    #[test]
    fn test_error_without_span() {
        let source = "F main() = 42";
        let reporter = ErrorReporter::new(source);

        let output = reporter.format_error(
            "E002",
            "Cannot infer type",
            None,
            "type inference failed",
            None,
            &[],
        );

        println!("{}", output);
        assert!(output.contains("error"));
        assert!(output.contains("E002"));
    }
}
