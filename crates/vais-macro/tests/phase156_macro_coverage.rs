//! Phase 156: Additional unit tests for vais-macro coverage
//!
//! Focus areas:
//! - MacroRegistry advanced operations
//! - Macro expansion correctness (various MetaVarKind: ident, ty, lit, tt, pat, stmt)
//! - Hygiene validation across nested contexts
//! - Error cases (invalid patterns, recursion limit, undefined metavars)
//! - ProcMacro system edge cases
//! - Derive system coverage
//! - tokens_to_string edge cases
//! - Property macros registration
//! - AsyncMacroExpander deeper coverage
//! - ExpansionError display

use vais_ast::{
    Delimiter, MacroDef, MacroInvoke, MacroLiteral, MacroPattern, MacroPatternElement, MacroRule,
    MacroTemplate, MacroTemplateElement, MacroToken, MetaVarKind, Module, RepetitionKind, Span,
    Spanned,
};
use vais_macro::{
    collect_macros, expand_macros, process_derives, register_async_macros,
    register_property_macros, tokens_to_string, AstExpander, AsyncMacroExpander, DeriveRegistry,
    HygienicContext, LiteralToken, MacroExpander, MacroRegistry, ProcMacro, ProcMacroError,
    ProcMacroKind, ProcMacroRegistry, ProcMacroResult, TokenStream, TokenTree, ASSERT_PROP_MACRO,
    CHECK_MACRO, FORALL_MACRO, JOIN_MACRO, SELECT_MACRO, TIMEOUT_MACRO,
};

// ==================== MacroRegistry advanced ====================

#[test]
fn test_registry_overwrite_existing() {
    let mut registry = MacroRegistry::new();

    registry.register(MacroDef {
        name: Spanned::new("foo".to_string(), Span::new(0, 3)),
        rules: vec![],
        is_pub: false,
    });
    assert_eq!(registry.macros_count(), 1);

    // Overwrite with a new def — count stays 1
    registry.register(MacroDef {
        name: Spanned::new("foo".to_string(), Span::new(0, 3)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Empty,
            template: MacroTemplate::Empty,
        }],
        is_pub: false,
    });
    assert_eq!(registry.macros_count(), 1);

    let def = registry.get("foo").unwrap();
    assert_eq!(def.rules.len(), 1);
}

#[test]
fn test_registry_many_macros() {
    let mut registry = MacroRegistry::new();
    for i in 0..20 {
        registry.register(MacroDef {
            name: Spanned::new(format!("macro_{}", i), Span::new(0, 5)),
            rules: vec![],
            is_pub: false,
        });
    }
    assert_eq!(registry.macros_count(), 20);
    for i in 0..20 {
        assert!(registry.contains(&format!("macro_{}", i)));
    }
    assert!(!registry.contains("macro_20"));
}

#[test]
fn test_registry_pub_macro_stored() {
    let mut registry = MacroRegistry::new();
    registry.register(MacroDef {
        name: Spanned::new("pub_mac".to_string(), Span::new(0, 7)),
        rules: vec![],
        is_pub: true,
    });
    let def = registry.get("pub_mac").unwrap();
    assert!(def.is_pub);
}

// ==================== MacroExpander MetaVarKind coverage ====================

fn make_simple_registry_with(
    name: &str,
    kind: MetaVarKind,
    template_suffix: Vec<MacroTemplateElement>,
) -> MacroRegistry {
    let mut registry = MacroRegistry::new();
    registry.register(MacroDef {
        name: Spanned::new(name.to_string(), Span::new(0, name.len())),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![MacroPatternElement::MetaVar {
                name: "v".to_string(),
                kind,
            }]),
            template: MacroTemplate::Sequence({
                let mut elems = vec![MacroTemplateElement::MetaVar("v".to_string())];
                elems.extend(template_suffix);
                elems
            }),
        }],
        is_pub: false,
    });
    registry
}

#[test]
fn test_expander_metavar_ident_kind() {
    let registry = make_simple_registry_with("wrap_ident", MetaVarKind::Ident, vec![]);
    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("wrap_ident".to_string(), Span::new(0, 10)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Ident("myvar".to_string())],
    };
    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "myvar");
}

