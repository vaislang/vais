//! Core type definitions for the Vais type system
//!
//! This module contains the fundamental type definitions used throughout
//! the type checker, including resolved types, type errors, and type signatures.

use std::collections::HashMap;
use thiserror::Error;
use vais_ast::*;

/// Calculate Levenshtein distance between two strings
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,      // deletion
                    matrix[i][j - 1] + 1,      // insertion
                ),
                matrix[i - 1][j - 1] + cost,   // substitution
            );
        }
    }

    matrix[a_len][b_len]
}

/// Find the most similar name from a list of candidates
/// Returns None if no name is similar enough (distance > threshold)
pub fn find_similar_name<'a>(name: &str, candidates: impl Iterator<Item = &'a str>) -> Option<String> {
    let name_lower = name.to_lowercase();
    let max_distance = std::cmp::max(2, name.len() / 3); // Allow ~1/3 of chars to be different

    let mut best_match: Option<(String, usize)> = None;

    for candidate in candidates {
        let candidate_lower = candidate.to_lowercase();
        let distance = levenshtein_distance(&name_lower, &candidate_lower);

        if distance <= max_distance {
            if let Some((_, best_dist)) = &best_match {
                if distance < *best_dist {
                    best_match = Some((candidate.to_string(), distance));
                }
            } else {
                best_match = Some((candidate.to_string(), distance));
            }
        }
    }

    best_match.map(|(name, _)| name)
}

/// Type checking error
#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    Mismatch {
        expected: String,
        found: String,
        span: Option<Span>,
    },

    #[error("Undefined variable: {name}")]
    UndefinedVar {
        name: String,
        span: Option<Span>,
        suggestion: Option<String>,
    },

    #[error("Undefined type: {name}")]
    UndefinedType {
        name: String,
        span: Option<Span>,
        suggestion: Option<String>,
    },

    #[error("Undefined function: {name}")]
    UndefinedFunction {
        name: String,
        span: Option<Span>,
        suggestion: Option<String>,
    },

    #[error("Cannot call non-function type: {0}")]
    NotCallable(String, Option<Span>),

    #[error("Wrong number of arguments: expected {expected}, got {got}")]
    ArgCount {
        expected: usize,
        got: usize,
        span: Option<Span>,
    },

    #[error("Cannot infer type")]
    CannotInfer,

    #[error("Duplicate definition: {0}")]
    Duplicate(String, Option<Span>),

    #[error("Cannot assign to immutable variable: {0}")]
    ImmutableAssign(String, Option<Span>),

    #[error("Non-exhaustive match: missing patterns {0}")]
    NonExhaustiveMatch(String, Option<Span>),

    #[error("Unreachable pattern at arm {0}")]
    UnreachablePattern(usize, Option<Span>),
}

