use super::*;

fn make_function(name: &str, is_pub: bool, body: Expr) -> Spanned<Item> {
    Spanned::new(
        Item::Function(Function {
            name: Spanned::new(name.to_string(), Span::new(0, name.len())),
            generics: vec![],
            params: vec![],
            ret_type: None,
            body: FunctionBody::Expr(Box::new(Spanned::new(body, Span::new(0, 1)))),
            is_pub,
            is_async: false,
            attributes: vec![],
            where_clause: vec![],
        }),
        Span::new(0, 1),
    )
}

fn make_call(func_name: &str) -> Expr {
    Expr::Call {
        func: Box::new(Spanned::new(
            Expr::Ident(func_name.to_string()),
            Span::new(0, func_name.len()),
        )),
        args: vec![],
    }
}

#[test]
fn test_unreferenced_private_function_removed() {
    let module = Module {
        items: vec![
            make_function("main", false, Expr::Int(42)),
            make_function("unused", false, Expr::Int(0)),
        ],
        modules_map: None,
    };

    let shaken = TreeShaker::shake(&module);

    assert_eq!(shaken.items.len(), 1);
    match &shaken.items[0].node {
        Item::Function(f) => assert_eq!(f.name.node, "main"),
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_transitively_referenced_functions_kept() {
    let module = Module {
        items: vec![
            make_function("main", false, make_call("helper1")),
            make_function("helper1", false, make_call("helper2")),
            make_function("helper2", false, Expr::Int(42)),
            make_function("unused", false, Expr::Int(0)),
        ],
        modules_map: None,
    };

    let shaken = TreeShaker::shake(&module);

    assert_eq!(shaken.items.len(), 3);
    let names: Vec<String> = shaken
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Function(f) => Some(f.name.node.clone()),
            _ => None,
        })
        .collect();

    assert!(names.contains(&"main".to_string()));
    assert!(names.contains(&"helper1".to_string()));
    assert!(names.contains(&"helper2".to_string()));
    assert!(!names.contains(&"unused".to_string()));
}

#[test]
fn test_public_functions_always_kept() {
    let module = Module {
        items: vec![
            make_function("main", false, Expr::Int(42)),
            make_function("public_api", true, Expr::Int(100)),
            make_function("unused_private", false, Expr::Int(0)),
        ],
        modules_map: None,
    };

    let shaken = TreeShaker::shake(&module);

    assert_eq!(shaken.items.len(), 2);
    let names: Vec<String> = shaken
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Function(f) => Some(f.name.node.clone()),
            _ => None,
        })
        .collect();

    assert!(names.contains(&"main".to_string()));
    assert!(names.contains(&"public_api".to_string()));
    assert!(!names.contains(&"unused_private".to_string()));
}

#[test]
fn test_struct_lit_marks_struct_reachable() {
    let module = Module {
        items: vec![
            make_function(
                "main",
                false,
                Expr::StructLit {
                    name: Spanned::new("Point".to_string(), Span::new(0, 5)),
                    fields: vec![],
                },
            ),
            Spanned::new(
                Item::Struct(Struct {
                    name: Spanned::new("Point".to_string(), Span::new(0, 5)),
                    generics: vec![],
                    fields: vec![],
                    methods: vec![],
                    is_pub: false,
                    attributes: vec![],
                    where_clause: vec![],
                }),
                Span::new(0, 1),
            ),
            Spanned::new(
                Item::Struct(Struct {
                    name: Spanned::new("Unused".to_string(), Span::new(0, 6)),
                    generics: vec![],
                    fields: vec![],
                    methods: vec![],
                    is_pub: false,
                    attributes: vec![],
                    where_clause: vec![],
                }),
                Span::new(0, 1),
            ),
        ],
        modules_map: None,
    };

    let shaken = TreeShaker::shake(&module);

    assert_eq!(shaken.items.len(), 2);
    let has_point = shaken.items.iter().any(|item| match &item.node {
        Item::Struct(s) => s.name.node == "Point",
        _ => false,
    });
    let has_unused = shaken.items.iter().any(|item| match &item.node {
        Item::Struct(s) => s.name.node == "Unused",
        _ => false,
    });

    assert!(has_point);
    assert!(!has_unused);
}