#[test]
fn test_expander_metavar_ident_rejects_literal() {
    let registry = make_simple_registry_with("wrap_ident2", MetaVarKind::Ident, vec![]);
    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("wrap_ident2".to_string(), Span::new(0, 11)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Literal(MacroLiteral::Int(5))],
    };
    // Ident kind should fail to match a literal
    let result = expander.expand(&invoke);
    assert!(result.is_err());
}

#[test]
fn test_expander_metavar_lit_kind() {
    let registry = make_simple_registry_with("wrap_lit", MetaVarKind::Lit, vec![]);
    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("wrap_lit".to_string(), Span::new(0, 8)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Literal(MacroLiteral::Float(3.14))],
    };
    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "3.14");
}

#[test]
fn test_expander_metavar_tt_kind_captures_ident() {
    let registry = make_simple_registry_with("wrap_tt", MetaVarKind::Tt, vec![]);
    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("wrap_tt".to_string(), Span::new(0, 7)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Ident("anything".to_string())],
    };
    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "anything");
}

#[test]
fn test_expander_metavar_tt_kind_captures_punct() {
    let registry = make_simple_registry_with("wrap_tt2", MetaVarKind::Tt, vec![]);
    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("wrap_tt2".to_string(), Span::new(0, 8)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Punct('@')],
    };
    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "@");
}

#[test]
fn test_expander_metavar_ty_kind() {
    let registry = make_simple_registry_with("wrap_ty", MetaVarKind::Ty, vec![]);
    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("wrap_ty".to_string(), Span::new(0, 7)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Ident("i64".to_string())],
    };
    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "i64");
}

#[test]
fn test_expander_no_matching_rule_error() {
    let mut registry = MacroRegistry::new();
    // macro that only matches empty; invoke with tokens → NoMatchingRule
    registry.register(MacroDef {
        name: Spanned::new("strict".to_string(), Span::new(0, 6)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Empty,
            template: MacroTemplate::Empty,
        }],
        is_pub: false,
    });
    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("strict".to_string(), Span::new(0, 6)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Literal(MacroLiteral::Int(1))],
    };
    let result = expander.expand(&invoke);
    assert!(result.is_err());
}

#[test]
fn test_expander_one_or_more_repetition() {
    let mut registry = MacroRegistry::new();
    // macro plus! { ($($x:expr)+) => { [$($x),*] } }
    registry.register(MacroDef {
        name: Spanned::new("plus".to_string(), Span::new(0, 4)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![MacroPatternElement::Repetition {
                patterns: vec![MacroPatternElement::MetaVar {
                    name: "x".to_string(),
                    kind: MetaVarKind::Expr,
                }],
                separator: None,
                kind: RepetitionKind::OneOrMore,
            }]),
            template: MacroTemplate::Sequence(vec![MacroTemplateElement::Group {
                delimiter: Delimiter::Bracket,
                content: vec![MacroTemplateElement::Repetition {
                    elements: vec![MacroTemplateElement::MetaVar("x".to_string())],
                    separator: Some(MacroToken::Punct(',')),
                    kind: RepetitionKind::ZeroOrMore,
                }],
            }]),
        }],
        is_pub: false,
    });

    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("plus".to_string(), Span::new(0, 4)),
        delimiter: Delimiter::Paren,
        tokens: vec![
            MacroToken::Literal(MacroLiteral::Int(10)),
            MacroToken::Literal(MacroLiteral::Int(20)),
        ],
    };
    let result = expander.expand(&invoke).unwrap();
    // No separator in pattern, template uses comma separator; tokens adjacent without separator
    assert!(tokens_to_string(&result).contains("10"));
    assert!(tokens_to_string(&result).contains("20"));
}

