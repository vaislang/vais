//! Integration tests for vais-macro crate
//!
//! Tests the full macro expansion pipeline including:
//! - MacroRegistry and MacroExpander
//! - ProcMacro system
//! - Async macros (join!, select!, timeout!)
//! - Derive system
//! - AST expansion with HygienicContext

use vais_ast::{
    Delimiter, MacroDef, MacroInvoke, MacroLiteral, MacroPattern, MacroPatternElement, MacroRule,
    MacroTemplate, MacroTemplateElement, MacroToken, MetaVarKind, Module, RepetitionKind, Span,
    Spanned,
};
use vais_macro::{
    // Expansion
    collect_macros,
    expand_macros,
    // Derive
    process_derives,
    // Async
    register_async_macros,
    // Core
    tokens_to_string,
    AstExpander,
    AsyncMacroExpander,
    DeriveRegistry,
    HygienicContext,
    // Proc macros
    LiteralToken,
    MacroExpander,
    MacroRegistry,
    ProcMacro,
    ProcMacroError,
    ProcMacroKind,
    ProcMacroRegistry,
    ProcMacroResult,
    TokenStream,
    TokenTree,
    JOIN_MACRO,
    SELECT_MACRO,
    TIMEOUT_MACRO,
};

// ============================ 1. MacroRegistry Basic Operations ============================

#[test]
fn test_registry_register_and_lookup() {
    let mut registry = MacroRegistry::new();

    let macro_def = MacroDef {
        name: Spanned::new("test_macro".to_string(), Span::new(0, 10)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Empty,
            template: MacroTemplate::Empty,
        }],
        is_pub: false,
    };

    registry.register(macro_def);

    assert!(registry.get("test_macro").is_some());
    assert_eq!(registry.macros_count(), 1);
}

#[test]
fn test_registry_lookup_nonexistent() {
    let registry = MacroRegistry::new();
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn test_registry_macros_count() {
    let mut registry = MacroRegistry::new();
    assert_eq!(registry.macros_count(), 0);

    registry.register(MacroDef {
        name: Spanned::new("macro1".to_string(), Span::new(0, 6)),
        rules: vec![],
        is_pub: false,
    });

    registry.register(MacroDef {
        name: Spanned::new("macro2".to_string(), Span::new(0, 6)),
        rules: vec![],
        is_pub: false,
    });

    assert_eq!(registry.macros_count(), 2);
}

#[test]
fn test_registry_contains() {
    let mut registry = MacroRegistry::new();

    registry.register(MacroDef {
        name: Spanned::new("exists".to_string(), Span::new(0, 6)),
        rules: vec![],
        is_pub: false,
    });

    assert!(registry.contains("exists"));
    assert!(!registry.contains("does_not_exist"));
}

// ============================ 2. MacroExpander Macro Expansion ============================

#[test]
fn test_expander_empty_pattern() {
    let mut registry = MacroRegistry::new();

    registry.register(MacroDef {
        name: Spanned::new("unit".to_string(), Span::new(0, 4)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Empty,
            template: MacroTemplate::Sequence(vec![
                MacroTemplateElement::Token(MacroToken::Punct('(')),
                MacroTemplateElement::Token(MacroToken::Punct(')')),
            ]),
        }],
        is_pub: false,
    });

    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("unit".to_string(), Span::new(0, 4)),
        delimiter: Delimiter::Paren,
        tokens: vec![],
    };

    let result = expander.expand(&invoke).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(tokens_to_string(&result), "()");
}

#[test]
fn test_expander_simple_substitution() {
    let mut registry = MacroRegistry::new();

    // macro id! { ($x:expr) => { $x } }
    registry.register(MacroDef {
        name: Spanned::new("id".to_string(), Span::new(0, 2)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![MacroPatternElement::MetaVar {
                name: "x".to_string(),
                kind: MetaVarKind::Expr,
            }]),
            template: MacroTemplate::Sequence(vec![MacroTemplateElement::MetaVar("x".to_string())]),
        }],
        is_pub: false,
    });

    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("id".to_string(), Span::new(0, 2)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Literal(MacroLiteral::Int(99))],
    };

    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "99");
}