impl TypeError {
    /// Get the span associated with this error, if available
    pub fn span(&self) -> Option<Span> {
        match self {
            TypeError::Mismatch { span, .. } => *span,
            TypeError::UndefinedVar { span, .. } => *span,
            TypeError::UndefinedType { span, .. } => *span,
            TypeError::UndefinedFunction { span, .. } => *span,
            TypeError::NotCallable(_, span) => *span,
            TypeError::ArgCount { span, .. } => *span,
            TypeError::CannotInfer => None,
            TypeError::Duplicate(_, span) => *span,
            TypeError::ImmutableAssign(_, span) => *span,
            TypeError::NonExhaustiveMatch(_, span) => *span,
            TypeError::UnreachablePattern(_, span) => *span,
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> &str {
        match self {
            TypeError::Mismatch { .. } => "E001",
            TypeError::UndefinedVar { .. } => "E002",
            TypeError::UndefinedType { .. } => "E003",
            TypeError::UndefinedFunction { .. } => "E004",
            TypeError::NotCallable(..) => "E005",
            TypeError::ArgCount { .. } => "E006",
            TypeError::CannotInfer => "E007",
            TypeError::Duplicate(..) => "E008",
            TypeError::ImmutableAssign(..) => "E009",
            TypeError::NonExhaustiveMatch(..) => "E010",
            TypeError::UnreachablePattern(..) => "E011",
        }
    }

    /// Get a helpful message for this error
    pub fn help(&self) -> Option<String> {
        match self {
            TypeError::Mismatch { expected, found, .. } => {
                if expected == "i64" && found == "Str" {
                    Some("consider converting the string to a number".to_string())
                } else if expected.starts_with('i') || expected.starts_with('u') || expected.starts_with('f') {
                    Some("try using a type cast or conversion function".to_string())
                } else {
                    None
                }
            }
            TypeError::UndefinedVar { name, suggestion, .. } => {
                if let Some(sug) = suggestion {
                    Some(format!("did you mean '{}'?", sug))
                } else {
                    Some(format!("variable '{}' not found in this scope", name))
                }
            }
            TypeError::UndefinedType { name, suggestion, .. } => {
                if let Some(sug) = suggestion {
                    Some(format!("did you mean '{}'?", sug))
                } else {
                    Some(format!("type '{}' not found in this scope", name))
                }
            }
            TypeError::UndefinedFunction { name, suggestion, .. } => {
                if let Some(sug) = suggestion {
                    Some(format!("did you mean '{}'?", sug))
                } else {
                    Some(format!("function '{}' not found in this scope", name))
                }
            }
            TypeError::ImmutableAssign(name, _) => {
                Some(format!("consider declaring '{}' as mutable: '{}: mut Type'", name, name))
            }
            _ => None,
        }
    }

    /// Get the localized title for this error
    pub fn localized_title(&self) -> String {
        let key = format!("type.{}.title", self.error_code());
        vais_i18n::get_simple(&key)
    }

    /// Get the localized message for this error
    pub fn localized_message(&self) -> String {
        let key = format!("type.{}.message", self.error_code());
        match self {
            TypeError::Mismatch { expected, found, .. } => {
                vais_i18n::get(&key, &[("expected", expected), ("found", found)])
            }
            TypeError::UndefinedVar { name, .. } => {
                vais_i18n::get(&key, &[("name", name)])
            }
            TypeError::UndefinedType { name, .. } => {
                vais_i18n::get(&key, &[("name", name)])
            }
            TypeError::UndefinedFunction { name, .. } => {
                vais_i18n::get(&key, &[("name", name)])
            }
            TypeError::NotCallable(type_name, _) => {
                vais_i18n::get(&key, &[("type", type_name)])
            }
            TypeError::ArgCount { expected, got, .. } => {
                vais_i18n::get(&key, &[
                    ("expected", &expected.to_string()),
                    ("got", &got.to_string()),
                ])
            }
            TypeError::CannotInfer => {
                vais_i18n::get_simple(&key)
            }
            TypeError::Duplicate(name, _) => {
                vais_i18n::get(&key, &[("name", name)])
            }
            TypeError::ImmutableAssign(name, _) => {
                vais_i18n::get(&key, &[("name", name)])
            }
            TypeError::NonExhaustiveMatch(patterns, _) => {
                vais_i18n::get(&key, &[("patterns", patterns)])
            }
            TypeError::UnreachablePattern(arm, _) => {
                vais_i18n::get(&key, &[("arm", &arm.to_string())])
            }
        }
    }

    /// Get the localized help message for this error
    pub fn localized_help(&self) -> Option<String> {
        let key = format!("type.{}.help", self.error_code());
        if vais_i18n::has_key(&key) {
            Some(match self {
                TypeError::UndefinedVar { name, .. } => {
                    vais_i18n::get(&key, &[("name", name)])
                }
                TypeError::UndefinedFunction { name, .. } => {
                    vais_i18n::get(&key, &[("name", name)])
                }
                TypeError::ImmutableAssign(name, _) => {
                    vais_i18n::get(&key, &[("name", name)])
                }
                _ => vais_i18n::get_simple(&key),
            })
        } else {
            None
        }
    }
}

/// Type checking result
pub type TypeResult<T> = Result<T, TypeError>;

/// Resolved const value for const generics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedConst {
    /// Concrete integer value
    Value(i64),
    /// Unresolved const parameter
    Param(String),
    /// Binary operation (for type display/error messages)
    BinOp {
        op: ConstBinOp,
        left: Box<ResolvedConst>,
        right: Box<ResolvedConst>,
    },
}

impl ResolvedConst {
    /// Try to evaluate to a concrete value
    pub fn try_evaluate(&self) -> Option<i64> {
        match self {
            ResolvedConst::Value(n) => Some(*n),
            ResolvedConst::Param(_) => None,
            ResolvedConst::BinOp { op, left, right } => {
                let l = left.try_evaluate()?;
                let r = right.try_evaluate()?;
                Some(match op {
                    ConstBinOp::Add => l.checked_add(r)?,
                    ConstBinOp::Sub => l.checked_sub(r)?,
                    ConstBinOp::Mul => l.checked_mul(r)?,
                    ConstBinOp::Div => {
                        if r == 0 {
                            return None;
                        }
                        l.checked_div(r)?
                    }
                })
            }
        }
    }
}

impl std::fmt::Display for ResolvedConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedConst::Value(n) => write!(f, "{}", n),
            ResolvedConst::Param(name) => write!(f, "{}", name),
            ResolvedConst::BinOp { op, left, right } => write!(f, "({} {} {})", left, op, right),
        }
    }
}