#[test]
fn test_expander_zero_or_one_repetition() {
    let mut registry = MacroRegistry::new();
    // macro opt! { ($($x:expr)?) => { $($x)? } }
    registry.register(MacroDef {
        name: Spanned::new("opt".to_string(), Span::new(0, 3)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![MacroPatternElement::Repetition {
                patterns: vec![MacroPatternElement::MetaVar {
                    name: "x".to_string(),
                    kind: MetaVarKind::Expr,
                }],
                separator: None,
                kind: RepetitionKind::ZeroOrOne,
            }]),
            template: MacroTemplate::Sequence(vec![MacroTemplateElement::Repetition {
                elements: vec![MacroTemplateElement::MetaVar("x".to_string())],
                separator: None,
                kind: RepetitionKind::ZeroOrOne,
            }]),
        }],
        is_pub: false,
    });

    // Zero case
    let expander = MacroExpander::new(&registry);
    let invoke_zero = MacroInvoke {
        name: Spanned::new("opt".to_string(), Span::new(0, 3)),
        delimiter: Delimiter::Paren,
        tokens: vec![],
    };
    let result_zero = expander.expand(&invoke_zero).unwrap();
    assert_eq!(tokens_to_string(&result_zero), "");

    // One case
    let invoke_one = MacroInvoke {
        name: Spanned::new("opt".to_string(), Span::new(0, 3)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Literal(MacroLiteral::Int(42))],
    };
    let result_one = expander.expand(&invoke_one).unwrap();
    assert_eq!(tokens_to_string(&result_one), "42");
}

#[test]
fn test_expander_multiple_rules_first_wins() {
    let mut registry = MacroRegistry::new();
    // Two rules: first matches empty, second matches expr
    registry.register(MacroDef {
        name: Spanned::new("multi".to_string(), Span::new(0, 5)),
        rules: vec![
            MacroRule {
                pattern: MacroPattern::Empty,
                template: MacroTemplate::Sequence(vec![MacroTemplateElement::Token(
                    MacroToken::Literal(MacroLiteral::Int(0)),
                )]),
            },
            MacroRule {
                pattern: MacroPattern::Sequence(vec![MacroPatternElement::MetaVar {
                    name: "x".to_string(),
                    kind: MetaVarKind::Expr,
                }]),
                template: MacroTemplate::Sequence(vec![MacroTemplateElement::MetaVar(
                    "x".to_string(),
                )]),
            },
        ],
        is_pub: false,
    });

    let expander = MacroExpander::new(&registry);

    // Empty → first rule → 0
    let invoke_empty = MacroInvoke {
        name: Spanned::new("multi".to_string(), Span::new(0, 5)),
        delimiter: Delimiter::Paren,
        tokens: vec![],
    };
    let result_empty = expander.expand(&invoke_empty).unwrap();
    assert_eq!(tokens_to_string(&result_empty), "0");

    // With token → second rule → token
    let invoke_expr = MacroInvoke {
        name: Spanned::new("multi".to_string(), Span::new(0, 5)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Literal(MacroLiteral::Int(7))],
    };
    let result_expr = expander.expand(&invoke_expr).unwrap();
    assert_eq!(tokens_to_string(&result_expr), "7");
}

#[test]
fn test_expander_template_token_match() {
    let mut registry = MacroRegistry::new();
    // macro with a literal token in the pattern (not a metavar)
    registry.register(MacroDef {
        name: Spanned::new("kw".to_string(), Span::new(0, 2)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![
                MacroPatternElement::Token(MacroToken::Ident("let".to_string())),
                MacroPatternElement::MetaVar {
                    name: "x".to_string(),
                    kind: MetaVarKind::Ident,
                },
            ]),
            template: MacroTemplate::Sequence(vec![
                MacroTemplateElement::Token(MacroToken::Ident("bind".to_string())),
                MacroTemplateElement::MetaVar("x".to_string()),
            ]),
        }],
        is_pub: false,
    });

    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("kw".to_string(), Span::new(0, 2)),
        delimiter: Delimiter::Paren,
        tokens: vec![
            MacroToken::Ident("let".to_string()),
            MacroToken::Ident("y".to_string()),
        ],
    };
    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "bind y");
}

