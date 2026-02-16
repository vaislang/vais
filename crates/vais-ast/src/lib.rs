//! Vais Abstract Syntax Tree
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
/// Attributes like `#[cfg(test)]`, `#\[inline\]`, `#[repr(C)]`, etc. provide metadata
/// and control compilation behavior.
///
/// Contract attributes (`requires`, `ensures`, `invariant`) can contain full expressions
/// that are stored in the `expr` field for formal verification.
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    /// Attribute name (e.g., "cfg", "inline", "repr", "requires", "ensures")
    pub name: String,
    /// Attribute arguments (e.g., cfg(test) -> ["test"], repr(C) -> ["C"])
    /// For contract attributes, this contains the original expression string
    pub args: Vec<String>,
    /// Contract expression for requires/ensures/invariant attributes
    /// Contains the parsed AST expression for formal verification
    pub expr: Option<Box<Spanned<Expr>>>,
}

/// Top-level module containing all program items.
///
/// A module represents a complete Vais source file and contains
/// all top-level definitions (functions, structs, enums, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    /// List of top-level items in this module
    pub items: Vec<Spanned<Item>>,
    /// Per-module item indices (module_path → item indices in `items`)
    /// Populated during import resolution for per-module codegen.
    /// None when not using per-module compilation.
    pub modules_map: Option<std::collections::HashMap<std::path::PathBuf, Vec<usize>>>,
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
    /// Trait alias: `T Name = TraitA + TraitB`
    TraitAlias(TraitAlias),
    /// Import statement: `U module` or `U module::{items}`
    Use(Use),
    /// Trait definition: `W Name { methods }` (W = "What" interface)
    Trait(Trait),
    /// Implementation block: `X Type: Trait { methods }` (X = "eXtend")
    Impl(Impl),
    /// Macro definition: `macro name! { rules }`
    Macro(MacroDef),
    /// Extern block: `N "C" { declarations }`
    ExternBlock(ExternBlock),
    /// Constant definition: `C NAME: Type = value`
    Const(ConstDef),
    /// Global variable definition: `G name: Type = value`
    Global(GlobalDef),
    /// Error recovery node - represents an item that failed to parse
    /// Used for continuing parsing after errors to report multiple errors at once.
    Error {
        /// Error message describing what went wrong
        message: String,
        /// Tokens that were skipped during recovery
        skipped_tokens: Vec<String>,
    },
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

/// Variance annotation for generic type parameters
/// Controls subtyping relationship between parameterized types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variance {
    /// Invariant (default): T is neither covariant nor contravariant
    /// Container<A> has no subtyping relation with Container<B>
    #[default]
    Invariant,
    /// Covariant (+T): if A <: B then Container<A> <: Container<B>
    /// Used for read-only/producer types (e.g., Iterator, Supplier)
    Covariant,
    /// Contravariant (-T): if A <: B then Container<B> <: Container<A>
    /// Used for write-only/consumer types (e.g., Predicate, Consumer)
    Contravariant,
}

/// Generic parameter kind - either a type parameter, const parameter, lifetime parameter,
/// or higher-kinded type parameter
#[derive(Debug, Clone, PartialEq)]
pub enum GenericParamKind {
    /// Type parameter with optional trait bounds (e.g., T, T: Display + Clone)
    Type { bounds: Vec<Spanned<String>> },
    /// Const parameter with a type (e.g., const N: u64)
    Const { ty: Spanned<Type> },
    /// Lifetime parameter (e.g., 'a, 'static)
    Lifetime {
        /// Lifetime bounds (e.g., 'a: 'b means 'a outlives 'b)
        bounds: Vec<String>,
    },
    /// Higher-kinded type parameter (e.g., F<_> or F<_, _>)
    /// Represents a type constructor that takes `arity` type arguments.
    /// Example: `F<_>` has arity 1, `F<_, _>` has arity 2
    HigherKinded {
        /// Number of type arguments this constructor takes
        arity: usize,
        /// Optional trait bounds on the type constructor
        bounds: Vec<Spanned<String>>,
    },
}

/// Generic parameter with optional trait bounds and variance annotation
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub name: Spanned<String>,
    pub bounds: Vec<Spanned<String>>, // Trait constraints (e.g., T: Display + Clone) - kept for backward compatibility
    pub kind: GenericParamKind,
    pub variance: Variance, // Variance annotation: Invariant (default), Covariant (+), Contravariant (-)
}

impl GenericParam {
    /// Create a type generic parameter (backward compatible constructor)
    pub fn new_type(name: Spanned<String>, bounds: Vec<Spanned<String>>) -> Self {
        Self {
            name,
            bounds: bounds.clone(),
            kind: GenericParamKind::Type { bounds },
            variance: Variance::Invariant,
        }
    }

    /// Create a type generic parameter with variance annotation
    pub fn new_type_with_variance(
        name: Spanned<String>,
        bounds: Vec<Spanned<String>>,
        variance: Variance,
    ) -> Self {
        Self {
            name,
            bounds: bounds.clone(),
            kind: GenericParamKind::Type { bounds },
            variance,
        }
    }