/// Const binary operation for const expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Resolved type in the type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedType {
    // Primitives
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Str,
    Unit,

    // Compound types
    Array(Box<ResolvedType>),
    /// Const-sized array: `[T; N]` where N is a const expression
    ConstArray {
        element: Box<ResolvedType>,
        size: ResolvedConst,
    },
    Map(Box<ResolvedType>, Box<ResolvedType>),
    Tuple(Vec<ResolvedType>),
    Optional(Box<ResolvedType>),
    Result(Box<ResolvedType>),
    Pointer(Box<ResolvedType>),
    Ref(Box<ResolvedType>),
    RefMut(Box<ResolvedType>),
    Range(Box<ResolvedType>),
    Future(Box<ResolvedType>),

    // Function type
    Fn {
        params: Vec<ResolvedType>,
        ret: Box<ResolvedType>,
    },

    // Function pointer type (for C FFI callbacks)
    FnPtr {
        params: Vec<ResolvedType>,
        ret: Box<ResolvedType>,
        is_vararg: bool,
    },

    // Named type (struct/enum)
    Named {
        name: String,
        generics: Vec<ResolvedType>,
    },

    // Type variable for inference
    Var(usize),

    // Generic type parameter (e.g., T in F foo<T>)
    Generic(String),

    // Const generic parameter (e.g., N in F foo<const N: u64>)
    ConstGeneric(String),

    // Unknown/Error type
    Unknown,

    // Never type - represents a type that never returns (e.g., return, break, continue)
    // This type unifies with any other type
    Never,

    // SIMD vector type: <lanes x element_type>
    // e.g., Vector { element: F32, lanes: 4 } -> <4 x float>
    Vector {
        element: Box<ResolvedType>,
        lanes: u32,
    },

    /// Dynamic trait object: `dyn Trait` or `dyn Trait<T>`
    /// Stored as a fat pointer: (vtable*, data*)
    /// Used for runtime polymorphism via vtable-based dispatch.
    DynTrait {
        trait_name: String,
        generics: Vec<ResolvedType>,
    },
}

