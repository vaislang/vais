//! Comprehensive type substitution coverage tests
//!
//! Targets uncovered lines in types/substitute.rs (229 uncovered, 58% coverage)
//! Focus: substitute_type for various ResolvedType variants including
//! Vector, ConstGeneric, ConstArray, HigherKinded, Map, Range, FnPtr,
//! DynTrait, ImplTrait, Associated, Lazy, Linear, Affine, Dependent,
//! RefLifetime, RefMutLifetime

use std::collections::HashMap;
use vais_types::{substitute_type, EffectSet, ResolvedType};

fn make_sub(name: &str, ty: ResolvedType) -> HashMap<String, ResolvedType> {
    let mut map = HashMap::new();
    map.insert(name.to_string(), ty);
    map
}

// ============================================================================
// Basic substitution
// ============================================================================

#[test]
fn test_substitute_generic_found() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Generic("T".to_string());
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::I64);
}

#[test]
fn test_substitute_generic_not_found() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Generic("U".to_string());
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ty);
}

#[test]
fn test_substitute_primitive_unchanged() {
    let subs = make_sub("T", ResolvedType::I64);
    assert_eq!(substitute_type(&ResolvedType::I32, &subs), ResolvedType::I32);
    assert_eq!(substitute_type(&ResolvedType::F64, &subs), ResolvedType::F64);
    assert_eq!(substitute_type(&ResolvedType::Bool, &subs), ResolvedType::Bool);
    assert_eq!(substitute_type(&ResolvedType::Str, &subs), ResolvedType::Str);
    assert_eq!(substitute_type(&ResolvedType::Unit, &subs), ResolvedType::Unit);
    assert_eq!(substitute_type(&ResolvedType::Never, &subs), ResolvedType::Never);
}

// ============================================================================
// Container types
// ============================================================================

#[test]
fn test_substitute_array() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Array(Box::new(ResolvedType::Generic("T".to_string())));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::Array(Box::new(ResolvedType::I64)));
}

#[test]
fn test_substitute_array_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Array(Box::new(ResolvedType::F64));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ty);
}

#[test]
fn test_substitute_optional() {
    let subs = make_sub("T", ResolvedType::Bool);
    let ty = ResolvedType::Optional(Box::new(ResolvedType::Generic("T".to_string())));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::Optional(Box::new(ResolvedType::Bool)));
}

#[test]
fn test_substitute_result() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Result(
        Box::new(ResolvedType::Generic("T".to_string())),
        Box::new(ResolvedType::Str),
    );
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::Result(Box::new(ResolvedType::I64), Box::new(ResolvedType::Str))
    );
}

#[test]
fn test_substitute_tuple() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Tuple(vec![
        ResolvedType::Generic("T".to_string()),
        ResolvedType::Bool,
    ]);
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::Tuple(vec![ResolvedType::I64, ResolvedType::Bool])
    );
}

#[test]
fn test_substitute_named() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Named {
        name: "Vec".to_string(),
        generics: vec![ResolvedType::Generic("T".to_string())],
    };
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::Named {
            name: "Vec".to_string(),
            generics: vec![ResolvedType::I64],
        }
    );
}

#[test]
fn test_substitute_fn() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Fn {
        params: vec![ResolvedType::Generic("T".to_string())],
        ret: Box::new(ResolvedType::Generic("T".to_string())),
        effects: None,
    };
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::Fn {
            params: vec![ResolvedType::I64],
            ret: Box::new(ResolvedType::I64),
            effects: None,
        }
    );
}

// ============================================================================
// Advanced types (uncovered paths)
// ============================================================================

#[test]
fn test_substitute_vector() {
    let subs = make_sub("T", ResolvedType::F32);
    let ty = ResolvedType::Vector {
        element: Box::new(ResolvedType::Generic("T".to_string())),
        lanes: 4,
    };
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::Vector {
            element: Box::new(ResolvedType::F32),
            lanes: 4,
        }
    );
}

#[test]
fn test_substitute_vector_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Vector {
        element: Box::new(ResolvedType::F32),
        lanes: 8,
    };
    assert_eq!(substitute_type(&ty, &subs), ty);
}

#[test]
fn test_substitute_const_generic() {
    let subs = make_sub("N", ResolvedType::I64);
    let ty = ResolvedType::ConstGeneric("N".to_string());
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::I64);
}

#[test]
fn test_substitute_const_generic_not_found() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::ConstGeneric("N".to_string());
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ty);
}

