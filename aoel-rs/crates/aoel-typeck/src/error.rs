//! Type check error types
//!
//! Provides detailed error messages for type check failures.

use aoel_lexer::Span;
use ariadne::{Color, Label, Report, ReportKind, Source};
use thiserror::Error;

/// Type check error types
#[derive(Debug, Clone, Error)]
pub enum TypeCheckError {
    // === Symbol Table Errors ===

    #[error("Duplicate field definition: `{name}`")]
    DuplicateField {
        name: String,
        first_span: Span,
        duplicate_span: Span,
    },

    #[error("Duplicate node ID: `{id}`")]
    DuplicateNodeId {
        id: String,
        first_span: Span,
        duplicate_span: Span,
    },

    // === Reference Errors ===

    #[error("Undefined reference: `{name}`")]
    UndefinedReference {
        name: String,
        span: Span,
        suggestions: Vec<String>,
    },

    #[error("Invalid field access: `{field}` does not exist on `{base}`")]
    InvalidFieldAccess {
        base: String,
        field: String,
        span: Span,
        available_fields: Vec<String>,
    },

    #[error("Undefined node reference: `{id}`")]
    UndefinedNode {
        id: String,
        span: Span,
        available_nodes: Vec<String>,
    },

    // === Type Errors ===

    #[error("Type mismatch: expected `{expected}`, found `{found}`")]
    TypeMismatch {
        expected: String,
        found: String,
        span: Span,
        context: Option<String>,
    },

    #[error("Invalid operand type for `{operator}`: expected `{expected}`, found `{found}`")]
    InvalidOperandType {
        operator: String,
        expected: String,
        found: String,
        span: Span,
    },

    #[error("Constraint expression must be BOOL, found `{found}`")]
    NonBoolConstraint { found: String, span: Span },

    #[error("Verify expression must be BOOL, found `{found}`")]
    NonBoolVerify { found: String, span: Span },

    // === Flow Errors ===

    #[error("Invalid edge source: `{name}`")]
    InvalidEdgeSource { name: String, span: Span },

    #[error("Invalid edge target: `{name}`")]
    InvalidEdgeTarget { name: String, span: Span },

    #[error("Edge condition must be BOOL, found `{found}`")]
    NonBoolEdgeCondition { found: String, span: Span },

    // === Intent Errors ===

    #[error("GOAL references undefined {kind}: `{field}`")]
    UndefinedGoalField {
        field: String,
        kind: &'static str, // "input" or "output"
        span: Span,
    },

    // === General ===

    #[error("{message}")]
    Custom { message: String, span: Span },
}

impl TypeCheckError {
    /// Get the primary span of this error
    pub fn span(&self) -> Span {
        match self {
            TypeCheckError::DuplicateField { duplicate_span, .. } => *duplicate_span,
            TypeCheckError::DuplicateNodeId { duplicate_span, .. } => *duplicate_span,
            TypeCheckError::UndefinedReference { span, .. } => *span,
            TypeCheckError::InvalidFieldAccess { span, .. } => *span,
            TypeCheckError::UndefinedNode { span, .. } => *span,
            TypeCheckError::TypeMismatch { span, .. } => *span,
            TypeCheckError::InvalidOperandType { span, .. } => *span,
            TypeCheckError::NonBoolConstraint { span, .. } => *span,
            TypeCheckError::NonBoolVerify { span, .. } => *span,
            TypeCheckError::InvalidEdgeSource { span, .. } => *span,
            TypeCheckError::InvalidEdgeTarget { span, .. } => *span,
            TypeCheckError::NonBoolEdgeCondition { span, .. } => *span,
            TypeCheckError::UndefinedGoalField { span, .. } => *span,
            TypeCheckError::Custom { span, .. } => *span,
        }
    }

    /// Generate a pretty error report using ariadne
    pub fn report(&self, source: &str, filename: &str) -> String {
        let mut output = Vec::new();

        let span = self.span();
        let label_message = self.to_string();

        let mut builder = Report::build(ReportKind::Error, filename, span.start)
            .with_message(self.error_title())
            .with_label(
                Label::new((filename, span.start..span.end))
                    .with_message(&label_message)
                    .with_color(Color::Red),
            );

        // Add secondary labels for duplicate definitions
        match self {
            TypeCheckError::DuplicateField { first_span, .. }
            | TypeCheckError::DuplicateNodeId { first_span, .. } => {
                builder = builder.with_label(
                    Label::new((filename, first_span.start..first_span.end))
                        .with_message("first defined here")
                        .with_color(Color::Blue),
                );
            }
            _ => {}
        }

        builder = builder.with_note(self.help_message());

        builder
            .finish()
            .write((filename, Source::from(source)), &mut output)
            .unwrap();

        String::from_utf8(output).unwrap()
    }

