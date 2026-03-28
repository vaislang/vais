use crate::expr_helpers::*;
use crate::JsCodeGenerator;
use vais_ast::*;

#[test]
fn test_escape_js_string() {
    assert_eq!(escape_js_string("hello"), "hello");
    assert_eq!(escape_js_string("he\"llo"), "he\\\"llo");
    assert_eq!(escape_js_string("line\nnew"), "line\\nnew");
}

#[test]
fn test_sanitize_js_ident() {
    assert_eq!(sanitize_js_ident("foo"), "foo");
    assert_eq!(sanitize_js_ident("class"), "_class");
    assert_eq!(sanitize_js_ident("yield"), "_yield");
}

#[test]
fn test_binop_to_js() {
    assert_eq!(binop_to_js(&BinOp::Add), "+");
    assert_eq!(binop_to_js(&BinOp::Eq), "===");
    assert_eq!(binop_to_js(&BinOp::Neq), "!==");
    assert_eq!(binop_to_js(&BinOp::And), "&&");
}

#[test]
fn test_binop_all_variants() {
    assert_eq!(binop_to_js(&BinOp::Sub), "-");
    assert_eq!(binop_to_js(&BinOp::Mul), "*");
    assert_eq!(binop_to_js(&BinOp::Div), "/");
    assert_eq!(binop_to_js(&BinOp::Mod), "%");
    assert_eq!(binop_to_js(&BinOp::Lt), "<");
    assert_eq!(binop_to_js(&BinOp::Lte), "<=");
    assert_eq!(binop_to_js(&BinOp::Gt), ">");
    assert_eq!(binop_to_js(&BinOp::Gte), ">=");
    assert_eq!(binop_to_js(&BinOp::Or), "||");
    assert_eq!(binop_to_js(&BinOp::BitAnd), "&");
    assert_eq!(binop_to_js(&BinOp::BitOr), "|");
    assert_eq!(binop_to_js(&BinOp::BitXor), "^");
    assert_eq!(binop_to_js(&BinOp::Shl), "<<");
    assert_eq!(binop_to_js(&BinOp::Shr), ">>");
}

#[test]
fn test_unaryop_to_js() {
    assert_eq!(unaryop_to_js(&UnaryOp::Neg), "-");
    assert_eq!(unaryop_to_js(&UnaryOp::Not), "!");
    assert_eq!(unaryop_to_js(&UnaryOp::BitNot), "~");
}

#[test]
fn test_escape_js_string_special_chars() {
    assert_eq!(escape_js_string("tab\there"), "tab\\there");
    assert_eq!(escape_js_string("ret\rhere"), "ret\\rhere");
    assert_eq!(escape_js_string("null\0here"), "null\\0here");
    assert_eq!(escape_js_string("back\\slash"), "back\\\\slash");
}

#[test]
fn test_escape_template_literal() {
    assert_eq!(escape_template_literal("hello"), "hello");
    assert_eq!(escape_template_literal("back`tick"), "back\\`tick");
    assert_eq!(escape_template_literal("dollar$sign"), "dollar\\$sign");
    assert_eq!(escape_template_literal("back\\slash"), "back\\\\slash");
}

#[test]
fn test_sanitize_all_reserved_words() {
    let reserved = vec![
        ("class", "_class"),
        ("delete", "_delete"),
        ("export", "_export"),
        ("import", "_import"),
        ("new", "_new"),
        ("super", "_super"),
        ("switch", "_switch"),
        ("this", "_this"),
        ("throw", "_throw"),
        ("typeof", "_typeof"),
        ("var", "_var"),
        ("void", "_void"),
        ("with", "_with"),
        ("yield", "_yield"),
        ("await", "_await"),
        ("enum", "_enum"),
        ("implements", "_implements"),
        ("interface", "_interface"),
        ("package", "_package"),
        ("private", "_private"),
        ("protected", "_protected"),
        ("public", "_public"),
        ("static", "_static"),
        ("arguments", "_arguments"),
        ("eval", "_eval"),
    ];
    for (input, expected) in reserved {
        assert_eq!(sanitize_js_ident(input), expected, "Failed for {input}");
    }
}