#[test]
fn test_no_main_keeps_public_only() {
    let module = Module {
        items: vec![
            make_function("private1", false, Expr::Int(1)),
            make_function("public1", true, Expr::Int(2)),
            make_function("private2", false, Expr::Int(3)),
        ],
        modules_map: None,
    };

    let shaken = TreeShaker::shake(&module);

    // Only main (not present) and public items are entry points
    // So only public1 should remain
    assert_eq!(shaken.items.len(), 1);
    match &shaken.items[0].node {
        Item::Function(f) => assert_eq!(f.name.node, "public1"),
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_empty_module() {
    let module = Module {
        items: vec![],
        modules_map: None,
    };
    let shaken = TreeShaker::shake(&module);
    assert!(shaken.items.is_empty());
}

#[test]
fn test_is_reachable_before_mark() {
    let module = Module {
        items: vec![make_function("test", false, Expr::Int(1))],
        modules_map: None,
    };
    let shaker = TreeShaker::analyze(&module);
    // Before marking, nothing is reachable
    assert!(!shaker.is_reachable("test"));
}

#[test]
fn test_mark_reachable_explicit() {
    let module = Module {
        items: vec![
            make_function("fn1", false, Expr::Int(1)),
            make_function("fn2", false, Expr::Int(2)),
        ],
        modules_map: None,
    };
    let mut shaker = TreeShaker::analyze(&module);
    shaker.mark_reachable(&["fn1"]);

    assert!(shaker.is_reachable("fn1"));
    assert!(!shaker.is_reachable("fn2"));
}

#[test]
fn test_is_builtin_type() {
    assert!(TreeShaker::is_builtin_type("i32"));
    assert!(TreeShaker::is_builtin_type("bool"));
    assert!(TreeShaker::is_builtin_type("str"));
    assert!(TreeShaker::is_builtin_type("String"));
    assert!(TreeShaker::is_builtin_type("f64"));
    assert!(TreeShaker::is_builtin_type("usize"));
    assert!(TreeShaker::is_builtin_type("char"));
    assert!(!TreeShaker::is_builtin_type("MyStruct"));
    assert!(!TreeShaker::is_builtin_type("Vec"));
}

#[test]
fn test_filter_module_preserves_modules_map() {
    let module = Module {
        items: vec![make_function("main", false, Expr::Int(0))],
        modules_map: Some(std::collections::HashMap::new()),
    };
    let mut shaker = TreeShaker::analyze(&module);
    shaker.mark_reachable(&["main"]);
    let filtered = shaker.filter_module(&module);
    assert!(filtered.modules_map.is_some());
}

#[test]
fn test_binary_expr_deps() {
    // main calls helper via binary expression referencing its result
    let module = Module {
        items: vec![
            make_function(
                "main",
                false,
                Expr::Binary {
                    op: BinOp::Add,
                    left: Box::new(Spanned::new(make_call("helper"), Span::new(0, 1))),
                    right: Box::new(Spanned::new(Expr::Int(1), Span::new(0, 1))),
                },
            ),
            make_function("helper", false, Expr::Int(42)),
            make_function("unused", false, Expr::Int(0)),
        ],
        modules_map: None,
    };
    let shaken = TreeShaker::shake(&module);
    let names: Vec<String> = shaken
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Function(f) => Some(f.name.node.clone()),
            _ => None,
        })
        .collect();
    assert!(names.contains(&"main".to_string()));
    assert!(names.contains(&"helper".to_string()));
    assert!(!names.contains(&"unused".to_string()));
}

#[test]
fn test_all_public_items_kept() {
    let module = Module {
        items: vec![
            make_function("pub1", true, Expr::Int(1)),
            make_function("pub2", true, Expr::Int(2)),
            make_function("priv1", false, Expr::Int(3)),
        ],
        modules_map: None,
    };
    let shaken = TreeShaker::shake(&module);
    assert_eq!(shaken.items.len(), 2);
}

#[test]
fn test_circular_dependency_handled() {
    // fn_a calls fn_b, fn_b calls fn_a (circular)
    let module = Module {
        items: vec![
            make_function("main", false, make_call("fn_a")),
            make_function("fn_a", false, make_call("fn_b")),
            make_function("fn_b", false, make_call("fn_a")),
        ],
        modules_map: None,
    };
    let shaken = TreeShaker::shake(&module);
    assert_eq!(shaken.items.len(), 3);
}

#[test]
fn test_self_referencing_function() {
    // Function that calls itself
    let module = Module {
        items: vec![
            make_function("main", false, make_call("recursive")),
            make_function("recursive", false, make_call("recursive")),
        ],
        modules_map: None,
    };
    let shaken = TreeShaker::shake(&module);
    assert_eq!(shaken.items.len(), 2);
}

