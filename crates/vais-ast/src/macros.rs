//! Macro system types: MacroDef, MacroRule, MacroPattern, MacroTemplate, MacroInvoke, etc.

use crate::infrastructure::Spanned;

/// Macro definition: `macro name! { rules }`
///
/// Declarative macros use pattern matching to transform code.
/// Each rule has a pattern (matcher) and a template (transcriber).
///
/// # Example
/// ```vais
/// macro vec! {
///     () => { Vec::new() }
///     ($($item:expr),*) => { Vec::from([$($item),*]) }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MacroDef {
    pub name: Spanned<String>,
    pub rules: Vec<MacroRule>,
    pub is_pub: bool,
}

/// A single macro rule: `pattern => template`
#[derive(Debug, Clone, PartialEq)]
pub struct MacroRule {
    /// The pattern to match against
    pub pattern: MacroPattern,
    /// The template to expand into
    pub template: MacroTemplate,
}

/// Macro pattern for matching input tokens
#[derive(Debug, Clone, PartialEq)]
pub enum MacroPattern {
    /// Empty pattern: `()`
    Empty,
    /// Sequence of pattern elements: `($x:expr, $y:expr)`
    Sequence(Vec<MacroPatternElement>),
}

/// Element in a macro pattern
#[derive(Debug, Clone, PartialEq)]
pub enum MacroPatternElement {
    /// Literal token: `+`, `let`, etc.
    Token(MacroToken),
    /// Metavariable: `$x:expr`
    MetaVar { name: String, kind: MetaVarKind },
    /// Repetition: `$($x:expr),*` or `$($x:expr),+`
    Repetition {
        patterns: Vec<MacroPatternElement>,
        separator: Option<MacroToken>,
        kind: RepetitionKind,
    },
    /// Nested group: `($pattern)`
    Group {
        delimiter: Delimiter,
        content: Vec<MacroPatternElement>,
    },
}

/// Metavariable kinds (fragment specifiers)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaVarKind {
    /// Expression: `$x:expr`
    Expr,
    /// Type: `$t:ty`
    Ty,
    /// Identifier: `$i:ident`
    Ident,
    /// Pattern: `$p:pat`
    Pat,
    /// Statement: `$s:stmt`
    Stmt,
    /// Block: `$b:block`
    Block,
    /// Item: `$i:item`
    Item,
    /// Literal: `$l:lit`
    Lit,
    /// Token tree: `$t:tt`
    Tt,
}

impl std::str::FromStr for MetaVarKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "expr" => Ok(MetaVarKind::Expr),
            "ty" => Ok(MetaVarKind::Ty),
            "ident" => Ok(MetaVarKind::Ident),
            "pat" => Ok(MetaVarKind::Pat),
            "stmt" => Ok(MetaVarKind::Stmt),
            "block" => Ok(MetaVarKind::Block),
            "item" => Ok(MetaVarKind::Item),
            "lit" => Ok(MetaVarKind::Lit),
            "tt" => Ok(MetaVarKind::Tt),
            _ => Err(()),
        }
    }
}

/// Repetition kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepetitionKind {
    /// Zero or more: `*`
    ZeroOrMore,
    /// One or more: `+`
    OneOrMore,
    /// Zero or one: `?`
    ZeroOrOne,
}

/// Delimiter type for macro groups
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Paren,   // ()
    Bracket, // []
    Brace,   // {}
}

/// Token representation for macros
#[derive(Debug, Clone, PartialEq)]
pub enum MacroToken {
    Ident(String),
    Punct(char),
    Literal(MacroLiteral),
    Group(Delimiter, Vec<MacroToken>),
}

/// Literal in macro token stream
#[derive(Debug, Clone, PartialEq)]
pub enum MacroLiteral {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

/// Macro template for code generation
#[derive(Debug, Clone, PartialEq)]
pub enum MacroTemplate {
    /// Empty template
    Empty,
    /// Sequence of template elements
    Sequence(Vec<MacroTemplateElement>),
}

/// Element in a macro template
#[derive(Debug, Clone, PartialEq)]
pub enum MacroTemplateElement {
    /// Literal token
    Token(MacroToken),
    /// Metavariable substitution: `$x`
    MetaVar(String),
    /// Repetition: `$($x),*`
    Repetition {
        elements: Vec<MacroTemplateElement>,
        separator: Option<MacroToken>,
        kind: RepetitionKind,
    },
    /// Nested group
    Group {
        delimiter: Delimiter,
        content: Vec<MacroTemplateElement>,
    },
}

/// Macro invocation: `name!(args)`
///
/// This is used during parsing before macro expansion.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroInvoke {
    pub name: Spanned<String>,
    pub delimiter: Delimiter,
    pub tokens: Vec<MacroToken>,
}