#[test]
fn test_expander_repetition_pattern() {
    let mut registry = MacroRegistry::new();

    // macro list! { ($($x:expr),*) => { [$($x),*] } }
    registry.register(MacroDef {
        name: Spanned::new("list".to_string(), Span::new(0, 4)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![MacroPatternElement::Repetition {
                patterns: vec![MacroPatternElement::MetaVar {
                    name: "x".to_string(),
                    kind: MetaVarKind::Expr,
                }],
                separator: Some(MacroToken::Punct(',')),
                kind: RepetitionKind::ZeroOrMore,
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
        name: Spanned::new("list".to_string(), Span::new(0, 4)),
        delimiter: Delimiter::Paren,
        tokens: vec![
            MacroToken::Literal(MacroLiteral::Int(1)),
            MacroToken::Punct(','),
            MacroToken::Literal(MacroLiteral::Int(2)),
            MacroToken::Punct(','),
            MacroToken::Literal(MacroLiteral::Int(3)),
        ],
    };

    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "[1,2,3]");
}

#[test]
fn test_expander_undefined_macro_error() {
    let registry = MacroRegistry::new();
    let expander = MacroExpander::new(&registry);

    let invoke = MacroInvoke {
        name: Spanned::new("undefined".to_string(), Span::new(0, 9)),
        delimiter: Delimiter::Paren,
        tokens: vec![],
    };

    let result = expander.expand(&invoke);
    assert!(result.is_err());
}

#[test]
fn test_expander_multiple_metavars() {
    let mut registry = MacroRegistry::new();

    // macro add! { ($a:expr, $b:expr) => { $a + $b } }
    registry.register(MacroDef {
        name: Spanned::new("add".to_string(), Span::new(0, 3)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![
                MacroPatternElement::MetaVar {
                    name: "a".to_string(),
                    kind: MetaVarKind::Expr,
                },
                MacroPatternElement::Token(MacroToken::Punct(',')),
                MacroPatternElement::MetaVar {
                    name: "b".to_string(),
                    kind: MetaVarKind::Expr,
                },
            ]),
            template: MacroTemplate::Sequence(vec![
                MacroTemplateElement::MetaVar("a".to_string()),
                MacroTemplateElement::Token(MacroToken::Punct('+')),
                MacroTemplateElement::MetaVar("b".to_string()),
            ]),
        }],
        is_pub: false,
    });

    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("add".to_string(), Span::new(0, 3)),
        delimiter: Delimiter::Paren,
        tokens: vec![
            MacroToken::Literal(MacroLiteral::Int(10)),
            MacroToken::Punct(','),
            MacroToken::Literal(MacroLiteral::Int(20)),
        ],
    };

    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "10+20");
}

// ============================ 3. ProcMacro System ============================

#[test]
fn test_token_stream_creation() {
    let stream = TokenStream::new();
    assert!(stream.is_empty());
    assert_eq!(stream.len(), 0);
}

#[test]
fn test_token_stream_push_extend() {
    let mut stream = TokenStream::new();
    stream.push(TokenTree::Ident("hello".to_string()));
    stream.push(TokenTree::Punct('!'));

    assert_eq!(stream.len(), 2);

    let mut stream2 = TokenStream::new();
    stream2.push(TokenTree::Ident("world".to_string()));

    stream.extend(stream2);
    assert_eq!(stream.len(), 3);
}