impl ResolvedType {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            ResolvedType::I8
                | ResolvedType::I16
                | ResolvedType::I32
                | ResolvedType::I64
                | ResolvedType::I128
                | ResolvedType::U8
                | ResolvedType::U16
                | ResolvedType::U32
                | ResolvedType::U64
                | ResolvedType::U128
                | ResolvedType::F32
                | ResolvedType::F64
                | ResolvedType::Generic(_) // Generics are assumed to support numeric ops
                | ResolvedType::Var(_) // Type variables might resolve to numeric
                | ResolvedType::Unknown // Unknown might be numeric
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            ResolvedType::I8
                | ResolvedType::I16
                | ResolvedType::I32
                | ResolvedType::I64
                | ResolvedType::I128
                | ResolvedType::U8
                | ResolvedType::U16
                | ResolvedType::U32
                | ResolvedType::U64
                | ResolvedType::U128
        )
    }

    pub fn is_float(&self) -> bool {
        matches!(self, ResolvedType::F32 | ResolvedType::F64)
    }
}

impl std::fmt::Display for ResolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedType::I8 => write!(f, "i8"),
            ResolvedType::I16 => write!(f, "i16"),
            ResolvedType::I32 => write!(f, "i32"),
            ResolvedType::I64 => write!(f, "i64"),
            ResolvedType::I128 => write!(f, "i128"),
            ResolvedType::U8 => write!(f, "u8"),
            ResolvedType::U16 => write!(f, "u16"),
            ResolvedType::U32 => write!(f, "u32"),
            ResolvedType::U64 => write!(f, "u64"),
            ResolvedType::U128 => write!(f, "u128"),
            ResolvedType::F32 => write!(f, "f32"),
            ResolvedType::F64 => write!(f, "f64"),
            ResolvedType::Bool => write!(f, "bool"),
            ResolvedType::Str => write!(f, "str"),
            ResolvedType::Unit => write!(f, "()"),
            ResolvedType::Array(t) => write!(f, "[{}]", t),
            ResolvedType::ConstArray { element, size } => write!(f, "[{}; {}]", element, size),
            ResolvedType::Map(k, v) => write!(f, "[{}:{}]", k, v),
            ResolvedType::Tuple(ts) => {
                write!(f, "(")?;
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            ResolvedType::Optional(t) => write!(f, "{}?", t),
            ResolvedType::Result(t) => write!(f, "{}!", t),
            ResolvedType::Pointer(t) => write!(f, "*{}", t),
            ResolvedType::Ref(t) => write!(f, "&{}", t),
            ResolvedType::RefMut(t) => write!(f, "&mut {}", t),
            ResolvedType::Range(t) => write!(f, "Range<{}>", t),
            ResolvedType::Future(t) => write!(f, "Future<{}>", t),
            ResolvedType::Fn { params, ret } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ")->{}", ret)
            }
            ResolvedType::FnPtr { params, ret, is_vararg } => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", p)?;
                }
                if *is_vararg {
                    if !params.is_empty() {
                        write!(f, ",")?;
                    }
                    write!(f, "...")?;
                }
                write!(f, ")->{}", ret)
            }
            ResolvedType::Named { name, generics } => {
                write!(f, "{}", name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", g)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            ResolvedType::Var(id) => write!(f, "?{}", id),
            ResolvedType::Generic(name) => write!(f, "{}", name),
            ResolvedType::ConstGeneric(name) => write!(f, "const {}", name),
            ResolvedType::Unknown => write!(f, "?"),
            ResolvedType::Never => write!(f, "!"),
            ResolvedType::Vector { element, lanes } => write!(f, "Vec{}x{}", lanes, element),
            ResolvedType::DynTrait { trait_name, generics } => {
                write!(f, "dyn {}", trait_name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", g)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
        }
    }
}

/// Function signature
#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub generics: Vec<String>,
    pub generic_bounds: HashMap<String, Vec<String>>, // generic name -> trait bounds
    pub params: Vec<(String, ResolvedType, bool)>, // (name, type, is_mut)
    pub ret: ResolvedType,
    pub is_async: bool,
    pub is_vararg: bool, // true for variadic C functions (printf, etc.)
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: HashMap<String, ResolvedType>,
    pub methods: HashMap<String, FunctionSig>,
    pub repr_c: bool, // true if #[repr(C)] attribute is present
}

