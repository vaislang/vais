//! Function definition, parameters, and related types

use crate::ast_types::Type;
use crate::expressions::Expr;
use crate::generics::{GenericParam, WherePredicate};
use crate::infrastructure::{Attribute, Spanned};
use crate::statements::Stmt;

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
    /// Whether this function is marked `partial`.
    ///
    /// A `partial` function is allowed to panic at runtime — i.e. it may
    /// contain expressions that trigger `Effect::Panic` (division by zero,
    /// array / slice out-of-bounds, `Option::None` unwrap, `Result::Err`
    /// unwrap, explicit `panic!`, `assert`, `abort`, `exit`). Functions
    /// without the `partial` modifier are "total" and the type checker
    /// must prove they are panic-free before compilation succeeds.
    ///
    /// This is the Phase 4c.2 (Task #53) totality gate — see
    /// `crates/vais-types/src/checker_fn.rs` for the enforcement site and
    /// `crates/vais-types/src/effects.rs` for the inferrer that classifies
    /// which expressions raise `Effect::Panic`.
    pub is_partial: bool,
    /// Declared effect prefix, if any.
    ///
    /// `pure F foo() { ... }`, `io F foo() { ... }`, `alloc F foo() { ... }`
    /// attach an explicit effect annotation that the type checker verifies
    /// via `EffectInferrer::check_effects` in `vais-types/src/effects.rs`.
    ///
    /// Subtype rules (pure ⊆ io ⊆ alloc+io):
    /// - `pure` functions may only call `pure` callees (no IO, no Alloc).
    /// - `io` functions may call `pure` and `io`; not `alloc`.
    /// - `alloc` functions may call `pure` and `alloc`; not `io`.
    ///
    /// Functions without a declared effect remain **inferred** — the
    /// checker only enforces the subtype rule when an explicit annotation
    /// is present, preserving baseline compatibility with the existing
    /// E2E test suite.
    ///
    /// Phase 4c.3 (Task #54) — see `vais-types/src/effects.rs::get_declared_effects`.
    pub declared_effect: Option<EffectPrefix>,
    /// Attributes like `#[cfg(test)]`, `#\[inline\]`, etc.
    pub attributes: Vec<Attribute>,
    /// Where clause predicates (e.g., `where T: Display + Clone`)
    pub where_clause: Vec<WherePredicate>,
}

/// Effect prefix keyword on a function declaration.
///
/// Emitted by the parser whenever a function item begins with `pure`,
/// `io`, or `alloc`. The type checker uses this to drive subtype
/// verification in `EffectInferrer::check_effects`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectPrefix {
    /// `pure F foo()` — no IO, no allocation, no panic-effect calls.
    Pure,
    /// `io F foo()` — reads/writes external world; may call `pure`.
    Io,
    /// `alloc F foo()` — allocates; may call `pure`; may not call `io`.
    Alloc,
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
