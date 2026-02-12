//! Integration tests for vais-ast crate
//!
//! This test suite validates the AST data structures, constructors, and methods
//! for the Vais Abstract Syntax Tree. Tests cover Span, Spanned, all Item variants,
//! Expression types, Statement types, Pattern types, Type variants, and helper utilities.

use vais_ast::*;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a span for testing
fn span(start: usize, end: usize) -> Span {
    Span::new(start, end)
}

/// Create a spanned node for testing
fn spanned<T>(node: T, start: usize, end: usize) -> Spanned<T> {
    Spanned::new(node, span(start, end))
}

/// Create a spanned string for testing
fn sp_string(s: &str, start: usize, end: usize) -> Spanned<String> {
    spanned(s.to_string(), start, end)
}

// ============================================================================
// Span Tests
// ============================================================================

#[test]
fn test_span_new() {
    let s = Span::new(10, 20);
    assert_eq!(s.start, 10);
    assert_eq!(s.end, 20);
}

#[test]
fn test_span_default() {
    let s = Span::default();
    assert_eq!(s.start, 0);
    assert_eq!(s.end, 0);
}

#[test]
fn test_span_merge() {
    let s1 = Span::new(10, 20);
    let s2 = Span::new(15, 30);
    let merged = s1.merge(s2);
    assert_eq!(merged.start, 10);
    assert_eq!(merged.end, 30);
}

#[test]
fn test_span_merge_reversed() {
    let s1 = Span::new(20, 30);
    let s2 = Span::new(10, 25);
    let merged = s1.merge(s2);
    assert_eq!(merged.start, 10);
    assert_eq!(merged.end, 30);
}

#[test]
fn test_span_equality() {
    let s1 = Span::new(5, 10);
    let s2 = Span::new(5, 10);
    let s3 = Span::new(5, 11);
    assert_eq!(s1, s2);
    assert_ne!(s1, s3);
}

#[test]
fn test_span_clone() {
    let s1 = Span::new(100, 200);
    let s2 = s1.clone();
    assert_eq!(s1, s2);
}

// ============================================================================
// Spanned Tests
// ============================================================================

#[test]
fn test_spanned_new() {
    let node = 42;
    let s = Spanned::new(node, span(0, 10));
    assert_eq!(s.node, 42);
    assert_eq!(s.span, span(0, 10));
}

#[test]
fn test_spanned_clone() {
    let original = spanned("test".to_string(), 5, 9);
    let cloned = original.clone();
    assert_eq!(original, cloned);
    assert_eq!(cloned.node, "test");
    assert_eq!(cloned.span.start, 5);
    assert_eq!(cloned.span.end, 9);
}

#[test]
fn test_spanned_partial_eq() {
    let s1 = spanned(100, 0, 5);
    let s2 = spanned(100, 0, 5);
    let s3 = spanned(200, 0, 5);
    assert_eq!(s1, s2);
    assert_ne!(s1, s3);
}

// ============================================================================
// Attribute Tests
// ============================================================================

#[test]
fn test_attribute_simple() {
    let attr = Attribute {
        name: "inline".to_string(),
        args: vec![],
        expr: None,
    };
    assert_eq!(attr.name, "inline");
    assert!(attr.args.is_empty());
    assert!(attr.expr.is_none());
}

#[test]
fn test_attribute_with_args() {
    let attr = Attribute {
        name: "cfg".to_string(),
        args: vec!["test".to_string(), "debug".to_string()],
        expr: None,
    };
    assert_eq!(attr.name, "cfg");
    assert_eq!(attr.args.len(), 2);
    assert_eq!(attr.args[0], "test");
    assert_eq!(attr.args[1], "debug");
}

#[test]
fn test_attribute_with_expr() {
    let expr = spanned(Expr::Bool(true), 0, 4);
    let attr = Attribute {
        name: "requires".to_string(),
        args: vec!["x > 0".to_string()],
        expr: Some(Box::new(expr)),
    };
    assert_eq!(attr.name, "requires");
    assert!(attr.expr.is_some());
}

