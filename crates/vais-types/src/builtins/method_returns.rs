//! Phase 2.11 — Consolidated builtin method return-type table.
//!
//! Historically the compiler has had method return-type inference scattered
//! across many files (`checker_expr/calls.rs`, `checker_expr/collections.rs`,
//! `checker_expr/string_methods.rs`, codegen-side duplicates, etc.). This
//! module provides a single authoritative mapping `(receiver_shape,
//! method_name) -> return_type_rule` for the most common cases. New
//! callers should prefer this table; legacy scattered logic stays in place
//! until Phase 3.x completes a full consolidation.
//!
//! Scope: this module is **read-only** (a lookup table). It does not mutate
//! the type checker's state.

#![allow(dead_code)]

use crate::types::ResolvedType;

/// Receiver shape abstraction — matches the *kind* of receiver without
/// caring about the concrete generic arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverShape {
    Vec,
    VecMut,
    HashMap,
    HashMapMut,
    Str,
    StrRef,
    Option,
    Result,
}

impl ReceiverShape {
    /// Classify a `ResolvedType` into a `ReceiverShape` for lookup. Returns
    /// `None` if the type doesn't correspond to a known builtin container.
    pub fn from_type(ty: &ResolvedType) -> Option<Self> {
        match ty {
            ResolvedType::Named { name, .. } => match name.as_str() {
                "Vec" => Some(Self::Vec),
                "VecMut" => Some(Self::VecMut),
                "HashMap" => Some(Self::HashMap),
                "HashMapMut" => Some(Self::HashMapMut),
                "Option" => Some(Self::Option),
                "Result" => Some(Self::Result),
                _ => None,
            },
            ResolvedType::Optional(_) => Some(Self::Option),
            ResolvedType::Result(_, _) => Some(Self::Result),
            ResolvedType::Str => Some(Self::Str),
            ResolvedType::Ref(inner) if matches!(**inner, ResolvedType::Str) => Some(Self::StrRef),
            _ => None,
        }
    }
}

/// The shape of the return type. Some methods return a fixed type (e.g.
/// `Vec.len() -> i64`); others return a type parameterized by the receiver's
/// generic arguments (e.g. `Vec<T>.get(i) -> Option<&T>`).
#[derive(Debug, Clone)]
pub enum ReturnRule {
    /// Return the given concrete type regardless of receiver generics.
    Concrete(ResolvedType),
    /// Return `Option<ReceiverGeneric[0]>`.
    OptionOfFirstGeneric,
    /// Return `Option<&ReceiverGeneric[0]>` — borrow-returning get.
    OptionOfRefFirstGeneric,
    /// Return the receiver's first generic argument directly.
    FirstGeneric,
    /// Method is mutating with no meaningful return value.
    Unit,
}