    /// Create a const generic parameter
    pub fn new_const(name: Spanned<String>, ty: Spanned<Type>) -> Self {
        Self {
            name,
            bounds: vec![],
            kind: GenericParamKind::Const { ty },
            variance: Variance::Invariant,
        }
    }

    /// Create a lifetime generic parameter (e.g., 'a)
    pub fn new_lifetime(name: Spanned<String>, bounds: Vec<String>) -> Self {
        Self {
            name,
            bounds: vec![],
            kind: GenericParamKind::Lifetime { bounds },
            variance: Variance::Invariant,
        }
    }

    /// Create a higher-kinded type parameter (e.g., F<_>)
    pub fn new_higher_kinded(
        name: Spanned<String>,
        arity: usize,
        bounds: Vec<Spanned<String>>,
    ) -> Self {
        Self {
            name,
            bounds: vec![],
            kind: GenericParamKind::HigherKinded { arity, bounds },
            variance: Variance::Invariant,
        }
    }

    /// Check if this is a const generic parameter
    pub fn is_const(&self) -> bool {
        matches!(self.kind, GenericParamKind::Const { .. })
    }

    /// Check if this is a higher-kinded type parameter
    pub fn is_higher_kinded(&self) -> bool {
        matches!(self.kind, GenericParamKind::HigherKinded { .. })
    }

    /// Check if this parameter is covariant
    pub fn is_covariant(&self) -> bool {
        matches!(self.variance, Variance::Covariant)
    }

    /// Check if this parameter is contravariant
    pub fn is_contravariant(&self) -> bool {
        matches!(self.variance, Variance::Contravariant)
    }
}

/// Where clause predicate: `T: Display + Clone`
#[derive(Debug, Clone, PartialEq)]
pub struct WherePredicate {
    /// The type being constrained (usually a generic parameter name)
    pub ty: Spanned<String>,
    /// Trait bounds on this type
    pub bounds: Vec<Spanned<String>>,
}

/// Struct definition
#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: Spanned<String>,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<Field>,
    pub methods: Vec<Spanned<Function>>,
    pub is_pub: bool,
    pub attributes: Vec<Attribute>,
    /// Where clause predicates (e.g., `where T: Display + Clone`)
    pub where_clause: Vec<WherePredicate>,
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
    pub attributes: Vec<Attribute>,
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

/// Trait alias: `T Name = TraitA + TraitB`
#[derive(Debug, Clone, PartialEq)]
pub struct TraitAlias {
    pub name: Spanned<String>,
    pub generics: Vec<GenericParam>,
    pub bounds: Vec<Spanned<String>>,
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
    pub fields: Vec<Field>, // Reuse existing Field struct
    pub is_pub: bool,
}

/// Use/Import statement
#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub path: Vec<Spanned<String>>,
    pub alias: Option<Spanned<String>>,
    /// Selective import items: `U mod.Item` or `U mod.{A, B}`
    /// None means import the entire module (wildcard)
    pub items: Option<Vec<Spanned<String>>>,
}

/// Associated type in a trait
/// Supports Generic Associated Types (GAT): `T Item<'a, B: Clone>`
#[derive(Debug, Clone, PartialEq)]
pub struct AssociatedType {
    pub name: Spanned<String>,
    pub generics: Vec<GenericParam>, // GAT: generic parameters for this associated type
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
    /// Where clause predicates (e.g., `where T: Display + Clone`)
    pub where_clause: Vec<WherePredicate>,
}

/// Trait method signature (may have default impl)
#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name: Spanned<String>,
    pub params: Vec<Param>,
    pub ret_type: Option<Spanned<Type>>,
    pub default_body: Option<FunctionBody>,
    pub is_async: bool,
    pub is_const: bool, // Const trait method (compile-time evaluable)
}

/// Associated type implementation
#[derive(Debug, Clone, PartialEq)]
pub struct AssociatedTypeImpl {
    pub name: Spanned<String>,
    pub ty: Spanned<Type>,
}

/// Impl block: `X Type: Trait { methods }`
#[derive(Debug, Clone, PartialEq)]
pub struct Impl {
    pub target_type: Spanned<Type>,
    pub trait_name: Option<Spanned<String>>,
    pub generics: Vec<GenericParam>,
    pub associated_types: Vec<AssociatedTypeImpl>, // `T Item = i64`
    pub methods: Vec<Spanned<Function>>,
}

// =============================================================================
// Macro System
// =============================================================================

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

