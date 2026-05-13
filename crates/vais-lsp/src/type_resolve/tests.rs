use super::*;
use vais_ast::Span;

// ========== LspType::display_name tests ==========

#[test]
fn test_display_name_named() {
    assert_eq!(
        LspType::Named("MyStruct".to_string()).display_name(),
        "MyStruct"
    );
}

#[test]
fn test_display_name_primitive_i64() {
    assert_eq!(LspType::Primitive("i64".to_string()).display_name(), "i64");
}

#[test]
fn test_display_name_primitive_bool() {
    assert_eq!(
        LspType::Primitive("bool".to_string()).display_name(),
        "bool"
    );
}

#[test]
fn test_display_name_primitive_str() {
    assert_eq!(LspType::Primitive("str".to_string()).display_name(), "str");
}

#[test]
fn test_display_name_array() {
    let t = LspType::Array(Box::new(LspType::Primitive("i64".to_string())));
    assert_eq!(t.display_name(), "[i64]");
}

#[test]
fn test_display_name_array_nested() {
    let t = LspType::Array(Box::new(LspType::Array(Box::new(LspType::Primitive(
        "f64".to_string(),
    )))));
    assert_eq!(t.display_name(), "[[f64]]");
}

#[test]
fn test_display_name_tuple_empty() {
    let t = LspType::Tuple(vec![]);
    assert_eq!(t.display_name(), "()");
}

#[test]
fn test_display_name_tuple_single() {
    let t = LspType::Tuple(vec![LspType::Primitive("i64".to_string())]);
    assert_eq!(t.display_name(), "(i64)");
}

#[test]
fn test_display_name_tuple_multi() {
    let t = LspType::Tuple(vec![
        LspType::Primitive("i64".to_string()),
        LspType::Primitive("bool".to_string()),
        LspType::Primitive("str".to_string()),
    ]);
    assert_eq!(t.display_name(), "(i64, bool, str)");
}

#[test]
fn test_display_name_optional() {
    let t = LspType::Optional(Box::new(LspType::Primitive("i64".to_string())));
    assert_eq!(t.display_name(), "Option<i64>");
}

#[test]
fn test_display_name_result() {
    let t = LspType::Result(
        Box::new(LspType::Primitive("i64".to_string())),
        Box::new(LspType::Primitive("str".to_string())),
    );
    assert_eq!(t.display_name(), "Result<i64, str>");
}

#[test]
fn test_display_name_function() {
    let t = LspType::Function {
        params: vec![
            LspType::Primitive("i64".to_string()),
            LspType::Primitive("bool".to_string()),
        ],
        ret: Box::new(LspType::Primitive("str".to_string())),
    };
    assert_eq!(t.display_name(), "fn(i64, bool) -> str");
}

#[test]
fn test_display_name_function_no_params() {
    let t = LspType::Function {
        params: vec![],
        ret: Box::new(LspType::Unit),
    };
    assert_eq!(t.display_name(), "fn() -> ()");
}

#[test]
fn test_display_name_range() {
    assert_eq!(LspType::Range.display_name(), "Range");
}

#[test]
fn test_display_name_unit() {
    assert_eq!(LspType::Unit.display_name(), "()");
}

#[test]
fn test_display_name_unknown() {
    assert_eq!(LspType::Unknown.display_name(), "_");
}

// ========== LspType equality tests ==========

#[test]
fn test_lsp_type_eq_primitives() {
    assert_eq!(
        LspType::Primitive("i64".to_string()),
        LspType::Primitive("i64".to_string())
    );
    assert_ne!(
        LspType::Primitive("i64".to_string()),
        LspType::Primitive("f64".to_string())
    );
}

#[test]
fn test_lsp_type_eq_named() {
    assert_eq!(
        LspType::Named("Foo".to_string()),
        LspType::Named("Foo".to_string())
    );
    assert_ne!(
        LspType::Named("Foo".to_string()),
        LspType::Named("Bar".to_string())
    );
}

#[test]
fn test_lsp_type_eq_unit_vs_unknown() {
    assert_ne!(LspType::Unit, LspType::Unknown);
}

#[test]
fn test_lsp_type_eq_range() {
    assert_eq!(LspType::Range, LspType::Range);
}

// ========== parse_type_string tests ==========

#[test]
fn test_parse_type_string_i64() {
    assert_eq!(
        parse_type_string("i64"),
        LspType::Primitive("i64".to_string())
    );
}

#[test]
fn test_parse_type_string_f64() {
    assert_eq!(
        parse_type_string("f64"),
        LspType::Primitive("f64".to_string())
    );
}

#[test]
fn test_parse_type_string_bool() {
    assert_eq!(
        parse_type_string("bool"),
        LspType::Primitive("bool".to_string())
    );
}

#[test]
fn test_parse_type_string_str() {
    assert_eq!(
        parse_type_string("str"),
        LspType::Primitive("str".to_string())
    );
}

#[test]
fn test_parse_type_string_all_int_types() {
    for ty in [
        "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128",
    ] {
        assert_eq!(parse_type_string(ty), LspType::Primitive(ty.to_string()));
    }
}

#[test]
fn test_parse_type_string_float_types() {
    assert_eq!(
        parse_type_string("f32"),
        LspType::Primitive("f32".to_string())
    );
    assert_eq!(
        parse_type_string("f64"),
        LspType::Primitive("f64".to_string())
    );
}

#[test]
fn test_parse_type_string_size_types() {
    assert_eq!(
        parse_type_string("isize"),
        LspType::Primitive("isize".to_string())
    );
    assert_eq!(
        parse_type_string("usize"),
        LspType::Primitive("usize".to_string())
    );
}

#[test]
fn test_parse_type_string_char() {
    assert_eq!(
        parse_type_string("char"),
        LspType::Primitive("char".to_string())
    );
}

#[test]
fn test_parse_type_string_unit() {
    assert_eq!(parse_type_string("()"), LspType::Unit);
}

#[test]
fn test_parse_type_string_option() {
    assert!(matches!(
        parse_type_string("Option<i64>"),
        LspType::Optional(_)
    ));
}