#[test]
fn test_builtin_stringify_macro() {
    let registry = ProcMacroRegistry::new();
    let stringify = registry.get("stringify").unwrap();

    let mut input = TokenStream::new();
    input.push(TokenTree::Ident("x".to_string()));
    input.push(TokenTree::Punct('+'));
    input.push(TokenTree::Literal(LiteralToken::Integer(1)));

    let result = stringify.expand(input).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_builtin_concat_macro() {
    let registry = ProcMacroRegistry::new();
    let concat = registry.get("concat").unwrap();

    let mut input = TokenStream::new();
    input.push(TokenTree::Literal(LiteralToken::String(
        "hello".to_string(),
    )));
    input.push(TokenTree::Punct(','));
    input.push(TokenTree::Literal(LiteralToken::String(
        " world".to_string(),
    )));

    let result = concat.expand(input).unwrap();
    assert_eq!(result.len(), 1);
}

#[test]
fn test_proc_macro_registry_builtins() {
    let registry = ProcMacroRegistry::new();

    assert!(registry.contains("stringify"));
    assert!(registry.contains("concat"));
    assert!(registry.contains("env"));
    assert!(registry.contains("line"));
    assert!(registry.contains("file"));
    assert!(registry.contains("column"));
}

#[test]
fn test_custom_proc_macro() {
    struct ConstMacro;

    impl ProcMacro for ConstMacro {
        fn name(&self) -> &str {
            "const_value"
        }

        fn kind(&self) -> ProcMacroKind {
            ProcMacroKind::FunctionLike
        }

        fn expand(&self, _input: TokenStream) -> ProcMacroResult<TokenStream> {
            let mut output = TokenStream::new();
            output.push(TokenTree::Literal(LiteralToken::Integer(42)));
            Ok(output)
        }
    }

    let mut registry = ProcMacroRegistry::new();
    registry.register(Box::new(ConstMacro));

    assert!(registry.contains("const_value"));

    let mac = registry.get("const_value").unwrap();
    let result = mac.expand(TokenStream::new()).unwrap();
    assert_eq!(result.len(), 1);
}

// ============================ 4. Async Macros ============================

#[test]
fn test_async_join_two_futures() {
    let mut expander = AsyncMacroExpander::new();

    let futures = vec![
        vec![MacroToken::Ident("task1".to_string())],
        vec![MacroToken::Ident("task2".to_string())],
    ];

    let result = expander.expand_join(&futures).unwrap();
    let output = tokens_to_string(&result);

    assert!(output.contains("join"));
    assert!(output.contains("task1"));
    assert!(output.contains("task2"));
    assert!(output.contains(".await"));
}

#[test]
fn test_async_select_two_arms() {
    let mut expander = AsyncMacroExpander::new();

    let arms = vec![
        (
            "a".to_string(),
            vec![MacroToken::Ident("future1".to_string())],
            vec![MacroToken::Ident("a".to_string())],
        ),
        (
            "b".to_string(),
            vec![MacroToken::Ident("future2".to_string())],
            vec![MacroToken::Ident("b".to_string())],
        ),
    ];

    let result = expander.expand_select(&arms).unwrap();
    let output = tokens_to_string(&result);

    assert!(output.contains("select"));
    assert!(output.contains("M")); // match keyword
    assert!(output.contains("Left"));
    assert!(output.contains("Right"));
}

#[test]
fn test_async_timeout_expansion() {
    let mut expander = AsyncMacroExpander::new();

    let duration = vec![MacroToken::Literal(MacroLiteral::Int(5000))];
    let future = vec![MacroToken::Ident("long_task".to_string())];

    let result = expander.expand_timeout(&duration, &future).unwrap();
    let output = tokens_to_string(&result);

    assert!(output.contains("select"));
    assert!(output.contains("delay"));
    assert!(output.contains("5000"));
    assert!(output.contains(".await"));
}

#[test]
fn test_async_macro_registry() {
    let mut registry = MacroRegistry::new();
    register_async_macros(&mut registry);

    assert!(registry.contains(JOIN_MACRO));
    assert!(registry.contains(SELECT_MACRO));
    assert!(registry.contains(TIMEOUT_MACRO));
}

// ============================ 5. HygienicContext ============================

#[test]
fn test_hygienic_name_generation() {
    let mut ctx = HygienicContext::new();

    let name1 = ctx.hygienize("temp");
    let name2 = ctx.hygienize("var");

    // Different variable names should produce different hygienic names
    assert_ne!(name1, name2);
    assert!(name1.contains("__macro_"));
    assert!(name2.contains("__macro_"));
    assert!(name1.contains("temp"));
    assert!(name2.contains("var"));
}

#[test]
fn test_hygienic_context_push_pop() {
    let mut ctx = HygienicContext::new();
    let initial_ctx = ctx.current_context();

    ctx.push_context();
    let pushed_ctx = ctx.current_context();
    assert_ne!(initial_ctx, pushed_ctx);

    ctx.pop_context();
    assert_eq!(ctx.current_context(), initial_ctx);
}

#[test]
fn test_hygienic_name_lookup() {
    let mut ctx = HygienicContext::new();

    let hygienic = ctx.hygienize("x");
    assert_eq!(ctx.lookup("x"), Some(&hygienic));
    assert_eq!(ctx.lookup("y"), None);
}

// ============================ 6. AST Expansion ============================

#[test]
fn test_expand_macros_module() {
    use vais_parser::parse;

    let mut registry = MacroRegistry::new();

    // Register a simple macro
    registry.register(MacroDef {
        name: Spanned::new("answer".to_string(), Span::new(0, 6)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Empty,
            template: MacroTemplate::Sequence(vec![MacroTemplateElement::Token(
                MacroToken::Literal(MacroLiteral::Int(42)),
            )]),
        }],
        is_pub: false,
    });

    let source = "F test() = answer!()";
    let module = parse(source).unwrap();

    let result = expand_macros(module, &registry);
    assert!(result.is_ok());
}

