//! ResolvedType coverage tests
//!
//! Targets uncovered lines in types/resolved.rs (67 uncovered)
//! Focus: Display impl, ResolvedConst, helper methods

use vais_types::{ConstBinOp, ResolvedConst, ResolvedType};

// ============================================================================
// Display impl for all ResolvedType variants
// ============================================================================

#[test]
fn test_display_primitives() {
    assert_eq!(format!("{}", ResolvedType::I8), "i8");
    assert_eq!(format!("{}", ResolvedType::I16), "i16");
    assert_eq!(format!("{}", ResolvedType::I32), "i32");
    assert_eq!(format!("{}", ResolvedType::I64), "i64");
    assert_eq!(format!("{}", ResolvedType::I128), "i128");
    assert_eq!(format!("{}", ResolvedType::U8), "u8");
    assert_eq!(format!("{}", ResolvedType::U16), "u16");
    assert_eq!(format!("{}", ResolvedType::U32), "u32");
    assert_eq!(format!("{}", ResolvedType::U64), "u64");
    assert_eq!(format!("{}", ResolvedType::U128), "u128");
    assert_eq!(format!("{}", ResolvedType::F32), "f32");
    assert_eq!(format!("{}", ResolvedType::F64), "f64");
    assert_eq!(format!("{}", ResolvedType::Bool), "bool");
    assert_eq!(format!("{}", ResolvedType::Str), "str");
    assert_eq!(format!("{}", ResolvedType::Unit), "()");
}

#[test]
fn test_display_special() {
    assert_eq!(format!("{}", ResolvedType::Unknown), "?");
    assert_eq!(format!("{}", ResolvedType::Never), "!");
    // Infer is in AST Type, not ResolvedType
}

#[test]
fn test_display_generic() {
    assert_eq!(
        format!("{}", ResolvedType::Generic("T".to_string())),
        "T"
    );
}

#[test]
fn test_display_const_generic() {
    assert_eq!(
        format!("{}", ResolvedType::ConstGeneric("N".to_string())),
        "const N"
    );
}

#[test]
fn test_display_var() {
    assert_eq!(format!("{}", ResolvedType::Var(42)), "?42");
}

#[test]
fn test_display_array() {
    let ty = ResolvedType::Array(Box::new(ResolvedType::I64));
    assert_eq!(format!("{}", ty), "[i64]");
}

#[test]
fn test_display_pointer() {
    let ty = ResolvedType::Pointer(Box::new(ResolvedType::U8));
    assert_eq!(format!("{}", ty), "*u8");
}

#[test]
fn test_display_ref() {
    let ty = ResolvedType::Ref(Box::new(ResolvedType::I64));
    assert_eq!(format!("{}", ty), "&i64");
}

#[test]
fn test_display_ref_mut() {
    let ty = ResolvedType::RefMut(Box::new(ResolvedType::I64));
    assert_eq!(format!("{}", ty), "&mut i64");
}

#[test]
fn test_display_slice() {
    let ty = ResolvedType::Slice(Box::new(ResolvedType::F64));
    assert_eq!(format!("{}", ty), "&[f64]");
}

#[test]
fn test_display_slice_mut() {
    let ty = ResolvedType::SliceMut(Box::new(ResolvedType::F64));
    assert_eq!(format!("{}", ty), "&mut [f64]");
}

#[test]
fn test_display_optional() {
    let ty = ResolvedType::Optional(Box::new(ResolvedType::I64));
    assert_eq!(format!("{}", ty), "i64?");
}

#[test]
fn test_display_result() {
    let ty = ResolvedType::Result(
        Box::new(ResolvedType::I64),
        Box::new(ResolvedType::Str),
    );
    assert_eq!(format!("{}", ty), "Result<i64, str>");
}

#[test]
fn test_display_tuple() {
    let ty = ResolvedType::Tuple(vec![ResolvedType::I64, ResolvedType::Bool]);
    assert_eq!(format!("{}", ty), "(i64,bool)");
}

#[test]
fn test_display_tuple_single() {
    let ty = ResolvedType::Tuple(vec![ResolvedType::I64]);
    let s = format!("{}", ty);
    assert!(s.contains("i64"));
}

#[test]
fn test_display_named_no_generics() {
    let ty = ResolvedType::Named {
        name: "Point".to_string(),
        generics: vec![],
    };
    assert_eq!(format!("{}", ty), "Point");
}

#[test]
fn test_display_named_with_generics() {
    let ty = ResolvedType::Named {
        name: "Vec".to_string(),
        generics: vec![ResolvedType::I64],
    };
    assert_eq!(format!("{}", ty), "Vec<i64>");
}

#[test]
fn test_display_named_multi_generics() {
    let ty = ResolvedType::Named {
        name: "HashMap".to_string(),
        generics: vec![ResolvedType::Str, ResolvedType::I64],
    };
    assert_eq!(format!("{}", ty), "HashMap<str,i64>");
}

#[test]
fn test_display_fn() {
    let ty = ResolvedType::Fn {
        params: vec![ResolvedType::I64, ResolvedType::Bool],
        ret: Box::new(ResolvedType::Str),
        effects: None,
    };
    assert_eq!(format!("{}", ty), "(i64,bool)->str");
}

