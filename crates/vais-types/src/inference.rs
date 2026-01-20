//! Type inference logic for the Vais type system
//!
//! This module contains the type inference algorithms including unification,
//! substitution, and fresh type variable generation.

use std::collections::HashMap;
use crate::types::{ResolvedType, TypeError, TypeResult};
use crate::TypeChecker;

impl TypeChecker {
    /// Unify two types
    pub(crate) fn unify(&mut self, expected: &ResolvedType, found: &ResolvedType) -> TypeResult<()> {
        let expected = self.apply_substitutions(expected);
        let found = self.apply_substitutions(found);

        if expected == found {
            return Ok(());
        }

        match (&expected, &found) {
            // Type variables can unify with anything
            (ResolvedType::Var(id), t) | (t, ResolvedType::Var(id)) => {
                self.substitutions.insert(*id, t.clone());
                Ok(())
            }
            // Generic type parameters match with any type (type erasure)
            (ResolvedType::Generic(_), _) | (_, ResolvedType::Generic(_)) => Ok(()),
            (ResolvedType::Array(a), ResolvedType::Array(b)) => self.unify(a, b),
            (ResolvedType::Optional(a), ResolvedType::Optional(b)) => self.unify(a, b),
            (ResolvedType::Result(a), ResolvedType::Result(b)) => self.unify(a, b),
            (ResolvedType::Ref(a), ResolvedType::Ref(b)) => self.unify(a, b),
            (ResolvedType::RefMut(a), ResolvedType::RefMut(b)) => self.unify(a, b),
            (ResolvedType::Pointer(a), ResolvedType::Pointer(b)) => self.unify(a, b),
            (ResolvedType::Range(a), ResolvedType::Range(b)) => self.unify(a, b),
            (ResolvedType::Future(a), ResolvedType::Future(b)) => self.unify(a, b),
            (ResolvedType::Tuple(a), ResolvedType::Tuple(b)) if a.len() == b.len() => {
                for (ta, tb) in a.iter().zip(b.iter()) {
                    self.unify(ta, tb)?;
                }
                Ok(())
            }
            (
                ResolvedType::Fn {
                    params: pa,
                    ret: ra,
                },
                ResolvedType::Fn {
                    params: pb,
                    ret: rb,
                },
            ) if pa.len() == pb.len() => {
                for (ta, tb) in pa.iter().zip(pb.iter()) {
                    self.unify(ta, tb)?;
                }
                self.unify(ra, rb)
            }
            // Allow implicit integer type conversions (widening and narrowing)
            (a, b) if Self::is_integer_type(a) && Self::is_integer_type(b) => Ok(()),
            _ => Err(TypeError::Mismatch {
                expected: expected.to_string(),
                found: found.to_string(),
                span: None,
            }),
        }
    }

    /// Check if type is an integer type
    pub(crate) fn is_integer_type(ty: &ResolvedType) -> bool {
        matches!(
            ty,
            ResolvedType::I8
                | ResolvedType::I16
                | ResolvedType::I32
                | ResolvedType::I64
                | ResolvedType::U8
                | ResolvedType::U16
                | ResolvedType::U32
                | ResolvedType::U64
        )
    }

    /// Apply substitutions to a type
    pub(crate) fn apply_substitutions(&self, ty: &ResolvedType) -> ResolvedType {
        match ty {
            ResolvedType::Var(id) => {
                if let Some(subst) = self.substitutions.get(id) {
                    self.apply_substitutions(subst)
                } else {
                    ty.clone()
                }
            }
            ResolvedType::Array(inner) => {
                ResolvedType::Array(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Result(inner) => {
                ResolvedType::Result(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Ref(inner) => {
                ResolvedType::Ref(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Range(inner) => {
                ResolvedType::Range(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Tuple(types) => {
                ResolvedType::Tuple(types.iter().map(|t| self.apply_substitutions(t)).collect())
            }
            ResolvedType::Fn { params, ret } => ResolvedType::Fn {
                params: params.iter().map(|p| self.apply_substitutions(p)).collect(),
                ret: Box::new(self.apply_substitutions(ret)),
            },
            _ => ty.clone(),
        }
    }

    /// Create a fresh type variable
    pub(crate) fn fresh_type_var(&self) -> ResolvedType {
        let id = self.next_type_var.get();
        self.next_type_var.set(id + 1);
        ResolvedType::Var(id)
    }

    /// Substitute generic type parameters with concrete types
    pub(crate) fn substitute_generics(&self, ty: &ResolvedType, substitutions: &HashMap<String, ResolvedType>) -> ResolvedType {
        match ty {
            ResolvedType::Generic(name) => {
                substitutions.get(name).cloned().unwrap_or_else(|| ty.clone())
            }
            ResolvedType::Array(inner) => {
                ResolvedType::Array(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Map(key, value) => ResolvedType::Map(
                Box::new(self.substitute_generics(key, substitutions)),
                Box::new(self.substitute_generics(value, substitutions)),
            ),
            ResolvedType::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Result(inner) => {
                ResolvedType::Result(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Ref(inner) => {
                ResolvedType::Ref(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Range(inner) => {
                ResolvedType::Range(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Tuple(types) => {
                ResolvedType::Tuple(types.iter().map(|t| self.substitute_generics(t, substitutions)).collect())
            }
            ResolvedType::Fn { params, ret } => ResolvedType::Fn {
                params: params.iter().map(|p| self.substitute_generics(p, substitutions)).collect(),
                ret: Box::new(self.substitute_generics(ret, substitutions)),
            },
            ResolvedType::Named { name, generics } => ResolvedType::Named {
                name: name.clone(),
                generics: generics.iter().map(|g| self.substitute_generics(g, substitutions)).collect(),
            },
            _ => ty.clone(),
        }
    }
}