    fn error_title(&self) -> &'static str {
        match self {
            TypeCheckError::DuplicateField { .. } => "Duplicate Definition",
            TypeCheckError::DuplicateNodeId { .. } => "Duplicate Node ID",
            TypeCheckError::UndefinedReference { .. } => "Undefined Reference",
            TypeCheckError::InvalidFieldAccess { .. } => "Invalid Field Access",
            TypeCheckError::UndefinedNode { .. } => "Undefined Node",
            TypeCheckError::TypeMismatch { .. } => "Type Mismatch",
            TypeCheckError::InvalidOperandType { .. } => "Invalid Operand Type",
            TypeCheckError::NonBoolConstraint { .. } => "Invalid Constraint",
            TypeCheckError::NonBoolVerify { .. } => "Invalid Verify Expression",
            TypeCheckError::InvalidEdgeSource { .. } => "Invalid Edge Source",
            TypeCheckError::InvalidEdgeTarget { .. } => "Invalid Edge Target",
            TypeCheckError::NonBoolEdgeCondition { .. } => "Invalid Edge Condition",
            TypeCheckError::UndefinedGoalField { .. } => "Undefined Goal Field",
            TypeCheckError::Custom { .. } => "Type Error",
        }
    }

    fn help_message(&self) -> String {
        match self {
            TypeCheckError::DuplicateField { name, .. } => {
                format!(
                    "Field `{}` is already defined. Choose a different name.",
                    name
                )
            }
            TypeCheckError::DuplicateNodeId { id, .. } => {
                format!(
                    "Node `{}` is already defined. Choose a different ID.",
                    id
                )
            }
            TypeCheckError::UndefinedReference { suggestions, .. } if !suggestions.is_empty() => {
                format!("Did you mean: {}?", suggestions.join(", "))
            }
            TypeCheckError::UndefinedReference { .. } => {
                "Check the spelling and make sure it's defined".to_string()
            }
            TypeCheckError::InvalidFieldAccess {
                available_fields, ..
            } if !available_fields.is_empty() => {
                format!("Available fields: {}", available_fields.join(", "))
            }
            TypeCheckError::InvalidFieldAccess { .. } => {
                "Check that the field exists on this type".to_string()
            }
            TypeCheckError::UndefinedNode {
                available_nodes, ..
            } if !available_nodes.is_empty() => {
                format!("Available nodes: {}", available_nodes.join(", "))
            }
            TypeCheckError::UndefinedNode { .. } => {
                "Check that the node is defined in the FLOW block".to_string()
            }
            TypeCheckError::TypeMismatch { context: Some(ctx), .. } => {
                format!("In {}: ensure types match", ctx)
            }
            TypeCheckError::TypeMismatch { .. } => {
                "Ensure the types are compatible".to_string()
            }
            TypeCheckError::InvalidOperandType { operator, expected, .. } => {
                format!(
                    "Operator `{}` requires {} operands",
                    operator, expected
                )
            }
            TypeCheckError::NonBoolConstraint { .. } | TypeCheckError::NonBoolVerify { .. } => {
                "Constraint and verify expressions must evaluate to BOOL".to_string()
            }
            TypeCheckError::NonBoolEdgeCondition { .. } => {
                "Edge conditions (WHEN clause) must be BOOL".to_string()
            }
            TypeCheckError::InvalidEdgeSource { .. } | TypeCheckError::InvalidEdgeTarget { .. } => {
                "Edge endpoints must reference INPUT, OUTPUT, or a FLOW node".to_string()
            }
            TypeCheckError::UndefinedGoalField { kind, .. } => {
                format!("Check that the {} field is defined in the {} block", kind, kind.to_uppercase())
            }
            TypeCheckError::Custom { .. } => "Check the type annotations and try again".to_string(),
        }
    }
}

/// Result type alias for type check operations
pub type TypeCheckResult<T> = Result<T, Vec<TypeCheckError>>;