#[test]
fn test_parse_type_string_result() {
    assert!(matches!(
        parse_type_string("Result<i64, str>"),
        LspType::Result(_, _)
    ));
}

#[test]
fn test_parse_type_string_vec() {
    assert!(matches!(parse_type_string("Vec<i64>"), LspType::Array(_)));
}

#[test]
fn test_parse_type_string_array_bracket() {
    assert!(matches!(parse_type_string("[i64]"), LspType::Array(_)));
}

#[test]
fn test_parse_type_string_fn() {
    assert!(matches!(
        parse_type_string("fn(i64) -> bool"),
        LspType::Function { .. }
    ));
}

#[test]
fn test_parse_type_string_custom_named() {
    assert_eq!(
        parse_type_string("MyStruct"),
        LspType::Named("MyStruct".to_string())
    );
}

#[test]
fn test_parse_type_string_trimming() {
    assert_eq!(
        parse_type_string("  i64  "),
        LspType::Primitive("i64".to_string())
    );
}

// ========== ast_type_to_lsp tests ==========

#[test]
fn test_ast_type_to_lsp_named_primitive() {
    let ty = Type::Named {
        name: "i64".to_string(),
        generics: vec![],
    };
    assert_eq!(ast_type_to_lsp(&ty), LspType::Primitive("i64".to_string()));
}

#[test]
fn test_ast_type_to_lsp_named_all_primitives() {
    for name in [
        "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f32", "f64", "bool",
        "str", "isize", "usize", "char",
    ] {
        let ty = Type::Named {
            name: name.to_string(),
            generics: vec![],
        };
        assert_eq!(ast_type_to_lsp(&ty), LspType::Primitive(name.to_string()));
    }
}

#[test]
fn test_ast_type_to_lsp_named_custom() {
    let ty = Type::Named {
        name: "MyStruct".to_string(),
        generics: vec![],
    };
    assert_eq!(ast_type_to_lsp(&ty), LspType::Named("MyStruct".to_string()));
}

#[test]
fn test_ast_type_to_lsp_option() {
    use vais_ast::Spanned;
    let ty = Type::Named {
        name: "Option".to_string(),
        generics: vec![Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }],
    };
    let result = ast_type_to_lsp(&ty);
    assert!(matches!(result, LspType::Optional(_)));
}

#[test]
fn test_ast_type_to_lsp_result() {
    use vais_ast::Spanned;
    let ty = Type::Named {
        name: "Result".to_string(),
        generics: vec![
            Spanned {
                node: Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                span: Span::new(0, 1),
            },
            Spanned {
                node: Type::Named {
                    name: "str".to_string(),
                    generics: vec![],
                },
                span: Span::new(2, 3),
            },
        ],
    };
    let result = ast_type_to_lsp(&ty);
    assert!(matches!(result, LspType::Result(_, _)));
}

#[test]
fn test_ast_type_to_lsp_vec() {
    use vais_ast::Spanned;
    let ty = Type::Named {
        name: "Vec".to_string(),
        generics: vec![Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }],
    };
    let result = ast_type_to_lsp(&ty);
    assert!(matches!(result, LspType::Array(_)));
}

#[test]
fn test_ast_type_to_lsp_unit() {
    assert_eq!(ast_type_to_lsp(&Type::Unit), LspType::Unit);
}

#[test]
fn test_ast_type_to_lsp_infer() {
    assert_eq!(ast_type_to_lsp(&Type::Infer), LspType::Unknown);
}

#[test]
fn test_ast_type_to_lsp_array() {
    use vais_ast::Spanned;
    let ty = Type::Array(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    let result = ast_type_to_lsp(&ty);
    assert!(matches!(result, LspType::Array(_)));
}

#[test]
fn test_ast_type_to_lsp_tuple() {
    use vais_ast::Spanned;
    let ty = Type::Tuple(vec![
        Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        },
        Spanned {
            node: Type::Named {
                name: "bool".to_string(),
                generics: vec![],
            },
            span: Span::new(2, 3),
        },
    ]);
    let result = ast_type_to_lsp(&ty);
    assert!(matches!(result, LspType::Tuple(_)));
    if let LspType::Tuple(types) = result {
        assert_eq!(types.len(), 2);
    }
}

