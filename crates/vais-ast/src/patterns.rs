//! Patterns for destructuring, capture modes, and literals

use crate::infrastructure::Spanned;

/// Patterns for destructuring
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Wildcard: `_`
    Wildcard,
    /// Identifier binding: `x`
    Ident(String),
    /// Literal pattern
    Literal(Literal),
    /// Tuple pattern: `(a, b)`
    Tuple(Vec<Spanned<Pattern>>),
    /// Struct pattern: `Point{x, y}`
    Struct {
        name: Spanned<String>,
        fields: Vec<(Spanned<String>, Option<Spanned<Pattern>>)>,
    },
    /// Enum variant pattern: `Some(x)`
    Variant {
        name: Spanned<String>,
        fields: Vec<Spanned<Pattern>>,
    },
    /// Range pattern: `1..10`
    Range {
        start: Option<Box<Spanned<Pattern>>>,
        end: Option<Box<Spanned<Pattern>>>,
        inclusive: bool,
    },
    /// Or pattern: `a | b`
    Or(Vec<Spanned<Pattern>>),
    /// Pattern alias: `x @ Pattern` - binds the matched value to x while also matching against Pattern
    Alias {
        name: String,
        pattern: Box<Spanned<Pattern>>,
    },
}

/// Capture mode for closure variables
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureMode {
    /// Default: inferred by usage (by-value/move)
    ByValue,
    /// Explicit move: `move |x| ...`
    Move,
    /// By reference: `|&x| ...`
    ByRef,
    /// By mutable reference: `|&mut x| ...`
    ByMutRef,
}

/// Literal values for patterns
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}