/// Enum variant field types
#[derive(Debug, Clone)]
pub enum VariantFieldTypes {
    /// Unit variant (no fields)
    Unit,
    /// Tuple variant with positional fields
    Tuple(Vec<ResolvedType>),
    /// Struct variant with named fields
    Struct(HashMap<String, ResolvedType>),
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: HashMap<String, VariantFieldTypes>,
}

/// Union definition (untagged, C-style)
/// All fields share the same memory location (offset 0).
/// No runtime tag - caller is responsible for knowing which field is active.
#[derive(Debug, Clone)]
pub struct UnionDef {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: HashMap<String, ResolvedType>,
}

/// Variable info (internal to type checker)
#[derive(Debug, Clone)]
pub(crate) struct VarInfo {
    pub(crate) ty: ResolvedType,
    pub(crate) is_mut: bool,
}

/// Generic instantiation tracking for monomorphization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenericInstantiation {
    /// Base name of the generic item (function or struct)
    pub base_name: String,
    /// Concrete type arguments
    pub type_args: Vec<ResolvedType>,
    /// Mangled name for code generation
    pub mangled_name: String,
    /// Kind of instantiation
    pub kind: InstantiationKind,
}

/// Kind of generic instantiation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstantiationKind {
    Function,
    Struct,
    Method { struct_name: String },
}

impl GenericInstantiation {
    /// Create a new function instantiation
    pub fn function(base_name: &str, type_args: Vec<ResolvedType>) -> Self {
        let mangled = mangle_name(base_name, &type_args);
        Self {
            base_name: base_name.to_string(),
            type_args,
            mangled_name: mangled,
            kind: InstantiationKind::Function,
        }
    }

    /// Create a new struct instantiation
    pub fn struct_type(base_name: &str, type_args: Vec<ResolvedType>) -> Self {
        let mangled = mangle_name(base_name, &type_args);
        Self {
            base_name: base_name.to_string(),
            type_args,
            mangled_name: mangled,
            kind: InstantiationKind::Struct,
        }
    }

    /// Create a new method instantiation
    pub fn method(struct_name: &str, method_name: &str, type_args: Vec<ResolvedType>) -> Self {
        let base = format!("{}_{}", struct_name, method_name);
        let mangled = mangle_name(&base, &type_args);
        Self {
            base_name: method_name.to_string(),
            type_args,
            mangled_name: mangled,
            kind: InstantiationKind::Method {
                struct_name: struct_name.to_string(),
            },
        }
    }
}

/// Mangle a generic name with type arguments
pub fn mangle_name(base: &str, type_args: &[ResolvedType]) -> String {
    if type_args.is_empty() {
        base.to_string()
    } else {
        let args_str = type_args
            .iter()
            .map(|t| mangle_type(t))
            .collect::<Vec<_>>()
            .join("_");
        format!("{}${}", base, args_str)
    }
}

