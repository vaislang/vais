//! Diagnostics generation for Vais LSP

use tower_lsp::lsp_types::*;
use vais_parser::ParseError;

/// Convert parse errors to LSP diagnostics
#[allow(dead_code)]
pub fn parse_error_to_diagnostic(err: &ParseError, source: &str) -> Diagnostic {
    let (start, end, message) = match err {
        ParseError::UnexpectedToken { found, span, expected } => {
            let pos = offset_to_position(source, span.start);
            let end_pos = offset_to_position(source, span.end);
            (
                pos,
                end_pos,
                format!("Unexpected token {:?}, expected {}", found, expected),
            )
        }
        ParseError::UnexpectedEof => {
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

/// Publish diagnostics to the client
#[allow(dead_code)]
pub fn publish_diagnostics(errors: Vec<ParseError>, source: &str) -> Vec<Diagnostic> {
    errors
        .iter()
        .map(|err| parse_error_to_diagnostic(err, source))
        .collect()
}
