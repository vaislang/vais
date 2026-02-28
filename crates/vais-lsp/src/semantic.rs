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
        assert!(param_tokens.len() >= 2, "Should find at least 2 parameters");
    }

    // ========== offset_to_line_col tests ==========

    #[test]
    fn test_offset_to_line_col_start() {
        let (line, col) = offset_to_line_col("hello", 0);
        assert_eq!(line, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_offset_to_line_col_middle() {
        let (line, col) = offset_to_line_col("hello", 3);
        assert_eq!(line, 0);
        assert_eq!(col, 3);
    }

    #[test]
    fn test_offset_to_line_col_newline() {
        let (line, col) = offset_to_line_col("abc\ndef", 4);
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_offset_to_line_col_second_line() {
        let (line, col) = offset_to_line_col("abc\ndef", 6);
        assert_eq!(line, 1);
        assert_eq!(col, 2);
    }

    #[test]
    fn test_offset_to_line_col_multiple_lines() {
        let (line, col) = offset_to_line_col("a\nb\nc\nd", 6);
        assert_eq!(line, 3);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_offset_to_line_col_empty() {
        let (line, col) = offset_to_line_col("", 0);
        assert_eq!(line, 0);
        assert_eq!(col, 0);
    }

    // ========== Additional semantic token tests ==========

    #[test]
    fn test_semantic_tokens_empty_source() {
        let tokens = get_semantic_tokens("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_semantic_tokens_keyword_only() {
        let source = "F";
        let tokens = get_semantic_tokens(source);
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].token_type, TOKEN_KEYWORD);
    }

    #[test]
    fn test_semantic_tokens_number_literal() {
        let source = "F main() -> i64 { 42 }";
        let tokens = get_semantic_tokens(source);
        let number_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_NUMBER)
            .collect();
        assert!(!number_tokens.is_empty(), "Should find number literal");
    }

    #[test]
    fn test_semantic_tokens_string_literal() {
        let source = r#"F main() -> i64 { puts("hello") }"#;
        let tokens = get_semantic_tokens(source);
        let string_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_STRING)
            .collect();
        assert!(
            !string_tokens.is_empty(),
            "Should find string literal"
        );
    }

    #[test]
    fn test_semantic_tokens_type_annotation() {
        let source = "F foo(x: i64) -> f64 { 0.0 }";
        let tokens = get_semantic_tokens(source);
        let type_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_TYPE)
            .collect();
        assert!(
            type_tokens.len() >= 2,
            "Should find at least 2 type annotations (i64, f64)"
        );
    }

    #[test]
    fn test_semantic_tokens_variable() {
        let source = "F main() -> i64 { x := 5\n R x }";
        let tokens = get_semantic_tokens(source);
        let var_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_VARIABLE)
            .collect();
        assert!(!var_tokens.is_empty(), "Should find variables");
    }

    #[test]
    fn test_semantic_tokens_delta_line() {
        let source = "F main() -> i64 {\n    R 42\n}";
        let tokens = get_semantic_tokens(source);

        // Tokens should have proper delta_line values
        let mut has_nonzero_delta = false;
        for token in &tokens {
            if token.delta_line > 0 {
                has_nonzero_delta = true;
                break;
            }
        }
        assert!(
            has_nonzero_delta,
            "Multi-line code should have non-zero delta_line"
        );
    }

    #[test]
    fn test_semantic_tokens_delta_start_same_line() {
        let source = "F main() -> i64 = 42";
        let tokens = get_semantic_tokens(source);

        // All tokens on the same line should have delta_line == 0
        // and delta_start relative to previous token
        for token in &tokens {
            assert_eq!(
                token.delta_line, 0,
                "Single line source should have delta_line 0"
            );
        }
    }

    #[test]
    fn test_semantic_tokens_bool_keywords() {
        let source = "F main() -> bool { true }";
        let tokens = get_semantic_tokens(source);
        let keyword_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_KEYWORD)
            .collect();
        // Should have F and true as keywords
        assert!(keyword_tokens.len() >= 2);
    }

    #[test]
    fn test_semantic_tokens_all_keywords() {
        let source = "F I L M R B C W X U A";
        let tokens = get_semantic_tokens(source);
        let keyword_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_KEYWORD)
            .collect();
        // All single-letter keywords should be detected
        assert!(
            keyword_tokens.len() >= 8,
            "Should detect most single-letter keywords, got {}",
            keyword_tokens.len()
        );
    }

    #[test]
    fn test_semantic_tokens_all_types() {
        let source = "F foo(a: i8, b: i16, c: i32, d: i64, e: f32, f: f64, g: bool, h: str) -> i64 { 0 }";
        let tokens = get_semantic_tokens(source);
        let type_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_TYPE)
            .collect();
        assert!(
            type_tokens.len() >= 8,
            "Should find all type annotations, got {}",
            type_tokens.len()
        );
    }

    #[test]
    fn test_semantic_tokens_uppercase_identifier_is_struct() {
        let source = "F main() -> i64 { x := MyType\n R 0 }";
        let tokens = get_semantic_tokens(source);
        let struct_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_STRUCT)
            .collect();
        assert!(
            !struct_tokens.is_empty(),
            "Uppercase identifier should be classified as struct"
        );
    }

    #[test]
    fn test_semantic_tokens_enum_definition() {
        let source = "E Color { Red, Green, Blue }";
        let tokens = get_semantic_tokens(source);
        let enum_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_ENUM)
            .collect();
        assert!(
            !enum_tokens.is_empty(),
            "Should find enum name after E keyword"
        );
    }

    #[test]
    fn test_semantic_tokens_modifiers_zero() {
        let source = "F main() -> i64 { 0 }";
        let tokens = get_semantic_tokens(source);
        for token in &tokens {
            assert_eq!(
                token.token_modifiers_bitset, 0,
                "All token modifiers should be 0"
            );
        }
    }

    #[test]
    fn test_semantic_tokens_invalid_source() {
        // get_semantic_tokens returns empty Vec on error
        let source = "!!!@@@###$$$%%%";
        let tokens = get_semantic_tokens(source);
        // Should not panic - may return empty or partial
        let _ = tokens;
    }

    #[test]
    fn test_semantic_tokens_nested_calls() {
        let source = "F main() -> i64 { R foo(bar(1)) }";
        let tokens = get_semantic_tokens(source);
        let function_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_FUNCTION)
            .collect();
        // Should find 'main', 'foo', 'bar' as functions
        assert!(
            function_tokens.len() >= 3,
            "Should find nested function calls, got {}",
            function_tokens.len()
        );
    }

    #[test]
    fn test_semantic_tokens_doc_comment() {
        let source = "/// A doc comment\nF main() -> i64 { 0 }";
        let tokens = get_semantic_tokens(source);
        let comment_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_COMMENT)
            .collect();
        assert!(
            !comment_tokens.is_empty(),
            "Should find doc comment"
        );
    }

    #[test]
    fn test_semantic_tokens_multiple_functions() {
        let source = "F foo() -> i64 { 0 }\nF bar() -> i64 { 0 }\nF baz() -> i64 { 0 }";
        let tokens = get_semantic_tokens(source);
        let function_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_FUNCTION)
            .collect();
        assert!(
            function_tokens.len() >= 3,
            "Should find 3 function definitions"
        );
    }

    #[test]
    fn test_semantic_tokens_return_keyword() {
        let source = "F main() -> i64 { R 42 }";
        let tokens = get_semantic_tokens(source);
        let keyword_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_KEYWORD)
            .collect();
        // Should have F and R as keywords (at minimum)
        assert!(
            keyword_tokens.len() >= 2,
            "Should find F and R keywords"
        );
    }

    #[test]
    fn test_semantic_tokens_u_types() {
        let source = "F foo(a: u8, b: u16, c: u32, d: u64, e: u128) -> i64 { 0 }";
        let tokens = get_semantic_tokens(source);
        let type_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_TYPE)
            .collect();
        // Should find u8, u16, u32, u64, u128, and i64
        assert!(
            type_tokens.len() >= 5,
            "Should find unsigned type annotations, got {}",
            type_tokens.len()
        );
    }

    #[test]
    fn test_semantic_tokens_float_literal() {
        let source = "F main() -> f64 { 3.14 }";
        let tokens = get_semantic_tokens(source);
        let number_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_NUMBER)
            .collect();
        assert!(!number_tokens.is_empty(), "Should find float literal");
    }
}