#[test]
fn test_display_fn_no_params() {
    let ty = ResolvedType::Fn {
        params: vec![],
        ret: Box::new(ResolvedType::Unit),
        effects: None,
    };
    assert_eq!(format!("{}", ty), "()->()");
}

#[test]
fn test_display_fn_ptr() {
    let ty = ResolvedType::FnPtr {
        params: vec![ResolvedType::I64],
        ret: Box::new(ResolvedType::Bool),
        is_vararg: false,
        effects: None,
    };
    assert_eq!(format!("{}", ty), "fn(i64)->bool");
}

#[test]
fn test_display_fn_ptr_vararg() {
    let ty = ResolvedType::FnPtr {
        params: vec![ResolvedType::I64],
        ret: Box::new(ResolvedType::Bool),
        is_vararg: true,
        effects: None,
    };
    assert_eq!(format!("{}", ty), "fn(i64,...)->bool");
}

#[test]
fn test_display_future() {
    let ty = ResolvedType::Future(Box::new(ResolvedType::I64));
    assert_eq!(format!("{}", ty), "Future<i64>");
}

#[test]
fn test_display_lazy() {
    let ty = ResolvedType::Lazy(Box::new(ResolvedType::I64));
    assert_eq!(format!("{}", ty), "Lazy<i64>");
}

#[test]
fn test_display_vector() {
    let ty = ResolvedType::Vector {
        element: Box::new(ResolvedType::F32),
        lanes: 4,
    };
    assert_eq!(format!("{}", ty), "Vec4xf32");
}

#[test]
fn test_display_dyn_trait() {
    let ty = ResolvedType::DynTrait {
        trait_name: "Display".to_string(),
        generics: vec![],
    };
    assert_eq!(format!("{}", ty), "dyn Display");
}

#[test]
fn test_display_dyn_trait_with_generics() {
    let ty = ResolvedType::DynTrait {
        trait_name: "Iterator".to_string(),
        generics: vec![ResolvedType::I64],
    };
    assert_eq!(format!("{}", ty), "dyn Iterator<i64>");
}

#[test]
fn test_display_impl_trait() {
    let ty = ResolvedType::ImplTrait {
        bounds: vec!["Display".to_string(), "Debug".to_string()],
    };
    assert_eq!(format!("{}", ty), "impl Display + Debug");
}

#[test]
fn test_display_impl_trait_single_bound() {
    let ty = ResolvedType::ImplTrait {
        bounds: vec!["Clone".to_string()],
    };
    assert_eq!(format!("{}", ty), "impl Clone");
}

#[test]
fn test_display_associated() {
    let ty = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("Iterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![],
    };
    assert_eq!(format!("{}", ty), "<T as Iterator>::Item");
}

#[test]
fn test_display_associated_no_trait() {
    let ty = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("Self".to_string())),
        trait_name: None,
        assoc_name: "Output".to_string(),
        generics: vec![],
    };
    assert_eq!(format!("{}", ty), "Self::Output");
}

#[test]
fn test_display_associated_with_generics() {
    let ty = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("Trait".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![ResolvedType::I64],
    };
    assert_eq!(format!("{}", ty), "<T as Trait>::Item<i64>");
}

#[test]
fn test_display_linear() {
    let ty = ResolvedType::Linear(Box::new(ResolvedType::I64));
    assert_eq!(format!("{}", ty), "linear i64");
}

#[test]
fn test_display_affine() {
    let ty = ResolvedType::Affine(Box::new(ResolvedType::Str));
    assert_eq!(format!("{}", ty), "affine str");
}

#[test]
fn test_display_higher_kinded() {
    let ty = ResolvedType::HigherKinded {
        name: "F".to_string(),
        arity: 1,
    };
    let s = format!("{}", ty);
    assert!(s.contains("F"));
}

// ============================================================================
// ResolvedConst
// ============================================================================

#[test]
fn test_resolved_const_value() {
    let c = ResolvedConst::Value(42);
    assert_eq!(c.try_evaluate(), Some(42));
}

#[test]
fn test_resolved_const_param() {
    let c = ResolvedConst::Param("N".to_string());
    assert_eq!(c.try_evaluate(), None);
}

#[test]
fn test_resolved_const_binop_add() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Add,
        left: Box::new(ResolvedConst::Value(10)),
        right: Box::new(ResolvedConst::Value(20)),
    };
    assert_eq!(c.try_evaluate(), Some(30));
}

#[test]
fn test_resolved_const_binop_sub() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Sub,
        left: Box::new(ResolvedConst::Value(30)),
        right: Box::new(ResolvedConst::Value(10)),
    };
    assert_eq!(c.try_evaluate(), Some(20));
}

#[test]
fn test_resolved_const_binop_mul() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Mul,
        left: Box::new(ResolvedConst::Value(5)),
        right: Box::new(ResolvedConst::Value(6)),
    };
    assert_eq!(c.try_evaluate(), Some(30));
}