#[test]
fn test_ast_type_to_lsp_optional() {
    use vais_ast::Spanned;
    let ty = Type::Optional(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert!(matches!(ast_type_to_lsp(&ty), LspType::Optional(_)));
}

#[test]
fn test_ast_type_to_lsp_result_shorthand() {
    use vais_ast::Spanned;
    let ty = Type::Result(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    let result = ast_type_to_lsp(&ty);
    assert!(matches!(result, LspType::Result(_, _)));
}

#[test]
fn test_ast_type_to_lsp_ref() {
    use vais_ast::Spanned;
    let ty = Type::Ref(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert_eq!(ast_type_to_lsp(&ty), LspType::Primitive("i64".to_string()));
}

#[test]
fn test_ast_type_to_lsp_ref_mut() {
    use vais_ast::Spanned;
    let ty = Type::RefMut(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert_eq!(ast_type_to_lsp(&ty), LspType::Primitive("i64".to_string()));
}

#[test]
fn test_ast_type_to_lsp_pointer() {
    use vais_ast::Spanned;
    let ty = Type::Pointer(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert_eq!(ast_type_to_lsp(&ty), LspType::Primitive("i64".to_string()));
}

#[test]
fn test_ast_type_to_lsp_slice() {
    use vais_ast::Spanned;
    let ty = Type::Slice(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert!(matches!(ast_type_to_lsp(&ty), LspType::Array(_)));
}

#[test]
fn test_ast_type_to_lsp_slice_mut() {
    use vais_ast::Spanned;
    let ty = Type::SliceMut(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert!(matches!(ast_type_to_lsp(&ty), LspType::Array(_)));
}

#[test]
fn test_ast_type_to_lsp_option_no_generic() {
    let ty = Type::Named {
        name: "Option".to_string(),
        generics: vec![],
    };
    let result = ast_type_to_lsp(&ty);
    if let LspType::Optional(inner) = result {
        assert_eq!(*inner, LspType::Unknown);
    } else {
        panic!("Expected Optional");
    }
}

// ========== format_type tests ==========

#[test]
fn test_format_type_named_simple() {
    let ty = Type::Named {
        name: "i64".to_string(),
        generics: vec![],
    };
    assert_eq!(format_type(&ty), "i64");
}

#[test]
fn test_format_type_named_with_generics() {
    use vais_ast::Spanned;
    let ty = Type::Named {
        name: "Vec".to_string(),
        generics: vec![Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }],
    };
    assert_eq!(format_type(&ty), "Vec<i64>");
}

#[test]
fn test_format_type_named_multi_generics() {
    use vais_ast::Spanned;
    let ty = Type::Named {
        name: "HashMap".to_string(),
        generics: vec![
            Spanned {
                node: Type::Named {
                    name: "str".to_string(),
                    generics: vec![],
                },
                span: Span::new(0, 1),
            },
            Spanned {
                node: Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                span: Span::new(2, 3),
            },
        ],
    };
    assert_eq!(format_type(&ty), "HashMap<str, i64>");
}

#[test]
fn test_format_type_array() {
    use vais_ast::Spanned;
    let ty = Type::Array(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert_eq!(format_type(&ty), "[i64]");
}

#[test]
fn test_format_type_tuple() {
    use vais_ast::Spanned;
    let ty = Type::Tuple(vec![
        Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        },
        Spanned {
            node: Type::Named {
                name: "bool".to_string(),
                generics: vec![],
            },
            span: Span::new(2, 3),
        },
    ]);
    assert_eq!(format_type(&ty), "(i64, bool)");
}

#[test]
fn test_format_type_unit() {
    assert_eq!(format_type(&Type::Unit), "()");
}

#[test]
fn test_format_type_infer() {
    assert_eq!(format_type(&Type::Infer), "_");
}

#[test]
fn test_format_type_pointer() {
    use vais_ast::Spanned;
    let ty = Type::Pointer(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert_eq!(format_type(&ty), "*i64");
}

#[test]
fn test_format_type_ref() {
    use vais_ast::Spanned;
    let ty = Type::Ref(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert_eq!(format_type(&ty), "&i64");
}

#[test]
fn test_format_type_ref_mut() {
    use vais_ast::Spanned;
    let ty = Type::RefMut(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert_eq!(format_type(&ty), "&mut i64");
}

#[test]
fn test_format_type_slice() {
    use vais_ast::Spanned;
    let ty = Type::Slice(Box::new(Spanned {
        node: Type::Named {
            name: "u8".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert_eq!(format_type(&ty), "&[u8]");
}

#[test]
fn test_format_type_slice_mut() {
    use vais_ast::Spanned;
    let ty = Type::SliceMut(Box::new(Spanned {
        node: Type::Named {
            name: "u8".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert_eq!(format_type(&ty), "&mut [u8]");
}

#[test]
fn test_format_type_optional() {
    use vais_ast::Spanned;
    let ty = Type::Optional(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert_eq!(format_type(&ty), "i64?");
}

#[test]
fn test_format_type_result_shorthand() {
    use vais_ast::Spanned;
    let ty = Type::Result(Box::new(Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }));
    assert_eq!(format_type(&ty), "i64!");
}

// ========== TypeContext::infer_expr_type tests ==========

fn make_ctx() -> TypeContext {
    TypeContext {
        structs: HashMap::new(),
        type_methods: HashMap::new(),
        trait_methods: HashMap::new(),
        type_traits: HashMap::new(),
        enum_variants: HashMap::new(),
        function_returns: HashMap::new(),
        variable_types: HashMap::new(),
    }
}

fn spanned<T>(node: T) -> Spanned<T> {
    Spanned {
        node,
        span: Span::new(0, 1),
    }
}

#[test]
fn test_infer_int() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Int(42));
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Primitive("i64".to_string())
    );
}

#[test]
fn test_infer_float() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Float(3.14));
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Primitive("f64".to_string())
    );
}

#[test]
fn test_infer_bool() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Bool(true));
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Primitive("bool".to_string())
    );
}

#[test]
fn test_infer_string() {
    let ctx = make_ctx();
    let expr = spanned(Expr::String("hello".to_string()));
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Primitive("str".to_string())
    );
}

#[test]
fn test_infer_ident_known() {
    let mut ctx = make_ctx();
    ctx.variable_types
        .insert("x".to_string(), LspType::Primitive("i64".to_string()));
    let expr = spanned(Expr::Ident("x".to_string()));
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Primitive("i64".to_string())
    );
}

#[test]
fn test_infer_ident_unknown() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Ident("x".to_string()));
    assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
}

#[test]
fn test_infer_array_empty() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Array(vec![]));
    let result = ctx.infer_expr_type(&expr);
    assert!(matches!(result, LspType::Array(_)));
}

#[test]
fn test_infer_array_with_elements() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Array(vec![
        spanned(Expr::Int(1)),
        spanned(Expr::Int(2)),
    ]));
    let result = ctx.infer_expr_type(&expr);
    if let LspType::Array(inner) = result {
        assert_eq!(*inner, LspType::Primitive("i64".to_string()));
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_infer_tuple() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Tuple(vec![
        spanned(Expr::Int(1)),
        spanned(Expr::Bool(true)),
    ]));
    let result = ctx.infer_expr_type(&expr);
    if let LspType::Tuple(types) = result {
        assert_eq!(types.len(), 2);
        assert_eq!(types[0], LspType::Primitive("i64".to_string()));
        assert_eq!(types[1], LspType::Primitive("bool".to_string()));
    } else {
        panic!("Expected Tuple");
    }
}

#[test]
fn test_infer_call_some() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Call {
        func: Box::new(spanned(Expr::Ident("Some".to_string()))),
        args: vec![spanned(Expr::Int(42))],
    });
    assert!(matches!(ctx.infer_expr_type(&expr), LspType::Optional(_)));
}

#[test]
fn test_infer_call_ok() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Call {
        func: Box::new(spanned(Expr::Ident("Ok".to_string()))),
        args: vec![spanned(Expr::Int(42))],
    });
    assert!(matches!(ctx.infer_expr_type(&expr), LspType::Result(_, _)));
}

