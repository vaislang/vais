//! Vais 0.0.1 Abstract Syntax Tree
//!
//! AI-optimized AST with minimal node types for efficient parsing and code generation.

/// Source location information for error reporting and diagnostics.
///
/// Spans track the start and end positions of AST nodes in the source code,
/// enabling precise error messages and IDE features like go-to-definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Byte offset of the start of this span in the source file
    pub start: usize,
    /// Byte offset of the end of this span in the source file
    pub end: usize,
}

impl Span {
    /// Creates a new span from start and end positions.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Merges two spans into a single span covering both ranges.
    ///
    /// The resulting span starts at the minimum start position and
    /// ends at the maximum end position of the two input spans.
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// AST node wrapper that includes source location information.
///
/// This generic wrapper associates any AST node type with its source span,
/// enabling error reporting and tooling features throughout the compiler pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    /// The actual AST node
    pub node: T,
    /// Source location of this node
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Creates a new spanned node with the given value and location.
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

/// Attribute for conditional compilation and metadata annotations.
///
/// Attributes like `#[cfg(test)]`, `#\[inline\]`, etc. provide metadata
/// and control compilation behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    /// Attribute name (e.g., "cfg", "inline")
    pub name: String,
    /// Attribute arguments (e.g., cfg(test) -> ["test"])
    pub args: Vec<String>,
}

/// Top-level module containing all program items.
///
/// A module represents a complete Vais source file and contains
/// all top-level definitions (functions, structs, enums, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    /// List of top-level items in this module
    pub items: Vec<Spanned<Item>>,
}

/// Top-level item definitions in a module.
///
/// Represents the various kinds of declarations that can appear at module level.
/// Vais uses single-letter keywords for token efficiency.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// Function definition: `F name(params)->ret=expr` or `F name(params)->ret{...}`
    Function(Function),
    /// Struct definition: `S Name{fields}`
    Struct(Struct),
    /// Enum definition: `E Name{variants}`
    Enum(Enum),
    /// Union definition: `O Name{fields}` (untagged, C-style)
    Union(Union),
    /// Type alias: `T Name=Type`
    TypeAlias(TypeAlias),
    /// Import statement: `U module` or `U module::{items}`
    Use(Use),
    /// Trait definition: `W Name { methods }` (W = "What" interface)
    Trait(Trait),
    /// Implementation block: `X Type: Trait { methods }` (X = "eXtend")
    Impl(Impl),
}

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

/// Generic parameter kind - either a type parameter or a const parameter
#[derive(Debug, Clone, PartialEq)]
pub enum GenericParamKind {
    /// Type parameter with optional trait bounds (e.g., T, T: Display + Clone)
    Type {
        bounds: Vec<Spanned<String>>,
    },
    /// Const parameter with a type (e.g., const N: u64)
    Const {
        ty: Spanned<Type>,
    },
}

/// Generic parameter with optional trait bounds
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub name: Spanned<String>,
    pub bounds: Vec<Spanned<String>>, // Trait constraints (e.g., T: Display + Clone) - kept for backward compatibility
    pub kind: GenericParamKind,
}

impl GenericParam {
    /// Create a type generic parameter (backward compatible constructor)
    pub fn new_type(name: Spanned<String>, bounds: Vec<Spanned<String>>) -> Self {
        Self {
            name,
            bounds: bounds.clone(),
            kind: GenericParamKind::Type { bounds },
        }
    }

    /// Create a const generic parameter
    pub fn new_const(name: Spanned<String>, ty: Spanned<Type>) -> Self {
        Self {
            name,
            bounds: vec![],
            kind: GenericParamKind::Const { ty },
        }
    }

    /// Check if this is a const generic parameter
    pub fn is_const(&self) -> bool {
        matches!(self.kind, GenericParamKind::Const { .. })
    }
}

/// Struct definition
#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: Spanned<String>,
    pub generics: Vec<GenericParam>,
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
    pub generics: Vec<GenericParam>,
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
    pub generics: Vec<GenericParam>,
    pub ty: Spanned<Type>,
    pub is_pub: bool,
}