/// Type expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Named type: `i64`, `String`, `Vec<T>`
    Named {
        name: String,
        generics: Vec<Spanned<Type>>,
    },
    /// Function pointer type: `fn(A, B) -> C`
    FnPtr {
        params: Vec<Spanned<Type>>,
        ret: Box<Spanned<Type>>,
        is_vararg: bool,
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
    /// Reference: `&T` or `&'a T` (with lifetime)
    Ref(Box<Spanned<Type>>),
    /// Mutable reference: `&mut T` or `&'a mut T` (with lifetime)
    RefMut(Box<Spanned<Type>>),
    /// Immutable slice: `&[T]` — fat pointer (ptr, len)
    Slice(Box<Spanned<Type>>),
    /// Mutable slice: `&mut [T]` — fat pointer (ptr, len)
    SliceMut(Box<Spanned<Type>>),
    /// Reference with explicit lifetime: `&'a T`
    RefLifetime {
        lifetime: String,
        inner: Box<Spanned<Type>>,
    },
    /// Mutable reference with explicit lifetime: `&'a mut T`
    RefMutLifetime {
        lifetime: String,
        inner: Box<Spanned<Type>>,
    },
    /// Lazy type: `Lazy<T>` - Deferred evaluation thunk
    Lazy(Box<Spanned<Type>>),
    /// Function type: `(A,B)->C`
    Fn {
        params: Vec<Spanned<Type>>,
        ret: Box<Spanned<Type>>,
    },
    /// Unit type: `()`
    Unit,
    /// Inferred type (for internal use)
    Infer,
    /// Dynamic trait object: `dyn Trait` or `dyn Trait<T>`
    /// Used for runtime polymorphism via vtable dispatch.
    DynTrait {
        trait_name: String,
        generics: Vec<Spanned<Type>>,
    },
    /// Associated type: `<T as Trait>::Item` or `Self::Item`
    /// GAT support: `<T as Trait>::Item<'a, B>` with generic arguments
    Associated {
        base: Box<Spanned<Type>>,
        trait_name: Option<String>, // None for Self::Item
        assoc_name: String,
        /// GAT generic arguments (e.g., <'a, i64> in Self::Item<'a, i64>)
        generics: Vec<Spanned<Type>>,
    },
    /// Linear type: `linear T` - must be used exactly once
    Linear(Box<Spanned<Type>>),
    /// Affine type: `affine T` - can be used at most once
    Affine(Box<Spanned<Type>>),
    /// Existential type: `X Trait` or `X Trait + Trait2` in return position
    /// Represents an opaque return type that implements the given trait bounds.
    /// Resolved to concrete type during monomorphization.
    ImplTrait { bounds: Vec<Spanned<String>> },
    /// Dependent type (Refinement type): `{x: T | predicate}`
    /// A type `T` refined by a predicate that must hold for all values.
    /// Example: `{n: i64 | n > 0}` (positive integers)
    Dependent {
        /// The bound variable name (e.g., "n" in {n: i64 | n > 0})
        var_name: String,
        /// The base type being refined
        base: Box<Spanned<Type>>,
        /// The predicate expression that must evaluate to bool
        predicate: Box<Spanned<Expr>>,
    },
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

/// Extern block: `N "C" { declarations }`
#[derive(Debug, Clone, PartialEq)]
pub struct ExternBlock {
    /// ABI string (e.g., "C", "Rust")
    pub abi: String,
    /// Extern function declarations
    pub functions: Vec<ExternFunction>,
}

/// Extern function declaration in extern block
#[derive(Debug, Clone, PartialEq)]
pub struct ExternFunction {
    pub name: Spanned<String>,
    pub params: Vec<Param>,
    pub ret_type: Option<Spanned<Type>>,
    pub is_vararg: bool,
    /// Attributes like `#[wasm_import("env", "js_alert")]`
    pub attributes: Vec<Attribute>,
}

pub mod formatter;

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
            Type::FnPtr {
                params,
                ret,
                is_vararg,
            } => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.node)?;
                }
                if *is_vararg {
                    if !params.is_empty() {
                        write!(f, ", ")?;
                    }
                    write!(f, "...")?;
                }
                write!(f, ") -> {}", ret.node)
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
            Type::Slice(inner) => write!(f, "&[{}]", inner.node),
            Type::SliceMut(inner) => write!(f, "&mut [{}]", inner.node),
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
            Type::DynTrait {
                trait_name,
                generics,
            } => {
                write!(f, "dyn {}", trait_name)?;
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
            Type::Associated {
                base,
                trait_name,
                assoc_name,
                generics,
            } => {
                if let Some(trait_name) = trait_name {
                    write!(f, "<{} as {}>::{}", base.node, trait_name, assoc_name)?;
                } else {
                    write!(f, "{}::{}", base.node, assoc_name)?;
                }
                // Display GAT parameters if present
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
            Type::Linear(inner) => write!(f, "linear {}", inner.node),
            Type::Affine(inner) => write!(f, "affine {}", inner.node),
            Type::Dependent {
                var_name,
                base,
                predicate,
            } => {
                write!(f, "{{{}: {} | {:?}}}", var_name, base.node, predicate.node)
            }
            Type::RefLifetime { lifetime, inner } => write!(f, "&'{} {}", lifetime, inner.node),
            Type::RefMutLifetime { lifetime, inner } => {
                write!(f, "&'{} mut {}", lifetime, inner.node)
            }
            Type::Lazy(inner) => write!(f, "Lazy<{}>", inner.node),
            Type::ImplTrait { bounds } => {
                let names: Vec<&str> = bounds.iter().map(|b| b.node.as_str()).collect();
                write!(f, "impl {}", names.join(" + "))
            }
        }
    }
}
