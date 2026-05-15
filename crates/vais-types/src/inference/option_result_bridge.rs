//! Phase 2.13 — Option/Result ↔ Named bridge helpers.
//!
//! Vais has two canonical spellings for Option/Result:
//! - Primitive: `ResolvedType::Optional(Box<T>)` and `ResolvedType::Result(ok, err)`
//! - Named: `ResolvedType::Named { name: "Option", generics: [T] }` and
//!   `ResolvedType::Named { name: "Result", generics: [ok, err] }`
//!
//! Historically the compiler has had 11+ scattered bridge sites that
//! destructure both forms individually. This module consolidates the
//! normalization into a single place. Callers should prefer these helpers
//! over ad-hoc match arms so future type-system changes only need to
//! update one file.
//!
//! **Non-goal**: this module does NOT remove the existing scattered
//! bridge sites — that is a Phase 3.x refactoring with wider blast
//! radius. This module is purely additive: a canonical single-source
//! for new call sites + a reference for future consolidation.
//!
//! Source of truth: `unification.rs:231,247,253` (Generic no-op + bridge).

#![allow(dead_code)]

use crate::types::ResolvedType;

/// Normalize a type into its primitive Option/Result form if it is one
/// of the Named-builtin aliases. Otherwise returns the input unchanged.
///
/// - `Named{"Option", [T]}` → `Optional(T)`
/// - `Named{"Result", [Ok, Err]}` → `Result(Ok, Err)`
/// - otherwise: unchanged
pub fn normalize_to_primitive(ty: &ResolvedType) -> ResolvedType {
    match ty {
        ResolvedType::Named { name, generics } if name == "Option" && generics.len() == 1 => {
            ResolvedType::Optional(Box::new(generics[0].clone()))
        }
        ResolvedType::Named { name, generics } if name == "Result" && generics.len() == 2 => {
            ResolvedType::Result(Box::new(generics[0].clone()), Box::new(generics[1].clone()))
        }
        _ => ty.clone(),
    }
}

/// Reverse direction: normalize primitive Optional/Result to the Named form.
/// Useful when code paths consistently expect the Named shape.
pub fn normalize_to_named(ty: &ResolvedType) -> ResolvedType {
    match ty {
        ResolvedType::Optional(inner) => ResolvedType::Named {
            name: "Option".to_string(),
            generics: vec![(**inner).clone()],
        },
        ResolvedType::Result(ok, err) => ResolvedType::Named {
            name: "Result".to_string(),
            generics: vec![(**ok).clone(), (**err).clone()],
        },
        _ => ty.clone(),
    }
}

/// Returns true if `ty` is either the primitive Optional or the Named
/// `Option<T>` form. For use in places that need "is this an Option?"
/// without caring about the spelling.
pub fn is_option_shape(ty: &ResolvedType) -> bool {
    matches!(ty, ResolvedType::Optional(_))
        || matches!(ty,
            ResolvedType::Named { name, generics }
            if name == "Option" && generics.len() == 1)
}

/// Returns true if `ty` is either primitive Result or Named `Result<T,E>`.
pub fn is_result_shape(ty: &ResolvedType) -> bool {
    matches!(ty, ResolvedType::Result(_, _))
        || matches!(ty,
            ResolvedType::Named { name, generics }
            if name == "Result" && generics.len() == 2)
}

/// Extract the inner `T` from an Option-shaped type, regardless of spelling.
pub fn option_inner(ty: &ResolvedType) -> Option<ResolvedType> {
    match ty {
        ResolvedType::Optional(inner) => Some((**inner).clone()),
        ResolvedType::Named { name, generics } if name == "Option" && generics.len() == 1 => {
            Some(generics[0].clone())
        }
        _ => None,
    }
}

/// Extract the `(Ok, Err)` pair from a Result-shaped type, regardless
/// of spelling.
pub fn result_inner(ty: &ResolvedType) -> Option<(ResolvedType, ResolvedType)> {
    match ty {
        ResolvedType::Result(ok, err) => Some(((**ok).clone(), (**err).clone())),
        ResolvedType::Named { name, generics } if name == "Result" && generics.len() == 2 => {
            Some((generics[0].clone(), generics[1].clone()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_option_normalizes_to_primitive() {
        let named = ResolvedType::Named {
            name: "Option".into(),
            generics: vec![ResolvedType::I64],
        };
        let prim = ResolvedType::Optional(Box::new(ResolvedType::I64));
        assert_eq!(normalize_to_primitive(&named), prim);
    }

    #[test]
    fn primitive_option_normalizes_to_named() {
        let prim = ResolvedType::Optional(Box::new(ResolvedType::I64));
        let named = ResolvedType::Named {
            name: "Option".into(),
            generics: vec![ResolvedType::I64],
        };
        assert_eq!(normalize_to_named(&prim), named);
    }

    #[test]
    fn is_option_shape_detects_both_forms() {
        let prim = ResolvedType::Optional(Box::new(ResolvedType::I64));
        let named = ResolvedType::Named {
            name: "Option".into(),
            generics: vec![ResolvedType::I64],
        };
        let neither = ResolvedType::I64;
        assert!(is_option_shape(&prim));
        assert!(is_option_shape(&named));
        assert!(!is_option_shape(&neither));
    }

    #[test]
    fn is_result_shape_detects_both_forms() {
        let prim = ResolvedType::Result(Box::new(ResolvedType::I64), Box::new(ResolvedType::Str));
        let named = ResolvedType::Named {
            name: "Result".into(),
            generics: vec![ResolvedType::I64, ResolvedType::Str],
        };
        assert!(is_result_shape(&prim));
        assert!(is_result_shape(&named));
    }

    #[test]
    fn option_inner_extracts_both_forms() {
        let prim = ResolvedType::Optional(Box::new(ResolvedType::I64));
        let named = ResolvedType::Named {
            name: "Option".into(),
            generics: vec![ResolvedType::Str],
        };
        assert_eq!(option_inner(&prim), Some(ResolvedType::I64));
        assert_eq!(option_inner(&named), Some(ResolvedType::Str));
        assert_eq!(option_inner(&ResolvedType::I64), None);
    }

    #[test]
    fn result_inner_extracts_both_forms() {
        let prim = ResolvedType::Result(Box::new(ResolvedType::I64), Box::new(ResolvedType::Str));
        let named = ResolvedType::Named {
            name: "Result".into(),
            generics: vec![ResolvedType::Bool, ResolvedType::I64],
        };
        assert_eq!(
            result_inner(&prim),
            Some((ResolvedType::I64, ResolvedType::Str))
        );
        assert_eq!(
            result_inner(&named),
            Some((ResolvedType::Bool, ResolvedType::I64))
        );
    }
}