/// Union definition (untagged, C-style)
/// All fields share the same memory location (offset 0).
/// Unlike tagged enums, there is no runtime tag - the caller is responsible
/// for knowing which field is active.
#[derive(Debug, Clone, PartialEq)]
pub struct Union {
    pub name: Spanned<String>,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<Field>,  // Reuse existing Field struct
    pub is_pub: bool,
}

/// Use/Import statement
#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub path: Vec<Spanned<String>>,
    pub alias: Option<Spanned<String>>,
}

/// Associated type in a trait
#[derive(Debug, Clone, PartialEq)]
pub struct AssociatedType {
    pub name: Spanned<String>,
    pub bounds: Vec<Spanned<String>>, // Optional trait bounds
    pub default: Option<Spanned<Type>>, // Optional default type
}

/// Trait definition: `W Name { methods }`
#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    pub name: Spanned<String>,
    pub generics: Vec<GenericParam>,
    pub super_traits: Vec<Spanned<String>>, // Super trait bounds (e.g., W Iterator: Iterable)
    pub associated_types: Vec<AssociatedType>, // Associated types (e.g., T Item)
    pub methods: Vec<TraitMethod>,
    pub is_pub: bool,
}

/// Trait method signature (may have default impl)
#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name: Spanned<String>,
    pub params: Vec<Param>,
    pub ret_type: Option<Spanned<Type>>,
    pub default_body: Option<FunctionBody>,
    pub is_async: bool,
}

/// Impl block: `X Type: Trait { methods }`
#[derive(Debug, Clone, PartialEq)]
pub struct Impl {
    pub target_type: Spanned<Type>,
    pub trait_name: Option<Spanned<String>>,
    pub generics: Vec<GenericParam>,
    pub methods: Vec<Spanned<Function>>,
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
}

impl std::fmt::Display for ConstExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstExpr::Literal(n) => write!(f, "{}", n),
            ConstExpr::Param(name) => write!(f, "{}", name),
            ConstExpr::BinOp { op, left, right } => write!(f, "({} {} {})", left, op, right),
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
}

impl std::fmt::Display for ConstBinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstBinOp::Add => write!(f, "+"),
            ConstBinOp::Sub => write!(f, "-"),
            ConstBinOp::Mul => write!(f, "*"),
            ConstBinOp::Div => write!(f, "/"),
        }
    }
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
    /// Const-sized array: `[T; N]` where N is a const expression
    ConstArray {
        element: Box<Spanned<Type>>,
        size: ConstExpr,
    },
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
    /// Defer: `D expr` - Execute expr when scope exits (LIFO order)
    Defer(Box<Spanned<Expr>>),
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
        /// Captured variables from enclosing scope (filled during type checking)
        captures: Vec<String>,
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
    /// Returns the precedence level of this operator.
    ///
    /// Higher numbers indicate higher precedence (tighter binding).
    /// For example, multiplication (10) binds tighter than addition (9).
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

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Named { name, generics } => {
                write!(f, "{}", name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", g.node)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::Array(inner) => write!(f, "[{}]", inner.node),
            Type::ConstArray { element, size } => write!(f, "[{}; {}]", element.node, size),
            Type::Map(key, val) => write!(f, "[{}:{}]", key.node, val.node),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t.node)?;
                }
                write!(f, ")")
            }
            Type::Optional(inner) => write!(f, "{}?", inner.node),
            Type::Result(inner) => write!(f, "{}!", inner.node),
            Type::Pointer(inner) => write!(f, "*{}", inner.node),
            Type::Ref(inner) => write!(f, "&{}", inner.node),
            Type::RefMut(inner) => write!(f, "&mut {}", inner.node),
            Type::Fn { params, ret } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.node)?;
                }
                write!(f, ") -> {}", ret.node)
            }
            Type::Unit => write!(f, "()"),
            Type::Infer => write!(f, "_"),
        }
    }
}
