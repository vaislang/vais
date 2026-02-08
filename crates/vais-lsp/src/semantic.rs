//! Semantic token generation for Vais LSP

use tower_lsp::lsp_types::SemanticToken;
use vais_lexer::{tokenize, Token};

/// Token type indices (must match legend in backend.rs)
const TOKEN_FUNCTION: u32 = 0;
const TOKEN_VARIABLE: u32 = 1;
const TOKEN_STRUCT: u32 = 2;
const TOKEN_ENUM: u32 = 3;
const TOKEN_KEYWORD: u32 = 4;
const TOKEN_TYPE: u32 = 5;
const TOKEN_NUMBER: u32 = 6;
const TOKEN_STRING: u32 = 7;
const TOKEN_COMMENT: u32 = 8;
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

    // Track context for smarter classification
    let mut prev_token: Option<&Token> = None;
    let mut in_function_params = false; // Track if we're inside function parameter list

    for (idx, spanned) in tokens.iter().enumerate() {
        let (line, col) = offset_to_line_col(source, spanned.span.start);
        let length = (spanned.span.end - spanned.span.start) as u32;

        let token_type = match &spanned.token {
            // Doc comments
            Token::DocComment(_) => Some(TOKEN_COMMENT),

            // Keywords (single letter in Vais)
            Token::Function
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
            | Token::Await
            | Token::Mut
            | Token::Const
            | Token::Extern
            | Token::Global
            | Token::Defer
            | Token::Yield => Some(TOKEN_KEYWORD),

            // Struct keyword - highlight following identifier as struct name
            Token::Struct => Some(TOKEN_KEYWORD),

            // Enum keyword - highlight following identifier as enum name
            Token::Enum => Some(TOKEN_KEYWORD),

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

            // Track parentheses for function parameters
            Token::LParen => {
                // If previous token was a function name (after F keyword or function call)
                // then we're entering function parameters
                if matches!(prev_token, Some(Token::Ident(_))) {
                    // Check if the identifier before LParen came after F keyword
                    if idx >= 2 {
                        if let Token::Function = tokens[idx - 2].token {
                            in_function_params = true;
                        }
                    }
                }
                None
            }

            Token::RParen => {
                in_function_params = false;
                None
            }

            // Identifiers - context-dependent classification
            Token::Ident(name) => {
                // Inside function parameter list
                if in_function_params {
                    // Skip type names in parameters (after ':')
                    if matches!(prev_token, Some(Token::Colon)) {
                        None // Type names handled separately
                    } else {
                        Some(TOKEN_PARAMETER)
                    }
                } else {
                    // Check previous token for context
                    match prev_token {
                        // After F keyword: function definition
                        Some(Token::Function) => Some(TOKEN_FUNCTION),

                        // After S keyword: struct definition
                        Some(Token::Struct) => Some(TOKEN_STRUCT),

                        // After E keyword: enum definition
                        Some(Token::Enum) => Some(TOKEN_ENUM),

                        // Uppercase first letter often indicates type/struct/enum usage
                        _ if name.chars().next().is_some_and(|c| c.is_uppercase()) => {
                            Some(TOKEN_STRUCT)
                        }

                        // Check if next token is '(' - indicates function call
                        _ => {
                            if idx + 1 < tokens.len() {
                                if let Token::LParen = tokens[idx + 1].token {
                                    Some(TOKEN_FUNCTION)
                                } else {
                                    Some(TOKEN_VARIABLE)
                                }
                            } else {
                                Some(TOKEN_VARIABLE)
                            }
                        }
                    }
                }
            }

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

        prev_token = Some(&spanned.token);
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

    #[test]
    fn test_semantic_tokens_comprehensive() {
        let source = r#"
/// This is a doc comment
F add(x: i64, y: i64) -> i64 {
    R x + y
}

S Point {
    x: i64,
    y: i64,
}

E Option<T> {
    Some(T),
    None,
}

F main() -> i64 {
    result := add(1, 2)
    p := Point { x: 10, y: 20 }
    R result
}
"#;
        let tokens = get_semantic_tokens(source);

        // Should have tokens for:
        // - DocComment (TOKEN_COMMENT)
        // - Function keywords (TOKEN_KEYWORD)
        // - Function names after F (TOKEN_FUNCTION)
        // - Function parameters (TOKEN_PARAMETER)
        // - Struct keyword and name (TOKEN_KEYWORD + TOKEN_STRUCT)
        // - Enum keyword and name (TOKEN_KEYWORD + TOKEN_ENUM)
        // - Type names (TOKEN_TYPE)
        // - Numbers (TOKEN_NUMBER)
        // - Variables (TOKEN_VARIABLE)
        // - Function calls (TOKEN_FUNCTION)

        assert!(!tokens.is_empty());

        // Count token types
        let mut comment_count = 0;
        let mut function_count = 0;
        let mut struct_count = 0;
        let mut enum_count = 0;
        let mut parameter_count = 0;

        for token in &tokens {
            match token.token_type {
                TOKEN_COMMENT => comment_count += 1,
                TOKEN_FUNCTION => function_count += 1,
                TOKEN_STRUCT => struct_count += 1,
                TOKEN_ENUM => enum_count += 1,
                TOKEN_PARAMETER => parameter_count += 1,
                _ => {}
            }
        }

        // Verify we found the expected token types
        assert!(comment_count > 0, "Should find doc comments");
        assert!(function_count > 0, "Should find function names");
        assert!(struct_count > 0, "Should find struct names");
        assert!(enum_count > 0, "Should find enum names");
        assert!(parameter_count > 0, "Should find function parameters");
    }

    #[test]
    fn test_function_call_vs_definition() {
        let source = r#"
F foo(x: i64) -> i64 { R x }
F main() -> i64 {
    result := foo(42)
    R result
}
"#;
        let tokens = get_semantic_tokens(source);

        let function_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_FUNCTION)
            .collect();

        // Should have function names: 'foo' (definition), 'main' (definition), 'foo' (call)
        assert!(
            function_tokens.len() >= 3,
            "Should find at least 3 function names (2 definitions + 1 call)"
        );
    }

    #[test]
    fn test_struct_name_detection() {
        let source = r#"
S MyStruct { x: i64 }
F main() -> i64 {
    obj := MyStruct { x: 10 }
    R obj.x
}
"#;
        let tokens = get_semantic_tokens(source);

        let struct_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_STRUCT)
            .collect();

        // Should find 'MyStruct' in definition and usage (uppercase names)
        assert!(
            struct_tokens.len() >= 2,
            "Should find struct name in definition and usage"
        );
    }

    #[test]
    fn test_parameter_detection() {
        let source = "F add(x: i64, y: i64) -> i64 { R x + y }";
        let tokens = get_semantic_tokens(source);

        let param_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_PARAMETER)
            .collect();

        // Should find parameters 'x' and 'y'
        assert!(
            param_tokens.len() >= 2,
            "Should find at least 2 parameters"
        );
    }
}