// ============================================================================
// Module Tests
// ============================================================================

#[test]
fn test_module_empty() {
    let module = Module {
        items: vec![],
        modules_map: None,
    };
    assert!(module.items.is_empty());
    assert!(module.modules_map.is_none());
}

#[test]
fn test_module_with_items() {
    let func = Function {
        name: sp_string("test", 0, 4),
        generics: vec![],
        params: vec![],
        ret_type: None,
        body: FunctionBody::Expr(Box::new(spanned(Expr::Unit, 5, 7))),
        is_pub: false,
        is_async: false,
        attributes: vec![],
    };

    let module = Module {
        items: vec![spanned(Item::Function(func), 0, 10)],
        modules_map: None,
    };
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Item Tests
// ============================================================================

#[test]
fn test_item_function() {
    let func = Function {
        name: sp_string("main", 0, 4),
        generics: vec![],
        params: vec![],
        ret_type: Some(spanned(Type::Unit, 5, 7)),
        body: FunctionBody::Block(vec![]),
        is_pub: true,
        is_async: false,
        attributes: vec![],
    };

    let item = Item::Function(func.clone());
    match item {
        Item::Function(f) => {
            assert_eq!(f.name.node, "main");
            assert!(f.is_pub);
            assert!(!f.is_async);
        }
        _ => panic!("Expected Function item"),
    }
}

#[test]
fn test_item_struct() {
    let s = Struct {
        name: sp_string("Point", 0, 5),
        generics: vec![],
        fields: vec![Field {
            name: sp_string("x", 6, 7),
            ty: spanned(
                Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                8,
                11,
            ),
            is_pub: true,
        }],
        methods: vec![],
        is_pub: true,
        attributes: vec![],
    };

    let item = Item::Struct(s);
    match item {
        Item::Struct(struct_def) => {
            assert_eq!(struct_def.name.node, "Point");
            assert_eq!(struct_def.fields.len(), 1);
            assert_eq!(struct_def.fields[0].name.node, "x");
        }
        _ => panic!("Expected Struct item"),
    }
}

#[test]
fn test_item_enum() {
    let e = Enum {
        name: sp_string("Option", 0, 6),
        generics: vec![],
        variants: vec![
            Variant {
                name: sp_string("None", 7, 11),
                fields: VariantFields::Unit,
            },
            Variant {
                name: sp_string("Some", 12, 16),
                fields: VariantFields::Tuple(vec![spanned(
                    Type::Named {
                        name: "T".to_string(),
                        generics: vec![],
                    },
                    17,
                    18,
                )]),
            },
        ],
        is_pub: true,
    };

    let item = Item::Enum(e);
    match item {
        Item::Enum(enum_def) => {
            assert_eq!(enum_def.name.node, "Option");
            assert_eq!(enum_def.variants.len(), 2);
            assert_eq!(enum_def.variants[0].name.node, "None");
            assert_eq!(enum_def.variants[1].name.node, "Some");
        }
        _ => panic!("Expected Enum item"),
    }
}

#[test]
fn test_item_union() {
    let u = Union {
        name: sp_string("Value", 0, 5),
        generics: vec![],
        fields: vec![Field {
            name: sp_string("int_val", 6, 13),
            ty: spanned(
                Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                14,
                17,
            ),
            is_pub: false,
        }],
        is_pub: false,
    };

    let item = Item::Union(u);
    match item {
        Item::Union(union_def) => {
            assert_eq!(union_def.name.node, "Value");
            assert!(!union_def.is_pub);
            assert_eq!(union_def.fields.len(), 1);
        }
        _ => panic!("Expected Union item"),
    }
}

#[test]
fn test_item_type_alias() {
    let ta = TypeAlias {
        name: sp_string("Int", 0, 3),
        generics: vec![],
        ty: spanned(
            Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            4,
            7,
        ),
        is_pub: true,
    };

    let item = Item::TypeAlias(ta);
    match item {
        Item::TypeAlias(type_alias) => {
            assert_eq!(type_alias.name.node, "Int");
            assert!(type_alias.is_pub);
        }
        _ => panic!("Expected TypeAlias item"),
    }
}

#[test]
fn test_item_use() {
    let use_stmt = Use {
        path: vec![sp_string("std", 0, 3), sp_string("io", 5, 7)],
        alias: None,
        items: None,
    };

    let item = Item::Use(use_stmt);
    match item {
        Item::Use(u) => {
            assert_eq!(u.path.len(), 2);
            assert_eq!(u.path[0].node, "std");
            assert_eq!(u.path[1].node, "io");
        }
        _ => panic!("Expected Use item"),
    }
}

#[test]
fn test_item_trait() {
    let t = Trait {
        name: sp_string("Display", 0, 7),
        generics: vec![],
        super_traits: vec![],
        associated_types: vec![],
        methods: vec![],
        is_pub: true,
    };

    let item = Item::Trait(t);
    match item {
        Item::Trait(trait_def) => {
            assert_eq!(trait_def.name.node, "Display");
            assert!(trait_def.is_pub);
        }
        _ => panic!("Expected Trait item"),
    }
}

#[test]
fn test_item_impl() {
    let impl_block = Impl {
        target_type: spanned(
            Type::Named {
                name: "Point".to_string(),
                generics: vec![],
            },
            0,
            5,
        ),
        trait_name: None,
        generics: vec![],
        associated_types: vec![],
        methods: vec![],
    };

    let item = Item::Impl(impl_block);
    match item {
        Item::Impl(impl_def) => {
            assert!(impl_def.trait_name.is_none());
            assert!(impl_def.methods.is_empty());
        }
        _ => panic!("Expected Impl item"),
    }
}

#[test]
fn test_item_const() {
    let const_def = ConstDef {
        name: sp_string("PI", 0, 2),
        ty: spanned(
            Type::Named {
                name: "f64".to_string(),
                generics: vec![],
            },
            3,
            6,
        ),
        value: spanned(Expr::Float(3.14159), 7, 14),
        is_pub: true,
        attributes: vec![],
    };

    let item = Item::Const(const_def);
    match item {
        Item::Const(c) => {
            assert_eq!(c.name.node, "PI");
            assert!(c.is_pub);
        }
        _ => panic!("Expected Const item"),
    }
}

#[test]
fn test_item_global() {
    let global_def = GlobalDef {
        name: sp_string("COUNTER", 0, 7),
        ty: spanned(
            Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            8,
            11,
        ),
        value: spanned(Expr::Int(0), 12, 13),
        is_pub: false,
        is_mutable: true,
    };

    let item = Item::Global(global_def);
    match item {
        Item::Global(g) => {
            assert_eq!(g.name.node, "COUNTER");
            assert!(g.is_mutable);
            assert!(!g.is_pub);
        }
        _ => panic!("Expected Global item"),
    }
}

#[test]
fn test_item_error() {
    let item = Item::Error {
        message: "Parse error".to_string(),
        skipped_tokens: vec!["bad".to_string(), "tokens".to_string()],
    };

    match item {
        Item::Error {
            message,
            skipped_tokens,
        } => {
            assert_eq!(message, "Parse error");
            assert_eq!(skipped_tokens.len(), 2);
        }
        _ => panic!("Expected Error item"),
    }
}

// ============================================================================
// Function Tests
// ============================================================================

#[test]
fn test_function_simple() {
    let func = Function {
        name: sp_string("add", 0, 3),
        generics: vec![],
        params: vec![Param {
            name: sp_string("a", 4, 5),
            ty: spanned(
                Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                6,
                9,
            ),
            is_mut: false,
            is_vararg: false,
            ownership: Ownership::Regular,
            default_value: None,
        }],
        ret_type: Some(spanned(
            Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            10,
            13,
        )),
        body: FunctionBody::Expr(Box::new(spanned(Expr::Ident("a".to_string()), 14, 15))),
        is_pub: false,
        is_async: false,
        attributes: vec![],
    };

    assert_eq!(func.name.node, "add");
    assert_eq!(func.params.len(), 1);
    assert!(!func.is_async);
}

#[test]
fn test_function_with_generics() {
    let gen_param = GenericParam::new_type(sp_string("T", 0, 1), vec![sp_string("Clone", 2, 7)]);

    let func = Function {
        name: sp_string("identity", 0, 8),
        generics: vec![gen_param],
        params: vec![],
        ret_type: None,
        body: FunctionBody::Block(vec![]),
        is_pub: true,
        is_async: false,
        attributes: vec![],
    };

    assert_eq!(func.generics.len(), 1);
    assert_eq!(func.generics[0].name.node, "T");
}

#[test]
fn test_function_async() {
    let func = Function {
        name: sp_string("fetch", 0, 5),
        generics: vec![],
        params: vec![],
        ret_type: None,
        body: FunctionBody::Block(vec![]),
        is_pub: true,
        is_async: true,
        attributes: vec![],
    };

    assert!(func.is_async);
}

// ============================================================================
// GenericParam Tests
// ============================================================================

#[test]
fn test_generic_param_type() {
    let param = GenericParam::new_type(sp_string("T", 0, 1), vec![sp_string("Display", 2, 9)]);

    assert_eq!(param.name.node, "T");
    assert_eq!(param.bounds.len(), 1);
    assert!(!param.is_const());
    assert!(!param.is_covariant());
    assert!(!param.is_contravariant());
}

#[test]
fn test_generic_param_const() {
    let param = GenericParam::new_const(
        sp_string("N", 0, 1),
        spanned(
            Type::Named {
                name: "usize".to_string(),
                generics: vec![],
            },
            2,
            7,
        ),
    );

    assert_eq!(param.name.node, "N");
    assert!(param.is_const());
}

#[test]
fn test_generic_param_lifetime() {
    let param = GenericParam::new_lifetime(sp_string("'a", 0, 2), vec!["'b".to_string()]);

    assert_eq!(param.name.node, "'a");
    match param.kind {
        GenericParamKind::Lifetime { bounds } => {
            assert_eq!(bounds.len(), 1);
            assert_eq!(bounds[0], "'b");
        }
        _ => panic!("Expected Lifetime kind"),
    }
}

#[test]
fn test_generic_param_variance() {
    let param =
        GenericParam::new_type_with_variance(sp_string("T", 0, 1), vec![], Variance::Covariant);

    assert!(param.is_covariant());
    assert!(!param.is_contravariant());
}

// ============================================================================
// Variance Tests
// ============================================================================

#[test]
fn test_variance_invariant() {
    let v = Variance::Invariant;
    assert_eq!(v, Variance::default());
}

#[test]
fn test_variance_covariant() {
    let v = Variance::Covariant;
    assert_ne!(v, Variance::Invariant);
}

#[test]
fn test_variance_contravariant() {
    let v = Variance::Contravariant;
    assert_ne!(v, Variance::Covariant);
}

// ============================================================================
// Enum and Variant Tests
// ============================================================================

#[test]
fn test_variant_unit() {
    let variant = Variant {
        name: sp_string("Unit", 0, 4),
        fields: VariantFields::Unit,
    };

    assert_eq!(variant.name.node, "Unit");
    match variant.fields {
        VariantFields::Unit => {}
        _ => panic!("Expected Unit variant"),
    }
}

#[test]
fn test_variant_tuple() {
    let variant = Variant {
        name: sp_string("Tuple", 0, 5),
        fields: VariantFields::Tuple(vec![spanned(
            Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            6,
            9,
        )]),
    };

    match variant.fields {
        VariantFields::Tuple(types) => {
            assert_eq!(types.len(), 1);
        }
        _ => panic!("Expected Tuple variant"),
    }
}

#[test]
fn test_variant_struct() {
    let variant = Variant {
        name: sp_string("Struct", 0, 6),
        fields: VariantFields::Struct(vec![Field {
            name: sp_string("field", 7, 12),
            ty: spanned(
                Type::Named {
                    name: "str".to_string(),
                    generics: vec![],
                },
                13,
                16,
            ),
            is_pub: true,
        }]),
    };

    match variant.fields {
        VariantFields::Struct(fields) => {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].name.node, "field");
        }
        _ => panic!("Expected Struct variant"),
    }
}