#[test]
fn test_expander_bracket_group_template() {
    let mut registry = MacroRegistry::new();
    registry.register(MacroDef {
        name: Spanned::new("bracket".to_string(), Span::new(0, 7)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![MacroPatternElement::MetaVar {
                name: "x".to_string(),
                kind: MetaVarKind::Expr,
            }]),
            template: MacroTemplate::Sequence(vec![MacroTemplateElement::Group {
                delimiter: Delimiter::Bracket,
                content: vec![MacroTemplateElement::MetaVar("x".to_string())],
            }]),
        }],
        is_pub: false,
    });

    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("bracket".to_string(), Span::new(0, 7)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Literal(MacroLiteral::Int(99))],
    };
    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "[99]");
}

#[test]
fn test_expander_brace_group_template() {
    let mut registry = MacroRegistry::new();
    registry.register(MacroDef {
        name: Spanned::new("brace".to_string(), Span::new(0, 5)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![MacroPatternElement::MetaVar {
                name: "e".to_string(),
                kind: MetaVarKind::Expr,
            }]),
            template: MacroTemplate::Sequence(vec![MacroTemplateElement::Group {
                delimiter: Delimiter::Brace,
                content: vec![MacroTemplateElement::MetaVar("e".to_string())],
            }]),
        }],
        is_pub: false,
    });

    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("brace".to_string(), Span::new(0, 5)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Ident("x".to_string())],
    };
    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "{x}");
}

#[test]
fn test_expander_bool_literal() {
    let registry = make_simple_registry_with("wrap_bool", MetaVarKind::Lit, vec![]);
    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("wrap_bool".to_string(), Span::new(0, 9)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Literal(MacroLiteral::Bool(true))],
    };
    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "true");
}

#[test]
fn test_expander_string_literal_in_template() {
    let registry = make_simple_registry_with("wrap_str", MetaVarKind::Lit, vec![]);
    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("wrap_str".to_string(), Span::new(0, 8)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Literal(MacroLiteral::String(
            "hello".to_string(),
        ))],
    };
    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "\"hello\"");
}

// ==================== tokens_to_string edge cases ====================

#[test]
fn test_tokens_to_string_ident_space() {
    // Two adjacent idents need a space
    let tokens = vec![
        MacroToken::Ident("let".to_string()),
        MacroToken::Ident("x".to_string()),
    ];
    let output = tokens_to_string(&tokens);
    assert_eq!(output, "let x");
}

#[test]
fn test_tokens_to_string_ident_then_literal() {
    let tokens = vec![
        MacroToken::Ident("x".to_string()),
        MacroToken::Literal(MacroLiteral::Int(42)),
    ];
    let output = tokens_to_string(&tokens);
    // ident followed by literal should also get a space
    assert_eq!(output, "x 42");
}

#[test]
fn test_tokens_to_string_punct_no_space() {
    let tokens = vec![
        MacroToken::Punct('('),
        MacroToken::Ident("x".to_string()),
        MacroToken::Punct(')'),
    ];
    let output = tokens_to_string(&tokens);
    assert_eq!(output, "(x)");
}

#[test]
fn test_tokens_to_string_nested_group() {
    let tokens = vec![MacroToken::Group(
        Delimiter::Paren,
        vec![MacroToken::Group(
            Delimiter::Bracket,
            vec![MacroToken::Literal(MacroLiteral::Int(1))],
        )],
    )];
    let output = tokens_to_string(&tokens);
    assert_eq!(output, "([1])");
}

#[test]
fn test_tokens_to_string_float_literal() {
    let tokens = vec![MacroToken::Literal(MacroLiteral::Float(2.718))];
    let output = tokens_to_string(&tokens);
    assert!(output.starts_with("2.718"));
}

// ==================== HygienicContext deeper coverage ====================

#[test]
fn test_hygienic_context_multiple_vars_unique() {
    let mut ctx = HygienicContext::new();
    let names: Vec<_> = (0..10)
        .map(|i| ctx.hygienize(&format!("var{}", i)))
        .collect();
    // All hygienic names should be unique
    let unique_count = {
        let mut set = std::collections::HashSet::new();
        names.iter().for_each(|n| {
            set.insert(n.clone());
        });
        set.len()
    };
    assert_eq!(unique_count, 10);
}