#[test]
fn test_infer_call_err() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Call {
        func: Box::new(spanned(Expr::Ident("Err".to_string()))),
        args: vec![spanned(Expr::String("error".to_string()))],
    });
    assert!(matches!(ctx.infer_expr_type(&expr), LspType::Result(_, _)));
}

#[test]
fn test_infer_call_none() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Call {
        func: Box::new(spanned(Expr::Ident("None".to_string()))),
        args: vec![],
    });
    assert!(matches!(ctx.infer_expr_type(&expr), LspType::Optional(_)));
}

#[test]
fn test_infer_call_known_function() {
    let mut ctx = make_ctx();
    ctx.function_returns
        .insert("foo".to_string(), LspType::Primitive("bool".to_string()));
    let expr = spanned(Expr::Call {
        func: Box::new(spanned(Expr::Ident("foo".to_string()))),
        args: vec![],
    });
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Primitive("bool".to_string())
    );
}

#[test]
fn test_infer_call_struct_constructor() {
    let mut ctx = make_ctx();
    ctx.structs.insert("Point".to_string(), vec![]);
    let expr = spanned(Expr::Call {
        func: Box::new(spanned(Expr::Ident("Point".to_string()))),
        args: vec![],
    });
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Named("Point".to_string())
    );
}

#[test]
fn test_infer_call_unknown_function() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Call {
        func: Box::new(spanned(Expr::Ident("unknown_fn".to_string()))),
        args: vec![],
    });
    assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
}

#[test]
fn test_infer_struct_lit() {
    let ctx = make_ctx();
    let expr = spanned(Expr::StructLit {
        name: spanned("Point".to_string()),
        fields: vec![],
        enum_name: None,
    });
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Named("Point".to_string())
    );
}

#[test]
fn test_infer_field_access() {
    let mut ctx = make_ctx();
    ctx.structs.insert(
        "Point".to_string(),
        vec![
            FieldInfo {
                name: "x".to_string(),
                ty: LspType::Primitive("i64".to_string()),
                type_display: "i64".to_string(),
            },
            FieldInfo {
                name: "y".to_string(),
                ty: LspType::Primitive("f64".to_string()),
                type_display: "f64".to_string(),
            },
        ],
    );
    ctx.variable_types
        .insert("p".to_string(), LspType::Named("Point".to_string()));
    let expr = spanned(Expr::Field {
        expr: Box::new(spanned(Expr::Ident("p".to_string()))),
        field: spanned("x".to_string()),
    });
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Primitive("i64".to_string())
    );
}

#[test]
fn test_infer_field_access_unknown_field() {
    let mut ctx = make_ctx();
    ctx.structs.insert(
        "Point".to_string(),
        vec![FieldInfo {
            name: "x".to_string(),
            ty: LspType::Primitive("i64".to_string()),
            type_display: "i64".to_string(),
        }],
    );
    ctx.variable_types
        .insert("p".to_string(), LspType::Named("Point".to_string()));
    let expr = spanned(Expr::Field {
        expr: Box::new(spanned(Expr::Ident("p".to_string()))),
        field: spanned("z".to_string()),
    });
    assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
}

#[test]
fn test_infer_method_call() {
    let mut ctx = make_ctx();
    ctx.type_methods.insert(
        "Vec".to_string(),
        vec![MethodInfo {
            name: "len".to_string(),
            params: vec![],
            ret_type: Some("i64".to_string()),
            from_trait: None,
        }],
    );
    ctx.variable_types
        .insert("v".to_string(), LspType::Named("Vec".to_string()));
    let expr = spanned(Expr::MethodCall {
        receiver: Box::new(spanned(Expr::Ident("v".to_string()))),
        method: spanned("len".to_string()),
        args: vec![],
    });
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Primitive("i64".to_string())
    );
}

#[test]
fn test_infer_range() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Range {
        start: Some(Box::new(spanned(Expr::Int(0)))),
        end: Some(Box::new(spanned(Expr::Int(10)))),
        inclusive: false,
    });
    assert_eq!(ctx.infer_expr_type(&expr), LspType::Range);
}

#[test]
fn test_infer_block_empty() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Block(vec![]));
    assert_eq!(ctx.infer_expr_type(&expr), LspType::Unit);
}

#[test]
fn test_infer_block_with_expr() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Block(vec![spanned(Stmt::Expr(Box::new(spanned(
        Expr::Int(42),
    ))))]));
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Primitive("i64".to_string())
    );
}

#[test]
fn test_infer_block_with_return() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Block(vec![spanned(Stmt::Return(Some(Box::new(
        spanned(Expr::Bool(true)),
    ))))]));
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Primitive("bool".to_string())
    );
}

// ========== TypeContext::from_module tests ==========

fn parse_module(source: &str) -> Module {
    vais_parser::parse(source).expect("parse failed")
}

#[test]
fn test_from_module_function() {
    let ast = parse_module("F foo() -> i64 { 42 }");
    let ctx = TypeContext::from_module(&ast);
    assert!(ctx.function_returns.contains_key("foo"));
}

#[test]
fn test_from_module_struct() {
    let ast = parse_module("S Point { x: i64, y: i64 }");
    let ctx = TypeContext::from_module(&ast);
    assert!(ctx.structs.contains_key("Point"));
    assert_eq!(ctx.structs["Point"].len(), 2);
    assert_eq!(ctx.structs["Point"][0].name, "x");
    assert_eq!(ctx.structs["Point"][1].name, "y");
}

#[test]
fn test_from_module_enum() {
    let ast = parse_module("E Color { Red, Green, Blue }");
    let ctx = TypeContext::from_module(&ast);
    assert!(ctx.enum_variants.contains_key("Color"));
    assert_eq!(ctx.enum_variants["Color"], vec!["Red", "Green", "Blue"]);
}

#[test]
fn test_from_module_impl() {
    let ast = parse_module("S Point { x: i64 }\nX Point { F get_x(self) -> i64 { self.x } }");
    let ctx = TypeContext::from_module(&ast);
    assert!(ctx.type_methods.contains_key("Point"));
    assert_eq!(ctx.type_methods["Point"].len(), 1);
    assert_eq!(ctx.type_methods["Point"][0].name, "get_x");
}

