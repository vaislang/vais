//! Vais 2.0 Abstract Syntax Tree
//!
//! AI-optimized AST with minimal node types for efficient parsing and code generation.

/// Source location for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// AST node with span information
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

/// Top-level module
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub items: Vec<Spanned<Item>>,
}

/// Top-level items
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// `F name(params)->ret=expr` or `F name(params)->ret{...}`
    Function(Function),
    /// `S Name{fields}`
    Struct(Struct),
    /// `E Name{variants}`
    Enum(Enum),
    /// `T Name=Type`
    TypeAlias(TypeAlias),
    /// `U module` or `U module::{items}`
    Use(Use),
}

/// Function definition
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Spanned<String>,
    pub generics: Vec<Spanned<String>>,
    pub params: Vec<Param>,
    pub ret_type: Option<Spanned<Type>>,
    pub body: FunctionBody,
    pub is_pub: bool,
    pub is_async: bool,
}

/// Function body - either expression or block
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    /// `=expr`
    Expr(Box<Spanned<Expr>>),
    /// `{stmts}`
    Block(Vec<Spanned<Stmt>>),
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Spanned<String>,
    pub ty: Spanned<Type>,
    pub is_mut: bool,
}

/// Struct definition
#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: Spanned<String>,
    pub generics: Vec<Spanned<String>>,
    pub fields: Vec<Field>,
    pub methods: Vec<Spanned<Function>>,
    pub is_pub: bool,
}

/// Struct field
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: Spanned<String>,
    pub ty: Spanned<Type>,
    pub is_pub: bool,
}

/// Enum definition
#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: Spanned<String>,
    pub generics: Vec<Spanned<String>>,
    pub variants: Vec<Variant>,
    pub is_pub: bool,
}

/// Enum variant
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: Spanned<String>,
    pub fields: VariantFields,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariantFields {
    Unit,
    Tuple(Vec<Spanned<Type>>),
    Struct(Vec<Field>),
}

/// Type alias
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub name: Spanned<String>,
    pub generics: Vec<Spanned<String>>,
    pub ty: Spanned<Type>,
    pub is_pub: bool,
}

/// Use/Import statement
#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub path: Vec<Spanned<String>>,
    pub alias: Option<Spanned<String>>,
}

/// Type expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Named type: `i64`, `String`, `Vec<T>`
    Named {
        name: String,
        generics: Vec<Spanned<Type>>,
    },
    /// Array: `[T]`
    Array(Box<Spanned<Type>>),
    /// Map: `[K:V]`
    Map(Box<Spanned<Type>>, Box<Spanned<Type>>),
    /// Tuple: `(T1, T2, ...)`
    Tuple(Vec<Spanned<Type>>),
    /// Optional: `T?`
    Optional(Box<Spanned<Type>>),
    /// Result: `T!`
    Result(Box<Spanned<Type>>),
    /// Pointer: `*T`
    Pointer(Box<Spanned<Type>>),
    /// Reference: `&T`
    Ref(Box<Spanned<Type>>),
    /// Mutable reference: `&mut T`
    RefMut(Box<Spanned<Type>>),
    /// Function type: `(A,B)->C`
    Fn {
        params: Vec<Spanned<Type>>,
        ret: Box<Spanned<Type>>,
    },
    /// Unit type: `()`
    Unit,
    /// Inferred type (for internal use)
    Infer,
}

/// Statements
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Variable declaration: `x := expr` or `x: T = expr`
    Let {
        name: Spanned<String>,
        ty: Option<Spanned<Type>>,
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
    /// Reference: `&expr`
    Ref(Box<Spanned<Expr>>),
    /// Dereference: `*expr`
    Deref(Box<Spanned<Expr>>),
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
    },
    /// Spawn: `spawn{expr}`
    Spawn(Box<Spanned<Expr>>),
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
}

/// Literal values for patterns
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Neq,
    // Logical
    And,
    Or,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

impl BinOp {
    pub fn precedence(self) -> u8 {
        match self {
            BinOp::Or => 1,
            BinOp::And => 2,
            BinOp::BitOr => 3,
            BinOp::BitXor => 4,
            BinOp::BitAnd => 5,
            BinOp::Eq | BinOp::Neq => 6,
            BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte => 7,
            BinOp::Shl | BinOp::Shr => 8,
            BinOp::Add | BinOp::Sub => 9,
            BinOp::Mul | BinOp::Div | BinOp::Mod => 10,
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,    // -
    Not,    // !
    BitNot, // ~
}