/// Look up a return rule for `(receiver, method)`. Returns `None` if the
/// combination is not in the table — callers should then fall back to the
/// scattered legacy inference paths.
pub fn lookup_method_return(shape: ReceiverShape, method: &str) -> Option<ReturnRule> {
    let ci64 = ReturnRule::Concrete(ResolvedType::I64);
    let cbool = ReturnRule::Concrete(ResolvedType::Bool);
    let cstr = ReturnRule::Concrete(ResolvedType::Str);
    Some(match (shape, method) {
        // Vec / VecMut
        (ReceiverShape::Vec, "len") | (ReceiverShape::VecMut, "len") => ci64.clone(),
        (ReceiverShape::Vec, "capacity") | (ReceiverShape::VecMut, "capacity") => ci64.clone(),
        (ReceiverShape::Vec, "is_empty") | (ReceiverShape::VecMut, "is_empty") => cbool.clone(),
        (ReceiverShape::Vec, "contains") | (ReceiverShape::VecMut, "contains") => cbool.clone(),
        (ReceiverShape::Vec, "push") | (ReceiverShape::VecMut, "push") => ReturnRule::Unit,
        (ReceiverShape::Vec, "pop") | (ReceiverShape::VecMut, "pop") => ReturnRule::FirstGeneric,
        (ReceiverShape::Vec, "clear") | (ReceiverShape::VecMut, "clear") => ReturnRule::Unit,
        (ReceiverShape::Vec, "get") | (ReceiverShape::VecMut, "get") => {
            ReturnRule::OptionOfRefFirstGeneric
        }
        (ReceiverShape::Vec, "first") | (ReceiverShape::VecMut, "first") => {
            ReturnRule::OptionOfRefFirstGeneric
        }
        (ReceiverShape::Vec, "last") | (ReceiverShape::VecMut, "last") => {
            ReturnRule::OptionOfRefFirstGeneric
        }
        (ReceiverShape::Vec, "reverse") | (ReceiverShape::VecMut, "reverse") => ReturnRule::Unit,
        (ReceiverShape::Vec, "sort") | (ReceiverShape::VecMut, "sort") => ReturnRule::Unit,
        (ReceiverShape::Vec, "extend") | (ReceiverShape::VecMut, "extend") => ReturnRule::Unit,
        (ReceiverShape::Vec, "truncate") | (ReceiverShape::VecMut, "truncate") => ReturnRule::Unit,
        (ReceiverShape::Vec, "swap") | (ReceiverShape::VecMut, "swap") => ReturnRule::Unit,
        // HashMap
        (ReceiverShape::HashMap, "len") | (ReceiverShape::HashMapMut, "len") => ci64,
        (ReceiverShape::HashMap, "is_empty") | (ReceiverShape::HashMapMut, "is_empty") => {
            cbool.clone()
        }
        (ReceiverShape::HashMap, "insert") | (ReceiverShape::HashMapMut, "insert") => {
            ReturnRule::Unit
        }
        (ReceiverShape::HashMap, "contains_key") | (ReceiverShape::HashMapMut, "contains_key") => {
            cbool.clone()
        }
        (ReceiverShape::HashMap, "get") | (ReceiverShape::HashMapMut, "get") => {
            ReturnRule::OptionOfRefFirstGeneric
        }
        (ReceiverShape::HashMap, "remove") | (ReceiverShape::HashMapMut, "remove") => {
            ReturnRule::Unit
        }
        (ReceiverShape::HashMap, "clear") | (ReceiverShape::HashMapMut, "clear") => {
            ReturnRule::Unit
        }
        // Str / &str
        (ReceiverShape::Str, "len") | (ReceiverShape::StrRef, "len") => {
            ReturnRule::Concrete(ResolvedType::I64)
        }
        (ReceiverShape::Str, "is_empty") | (ReceiverShape::StrRef, "is_empty") => cbool.clone(),
        (ReceiverShape::Str, "contains") | (ReceiverShape::StrRef, "contains") => cbool.clone(),
        (ReceiverShape::Str, "starts_with") | (ReceiverShape::StrRef, "starts_with") => {
            cbool.clone()
        }
        (ReceiverShape::Str, "ends_with") | (ReceiverShape::StrRef, "ends_with") => cbool.clone(),
        (ReceiverShape::Str, "to_upper") | (ReceiverShape::StrRef, "to_upper") => cstr.clone(),
        (ReceiverShape::Str, "to_lower") | (ReceiverShape::StrRef, "to_lower") => cstr.clone(),
        (ReceiverShape::Str, "trim") | (ReceiverShape::StrRef, "trim") => cstr,
        // Option
        (ReceiverShape::Option, "is_some") => cbool.clone(),
        (ReceiverShape::Option, "is_none") => cbool.clone(),
        (ReceiverShape::Option, "unwrap") => ReturnRule::FirstGeneric,
        // Result
        (ReceiverShape::Result, "is_ok") => cbool.clone(),
        (ReceiverShape::Result, "is_err") => cbool,
        (ReceiverShape::Result, "unwrap") => ReturnRule::FirstGeneric,
        _ => return None,
    })
}

/// Expand a `ReturnRule` against a concrete receiver type, producing the
/// fully-resolved return type. Returns `None` if the receiver's generics
/// don't match what the rule expects (e.g. `FirstGeneric` on a Vec with
/// no generics).
pub fn expand_return_rule(rule: &ReturnRule, receiver: &ResolvedType) -> Option<ResolvedType> {
    match rule {
        ReturnRule::Concrete(t) => Some(t.clone()),
        ReturnRule::Unit => Some(ResolvedType::Unit),
        ReturnRule::FirstGeneric => first_generic(receiver),
        ReturnRule::OptionOfFirstGeneric => {
            first_generic(receiver).map(|inner| ResolvedType::Optional(Box::new(inner)))
        }
        ReturnRule::OptionOfRefFirstGeneric => first_generic(receiver)
            .map(|inner| ResolvedType::Optional(Box::new(ResolvedType::Ref(Box::new(inner))))),
    }
}

fn first_generic(ty: &ResolvedType) -> Option<ResolvedType> {
    match ty {
        ResolvedType::Named { generics, .. } if !generics.is_empty() => Some(generics[0].clone()),
        ResolvedType::Optional(inner) | ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
            first_generic(inner)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_len_is_concrete_i64() {
        let rule = lookup_method_return(ReceiverShape::Vec, "len").unwrap();
        let receiver = ResolvedType::Named {
            name: "Vec".into(),
            generics: vec![ResolvedType::I64],
        };
        assert_eq!(
            expand_return_rule(&rule, &receiver),
            Some(ResolvedType::I64)
        );
    }

    #[test]
    fn vec_get_returns_option_ref_first() {
        let rule = lookup_method_return(ReceiverShape::Vec, "get").unwrap();
        let receiver = ResolvedType::Named {
            name: "Vec".into(),
            generics: vec![ResolvedType::I64],
        };
        let expected =
            ResolvedType::Optional(Box::new(ResolvedType::Ref(Box::new(ResolvedType::I64))));
        assert_eq!(expand_return_rule(&rule, &receiver), Some(expected));
    }

    #[test]
    fn unknown_method_returns_none() {
        assert!(lookup_method_return(ReceiverShape::Vec, "mystery_method").is_none());
    }

    #[test]
    fn shape_from_named_vec() {
        let ty = ResolvedType::Named {
            name: "Vec".into(),
            generics: vec![ResolvedType::I64],
        };
        assert_eq!(ReceiverShape::from_type(&ty), Some(ReceiverShape::Vec));
    }
}