// ============================================================================
// CallArgs Tests
// ============================================================================

#[test]
fn test_call_args_positional() {
    let args = CallArgs::Positional(vec![
        spanned(Expr::Int(1), 0, 1),
        spanned(Expr::Int(2), 2, 3),
    ]);

    match args {
        CallArgs::Positional(exprs) => {
            assert_eq!(exprs.len(), 2);
        }
        _ => panic!("Expected Positional args"),
    }
}

#[test]
fn test_call_args_named() {
    let args = CallArgs::Named {
        positional: vec![spanned(Expr::Int(1), 0, 1)],
        named: vec![NamedArg {
            name: sp_string("x", 2, 3),
            value: spanned(Expr::Int(2), 4, 5),
        }],
    };

    match args {
        CallArgs::Named { positional, named } => {
            assert_eq!(positional.len(), 1);
            assert_eq!(named.len(), 1);
            assert_eq!(named[0].name.node, "x");
        }
        _ => panic!("Expected Named args"),
    }
}

// ============================================================================
// Type Tests
// ============================================================================

#[test]
fn test_type_named() {
    let ty = Type::Named {
        name: "i64".to_string(),
        generics: vec![],
    };

    match ty {
        Type::Named { name, generics } => {
            assert_eq!(name, "i64");
            assert!(generics.is_empty());
        }
        _ => panic!("Expected Named type"),
    }
}