#[test]
fn test_substitute_higher_kinded() {
    let subs = make_sub("F", ResolvedType::Named {
        name: "Vec".to_string(),
        generics: vec![],
    });
    let ty = ResolvedType::HigherKinded {
        name: "F".to_string(),
        arity: 1,
    };
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::Named {
            name: "Vec".to_string(),
            generics: vec![],
        }
    );
}

#[test]
fn test_substitute_higher_kinded_not_found() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::HigherKinded {
        name: "F".to_string(),
        arity: 1,
    };
    assert_eq!(substitute_type(&ty, &subs), ty);
}

#[test]
fn test_substitute_map() {
    let subs = make_sub("K", ResolvedType::Str);
    let ty = ResolvedType::Map(
        Box::new(ResolvedType::Generic("K".to_string())),
        Box::new(ResolvedType::I64),
    );
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::Map(Box::new(ResolvedType::Str), Box::new(ResolvedType::I64))
    );
}

#[test]
fn test_substitute_map_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Map(
        Box::new(ResolvedType::Str),
        Box::new(ResolvedType::Bool),
    );
    assert_eq!(substitute_type(&ty, &subs), ty);
}

#[test]
fn test_substitute_range() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Range(Box::new(ResolvedType::Generic("T".to_string())));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::Range(Box::new(ResolvedType::I64)));
}

#[test]
fn test_substitute_range_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Range(Box::new(ResolvedType::I32));
    assert_eq!(substitute_type(&ty, &subs), ty);
}

#[test]
fn test_substitute_fn_ptr() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::FnPtr {
        params: vec![ResolvedType::Generic("T".to_string())],
        ret: Box::new(ResolvedType::Generic("T".to_string())),
        is_vararg: false,
        effects: Some(Box::new(EffectSet::pure())),
    };
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::FnPtr {
            params: vec![ResolvedType::I64],
            ret: Box::new(ResolvedType::I64),
            is_vararg: false,
            effects: Some(Box::new(EffectSet::pure())),
        }
    );
}

#[test]
fn test_substitute_fn_ptr_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::FnPtr {
        params: vec![ResolvedType::Bool],
        ret: Box::new(ResolvedType::Str),
        is_vararg: true,
        effects: None,
    };
    assert_eq!(substitute_type(&ty, &subs), ty);
}

#[test]
fn test_substitute_dyn_trait() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::DynTrait {
        trait_name: "Iterator".to_string(),
        generics: vec![ResolvedType::Generic("T".to_string())],
    };
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::DynTrait {
            trait_name: "Iterator".to_string(),
            generics: vec![ResolvedType::I64],
        }
    );
}

#[test]
fn test_substitute_dyn_trait_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::DynTrait {
        trait_name: "Display".to_string(),
        generics: vec![],
    };
    assert_eq!(substitute_type(&ty, &subs), ty);
}

#[test]
fn test_substitute_impl_trait() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::ImplTrait {
        bounds: vec!["Display".to_string()],
    };
    // ImplTrait bounds are Strings, no type substitution
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ty);
}

#[test]
fn test_substitute_associated() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("Iterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![],
    };
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::Associated {
            base: Box::new(ResolvedType::I64),
            trait_name: Some("Iterator".to_string()),
            assoc_name: "Item".to_string(),
            generics: vec![],
        }
    );
}

#[test]
fn test_substitute_associated_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Associated {
        base: Box::new(ResolvedType::Str),
        trait_name: Some("Iterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![ResolvedType::Bool],
    };
    assert_eq!(substitute_type(&ty, &subs), ty);
}

#[test]
fn test_substitute_associated_generics_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Associated {
        base: Box::new(ResolvedType::Str),
        trait_name: Some("Trait".to_string()),
        assoc_name: "Assoc".to_string(),
        generics: vec![ResolvedType::Generic("T".to_string())],
    };
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::Associated {
            base: Box::new(ResolvedType::Str),
            trait_name: Some("Trait".to_string()),
            assoc_name: "Assoc".to_string(),
            generics: vec![ResolvedType::I64],
        }
    );
}

#[test]
fn test_substitute_lazy() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Lazy(Box::new(ResolvedType::Generic("T".to_string())));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::Lazy(Box::new(ResolvedType::I64)));
}

#[test]
fn test_substitute_lazy_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Lazy(Box::new(ResolvedType::Bool));
    assert_eq!(substitute_type(&ty, &subs), ty);
}

#[test]
fn test_substitute_linear() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Linear(Box::new(ResolvedType::Generic("T".to_string())));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::Linear(Box::new(ResolvedType::I64)));
}