#[test]
fn test_sanitize_non_reserved() {
    assert_eq!(sanitize_js_ident("my_var"), "my_var");
    assert_eq!(sanitize_js_ident("x"), "x");
    assert_eq!(sanitize_js_ident("counter"), "counter");
}

#[test]
fn test_generate_int_literal() {
    let mut gen = JsCodeGenerator::new();
    assert_eq!(gen.generate_expr(&Expr::Int(42)).unwrap(), "42");
    assert_eq!(gen.generate_expr(&Expr::Int(0)).unwrap(), "0");
    assert_eq!(gen.generate_expr(&Expr::Int(-1)).unwrap(), "-1");
}

#[test]
fn test_generate_float_literal() {
    let mut gen = JsCodeGenerator::new();
    assert_eq!(gen.generate_expr(&Expr::Float(3.14)).unwrap(), "3.14");
    // Integer-valued float should get .0 appended
    let result = gen.generate_expr(&Expr::Float(1.0)).unwrap();
    assert!(result.contains('.') || result.contains('e'));
}

#[test]
fn test_generate_bool_literal() {
    let mut gen = JsCodeGenerator::new();
    assert_eq!(gen.generate_expr(&Expr::Bool(true)).unwrap(), "true");
    assert_eq!(gen.generate_expr(&Expr::Bool(false)).unwrap(), "false");
}

#[test]
fn test_generate_string_literal() {
    let mut gen = JsCodeGenerator::new();
    assert_eq!(
        gen.generate_expr(&Expr::String("hello".to_string()))
            .unwrap(),
        "\"hello\""
    );
    assert_eq!(
        gen.generate_expr(&Expr::String("say \"hi\"".to_string()))
            .unwrap(),
        "\"say \\\"hi\\\"\""
    );
}

#[test]
fn test_generate_unit() {
    let mut gen = JsCodeGenerator::new();
    assert_eq!(gen.generate_expr(&Expr::Unit).unwrap(), "undefined");
}

#[test]
fn test_generate_ident() {
    let mut gen = JsCodeGenerator::new();
    assert_eq!(
        gen.generate_expr(&Expr::Ident("foo".to_string())).unwrap(),
        "foo"
    );
    // Reserved word gets sanitized
    assert_eq!(
        gen.generate_expr(&Expr::Ident("class".to_string()))
            .unwrap(),
        "_class"
    );
}

#[test]
fn test_generate_self_call_with_function() {
    let mut gen = JsCodeGenerator::new();
    gen.current_function = Some("fibonacci".to_string());
    assert_eq!(gen.generate_expr(&Expr::SelfCall).unwrap(), "fibonacci");
}

#[test]
fn test_generate_self_call_without_function() {
    let mut gen = JsCodeGenerator::new();
    assert_eq!(
        gen.generate_expr(&Expr::SelfCall).unwrap(),
        "arguments.callee"
    );
}

#[test]
fn test_generate_binary_expr() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Binary {
        op: BinOp::Add,
        left: Box::new(Spanned::new(Expr::Int(1), Span::new(0, 1))),
        right: Box::new(Spanned::new(Expr::Int(2), Span::new(3, 4))),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "(1 + 2)");
}

#[test]
fn test_generate_unary_expr() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(Spanned::new(Expr::Int(5), Span::new(1, 2))),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "-5");

    let expr_not = Expr::Unary {
        op: UnaryOp::Not,
        expr: Box::new(Spanned::new(Expr::Bool(true), Span::new(1, 5))),
    };
    assert_eq!(gen.generate_expr(&expr_not).unwrap(), "!true");
}

#[test]
fn test_generate_ternary_expr() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Ternary {
        cond: Box::new(Spanned::new(Expr::Bool(true), Span::new(0, 4))),
        then: Box::new(Spanned::new(Expr::Int(1), Span::new(7, 8))),
        else_: Box::new(Spanned::new(Expr::Int(0), Span::new(11, 12))),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "(true ? 1 : 0)");
}

#[test]
fn test_generate_array_literal() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Array(vec![
        Spanned::new(Expr::Int(1), Span::new(1, 2)),
        Spanned::new(Expr::Int(2), Span::new(4, 5)),
        Spanned::new(Expr::Int(3), Span::new(7, 8)),
    ]);
    assert_eq!(gen.generate_expr(&expr).unwrap(), "[1, 2, 3]");
}

