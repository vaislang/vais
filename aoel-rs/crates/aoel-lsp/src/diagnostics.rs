//! Diagnostics generation

use tower_lsp::lsp_types::*;
use aoel_lexer::Span;

use crate::document::Document;

/// 파싱 및 타입 체크 후 진단 정보 생성
pub fn generate_diagnostics(doc: &Document) -> Vec<Diagnostic> {
    let source = doc.text();
    let mut diagnostics = Vec::new();

    // 파싱
    let program = match aoel_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            let span = get_error_span(&e);
            let range = span_to_range(doc, span);
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("aoel".to_string()),
                message: format!("{:?}", e),
                ..Default::default()
            });
            return diagnostics;
        }
    };

    // 타입 체크
    if let Err(e) = aoel_typeck::check(&program) {
        let span = get_type_error_span(&e);
        let range = span_to_range(doc, span);
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("aoel".to_string()),
            message: format!("{}", e),
            ..Default::default()
        });
    }

    diagnostics
}

fn get_error_span(e: &aoel_parser::ParseError) -> Span {
    match e {
        aoel_parser::ParseError::UnexpectedToken { span, .. } => *span,
        aoel_parser::ParseError::UnexpectedEof { span } => *span,
        aoel_parser::ParseError::InvalidNumber { span, .. } => *span,
        aoel_parser::ParseError::InvalidSyntax { span, .. } => *span,
        aoel_parser::ParseError::LexError { span, .. } => *span,
        aoel_parser::ParseError::ModuleNotFound { span, .. } => *span,
        aoel_parser::ParseError::ModuleError { span, .. } => *span,
    }
}

fn get_type_error_span(e: &aoel_typeck::TypeError) -> Span {
    match e {
        aoel_typeck::TypeError::Mismatch { span, .. } => *span,
        aoel_typeck::TypeError::UndefinedVariable { span, .. } => *span,
        aoel_typeck::TypeError::UndefinedFunction { span, .. } => *span,
        aoel_typeck::TypeError::ArgumentCount { span, .. } => *span,
        aoel_typeck::TypeError::RecursiveInference { span } => *span,
        aoel_typeck::TypeError::InvalidOperator { span, .. } => *span,
        aoel_typeck::TypeError::InvalidIndex { span, .. } => *span,
        aoel_typeck::TypeError::InvalidField { span, .. } => *span,
        aoel_typeck::TypeError::NotAFunction { span, .. } => *span,
        _ => Span::default(),
    }
}

fn span_to_range(doc: &Document, span: Span) -> Range {
    Range {
        start: doc.offset_to_position(span.start),
        end: doc.offset_to_position(span.end),
    }
}
