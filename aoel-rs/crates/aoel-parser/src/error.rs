//! Parser error types
//!
//! Provides detailed error messages for parse failures.

use aoel_lexer::{Span, TokenKind};
use thiserror::Error;
use ariadne::{Color, Label, Report, ReportKind, Source};

/// Parse error types
#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: TokenKind,
        span: Span,
    },

    #[error("Unexpected end of file")]
    UnexpectedEof {
        expected: String,
        span: Span,
    },

    #[error("Invalid token")]
    InvalidToken {
        span: Span,
    },

    #[error("Expected block keyword")]
    ExpectedBlockKeyword {
        found: TokenKind,
        span: Span,
    },

    #[error("Missing required block: {block}")]
    MissingBlock {
        block: &'static str,
        span: Span,
    },

    #[error("Invalid version format: {version}")]
    InvalidVersion {
        version: String,
        span: Span,
    },

    #[error("Invalid type: {ty}")]
    InvalidType {
        ty: String,
        span: Span,
    },

    #[error("Invalid operator: {op}")]
    InvalidOperator {
        op: String,
        span: Span,
    },

    #[error("Invalid goal type: {goal}")]
    InvalidGoalType {
        goal: String,
        span: Span,
    },

    #[error("Invalid constraint kind: {kind}")]
    InvalidConstraintKind {
        kind: String,
        span: Span,
    },

    #[error("Invalid flow operation: {op}")]
    InvalidFlowOp {
        op: String,
        span: Span,
    },

    #[error("Invalid meta key: {key}")]
    InvalidMetaKey {
        key: String,
        span: Span,
    },

    #[error("Duplicate definition: {name}")]
    DuplicateDefinition {
        name: String,
        span: Span,
    },

    #[error("Invalid expression")]
    InvalidExpression {
        span: Span,
    },

    #[error("Invalid literal")]
    InvalidLiteral {
        span: Span,
    },

    #[error("{message}")]
    Custom {
        message: String,
        span: Span,
    },
}

impl ParseError {
    /// Get the span of this error
    pub fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken { span, .. } => *span,
            ParseError::UnexpectedEof { span, .. } => *span,
            ParseError::InvalidToken { span } => *span,
            ParseError::ExpectedBlockKeyword { span, .. } => *span,
            ParseError::MissingBlock { span, .. } => *span,
            ParseError::InvalidVersion { span, .. } => *span,
            ParseError::InvalidType { span, .. } => *span,
            ParseError::InvalidOperator { span, .. } => *span,
            ParseError::InvalidGoalType { span, .. } => *span,
            ParseError::InvalidConstraintKind { span, .. } => *span,
            ParseError::InvalidFlowOp { span, .. } => *span,
            ParseError::InvalidMetaKey { span, .. } => *span,
            ParseError::DuplicateDefinition { span, .. } => *span,
            ParseError::InvalidExpression { span } => *span,
            ParseError::InvalidLiteral { span } => *span,
            ParseError::Custom { span, .. } => *span,
        }
    }

    /// Generate a pretty error report using ariadne
    pub fn report(&self, source: &str, filename: &str) -> String {
        let mut output = Vec::new();

        let span = self.span();
        let label_message = self.to_string();

        Report::build(ReportKind::Error, filename, span.start)
            .with_message(self.error_title())
            .with_label(
                Label::new((filename, span.start..span.end))
                    .with_message(&label_message)
                    .with_color(Color::Red),
            )
            .with_note(self.help_message())
            .finish()
            .write((filename, Source::from(source)), &mut output)
            .unwrap();

        String::from_utf8(output).unwrap()
    }

    fn error_title(&self) -> &'static str {
        match self {
            ParseError::UnexpectedToken { .. } => "Syntax Error",
            ParseError::UnexpectedEof { .. } => "Unexpected End of File",
            ParseError::InvalidToken { .. } => "Invalid Token",
            ParseError::ExpectedBlockKeyword { .. } => "Expected Block Keyword",
            ParseError::MissingBlock { .. } => "Missing Required Block",
            ParseError::InvalidVersion { .. } => "Invalid Version Format",
            ParseError::InvalidType { .. } => "Invalid Type",
            ParseError::InvalidOperator { .. } => "Invalid Operator",
            ParseError::InvalidGoalType { .. } => "Invalid Goal Type",
            ParseError::InvalidConstraintKind { .. } => "Invalid Constraint",
            ParseError::InvalidFlowOp { .. } => "Invalid Flow Operation",
            ParseError::InvalidMetaKey { .. } => "Invalid Meta Key",
            ParseError::DuplicateDefinition { .. } => "Duplicate Definition",
            ParseError::InvalidExpression { .. } => "Invalid Expression",
            ParseError::InvalidLiteral { .. } => "Invalid Literal",
            ParseError::Custom { .. } => "Parse Error",
        }
    }

    fn help_message(&self) -> String {
        match self {
            ParseError::UnexpectedToken { expected, .. } => {
                format!("Expected {}", expected)
            }
            ParseError::UnexpectedEof { expected, .. } => {
                format!("The file ended unexpectedly. Expected {}", expected)
            }
            ParseError::InvalidToken { .. } => {
                "This character or sequence is not valid in AOEL".to_string()
            }
            ParseError::ExpectedBlockKeyword { .. } => {
                "Expected one of: META, INPUT, OUTPUT, INTENT, CONSTRAINT, FLOW, EXECUTION, VERIFY, END".to_string()
            }
            ParseError::MissingBlock { block, .. } => {
                format!("The {} block is required in every AOEL unit", block)
            }
            ParseError::InvalidVersion { .. } => {
                "Version should be in format V<major>.<minor>.<patch>, e.g., V1.0.0".to_string()
            }
            ParseError::InvalidType { .. } => {
                "Valid types: INT, FLOAT64, STRING, BOOL, ARRAY<T>, OPTIONAL<T>, etc.".to_string()
            }
            ParseError::InvalidOperator { .. } => {
                "Valid operators: AND, OR, NOT, ==, !=, <, >, <=, >=, +, -, *, /".to_string()
            }
            ParseError::InvalidGoalType { .. } => {
                "Valid goals: TRANSFORM, VALIDATE, AGGREGATE, FILTER, ROUTE, COMPOSE, FETCH".to_string()
            }
            ParseError::InvalidConstraintKind { .. } => {
                "Valid constraints: REQUIRE, FORBID, PREFER, INVARIANT".to_string()
            }
            ParseError::InvalidFlowOp { .. } => {
                "Valid operations: MAP, FILTER, REDUCE, TRANSFORM, BRANCH, MERGE, CALL, etc.".to_string()
            }
            ParseError::InvalidMetaKey { .. } => {
                "Valid meta keys: DOMAIN, DETERMINISM, IDEMPOTENT, PURE, TIMEOUT, RETRY".to_string()
            }
            ParseError::DuplicateDefinition { name, .. } => {
                format!("'{}' was already defined earlier in this scope", name)
            }
            ParseError::InvalidExpression { .. } => {
                "Expected a valid expression (literal, identifier, function call, etc.)".to_string()
            }
            ParseError::InvalidLiteral { .. } => {
                "Expected a literal value (number, string, boolean)".to_string()
            }
            ParseError::Custom { .. } => {
                "Check the syntax and try again".to_string()
            }
        }
    }
}

/// Result type alias for parser operations
pub type ParseResult<T> = Result<T, ParseError>;
