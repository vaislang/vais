//! Constant definitions: ConstDef, GlobalDef, ConstExpr, ConstBinOp

use crate::infrastructure::{Attribute, Spanned};
use crate::ast_types::Type;
use crate::expressions::Expr;

/// Constant definition: `C NAME: Type = value`
///
/// Constants are compile-time evaluated and inlined.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstDef {
    /// Constant name (typically SCREAMING_SNAKE_CASE)
    pub name: Spanned<String>,
    /// Constant type
    pub ty: Spanned<Type>,
    /// Constant value (must be compile-time evaluable)
    pub value: Spanned<Expr>,
    /// Whether the constant is public
    pub is_pub: bool,
    /// Attributes (e.g., `#[cfg(target_os = "linux")]` for conditional compilation)
    pub attributes: Vec<Attribute>,
}

/// Global variable definition: `G name: Type = value`
///
/// Global variables have static storage duration.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDef {
    /// Variable name
    pub name: Spanned<String>,
    /// Variable type
    pub ty: Spanned<Type>,
    /// Initial value
    pub value: Spanned<Expr>,
    /// Whether the global is public
    pub is_pub: bool,
    /// Whether the global is mutable (default true for globals)
    pub is_mutable: bool,
}

/// Const expression for const generic parameters
#[derive(Debug, Clone, PartialEq)]
pub enum ConstExpr {
    /// Literal integer: 10, 32
    Literal(i64),
    /// Const parameter reference: N
    Param(String),
    /// Binary operation: N + 1, A * B
    BinOp {
        op: ConstBinOp,
        left: Box<ConstExpr>,
        right: Box<ConstExpr>,
    },
    /// Unary negation: -N
    Negate(Box<ConstExpr>),
}

impl std::fmt::Display for ConstExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstExpr::Literal(n) => write!(f, "{}", n),
            ConstExpr::Param(name) => write!(f, "{}", name),
            ConstExpr::BinOp { op, left, right } => write!(f, "({} {} {})", left, op, right),
            ConstExpr::Negate(inner) => write!(f, "(-{})", inner),
        }
    }
}

/// Binary operators for const expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

impl std::fmt::Display for ConstBinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstBinOp::Add => write!(f, "+"),
            ConstBinOp::Sub => write!(f, "-"),
            ConstBinOp::Mul => write!(f, "*"),
            ConstBinOp::Div => write!(f, "/"),
            ConstBinOp::Mod => write!(f, "%"),
            ConstBinOp::BitAnd => write!(f, "&"),
            ConstBinOp::BitOr => write!(f, "|"),
            ConstBinOp::BitXor => write!(f, "^"),
            ConstBinOp::Shl => write!(f, "<<"),
            ConstBinOp::Shr => write!(f, ">>"),
        }
    }
}
