//! Aggregate type definitions: Struct, Field, Enum, Variant, VariantFields, Union

use crate::infrastructure::{Attribute, Spanned};
use crate::ast_types::Type;
use crate::function::Function;
use crate::generics::{GenericParam, WherePredicate};

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