#[test]
fn test_substitute_linear_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Linear(Box::new(ResolvedType::F32));
    assert_eq!(substitute_type(&ty, &subs), ty);
}

#[test]
fn test_substitute_affine() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Affine(Box::new(ResolvedType::Generic("T".to_string())));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::Affine(Box::new(ResolvedType::I64)));
}

#[test]
fn test_substitute_affine_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Affine(Box::new(ResolvedType::U8));
    assert_eq!(substitute_type(&ty, &subs), ty);
}

#[test]
fn test_substitute_dependent() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Dependent {
        var_name: "n".to_string(),
        base: Box::new(ResolvedType::Generic("T".to_string())),
        predicate: "n > 0".to_string(),
    };
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::Dependent {
            var_name: "n".to_string(),
            base: Box::new(ResolvedType::I64),
            predicate: "n > 0".to_string(),
        }
    );
}

#[test]
fn test_substitute_dependent_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Dependent {
        var_name: "n".to_string(),
        base: Box::new(ResolvedType::I32),
        predicate: "n > 0".to_string(),
    };
    assert_eq!(substitute_type(&ty, &subs), ty);
}

#[test]
fn test_substitute_ref_lifetime() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::RefLifetime {
        lifetime: "'a".to_string(),
        inner: Box::new(ResolvedType::Generic("T".to_string())),
    };
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::RefLifetime {
            lifetime: "'a".to_string(),
            inner: Box::new(ResolvedType::I64),
        }
    );
}

#[test]
fn test_substitute_ref_lifetime_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::RefLifetime {
        lifetime: "'a".to_string(),
        inner: Box::new(ResolvedType::Str),
    };
    assert_eq!(substitute_type(&ty, &subs), ty);
}

#[test]
fn test_substitute_ref_mut_lifetime() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::RefMutLifetime {
        lifetime: "'a".to_string(),
        inner: Box::new(ResolvedType::Generic("T".to_string())),
    };
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::RefMutLifetime {
            lifetime: "'a".to_string(),
            inner: Box::new(ResolvedType::I64),
        }
    );
}

#[test]
fn test_substitute_ref_mut_lifetime_no_change() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::RefMutLifetime {
        lifetime: "'a".to_string(),
        inner: Box::new(ResolvedType::Bool),
    };
    assert_eq!(substitute_type(&ty, &subs), ty);
}

// ============================================================================
// Ref/RefMut/Slice/SliceMut/Pointer/Future
// ============================================================================

#[test]
fn test_substitute_ref() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Ref(Box::new(ResolvedType::Generic("T".to_string())));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::Ref(Box::new(ResolvedType::I64)));
}

#[test]
fn test_substitute_ref_mut() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::RefMut(Box::new(ResolvedType::Generic("T".to_string())));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::RefMut(Box::new(ResolvedType::I64)));
}

#[test]
fn test_substitute_slice() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Slice(Box::new(ResolvedType::Generic("T".to_string())));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::Slice(Box::new(ResolvedType::I64)));
}

#[test]
fn test_substitute_slice_mut() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::SliceMut(Box::new(ResolvedType::Generic("T".to_string())));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::SliceMut(Box::new(ResolvedType::I64)));
}

#[test]
fn test_substitute_pointer() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Pointer(Box::new(ResolvedType::Generic("T".to_string())));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::Pointer(Box::new(ResolvedType::I64)));
}

#[test]
fn test_substitute_future() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Future(Box::new(ResolvedType::Generic("T".to_string())));
    let result = substitute_type(&ty, &subs);
    assert_eq!(result, ResolvedType::Future(Box::new(ResolvedType::I64)));
}

// ============================================================================
// Nested substitution
// ============================================================================

#[test]
fn test_substitute_nested() {
    let subs = make_sub("T", ResolvedType::I64);
    let ty = ResolvedType::Array(Box::new(ResolvedType::Optional(Box::new(
        ResolvedType::Generic("T".to_string()),
    ))));
    let result = substitute_type(&ty, &subs);
    assert_eq!(
        result,
        ResolvedType::Array(Box::new(ResolvedType::Optional(Box::new(ResolvedType::I64))))
    );
}

// ============================================================================
// Empty substitution
// ============================================================================

#[test]
fn test_substitute_empty_map() {
    let subs = HashMap::new();
    let ty = ResolvedType::Generic("T".to_string());
    assert_eq!(substitute_type(&ty, &subs), ty);
}
