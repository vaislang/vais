//! Statement AST node

use crate::ast_types::Type;
use crate::expressions::Expr;
use crate::function::Ownership;
use crate::infrastructure::Spanned;
use crate::patterns::Pattern;

/// Statements
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Variable declaration: `x := expr` or `x: T = expr`
    Let {
        name: Spanned<String>,
        ty: Option<Spanned<Type>>,
        value: Box<Spanned<Expr>>,
        is_mut: bool,
        ownership: Ownership, // Linear type ownership mode
    },
    /// Tuple destructuring: `(a, b) := expr`
    LetDestructure {
        pattern: Spanned<Pattern>,
        value: Box<Spanned<Expr>>,
        is_mut: bool,
    },
    /// Expression statement
    Expr(Box<Spanned<Expr>>),
    /// Return: `R expr` or implicit last expression
    Return(Option<Box<Spanned<Expr>>>),
    /// Break: `B` or `B expr`
    Break(Option<Box<Spanned<Expr>>>),
    /// Continue: `C`
    Continue,
    /// Defer: `D expr` - Execute expr when scope exits (LIFO order)
    Defer(Box<Spanned<Expr>>),
    /// Error recovery node - represents a statement that failed to parse
    /// Used for continuing parsing after errors to report multiple errors at once.
    Error {
        /// Error message describing what went wrong
        message: String,
        /// Tokens that were skipped during recovery
        skipped_tokens: Vec<String>,
    },
}
