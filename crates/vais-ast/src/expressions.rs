//! Expression AST nodes: Expr, IfElse, MatchArm, StringInterpPart

use crate::infrastructure::Spanned;
use crate::ast_types::Type;
use crate::operators::{BinOp, UnaryOp};
use crate::patterns::{CaptureMode, Pattern};
use crate::function::Param;
use crate::macros::MacroInvoke;
use crate::statements::Stmt;

/// Part of a string interpolation expression.
#[derive(Debug, Clone, PartialEq)]
pub enum StringInterpPart {
    /// Literal text segment
    Lit(String),
    /// Interpolated expression: `{expr}`
    Expr(Box<Spanned<Expr>>),
}

/// Expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal
    Int(i64),
    /// Float literal
    Float(f64),
    /// Boolean literal
    Bool(bool),
    /// String literal
    String(String),
    /// String interpolation: `"hello {name}, {x + 1}"`
    StringInterp(Vec<StringInterpPart>),
    /// Unit literal: `()`
    Unit,
    /// Identifier
    Ident(String),
    /// Self-recursion: `@`
    SelfCall,
    /// Binary operation
    Binary {
        op: BinOp,
        left: Box<Spanned<Expr>>,
        right: Box<Spanned<Expr>>,
    },
    /// Unary operation
    Unary {
        op: UnaryOp,
        expr: Box<Spanned<Expr>>,
    },
    /// Ternary: `cond ? then : else`
    Ternary {
        cond: Box<Spanned<Expr>>,
        then: Box<Spanned<Expr>>,
        else_: Box<Spanned<Expr>>,
    },
    /// If expression: `I cond{...}E{...}`
    If {
        cond: Box<Spanned<Expr>>,
        then: Vec<Spanned<Stmt>>,
        else_: Option<IfElse>,
    },
    /// Loop: `L pattern:iter{...}`
    Loop {
        pattern: Option<Spanned<Pattern>>,
        iter: Option<Box<Spanned<Expr>>>,
        body: Vec<Spanned<Stmt>>,
    },
    /// While loop: `L condition{...}`
    While {
        condition: Box<Spanned<Expr>>,
        body: Vec<Spanned<Stmt>>,
    },
    /// Match: `M expr{arms}`
    Match {
        expr: Box<Spanned<Expr>>,
        arms: Vec<MatchArm>,
    },
    /// Function call: `f(args)`
    Call {
        func: Box<Spanned<Expr>>,
        args: Vec<Spanned<Expr>>,
    },
    /// Method call: `obj.method(args)`
    MethodCall {
        receiver: Box<Spanned<Expr>>,
        method: Spanned<String>,
        args: Vec<Spanned<Expr>>,
    },
    /// Static method call: `Type.method(args)`
    StaticMethodCall {
        type_name: Spanned<String>,
        method: Spanned<String>,
        args: Vec<Spanned<Expr>>,
    },
    /// Field access: `obj.field`
    Field {
        expr: Box<Spanned<Expr>>,
        field: Spanned<String>,
    },
    /// Index: `arr[idx]`
    Index {
        expr: Box<Spanned<Expr>>,
        index: Box<Spanned<Expr>>,
    },
    /// Array literal: `[a, b, c]`
    Array(Vec<Spanned<Expr>>),
    /// Tuple literal: `(a, b, c)`
    Tuple(Vec<Spanned<Expr>>),
    /// Struct literal: `Point{x:1,y:2}`
    StructLit {
        name: Spanned<String>,
        fields: Vec<(Spanned<String>, Spanned<Expr>)>,
    },
    /// Range: `start..end`
    Range {
        start: Option<Box<Spanned<Expr>>>,
        end: Option<Box<Spanned<Expr>>>,
        inclusive: bool,
    },
    /// Block expression: `{stmts}`
    Block(Vec<Spanned<Stmt>>),
    /// Await: `expr.await`
    Await(Box<Spanned<Expr>>),
    /// Try: `expr?`
    Try(Box<Spanned<Expr>>),
    /// Unwrap: `expr!`
    Unwrap(Box<Spanned<Expr>>),
    /// Map literal: `{k1: v1, k2: v2}`
    MapLit(Vec<(Spanned<Expr>, Spanned<Expr>)>),
    /// Spread: `..expr` in array literals
    Spread(Box<Spanned<Expr>>),
    /// Reference: `&expr`
    Ref(Box<Spanned<Expr>>),
    /// Dereference: `*expr`
    Deref(Box<Spanned<Expr>>),
    /// Type cast: `expr as Type`
    Cast {
        expr: Box<Spanned<Expr>>,
        ty: Spanned<Type>,
    },
    /// Assignment: `lhs = rhs`
    Assign {
        target: Box<Spanned<Expr>>,
        value: Box<Spanned<Expr>>,
    },
    /// Compound assignment: `lhs += rhs`
    AssignOp {
        op: BinOp,
        target: Box<Spanned<Expr>>,
        value: Box<Spanned<Expr>>,
    },
    /// Lambda: `|params| expr` or `|params| {body}`
    Lambda {
        params: Vec<Param>,
        body: Box<Spanned<Expr>>,
        /// Captured variables from enclosing scope (filled during type checking)
        captures: Vec<String>,
        /// Capture mode for closure (by-value, move, by-ref, by-mut-ref)
        capture_mode: CaptureMode,
    },
    /// Spawn: `spawn{expr}`
    Spawn(Box<Spanned<Expr>>),
    /// Yield: `Y expr` - Yield a value from a generator function
    Yield(Box<Spanned<Expr>>),
    /// Comptime: `comptime { expr }` - Evaluated at compile time
    Comptime { body: Box<Spanned<Expr>> },
    /// Macro invocation: `name!(args)`
    MacroInvoke(MacroInvoke),
    /// Old: `old(expr)` - Reference to pre-state value in ensures clause
    /// Captures the value of an expression before function execution.
    Old(Box<Spanned<Expr>>),
    /// Assert: `assert(expr)` or `assert(expr, msg)` - Runtime assertion
    /// Checks condition at runtime, panics with message if false.
    Assert {
        condition: Box<Spanned<Expr>>,
        message: Option<Box<Spanned<Expr>>>,
    },
    /// Assume: `assume(expr)` - Compiler assumption for verification
    /// Tells the verifier to assume this condition is true.
    Assume(Box<Spanned<Expr>>),
    /// Error recovery node - represents an expression that failed to parse
    /// Used for continuing parsing after errors to report multiple errors at once.
    Error {
        /// Error message describing what went wrong
        message: String,
        /// Tokens that were skipped during recovery
        skipped_tokens: Vec<String>,
    },
    /// Lazy expression: `lazy expr` - Deferred evaluation
    /// The expression is not evaluated until explicitly forced.
    /// Creates a Lazy<T> thunk that memoizes the result.
    Lazy(Box<Spanned<Expr>>),
    /// Force expression: `force expr` - Force evaluation of lazy value
    /// Evaluates a lazy thunk and returns the cached result.
    Force(Box<Spanned<Expr>>),
}

/// If-else branch
#[derive(Debug, Clone, PartialEq)]
pub enum IfElse {
    /// `E cond{...}` - else if
    ElseIf(Box<Spanned<Expr>>, Vec<Spanned<Stmt>>, Option<Box<IfElse>>),
    /// `E{...}` - else
    Else(Vec<Spanned<Stmt>>),
}

/// Match arm
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Spanned<Pattern>,
    pub guard: Option<Box<Spanned<Expr>>>,
    pub body: Box<Spanned<Expr>>,
}