#[test]
fn test_hygienic_context_same_name_different_contexts() {
    let mut ctx = HygienicContext::new();

    let outer = ctx.hygienize("x");
    ctx.push_context();
    let inner = ctx.hygienize("x");
    ctx.pop_context();

    // Same source name but different context → different hygienic name
    assert_ne!(outer, inner);
}

#[test]
fn test_hygienic_context_pop_restores() {
    let mut ctx = HygienicContext::new();
    let before = ctx.current_context();
    ctx.push_context();
    ctx.push_context();
    ctx.pop_context();
    ctx.pop_context();
    assert_eq!(ctx.current_context(), before);
}

#[test]
fn test_hygienic_lookup_after_context_switch() {
    let mut ctx = HygienicContext::new();
    let h1 = ctx.hygienize("val");
    assert!(ctx.lookup("val").is_some());
    assert_eq!(ctx.lookup("val"), Some(&h1));

    // In new context, old lookup still valid (name_map is flat)
    ctx.push_context();
    // The name "val" was mapped in outer context; lookup is global
    assert!(ctx.lookup("val").is_some());
    ctx.pop_context();
}

// ==================== ExpansionError display ====================

#[test]
fn test_expansion_error_recursion_limit_display() {
    use vais_macro::ExpansionError;

    let err = ExpansionError::RecursionLimit {
        macro_name: "recurse".to_string(),
        depth: 128,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("recurse"));
    assert!(msg.contains("128"));
}

#[test]
fn test_expansion_error_parse_error_display() {
    use vais_macro::ExpansionError;

    let err = ExpansionError::ParseError("unexpected token".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("unexpected token"));
}

#[test]
fn test_expansion_error_hygiene_display() {
    use vais_macro::ExpansionError;

    let err = ExpansionError::HygienicError("collision".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("collision"));
}

#[test]
fn test_expansion_error_macro_error_display() {
    use vais_macro::ExpansionError;

    let err = ExpansionError::MacroError("undefined x".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("undefined x"));
}

// ==================== AstExpander with modules ====================

#[test]
fn test_ast_expander_with_non_empty_module() {
    use vais_parser::parse;

    let registry = MacroRegistry::new();
    let mut expander = AstExpander::new(&registry);

    let source = "F add(a: i64, b: i64) -> i64 = a + b";
    let module = parse(source).unwrap();
    let result = expander.expand_module(module);
    assert!(result.is_ok());
}

#[test]
fn test_expand_macros_with_undefined_macro_in_source() {
    use vais_parser::parse;

    let registry = MacroRegistry::new(); // empty registry
    let source = "F test() = 42";
    let module = parse(source).unwrap();
    // No macro invocations → no error
    let result = expand_macros(module, &registry);
    assert!(result.is_ok());
}

#[test]
fn test_collect_macros_multiple_defs() {
    use vais_parser::parse;

    let source = r#"
        macro foo! { ($x:expr) => { $x } }
        macro bar! { ($a:expr, $b:expr) => { $a + $b } }
        F dummy() = 1
    "#;

    let module = parse(source).unwrap();
    let mut registry = MacroRegistry::new();
    collect_macros(&module, &mut registry);

    assert!(registry.contains("foo"));
    assert!(registry.contains("bar"));
    assert_eq!(registry.macros_count(), 2);
}

// ==================== Derive system deeper coverage ====================

#[test]
fn test_derive_registry_all_supported() {
    let registry = DeriveRegistry::new();
    for name in &["Debug", "Clone", "PartialEq", "Default", "Hash", "Error"] {
        assert!(registry.is_supported(name), "{} should be supported", name);
    }
}

#[test]
fn test_derive_registry_case_sensitive() {
    let registry = DeriveRegistry::new();
    // All lowercase variants should NOT be supported
    assert!(!registry.is_supported("debug"));
    assert!(!registry.is_supported("clone"));
    assert!(!registry.is_supported("partial_eq"));
}

#[test]
fn test_process_derives_module_with_items() {
    use vais_parser::parse;

    let source = r#"
        S Point { x: i64, y: i64 }
        F origin() -> Point = Point { x: 0, y: 0 }
    "#;
    let module = parse(source).unwrap();
    let mut module = module;
    let result = process_derives(&mut module);
    assert!(result.is_ok());
}

