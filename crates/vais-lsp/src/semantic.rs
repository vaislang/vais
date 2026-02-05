//! Semantic token generation for Vais LSP

use tower_lsp::lsp_types::SemanticToken;
use vais_lexer::{tokenize, Token};

/// Token type indices (must match legend in backend.rs)
#[allow(dead_code)]
const TOKEN_FUNCTION: u32 = 0;
const TOKEN_VARIABLE: u32 = 1;
#[allow(dead_code)]
const TOKEN_STRUCT: u32 = 2;
#[allow(dead_code)]
const TOKEN_ENUM: u32 = 3;
const TOKEN_KEYWORD: u32 = 4;
const TOKEN_TYPE: u32 = 5;
const TOKEN_NUMBER: u32 = 6;
const TOKEN_STRING: u32 = 7;
#[allow(dead_code)]
const TOKEN_COMMENT: u32 = 8;
#[allow(dead_code)]
const TOKEN_PARAMETER: u32 = 9;

/// Generate semantic tokens for syntax highlighting
pub fn get_semantic_tokens(source: &str) -> Vec<SemanticToken> {
    let tokens = match tokenize(source) {
        Ok(t) => t,
        Err(_) => return vec![],
    };
    let mut result = Vec::new();

    let mut prev_line = 0u32;
    let mut prev_col = 0u32;

    for spanned in &tokens {
        let (line, col) = offset_to_line_col(source, spanned.span.start);
        let length = (spanned.span.end - spanned.span.start) as u32;

        let token_type = match &spanned.token {
            // Keywords (single letter in Vais)
            Token::Function
            | Token::Struct
            | Token::Enum
            | Token::If
            | Token::Loop
            | Token::Match
            | Token::Return
            | Token::Break
            | Token::Continue
            | Token::TypeKeyword
            | Token::Use
            | Token::Pub
            | Token::Trait
            | Token::Impl
            | Token::Async
            | Token::Spawn
            | Token::Await => Some(TOKEN_KEYWORD),

            // Types
            Token::I8
            | Token::I16
            | Token::I32
            | Token::I64
            | Token::I128
            | Token::U8
            | Token::U16
            | Token::U32
            | Token::U64
            | Token::U128
            | Token::F32
            | Token::F64
            | Token::Bool
            | Token::Str => Some(TOKEN_TYPE),

            // Literals
            Token::Int(_) => Some(TOKEN_NUMBER),
            Token::Float(_) => Some(TOKEN_NUMBER),
            Token::String(_) => Some(TOKEN_STRING),
            Token::True | Token::False => Some(TOKEN_KEYWORD),

            // Identifiers - context-dependent, default to variable
            Token::Ident(_) => Some(TOKEN_VARIABLE),

            // Skip operators and punctuation
            _ => None,
        };

        if let Some(token_type) = token_type {
            let delta_line = line - prev_line;
            let delta_col = if delta_line == 0 { col - prev_col } else { col };

            result.push(SemanticToken {
                delta_line,
                delta_start: delta_col,
                length,
                token_type,
                token_modifiers_bitset: 0,
            });

            prev_line = line;
            prev_col = col;
        }
    }

    result
}

/// Convert byte offset to line and column
fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;

    for (i, c) in source.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_tokens() {
        let source = "F main() -> i64 { 42 }";
        let tokens = get_semantic_tokens(source);
        assert!(!tokens.is_empty());
    }
}
