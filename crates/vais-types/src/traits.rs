//! Trait system for Vais type checker
//!
//! This module contains trait-related definitions and implementation checking.

use crate::{ResolvedType, TypeChecker, TypeError, TypeResult};
use std::collections::{HashMap, HashSet};

/// Trait method signature
#[derive(Debug, Clone)]
pub struct TraitMethodSig {
    pub name: String,
    pub params: Vec<(String, ResolvedType, bool)>, // (name, type, is_mut) - first param is &self
    pub ret: ResolvedType,
    pub has_default: bool,
    pub is_async: bool,
    pub is_const: bool, // Const trait method (compile-time evaluable)
}

/// Associated type definition
/// Supports Generic Associated Types (GAT): associated types with their own generic params
#[derive(Debug, Clone)]
pub struct AssociatedTypeDef {
    pub name: String,
    pub generics: Vec<String>, // GAT: generic parameter names (e.g., ["'a", "B"])
    pub generic_bounds: HashMap<String, Vec<String>>, // GAT: bounds for each generic param
    pub bounds: Vec<String>,
    pub default: Option<ResolvedType>,
}

/// Trait definition
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: String,
    pub generics: Vec<String>,
    pub super_traits: Vec<String>,
    pub associated_types: HashMap<String, AssociatedTypeDef>,
    pub methods: HashMap<String, TraitMethodSig>,
}

/// Tracks which types implement which traits
#[derive(Debug, Clone)]
pub(crate) struct TraitImpl {
    pub(crate) trait_name: String,
    pub(crate) type_name: String,
    /// Concrete types for associated types
    pub(crate) associated_types: HashMap<String, ResolvedType>,
}

impl TypeChecker {
    /// Check if a type implements a trait (with trait alias expansion)
    pub(crate) fn type_implements_trait(&self, ty: &ResolvedType, trait_name: &str) -> bool {
        let mut visited = HashSet::new();
        self.type_implements_trait_impl(ty, trait_name, &mut visited)
    }

    fn type_implements_trait_impl(
        &self,
        ty: &ResolvedType,
        trait_name: &str,
        visited: &mut HashSet<String>,
    ) -> bool {
        // Expand trait aliases with cycle detection
        if let Some(bounds) = self.trait_aliases.get(trait_name) {
            if !visited.insert(trait_name.to_string()) {
                return false; // Cycle detected — conservatively reject
            }
            return bounds
                .iter()
                .all(|bound| self.type_implements_trait_impl(ty, bound, visited));
        }

        // Check if there's an explicit impl
        if let ResolvedType::Named { name, .. } = ty {
            for impl_ in &self.trait_impls {
                if &impl_.type_name == name && impl_.trait_name == trait_name {
                    return true;
                }
            }
        }

        // Generic types are assumed to implement their bounds
        if let ResolvedType::Generic(name) = ty {
            if let Some(bounds) = self.current_generic_bounds.get(name) {
                // Also expand trait aliases in bounds
                if bounds.contains(&trait_name.to_string()) {
                    return true;
                }
                // Check if any bound is a trait alias that expands to include trait_name
                for bound in bounds {
                    if let Some(alias_bounds) = self.trait_aliases.get(bound.as_str()) {
                        if alias_bounds.contains(&trait_name.to_string()) {
                            return true;
                        }
                    }
                }
                return false;
            }
        }

        // Generic type parameters with no explicit bounds are assumed to satisfy any trait
        // (will be verified at call site)
        if matches!(ty, ResolvedType::Generic(_)) {
            return true;
        }

        // Primitive types - check for built-in trait implementations
        self.primitive_implements_trait(ty, trait_name)
    }

    /// Check if a primitive type implements a built-in trait
    pub(crate) fn primitive_implements_trait(&self, ty: &ResolvedType, trait_name: &str) -> bool {
        match trait_name {
            // All types can be compared for equality
            "Eq" | "PartialEq" => true,
            // Numeric types support ordering
            "Ord" | "PartialOrd" => ty.is_numeric(),
            // All types can be cloned
            "Clone" | "Copy" => matches!(
                ty,
                ResolvedType::I8
                    | ResolvedType::I16
                    | ResolvedType::I32
                    | ResolvedType::I64
                    | ResolvedType::U8
                    | ResolvedType::U16
                    | ResolvedType::U32
                    | ResolvedType::U64
                    | ResolvedType::F32
                    | ResolvedType::F64
                    | ResolvedType::Bool
            ),
            // Default is only for some types
            "Default" => matches!(
                ty,
                ResolvedType::I8
                    | ResolvedType::I16
                    | ResolvedType::I32
                    | ResolvedType::I64
                    | ResolvedType::U8
                    | ResolvedType::U16
                    | ResolvedType::U32
                    | ResolvedType::U64
                    | ResolvedType::F32
                    | ResolvedType::F64
                    | ResolvedType::Bool
                    | ResolvedType::Str
            ),
            // Display/Debug - primitives support these
            "Display" | "Debug" => matches!(
                ty,
                ResolvedType::I8
                    | ResolvedType::I16
                    | ResolvedType::I32
                    | ResolvedType::I64
                    | ResolvedType::U8
                    | ResolvedType::U16
                    | ResolvedType::U32
                    | ResolvedType::U64
                    | ResolvedType::F32
                    | ResolvedType::F64
                    | ResolvedType::Bool
                    | ResolvedType::Str
            ),
            // Drop/AsyncDrop - not implemented by primitives
            "Drop" | "AsyncDrop" => false,
            // Future - not implemented by primitives
            "Future" => false,
            // Unknown traits - assume not implemented
            _ => false,
        }
    }

    /// Verify trait bounds when calling a generic function.
    /// Reserved for stricter trait bound checking.
    #[allow(dead_code)]
    pub(crate) fn verify_trait_bounds(
        &self,
        generic_args: &[(String, ResolvedType)],
        bounds: &HashMap<String, Vec<String>>,
    ) -> TypeResult<()> {
        for (generic_name, concrete_type) in generic_args {
            if let Some(required_traits) = bounds.get(generic_name) {
                for trait_name in required_traits {
                    if !self.type_implements_trait(concrete_type, trait_name) {
                        return Err(TypeError::Mismatch {
                            expected: format!("type implementing trait '{}'", trait_name),
                            found: format!(
                                "type '{}' which does not implement '{}'",
                                concrete_type, trait_name
                            ),
                            span: None,
                        });
                    }
                }
            }
        }
        Ok(())
    }
}