#[test]
fn test_type_array() {
    let ty = Type::Array(Box::new(spanned(
        Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        0,
        3,
    )));

    match ty {
        Type::Array(_) => {}
        _ => panic!("Expected Array type"),
    }
}

#[test]
fn test_type_tuple() {
    let ty = Type::Tuple(vec![
        spanned(
            Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            0,
            3,
        ),
        spanned(
            Type::Named {
                name: "str".to_string(),
                generics: vec![],
            },
            4,
            7,
        ),
    ]);

    match ty {
        Type::Tuple(types) => {
            assert_eq!(types.len(), 2);
        }
        _ => panic!("Expected Tuple type"),
    }
}

#[test]
fn test_type_optional() {
    let ty = Type::Optional(Box::new(spanned(
        Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        0,
        3,
    )));

    match ty {
        Type::Optional(_) => {}
        _ => panic!("Expected Optional type"),
    }
}

#[test]
fn test_type_result() {
    let ty = Type::Result(Box::new(spanned(
        Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        0,
        3,
    )));

    match ty {
        Type::Result(_) => {}
        _ => panic!("Expected Result type"),
    }
}

#[test]
fn test_type_ref() {
    let ty = Type::Ref(Box::new(spanned(
        Type::Named {
            name: "str".to_string(),
            generics: vec![],
        },
        0,
        3,
    )));

    match ty {
        Type::Ref(_) => {}
        _ => panic!("Expected Ref type"),
    }
}

