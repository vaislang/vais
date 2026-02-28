//! Comprehensive effect system coverage tests
//!
//! Targets uncovered lines in effects.rs (338 uncovered, 20% coverage)
//! Focus: EffectInferrer methods - builtin effects, register/get function effects,
//! infer_expr_effects, infer_stmt_effects via parsed AST

use std::collections::HashMap;
use vais_ast::*;
use vais_types::{Effect, EffectAnnotation, EffectInferrer, EffectSet, FunctionSig, ResolvedType};

fn span() -> Span {
    Span::new(0, 1)
}

fn spanned<T>(node: T) -> Spanned<T> {
    Spanned::new(node, span())
}

fn int_expr(n: i64) -> Spanned<Expr> {
    spanned(Expr::Int(n))
}

fn ident_expr(name: &str) -> Spanned<Expr> {
    spanned(Expr::Ident(name.to_string()))
}

fn empty_fns<'a>() -> HashMap<String, &'a FunctionSig> {
    HashMap::new()
}

// ============================================================================
// EffectInferrer creation & builtin effects
// ============================================================================

#[test]
fn test_new_inferrer() {
    let inferrer = EffectInferrer::new();
    assert!(inferrer.get_builtin_effects("print").is_some());
    assert!(inferrer.get_builtin_effects("malloc").is_some());
    assert!(inferrer.get_builtin_effects("abs").is_some());
    assert!(inferrer.get_builtin_effects("panic").is_some());
    assert!(inferrer.get_builtin_effects("memcpy").is_some());
}

#[test]
fn test_default_inferrer() {
    let inferrer = EffectInferrer::default();
    assert!(inferrer.get_builtin_effects("println").is_some());
}

#[test]
fn test_io_builtins() {
    let inferrer = EffectInferrer::new();
    for name in &[
        "print", "println", "eprint", "eprintln", "puts", "putchar", "printf",
    ] {
        let effects = inferrer.get_builtin_effects(name).unwrap();
        assert!(effects.contains(Effect::IO), "Expected IO for {}", name);
    }
}

#[test]
fn test_file_io_builtins() {
    let inferrer = EffectInferrer::new();
    for name in &[
        "fopen", "fclose", "fread", "fwrite", "open", "close", "read", "write",
    ] {
        let effects = inferrer.get_builtin_effects(name).unwrap();
        assert!(effects.contains(Effect::IO), "Expected IO for {}", name);
    }
}

#[test]
fn test_network_builtins() {
    let inferrer = EffectInferrer::new();
    for name in &[
        "socket", "bind", "listen", "accept", "connect", "send", "recv", "sendto", "recvfrom",
    ] {
        let effects = inferrer.get_builtin_effects(name).unwrap();
        assert!(effects.contains(Effect::IO), "Expected IO for {}", name);
    }
}

#[test]
fn test_alloc_builtins() {
    let inferrer = EffectInferrer::new();
    for name in &["malloc", "calloc", "realloc", "free", "alloc", "dealloc"] {
        let effects = inferrer.get_builtin_effects(name).unwrap();
        assert!(effects.contains(Effect::Alloc), "Expected Alloc for {}", name);
    }
}

#[test]
fn test_pure_math_builtins() {
    let inferrer = EffectInferrer::new();
    for name in &[
        "abs", "min", "max", "sqrt", "sin", "cos", "pow", "floor", "ceil", "round",
        "fabs", "clamp", "tan", "asin", "acos", "atan", "atan2", "exp", "log", "log2", "log10",
    ] {
        let effects = inferrer.get_builtin_effects(name).unwrap();
        assert!(effects.is_pure(), "Expected pure for {}", name);
    }
}

#[test]
fn test_unsafe_builtins() {
    let inferrer = EffectInferrer::new();
    for name in &[
        "memcpy", "memmove", "memset", "memcmp", "strlen", "strcpy", "strcat", "strcmp",
    ] {
        let effects = inferrer.get_builtin_effects(name).unwrap();
        assert!(effects.contains(Effect::Unsafe), "Expected Unsafe for {}", name);
    }
}

#[test]
fn test_panic_builtins() {
    let inferrer = EffectInferrer::new();
    for name in &["panic", "abort", "exit", "assert", "__panic"] {
        let effects = inferrer.get_builtin_effects(name).unwrap();
        assert!(effects.contains(Effect::Panic), "Expected Panic for {}", name);
    }
}

#[test]
fn test_unknown_builtin() {
    let inferrer = EffectInferrer::new();
    assert!(inferrer.get_builtin_effects("not_a_builtin").is_none());
}

// ============================================================================
// Register & get function effects
// ============================================================================

#[test]
fn test_register_and_get_function_effects() {
    let mut inferrer = EffectInferrer::new();
    inferrer.register_function_effects("my_func".to_string(), EffectSet::io());
    let effects = inferrer.get_function_effects("my_func").unwrap();
    assert!(effects.contains(Effect::IO));
}