/// Mangle a single type for use in mangled names
pub fn mangle_type(ty: &ResolvedType) -> String {
    match ty {
        ResolvedType::I8 => "i8".to_string(),
        ResolvedType::I16 => "i16".to_string(),
        ResolvedType::I32 => "i32".to_string(),
        ResolvedType::I64 => "i64".to_string(),
        ResolvedType::I128 => "i128".to_string(),
        ResolvedType::U8 => "u8".to_string(),
        ResolvedType::U16 => "u16".to_string(),
        ResolvedType::U32 => "u32".to_string(),
        ResolvedType::U64 => "u64".to_string(),
        ResolvedType::U128 => "u128".to_string(),
        ResolvedType::F32 => "f32".to_string(),
        ResolvedType::F64 => "f64".to_string(),
        ResolvedType::Bool => "bool".to_string(),
        ResolvedType::Str => "str".to_string(),
        ResolvedType::Unit => "unit".to_string(),
        ResolvedType::Named { name, generics } => {
            if generics.is_empty() {
                name.clone()
            } else {
                let args = generics
                    .iter()
                    .map(|g| mangle_type(g))
                    .collect::<Vec<_>>()
                    .join("_");
                format!("{}_{}", name, args)
            }
        }
        ResolvedType::Array(inner) => format!("arr_{}", mangle_type(inner)),
        ResolvedType::Pointer(inner) => format!("ptr_{}", mangle_type(inner)),
        ResolvedType::Ref(inner) => format!("ref_{}", mangle_type(inner)),
        ResolvedType::RefMut(inner) => format!("refmut_{}", mangle_type(inner)),
        ResolvedType::Optional(inner) => format!("opt_{}", mangle_type(inner)),
        ResolvedType::Result(inner) => format!("res_{}", mangle_type(inner)),
        ResolvedType::Future(inner) => format!("fut_{}", mangle_type(inner)),
        ResolvedType::Tuple(types) => {
            let args = types
                .iter()
                .map(|t| mangle_type(t))
                .collect::<Vec<_>>()
                .join("_");
            format!("tup_{}", args)
        }
        ResolvedType::Fn { params, ret } => {
            let params_str = params
                .iter()
                .map(|p| mangle_type(p))
                .collect::<Vec<_>>()
                .join("_");
            format!("fn_{}_{}", params_str, mangle_type(ret))
        }
        ResolvedType::Generic(name) => name.clone(),
        ResolvedType::Var(id) => format!("v{}", id),
        ResolvedType::Vector { element, lanes } => format!("vec{}_{}", lanes, mangle_type(element)),
        _ => "unknown".to_string(),
    }
}

/// Substitute generic type parameters with concrete types
pub fn substitute_type(
    ty: &ResolvedType,
    substitutions: &HashMap<String, ResolvedType>,
) -> ResolvedType {
    match ty {
        ResolvedType::Generic(name) => {
            substitutions.get(name).cloned().unwrap_or_else(|| ty.clone())
        }
        ResolvedType::Named { name, generics } => {
            let new_generics = generics
                .iter()
                .map(|g| substitute_type(g, substitutions))
                .collect();
            ResolvedType::Named {
                name: name.clone(),
                generics: new_generics,
            }
        }
        ResolvedType::Array(inner) => {
            ResolvedType::Array(Box::new(substitute_type(inner, substitutions)))
        }
        ResolvedType::Pointer(inner) => {
            ResolvedType::Pointer(Box::new(substitute_type(inner, substitutions)))
        }
        ResolvedType::Ref(inner) => {
            ResolvedType::Ref(Box::new(substitute_type(inner, substitutions)))
        }
        ResolvedType::RefMut(inner) => {
            ResolvedType::RefMut(Box::new(substitute_type(inner, substitutions)))
        }
        ResolvedType::Optional(inner) => {
            ResolvedType::Optional(Box::new(substitute_type(inner, substitutions)))
        }
        ResolvedType::Result(inner) => {
            ResolvedType::Result(Box::new(substitute_type(inner, substitutions)))
        }
        ResolvedType::Future(inner) => {
            ResolvedType::Future(Box::new(substitute_type(inner, substitutions)))
        }
        ResolvedType::Tuple(types) => {
            let new_types = types
                .iter()
                .map(|t| substitute_type(t, substitutions))
                .collect();
            ResolvedType::Tuple(new_types)
        }
        ResolvedType::Fn { params, ret } => {
            let new_params = params
                .iter()
                .map(|p| substitute_type(p, substitutions))
                .collect();
            let new_ret = Box::new(substitute_type(ret, substitutions));
            ResolvedType::Fn {
                params: new_params,
                ret: new_ret,
            }
        }
        ResolvedType::Vector { element, lanes } => {
            ResolvedType::Vector {
                element: Box::new(substitute_type(element, substitutions)),
                lanes: *lanes,
            }
        }
        // Primitives and other types pass through unchanged
        _ => ty.clone(),
    }
}
