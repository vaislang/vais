//! AOEL v6b Lexer Errors

use thiserror::Error;

use crate::token::Span;

/// 렉서 에러
#[derive(Debug, Error)]
pub enum LexError {
    #[error("Unexpected character '{char}' at position {}", span.start)]
    UnexpectedCharacter { char: char, span: Span },

    #[error("Unterminated string starting at position {}", span.start)]
    UnterminatedString { span: Span },

    #[error("Invalid number format at position {}", span.start)]
    InvalidNumber { span: Span },
}

impl LexError {
    /// 에러 위치 반환
    pub fn span(&self) -> Span {
        match self {
            LexError::UnexpectedCharacter { span, .. } => *span,
            LexError::UnterminatedString { span } => *span,
            LexError::InvalidNumber { span } => *span,
        }
    }
}