#[test]
fn test_generate_empty_array() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Array(vec![]);
    assert_eq!(gen.generate_expr(&expr).unwrap(), "[]");
}

#[test]
fn test_generate_tuple_as_array() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Tuple(vec![
        Spanned::new(Expr::Int(1), Span::new(1, 2)),
        Spanned::new(Expr::String("hello".to_string()), Span::new(4, 11)),
    ]);
    assert_eq!(gen.generate_expr(&expr).unwrap(), "[1, \"hello\"]");
}

#[test]
fn test_generate_function_call() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Call {
        func: Box::new(Spanned::new(
            Expr::Ident("print".to_string()),
            Span::new(0, 5),
        )),
        args: vec![Spanned::new(Expr::Int(42), Span::new(6, 8))],
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "print(42)");
}

#[test]
fn test_generate_method_call() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::MethodCall {
        receiver: Box::new(Spanned::new(
            Expr::Ident("vec".to_string()),
            Span::new(0, 3),
        )),
        method: Spanned::new("push".to_string(), Span::new(4, 8)),
        args: vec![Spanned::new(Expr::Int(5), Span::new(9, 10))],
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "vec.push(5)");
}

#[test]
fn test_generate_static_method_call() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::StaticMethodCall {
        type_name: Spanned::new("Vec".to_string(), Span::new(0, 3)),
        method: Spanned::new("new".to_string(), Span::new(5, 8)),
        args: vec![],
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "Vec._new()");
}

#[test]
fn test_generate_field_access() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Field {
        expr: Box::new(Spanned::new(
            Expr::Ident("point".to_string()),
            Span::new(0, 5),
        )),
        field: Spanned::new("x".to_string(), Span::new(6, 7)),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "point.x");
}

#[test]
fn test_generate_index() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Index {
        expr: Box::new(Spanned::new(
            Expr::Ident("arr".to_string()),
            Span::new(0, 3),
        )),
        index: Box::new(Spanned::new(Expr::Int(0), Span::new(4, 5))),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "arr[0]");
}

#[test]
fn test_generate_map_literal() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::MapLit(vec![(
        Spanned::new(Expr::String("key".to_string()), Span::new(0, 5)),
        Spanned::new(Expr::Int(1), Span::new(8, 9)),
    )]);
    assert_eq!(gen.generate_expr(&expr).unwrap(), "new Map([[\"key\", 1]])");
}

#[test]
fn test_generate_struct_literal() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::StructLit {
        name: Spanned::new("Point".to_string(), Span::new(0, 5)),
        fields: vec![
            (
                Spanned::new("x".to_string(), Span::new(6, 7)),
                Spanned::new(Expr::Int(1), Span::new(9, 10)),
            ),
            (
                Spanned::new("y".to_string(), Span::new(12, 13)),
                Spanned::new(Expr::Int(2), Span::new(15, 16)),
            ),
        ],
        enum_name: None,
    };
    let result = gen.generate_expr(&expr).unwrap();
    assert!(result.contains("new Point"));
    assert!(result.contains("x: 1"));
    assert!(result.contains("y: 2"));
}

#[test]
fn test_generate_lambda() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Lambda {
        params: vec![Param {
            name: Spanned::new("x".to_string(), Span::new(1, 2)),
            ty: Spanned::new(Type::Infer, Span::new(0, 0)),
            is_mut: false,
            is_vararg: false,
            ownership: Ownership::Regular,
            default_value: None,
        }],
        body: Box::new(Spanned::new(
            Expr::Binary {
                op: BinOp::Mul,
                left: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(5, 6))),
                right: Box::new(Spanned::new(Expr::Int(2), Span::new(9, 10))),
            },
            Span::new(5, 10),
        )),
        captures: vec![],
        capture_mode: CaptureMode::ByValue,
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "(x) => (x * 2)");
}

#[test]
fn test_generate_await() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Await(Box::new(Spanned::new(
        Expr::Call {
            func: Box::new(Spanned::new(
                Expr::Ident("fetch".to_string()),
                Span::new(0, 5),
            )),
            args: vec![],
        },
        Span::new(0, 7),
    )));
    assert_eq!(gen.generate_expr(&expr).unwrap(), "(await fetch())");
}

