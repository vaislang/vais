//! Tests for Display implementations and Formatter functionality
//!
//! Covers Type::Display, ConstExpr::Display, ConstBinOp::Display, MetaVarKind::FromStr,
//! GenericParam helpers, BinOp::precedence completeness, Formatter::format_module,
//! and edge cases for patterns, expressions, and statements.

use vais_ast::formatter::{FormatConfig, Formatter};
use vais_ast::*;

// ============================================================================
// Helper Functions
// ============================================================================

fn span(start: usize, end: usize) -> Span {
    Span::new(start, end)
}

fn spanned<T>(node: T, start: usize, end: usize) -> Spanned<T> {
    Spanned::new(node, span(start, end))
}

fn sp_str(s: &str) -> Spanned<String> {
    spanned(s.to_string(), 0, s.len())
}

fn sp_type(ty: Type) -> Spanned<Type> {
    spanned(ty, 0, 1)
}

fn named_type(name: &str) -> Spanned<Type> {
    sp_type(Type::Named {
        name: name.to_string(),
        generics: vec![],
    })
}

fn sp_expr(expr: Expr) -> Spanned<Expr> {
    spanned(expr, 0, 1)
}

fn sp_stmt(stmt: Stmt) -> Spanned<Stmt> {
    spanned(stmt, 0, 1)
}

fn default_formatter() -> Formatter {
    Formatter::new(FormatConfig::default())
}

/// Wrap a single Item in a Module and format it
fn format_item(item: Item) -> String {
    let mut fmt = default_formatter();
    let module = Module {
        items: vec![spanned(item, 0, 1)],
        modules_map: None,
    };
    fmt.format_module(&module)
}

// ============================================================================
// Type Display Tests - Complete coverage of all Type variants
// ============================================================================

#[test]
fn test_display_type_named_no_generics() {
    let ty = Type::Named {
        name: "i64".to_string(),
        generics: vec![],
    };
    assert_eq!(format!("{}", ty), "i64");
}

#[test]
fn test_display_type_named_with_generics() {
    let ty = Type::Named {
        name: "Vec".to_string(),
        generics: vec![named_type("i32")],
    };
    assert_eq!(format!("{}", ty), "Vec<i32>");
}

#[test]
fn test_display_type_named_multiple_generics() {
    let ty = Type::Named {
        name: "HashMap".to_string(),
        generics: vec![named_type("str"), named_type("i64")],
    };
    assert_eq!(format!("{}", ty), "HashMap<str, i64>");
}

#[test]
fn test_display_type_fnptr() {
    let ty = Type::FnPtr {
        params: vec![named_type("i32"), named_type("i32")],
        ret: Box::new(named_type("bool")),
        is_vararg: false,
    };
    assert_eq!(format!("{}", ty), "fn(i32, i32) -> bool");
}

#[test]
fn test_display_type_fnptr_vararg() {
    let ty = Type::FnPtr {
        params: vec![named_type("i32")],
        ret: Box::new(sp_type(Type::Unit)),
        is_vararg: true,
    };
    assert_eq!(format!("{}", ty), "fn(i32, ...) -> ()");
}

#[test]
fn test_display_type_fnptr_vararg_no_params() {
    let ty = Type::FnPtr {
        params: vec![],
        ret: Box::new(sp_type(Type::Unit)),
        is_vararg: true,
    };
    assert_eq!(format!("{}", ty), "fn(...) -> ()");
}

#[test]
fn test_display_type_array() {
    let ty = Type::Array(Box::new(named_type("i64")));
    assert_eq!(format!("{}", ty), "[i64]");
}

#[test]
fn test_display_type_const_array() {
    let ty = Type::ConstArray {
        element: Box::new(named_type("u8")),
        size: ConstExpr::Literal(10),
    };
    assert_eq!(format!("{}", ty), "[u8; 10]");
}

#[test]
fn test_display_type_map() {
    let ty = Type::Map(Box::new(named_type("str")), Box::new(named_type("i64")));
    assert_eq!(format!("{}", ty), "[str:i64]");
}

#[test]
fn test_display_type_tuple_empty() {
    let ty = Type::Tuple(vec![]);
    assert_eq!(format!("{}", ty), "()");
}

#[test]
fn test_display_type_tuple() {
    let ty = Type::Tuple(vec![named_type("i32"), named_type("bool")]);
    assert_eq!(format!("{}", ty), "(i32, bool)");
}

#[test]
fn test_display_type_optional() {
    let ty = Type::Optional(Box::new(named_type("i64")));
    assert_eq!(format!("{}", ty), "i64?");
}

#[test]
fn test_display_type_result() {
    let ty = Type::Result(Box::new(named_type("i64")));
    assert_eq!(format!("{}", ty), "i64!");
}

#[test]
fn test_display_type_pointer() {
    let ty = Type::Pointer(Box::new(named_type("u8")));
    assert_eq!(format!("{}", ty), "*u8");
}

#[test]
fn test_display_type_ref() {
    let ty = Type::Ref(Box::new(named_type("str")));
    assert_eq!(format!("{}", ty), "&str");
}

#[test]
fn test_display_type_ref_mut() {
    let ty = Type::RefMut(Box::new(named_type("Vec")));
    assert_eq!(format!("{}", ty), "&mut Vec");
}

#[test]
fn test_display_type_slice() {
    let ty = Type::Slice(Box::new(named_type("u8")));
    assert_eq!(format!("{}", ty), "&[u8]");
}

#[test]
fn test_display_type_slice_mut() {
    let ty = Type::SliceMut(Box::new(named_type("i32")));
    assert_eq!(format!("{}", ty), "&mut [i32]");
}

#[test]
fn test_display_type_fn() {
    let ty = Type::Fn {
        params: vec![named_type("i32")],
        ret: Box::new(named_type("bool")),
    };
    assert_eq!(format!("{}", ty), "(i32) -> bool");
}

#[test]
fn test_display_type_unit() {
    assert_eq!(format!("{}", Type::Unit), "()");
}

#[test]
fn test_display_type_infer() {
    assert_eq!(format!("{}", Type::Infer), "_");
}

#[test]
fn test_display_type_dyn_trait_no_generics() {
    let ty = Type::DynTrait {
        trait_name: "Display".to_string(),
        generics: vec![],
    };
    assert_eq!(format!("{}", ty), "dyn Display");
}

#[test]
fn test_display_type_dyn_trait_with_generics() {
    let ty = Type::DynTrait {
        trait_name: "Iterator".to_string(),
        generics: vec![named_type("i64")],
    };
    assert_eq!(format!("{}", ty), "dyn Iterator<i64>");
}

