//! Trait and impl definitions

use crate::aliases::AssociatedType;
use crate::ast_types::Type;
use crate::function::{Function, FunctionBody, Param};
use crate::generics::{GenericParam, WherePredicate};
use crate::infrastructure::Spanned;

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