#[test]
fn test_generate_spawn() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Spawn(Box::new(Spanned::new(
        Expr::Ident("task".to_string()),
        Span::new(0, 4),
    )));
    assert_eq!(
        gen.generate_expr(&expr).unwrap(),
        "Promise.resolve().then(() => task)"
    );
}

#[test]
fn test_generate_try_operator() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Try(Box::new(Spanned::new(
        Expr::Ident("result".to_string()),
        Span::new(0, 6),
    )));
    assert_eq!(gen.generate_expr(&expr).unwrap(), "__unwrapOrThrow(result)");
}

#[test]
fn test_generate_unwrap() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Unwrap(Box::new(Spanned::new(
        Expr::Ident("opt".to_string()),
        Span::new(0, 3),
    )));
    assert_eq!(gen.generate_expr(&expr).unwrap(), "__unwrap(opt)");
}

#[test]
fn test_generate_ref_deref_noop() {
    let mut gen = JsCodeGenerator::new();
    let inner = Expr::Ident("x".to_string());
    let ref_expr = Expr::Ref(Box::new(Spanned::new(inner.clone(), Span::new(0, 1))));
    assert_eq!(gen.generate_expr(&ref_expr).unwrap(), "x");

    let deref_expr = Expr::Deref(Box::new(Spanned::new(
        Expr::Ident("x".to_string()),
        Span::new(0, 1),
    )));
    assert_eq!(gen.generate_expr(&deref_expr).unwrap(), "x");
}

#[test]
fn test_generate_spread() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Spread(Box::new(Spanned::new(
        Expr::Ident("args".to_string()),
        Span::new(0, 4),
    )));
    assert_eq!(gen.generate_expr(&expr).unwrap(), "...args");
}

#[test]
fn test_generate_cast_i32() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Cast {
        expr: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
        ty: Spanned::new(
            Type::Named {
                name: "i32".to_string(),
                generics: vec![],
            },
            Span::new(5, 8),
        ),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "(x | 0)");
}

#[test]
fn test_generate_cast_f64() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Cast {
        expr: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
        ty: Spanned::new(
            Type::Named {
                name: "f64".to_string(),
                generics: vec![],
            },
            Span::new(5, 8),
        ),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "Number(x)");
}

#[test]
fn test_generate_cast_bool() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Cast {
        expr: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
        ty: Spanned::new(
            Type::Named {
                name: "bool".to_string(),
                generics: vec![],
            },
            Span::new(5, 9),
        ),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "Boolean(x)");
}

#[test]
fn test_generate_cast_string() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Cast {
        expr: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
        ty: Spanned::new(
            Type::Named {
                name: "String".to_string(),
                generics: vec![],
            },
            Span::new(5, 11),
        ),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "String(x)");
}

#[test]
fn test_generate_cast_unknown_type() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Cast {
        expr: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
        ty: Spanned::new(
            Type::Named {
                name: "MyType".to_string(),
                generics: vec![],
            },
            Span::new(5, 11),
        ),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "x");
}

#[test]
fn test_generate_assign() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Assign {
        target: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
        value: Box::new(Spanned::new(Expr::Int(10), Span::new(4, 6))),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "x = 10");
}

#[test]
fn test_generate_assign_op() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::AssignOp {
        op: BinOp::Add,
        target: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
        value: Box::new(Spanned::new(Expr::Int(1), Span::new(5, 6))),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "x += 1");
}

#[test]
fn test_generate_yield() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Yield(Box::new(Spanned::new(Expr::Int(42), Span::new(6, 8))));
    assert_eq!(gen.generate_expr(&expr).unwrap(), "yield 42");
}

#[test]
fn test_generate_lazy() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Lazy(Box::new(Spanned::new(Expr::Int(42), Span::new(0, 2))));
    assert_eq!(gen.generate_expr(&expr).unwrap(), "(() => 42)");
}

#[test]
fn test_generate_force() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Force(Box::new(Spanned::new(
        Expr::Ident("lazy_val".to_string()),
        Span::new(0, 8),
    )));
    assert_eq!(gen.generate_expr(&expr).unwrap(), "lazy_val()");
}

#[test]
fn test_generate_assert_without_message() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Assert {
        condition: Box::new(Spanned::new(Expr::Bool(true), Span::new(0, 4))),
        message: None,
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "console.assert(true)");
}