#[test]
fn test_display_type_associated_with_trait() {
    let ty = Type::Associated {
        base: Box::new(named_type("T")),
        trait_name: Some("Iterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![],
    };
    assert_eq!(format!("{}", ty), "<T as Iterator>::Item");
}

#[test]
fn test_display_type_associated_without_trait() {
    let ty = Type::Associated {
        base: Box::new(named_type("Self")),
        trait_name: None,
        assoc_name: "Output".to_string(),
        generics: vec![],
    };
    assert_eq!(format!("{}", ty), "Self::Output");
}

#[test]
fn test_display_type_associated_with_gat_generics() {
    let ty = Type::Associated {
        base: Box::new(named_type("T")),
        trait_name: Some("Lending".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![named_type("i64")],
    };
    assert_eq!(format!("{}", ty), "<T as Lending>::Item<i64>");
}

#[test]
fn test_display_type_linear() {
    let ty = Type::Linear(Box::new(named_type("File")));
    assert_eq!(format!("{}", ty), "linear File");
}

#[test]
fn test_display_type_affine() {
    let ty = Type::Affine(Box::new(named_type("Socket")));
    assert_eq!(format!("{}", ty), "affine Socket");
}

#[test]
fn test_display_type_dependent() {
    let ty = Type::Dependent {
        var_name: "n".to_string(),
        base: Box::new(named_type("i64")),
        predicate: Box::new(sp_expr(Expr::Binary {
            op: BinOp::Gt,
            left: Box::new(sp_expr(Expr::Ident("n".to_string()))),
            right: Box::new(sp_expr(Expr::Int(0))),
        })),
    };
    let display = format!("{}", ty);
    assert!(display.starts_with("{n: i64 | "));
}

#[test]
fn test_display_type_ref_lifetime() {
    let ty = Type::RefLifetime {
        lifetime: "a".to_string(),
        inner: Box::new(named_type("str")),
    };
    assert_eq!(format!("{}", ty), "&'a str");
}

#[test]
fn test_display_type_ref_mut_lifetime() {
    let ty = Type::RefMutLifetime {
        lifetime: "b".to_string(),
        inner: Box::new(named_type("Vec")),
    };
    assert_eq!(format!("{}", ty), "&'b mut Vec");
}

#[test]
fn test_display_type_lazy() {
    let ty = Type::Lazy(Box::new(named_type("i64")));
    assert_eq!(format!("{}", ty), "Lazy<i64>");
}

#[test]
fn test_display_type_impl_trait_single() {
    let ty = Type::ImplTrait {
        bounds: vec![sp_str("Display")],
    };
    assert_eq!(format!("{}", ty), "impl Display");
}

#[test]
fn test_display_type_impl_trait_multiple() {
    let ty = Type::ImplTrait {
        bounds: vec![sp_str("Display"), sp_str("Clone")],
    };
    assert_eq!(format!("{}", ty), "impl Display + Clone");
}

// ============================================================================
// ConstExpr Display Tests
// ============================================================================

#[test]
fn test_display_const_expr_literal() {
    assert_eq!(format!("{}", ConstExpr::Literal(42)), "42");
    assert_eq!(format!("{}", ConstExpr::Literal(-1)), "-1");
    assert_eq!(format!("{}", ConstExpr::Literal(0)), "0");
}

#[test]
fn test_display_const_expr_param() {
    assert_eq!(format!("{}", ConstExpr::Param("N".to_string())), "N");
    assert_eq!(format!("{}", ConstExpr::Param("SIZE".to_string())), "SIZE");
}

#[test]
fn test_display_const_expr_binop() {
    let expr = ConstExpr::BinOp {
        op: ConstBinOp::Add,
        left: Box::new(ConstExpr::Param("N".to_string())),
        right: Box::new(ConstExpr::Literal(1)),
    };
    assert_eq!(format!("{}", expr), "(N + 1)");
}

#[test]
fn test_display_const_expr_negate() {
    let expr = ConstExpr::Negate(Box::new(ConstExpr::Param("N".to_string())));
    assert_eq!(format!("{}", expr), "(-N)");
}

#[test]
fn test_display_const_expr_nested() {
    let expr = ConstExpr::BinOp {
        op: ConstBinOp::Mul,
        left: Box::new(ConstExpr::BinOp {
            op: ConstBinOp::Add,
            left: Box::new(ConstExpr::Param("A".to_string())),
            right: Box::new(ConstExpr::Literal(2)),
        }),
        right: Box::new(ConstExpr::Param("B".to_string())),
    };
    assert_eq!(format!("{}", expr), "((A + 2) * B)");
}

// ============================================================================
// ConstBinOp Display Tests - All variants
// ============================================================================

#[test]
fn test_display_const_binop_all() {
    assert_eq!(format!("{}", ConstBinOp::Add), "+");
    assert_eq!(format!("{}", ConstBinOp::Sub), "-");
    assert_eq!(format!("{}", ConstBinOp::Mul), "*");
    assert_eq!(format!("{}", ConstBinOp::Div), "/");
    assert_eq!(format!("{}", ConstBinOp::Mod), "%");
    assert_eq!(format!("{}", ConstBinOp::BitAnd), "&");
    assert_eq!(format!("{}", ConstBinOp::BitOr), "|");
    assert_eq!(format!("{}", ConstBinOp::BitXor), "^");
    assert_eq!(format!("{}", ConstBinOp::Shl), "<<");
    assert_eq!(format!("{}", ConstBinOp::Shr), ">>");
}

// ============================================================================
// MetaVarKind::from_str Tests
// ============================================================================

#[test]
fn test_metavar_kind_from_str_all_valid() {
    assert_eq!("expr".parse::<MetaVarKind>(), Ok(MetaVarKind::Expr));
    assert_eq!("ty".parse::<MetaVarKind>(), Ok(MetaVarKind::Ty));
    assert_eq!("ident".parse::<MetaVarKind>(), Ok(MetaVarKind::Ident));
    assert_eq!("pat".parse::<MetaVarKind>(), Ok(MetaVarKind::Pat));
    assert_eq!("stmt".parse::<MetaVarKind>(), Ok(MetaVarKind::Stmt));
    assert_eq!("block".parse::<MetaVarKind>(), Ok(MetaVarKind::Block));
    assert_eq!("item".parse::<MetaVarKind>(), Ok(MetaVarKind::Item));
    assert_eq!("lit".parse::<MetaVarKind>(), Ok(MetaVarKind::Lit));
    assert_eq!("tt".parse::<MetaVarKind>(), Ok(MetaVarKind::Tt));
}

#[test]
fn test_metavar_kind_from_str_invalid() {
    assert_eq!("unknown".parse::<MetaVarKind>(), Err(()));
    assert_eq!("Expr".parse::<MetaVarKind>(), Err(()));
    assert_eq!("".parse::<MetaVarKind>(), Err(()));
    assert_eq!("expression".parse::<MetaVarKind>(), Err(()));
}

// ============================================================================
// GenericParam Helper Method Tests
// ============================================================================

#[test]
fn test_generic_param_new_type() {
    let param = GenericParam::new_type(sp_str("T"), vec![sp_str("Display")]);
    assert_eq!(param.name.node, "T");
    assert!(!param.is_const());
    assert!(!param.is_higher_kinded());
    assert!(!param.is_covariant());
    assert!(!param.is_contravariant());
    assert!(matches!(param.kind, GenericParamKind::Type { .. }));
}

#[test]
fn test_generic_param_new_type_with_variance_covariant() {
    let param = GenericParam::new_type_with_variance(sp_str("T"), vec![], Variance::Covariant);
    assert!(param.is_covariant());
    assert!(!param.is_contravariant());
}

#[test]
fn test_generic_param_new_type_with_variance_contravariant() {
    let param = GenericParam::new_type_with_variance(sp_str("T"), vec![], Variance::Contravariant);
    assert!(!param.is_covariant());
    assert!(param.is_contravariant());
}

#[test]
fn test_generic_param_new_const() {
    let param = GenericParam::new_const(sp_str("N"), named_type("u64"));
    assert!(param.is_const());
    assert!(!param.is_higher_kinded());
    assert!(param.bounds.is_empty());
}

#[test]
fn test_generic_param_new_lifetime() {
    let param = GenericParam::new_lifetime(sp_str("'a"), vec!["'b".to_string()]);
    assert!(!param.is_const());
    assert!(!param.is_higher_kinded());
    match &param.kind {
        GenericParamKind::Lifetime { bounds } => {
            assert_eq!(bounds, &["'b"]);
        }
        _ => panic!("Expected Lifetime kind"),
    }
}

#[test]
fn test_generic_param_new_higher_kinded() {
    let param = GenericParam::new_higher_kinded(sp_str("F"), 2, vec![sp_str("Functor")]);
    assert!(param.is_higher_kinded());
    assert!(!param.is_const());
    match &param.kind {
        GenericParamKind::HigherKinded { arity, bounds } => {
            assert_eq!(*arity, 2);
            assert_eq!(bounds.len(), 1);
        }
        _ => panic!("Expected HigherKinded kind"),
    }
}

// ============================================================================
// BinOp Precedence Tests - Complete coverage
// ============================================================================

#[test]
fn test_binop_precedence_all_operators() {
    assert_eq!(BinOp::Or.precedence(), 1);
    assert_eq!(BinOp::And.precedence(), 2);
    assert_eq!(BinOp::BitOr.precedence(), 3);
    assert_eq!(BinOp::BitXor.precedence(), 4);
    assert_eq!(BinOp::BitAnd.precedence(), 5);
    assert_eq!(BinOp::Eq.precedence(), 6);
    assert_eq!(BinOp::Neq.precedence(), 6);
    assert_eq!(BinOp::Lt.precedence(), 7);
    assert_eq!(BinOp::Lte.precedence(), 7);
    assert_eq!(BinOp::Gt.precedence(), 7);
    assert_eq!(BinOp::Gte.precedence(), 7);
    assert_eq!(BinOp::Shl.precedence(), 8);
    assert_eq!(BinOp::Shr.precedence(), 8);
    assert_eq!(BinOp::Add.precedence(), 9);
    assert_eq!(BinOp::Sub.precedence(), 9);
    assert_eq!(BinOp::Mul.precedence(), 10);
    assert_eq!(BinOp::Div.precedence(), 10);
    assert_eq!(BinOp::Mod.precedence(), 10);
}

#[test]
fn test_binop_precedence_ordering() {
    assert!(BinOp::Mul.precedence() > BinOp::Add.precedence());
    assert!(BinOp::Add.precedence() > BinOp::Lt.precedence());
    assert!(BinOp::Lt.precedence() > BinOp::And.precedence());
    assert!(BinOp::And.precedence() > BinOp::Or.precedence());
    assert!(BinOp::Shl.precedence() < BinOp::Add.precedence());
    assert!(BinOp::BitAnd.precedence() < BinOp::Eq.precedence());
    assert!(BinOp::BitAnd.precedence() > BinOp::BitXor.precedence());
}

// ============================================================================
// Ownership Tests
// ============================================================================

#[test]
fn test_ownership_default_is_regular() {
    let o: Ownership = Default::default();
    assert_eq!(o, Ownership::Regular);
}

#[test]
fn test_ownership_variants_all() {
    let variants = [
        Ownership::Regular,
        Ownership::Linear,
        Ownership::Affine,
        Ownership::Move,
    ];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn test_ownership_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Ownership::Regular);
    set.insert(Ownership::Linear);
    set.insert(Ownership::Affine);
    set.insert(Ownership::Move);
    assert_eq!(set.len(), 4);
}

// ============================================================================
// Variance Tests
// ============================================================================

#[test]
fn test_variance_default() {
    let v: Variance = Default::default();
    assert_eq!(v, Variance::Invariant);
}

#[test]
fn test_variance_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Variance::Invariant);
    set.insert(Variance::Covariant);
    set.insert(Variance::Contravariant);
    assert_eq!(set.len(), 3);
}

