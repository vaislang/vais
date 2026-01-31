//! Object safety checking for trait objects
//!
//! This module implements checks to determine which traits can be used as `dyn Trait`.
//! A trait is "object-safe" only if it meets specific criteria that allow vtable-based
//! dynamic dispatch.

use crate::{TraitDef, ResolvedType};

/// Reasons a trait may not be object-safe
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectSafetyViolation {
    /// Method has type parameters (generic method)
    MethodHasTypeParams { method_name: String },

    /// Method returns `Self` type
    MethodReturnsSelf { method_name: String },

    /// Method has no receiver (static method or associated function)
    MethodMissingReceiver { method_name: String },

    /// Trait requires `Self: Sized` bound
    TraitHasSizedBound,

    /// Method uses `Self` in non-receiver parameter position
    MethodUsesSelfInArgs { method_name: String },

    /// Method has `Self: Sized` bound (method-level)
    MethodHasSizedBound { method_name: String },
}

impl ObjectSafetyViolation {
    /// Get a human-readable description of this violation
    pub fn description(&self) -> String {
        match self {
            ObjectSafetyViolation::MethodHasTypeParams { method_name } => {
                format!("method `{}` has type parameters (generic methods are not object-safe)", method_name)
            }
            ObjectSafetyViolation::MethodReturnsSelf { method_name } => {
                format!("method `{}` returns `Self` type (cannot be known at runtime)", method_name)
            }
            ObjectSafetyViolation::MethodMissingReceiver { method_name } => {
                format!("method `{}` has no receiver (static methods are not object-safe)", method_name)
            }
            ObjectSafetyViolation::TraitHasSizedBound => {
                "trait requires `Self: Sized` bound".to_string()
            }
            ObjectSafetyViolation::MethodUsesSelfInArgs { method_name } => {
                format!("method `{}` uses `Self` in parameter position (only receiver can be Self)", method_name)
            }
            ObjectSafetyViolation::MethodHasSizedBound { method_name } => {
                format!("method `{}` has `Self: Sized` bound", method_name)
            }
        }
    }
}