#[test]
fn test_generate_assert_with_message() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Assert {
        condition: Box::new(Spanned::new(Expr::Bool(false), Span::new(0, 5))),
        message: Some(Box::new(Spanned::new(
            Expr::String("failed".to_string()),
            Span::new(7, 15),
        ))),
    };
    assert_eq!(
        gen.generate_expr(&expr).unwrap(),
        "console.assert(false, \"failed\")"
    );
}

#[test]
fn test_generate_comptime() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Comptime {
        body: Box::new(Spanned::new(Expr::Int(42), Span::new(0, 2))),
    };
    assert_eq!(gen.generate_expr(&expr).unwrap(), "42");
}

#[test]
fn test_generate_assume() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Assume(Box::new(Spanned::new(Expr::Bool(true), Span::new(0, 4))));
    assert_eq!(gen.generate_expr(&expr).unwrap(), "/* assume */");
}

#[test]
fn test_generate_old() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Old(Box::new(Spanned::new(
        Expr::Ident("x".to_string()),
        Span::new(0, 1),
    )));
    assert_eq!(gen.generate_expr(&expr).unwrap(), "/* old */");
}

#[test]
fn test_generate_error_expr() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Error {
        message: "something went wrong".to_string(),
        skipped_tokens: vec![],
    };
    let result = gen.generate_expr(&expr).unwrap();
    assert!(result.contains("codegen error"));
    assert!(result.contains("something went wrong"));
}

#[test]
fn test_generate_string_interp() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::StringInterp(vec![
        StringInterpPart::Lit("Hello, ".to_string()),
        StringInterpPart::Expr(Box::new(Spanned::new(
            Expr::Ident("name".to_string()),
            Span::new(0, 4),
        ))),
        StringInterpPart::Lit("!".to_string()),
    ]);
    assert_eq!(gen.generate_expr(&expr).unwrap(), "`Hello, ${name}!`");
}

#[test]
fn test_generate_block_expr_empty() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Block(vec![]);
    assert_eq!(gen.generate_expr(&expr).unwrap(), "undefined");
}

#[test]
fn test_generate_block_expr_single() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Block(vec![Spanned::new(
        Stmt::Expr(Box::new(Spanned::new(Expr::Int(42), Span::new(0, 2)))),
        Span::new(0, 3),
    )]);
    let result = gen.generate_expr(&expr).unwrap();
    assert!(result.contains("return 42;"));
}

#[test]
fn test_generate_while_loop() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::While {
        condition: Box::new(Spanned::new(Expr::Bool(true), Span::new(0, 4))),
        body: vec![Spanned::new(Stmt::Break(None), Span::new(5, 11))],
    };
    let result = gen.generate_expr(&expr).unwrap();
    assert!(result.contains("while (true)"));
    assert!(result.contains("break;"));
}

#[test]
fn test_generate_infinite_loop() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Loop {
        pattern: None,
        iter: None,
        body: vec![Spanned::new(Stmt::Continue, Span::new(0, 8))],
    };
    let result = gen.generate_expr(&expr).unwrap();
    assert!(result.contains("while (true)"));
    assert!(result.contains("continue;"));
}

#[test]
fn test_generate_for_of_loop() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Loop {
        pattern: Some(Spanned::new(
            Pattern::Ident("x".to_string()),
            Span::new(0, 1),
        )),
        iter: Some(Box::new(Spanned::new(
            Expr::Ident("items".to_string()),
            Span::new(3, 8),
        ))),
        body: vec![],
    };
    let result = gen.generate_expr(&expr).unwrap();
    assert!(result.contains("for (const x of items)"));
}

#[test]
fn test_generate_range_exclusive() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Range {
        start: Some(Box::new(Spanned::new(Expr::Int(0), Span::new(0, 1)))),
        end: Some(Box::new(Spanned::new(Expr::Int(10), Span::new(3, 5)))),
        inclusive: false,
    };
    let result = gen.generate_expr(&expr).unwrap();
    assert_eq!(result, "__range(0, 10)");
    // Verify helper was registered
    assert!(!gen.helpers.is_empty());
}

#[test]
fn test_generate_range_inclusive() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Range {
        start: Some(Box::new(Spanned::new(Expr::Int(1), Span::new(0, 1)))),
        end: Some(Box::new(Spanned::new(Expr::Int(5), Span::new(3, 4)))),
        inclusive: true,
    };
    let result = gen.generate_expr(&expr).unwrap();
    assert_eq!(result, "__range(1, 5 + 1)");
}