#[test]
fn test_type_unit() {
    let ty = Type::Unit;
    assert_eq!(ty, Type::Unit);
}

#[test]
fn test_type_inferred() {
    let ty = Type::Infer;
    assert_eq!(ty, Type::Infer);
}

// ============================================================================
// Pattern Tests
// ============================================================================

#[test]
fn test_pattern_wildcard() {
    let pat = Pattern::Wildcard;
    assert_eq!(pat, Pattern::Wildcard);
}

#[test]
fn test_pattern_ident() {
    let pat = Pattern::Ident("x".to_string());
    match pat {
        Pattern::Ident(name) => assert_eq!(name, "x"),
        _ => panic!("Expected Ident pattern"),
    }
}

#[test]
fn test_pattern_literal() {
    let pat = Pattern::Literal(Literal::Int(42));
    match pat {
        Pattern::Literal(Literal::Int(n)) => assert_eq!(n, 42),
        _ => panic!("Expected Literal pattern"),
    }
}

#[test]
fn test_pattern_tuple() {
    let pat = Pattern::Tuple(vec![
        spanned(Pattern::Ident("a".to_string()), 0, 1),
        spanned(Pattern::Wildcard, 2, 3),
    ]);

    match pat {
        Pattern::Tuple(pats) => {
            assert_eq!(pats.len(), 2);
        }
        _ => panic!("Expected Tuple pattern"),
    }
}