#[test]
fn test_collect_macros_from_module() {
    use vais_parser::parse;

    let source = r#"
        macro double! { ($x:expr) => { $x + $x } }
        F test() = 1
    "#;

    let module = parse(source).unwrap();
    let mut registry = MacroRegistry::new();

    collect_macros(&module, &mut registry);

    assert!(registry.contains("double"));
    assert_eq!(registry.macros_count(), 1);

    // Verify the macro definition is correct
    let def = registry.get("double").unwrap();
    assert_eq!(def.name.node, "double");
    assert_eq!(def.rules.len(), 1);
}

#[test]
fn test_expand_module_no_macros() {
    use vais_parser::parse;

    let registry = MacroRegistry::new();
    let source = "F identity(x: i64) -> i64 = x";
    let module = parse(source).unwrap();

    let result = expand_macros(module, &registry);
    assert!(result.is_ok());
}

#[test]
fn test_ast_expander_validation() {
    let registry = MacroRegistry::new();
    let mut expander = AstExpander::new(&registry);

    let module = Module {
        items: vec![],
        modules_map: Default::default(),
    };

    let result = expander.expand_module(module);
    assert!(result.is_ok());
}

// ============================ 7. Derive System ============================

#[test]
fn test_derive_registry_built_in_derives() {
    let registry = DeriveRegistry::new();

    assert!(registry.is_supported("Debug"));
    assert!(registry.is_supported("Clone"));
    assert!(registry.is_supported("PartialEq"));
    assert!(registry.is_supported("Default"));
    assert!(registry.is_supported("Hash"));
    assert!(registry.is_supported("Error"));
}

#[test]
fn test_derive_registry_unsupported() {
    let registry = DeriveRegistry::new();

    assert!(!registry.is_supported("CustomDerive"));
    assert!(!registry.is_supported("NonExistent"));
}

#[test]
fn test_process_derives_noop() {
    let mut module = Module {
        items: vec![],
        modules_map: Default::default(),
    };

    let result = process_derives(&mut module);
    assert!(result.is_ok());
}

// ============================ 8. tokens_to_string ============================

#[test]
fn test_tokens_to_string_various_types() {
    let tokens = vec![
        MacroToken::Ident("foo".to_string()),
        MacroToken::Punct('('),
        MacroToken::Literal(MacroLiteral::Int(42)),
        MacroToken::Punct(','),
        MacroToken::Literal(MacroLiteral::String("hello".to_string())),
        MacroToken::Punct(')'),
    ];

    let output = tokens_to_string(&tokens);
    assert_eq!(output, "foo(42,\"hello\")");
}

