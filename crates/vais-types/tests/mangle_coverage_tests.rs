//! Mangle coverage tests
//!
//! Targets uncovered lines in types/mangle.rs (124 uncovered)

use vais_types::{mangle_name, mangle_type, ResolvedType};
use vais_types::types::mangle_name_with_consts;

// ============================================================================
// mangle_name
// ============================================================================

#[test]
fn test_mangle_name_no_args() {
    assert_eq!(mangle_name("foo", &[]), "foo");
}

#[test]
fn test_mangle_name_single_i64() {
    assert_eq!(mangle_name("foo", &[ResolvedType::I64]), "foo$i64");
}

#[test]
fn test_mangle_name_two_args() {
    assert_eq!(
        mangle_name("foo", &[ResolvedType::I64, ResolvedType::Bool]),
        "foo$i64_bool"
    );
}

#[test]
fn test_mangle_name_three_args() {
    assert_eq!(
        mangle_name("bar", &[ResolvedType::F32, ResolvedType::Str, ResolvedType::U8]),
        "bar$f32_str_u8"
    );
}

// ============================================================================
// mangle_name_with_consts
// ============================================================================

#[test]
fn test_mangle_name_with_consts_empty() {
    assert_eq!(mangle_name_with_consts("foo", &[], &[]), "foo");
}

#[test]
fn test_mangle_name_with_consts_types_only() {
    assert_eq!(
        mangle_name_with_consts("foo", &[ResolvedType::I64], &[]),
        "foo$i64"
    );
}

#[test]
fn test_mangle_name_with_consts_consts_only() {
    assert_eq!(
        mangle_name_with_consts("foo", &[], &[("N".to_string(), 42)]),
        "foo$c42"
    );
}

#[test]
fn test_mangle_name_with_consts_both() {
    assert_eq!(
        mangle_name_with_consts(
            "foo",
            &[ResolvedType::I64],
            &[("N".to_string(), 10)]
        ),
        "foo$i64_c10"
    );
}

#[test]
fn test_mangle_name_with_consts_multiple_consts() {
    assert_eq!(
        mangle_name_with_consts(
            "bar",
            &[],
            &[("A".to_string(), 1), ("B".to_string(), 2)]
        ),
        "bar$c1_c2"
    );
}

// ============================================================================
// mangle_type - primitives
// ============================================================================

#[test]
fn test_mangle_type_i8() {
    assert_eq!(mangle_type(&ResolvedType::I8), "i8");
}

#[test]
fn test_mangle_type_i16() {
    assert_eq!(mangle_type(&ResolvedType::I16), "i16");
}

#[test]
fn test_mangle_type_i32() {
    assert_eq!(mangle_type(&ResolvedType::I32), "i32");
}

#[test]
fn test_mangle_type_i64() {
    assert_eq!(mangle_type(&ResolvedType::I64), "i64");
}

#[test]
fn test_mangle_type_i128() {
    assert_eq!(mangle_type(&ResolvedType::I128), "i128");
}

#[test]
fn test_mangle_type_u8() {
    assert_eq!(mangle_type(&ResolvedType::U8), "u8");
}

#[test]
fn test_mangle_type_u16() {
    assert_eq!(mangle_type(&ResolvedType::U16), "u16");
}

#[test]
fn test_mangle_type_u32() {
    assert_eq!(mangle_type(&ResolvedType::U32), "u32");
}

#[test]
fn test_mangle_type_u64() {
    assert_eq!(mangle_type(&ResolvedType::U64), "u64");
}

#[test]
fn test_mangle_type_u128() {
    assert_eq!(mangle_type(&ResolvedType::U128), "u128");
}

#[test]
fn test_mangle_type_f32() {
    assert_eq!(mangle_type(&ResolvedType::F32), "f32");
}

#[test]
fn test_mangle_type_f64() {
    assert_eq!(mangle_type(&ResolvedType::F64), "f64");
}

#[test]
fn test_mangle_type_bool() {
    assert_eq!(mangle_type(&ResolvedType::Bool), "bool");
}

#[test]
fn test_mangle_type_str() {
    assert_eq!(mangle_type(&ResolvedType::Str), "str");
}

#[test]
fn test_mangle_type_unit() {
    assert_eq!(mangle_type(&ResolvedType::Unit), "unit");
}

// ============================================================================
// mangle_type - compound types
// ============================================================================

#[test]
fn test_mangle_type_named_no_generics() {
    let ty = ResolvedType::Named {
        name: "Point".to_string(),
        generics: vec![],
    };
    assert_eq!(mangle_type(&ty), "Point");
}

#[test]
fn test_mangle_type_named_with_generics() {
    let ty = ResolvedType::Named {
        name: "Vec".to_string(),
        generics: vec![ResolvedType::I64],
    };
    assert_eq!(mangle_type(&ty), "Vec_i64");
}

#[test]
fn test_mangle_type_named_multi_generics() {
    let ty = ResolvedType::Named {
        name: "HashMap".to_string(),
        generics: vec![ResolvedType::Str, ResolvedType::I64],
    };
    assert_eq!(mangle_type(&ty), "HashMap_str_i64");
}

#[test]
fn test_mangle_type_array() {
    assert_eq!(
        mangle_type(&ResolvedType::Array(Box::new(ResolvedType::I64))),
        "arr_i64"
    );
}