/// Check if a trait is object-safe (can be used as `dyn Trait`)
///
/// # Object Safety Rules
///
/// A trait is object-safe if ALL of the following are true:
/// 1. The trait does NOT require `Self: Sized`
/// 2. ALL methods:
///    - Have a receiver (`self`, `&self`, `&mut self`)
///    - Do NOT have type parameters (no generic methods)
///    - Do NOT return `Self` type
///    - Do NOT use `Self` in non-receiver parameter positions
///    - Do NOT have a `Self: Sized` bound
///
/// # Returns
///
/// - `Ok(())` if the trait is object-safe
/// - `Err(Vec<ObjectSafetyViolation>)` with all violations found
///
/// # Examples
///
/// Object-safe trait:
/// ```vais
/// W Drawable {
///     F draw(&self) -> i64
///     F get_color(&self) -> i64
/// }
/// ```
///
/// NOT object-safe (generic method):
/// ```vais
/// W Container {
///     F get<T>(&self) -> T  // Generic method!
/// }
/// ```
pub fn check_object_safety(trait_def: &TraitDef) -> Result<(), Vec<ObjectSafetyViolation>> {
    let mut violations = Vec::new();

    // Check 1: Trait must not have Self: Sized bound
    // Note: In current implementation, we check super_traits for "Sized"
    if trait_def.super_traits.contains(&"Sized".to_string()) {
        violations.push(ObjectSafetyViolation::TraitHasSizedBound);
    }

    // Check each method for object safety
    for (method_name, method_sig) in &trait_def.methods {
        // Check 2: Method must have a receiver (self parameter)
        if method_sig.params.is_empty() {
            violations.push(ObjectSafetyViolation::MethodMissingReceiver {
                method_name: method_name.clone(),
            });
            continue;
        }

        // First parameter must be some form of self
        let (first_param_name, first_param_type, _) = &method_sig.params[0];
        let has_receiver = first_param_name == "self" ||
                          is_self_receiver_type(first_param_type);

        if !has_receiver {
            violations.push(ObjectSafetyViolation::MethodMissingReceiver {
                method_name: method_name.clone(),
            });
        }

        // Check 3: Method must not return Self
        if contains_self_type(&method_sig.ret) {
            violations.push(ObjectSafetyViolation::MethodReturnsSelf {
                method_name: method_name.clone(),
            });
        }

        // Check 4: Non-receiver parameters must not use Self
        for (_, param_type, _) in method_sig.params.iter().skip(1) {
            if contains_self_type(param_type) {
                violations.push(ObjectSafetyViolation::MethodUsesSelfInArgs {
                    method_name: method_name.clone(),
                });
                break;
            }
        }

        // Note: Check 5 (generic methods) is not yet implemented because the current
        // TraitMethodSig doesn't store generic parameters. When trait methods gain
        // generic support, we would check here:
        // if !method_sig.generics.is_empty() {
        //     violations.push(ObjectSafetyViolation::MethodHasTypeParams { ... });
        // }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Check if a type is a valid receiver type (Self, &Self, &mut Self, etc.)
fn is_self_receiver_type(ty: &ResolvedType) -> bool {
    match ty {
        ResolvedType::Generic(name) if name == "Self" => true,
        ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
            matches!(inner.as_ref(), ResolvedType::Generic(name) if name == "Self")
        }
        ResolvedType::Pointer(inner) => {
            matches!(inner.as_ref(), ResolvedType::Generic(name) if name == "Self")
        }
        _ => false,
    }
}

/// Check if a type contains the `Self` type anywhere in its structure
fn contains_self_type(ty: &ResolvedType) -> bool {
    match ty {
        ResolvedType::Generic(name) if name == "Self" => true,

        // Recursive checks for compound types
        ResolvedType::Ref(inner) |
        ResolvedType::RefMut(inner) |
        ResolvedType::Pointer(inner) |
        ResolvedType::Array(inner) |
        ResolvedType::Optional(inner) |
        ResolvedType::Result(inner) |
        ResolvedType::Future(inner) |
        ResolvedType::Range(inner) => contains_self_type(inner),

        ResolvedType::ConstArray { element, .. } => contains_self_type(element),

        ResolvedType::Map(k, v) => contains_self_type(k) || contains_self_type(v),

        ResolvedType::Tuple(types) => types.iter().any(contains_self_type),

        ResolvedType::Fn { params, ret, .. } |
        ResolvedType::FnPtr { params, ret, .. } => {
            params.iter().any(contains_self_type) || contains_self_type(ret)
        }

        ResolvedType::Named { generics, .. } => {
            generics.iter().any(contains_self_type)
        }

        ResolvedType::DynTrait { generics, .. } => {
            generics.iter().any(contains_self_type)
        }

        ResolvedType::Associated { base, generics, .. } => {
            contains_self_type(base) || generics.iter().any(contains_self_type)
        }

        // Primitives and simple types don't contain Self
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TraitMethodSig;
    use std::collections::HashMap;

    fn create_method(
        name: &str,
        params: Vec<(&str, ResolvedType)>,
        ret: ResolvedType,
    ) -> (String, TraitMethodSig) {
        (
            name.to_string(),
            TraitMethodSig {
                name: name.to_string(),
                params: params
                    .into_iter()
                    .map(|(n, t)| (n.to_string(), t, false))
                    .collect(),
                ret,
                has_default: false,
                is_async: false,
                is_const: false,
            },
        )
    }

    #[test]
    fn test_object_safe_basic_trait() {
        let mut methods = HashMap::new();
        methods.insert(
            "draw".to_string(),
            TraitMethodSig {
                name: "draw".to_string(),
                params: vec![(
                    "self".to_string(),
                    ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
                    false,
                )],
                ret: ResolvedType::I64,
                has_default: false,
                is_async: false,
                is_const: false,
            },
        );

        let trait_def = TraitDef {
            name: "Drawable".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods,
        };

        assert!(
            check_object_safety(&trait_def).is_ok(),
            "Basic trait with &self should be object-safe"
        );
    }

    #[test]
    fn test_not_object_safe_returns_self() {
        let mut methods = HashMap::new();
        methods.insert(
            "clone".to_string(),
            TraitMethodSig {
                name: "clone".to_string(),
                params: vec![(
                    "self".to_string(),
                    ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
                    false,
                )],
                ret: ResolvedType::Generic("Self".to_string()), // Returns Self!
                has_default: false,
                is_async: false,
                is_const: false,
            },
        );

        let trait_def = TraitDef {
            name: "Clone".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods,
        };

        let result = check_object_safety(&trait_def);
        assert!(result.is_err(), "Trait returning Self should not be object-safe");

        let violations = result.unwrap_err();
        assert!(violations.iter().any(|v| matches!(
            v,
            ObjectSafetyViolation::MethodReturnsSelf { method_name } if method_name == "clone"
        )));
    }

    #[test]
    fn test_not_object_safe_no_receiver() {
        let mut methods = HashMap::new();
        methods.insert(
            "new".to_string(),
            TraitMethodSig {
                name: "new".to_string(),
                params: vec![], // No self parameter!
                ret: ResolvedType::Generic("Self".to_string()),
                has_default: false,
                is_async: false,
                is_const: false,
            },
        );

        let trait_def = TraitDef {
            name: "Constructor".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods,
        };

        let result = check_object_safety(&trait_def);
        assert!(result.is_err(), "Static method should not be object-safe");

        let violations = result.unwrap_err();
        assert!(violations.iter().any(|v| matches!(
            v,
            ObjectSafetyViolation::MethodMissingReceiver { method_name } if method_name == "new"
        )));
    }

    #[test]
    fn test_not_object_safe_self_in_params() {
        let mut methods = HashMap::new();
        methods.insert(
            "merge".to_string(),
            TraitMethodSig {
                name: "merge".to_string(),
                params: vec![
                    (
                        "self".to_string(),
                        ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
                        false,
                    ),
                    (
                        "other".to_string(),
                        ResolvedType::Generic("Self".to_string()), // Self in parameter!
                        false,
                    ),
                ],
                ret: ResolvedType::Unit,
                has_default: false,
                is_async: false,
                is_const: false,
            },
        );

        let trait_def = TraitDef {
            name: "Mergeable".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods,
        };

        let result = check_object_safety(&trait_def);
        assert!(
            result.is_err(),
            "Method with Self in params should not be object-safe"
        );

        let violations = result.unwrap_err();
        assert!(violations.iter().any(|v| matches!(
            v,
            ObjectSafetyViolation::MethodUsesSelfInArgs { method_name } if method_name == "merge"
        )));
    }

    #[test]
    fn test_not_object_safe_sized_bound() {
        let methods = HashMap::new();

        let trait_def = TraitDef {
            name: "SizedTrait".to_string(),
            generics: vec![],
            super_traits: vec!["Sized".to_string()], // Sized bound!
            associated_types: HashMap::new(),
            methods,
        };

        let result = check_object_safety(&trait_def);
        assert!(result.is_err(), "Trait with Sized bound should not be object-safe");

        let violations = result.unwrap_err();
        assert!(violations
            .iter()
            .any(|v| matches!(v, ObjectSafetyViolation::TraitHasSizedBound)));
    }

    #[test]
    fn test_object_safe_empty_trait() {
        let trait_def = TraitDef {
            name: "Marker".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods: HashMap::new(),
        };

        assert!(
            check_object_safety(&trait_def).is_ok(),
            "Empty trait should be object-safe"
        );
    }

    #[test]
    fn test_object_safe_multiple_methods() {
        let mut methods = HashMap::new();

        let (name1, sig1) = create_method(
            "draw",
            vec![(
                "self",
                ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
            )],
            ResolvedType::Unit,
        );
        methods.insert(name1, sig1);

        let (name2, sig2) = create_method(
            "area",
            vec![(
                "self",
                ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
            )],
            ResolvedType::F64,
        );
        methods.insert(name2, sig2);

        let trait_def = TraitDef {
            name: "Shape".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods,
        };

        assert!(
            check_object_safety(&trait_def).is_ok(),
            "Trait with multiple safe methods should be object-safe"
        );
    }

    #[test]
    fn test_mixed_safe_and_unsafe_methods() {
        let mut methods = HashMap::new();

        // Safe method
        methods.insert(
            "get".to_string(),
            TraitMethodSig {
                name: "get".to_string(),
                params: vec![(
                    "self".to_string(),
                    ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
                    false,
                )],
                ret: ResolvedType::I64,
                has_default: false,
                is_async: false,
                is_const: false,
            },
        );

        // Unsafe method (returns Self)
        methods.insert(
            "clone".to_string(),
            TraitMethodSig {
                name: "clone".to_string(),
                params: vec![(
                    "self".to_string(),
                    ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
                    false,
                )],
                ret: ResolvedType::Generic("Self".to_string()),
                has_default: false,
                is_async: false,
                is_const: false,
            },
        );

        let trait_def = TraitDef {
            name: "MixedTrait".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods,
        };

        let result = check_object_safety(&trait_def);
        assert!(
            result.is_err(),
            "Trait with any unsafe method should not be object-safe"
        );
    }

    #[test]
    fn test_contains_self_type() {
        assert!(contains_self_type(&ResolvedType::Generic("Self".to_string())));

        assert!(contains_self_type(&ResolvedType::Ref(Box::new(
            ResolvedType::Generic("Self".to_string())
        ))));

        assert!(contains_self_type(&ResolvedType::Optional(Box::new(
            ResolvedType::Generic("Self".to_string())
        ))));

        assert!(contains_self_type(&ResolvedType::Tuple(vec![
            ResolvedType::I64,
            ResolvedType::Generic("Self".to_string()),
        ])));

        assert!(!contains_self_type(&ResolvedType::I64));
        assert!(!contains_self_type(&ResolvedType::Str));
    }

    #[test]
    fn test_violation_descriptions() {
        let v1 = ObjectSafetyViolation::MethodHasTypeParams {
            method_name: "foo".to_string(),
        };
        assert!(v1.description().contains("foo"));
        assert!(v1.description().contains("type parameters"));

        let v2 = ObjectSafetyViolation::MethodReturnsSelf {
            method_name: "bar".to_string(),
        };
        assert!(v2.description().contains("bar"));
        assert!(v2.description().contains("returns `Self`"));

        let v3 = ObjectSafetyViolation::TraitHasSizedBound;
        assert!(v3.description().contains("Sized"));
    }
}