#[test]
fn test_get_unregistered_function() {
    let inferrer = EffectInferrer::new();
    assert!(inferrer.get_function_effects("nonexistent").is_none());
}

// ============================================================================
// infer_expr_effects tests (using AST nodes directly)
// ============================================================================

#[test]
fn test_infer_literal_effects() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();

    assert!(inferrer.infer_expr_effects(&Expr::Int(42), &fns).is_pure());
    assert!(inferrer.infer_expr_effects(&Expr::Float(3.14), &fns).is_pure());
    assert!(inferrer.infer_expr_effects(&Expr::Bool(true), &fns).is_pure());
    assert!(inferrer.infer_expr_effects(&Expr::String("hi".into()), &fns).is_pure());
    assert!(inferrer.infer_expr_effects(&Expr::Unit, &fns).is_pure());
    // Char is not a separate Expr variant in this AST
}

#[test]
fn test_infer_ident_effects() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    assert!(inferrer.infer_expr_effects(&Expr::Ident("x".into()), &fns).is_pure());
}

#[test]
fn test_infer_binary_effects() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let bin = Expr::Binary {
        op: BinOp::Add,
        left: Box::new(int_expr(1)),
        right: Box::new(int_expr(2)),
    };
    assert!(inferrer.infer_expr_effects(&bin, &fns).is_pure());
}

#[test]
fn test_infer_unary_effects() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let un = Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(int_expr(1)),
    };
    assert!(inferrer.infer_expr_effects(&un, &fns).is_pure());
}

#[test]
fn test_infer_call_builtin_io() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let call = Expr::Call {
        func: Box::new(ident_expr("println")),
        args: vec![spanned(Expr::String("hello".into()))],
    };
    let effects = inferrer.infer_expr_effects(&call, &fns);
    assert!(effects.contains(Effect::IO));
}

#[test]
fn test_infer_call_pure_math() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let call = Expr::Call {
        func: Box::new(ident_expr("abs")),
        args: vec![int_expr(-5)],
    };
    assert!(inferrer.infer_expr_effects(&call, &fns).is_pure());
}

#[test]
fn test_infer_call_registered_function() {
    let mut inferrer = EffectInferrer::new();
    inferrer.register_function_effects("my_io_func".to_string(), EffectSet::io());
    let fns = empty_fns();
    let call = Expr::Call {
        func: Box::new(ident_expr("my_io_func")),
        args: vec![],
    };
    let effects = inferrer.infer_expr_effects(&call, &fns);
    assert!(effects.contains(Effect::IO));
}

#[test]
fn test_infer_call_with_sig_effects() {
    let mut inferrer = EffectInferrer::new();
    let sig = FunctionSig {
        name: "alloc_func".to_string(),
        generics: vec![],
        generic_bounds: HashMap::new(),
        params: vec![],
        ret: ResolvedType::I64,
        is_async: false,
        is_vararg: false,
        required_params: None,
        contracts: None,
        effect_annotation: EffectAnnotation::Infer,
        inferred_effects: Some(EffectSet::alloc()),
        hkt_params: HashMap::new(),
        generic_callees: vec![],
    };
    let mut fns = HashMap::new();
    fns.insert("alloc_func".to_string(), &sig);
    let call = Expr::Call {
        func: Box::new(ident_expr("alloc_func")),
        args: vec![],
    };
    let effects = inferrer.infer_expr_effects(&call, &fns);
    assert!(effects.contains(Effect::Alloc));
}

#[test]
fn test_infer_call_with_sig_no_effects() {
    let mut inferrer = EffectInferrer::new();
    let sig = FunctionSig {
        name: "unknown_func".to_string(),
        generics: vec![],
        generic_bounds: HashMap::new(),
        params: vec![],
        ret: ResolvedType::I64,
        is_async: false,
        is_vararg: false,
        required_params: None,
        contracts: None,
        effect_annotation: EffectAnnotation::Infer,
        inferred_effects: None,
        hkt_params: HashMap::new(),
        generic_callees: vec![],
    };
    let mut fns = HashMap::new();
    fns.insert("unknown_func".to_string(), &sig);
    let call = Expr::Call {
        func: Box::new(ident_expr("unknown_func")),
        args: vec![],
    };
    let effects = inferrer.infer_expr_effects(&call, &fns);
    assert!(!effects.is_pure()); // total effects
}

#[test]
fn test_infer_indirect_call() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let call = Expr::Call {
        func: Box::new(int_expr(42)),
        args: vec![],
    };
    let effects = inferrer.infer_expr_effects(&call, &fns);
    assert!(!effects.is_pure()); // total effects
}