#[test]
fn test_mangle_type_pointer() {
    assert_eq!(
        mangle_type(&ResolvedType::Pointer(Box::new(ResolvedType::U8))),
        "ptr_u8"
    );
}

#[test]
fn test_mangle_type_ref() {
    assert_eq!(
        mangle_type(&ResolvedType::Ref(Box::new(ResolvedType::I64))),
        "ref_i64"
    );
}

#[test]
fn test_mangle_type_ref_mut() {
    assert_eq!(
        mangle_type(&ResolvedType::RefMut(Box::new(ResolvedType::I64))),
        "refmut_i64"
    );
}

#[test]
fn test_mangle_type_slice() {
    assert_eq!(
        mangle_type(&ResolvedType::Slice(Box::new(ResolvedType::F64))),
        "slice_f64"
    );
}

#[test]
fn test_mangle_type_slice_mut() {
    assert_eq!(
        mangle_type(&ResolvedType::SliceMut(Box::new(ResolvedType::F64))),
        "slicemut_f64"
    );
}

#[test]
fn test_mangle_type_optional() {
    assert_eq!(
        mangle_type(&ResolvedType::Optional(Box::new(ResolvedType::Str))),
        "opt_str"
    );
}

#[test]
fn test_mangle_type_result() {
    assert_eq!(
        mangle_type(&ResolvedType::Result(
            Box::new(ResolvedType::I64),
            Box::new(ResolvedType::Str)
        )),
        "res_i64_str"
    );
}

#[test]
fn test_mangle_type_future() {
    assert_eq!(
        mangle_type(&ResolvedType::Future(Box::new(ResolvedType::I64))),
        "fut_i64"
    );
}

#[test]
fn test_mangle_type_tuple() {
    assert_eq!(
        mangle_type(&ResolvedType::Tuple(vec![
            ResolvedType::I64,
            ResolvedType::Bool,
            ResolvedType::Str
        ])),
        "tup_i64_bool_str"
    );
}

#[test]
fn test_mangle_type_tuple_single() {
    assert_eq!(
        mangle_type(&ResolvedType::Tuple(vec![ResolvedType::I64])),
        "tup_i64"
    );
}

#[test]
fn test_mangle_type_fn() {
    let ty = ResolvedType::Fn {
        params: vec![ResolvedType::I64, ResolvedType::Bool],
        ret: Box::new(ResolvedType::Str),
        effects: None,
    };
    assert_eq!(mangle_type(&ty), "fn_i64_bool_str");
}

#[test]
fn test_mangle_type_fn_no_params() {
    let ty = ResolvedType::Fn {
        params: vec![],
        ret: Box::new(ResolvedType::Unit),
        effects: None,
    };
    assert_eq!(mangle_type(&ty), "fn__unit");
}

#[test]
fn test_mangle_type_generic() {
    assert_eq!(
        mangle_type(&ResolvedType::Generic("T".to_string())),
        "T"
    );
}

#[test]
fn test_mangle_type_const_generic() {
    assert_eq!(
        mangle_type(&ResolvedType::ConstGeneric("N".to_string())),
        "cg_N"
    );
}

#[test]
fn test_mangle_type_var() {
    assert_eq!(mangle_type(&ResolvedType::Var(42)), "v42");
}

#[test]
fn test_mangle_type_vector() {
    let ty = ResolvedType::Vector {
        element: Box::new(ResolvedType::F32),
        lanes: 4,
    };
    assert_eq!(mangle_type(&ty), "vec4_f32");
}

#[test]
fn test_mangle_type_higher_kinded() {
    let ty = ResolvedType::HigherKinded {
        name: "Functor".to_string(),
        arity: 1,
    };
    assert_eq!(mangle_type(&ty), "hkt1_Functor");
}

#[test]
fn test_mangle_type_unknown_fallback() {
    // Types not explicitly handled should produce "unknown"
    assert_eq!(mangle_type(&ResolvedType::Unknown), "unknown");
    assert_eq!(mangle_type(&ResolvedType::Never), "unknown");
}

// ============================================================================
// Nested/complex mangling
// ============================================================================

#[test]
fn test_mangle_type_nested_array_of_optional() {
    let ty = ResolvedType::Array(Box::new(ResolvedType::Optional(Box::new(
        ResolvedType::I64,
    ))));
    assert_eq!(mangle_type(&ty), "arr_opt_i64");
}

#[test]
fn test_mangle_type_result_of_generics() {
    let ty = ResolvedType::Result(
        Box::new(ResolvedType::Named {
            name: "Vec".to_string(),
            generics: vec![ResolvedType::I64],
        }),
        Box::new(ResolvedType::Str),
    );
    assert_eq!(mangle_type(&ty), "res_Vec_i64_str");
}

#[test]
fn test_mangle_name_nested_types() {
    let ty = ResolvedType::Fn {
        params: vec![ResolvedType::Ref(Box::new(ResolvedType::Array(
            Box::new(ResolvedType::I64),
        )))],
        ret: Box::new(ResolvedType::Optional(Box::new(ResolvedType::Bool))),
        effects: None,
    };
    assert_eq!(
        mangle_name("process", &[ty]),
        "process$fn_ref_arr_i64_opt_bool"
    );
}