#[test]
fn test_tokens_to_string_empty() {
    let tokens: Vec<MacroToken> = vec![];
    let output = tokens_to_string(&tokens);
    assert_eq!(output, "");
}

// ============================ Additional Edge Cases ============================

#[test]
fn test_macro_expander_with_groups() {
    let mut registry = MacroRegistry::new();

    // macro paren! { ($x:expr) => { ($x) } }
    registry.register(MacroDef {
        name: Spanned::new("paren".to_string(), Span::new(0, 5)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![MacroPatternElement::MetaVar {
                name: "x".to_string(),
                kind: MetaVarKind::Expr,
            }]),
            template: MacroTemplate::Sequence(vec![MacroTemplateElement::Group {
                delimiter: Delimiter::Paren,
                content: vec![MacroTemplateElement::MetaVar("x".to_string())],
            }]),
        }],
        is_pub: false,
    });

    let expander = MacroExpander::new(&registry);
    let invoke = MacroInvoke {
        name: Spanned::new("paren".to_string(), Span::new(0, 5)),
        delimiter: Delimiter::Paren,
        tokens: vec![MacroToken::Literal(MacroLiteral::Int(7))],
    };

    let result = expander.expand(&invoke).unwrap();
    assert_eq!(tokens_to_string(&result), "(7)");
}

#[test]
fn test_async_join_single_future() {
    let mut expander = AsyncMacroExpander::new();

    let futures = vec![vec![MacroToken::Ident("single".to_string())]];

    let result = expander.expand_join(&futures).unwrap();
    let output = tokens_to_string(&result);

    // Single future should just be awaited
    assert_eq!(output, "single.await");
}

#[test]
fn test_async_join_empty_error() {
    let mut expander = AsyncMacroExpander::new();
    let futures: Vec<Vec<MacroToken>> = vec![];

    let result = expander.expand_join(&futures);
    assert!(result.is_err());
}

#[test]
fn test_async_select_too_few_arms_error() {
    let mut expander = AsyncMacroExpander::new();

    let arms = vec![(
        "a".to_string(),
        vec![MacroToken::Ident("f1".to_string())],
        vec![MacroToken::Ident("a".to_string())],
    )];

    let result = expander.expand_select(&arms);
    assert!(result.is_err());
}

#[test]
fn test_token_stream_to_source() {
    let mut stream = TokenStream::new();
    stream.push(TokenTree::Ident("print".to_string()));
    stream.push(TokenTree::Punct('('));
    stream.push(TokenTree::Literal(LiteralToken::String("test".to_string())));
    stream.push(TokenTree::Punct(')'));

    let source = stream.to_source();
    assert_eq!(source, "print ( \"test\" )");
}

#[test]
fn test_proc_macro_error_creation() {
    let err = ProcMacroError::new("test error");
    assert_eq!(err.message, "test error");
    assert!(err.span.is_none());

    let err_with_span = ProcMacroError::with_span("span error", Span::new(10, 20));
    assert_eq!(err_with_span.message, "span error");
    assert!(err_with_span.span.is_some());
}

#[test]
fn test_hygienic_context_default() {
    let ctx = HygienicContext::default();
    assert_eq!(ctx.current_context(), 0);
}

#[test]
fn test_async_join_three_futures() {
    let mut expander = AsyncMacroExpander::new();

    let futures = vec![
        vec![MacroToken::Ident("a".to_string())],
        vec![MacroToken::Ident("b".to_string())],
        vec![MacroToken::Ident("c".to_string())],
    ];

    let result = expander.expand_join(&futures).unwrap();
    let output = tokens_to_string(&result);

    // Should be nested: join(join(a, b), c).await
    assert!(output.contains("join(join(a,b),c).await"));
}