// ============================================================================
// CaptureMode Tests
// ============================================================================

#[test]
fn test_capture_mode_variants() {
    let modes = [
        CaptureMode::ByValue,
        CaptureMode::Move,
        CaptureMode::ByRef,
        CaptureMode::ByMutRef,
    ];
    for (i, a) in modes.iter().enumerate() {
        for (j, b) in modes.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn test_capture_mode_copy() {
    let mode = CaptureMode::Move;
    let copied = mode;
    assert_eq!(mode, copied);
}

// ============================================================================
// Pattern Edge Case Tests
// ============================================================================

#[test]
fn test_pattern_range_inclusive() {
    let pat = Pattern::Range {
        start: Some(Box::new(spanned(Pattern::Literal(Literal::Int(1)), 0, 1))),
        end: Some(Box::new(spanned(Pattern::Literal(Literal::Int(10)), 0, 2))),
        inclusive: true,
    };
    assert!(matches!(
        pat,
        Pattern::Range {
            inclusive: true,
            ..
        }
    ));
}

#[test]
fn test_pattern_range_exclusive() {
    let pat = Pattern::Range {
        start: Some(Box::new(spanned(Pattern::Literal(Literal::Int(0)), 0, 1))),
        end: Some(Box::new(spanned(Pattern::Literal(Literal::Int(5)), 0, 1))),
        inclusive: false,
    };
    assert!(matches!(
        pat,
        Pattern::Range {
            inclusive: false,
            ..
        }
    ));
}

#[test]
fn test_pattern_range_open_start() {
    let pat = Pattern::Range {
        start: None,
        end: Some(Box::new(spanned(Pattern::Literal(Literal::Int(10)), 0, 2))),
        inclusive: false,
    };
    assert!(matches!(pat, Pattern::Range { start: None, .. }));
}

#[test]
fn test_pattern_or() {
    let pat = Pattern::Or(vec![
        spanned(Pattern::Literal(Literal::Int(1)), 0, 1),
        spanned(Pattern::Literal(Literal::Int(2)), 0, 1),
        spanned(Pattern::Literal(Literal::Int(3)), 0, 1),
    ]);
    if let Pattern::Or(patterns) = pat {
        assert_eq!(patterns.len(), 3);
    } else {
        panic!("Expected Or pattern");
    }
}

#[test]
fn test_pattern_alias() {
    let pat = Pattern::Alias {
        name: "x".to_string(),
        pattern: Box::new(spanned(Pattern::Literal(Literal::Int(42)), 0, 2)),
    };
    if let Pattern::Alias { name, pattern } = pat {
        assert_eq!(name, "x");
        assert_eq!(pattern.node, Pattern::Literal(Literal::Int(42)));
    } else {
        panic!("Expected Alias pattern");
    }
}

// ============================================================================
// Literal Tests
// ============================================================================

#[test]
fn test_literal_equality() {
    assert_eq!(Literal::Int(42), Literal::Int(42));
    assert_ne!(Literal::Int(42), Literal::Int(43));
    assert_eq!(Literal::Bool(true), Literal::Bool(true));
    assert_ne!(Literal::Bool(true), Literal::Bool(false));
    assert_eq!(
        Literal::String("hello".to_string()),
        Literal::String("hello".to_string())
    );
    assert_ne!(
        Literal::String("a".to_string()),
        Literal::String("b".to_string())
    );
}

#[test]
fn test_literal_float() {
    let lit = Literal::Float(3.14);
    assert_eq!(lit, Literal::Float(3.14));
}

// ============================================================================
// Expr Edge Case Tests
// ============================================================================

#[test]
fn test_expr_range() {
    let expr = Expr::Range {
        start: Some(Box::new(sp_expr(Expr::Int(0)))),
        end: Some(Box::new(sp_expr(Expr::Int(10)))),
        inclusive: false,
    };
    assert!(matches!(
        expr,
        Expr::Range {
            inclusive: false,
            ..
        }
    ));
}

#[test]
fn test_expr_range_inclusive() {
    let expr = Expr::Range {
        start: Some(Box::new(sp_expr(Expr::Int(1)))),
        end: Some(Box::new(sp_expr(Expr::Int(5)))),
        inclusive: true,
    };
    assert!(matches!(
        expr,
        Expr::Range {
            inclusive: true,
            ..
        }
    ));
}

#[test]
fn test_expr_struct_lit() {
    let expr = Expr::StructLit {
        name: sp_str("Point"),
        fields: vec![
            (sp_str("x"), sp_expr(Expr::Int(1))),
            (sp_str("y"), sp_expr(Expr::Int(2))),
        ],
    };
    if let Expr::StructLit { name, fields } = expr {
        assert_eq!(name.node, "Point");
        assert_eq!(fields.len(), 2);
    }
}

#[test]
fn test_expr_map_lit() {
    let expr = Expr::MapLit(vec![(
        sp_expr(Expr::String("key".to_string())),
        sp_expr(Expr::Int(42)),
    )]);
    if let Expr::MapLit(pairs) = expr {
        assert_eq!(pairs.len(), 1);
    }
}

#[test]
fn test_expr_cast() {
    let expr = Expr::Cast {
        expr: Box::new(sp_expr(Expr::Int(42))),
        ty: named_type("f64"),
    };
    assert!(matches!(expr, Expr::Cast { .. }));
}

#[test]
fn test_expr_assign() {
    let expr = Expr::Assign {
        target: Box::new(sp_expr(Expr::Ident("x".to_string()))),
        value: Box::new(sp_expr(Expr::Int(10))),
    };
    assert!(matches!(expr, Expr::Assign { .. }));
}

#[test]
fn test_expr_assign_op() {
    let expr = Expr::AssignOp {
        op: BinOp::Add,
        target: Box::new(sp_expr(Expr::Ident("x".to_string()))),
        value: Box::new(sp_expr(Expr::Int(1))),
    };
    assert!(matches!(expr, Expr::AssignOp { op: BinOp::Add, .. }));
}

#[test]
fn test_expr_lambda() {
    let expr = Expr::Lambda {
        params: vec![Param {
            name: sp_str("x"),
            ty: named_type("i64"),
            is_mut: false,
            is_vararg: false,
            ownership: Ownership::Regular,
            default_value: None,
        }],
        body: Box::new(sp_expr(Expr::Binary {
            op: BinOp::Mul,
            left: Box::new(sp_expr(Expr::Ident("x".to_string()))),
            right: Box::new(sp_expr(Expr::Int(2))),
        })),
        captures: vec![],
        capture_mode: CaptureMode::ByValue,
    };
    assert!(matches!(expr, Expr::Lambda { .. }));
}

#[test]
fn test_expr_spawn() {
    let expr = Expr::Spawn(Box::new(sp_expr(Expr::Int(42))));
    assert!(matches!(expr, Expr::Spawn(_)));
}

#[test]
fn test_expr_yield() {
    let expr = Expr::Yield(Box::new(sp_expr(Expr::Int(42))));
    assert!(matches!(expr, Expr::Yield(_)));
}

#[test]
fn test_expr_comptime() {
    let expr = Expr::Comptime {
        body: Box::new(sp_expr(Expr::Int(100))),
    };
    assert!(matches!(expr, Expr::Comptime { .. }));
}

#[test]
fn test_expr_lazy_force() {
    let lazy_expr = Expr::Lazy(Box::new(sp_expr(Expr::Int(42))));
    let force_expr = Expr::Force(Box::new(sp_expr(lazy_expr)));
    assert!(matches!(force_expr, Expr::Force(_)));
}

#[test]
fn test_expr_old() {
    let expr = Expr::Old(Box::new(sp_expr(Expr::Ident("x".to_string()))));
    assert!(matches!(expr, Expr::Old(_)));
}

#[test]
fn test_expr_assert_no_message() {
    let expr = Expr::Assert {
        condition: Box::new(sp_expr(Expr::Bool(true))),
        message: None,
    };
    assert!(matches!(expr, Expr::Assert { message: None, .. }));
}

#[test]
fn test_expr_assert_with_message() {
    let expr = Expr::Assert {
        condition: Box::new(sp_expr(Expr::Bool(true))),
        message: Some(Box::new(sp_expr(Expr::String(
            "should be true".to_string(),
        )))),
    };
    assert!(matches!(
        expr,
        Expr::Assert {
            message: Some(_),
            ..
        }
    ));
}

#[test]
fn test_expr_assume() {
    let expr = Expr::Assume(Box::new(sp_expr(Expr::Bool(true))));
    assert!(matches!(expr, Expr::Assume(_)));
}

#[test]
fn test_expr_error() {
    let expr = Expr::Error {
        message: "unexpected token".to_string(),
        skipped_tokens: vec!["foo".to_string(), "bar".to_string()],
    };
    if let Expr::Error {
        message,
        skipped_tokens,
    } = expr
    {
        assert_eq!(message, "unexpected token");
        assert_eq!(skipped_tokens.len(), 2);
    }
}

#[test]
fn test_expr_spread() {
    let expr = Expr::Spread(Box::new(sp_expr(Expr::Ident("items".to_string()))));
    assert!(matches!(expr, Expr::Spread(_)));
}

#[test]
fn test_expr_ref_deref() {
    let ref_expr = Expr::Ref(Box::new(sp_expr(Expr::Ident("x".to_string()))));
    let deref_expr = Expr::Deref(Box::new(sp_expr(Expr::Ident("p".to_string()))));
    assert!(matches!(ref_expr, Expr::Ref(_)));
    assert!(matches!(deref_expr, Expr::Deref(_)));
}

#[test]
fn test_expr_self_call() {
    let expr = Expr::SelfCall;
    assert!(matches!(expr, Expr::SelfCall));
}

#[test]
fn test_expr_string_interp() {
    let expr = Expr::StringInterp(vec![
        StringInterpPart::Lit("hello ".to_string()),
        StringInterpPart::Expr(Box::new(sp_expr(Expr::Ident("name".to_string())))),
        StringInterpPart::Lit("!".to_string()),
    ]);
    if let Expr::StringInterp(parts) = expr {
        assert_eq!(parts.len(), 3);
    }
}

#[test]
fn test_expr_method_call() {
    let expr = Expr::MethodCall {
        receiver: Box::new(sp_expr(Expr::Ident("vec".to_string()))),
        method: sp_str("push"),
        args: vec![sp_expr(Expr::Int(42))],
    };
    if let Expr::MethodCall { method, args, .. } = expr {
        assert_eq!(method.node, "push");
        assert_eq!(args.len(), 1);
    }
}

#[test]
fn test_expr_static_method_call() {
    let expr = Expr::StaticMethodCall {
        type_name: sp_str("Vec"),
        method: sp_str("new"),
        args: vec![],
    };
    if let Expr::StaticMethodCall {
        type_name, method, ..
    } = expr
    {
        assert_eq!(type_name.node, "Vec");
        assert_eq!(method.node, "new");
    }
}

#[test]
fn test_expr_ternary() {
    let expr = Expr::Ternary {
        cond: Box::new(sp_expr(Expr::Bool(true))),
        then: Box::new(sp_expr(Expr::Int(1))),
        else_: Box::new(sp_expr(Expr::Int(0))),
    };
    assert!(matches!(expr, Expr::Ternary { .. }));
}

#[test]
fn test_expr_while() {
    let expr = Expr::While {
        condition: Box::new(sp_expr(Expr::Bool(true))),
        body: vec![],
    };
    assert!(matches!(expr, Expr::While { .. }));
}

#[test]
fn test_expr_macro_invoke() {
    let invoke = MacroInvoke {
        name: sp_str("vec"),
        delimiter: Delimiter::Bracket,
        tokens: vec![MacroToken::Literal(MacroLiteral::Int(1))],
    };
    let expr = Expr::MacroInvoke(invoke);
    assert!(matches!(expr, Expr::MacroInvoke(_)));
}

// ============================================================================
// IfElse Tests
// ============================================================================

#[test]
fn test_if_else_else_branch() {
    let else_branch = IfElse::Else(vec![sp_stmt(Stmt::Return(None))]);
    assert!(matches!(else_branch, IfElse::Else(_)));
}

#[test]
fn test_if_else_else_if_branch() {
    let else_if = IfElse::ElseIf(
        Box::new(sp_expr(Expr::Bool(false))),
        vec![],
        Some(Box::new(IfElse::Else(vec![]))),
    );
    assert!(matches!(else_if, IfElse::ElseIf(_, _, Some(_))));
}

// ============================================================================
// Stmt Tests
// ============================================================================

#[test]
fn test_stmt_let_with_type() {
    let stmt = Stmt::Let {
        name: sp_str("x"),
        ty: Some(named_type("i64")),
        value: Box::new(sp_expr(Expr::Int(42))),
        is_mut: false,
        ownership: Ownership::Regular,
    };
    if let Stmt::Let { ty, .. } = &stmt {
        assert!(ty.is_some());
    }
}

#[test]
fn test_stmt_let_mutable() {
    let stmt = Stmt::Let {
        name: sp_str("count"),
        ty: None,
        value: Box::new(sp_expr(Expr::Int(0))),
        is_mut: true,
        ownership: Ownership::Regular,
    };
    if let Stmt::Let { is_mut, .. } = &stmt {
        assert!(*is_mut);
    }
}

#[test]
fn test_stmt_defer() {
    let stmt = Stmt::Defer(Box::new(sp_expr(Expr::Call {
        func: Box::new(sp_expr(Expr::Ident("close".to_string()))),
        args: vec![],
    })));
    assert!(matches!(stmt, Stmt::Defer(_)));
}

#[test]
fn test_stmt_error() {
    let stmt = Stmt::Error {
        message: "parse error".to_string(),
        skipped_tokens: vec!["bad".to_string()],
    };
    if let Stmt::Error {
        message,
        skipped_tokens,
    } = stmt
    {
        assert_eq!(message, "parse error");
        assert_eq!(skipped_tokens.len(), 1);
    }
}

// ============================================================================
// Macro System Tests
// ============================================================================

#[test]
fn test_macro_token_variants() {
    let _ident = MacroToken::Ident("foo".to_string());
    let _punct = MacroToken::Punct('+');
    let _lit = MacroToken::Literal(MacroLiteral::Int(42));
    let _group = MacroToken::Group(Delimiter::Paren, vec![]);
}

#[test]
fn test_macro_literal_variants() {
    assert_eq!(MacroLiteral::Int(1), MacroLiteral::Int(1));
    assert_eq!(MacroLiteral::Float(1.0), MacroLiteral::Float(1.0));
    assert_eq!(
        MacroLiteral::String("s".to_string()),
        MacroLiteral::String("s".to_string())
    );
    assert_eq!(MacroLiteral::Bool(true), MacroLiteral::Bool(true));
    assert_ne!(MacroLiteral::Int(1), MacroLiteral::Bool(true));
}

#[test]
fn test_macro_pattern_empty() {
    let pat = MacroPattern::Empty;
    assert!(matches!(pat, MacroPattern::Empty));
}

#[test]
fn test_macro_pattern_sequence() {
    let pat = MacroPattern::Sequence(vec![
        MacroPatternElement::Token(MacroToken::Ident("let".to_string())),
        MacroPatternElement::MetaVar {
            name: "x".to_string(),
            kind: MetaVarKind::Ident,
        },
    ]);
    if let MacroPattern::Sequence(elements) = pat {
        assert_eq!(elements.len(), 2);
    }
}

#[test]
fn test_macro_pattern_repetition() {
    let elem = MacroPatternElement::Repetition {
        patterns: vec![MacroPatternElement::MetaVar {
            name: "x".to_string(),
            kind: MetaVarKind::Expr,
        }],
        separator: Some(MacroToken::Punct(',')),
        kind: RepetitionKind::ZeroOrMore,
    };
    assert!(matches!(
        elem,
        MacroPatternElement::Repetition {
            kind: RepetitionKind::ZeroOrMore,
            ..
        }
    ));
}

#[test]
fn test_macro_pattern_group() {
    let elem = MacroPatternElement::Group {
        delimiter: Delimiter::Brace,
        content: vec![],
    };
    assert!(matches!(
        elem,
        MacroPatternElement::Group {
            delimiter: Delimiter::Brace,
            ..
        }
    ));
}

#[test]
fn test_macro_template_empty() {
    let tmpl = MacroTemplate::Empty;
    assert!(matches!(tmpl, MacroTemplate::Empty));
}

#[test]
fn test_macro_template_sequence() {
    let tmpl = MacroTemplate::Sequence(vec![
        MacroTemplateElement::Token(MacroToken::Ident("result".to_string())),
        MacroTemplateElement::MetaVar("x".to_string()),
    ]);
    if let MacroTemplate::Sequence(elems) = tmpl {
        assert_eq!(elems.len(), 2);
    }
}

#[test]
fn test_macro_template_repetition() {
    let elem = MacroTemplateElement::Repetition {
        elements: vec![MacroTemplateElement::MetaVar("x".to_string())],
        separator: Some(MacroToken::Punct(',')),
        kind: RepetitionKind::OneOrMore,
    };
    assert!(matches!(
        elem,
        MacroTemplateElement::Repetition {
            kind: RepetitionKind::OneOrMore,
            ..
        }
    ));
}

#[test]
fn test_macro_template_group() {
    let elem = MacroTemplateElement::Group {
        delimiter: Delimiter::Bracket,
        content: vec![],
    };
    assert!(matches!(
        elem,
        MacroTemplateElement::Group {
            delimiter: Delimiter::Bracket,
            ..
        }
    ));
}

#[test]
fn test_repetition_kind_variants() {
    let kinds = [
        RepetitionKind::ZeroOrMore,
        RepetitionKind::OneOrMore,
        RepetitionKind::ZeroOrOne,
    ];
    for (i, a) in kinds.iter().enumerate() {
        for (j, b) in kinds.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn test_delimiter_variants() {
    assert_ne!(Delimiter::Paren, Delimiter::Bracket);
    assert_ne!(Delimiter::Bracket, Delimiter::Brace);
    assert_eq!(Delimiter::Paren, Delimiter::Paren);
}

// ============================================================================
// Trait/Impl Structure Tests
// ============================================================================

#[test]
fn test_trait_with_super_traits() {
    let trait_def = Trait {
        name: sp_str("Iterator"),
        generics: vec![],
        super_traits: vec![sp_str("Iterable")],
        associated_types: vec![AssociatedType {
            name: sp_str("Item"),
            generics: vec![],
            bounds: vec![],
            default: None,
        }],
        methods: vec![],
        is_pub: true,
        where_clause: vec![],
    };
    assert_eq!(trait_def.super_traits.len(), 1);
    assert_eq!(trait_def.associated_types.len(), 1);
}

#[test]
fn test_trait_method_with_default() {
    let method = TraitMethod {
        name: sp_str("default_method"),
        params: vec![],
        ret_type: Some(named_type("i64")),
        default_body: Some(FunctionBody::Expr(Box::new(sp_expr(Expr::Int(0))))),
        is_async: false,
        is_const: false,
    };
    assert!(method.default_body.is_some());
}

#[test]
fn test_trait_method_async_const() {
    let method = TraitMethod {
        name: sp_str("compute"),
        params: vec![],
        ret_type: None,
        default_body: None,
        is_async: true,
        is_const: true,
    };
    assert!(method.is_async);
    assert!(method.is_const);
}

#[test]
fn test_impl_with_associated_types() {
    let impl_block = Impl {
        target_type: named_type("MyVec"),
        trait_name: Some(sp_str("Iterator")),
        generics: vec![],
        associated_types: vec![AssociatedTypeImpl {
            name: sp_str("Item"),
            ty: named_type("i64"),
        }],
        methods: vec![],
    };
    assert_eq!(impl_block.associated_types.len(), 1);
    assert_eq!(impl_block.associated_types[0].name.node, "Item");
}

// ============================================================================
// ExternBlock Tests
// ============================================================================

#[test]
fn test_extern_block() {
    let ext = ExternBlock {
        abi: "C".to_string(),
        functions: vec![ExternFunction {
            name: sp_str("printf"),
            params: vec![Param {
                name: sp_str("fmt"),
                ty: sp_type(Type::Pointer(Box::new(named_type("u8")))),
                is_mut: false,
                is_vararg: false,
                ownership: Ownership::Regular,
                default_value: None,
            }],
            ret_type: Some(named_type("i32")),
            is_vararg: true,
            attributes: vec![],
        }],
    };
    assert_eq!(ext.abi, "C");
    assert_eq!(ext.functions.len(), 1);
    assert!(ext.functions[0].is_vararg);
}

// ============================================================================
// Use/Import Tests
// ============================================================================

#[test]
fn test_use_with_alias() {
    let use_stmt = Use {
        path: vec![sp_str("std"), sp_str("io")],
        alias: Some(sp_str("stdio")),
        items: None,
    };
    assert_eq!(use_stmt.path.len(), 2);
    assert_eq!(use_stmt.alias.as_ref().unwrap().node, "stdio");
}

#[test]
fn test_use_with_selective_import() {
    let use_stmt = Use {
        path: vec![sp_str("std"), sp_str("collections")],
        alias: None,
        items: Some(vec![sp_str("HashMap"), sp_str("BTreeMap")]),
    };
    assert_eq!(use_stmt.items.as_ref().unwrap().len(), 2);
}

// ============================================================================
// Formatter Tests (via format_module public API)
// ============================================================================

#[test]
fn test_formatter_default_config() {
    let config = FormatConfig::default();
    assert_eq!(config.indent_size, 4);
    assert_eq!(config.max_line_length, 100);
    assert!(!config.use_tabs);
}

#[test]
fn test_formatter_format_empty_module() {
    let mut fmt = default_formatter();
    let module = Module {
        items: vec![],
        modules_map: None,
    };
    let result = fmt.format_module(&module);
    assert_eq!(result, "");
}

#[test]
fn test_formatter_format_const() {
    let result = format_item(Item::Const(ConstDef {
        name: sp_str("MAX"),
        ty: named_type("i64"),
        value: sp_expr(Expr::Int(100)),
        is_pub: true,
        attributes: vec![],
    }));
    assert!(result.contains("P C MAX"));
    assert!(result.contains("100"));
}

#[test]
fn test_formatter_format_global() {
    let result = format_item(Item::Global(GlobalDef {
        name: sp_str("counter"),
        ty: named_type("i64"),
        value: sp_expr(Expr::Int(0)),
        is_pub: false,
        is_mutable: true,
    }));
    assert!(result.contains("G counter"));
    assert!(result.contains("0"));
}

#[test]
fn test_formatter_format_function_simple() {
    let result = format_item(Item::Function(Function {
        name: sp_str("add"),
        generics: vec![],
        params: vec![
            Param {
                name: sp_str("a"),
                ty: named_type("i64"),
                is_mut: false,
                is_vararg: false,
                ownership: Ownership::Regular,
                default_value: None,
            },
            Param {
                name: sp_str("b"),
                ty: named_type("i64"),
                is_mut: false,
                is_vararg: false,
                ownership: Ownership::Regular,
                default_value: None,
            },
        ],
        ret_type: Some(named_type("i64")),
        body: FunctionBody::Expr(Box::new(sp_expr(Expr::Binary {
            op: BinOp::Add,
            left: Box::new(sp_expr(Expr::Ident("a".to_string()))),
            right: Box::new(sp_expr(Expr::Ident("b".to_string()))),
        }))),
        is_pub: false,
        is_async: false,
        attributes: vec![],
        where_clause: vec![],
    }));
    assert!(result.contains("F add(a: i64, b: i64) -> i64"));
    assert!(result.contains("a + b"));
}

#[test]
fn test_formatter_format_function_pub_async() {
    let result = format_item(Item::Function(Function {
        name: sp_str("fetch"),
        generics: vec![],
        params: vec![],
        ret_type: Some(named_type("i64")),
        body: FunctionBody::Block(vec![sp_stmt(Stmt::Return(Some(Box::new(sp_expr(
            Expr::Int(0),
        )))))]),
        is_pub: true,
        is_async: true,
        attributes: vec![],
        where_clause: vec![],
    }));
    assert!(result.contains("P A F fetch"));
}

#[test]
fn test_formatter_format_struct() {
    let result = format_item(Item::Struct(Struct {
        name: sp_str("Point"),
        generics: vec![],
        fields: vec![
            Field {
                name: sp_str("x"),
                ty: named_type("f64"),
                is_pub: false,
            },
            Field {
                name: sp_str("y"),
                ty: named_type("f64"),
                is_pub: false,
            },
        ],
        methods: vec![],
        is_pub: true,
        attributes: vec![],
        where_clause: vec![],
    }));
    assert!(result.contains("P S Point"));
    assert!(result.contains("x: f64"));
    assert!(result.contains("y: f64"));
}

#[test]
fn test_formatter_format_enum() {
    let result = format_item(Item::Enum(Enum {
        name: sp_str("Color"),
        generics: vec![],
        variants: vec![
            Variant {
                name: sp_str("Red"),
                fields: VariantFields::Unit,
            },
            Variant {
                name: sp_str("Green"),
                fields: VariantFields::Unit,
            },
            Variant {
                name: sp_str("Blue"),
                fields: VariantFields::Unit,
            },
        ],
        is_pub: false,
        attributes: vec![],
    }));
    assert!(result.contains("E Color"));
    assert!(result.contains("Red"));
    assert!(result.contains("Green"));
    assert!(result.contains("Blue"));
}

#[test]
fn test_formatter_format_type_alias() {
    let result = format_item(Item::TypeAlias(TypeAlias {
        name: sp_str("Num"),
        generics: vec![],
        ty: named_type("i64"),
        is_pub: false,
    }));
    assert!(result.contains("T Num = i64"));
}

#[test]
fn test_formatter_format_use() {
    let result = format_item(Item::Use(Use {
        path: vec![sp_str("std"), sp_str("io")],
        alias: None,
        items: None,
    }));
    assert!(result.contains("U std.io"));
}

#[test]
fn test_formatter_format_error_item() {
    let result = format_item(Item::Error {
        message: "parse failure".to_string(),
        skipped_tokens: vec![],
    });
    assert!(result.contains("# ERROR: parse failure"));
}

#[test]
fn test_formatter_format_extern_block() {
    let result = format_item(Item::ExternBlock(ExternBlock {
        abi: "C".to_string(),
        functions: vec![ExternFunction {
            name: sp_str("exit"),
            params: vec![Param {
                name: sp_str("code"),
                ty: named_type("i32"),
                is_mut: false,
                is_vararg: false,
                ownership: Ownership::Regular,
                default_value: None,
            }],
            ret_type: None,
            is_vararg: false,
            attributes: vec![],
        }],
    }));
    assert!(result.contains("extern \"C\""));
    assert!(result.contains("F exit(code: i32)"));
}

#[test]
fn test_formatter_format_union() {
    let result = format_item(Item::Union(Union {
        name: sp_str("Value"),
        generics: vec![],
        fields: vec![
            Field {
                name: sp_str("i"),
                ty: named_type("i64"),
                is_pub: false,
            },
            Field {
                name: sp_str("f"),
                ty: named_type("f64"),
                is_pub: false,
            },
        ],
        is_pub: false,
    }));
    assert!(result.contains("O Value"));
    assert!(result.contains("i: i64"));
    assert!(result.contains("f: f64"));
}

#[test]
fn test_formatter_format_multiple_items() {
    let mut fmt = default_formatter();
    let module = Module {
        items: vec![
            spanned(
                Item::Const(ConstDef {
                    name: sp_str("A"),
                    ty: named_type("i64"),
                    value: sp_expr(Expr::Int(1)),
                    is_pub: false,
                    attributes: vec![],
                }),
                0,
                1,
            ),
            spanned(
                Item::Const(ConstDef {
                    name: sp_str("B"),
                    ty: named_type("i64"),
                    value: sp_expr(Expr::Int(2)),
                    is_pub: false,
                    attributes: vec![],
                }),
                0,
                1,
            ),
        ],
        modules_map: None,
    };
    let result = fmt.format_module(&module);
    // Multiple items should be separated by newline
    assert!(result.contains("C A"));
    assert!(result.contains("C B"));
}

#[test]
fn test_formatter_format_trait() {
    let result = format_item(Item::Trait(Trait {
        name: sp_str("Printable"),
        generics: vec![],
        super_traits: vec![],
        associated_types: vec![],
        methods: vec![TraitMethod {
            name: sp_str("print"),
            params: vec![],
            ret_type: None,
            default_body: None,
            is_async: false,
            is_const: false,
        }],
        is_pub: true,
        where_clause: vec![],
    }));
    assert!(result.contains("P W Printable"));
    assert!(result.contains("F print"));
}

#[test]
fn test_formatter_format_impl() {
    let result = format_item(Item::Impl(Impl {
        target_type: named_type("Point"),
        trait_name: None,
        generics: vec![],
        associated_types: vec![],
        methods: vec![spanned(
            Function {
                name: sp_str("new"),
                generics: vec![],
                params: vec![],
                ret_type: Some(named_type("Point")),
                body: FunctionBody::Expr(Box::new(sp_expr(Expr::StructLit {
                    name: sp_str("Point"),
                    fields: vec![
                        (sp_str("x"), sp_expr(Expr::Int(0))),
                        (sp_str("y"), sp_expr(Expr::Int(0))),
                    ],
                }))),
                is_pub: false,
                is_async: false,
                attributes: vec![],
                where_clause: vec![],
            },
            0,
            1,
        )],
    }));
    assert!(result.contains("X Point"));
    assert!(result.contains("F new"));
}

#[test]
fn test_formatter_format_trait_alias() {
    let result = format_item(Item::TraitAlias(TraitAlias {
        name: sp_str("Printable"),
        generics: vec![],
        bounds: vec![sp_str("Display"), sp_str("Debug")],
        is_pub: false,
    }));
    assert!(result.contains("Printable"));
}

// ============================================================================
// ConstDef and GlobalDef Tests
// ============================================================================

#[test]
fn test_const_def() {
    let cd = ConstDef {
        name: sp_str("MAX"),
        ty: named_type("i64"),
        value: sp_expr(Expr::Int(100)),
        is_pub: true,
        attributes: vec![],
    };
    assert_eq!(cd.name.node, "MAX");
    assert!(cd.is_pub);
}

#[test]
fn test_global_def() {
    let gd = GlobalDef {
        name: sp_str("counter"),
        ty: named_type("i64"),
        value: sp_expr(Expr::Int(0)),
        is_pub: false,
        is_mutable: true,
    };
    assert_eq!(gd.name.node, "counter");
    assert!(gd.is_mutable);
    assert!(!gd.is_pub);
}

// ============================================================================
// TypeAlias and TraitAlias Tests
// ============================================================================

#[test]
fn test_type_alias() {
    let ta = TypeAlias {
        name: sp_str("Num"),
        generics: vec![],
        ty: named_type("i64"),
        is_pub: false,
    };
    assert_eq!(ta.name.node, "Num");
}

#[test]
fn test_trait_alias_data() {
    let ta = TraitAlias {
        name: sp_str("Printable"),
        generics: vec![],
        bounds: vec![sp_str("Display"), sp_str("Debug")],
        is_pub: true,
    };
    assert_eq!(ta.bounds.len(), 2);
}

// ============================================================================
// AssociatedType Tests
// ============================================================================

#[test]
fn test_associated_type_with_default() {
    let at = AssociatedType {
        name: sp_str("Item"),
        generics: vec![],
        bounds: vec![sp_str("Clone")],
        default: Some(named_type("i64")),
    };
    assert!(at.default.is_some());
    assert_eq!(at.bounds.len(), 1);
}

#[test]
fn test_associated_type_gat() {
    let at = AssociatedType {
        name: sp_str("Item"),
        generics: vec![GenericParam::new_lifetime(sp_str("'a"), vec![])],
        bounds: vec![],
        default: None,
    };
    assert_eq!(at.generics.len(), 1);
}

// ============================================================================
// WherePredicate Tests
// ============================================================================

#[test]
fn test_where_predicate_multiple_bounds() {
    let wp = WherePredicate {
        ty: sp_str("T"),
        bounds: vec![sp_str("Display"), sp_str("Clone"), sp_str("Send")],
    };
    assert_eq!(wp.bounds.len(), 3);
}

// ============================================================================
// Module with modules_map Tests
// ============================================================================

#[test]
fn test_module_with_modules_map() {
    use std::collections::HashMap;
    use std::path::PathBuf;

    let mut map = HashMap::new();
    map.insert(PathBuf::from("main.vais"), vec![0, 1]);
    map.insert(PathBuf::from("lib.vais"), vec![2]);

    let module = Module {
        items: vec![
            spanned(
                Item::Error {
                    message: "a".to_string(),
                    skipped_tokens: vec![],
                },
                0,
                1,
            ),
            spanned(
                Item::Error {
                    message: "b".to_string(),
                    skipped_tokens: vec![],
                },
                0,
                1,
            ),
            spanned(
                Item::Error {
                    message: "c".to_string(),
                    skipped_tokens: vec![],
                },
                0,
                1,
            ),
        ],
        modules_map: Some(map),
    };
    assert_eq!(module.items.len(), 3);
    let map = module.modules_map.as_ref().unwrap();
    assert_eq!(map.len(), 2);
    assert_eq!(map[&PathBuf::from("main.vais")], vec![0, 1]);
}

// ============================================================================
// Clone Tests for Complex Types
// ============================================================================

#[test]
fn test_function_clone() {
    let func = Function {
        name: sp_str("test"),
        generics: vec![GenericParam::new_type(sp_str("T"), vec![])],
        params: vec![Param {
            name: sp_str("x"),
            ty: named_type("T"),
            is_mut: false,
            is_vararg: false,
            ownership: Ownership::Regular,
            default_value: None,
        }],
        ret_type: Some(named_type("T")),
        body: FunctionBody::Expr(Box::new(sp_expr(Expr::Ident("x".to_string())))),
        is_pub: false,
        is_async: false,
        attributes: vec![],
        where_clause: vec![],
    };
    let cloned = func.clone();
    assert_eq!(func, cloned);
}

#[test]
fn test_struct_clone() {
    let s = Struct {
        name: sp_str("Point"),
        generics: vec![],
        fields: vec![
            Field {
                name: sp_str("x"),
                ty: named_type("f64"),
                is_pub: true,
            },
            Field {
                name: sp_str("y"),
                ty: named_type("f64"),
                is_pub: true,
            },
        ],
        methods: vec![],
        is_pub: true,
        attributes: vec![],
        where_clause: vec![],
    };
    let cloned = s.clone();
    assert_eq!(s, cloned);
}

#[test]
fn test_enum_clone() {
    let e = Enum {
        name: sp_str("Option"),
        generics: vec![GenericParam::new_type(sp_str("T"), vec![])],
        variants: vec![
            Variant {
                name: sp_str("None"),
                fields: VariantFields::Unit,
            },
            Variant {
                name: sp_str("Some"),
                fields: VariantFields::Tuple(vec![named_type("T")]),
            },
        ],
        is_pub: true,
        attributes: vec![],
    };
    let cloned = e.clone();
    assert_eq!(e, cloned);
}

// ============================================================================
// CallArgs Tests
// ============================================================================

#[test]
fn test_call_args_positional_empty() {
    let args = CallArgs::Positional(vec![]);
    if let CallArgs::Positional(items) = &args {
        assert!(items.is_empty());
    }
}

#[test]
fn test_call_args_named() {
    let args = CallArgs::Named {
        positional: vec![sp_expr(Expr::Int(1))],
        named: vec![NamedArg {
            name: sp_str("key"),
            value: sp_expr(Expr::String("val".to_string())),
        }],
    };
    if let CallArgs::Named { positional, named } = &args {
        assert_eq!(positional.len(), 1);
        assert_eq!(named.len(), 1);
        assert_eq!(named[0].name.node, "key");
    }
}

// ============================================================================
// FunctionBody Tests
// ============================================================================

#[test]
fn test_function_body_expr() {
    let body = FunctionBody::Expr(Box::new(sp_expr(Expr::Int(42))));
    assert!(matches!(body, FunctionBody::Expr(_)));
}

#[test]
fn test_function_body_block() {
    let body = FunctionBody::Block(vec![sp_stmt(Stmt::Return(Some(Box::new(sp_expr(
        Expr::Int(0),
    )))))]);
    if let FunctionBody::Block(stmts) = &body {
        assert_eq!(stmts.len(), 1);
    }
}

// ============================================================================
// Param Tests
// ============================================================================

#[test]
fn test_param_with_default() {
    let param = Param {
        name: sp_str("x"),
        ty: named_type("i64"),
        is_mut: false,
        is_vararg: false,
        ownership: Ownership::Regular,
        default_value: Some(Box::new(sp_expr(Expr::Int(0)))),
    };
    assert!(param.default_value.is_some());
}

#[test]
fn test_param_vararg() {
    let param = Param {
        name: sp_str("args"),
        ty: named_type("i64"),
        is_mut: false,
        is_vararg: true,
        ownership: Ownership::Regular,
        default_value: None,
    };
    assert!(param.is_vararg);
}

#[test]
fn test_param_mutable() {
    let param = Param {
        name: sp_str("x"),
        ty: named_type("i64"),
        is_mut: true,
        is_vararg: false,
        ownership: Ownership::Regular,
        default_value: None,
    };
    assert!(param.is_mut);
}

#[test]
fn test_param_linear_ownership() {
    let param = Param {
        name: sp_str("file"),
        ty: named_type("File"),
        is_mut: false,
        is_vararg: false,
        ownership: Ownership::Linear,
        default_value: None,
    };
    assert_eq!(param.ownership, Ownership::Linear);
}

// ============================================================================
// Struct/Field Tests
// ============================================================================

#[test]
fn test_struct_with_where_clause() {
    let s = Struct {
        name: sp_str("Container"),
        generics: vec![GenericParam::new_type(sp_str("T"), vec![])],
        fields: vec![Field {
            name: sp_str("value"),
            ty: named_type("T"),
            is_pub: false,
        }],
        methods: vec![],
        is_pub: false,
        attributes: vec![Attribute {
            name: "derive".to_string(),
            args: vec!["Clone".to_string()],
            expr: None,
        }],
        where_clause: vec![WherePredicate {
            ty: sp_str("T"),
            bounds: vec![sp_str("Clone")],
        }],
    };
    assert_eq!(s.where_clause.len(), 1);
    assert_eq!(s.attributes.len(), 1);
}

#[test]
fn test_field_pub() {
    let f = Field {
        name: sp_str("x"),
        ty: named_type("i64"),
        is_pub: true,
    };
    assert!(f.is_pub);
    let f2 = Field {
        name: sp_str("y"),
        ty: named_type("i64"),
        is_pub: false,
    };
    assert!(!f2.is_pub);
}

// ============================================================================
// Enum VariantFields Tests
// ============================================================================

#[test]
fn test_variant_fields_struct() {
    let vf = VariantFields::Struct(vec![
        Field {
            name: sp_str("x"),
            ty: named_type("i64"),
            is_pub: false,
        },
        Field {
            name: sp_str("y"),
            ty: named_type("i64"),
            is_pub: false,
        },
    ]);
    if let VariantFields::Struct(fields) = &vf {
        assert_eq!(fields.len(), 2);
    }
}

// ============================================================================
// Debug Tests
// ============================================================================

#[test]
fn test_span_debug() {
    let s = Span::new(0, 10);
    let debug = format!("{:?}", s);
    assert!(debug.contains("Span"));
    assert!(debug.contains("0"));
    assert!(debug.contains("10"));
}

#[test]
fn test_binop_debug() {
    let debug = format!("{:?}", BinOp::Add);
    assert_eq!(debug, "Add");
}

#[test]
fn test_unaryop_debug() {
    let debug = format!("{:?}", UnaryOp::Neg);
    assert_eq!(debug, "Neg");
}

#[test]
fn test_type_debug() {
    let ty = Type::Unit;
    let debug = format!("{:?}", ty);
    assert_eq!(debug, "Unit");
}
