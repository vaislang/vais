//! Function definition, parameters, and related types

use crate::infrastructure::{Attribute, Spanned};
use crate::ast_types::Type;
use crate::expressions::Expr;
use crate::statements::Stmt;
use crate::generics::{GenericParam, WherePredicate};

/// Function definition with signature and body.
///
/// Represents both expression-form (`F f(x)->i64=x+1`) and
/// block-form (`F f(x)->i64{R x+1}`) functions.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    /// Function name
    pub name: Spanned<String>,
    /// Generic type parameters
    pub generics: Vec<GenericParam>,
    /// Function parameters
    pub params: Vec<Param>,
    /// Return type (optional, can be inferred)
    pub ret_type: Option<Spanned<Type>>,
    /// Function body (expression or block)
    pub body: FunctionBody,
    /// Whether this function is public
    pub is_pub: bool,
    /// Whether this is an async function
    pub is_async: bool,
    /// Attributes like `#[cfg(test)]`, `#\[inline\]`, etc.
    pub attributes: Vec<Attribute>,
    /// Where clause predicates (e.g., `where T: Display + Clone`)
    pub where_clause: Vec<WherePredicate>,
}

/// Function body - either expression or block
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    /// `=expr`
    Expr(Box<Spanned<Expr>>),
    /// `{stmts}`
    Block(Vec<Spanned<Stmt>>),
}

/// Ownership mode for linear types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Ownership {
    /// Regular ownership (can be copied freely)
    #[default]
    Regular,
    /// Linear ownership (must be used exactly once)
    Linear,
    /// Affine ownership (can be used at most once)
    Affine,
    /// Move ownership (transferred on use)
    Move,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Spanned<String>,
    pub ty: Spanned<Type>,
    pub is_mut: bool,
    pub is_vararg: bool,      // true for variadic parameters (...)
    pub ownership: Ownership, // Linear type ownership mode
    pub default_value: Option<Box<Spanned<Expr>>>, // Default parameter value
}

/// Named argument in a function call: `name: expr`
#[derive(Debug, Clone, PartialEq)]
pub struct NamedArg {
    pub name: Spanned<String>,
    pub value: Spanned<Expr>,
}

/// Call arguments - either all positional or with named arguments
#[derive(Debug, Clone, PartialEq)]
pub enum CallArgs {
    /// Positional arguments only: `f(a, b, c)`
    Positional(Vec<Spanned<Expr>>),
    /// Named arguments (may include positional at the start): `f(a, name: b, other: c)`
    Named {
        positional: Vec<Spanned<Expr>>,
        named: Vec<NamedArg>,
    },
}
