//! Parser Errors

use aoel_v6b_lexer::{Span, TokenKind};
use thiserror::Error;

/// 파서 에러
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: TokenKind,
        span: Span,
    },

    #[error("Unexpected end of file")]
    UnexpectedEof { span: Span },

    #[error("Invalid number: {message}")]
    InvalidNumber { message: String, span: Span },

    #[error("Invalid syntax: {message}")]
    InvalidSyntax { message: String, span: Span },

    #[error("Lexer error: {message}")]
    LexError { message: String, span: Span },
}

impl ParseError {
    pub fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken { span, .. } => *span,
            ParseError::UnexpectedEof { span } => *span,
            ParseError::InvalidNumber { span, .. } => *span,
            ParseError::InvalidSyntax { span, .. } => *span,
            ParseError::LexError { span, .. } => *span,
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
