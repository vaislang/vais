//! Type aliases and trait aliases

use crate::ast_types::Type;
use crate::generics::GenericParam;
use crate::infrastructure::Spanned;

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

/// Associated type in a trait
/// Supports Generic Associated Types (GAT): `T Item<'a, B: Clone>`
#[derive(Debug, Clone, PartialEq)]
pub struct AssociatedType {
    pub name: Spanned<String>,
    pub generics: Vec<GenericParam>, // GAT: generic parameters for this associated type
    pub bounds: Vec<Spanned<String>>, // Optional trait bounds
    pub default: Option<Spanned<Type>>, // Optional default type
}
