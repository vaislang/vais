//! Expression definitions

use aoel_lexer::Span;
use serde::{Deserialize, Serialize};
use crate::{AstNode, Ident, ExternalRef};

/// All expression types in AOEL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Literal value
    Literal(Literal),

    /// Identifier
    Ident(Ident),

    /// External reference (@path)
    ExternalRef(ExternalRef),

    /// Field access (base.field)
    FieldAccess(Box<FieldAccess>),

    /// Binary operation (left op right)
    Binary(Box<BinaryExpr>),

    /// Unary operation (op operand)
    Unary(Box<UnaryExpr>),

    /// Function call (name(args...))
    Call(Box<CallExpr>),

    /// Array access (base[index])
    Index(Box<IndexExpr>),

    /// Grouped expression ((expr))
    Grouped(Box<GroupedExpr>),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(e) => e.span,
            Expr::Ident(e) => e.span,
            Expr::ExternalRef(e) => e.span,
            Expr::FieldAccess(e) => e.span,
            Expr::Binary(e) => e.span,
            Expr::Unary(e) => e.span,
            Expr::Call(e) => e.span,
            Expr::Index(e) => e.span,
            Expr::Grouped(e) => e.span,
        }
    }
}

/// Literal value kinds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralKind {
    /// Integer literal
    Integer(i64),

    /// Float literal
    Float(f64),

    /// String literal
    String(String),

    /// Boolean literal
    Bool(bool),

    /// Regex literal
    Regex(String),

    /// Duration literal (e.g., "10s", "5m")
    Duration(String),

    /// Size literal (e.g., "256MB")
    Size(String),

    /// Void/null literal
    Void,
}

/// Literal expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Literal {
    pub kind: LiteralKind,
    pub span: Span,
}

impl Literal {
    pub fn new(kind: LiteralKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn integer(value: i64, span: Span) -> Self {
        Self::new(LiteralKind::Integer(value), span)
    }

    pub fn float(value: f64, span: Span) -> Self {
        Self::new(LiteralKind::Float(value), span)
    }

    pub fn string(value: impl Into<String>, span: Span) -> Self {
        Self::new(LiteralKind::String(value.into()), span)
    }

    pub fn bool(value: bool, span: Span) -> Self {
        Self::new(LiteralKind::Bool(value), span)
    }

    pub fn void(span: Span) -> Self {
        Self::new(LiteralKind::Void, span)
    }
}

impl AstNode for Literal {
    fn span(&self) -> Span {
        self.span
    }
}

/// Field access expression (base.field)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldAccess {
    pub base: Expr,
    pub field: Ident,
    pub span: Span,
}

impl FieldAccess {
    pub fn new(base: Expr, field: Ident, span: Span) -> Self {
        Self { base, field, span }
    }
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,

    // Comparison
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,

    // Logical
    And,
    Or,
    Xor,
    Implies,

    // Other
    In,
    Match,
}

impl BinaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Eq => "==",
            BinaryOp::Neq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Lte => "<=",
            BinaryOp::Gte => ">=",
            BinaryOp::And => "AND",
            BinaryOp::Or => "OR",
            BinaryOp::Xor => "XOR",
            BinaryOp::Implies => "IMPLIES",
            BinaryOp::In => "IN",
            BinaryOp::Match => "MATCH",
        }
    }

    /// Get operator precedence (higher = binds tighter)
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,
            BinaryOp::Xor => 2,
            BinaryOp::And => 3,
            BinaryOp::Implies => 4,
            BinaryOp::Eq | BinaryOp::Neq => 5,
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Lte | BinaryOp::Gte => 6,
            BinaryOp::In | BinaryOp::Match => 7,
            BinaryOp::Add | BinaryOp::Sub => 8,
            BinaryOp::Mul | BinaryOp::Div => 9,
        }
    }
}

/// Binary expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryExpr {
    pub left: Expr,
    pub op: BinaryOp,
    pub right: Expr,
    pub span: Span,
}

impl BinaryExpr {
    pub fn new(left: Expr, op: BinaryOp, right: Expr, span: Span) -> Self {
        Self {
            left,
            op,
            right,
            span,
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Neg,
}

impl UnaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOp::Not => "NOT",
            UnaryOp::Neg => "-",
        }
    }
}

/// Unary expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: Expr,
    pub span: Span,
}

impl UnaryExpr {
    pub fn new(op: UnaryOp, operand: Expr, span: Span) -> Self {
        Self { op, operand, span }
    }
}

/// Function call expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallExpr {
    pub name: Ident,
    pub args: Vec<Expr>,
    pub span: Span,
}

impl CallExpr {
    pub fn new(name: Ident, args: Vec<Expr>, span: Span) -> Self {
        Self { name, args, span }
    }
}

/// Index/array access expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexExpr {
    pub base: Expr,
    pub index: Expr,
    pub span: Span,
}

impl IndexExpr {
    pub fn new(base: Expr, index: Expr, span: Span) -> Self {
        Self { base, index, span }
    }
}

/// Grouped (parenthesized) expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupedExpr {
    pub inner: Expr,
    pub span: Span,
}

impl GroupedExpr {
    pub fn new(inner: Expr, span: Span) -> Self {
        Self { inner, span }
    }
}