#[test]
fn test_infer_method_call() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let method = Expr::MethodCall {
        receiver: Box::new(ident_expr("obj")),
        method: spanned("do_something".to_string()),
        args: vec![int_expr(1)],
    };
    let effects = inferrer.infer_expr_effects(&method, &fns);
    assert!(effects.contains(Effect::Read));
    assert!(effects.contains(Effect::Write));
}

#[test]
fn test_infer_static_method_call() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let static_call = Expr::StaticMethodCall {
        type_name: spanned("MyType".to_string()),
        method: spanned("create".to_string()),
        args: vec![int_expr(1)],
    };
    let effects = inferrer.infer_expr_effects(&static_call, &fns);
    assert!(effects.contains(Effect::Read));
    assert!(effects.contains(Effect::Write));
}

#[test]
fn test_infer_field_access() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let field = Expr::Field {
        expr: Box::new(ident_expr("obj")),
        field: spanned("x".to_string()),
    };
    assert!(inferrer.infer_expr_effects(&field, &fns).is_pure());
}

#[test]
fn test_infer_index_access() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let idx = Expr::Index {
        expr: Box::new(ident_expr("arr")),
        index: Box::new(int_expr(0)),
    };
    assert!(inferrer.infer_expr_effects(&idx, &fns).is_pure());
}

#[test]
fn test_infer_assign() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let assign = Expr::Assign {
        target: Box::new(ident_expr("x")),
        value: Box::new(int_expr(42)),
    };
    assert!(inferrer.infer_expr_effects(&assign, &fns).contains(Effect::Write));
}

#[test]
fn test_infer_assign_op() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let assign_op = Expr::AssignOp {
        op: BinOp::Add,
        target: Box::new(ident_expr("x")),
        value: Box::new(int_expr(1)),
    };
    assert!(inferrer.infer_expr_effects(&assign_op, &fns).contains(Effect::Write));
}

#[test]
fn test_infer_if_with_io() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let if_expr = Expr::If {
        cond: Box::new(spanned(Expr::Bool(true))),
        then: vec![spanned(Stmt::Expr(Box::new(spanned(Expr::Call {
            func: Box::new(ident_expr("println")),
            args: vec![],
        }))))],
        else_: None,
    };
    assert!(inferrer.infer_expr_effects(&if_expr, &fns).contains(Effect::IO));
}

#[test]
fn test_infer_if_else() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let if_expr = Expr::If {
        cond: Box::new(spanned(Expr::Bool(true))),
        then: vec![spanned(Stmt::Expr(Box::new(int_expr(1))))],
        else_: Some(IfElse::Else(vec![spanned(Stmt::Expr(
            Box::new(spanned(Expr::Call {
                func: Box::new(ident_expr("malloc")),
                args: vec![int_expr(100)],
            })),
        ))])),
    };
    assert!(inferrer.infer_expr_effects(&if_expr, &fns).contains(Effect::Alloc));
}

#[test]
fn test_infer_if_elseif() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let if_expr = Expr::If {
        cond: Box::new(spanned(Expr::Bool(true))),
        then: vec![spanned(Stmt::Expr(Box::new(int_expr(1))))],
        else_: Some(IfElse::ElseIf(
            Box::new(spanned(Expr::Bool(false))),
            vec![spanned(Stmt::Expr(Box::new(spanned(Expr::Call {
                func: Box::new(ident_expr("panic")),
                args: vec![],
            }))))],
            None,
        )),
    };
    assert!(inferrer.infer_expr_effects(&if_expr, &fns).contains(Effect::Panic));
}

#[test]
fn test_infer_block() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let block = Expr::Block(vec![
        spanned(Stmt::Expr(Box::new(spanned(Expr::Call {
            func: Box::new(ident_expr("print")),
            args: vec![],
        })))),
        spanned(Stmt::Expr(Box::new(int_expr(42)))),
    ]);
    assert!(inferrer.infer_expr_effects(&block, &fns).contains(Effect::IO));
}

#[test]
fn test_infer_tuple() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let tuple = Expr::Tuple(vec![int_expr(1), int_expr(2)]);
    assert!(inferrer.infer_expr_effects(&tuple, &fns).is_pure());
}

#[test]
fn test_infer_array() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let arr = Expr::Array(vec![int_expr(1), int_expr(2)]);
    assert!(inferrer.infer_expr_effects(&arr, &fns).is_pure());
}

#[test]
fn test_infer_cast() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let cast = Expr::Cast {
        expr: Box::new(int_expr(42)),
        ty: spanned(Type::Named {
            name: "f64".to_string(),
            generics: vec![],
        }),
    };
    assert!(inferrer.infer_expr_effects(&cast, &fns).is_pure());
}