#[test]
fn test_deep_transitive_chain() {
    let module = Module {
        items: vec![
            make_function("main", false, make_call("a")),
            make_function("a", false, make_call("b")),
            make_function("b", false, make_call("c")),
            make_function("c", false, make_call("d")),
            make_function("d", false, Expr::Int(42)),
            make_function("unreachable", false, Expr::Int(0)),
        ],
        modules_map: None,
    };
    let shaken = TreeShaker::shake(&module);
    assert_eq!(shaken.items.len(), 5); // main, a, b, c, d
}

#[test]
fn test_is_builtin_type_all_primitives() {
    // Integer types
    for ty in &[
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
    ] {
        assert!(TreeShaker::is_builtin_type(ty), "{} should be builtin", ty);
    }
    // Float types
    assert!(TreeShaker::is_builtin_type("f32"));
    assert!(TreeShaker::is_builtin_type("f64"));
    // Other builtins
    assert!(TreeShaker::is_builtin_type("bool"));
    assert!(TreeShaker::is_builtin_type("char"));
    assert!(TreeShaker::is_builtin_type("str"));
    assert!(TreeShaker::is_builtin_type("String"));
    assert!(TreeShaker::is_builtin_type("()"));
    assert!(TreeShaker::is_builtin_type("unit"));
    // Non-builtins
    assert!(!TreeShaker::is_builtin_type("HashMap"));
    assert!(!TreeShaker::is_builtin_type("Option"));
}

#[test]
fn test_ternary_expr_deps() {
    let module = Module {
        items: vec![
            make_function(
                "main",
                false,
                Expr::Ternary {
                    cond: Box::new(Spanned::new(Expr::Bool(true), Span::new(0, 1))),
                    then: Box::new(Spanned::new(make_call("yes_fn"), Span::new(0, 1))),
                    else_: Box::new(Spanned::new(make_call("no_fn"), Span::new(0, 1))),
                },
            ),
            make_function("yes_fn", false, Expr::Int(1)),
            make_function("no_fn", false, Expr::Int(0)),
            make_function("unused", false, Expr::Int(99)),
        ],
        modules_map: None,
    };
    let shaken = TreeShaker::shake(&module);
    let names: Vec<String> = shaken
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Function(f) => Some(f.name.node.clone()),
            _ => None,
        })
        .collect();
    assert!(names.contains(&"yes_fn".to_string()));
    assert!(names.contains(&"no_fn".to_string()));
    assert!(!names.contains(&"unused".to_string()));
}

#[test]
fn test_unary_expr_deps() {
    let module = Module {
        items: vec![
            make_function(
                "main",
                false,
                Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(Spanned::new(make_call("neg_fn"), Span::new(0, 1))),
                },
            ),
            make_function("neg_fn", false, Expr::Int(42)),
        ],
        modules_map: None,
    };
    let shaken = TreeShaker::shake(&module);
    assert_eq!(shaken.items.len(), 2);
}

#[test]
fn test_only_main_no_deps() {
    let module = Module {
        items: vec![make_function("main", false, Expr::Int(42))],
        modules_map: None,
    };
    let shaken = TreeShaker::shake(&module);
    assert_eq!(shaken.items.len(), 1);
}

#[test]
fn test_method_call_deps() {
    let module = Module {
        items: vec![
            make_function(
                "main",
                false,
                Expr::MethodCall {
                    receiver: Box::new(Spanned::new(
                        Expr::Ident("obj".to_string()),
                        Span::new(0, 1),
                    )),
                    method: Spanned::new("do_thing".to_string(), Span::new(0, 1)),
                    args: vec![Spanned::new(make_call("helper"), Span::new(0, 1))],
                },
            ),
            make_function("helper", false, Expr::Int(1)),
            make_function("unrelated", false, Expr::Int(0)),
        ],
        modules_map: None,
    };
    let shaken = TreeShaker::shake(&module);
    let names: Vec<String> = shaken
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Function(f) => Some(f.name.node.clone()),
            _ => None,
        })
        .collect();
    assert!(names.contains(&"helper".to_string()));
    assert!(!names.contains(&"unrelated".to_string()));
}

#[test]
fn test_analyze_returns_correct_dep_count() {
    let module = Module {
        items: vec![
            make_function("main", false, make_call("a")),
            make_function("a", false, Expr::Int(1)),
        ],
        modules_map: None,
    };
    let shaker = TreeShaker::analyze(&module);
    // main should depend on 'a'
    assert!(shaker.deps.get("main").unwrap().contains("a"));
}

#[test]
fn test_shaker_clone() {
    let module = Module {
        items: vec![make_function("main", false, Expr::Int(0))],
        modules_map: None,
    };
    let shaker = TreeShaker::analyze(&module);
    let cloned = shaker.clone();
    assert_eq!(shaker.deps.len(), cloned.deps.len());
}