#[test]
fn test_from_module_trait() {
    let ast = parse_module("W Printable { F print(self) -> i64 }");
    let ctx = TypeContext::from_module(&ast);
    assert!(ctx.trait_methods.contains_key("Printable"));
}

#[test]
fn test_from_module_trait_impl() {
    let ast = parse_module("W Printable { F print(self) -> i64 }\nS Foo { x: i64 }\nX Foo: Printable { F print(self) -> i64 { self.x } }");
    let ctx = TypeContext::from_module(&ast);
    assert!(ctx.type_traits.contains_key("Foo"));
    assert!(ctx.type_traits["Foo"].contains(&"Printable".to_string()));
}

#[test]
fn test_from_module_function_no_return_type() {
    let ast = parse_module("F noop() { }");
    let ctx = TypeContext::from_module(&ast);
    if let Some(ret) = ctx.function_returns.get("noop") {
        assert_eq!(*ret, LspType::Unit);
    }
}

// ========== TypeContext::get_dot_completions tests ==========

#[test]
fn test_dot_completions_struct_fields() {
    let mut ctx = make_ctx();
    ctx.structs.insert(
        "Point".to_string(),
        vec![
            FieldInfo {
                name: "x".to_string(),
                ty: LspType::Primitive("i64".to_string()),
                type_display: "i64".to_string(),
            },
            FieldInfo {
                name: "y".to_string(),
                ty: LspType::Primitive("i64".to_string()),
                type_display: "i64".to_string(),
            },
        ],
    );
    let completions = ctx.get_dot_completions("Point");
    assert_eq!(completions.len(), 2);
    assert!(completions.iter().any(|c| c.label == "x"));
    assert!(completions.iter().any(|c| c.label == "y"));
}

#[test]
fn test_dot_completions_methods() {
    let mut ctx = make_ctx();
    ctx.type_methods.insert(
        "Vec".to_string(),
        vec![
            MethodInfo {
                name: "len".to_string(),
                params: vec![],
                ret_type: Some("i64".to_string()),
                from_trait: None,
            },
            MethodInfo {
                name: "push".to_string(),
                params: vec![("item".to_string(), "T".to_string())],
                ret_type: None,
                from_trait: None,
            },
        ],
    );
    let completions = ctx.get_dot_completions("Vec");
    assert_eq!(completions.len(), 2);
    assert!(completions.iter().any(|c| c.label == "len"));
    assert!(completions.iter().any(|c| c.label == "push"));
}

#[test]
fn test_dot_completions_trait_methods() {
    let mut ctx = make_ctx();
    ctx.type_traits
        .insert("Foo".to_string(), vec!["ToString".to_string()]);
    ctx.trait_methods.insert(
        "ToString".to_string(),
        vec![MethodInfo {
            name: "to_string".to_string(),
            params: vec![],
            ret_type: Some("str".to_string()),
            from_trait: Some("ToString".to_string()),
        }],
    );
    let completions = ctx.get_dot_completions("Foo");
    assert_eq!(completions.len(), 1);
    assert_eq!(completions[0].label, "to_string");
    assert_eq!(completions[0].from_trait, Some("ToString".to_string()));
}

#[test]
fn test_dot_completions_no_duplicates() {
    let mut ctx = make_ctx();
    ctx.type_methods.insert(
        "Foo".to_string(),
        vec![MethodInfo {
            name: "display".to_string(),
            params: vec![],
            ret_type: Some("str".to_string()),
            from_trait: None,
        }],
    );
    ctx.type_traits
        .insert("Foo".to_string(), vec!["Display".to_string()]);
    ctx.trait_methods.insert(
        "Display".to_string(),
        vec![MethodInfo {
            name: "display".to_string(),
            params: vec![],
            ret_type: Some("str".to_string()),
            from_trait: Some("Display".to_string()),
        }],
    );
    let completions = ctx.get_dot_completions("Foo");
    // Should only have 1 "display" entry (direct impl takes priority)
    assert_eq!(
        completions.iter().filter(|c| c.label == "display").count(),
        1
    );
}

#[test]
fn test_dot_completions_empty_type() {
    let ctx = make_ctx();
    let completions = ctx.get_dot_completions("NonExistent");
    assert!(completions.is_empty());
}

#[test]
fn test_dot_completions_fields_and_methods() {
    let mut ctx = make_ctx();
    ctx.structs.insert(
        "MyStruct".to_string(),
        vec![FieldInfo {
            name: "value".to_string(),
            ty: LspType::Primitive("i64".to_string()),
            type_display: "i64".to_string(),
        }],
    );
    ctx.type_methods.insert(
        "MyStruct".to_string(),
        vec![MethodInfo {
            name: "get_value".to_string(),
            params: vec![],
            ret_type: Some("i64".to_string()),
            from_trait: None,
        }],
    );
    let completions = ctx.get_dot_completions("MyStruct");
    assert_eq!(completions.len(), 2);
    let field = completions.iter().find(|c| c.label == "value").unwrap();
    assert!(matches!(field.kind, CompletionKind::Field));
    let method = completions.iter().find(|c| c.label == "get_value").unwrap();
    assert!(matches!(method.kind, CompletionKind::Method));
}

// ========== Additional LspType display_name edge cases ==========

#[test]
fn test_display_name_optional_nested() {
    let t = LspType::Optional(Box::new(LspType::Optional(Box::new(LspType::Primitive(
        "i64".to_string(),
    )))));
    assert_eq!(t.display_name(), "Option<Option<i64>>");
}

#[test]
fn test_display_name_result_nested() {
    let t = LspType::Result(
        Box::new(LspType::Array(Box::new(LspType::Primitive(
            "i64".to_string(),
        )))),
        Box::new(LspType::Primitive("str".to_string())),
    );
    assert_eq!(t.display_name(), "Result<[i64], str>");
}