// ==================== ProcMacro deeper coverage ====================

#[test]
fn test_token_stream_from_tokens() {
    let tokens = vec![
        TokenTree::Ident("x".to_string()),
        TokenTree::Punct('+'),
        TokenTree::Literal(LiteralToken::Integer(1)),
    ];
    let stream = TokenStream::from_tokens(tokens);
    assert_eq!(stream.len(), 3);
    assert!(!stream.is_empty());
}

#[test]
fn test_token_stream_iter() {
    let mut stream = TokenStream::new();
    stream.push(TokenTree::Ident("a".to_string()));
    stream.push(TokenTree::Ident("b".to_string()));
    let collected: Vec<_> = stream.iter().collect();
    assert_eq!(collected.len(), 2);
}

#[test]
fn test_token_tree_group() {
    let inner_stream = TokenStream::from_tokens(vec![TokenTree::Literal(LiteralToken::Integer(5))]);
    let group = TokenTree::Group {
        delimiter: vais_macro::proc_macro::Delimiter::Parenthesis,
        stream: inner_stream,
    };
    if let TokenTree::Group { stream, .. } = group {
        assert_eq!(stream.len(), 1);
    } else {
        panic!("Expected Group");
    }
}

#[test]
fn test_literal_token_variants() {
    let int_lit = LiteralToken::Integer(42);
    let float_lit = LiteralToken::Float(3.14);
    let str_lit = LiteralToken::String("hello".to_string());
    let char_lit = LiteralToken::Char('x');
    let bool_lit = LiteralToken::Bool(false);

    // Just confirm they can be created and matched
    assert!(matches!(int_lit, LiteralToken::Integer(42)));
    assert!(matches!(float_lit, LiteralToken::Float(_)));
    assert!(matches!(str_lit, LiteralToken::String(_)));
    assert!(matches!(char_lit, LiteralToken::Char('x')));
    assert!(matches!(bool_lit, LiteralToken::Bool(false)));
}

#[test]
fn test_proc_macro_env_builtin() {
    let registry = ProcMacroRegistry::new();
    let env = registry.get("env").unwrap();
    let mut input = TokenStream::new();
    input.push(TokenTree::Literal(LiteralToken::String("PATH".to_string())));
    let result = env.expand(input);
    // May succeed or fail depending on env var — just test no panic
    let _ = result;
}

#[test]
fn test_proc_macro_line_builtin() {
    let registry = ProcMacroRegistry::new();
    let line_mac = registry.get("line").unwrap();
    let result = line_mac.expand(TokenStream::new()).unwrap();
    assert_eq!(result.len(), 1);
    assert!(matches!(
        result.iter().next(),
        Some(TokenTree::Literal(LiteralToken::Integer(_)))
    ));
}

#[test]
fn test_proc_macro_column_builtin() {
    let registry = ProcMacroRegistry::new();
    let col_mac = registry.get("column").unwrap();
    let result = col_mac.expand(TokenStream::new()).unwrap();
    assert_eq!(result.len(), 1);
    assert!(matches!(
        result.iter().next(),
        Some(TokenTree::Literal(LiteralToken::Integer(_)))
    ));
}

#[test]
fn test_proc_macro_file_builtin() {
    let registry = ProcMacroRegistry::new();
    let file_mac = registry.get("file").unwrap();
    let result = file_mac.expand(TokenStream::new()).unwrap();
    assert_eq!(result.len(), 1);
    assert!(matches!(
        result.iter().next(),
        Some(TokenTree::Literal(LiteralToken::String(_)))
    ));
}

#[test]
fn test_custom_attribute_proc_macro() {
    struct LogMacro;

    impl ProcMacro for LogMacro {
        fn name(&self) -> &str {
            "log_calls"
        }

        fn kind(&self) -> ProcMacroKind {
            ProcMacroKind::Attribute
        }

        fn expand(&self, input: TokenStream) -> ProcMacroResult<TokenStream> {
            // passthrough
            Ok(input)
        }
    }

    let mut registry = ProcMacroRegistry::new();
    registry.register(Box::new(LogMacro));
    assert!(registry.contains("log_calls"));
}

