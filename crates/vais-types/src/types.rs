//! Core type definitions for the Vais type system
//!
//! This module contains the fundamental type definitions used throughout
//! the type checker, including resolved types, type errors, and type signatures.

use std::collections::HashMap;
use thiserror::Error;
use vais_ast::*;

/// Type checking error
#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    Mismatch {
        expected: String,
        found: String,
        span: Option<Span>,
    },

    #[error("Undefined variable: {0}")]
    UndefinedVar(String, Option<Span>),

    #[error("Undefined type: {0}")]
    UndefinedType(String, Option<Span>),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String, Option<Span>),

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
            TypeError::UndefinedVar(_, span) => *span,
            TypeError::UndefinedType(_, span) => *span,
            TypeError::UndefinedFunction(_, span) => *span,
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
            TypeError::UndefinedVar(..) => "E002",
            TypeError::UndefinedType(..) => "E003",
            TypeError::UndefinedFunction(..) => "E004",
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
                    Some(format!("try using a type cast or conversion function"))
                } else {
                    None
                }
            }
            TypeError::UndefinedVar(name, _) => {
                Some(format!("variable '{}' not found in this scope", name))
            }
            TypeError::UndefinedFunction(name, _) => {
                Some(format!("function '{}' not found in this scope", name))
            }
            TypeError::ImmutableAssign(name, _) => {
                Some(format!("consider declaring '{}' as mutable: '{}: mut Type'", name, name))
            }
            _ => None,
        }
    }
}

/// Type checking result
pub type TypeResult<T> = Result<T, TypeError>;

/// Resolved type in the type system
#[derive(Debug, Clone, PartialEq)]
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

    // Named type (struct/enum)
    Named {
        name: String,
        generics: Vec<ResolvedType>,
    },

    // Type variable for inference
    Var(usize),

    // Generic type parameter (e.g., T in F foo<T>)
    Generic(String),

    // Unknown/Error type
    Unknown,
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
            ResolvedType::Unknown => write!(f, "?"),
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
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: HashMap<String, ResolvedType>,
    pub methods: HashMap<String, FunctionSig>,
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

/// Variable info (internal to type checker)
#[derive(Debug, Clone)]
pub(crate) struct VarInfo {
    pub(crate) ty: ResolvedType,
    pub(crate) is_mut: bool,
}