#[test]
fn test_infer_lambda() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let lambda = Expr::Lambda {
        params: vec![],
        body: Box::new(int_expr(42)),
        captures: vec![],
        capture_mode: CaptureMode::ByValue,
    };
    assert!(inferrer.infer_expr_effects(&lambda, &fns).is_pure());
}

#[test]
fn test_infer_error_expr() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let error = Expr::Error {
        message: "error".to_string(),
        skipped_tokens: vec![],
    };
    assert!(!inferrer.infer_expr_effects(&error, &fns).is_pure());
}

#[test]
fn test_infer_macro_invoke() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let mac = Expr::MacroInvoke(MacroInvoke {
        name: spanned("test_macro".to_string()),
        delimiter: Delimiter::Paren,
        tokens: vec![],
    });
    assert!(!inferrer.infer_expr_effects(&mac, &fns).is_pure());
}

#[test]
fn test_infer_lazy_force() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    assert!(inferrer.infer_expr_effects(&Expr::Lazy(Box::new(int_expr(42))), &fns).is_pure());
    assert!(inferrer.infer_expr_effects(&Expr::Force(Box::new(ident_expr("x"))), &fns).is_pure());
}

#[test]
fn test_infer_map_literal() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let map = Expr::MapLit(vec![(
        spanned(Expr::String("key".into())),
        int_expr(1),
    )]);
    assert!(inferrer.infer_expr_effects(&map, &fns).is_pure());
}

#[test]
fn test_infer_string_interp() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let interp = Expr::StringInterp(vec![
        StringInterpPart::Lit("hello ".into()),
        StringInterpPart::Expr(Box::new(ident_expr("name"))),
    ]);
    assert!(inferrer.infer_expr_effects(&interp, &fns).is_pure());
}

// ============================================================================
// infer_stmt_effects tests
// ============================================================================

#[test]
fn test_infer_let_stmt() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let stmt = Stmt::Let {
        name: spanned("x".to_string()),
        ty: None,
        value: Box::new(spanned(Expr::Call {
            func: Box::new(ident_expr("malloc")),
            args: vec![int_expr(100)],
        })),
        is_mut: false,
        ownership: Ownership::Regular,
    };
    assert!(inferrer.infer_stmt_effects(&stmt, &fns).contains(Effect::Alloc));
}

#[test]
fn test_infer_expr_stmt() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let stmt = Stmt::Expr(Box::new(spanned(Expr::Call {
        func: Box::new(ident_expr("println")),
        args: vec![],
    })));
    assert!(inferrer.infer_stmt_effects(&stmt, &fns).contains(Effect::IO));
}

#[test]
fn test_infer_return_with_expr() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let stmt = Stmt::Return(Some(Box::new(spanned(Expr::Call {
        func: Box::new(ident_expr("panic")),
        args: vec![],
    }))));
    assert!(inferrer.infer_stmt_effects(&stmt, &fns).contains(Effect::Panic));
}

#[test]
fn test_infer_return_void() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    assert!(inferrer.infer_stmt_effects(&Stmt::Return(None), &fns).is_pure());
}

#[test]
fn test_infer_break_with_value() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    assert!(inferrer.infer_stmt_effects(&Stmt::Break(Some(Box::new(int_expr(42)))), &fns).is_pure());
}

#[test]
fn test_infer_break_no_value() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    assert!(inferrer.infer_stmt_effects(&Stmt::Break(None), &fns).is_pure());
}

#[test]
fn test_infer_continue() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    assert!(inferrer.infer_stmt_effects(&Stmt::Continue, &fns).is_pure());
}

#[test]
fn test_infer_defer_stmt() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let stmt = Stmt::Defer(Box::new(spanned(Expr::Call {
        func: Box::new(ident_expr("free")),
        args: vec![ident_expr("ptr")],
    })));
    assert!(inferrer.infer_stmt_effects(&stmt, &fns).contains(Effect::Alloc));
}

#[test]
fn test_infer_error_stmt() {
    let mut inferrer = EffectInferrer::new();
    let fns = empty_fns();
    let stmt = Stmt::Error {
        message: "error".to_string(),
        skipped_tokens: vec![],
    };
    assert!(!inferrer.infer_stmt_effects(&stmt, &fns).is_pure());
}

// ============================================================================
// EffectAnnotation tests
// ============================================================================

#[test]
fn test_effect_annotation_default() {
    assert_eq!(EffectAnnotation::default(), EffectAnnotation::Infer);
}

#[test]
fn test_effect_annotation_pure() {
    assert_eq!(EffectAnnotation::Pure, EffectAnnotation::Pure);
}

#[test]
fn test_effect_annotation_declared() {
    let ann = EffectAnnotation::Declared(EffectSet::io());
    match ann {
        EffectAnnotation::Declared(set) => assert!(set.contains(Effect::IO)),
        _ => panic!("Expected Declared"),
    }
}