#[test]
fn test_custom_derive_proc_macro() {
    struct MyDerive;

    impl ProcMacro for MyDerive {
        fn name(&self) -> &str {
            "Serialize"
        }

        fn kind(&self) -> ProcMacroKind {
            ProcMacroKind::Derive
        }

        fn expand(&self, _input: TokenStream) -> ProcMacroResult<TokenStream> {
            Ok(TokenStream::new())
        }
    }

    let mut registry = ProcMacroRegistry::new();
    registry.register(Box::new(MyDerive));
    assert!(registry.contains("Serialize"));

    let mac = registry.get("Serialize").unwrap();
    assert!(matches!(mac.kind(), ProcMacroKind::Derive));
}

#[test]
fn test_proc_macro_error_with_and_without_span() {
    let err1 = ProcMacroError::new("no span");
    assert!(err1.span.is_none());

    let err2 = ProcMacroError::with_span("has span", Span::new(5, 10));
    assert!(err2.span.is_some());

    let span = err2.span.unwrap();
    assert_eq!(span.start, 5);
    assert_eq!(span.end, 10);
}

// ==================== Async macro expander deeper coverage ====================

#[test]
fn test_async_join_five_futures() {
    let mut expander = AsyncMacroExpander::new();
    let futures: Vec<Vec<MacroToken>> = (0..5)
        .map(|i| vec![MacroToken::Ident(format!("f{}", i))])
        .collect();
    let result = expander.expand_join(&futures).unwrap();
    let output = tokens_to_string(&result);
    assert!(output.contains("join"));
    assert!(output.contains(".await"));
}

#[test]
fn test_async_timeout_zero_duration() {
    let mut expander = AsyncMacroExpander::new();
    let duration = vec![MacroToken::Literal(MacroLiteral::Int(0))];
    let future = vec![MacroToken::Ident("fut".to_string())];
    let result = expander.expand_timeout(&duration, &future).unwrap();
    let output = tokens_to_string(&result);
    assert!(output.contains("0"));
}

#[test]
fn test_async_register_all_three() {
    let mut registry = MacroRegistry::new();
    register_async_macros(&mut registry);
    assert!(registry.contains(JOIN_MACRO));
    assert!(registry.contains(SELECT_MACRO));
    assert!(registry.contains(TIMEOUT_MACRO));
    assert_eq!(registry.macros_count(), 3);
}

// ==================== Property macro registration ====================

#[test]
fn test_property_macros_registration() {
    let mut registry = MacroRegistry::new();
    register_property_macros(&mut registry);
    assert!(registry.macros_count() >= 2); // forall + check at minimum
}

#[test]
fn test_property_macro_constants_non_empty() {
    assert!(!FORALL_MACRO.is_empty());
    assert!(!CHECK_MACRO.is_empty());
    assert!(!ASSERT_PROP_MACRO.is_empty());
}

#[test]
fn test_property_macro_constants_contain_names() {
    assert!(FORALL_MACRO.contains("forall"));
    assert!(CHECK_MACRO.contains("check"));
    assert!(ASSERT_PROP_MACRO.contains("assert_prop"));
}

// ==================== MacroError display ====================

#[test]
fn test_macro_error_display_undefined() {
    use vais_macro::MacroError;
    let err = MacroError::UndefinedMacro("foo".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("foo"));
    assert!(msg.contains("Undefined"));
}

#[test]
fn test_macro_error_display_no_matching_rule() {
    use vais_macro::MacroError;
    let err = MacroError::NoMatchingRule("bar".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("bar"));
}

#[test]
fn test_macro_error_display_undefined_metavar() {
    use vais_macro::MacroError;
    let err = MacroError::UndefinedMetaVar("x".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("x"));
}

#[test]
fn test_macro_error_display_repetition_mismatch() {
    use vais_macro::MacroError;
    let err = MacroError::RepetitionMismatch("items".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("items"));
}

#[test]
fn test_macro_error_display_invalid_fragment() {
    use vais_macro::MacroError;
    let err = MacroError::InvalidFragment {
        expected: "expr".to_string(),
        found: "ident".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("expr"));
    assert!(msg.contains("ident"));
}
