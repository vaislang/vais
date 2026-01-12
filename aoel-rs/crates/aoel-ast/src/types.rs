//! Type system definitions

use aoel_lexer::Span;
use serde::{Deserialize, Serialize};
use crate::{AstNode, Ident, ExternalRef};

/// All type representations in AOEL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    /// Primitive types (INT32, STRING, BOOL, etc.)
    Primitive(PrimitiveType),

    /// Array type: ARRAY<T>
    Array(Box<ArrayType>),

    /// Map type: MAP<K, V>
    Map(Box<MapType>),

    /// Struct type: STRUCT { field: Type, ... }
    Struct(StructType),

    /// Optional type: OPTIONAL<T>
    Optional(Box<OptionalType>),

    /// Union type: UNION<T1 | T2 | ...>
    Union(UnionType),

    /// Reference to external type: @schemas.User
    Ref(ExternalRef),
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Primitive(t) => t.span,
            Type::Array(t) => t.span,
            Type::Map(t) => t.span,
            Type::Struct(t) => t.span,
            Type::Optional(t) => t.span,
            Type::Union(t) => t.span,
            Type::Ref(t) => t.span,
        }
    }
}

/// Primitive type kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveKind {
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
    Bool,
    String,
    Bytes,
    Void,
}

impl PrimitiveKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrimitiveKind::Int => "INT",
            PrimitiveKind::Int8 => "INT8",
            PrimitiveKind::Int16 => "INT16",
            PrimitiveKind::Int32 => "INT32",
            PrimitiveKind::Int64 => "INT64",
            PrimitiveKind::Uint => "UINT",
            PrimitiveKind::Uint8 => "UINT8",
            PrimitiveKind::Uint16 => "UINT16",
            PrimitiveKind::Uint32 => "UINT32",
            PrimitiveKind::Uint64 => "UINT64",
            PrimitiveKind::Float32 => "FLOAT32",
            PrimitiveKind::Float64 => "FLOAT64",
            PrimitiveKind::Bool => "BOOL",
            PrimitiveKind::String => "STRING",
            PrimitiveKind::Bytes => "BYTES",
            PrimitiveKind::Void => "VOID",
        }
    }
}

/// Primitive type node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrimitiveType {
    pub kind: PrimitiveKind,
    pub span: Span,
}

impl PrimitiveType {
    pub fn new(kind: PrimitiveKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Array type: ARRAY<element_type>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayType {
    pub element_type: Type,
    pub span: Span,
}

impl ArrayType {
    pub fn new(element_type: Type, span: Span) -> Self {
        Self { element_type, span }
    }
}

/// Map type: MAP<key_type, value_type>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapType {
    pub key_type: Type,
    pub value_type: Type,
    pub span: Span,
}

impl MapType {
    pub fn new(key_type: Type, value_type: Type, span: Span) -> Self {
        Self {
            key_type,
            value_type,
            span,
        }
    }
}

/// Struct field
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
}

/// Struct type: STRUCT { fields... }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructType {
    pub fields: Vec<StructField>,
    pub span: Span,
}

impl StructType {
    pub fn new(fields: Vec<StructField>, span: Span) -> Self {
        Self { fields, span }
    }
}

/// Optional type: OPTIONAL<inner_type>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionalType {
    pub inner_type: Type,
    pub span: Span,
}

impl OptionalType {
    pub fn new(inner_type: Type, span: Span) -> Self {
        Self { inner_type, span }
    }
}

/// Union type: UNION<types...>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnionType {
    pub types: Vec<Type>,
    pub span: Span,
}

impl UnionType {
    pub fn new(types: Vec<Type>, span: Span) -> Self {
        Self { types, span }
    }
}
