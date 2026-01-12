//! Lexer error types

use thiserror::Error;
use crate::token::Span;

/// Errors that can occur during lexing
#[derive(Error, Debug, Clone)]
pub enum LexerError {
    #[error("Invalid token at position {}: '{}'", .span.start, .text)]
    InvalidToken {
        span: Span,
        text: String,
    },

    #[error("Unterminated string literal starting at position {}", .span.start)]
    UnterminatedString {
        span: Span,
    },

    #[error("Unterminated regex literal starting at position {}", .span.start)]
    UnterminatedRegex {
        span: Span,
    },

    #[error("Invalid escape sequence '\\{}' at position {}", .char, .offset)]
    InvalidEscape {
        char: char,
        offset: usize,
    },
}

impl LexerError {
    /// Get the span of the error, if available
    pub fn span(&self) -> Option<Span> {
        match self {
            LexerError::InvalidToken { span, .. } => Some(*span),
            LexerError::UnterminatedString { span } => Some(*span),
            LexerError::UnterminatedRegex { span } => Some(*span),
            LexerError::InvalidEscape { offset, .. } => Some(Span::new(*offset, *offset + 1)),
        }
    }
}
