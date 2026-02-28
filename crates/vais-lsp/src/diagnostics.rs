//! Diagnostics generation for Vais LSP

use tower_lsp::lsp_types::*;
use vais_parser::ParseError;

/// Convert parse errors to LSP diagnostics
pub fn parse_error_to_diagnostic(err: &ParseError, source: &str) -> Diagnostic {
    let (start, end, message) = match err {
        ParseError::UnexpectedToken {
            found,
            span,
            expected,
        } => {
            let pos = offset_to_position(source, span.start);
            let end_pos = offset_to_position(source, span.end);
            (
                pos,
                end_pos,
                format!("Unexpected token {:?}, expected {}", found, expected),
            )
        }
        ParseError::UnexpectedEof { .. } => {
            let pos = offset_to_position(source, source.len());
            (pos, pos, "Unexpected end of file".to_string())
        }
        ParseError::InvalidExpression => {
            let pos = Position::new(0, 0);
            (pos, pos, "Invalid expression".to_string())
        }
    };

    Diagnostic {
        range: Range { start, end },
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: Some("vais".to_string()),
        message,
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Convert byte offset to LSP position
fn offset_to_position(source: &str, offset: usize) -> Position {
    let mut line = 0;
    let mut col = 0;

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

    Position::new(line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_lexer::Token;

    // ========================================================================
    // offset_to_position tests
    // ========================================================================

    #[test]
    fn test_offset_to_position_start() {
        let pos = offset_to_position("hello", 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn test_offset_to_position_middle_of_line() {
        let pos = offset_to_position("hello world", 5);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn test_offset_to_position_end_of_line() {
        let pos = offset_to_position("hello", 5);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn test_offset_to_position_second_line() {
        let pos = offset_to_position("line1\nline2", 6);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn test_offset_to_position_second_line_middle() {
        let pos = offset_to_position("line1\nline2", 9);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 3);
    }

    #[test]
    fn test_offset_to_position_third_line() {
        let pos = offset_to_position("a\nb\nc", 4);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn test_offset_to_position_empty_string() {
        let pos = offset_to_position("", 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn test_offset_to_position_newline_at_offset() {
        let pos = offset_to_position("abc\ndef", 3);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 3);
    }

    #[test]
    fn test_offset_to_position_multiple_newlines() {
        let pos = offset_to_position("\n\n\n", 2);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn test_offset_to_position_past_end() {
        let pos = offset_to_position("abc", 10);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 3);
    }

    #[test]
    fn test_offset_to_position_vais_code() {
        let source = "F main() -> i64 {\n    R 42\n}";
        // "R" is at offset 22 (line 1, col 4)
        let pos = offset_to_position(source, 22);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 4);
    }

    #[test]
    fn test_offset_to_position_empty_lines() {
        let source = "line1\n\nline3";
        // "line3" starts at offset 7
        let pos = offset_to_position(source, 7);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.character, 0);
    }

    // ========================================================================
    // parse_error_to_diagnostic tests
    // ========================================================================

    #[test]
    fn test_unexpected_token_diagnostic() {
        let err = ParseError::UnexpectedToken {
            found: Token::Int(42),
            span: 5..7,
            expected: "identifier".to_string(),
        };
        let diag = parse_error_to_diagnostic(&err, "F 42() -> i64 { 0 }");

        assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diag.source, Some("vais".to_string()));
        assert!(diag.message.contains("Unexpected token"));
        assert!(diag.message.contains("identifier"));
    }

    #[test]
    fn test_unexpected_eof_diagnostic() {
        let err = ParseError::UnexpectedEof { span: 10..10 };
        let source = "F main() {";
        let diag = parse_error_to_diagnostic(&err, source);

        assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diag.message, "Unexpected end of file");
        // Start should be at end of source
        assert_eq!(diag.range.start, diag.range.end);
    }

    #[test]
    fn test_invalid_expression_diagnostic() {
        let err = ParseError::InvalidExpression;
        let diag = parse_error_to_diagnostic(&err, "some code");

        assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diag.message, "Invalid expression");
        assert_eq!(diag.range.start.line, 0);
        assert_eq!(diag.range.start.character, 0);
    }

    #[test]
    fn test_diagnostic_source_is_vais() {
        let err = ParseError::InvalidExpression;
        let diag = parse_error_to_diagnostic(&err, "");
        assert_eq!(diag.source, Some("vais".to_string()));
    }

    #[test]
    fn test_diagnostic_has_no_code() {
        let err = ParseError::InvalidExpression;
        let diag = parse_error_to_diagnostic(&err, "");
        assert!(diag.code.is_none());
        assert!(diag.code_description.is_none());
    }

    #[test]
    fn test_diagnostic_has_no_related_info() {
        let err = ParseError::InvalidExpression;
        let diag = parse_error_to_diagnostic(&err, "");
        assert!(diag.related_information.is_none());
        assert!(diag.tags.is_none());
        assert!(diag.data.is_none());
    }

    #[test]
    fn test_unexpected_token_range() {
        let err = ParseError::UnexpectedToken {
            found: Token::Int(0),
            span: 0..3,
            expected: "F".to_string(),
        };
        let diag = parse_error_to_diagnostic(&err, "abc def");

        assert_eq!(diag.range.start.line, 0);
        assert_eq!(diag.range.start.character, 0);
        assert_eq!(diag.range.end.line, 0);
        assert_eq!(diag.range.end.character, 3);
    }

    #[test]
    fn test_unexpected_token_multiline() {
        let source = "F main() {\n    42\n}";
        let err = ParseError::UnexpectedToken {
            found: Token::Int(42),
            span: 15..17,
            expected: "statement".to_string(),
        };
        let diag = parse_error_to_diagnostic(&err, source);
        assert_eq!(diag.range.start.line, 1);
    }

    #[test]
    fn test_unexpected_eof_empty_source() {
        let err = ParseError::UnexpectedEof { span: 0..0 };
        let diag = parse_error_to_diagnostic(&err, "");
        assert_eq!(diag.range.start.line, 0);
        assert_eq!(diag.range.start.character, 0);
    }
}