#[test]
fn test_generate_range_no_start() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Range {
        start: None,
        end: Some(Box::new(Spanned::new(Expr::Int(10), Span::new(0, 2)))),
        inclusive: false,
    };
    let result = gen.generate_expr(&expr).unwrap();
    assert_eq!(result, "__range(0, 10)");
}

#[test]
fn test_generate_range_no_end() {
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::Range {
        start: Some(Box::new(Spanned::new(Expr::Int(5), Span::new(0, 1)))),
        end: None,
        inclusive: false,
    };
    let result = gen.generate_expr(&expr).unwrap();
    assert_eq!(result, "__range(5, Infinity)");
}

#[test]
fn test_pattern_binding_ident() {
    let gen = JsCodeGenerator::new();
    assert_eq!(
        gen.generate_pattern_binding(&Pattern::Ident("x".to_string())),
        "x"
    );
}

#[test]
fn test_pattern_binding_wildcard() {
    let gen = JsCodeGenerator::new();
    assert_eq!(gen.generate_pattern_binding(&Pattern::Wildcard), "_");
}

#[test]
fn test_pattern_binding_tuple() {
    let gen = JsCodeGenerator::new();
    let pat = Pattern::Tuple(vec![
        Spanned::new(Pattern::Ident("a".to_string()), Span::new(0, 1)),
        Spanned::new(Pattern::Ident("b".to_string()), Span::new(3, 4)),
    ]);
    assert_eq!(gen.generate_pattern_binding(&pat), "[a, b]");
}

#[test]
fn test_pattern_condition_wildcard() {
    let mut gen = JsCodeGenerator::new();
    assert_eq!(
        gen.generate_pattern_condition(&Pattern::Wildcard, "x")
            .unwrap(),
        "true"
    );
}

#[test]
fn test_pattern_condition_literal() {
    let mut gen = JsCodeGenerator::new();
    assert_eq!(
        gen.generate_pattern_condition(&Pattern::Literal(Literal::Int(42)), "x")
            .unwrap(),
        "x === 42"
    );
    assert_eq!(
        gen.generate_pattern_condition(
            &Pattern::Literal(Literal::String("hello".to_string())),
            "x"
        )
        .unwrap(),
        "x === \"hello\""
    );
    assert_eq!(
        gen.generate_pattern_condition(&Pattern::Literal(Literal::Bool(true)), "x")
            .unwrap(),
        "x === true"
    );
}

#[test]
fn test_pattern_condition_variant() {
    let mut gen = JsCodeGenerator::new();
    let pat = Pattern::Variant {
        name: Spanned::new("Ok".to_string(), Span::new(0, 2)),
        fields: vec![],
    };
    assert_eq!(
        gen.generate_pattern_condition(&pat, "val").unwrap(),
        "val.__tag === \"Ok\""
    );
}

#[test]
fn test_pattern_bindings_ident() {
    let mut gen = JsCodeGenerator::new();
    assert_eq!(
        gen.generate_pattern_bindings(&Pattern::Ident("x".to_string()), "val")
            .unwrap(),
        "const x = val;"
    );
}

#[test]
fn test_pattern_bindings_wildcard() {
    let mut gen = JsCodeGenerator::new();
    assert_eq!(
        gen.generate_pattern_bindings(&Pattern::Wildcard, "val")
            .unwrap(),
        ""
    );
}

#[test]
fn test_macro_invoke() {
    use vais_ast::macros::{Delimiter, MacroInvoke};
    let mut gen = JsCodeGenerator::new();
    let expr = Expr::MacroInvoke(MacroInvoke {
        name: Spanned::new("println".to_string(), Span::new(0, 7)),
        delimiter: Delimiter::Paren,
        tokens: vec![],
    });
    let result = gen.generate_expr(&expr).unwrap();
    assert!(result.contains("println"));
    assert!(result.contains("macro"));
}

#[test]
fn test_generate_stmts_as_return_empty() {
    let mut gen = JsCodeGenerator::new();
    assert_eq!(gen.generate_stmts_as_return(&[]).unwrap(), "");
}
