//! Type utilities for type checking
//!
//! Provides helpers for type comparison and formatting.

use aoel_ast::{PrimitiveKind, PrimitiveType, Type};
use aoel_lexer::Span;

/// Check if a type is BOOL
pub fn is_bool_type(ty: &Type) -> bool {
    matches!(ty, Type::Primitive(p) if p.kind == PrimitiveKind::Bool)
}

/// Check if a type is numeric (any integer or float)
pub fn is_numeric_type(ty: &Type) -> bool {
    matches!(ty, Type::Primitive(p) if is_numeric_primitive(p.kind))
}

/// Check if a primitive kind is numeric
pub fn is_numeric_primitive(kind: PrimitiveKind) -> bool {
    matches!(
        kind,
        PrimitiveKind::Int
            | PrimitiveKind::Int8
            | PrimitiveKind::Int16
            | PrimitiveKind::Int32
            | PrimitiveKind::Int64
            | PrimitiveKind::Uint
            | PrimitiveKind::Uint8
            | PrimitiveKind::Uint16
            | PrimitiveKind::Uint32
            | PrimitiveKind::Uint64
            | PrimitiveKind::Float32
            | PrimitiveKind::Float64
    )
}

/// Check if a type is a string
pub fn is_string_type(ty: &Type) -> bool {
    matches!(ty, Type::Primitive(p) if p.kind == PrimitiveKind::String)
}

/// Check if a type is void
pub fn is_void_type(ty: &Type) -> bool {
    matches!(ty, Type::Primitive(p) if p.kind == PrimitiveKind::Void)
}

/// Check if a type is an array
pub fn is_array_type(ty: &Type) -> bool {
    matches!(ty, Type::Array(_))
}

/// Check if a type is a map
pub fn is_map_type(ty: &Type) -> bool {
    matches!(ty, Type::Map(_))
}

/// Check if a type is a struct
pub fn is_struct_type(ty: &Type) -> bool {
    matches!(ty, Type::Struct(_))
}

/// Get element type from array
pub fn get_array_element_type(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Array(arr) => Some(&arr.element_type),
        _ => None,
    }
}

/// Get field type from struct by name
pub fn get_struct_field_type<'a>(ty: &'a Type, field_name: &str) -> Option<&'a Type> {
    match ty {
        Type::Struct(s) => s.fields.iter().find(|f| f.name.name == field_name).map(|f| &f.ty),
        _ => None,
    }
}

/// Get struct field names
pub fn get_struct_field_names(ty: &Type) -> Vec<String> {
    match ty {
        Type::Struct(s) => s.fields.iter().map(|f| f.name.name.clone()).collect(),
        _ => Vec::new(),
    }
}

/// Convert type to string for error messages
pub fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Primitive(p) => p.kind.as_str().to_string(),
        Type::Array(arr) => format!("ARRAY<{}>", type_to_string(&arr.element_type)),
        Type::Map(m) => format!(
            "MAP<{}, {}>",
            type_to_string(&m.key_type),
            type_to_string(&m.value_type)
        ),
        Type::Struct(s) => {
            let fields: Vec<String> = s
                .fields
                .iter()
                .map(|f| format!("{}: {}", f.name.name, type_to_string(&f.ty)))
                .collect();
            format!("STRUCT {{ {} }}", fields.join(", "))
        }
        Type::Optional(o) => format!("OPTIONAL<{}>", type_to_string(&o.inner_type)),
        Type::Union(u) => {
            let types: Vec<String> = u.types.iter().map(type_to_string).collect();
            format!("UNION<{}>", types.join(" | "))
        }
        Type::Ref(r) => format!("@{}", r.path),
    }
}

/// Create a BOOL type
pub fn bool_type(span: Span) -> Type {
    Type::Primitive(PrimitiveType::new(PrimitiveKind::Bool, span))
}

/// Create an INT64 type
pub fn int64_type(span: Span) -> Type {
    Type::Primitive(PrimitiveType::new(PrimitiveKind::Int64, span))
}

/// Create a FLOAT64 type
pub fn float64_type(span: Span) -> Type {
    Type::Primitive(PrimitiveType::new(PrimitiveKind::Float64, span))
}

/// Create a STRING type
pub fn string_type(span: Span) -> Type {
    Type::Primitive(PrimitiveType::new(PrimitiveKind::String, span))
}

/// Create a VOID type
pub fn void_type(span: Span) -> Type {
    Type::Primitive(PrimitiveType::new(PrimitiveKind::Void, span))
}

/// Check if two types are compatible for comparison operations
pub fn types_comparable(left: &Type, right: &Type) -> bool {
    match (left, right) {
        // Same primitive types are always comparable
        (Type::Primitive(l), Type::Primitive(r)) => {
            l.kind == r.kind || (is_numeric_primitive(l.kind) && is_numeric_primitive(r.kind))
        }
        // VOID can be compared with OPTIONAL
        (Type::Primitive(p), Type::Optional(_)) | (Type::Optional(_), Type::Primitive(p))
            if p.kind == PrimitiveKind::Void =>
        {
            true
        }
        // Same complex types
        (Type::Array(_), Type::Array(_)) => true,
        (Type::Optional(_), Type::Optional(_)) => true,
        // References might be comparable
        (Type::Ref(_), _) | (_, Type::Ref(_)) => true,
        _ => false,
    }
}

/// Check if a type is valid for arithmetic operations
pub fn valid_for_arithmetic(ty: &Type) -> bool {
    is_numeric_type(ty)
}

/// Check if two types can be used in logical operations (must both be bool)
pub fn valid_for_logical(left: &Type, right: &Type) -> bool {
    is_bool_type(left) && is_bool_type(right)
}