#[test]
fn test_display_name_function_complex() {
    let t = LspType::Function {
        params: vec![
            LspType::Array(Box::new(LspType::Primitive("i64".to_string()))),
            LspType::Optional(Box::new(LspType::Primitive("str".to_string()))),
        ],
        ret: Box::new(LspType::Result(
            Box::new(LspType::Primitive("bool".to_string())),
            Box::new(LspType::Primitive("str".to_string())),
        )),
    };
    assert_eq!(
        t.display_name(),
        "fn([i64], Option<str>) -> Result<bool, str>"
    );
}

#[test]
fn test_display_name_tuple_with_array() {
    let t = LspType::Tuple(vec![
        LspType::Array(Box::new(LspType::Primitive("i64".to_string()))),
        LspType::Primitive("bool".to_string()),
    ]);
    assert_eq!(t.display_name(), "([i64], bool)");
}

#[test]
fn test_display_name_array_of_tuples() {
    let t = LspType::Array(Box::new(LspType::Tuple(vec![
        LspType::Primitive("i64".to_string()),
        LspType::Primitive("str".to_string()),
    ])));
    assert_eq!(t.display_name(), "[(i64, str)]");
}

// ========== Additional LspType equality edge cases ==========

#[test]
fn test_lsp_type_clone() {
    let t = LspType::Array(Box::new(LspType::Primitive("i64".to_string())));
    let cloned = t.clone();
    assert_eq!(t, cloned);
}

#[test]
fn test_lsp_type_debug() {
    let t = LspType::Primitive("i64".to_string());
    let debug = format!("{:?}", t);
    assert!(debug.contains("Primitive"));
    assert!(debug.contains("i64"));
}

#[test]
fn test_lsp_type_ne_different_variants() {
    assert_ne!(
        LspType::Primitive("i64".to_string()),
        LspType::Named("i64".to_string())
    );
    assert_ne!(LspType::Unit, LspType::Range);
    assert_ne!(LspType::Array(Box::new(LspType::Unit)), LspType::Unit);
}

// ========== Additional parse_type_string edge cases ==========

#[test]
fn test_parse_type_string_tuple() {
    // parse_type_string doesn't parse tuple syntax, falls through to Named
    let result = parse_type_string("(i64, bool)");
    assert!(matches!(result, LspType::Named(_)));
}

#[test]
fn test_parse_type_string_hashmap() {
    // HashMap is not a known generic, should be Named
    let result = parse_type_string("HashMap<str, i64>");
    assert!(matches!(result, LspType::Named(_)));
}

#[test]
fn test_parse_type_string_empty_string() {
    // Empty string falls through to Named("")
    let result = parse_type_string("");
    assert_eq!(result, LspType::Named("".to_string()));
}

#[test]
fn test_parse_type_string_whitespace_only() {
    // Trimmed whitespace becomes empty string, falls through to Named
    let result = parse_type_string("   ");
    assert_eq!(result, LspType::Named("".to_string()));
}

// ========== Additional ast_type_to_lsp edge cases ==========

#[test]
fn test_ast_type_to_lsp_nested_generics() {
    use vais_ast::Spanned;
    let ty = Type::Named {
        name: "Vec".to_string(),
        generics: vec![Spanned {
            node: Type::Named {
                name: "Option".to_string(),
                generics: vec![Spanned {
                    node: Type::Named {
                        name: "i64".to_string(),
                        generics: vec![],
                    },
                    span: Span::new(0, 1),
                }],
            },
            span: Span::new(0, 1),
        }],
    };
    let result = ast_type_to_lsp(&ty);
    if let LspType::Array(inner) = result {
        assert!(matches!(*inner, LspType::Optional(_)));
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_ast_type_to_lsp_result_with_generics() {
    use vais_ast::Spanned;
    let ty = Type::Named {
        name: "Result".to_string(),
        generics: vec![
            Spanned {
                node: Type::Named {
                    name: "Vec".to_string(),
                    generics: vec![Spanned {
                        node: Type::Named {
                            name: "i64".to_string(),
                            generics: vec![],
                        },
                        span: Span::new(0, 1),
                    }],
                },
                span: Span::new(0, 1),
            },
            Spanned {
                node: Type::Named {
                    name: "str".to_string(),
                    generics: vec![],
                },
                span: Span::new(2, 3),
            },
        ],
    };
    let result = ast_type_to_lsp(&ty);
    if let LspType::Result(ok, _err) = result {
        assert!(matches!(*ok, LspType::Array(_)));
    } else {
        panic!("Expected Result");
    }
}

// ========== Additional format_type edge cases ==========

#[test]
fn test_format_type_nested_generics() {
    use vais_ast::Spanned;
    let ty = Type::Named {
        name: "Vec".to_string(),
        generics: vec![Spanned {
            node: Type::Named {
                name: "Vec".to_string(),
                generics: vec![Spanned {
                    node: Type::Named {
                        name: "i64".to_string(),
                        generics: vec![],
                    },
                    span: Span::new(0, 1),
                }],
            },
            span: Span::new(0, 1),
        }],
    };
    assert_eq!(format_type(&ty), "Vec<Vec<i64>>");
}

#[test]
fn test_format_type_empty_tuple() {
    let ty = Type::Tuple(vec![]);
    assert_eq!(format_type(&ty), "()");
}

#[test]
fn test_format_type_single_element_tuple() {
    use vais_ast::Spanned;
    let ty = Type::Tuple(vec![Spanned {
        node: Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        },
        span: Span::new(0, 1),
    }]);
    assert_eq!(format_type(&ty), "(i64)");
}

// ========== Additional TypeContext::infer_expr_type edge cases ==========

#[test]
fn test_infer_nested_call() {
    let mut ctx = make_ctx();
    ctx.function_returns
        .insert("inner".to_string(), LspType::Primitive("i64".to_string()));
    let expr = spanned(Expr::Call {
        func: Box::new(spanned(Expr::Ident("outer".to_string()))),
        args: vec![spanned(Expr::Call {
            func: Box::new(spanned(Expr::Ident("inner".to_string()))),
            args: vec![],
        })],
    });
    // outer is unknown, so result is Unknown
    assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
}

#[test]
fn test_infer_struct_lit_with_fields() {
    let ctx = make_ctx();
    let expr = spanned(Expr::StructLit {
        name: spanned("Config".to_string()),
        fields: vec![
            (spanned("debug".to_string()), spanned(Expr::Bool(true))),
            (spanned("level".to_string()), spanned(Expr::Int(3))),
        ],
        enum_name: None,
    });
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Named("Config".to_string())
    );
}