#[test]
fn test_pattern_struct() {
    let pat = Pattern::Struct {
        name: sp_string("Point", 0, 5),
        fields: vec![(
            sp_string("x", 6, 7),
            Some(spanned(Pattern::Ident("a".to_string()), 8, 9)),
        )],
    };

    match pat {
        Pattern::Struct { name, fields } => {
            assert_eq!(name.node, "Point");
            assert_eq!(fields.len(), 1);
        }
        _ => panic!("Expected Struct pattern"),
    }
}

#[test]
fn test_pattern_variant() {
    let pat = Pattern::Variant {
        name: sp_string("Some", 0, 4),
        fields: vec![spanned(Pattern::Ident("x".to_string()), 5, 6)],
    };

    match pat {
        Pattern::Variant { name, fields } => {
            assert_eq!(name.node, "Some");
            assert_eq!(fields.len(), 1);
        }
        _ => panic!("Expected Variant pattern"),
    }
}

// ============================================================================
// Stmt Tests
// ============================================================================

#[test]
fn test_stmt_let() {
    let stmt = Stmt::Let {
        name: sp_string("x", 0, 1),
        ty: None,
        value: Box::new(spanned(Expr::Int(42), 2, 4)),
        is_mut: false,
        ownership: Ownership::Regular,
    };

    match stmt {
        Stmt::Let { name, is_mut, .. } => {
            assert_eq!(name.node, "x");
            assert!(!is_mut);
        }
        _ => panic!("Expected Let stmt"),
    }
}

#[test]
fn test_stmt_expr() {
    let stmt = Stmt::Expr(Box::new(spanned(Expr::Int(42), 0, 2)));

    match stmt {
        Stmt::Expr(_) => {}
        _ => panic!("Expected Expr stmt"),
    }
}

#[test]
fn test_stmt_return() {
    let stmt = Stmt::Return(Some(Box::new(spanned(Expr::Int(42), 0, 2))));

    match stmt {
        Stmt::Return(Some(_)) => {}
        _ => panic!("Expected Return stmt"),
    }
}

#[test]
fn test_stmt_break() {
    let stmt = Stmt::Break(None);

    match stmt {
        Stmt::Break(None) => {}
        _ => panic!("Expected Break stmt"),
    }
}

#[test]
fn test_stmt_continue() {
    let stmt = Stmt::Continue;
    assert_eq!(stmt, Stmt::Continue);
}

// ============================================================================
// Expr Tests
// ============================================================================

#[test]
fn test_expr_int_literal() {
    let expr = Expr::Int(42);
    match expr {
        Expr::Int(n) => assert_eq!(n, 42),
        _ => panic!("Expected Int expr"),
    }
}

#[test]
fn test_expr_float_literal() {
    let expr = Expr::Float(3.14);
    match expr {
        Expr::Float(f) => assert!((f - 3.14).abs() < f64::EPSILON),
        _ => panic!("Expected Float expr"),
    }
}

#[test]
fn test_expr_bool_literal() {
    let expr = Expr::Bool(true);
    match expr {
        Expr::Bool(b) => assert!(b),
        _ => panic!("Expected Bool expr"),
    }
}

