//! Diagnostics generation

use tower_lsp::lsp_types::*;
use vais_lexer::Span;

use crate::document::Document;

/// 파싱 및 타입 체크 후 진단 정보 생성
pub fn generate_diagnostics(doc: &Document) -> Vec<Diagnostic> {
    let source = doc.text();
    let mut diagnostics = Vec::new();

    // 파싱
    let program = match vais_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            let span = get_error_span(&e);
            let range = span_to_range(doc, span);
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("vais".to_string()),
                message: format!("{:?}", e),
                ..Default::default()
            });
            return diagnostics;
        }
    };

    // 타입 체크
    if let Err(e) = vais_typeck::check(&program) {
        let span = get_type_error_span(&e);
        let range = span_to_range(doc, span);
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("vais".to_string()),
            message: format!("{}", e),
            ..Default::default()
        });
    }

    diagnostics
}

fn get_error_span(e: &vais_parser::ParseError) -> Span {
    match e {
        vais_parser::ParseError::UnexpectedToken { span, .. } => *span,
        vais_parser::ParseError::UnexpectedEof { span } => *span,
        vais_parser::ParseError::InvalidNumber { span, .. } => *span,
        vais_parser::ParseError::InvalidSyntax { span, .. } => *span,
        vais_parser::ParseError::LexError { span, .. } => *span,
        vais_parser::ParseError::ModuleNotFound { span, .. } => *span,
        vais_parser::ParseError::ModuleError { span, .. } => *span,
    }
}

fn get_type_error_span(e: &vais_typeck::TypeError) -> Span {
    match e {
        vais_typeck::TypeError::Mismatch { span, .. } => *span,
        vais_typeck::TypeError::UndefinedVariable { span, .. } => *span,
        vais_typeck::TypeError::UndefinedFunction { span, .. } => *span,
        vais_typeck::TypeError::ArgumentCount { span, .. } => *span,
        vais_typeck::TypeError::RecursiveInference { span } => *span,
        vais_typeck::TypeError::InvalidOperator { span, .. } => *span,
        vais_typeck::TypeError::InvalidIndex { span, .. } => *span,
        vais_typeck::TypeError::InvalidField { span, .. } => *span,
        vais_typeck::TypeError::NotAFunction { span, .. } => *span,
        _ => Span::default(),
    }
}

fn span_to_range(doc: &Document, span: Span) -> Range {
    Range {
        start: doc.offset_to_position(span.start),
        end: doc.offset_to_position(span.end),
    }
}