#[test]
fn test_infer_method_call_trait_not_resolved() {
    // infer_expr_type only checks type_methods, not trait_methods
    // so trait-only methods return Unknown
    let mut ctx = make_ctx();
    ctx.type_traits
        .insert("MyType".to_string(), vec!["Display".to_string()]);
    ctx.trait_methods.insert(
        "Display".to_string(),
        vec![MethodInfo {
            name: "to_string".to_string(),
            params: vec![],
            ret_type: Some("str".to_string()),
            from_trait: Some("Display".to_string()),
        }],
    );
    ctx.variable_types
        .insert("x".to_string(), LspType::Named("MyType".to_string()));
    let expr = spanned(Expr::MethodCall {
        receiver: Box::new(spanned(Expr::Ident("x".to_string()))),
        method: spanned("to_string".to_string()),
        args: vec![],
    });
    // Returns Unknown because infer_expr_type doesn't look up trait_methods
    assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
}

#[test]
fn test_infer_method_call_unknown_receiver() {
    let ctx = make_ctx();
    let expr = spanned(Expr::MethodCall {
        receiver: Box::new(spanned(Expr::Ident("unknown".to_string()))),
        method: spanned("foo".to_string()),
        args: vec![],
    });
    assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
}

#[test]
fn test_infer_range_partial() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Range {
        start: Some(Box::new(spanned(Expr::Int(0)))),
        end: None,
        inclusive: false,
    });
    assert_eq!(ctx.infer_expr_type(&expr), LspType::Range);
}

#[test]
fn test_infer_block_with_let() {
    let ctx = make_ctx();
    let expr = spanned(Expr::Block(vec![
        spanned(Stmt::Let {
            name: spanned("x".to_string()),
            ty: None,
            value: Box::new(spanned(Expr::Int(42))),
            is_mut: false,
            ownership: vais_ast::Ownership::Regular,
        }),
        spanned(Stmt::Expr(Box::new(spanned(Expr::Ident("x".to_string()))))),
    ]));
    // Block returns the type of the last stmt
    let result = ctx.infer_expr_type(&expr);
    // Since block processing uses last stmt, ident "x" won't be in scope at inference time
    // (infer_expr_type doesn't add let bindings to its scope) -- this is expected Unknown
    assert_eq!(result, LspType::Unknown);
}

// ========== Additional TypeContext::from_module edge cases ==========

#[test]
fn test_from_module_multiple_functions() {
    let ast = parse_module("F foo() -> i64 { 0 }\nF bar() -> bool { true }");
    let ctx = TypeContext::from_module(&ast);
    assert!(ctx.function_returns.contains_key("foo"));
    assert!(ctx.function_returns.contains_key("bar"));
}

#[test]
fn test_from_module_struct_with_typed_fields() {
    let ast = parse_module("S Config { debug: bool, level: i64, name: str }");
    let ctx = TypeContext::from_module(&ast);
    assert_eq!(ctx.structs["Config"].len(), 3);
    assert_eq!(ctx.structs["Config"][0].name, "debug");
    assert_eq!(ctx.structs["Config"][1].name, "level");
    assert_eq!(ctx.structs["Config"][2].name, "name");
}

#[test]
fn test_from_module_empty() {
    let ast = parse_module("");
    let ctx = TypeContext::from_module(&ast);
    assert!(ctx.structs.is_empty());
    assert!(ctx.function_returns.is_empty());
    assert!(ctx.enum_variants.is_empty());
}

#[test]
fn test_from_module_multiple_enums() {
    let ast = parse_module("E Color { Red, Blue }\nE Size { Small, Big }");
    let ctx = TypeContext::from_module(&ast);
    assert_eq!(ctx.enum_variants.len(), 2);
    assert_eq!(ctx.enum_variants["Color"].len(), 2);
    assert_eq!(ctx.enum_variants["Size"].len(), 2);
}

#[test]
fn test_from_module_impl_multiple_methods() {
    let ast = parse_module(
        "S Vec { len: i64 }\nX Vec { F push(self, x: i64) -> i64 { 0 }\nF pop(self) -> i64 { 0 } }",
    );
    let ctx = TypeContext::from_module(&ast);
    assert_eq!(ctx.type_methods["Vec"].len(), 2);
}

// ========== Additional TypeContext::get_dot_completions edge cases ==========

#[test]
fn test_dot_completions_method_detail() {
    let mut ctx = make_ctx();
    ctx.type_methods.insert(
        "Vec".to_string(),
        vec![MethodInfo {
            name: "len".to_string(),
            params: vec![],
            ret_type: Some("i64".to_string()),
            from_trait: None,
        }],
    );
    let completions = ctx.get_dot_completions("Vec");
    assert_eq!(completions[0].label, "len");
    assert!(completions[0].detail.contains("i64"));
}

#[test]
fn test_dot_completions_field_detail() {
    let mut ctx = make_ctx();
    ctx.structs.insert(
        "Point".to_string(),
        vec![FieldInfo {
            name: "x".to_string(),
            ty: LspType::Primitive("f64".to_string()),
            type_display: "f64".to_string(),
        }],
    );
    let completions = ctx.get_dot_completions("Point");
    assert_eq!(completions[0].label, "x");
    assert!(completions[0].detail.contains("f64"));
}

#[test]
fn test_dot_completions_multiple_traits() {
    let mut ctx = make_ctx();
    ctx.type_traits.insert(
        "MyType".to_string(),
        vec!["Display".to_string(), "Debug".to_string()],
    );
    ctx.trait_methods.insert(
        "Display".to_string(),
        vec![MethodInfo {
            name: "display".to_string(),
            params: vec![],
            ret_type: Some("str".to_string()),
            from_trait: Some("Display".to_string()),
        }],
    );
    ctx.trait_methods.insert(
        "Debug".to_string(),
        vec![MethodInfo {
            name: "debug".to_string(),
            params: vec![],
            ret_type: Some("str".to_string()),
            from_trait: Some("Debug".to_string()),
        }],
    );
    let completions = ctx.get_dot_completions("MyType");
    assert_eq!(completions.len(), 2);
    assert!(completions.iter().any(|c| c.label == "display"));
    assert!(completions.iter().any(|c| c.label == "debug"));
}