#[test]
fn test_resolved_const_binop_div() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Div,
        left: Box::new(ResolvedConst::Value(20)),
        right: Box::new(ResolvedConst::Value(4)),
    };
    assert_eq!(c.try_evaluate(), Some(5));
}

#[test]
fn test_resolved_const_binop_div_by_zero() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Div,
        left: Box::new(ResolvedConst::Value(10)),
        right: Box::new(ResolvedConst::Value(0)),
    };
    assert_eq!(c.try_evaluate(), None);
}

#[test]
fn test_resolved_const_binop_mod() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Mod,
        left: Box::new(ResolvedConst::Value(17)),
        right: Box::new(ResolvedConst::Value(5)),
    };
    assert_eq!(c.try_evaluate(), Some(2));
}

#[test]
fn test_resolved_const_binop_mod_by_zero() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Mod,
        left: Box::new(ResolvedConst::Value(10)),
        right: Box::new(ResolvedConst::Value(0)),
    };
    assert_eq!(c.try_evaluate(), None);
}

#[test]
fn test_resolved_const_binop_with_param() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Add,
        left: Box::new(ResolvedConst::Param("N".to_string())),
        right: Box::new(ResolvedConst::Value(1)),
    };
    assert_eq!(c.try_evaluate(), None);
}

#[test]
fn test_resolved_const_negate() {
    let c = ResolvedConst::Negate(Box::new(ResolvedConst::Value(5)));
    assert_eq!(c.try_evaluate(), Some(-5));
}

#[test]
fn test_resolved_const_negate_param() {
    let c = ResolvedConst::Negate(Box::new(ResolvedConst::Param("N".to_string())));
    assert_eq!(c.try_evaluate(), None);
}

#[test]
fn test_resolved_const_nested() {
    // (3 + 4) * 2 = 14
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Mul,
        left: Box::new(ResolvedConst::BinOp {
            op: ConstBinOp::Add,
            left: Box::new(ResolvedConst::Value(3)),
            right: Box::new(ResolvedConst::Value(4)),
        }),
        right: Box::new(ResolvedConst::Value(2)),
    };
    assert_eq!(c.try_evaluate(), Some(14));
}

// ============================================================================
// ResolvedConst bitwise ops
// ============================================================================

#[test]
fn test_resolved_const_binop_bitand() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::BitAnd,
        left: Box::new(ResolvedConst::Value(0xFF)),
        right: Box::new(ResolvedConst::Value(0x0F)),
    };
    assert_eq!(c.try_evaluate(), Some(0x0F));
}

#[test]
fn test_resolved_const_binop_bitor() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::BitOr,
        left: Box::new(ResolvedConst::Value(0xF0)),
        right: Box::new(ResolvedConst::Value(0x0F)),
    };
    assert_eq!(c.try_evaluate(), Some(0xFF));
}

#[test]
fn test_resolved_const_binop_bitxor() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::BitXor,
        left: Box::new(ResolvedConst::Value(0xFF)),
        right: Box::new(ResolvedConst::Value(0x0F)),
    };
    assert_eq!(c.try_evaluate(), Some(0xF0));
}

#[test]
fn test_resolved_const_binop_shl() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Shl,
        left: Box::new(ResolvedConst::Value(1)),
        right: Box::new(ResolvedConst::Value(4)),
    };
    assert_eq!(c.try_evaluate(), Some(16));
}

#[test]
fn test_resolved_const_binop_shr() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Shr,
        left: Box::new(ResolvedConst::Value(256)),
        right: Box::new(ResolvedConst::Value(4)),
    };
    assert_eq!(c.try_evaluate(), Some(16));
}

#[test]
fn test_resolved_const_binop_shl_overflow() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Shl,
        left: Box::new(ResolvedConst::Value(1)),
        right: Box::new(ResolvedConst::Value(64)),
    };
    assert_eq!(c.try_evaluate(), None);
}

#[test]
fn test_resolved_const_binop_shr_overflow() {
    let c = ResolvedConst::BinOp {
        op: ConstBinOp::Shr,
        left: Box::new(ResolvedConst::Value(1)),
        right: Box::new(ResolvedConst::Value(64)),
    };
    assert_eq!(c.try_evaluate(), None);
}

// ============================================================================
// ResolvedType helper methods
// ============================================================================

#[test]
fn test_is_numeric() {
    assert!(ResolvedType::I64.is_numeric());
    assert!(ResolvedType::F64.is_numeric());
    assert!(ResolvedType::U32.is_numeric());
    assert!(!ResolvedType::Bool.is_numeric());
    assert!(!ResolvedType::Str.is_numeric());
}

#[test]
fn test_is_integer() {
    assert!(ResolvedType::I64.is_integer());
    assert!(ResolvedType::U8.is_integer());
    assert!(!ResolvedType::F64.is_integer());
    assert!(!ResolvedType::Bool.is_integer());
}

#[test]
fn test_is_float() {
    assert!(ResolvedType::F32.is_float());
    assert!(ResolvedType::F64.is_float());
    assert!(!ResolvedType::I64.is_float());
}