#[test]
fn test_expr_string_literal() {
    let expr = Expr::String("hello".to_string());
    match expr {
        Expr::String(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected String expr"),
    }
}

#[test]
fn test_expr_unit() {
    let expr = Expr::Unit;
    assert_eq!(expr, Expr::Unit);
}

#[test]
fn test_expr_ident() {
    let expr = Expr::Ident("x".to_string());
    match expr {
        Expr::Ident(name) => assert_eq!(name, "x"),
        _ => panic!("Expected Ident expr"),
    }
}

#[test]
fn test_expr_binary_op() {
    let expr = Expr::Binary {
        op: BinOp::Add,
        left: Box::new(spanned(Expr::Int(1), 0, 1)),
        right: Box::new(spanned(Expr::Int(2), 2, 3)),
    };

    match expr {
        Expr::Binary { op, .. } => assert_eq!(op, BinOp::Add),
        _ => panic!("Expected Binary expr"),
    }
}

#[test]
fn test_expr_unary_op() {
    let expr = Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(spanned(Expr::Int(5), 0, 1)),
    };

    match expr {
        Expr::Unary { op, .. } => assert_eq!(op, UnaryOp::Neg),
        _ => panic!("Expected Unary expr"),
    }
}

#[test]
fn test_expr_if() {
    let expr = Expr::If {
        cond: Box::new(spanned(Expr::Bool(true), 0, 4)),
        then: vec![spanned(
            Stmt::Expr(Box::new(spanned(Expr::Int(1), 5, 6))),
            5,
            6,
        )],
        else_: None,
    };

    match expr {
        Expr::If {
            cond: _,
            then,
            else_,
        } => {
            assert_eq!(then.len(), 1);
            assert!(else_.is_none());
        }
        _ => panic!("Expected If expr"),
    }
}

#[test]
fn test_expr_call() {
    let expr = Expr::Call {
        func: Box::new(spanned(Expr::Ident("f".to_string()), 0, 1)),
        args: vec![spanned(Expr::Int(42), 2, 4)],
    };

    match expr {
        Expr::Call { func: _, args } => {
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected Call expr"),
    }
}

#[test]
fn test_expr_array() {
    let expr = Expr::Array(vec![
        spanned(Expr::Int(1), 0, 1),
        spanned(Expr::Int(2), 2, 3),
    ]);

    match expr {
        Expr::Array(elements) => {
            assert_eq!(elements.len(), 2);
        }
        _ => panic!("Expected Array expr"),
    }
}

#[test]
fn test_expr_tuple() {
    let expr = Expr::Tuple(vec![
        spanned(Expr::Int(1), 0, 1),
        spanned(Expr::String("test".to_string()), 2, 8),
    ]);

    match expr {
        Expr::Tuple(elements) => {
            assert_eq!(elements.len(), 2);
        }
        _ => panic!("Expected Tuple expr"),
    }
}

#[test]
fn test_expr_match() {
    let expr = Expr::Match {
        expr: Box::new(spanned(Expr::Ident("x".to_string()), 0, 1)),
        arms: vec![MatchArm {
            pattern: spanned(Pattern::Wildcard, 2, 3),
            guard: None,
            body: Box::new(spanned(Expr::Int(0), 4, 5)),
        }],
    };

    match expr {
        Expr::Match { arms, .. } => {
            assert_eq!(arms.len(), 1);
        }
        _ => panic!("Expected Match expr"),
    }
}

// ============================================================================
// Ownership Tests
// ============================================================================

#[test]
fn test_ownership_default() {
    let o = Ownership::default();
    assert_eq!(o, Ownership::Regular);
}

#[test]
fn test_ownership_variants() {
    assert_ne!(Ownership::Regular, Ownership::Linear);
    assert_ne!(Ownership::Linear, Ownership::Affine);
    assert_ne!(Ownership::Affine, Ownership::Move);
}

// ============================================================================
// BinOp Precedence Tests
// ============================================================================

#[test]
fn test_binop_precedence() {
    assert!(BinOp::Mul.precedence() > BinOp::Add.precedence());
    assert!(BinOp::Add.precedence() > BinOp::Eq.precedence());
    assert!(BinOp::Eq.precedence() > BinOp::And.precedence());
    assert!(BinOp::And.precedence() > BinOp::Or.precedence());
}

#[test]
fn test_binop_precedence_equality() {
    assert_eq!(BinOp::Add.precedence(), BinOp::Sub.precedence());
    assert_eq!(BinOp::Mul.precedence(), BinOp::Div.precedence());
    assert_eq!(BinOp::Eq.precedence(), BinOp::Neq.precedence());
}