// ========== FieldInfo and MethodInfo tests ==========

#[test]
fn test_field_info_clone() {
    let fi = FieldInfo {
        name: "x".to_string(),
        ty: LspType::Primitive("i64".to_string()),
        type_display: "i64".to_string(),
    };
    let cloned = fi.clone();
    assert_eq!(fi.name, cloned.name);
    assert_eq!(fi.type_display, cloned.type_display);
}

#[test]
fn test_method_info_clone() {
    let mi = MethodInfo {
        name: "foo".to_string(),
        params: vec![("x".to_string(), "i64".to_string())],
        ret_type: Some("bool".to_string()),
        from_trait: Some("MyTrait".to_string()),
    };
    let cloned = mi.clone();
    assert_eq!(mi.name, cloned.name);
    assert_eq!(mi.params, cloned.params);
    assert_eq!(mi.ret_type, cloned.ret_type);
    assert_eq!(mi.from_trait, cloned.from_trait);
}

#[test]
fn test_method_info_no_return() {
    let mi = MethodInfo {
        name: "set".to_string(),
        params: vec![("val".to_string(), "i64".to_string())],
        ret_type: None,
        from_trait: None,
    };
    assert!(mi.ret_type.is_none());
    assert!(mi.from_trait.is_none());
}

#[test]
fn test_completion_entry_field_kind() {
    let entry = CompletionEntry {
        label: "x".to_string(),
        kind: CompletionKind::Field,
        detail: "i64".to_string(),
        insert_text: "x".to_string(),
        from_trait: None,
    };
    assert!(matches!(entry.kind, CompletionKind::Field));
    assert!(entry.from_trait.is_none());
}

#[test]
fn test_completion_entry_method_kind() {
    let entry = CompletionEntry {
        label: "len".to_string(),
        kind: CompletionKind::Method,
        detail: "() -> i64".to_string(),
        insert_text: "len()".to_string(),
        from_trait: Some("Sized".to_string()),
    };
    assert!(matches!(entry.kind, CompletionKind::Method));
    assert_eq!(entry.from_trait, Some("Sized".to_string()));
}

#[test]
fn test_completion_entry_clone() {
    let entry = CompletionEntry {
        label: "test".to_string(),
        kind: CompletionKind::Field,
        detail: "detail".to_string(),
        insert_text: "test".to_string(),
        from_trait: None,
    };
    let cloned = entry.clone();
    assert_eq!(entry.label, cloned.label);
    assert_eq!(entry.detail, cloned.detail);
}

// ========== infer_expr_type — If expression ==========

#[test]
fn test_infer_if_then_branch() {
    let ctx = TypeContext {
        structs: HashMap::new(),
        type_methods: HashMap::new(),
        trait_methods: HashMap::new(),
        type_traits: HashMap::new(),
        enum_variants: HashMap::new(),
        function_returns: HashMap::new(),
        variable_types: HashMap::new(),
    };
    // If with then branch returning an int
    let expr = spanned(Expr::If {
        cond: Box::new(spanned(Expr::Bool(true))),
        then: vec![spanned(Stmt::Expr(Box::new(spanned(Expr::Int(42)))))],
        else_: None,
    });
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Primitive("i64".to_string())
    );
}

#[test]
fn test_infer_if_else_branch() {
    let ctx = TypeContext {
        structs: HashMap::new(),
        type_methods: HashMap::new(),
        trait_methods: HashMap::new(),
        type_traits: HashMap::new(),
        enum_variants: HashMap::new(),
        function_returns: HashMap::new(),
        variable_types: HashMap::new(),
    };
    // If with empty then, but else branch returning a string
    let expr = spanned(Expr::If {
        cond: Box::new(spanned(Expr::Bool(true))),
        then: vec![],
        else_: Some(IfElse::Else(vec![spanned(Stmt::Expr(Box::new(spanned(
            Expr::String("hello".to_string()),
        ))))])),
    });
    assert_eq!(
        ctx.infer_expr_type(&expr),
        LspType::Primitive("str".to_string())
    );
}

#[test]
fn test_infer_if_empty_both() {
    let ctx = TypeContext {
        structs: HashMap::new(),
        type_methods: HashMap::new(),
        trait_methods: HashMap::new(),
        type_traits: HashMap::new(),
        enum_variants: HashMap::new(),
        function_returns: HashMap::new(),
        variable_types: HashMap::new(),
    };
    let expr = spanned(Expr::If {
        cond: Box::new(spanned(Expr::Bool(true))),
        then: vec![],
        else_: None,
    });
    assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
}

// ========== parse_type_string — additional patterns ==========

#[test]
fn test_parse_type_string_option_generic() {
    let t = parse_type_string("Option<i64>");
    assert!(matches!(t, LspType::Optional(_)));
}

#[test]
fn test_parse_type_string_result_generic() {
    let t = parse_type_string("Result<i64, str>");
    assert!(matches!(t, LspType::Result(_, _)));
}

#[test]
fn test_parse_type_string_array_bracket_notation() {
    let t = parse_type_string("[i64]");
    assert!(matches!(t, LspType::Array(_)));
}

#[test]
fn test_parse_type_string_vec_generic() {
    let t = parse_type_string("Vec<bool>");
    assert!(matches!(t, LspType::Array(_)));
}

// ========== format_type — additional edge cases ==========

#[test]
fn test_format_type_optional_shorthand() {
    // Type::Optional produces "type?" format
    let ty = Type::Optional(Box::new(spanned(Type::Named {
        name: "i64".to_string(),
        generics: vec![],
    })));
    assert_eq!(format_type(&ty), "i64?");
}

#[test]
fn test_format_type_result_shorthand_explicit() {
    // Type::Result produces "type!" format
    let ty = Type::Result(Box::new(spanned(Type::Named {
        name: "str".to_string(),
        generics: vec![],
    })));
    assert_eq!(format_type(&ty), "str!");
}
