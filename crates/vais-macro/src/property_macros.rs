//! Property-Based Testing Macros for Vais
//!
//! Provides declarative macros for property-based testing:
//! - `forall!`: Run a test function on random inputs
//! - `prop!`: Define a property test that integrates with TestSuite
//! - `check!`: Quick property check with default settings
//!
//! # Example
//!
//! ```ignore
//! // Using forall!
//! forall!(Generator.i64_range(0, 100), |x| {
//!     assert(x >= 0 && x <= 100)
//! })
//!
//! // Using `#[prop]` attribute
//! #[prop(cases=100)]
//! F prop_addition_commutative(x: i64, y: i64) {
//!     assert_eq(x + y, y + x)
//! }
//! ```

use vais_ast::*;

/// Built-in property test macros
pub const FORALL_MACRO: &str = r#"
macro forall! {
    ($gen:expr, $test:expr) => {
        forall($gen, $test, 100)
    }
    ($gen:expr, $test:expr, $count:expr) => {
        forall($gen, $test, $count)
    }
}
"#;

pub const CHECK_MACRO: &str = r#"
macro check! {
    ($name:expr, $gen:expr, $test:expr) => {
        prop_check($name, $test, $gen)
    }
}
"#;

pub const ASSERT_PROP_MACRO: &str = r#"
macro assert_prop! {
    ($name:expr, $gen:expr, $test:expr) => {
        prop_assert($name, $test, $gen)
    }
}
"#;

/// Register property-based testing macros with the macro registry
pub fn register_property_macros(registry: &mut crate::MacroRegistry) {
    // forall! macro - basic property testing
    let forall_def = MacroDef {
        name: Spanned::new("forall".to_string(), Span::new(0, 6)),
        rules: vec![
            // forall!(gen, test) - uses default 100 tests
            MacroRule {
                pattern: MacroPattern::Sequence(vec![
                    MacroPatternElement::MetaVar {
                        name: "gen".to_string(),
                        kind: MetaVarKind::Expr,
                    },
                    MacroPatternElement::Token(MacroToken::Punct(',')),
                    MacroPatternElement::MetaVar {
                        name: "test".to_string(),
                        kind: MetaVarKind::Expr,
                    },
                ]),
                template: MacroTemplate::Sequence(vec![
                    MacroTemplateElement::Token(MacroToken::Ident("forall".to_string())),
                    MacroTemplateElement::Group {
                        delimiter: Delimiter::Paren,
                        content: vec![
                            MacroTemplateElement::MetaVar("gen".to_string()),
                            MacroTemplateElement::Token(MacroToken::Punct(',')),
                            MacroTemplateElement::MetaVar("test".to_string()),
                            MacroTemplateElement::Token(MacroToken::Punct(',')),
                            MacroTemplateElement::Token(MacroToken::Literal(MacroLiteral::Int(
                                100,
                            ))),
                        ],
                    },
                ]),
            },
            // forall!(gen, test, count) - custom test count
            MacroRule {
                pattern: MacroPattern::Sequence(vec![
                    MacroPatternElement::MetaVar {
                        name: "gen".to_string(),
                        kind: MetaVarKind::Expr,
                    },
                    MacroPatternElement::Token(MacroToken::Punct(',')),
                    MacroPatternElement::MetaVar {
                        name: "test".to_string(),
                        kind: MetaVarKind::Expr,
                    },
                    MacroPatternElement::Token(MacroToken::Punct(',')),
                    MacroPatternElement::MetaVar {
                        name: "count".to_string(),
                        kind: MetaVarKind::Expr,
                    },
                ]),
                template: MacroTemplate::Sequence(vec![
                    MacroTemplateElement::Token(MacroToken::Ident("forall".to_string())),
                    MacroTemplateElement::Group {
                        delimiter: Delimiter::Paren,
                        content: vec![
                            MacroTemplateElement::MetaVar("gen".to_string()),
                            MacroTemplateElement::Token(MacroToken::Punct(',')),
                            MacroTemplateElement::MetaVar("test".to_string()),
                            MacroTemplateElement::Token(MacroToken::Punct(',')),
                            MacroTemplateElement::MetaVar("count".to_string()),
                        ],
                    },
                ]),
            },
        ],
        is_pub: true,
    };
    registry.register(forall_def);

    // check! macro - quick property check
    let check_def = MacroDef {
        name: Spanned::new("check".to_string(), Span::new(0, 5)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![
                MacroPatternElement::MetaVar {
                    name: "name".to_string(),
                    kind: MetaVarKind::Expr,
                },
                MacroPatternElement::Token(MacroToken::Punct(',')),
                MacroPatternElement::MetaVar {
                    name: "gen".to_string(),
                    kind: MetaVarKind::Expr,
                },
                MacroPatternElement::Token(MacroToken::Punct(',')),
                MacroPatternElement::MetaVar {
                    name: "test".to_string(),
                    kind: MetaVarKind::Expr,
                },
            ]),
            template: MacroTemplate::Sequence(vec![
                MacroTemplateElement::Token(MacroToken::Ident("prop_check".to_string())),
                MacroTemplateElement::Group {
                    delimiter: Delimiter::Paren,
                    content: vec![
                        MacroTemplateElement::MetaVar("name".to_string()),
                        MacroTemplateElement::Token(MacroToken::Punct(',')),
                        MacroTemplateElement::MetaVar("test".to_string()),
                        MacroTemplateElement::Token(MacroToken::Punct(',')),
                        MacroTemplateElement::MetaVar("gen".to_string()),
                    ],
                },
            ]),
        }],
        is_pub: true,
    };
    registry.register(check_def);

    // assert_prop! macro - property assertion
    let assert_prop_def = MacroDef {
        name: Spanned::new("assert_prop".to_string(), Span::new(0, 11)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![
                MacroPatternElement::MetaVar {
                    name: "name".to_string(),
                    kind: MetaVarKind::Expr,
                },
                MacroPatternElement::Token(MacroToken::Punct(',')),
                MacroPatternElement::MetaVar {
                    name: "gen".to_string(),
                    kind: MetaVarKind::Expr,
                },
                MacroPatternElement::Token(MacroToken::Punct(',')),
                MacroPatternElement::MetaVar {
                    name: "test".to_string(),
                    kind: MetaVarKind::Expr,
                },
            ]),
            template: MacroTemplate::Sequence(vec![
                MacroTemplateElement::Token(MacroToken::Ident("prop_assert".to_string())),
                MacroTemplateElement::Group {
                    delimiter: Delimiter::Paren,
                    content: vec![
                        MacroTemplateElement::MetaVar("name".to_string()),
                        MacroTemplateElement::Token(MacroToken::Punct(',')),
                        MacroTemplateElement::MetaVar("test".to_string()),
                        MacroTemplateElement::Token(MacroToken::Punct(',')),
                        MacroTemplateElement::MetaVar("gen".to_string()),
                    ],
                },
            ]),
        }],
        is_pub: true,
    };
    registry.register(assert_prop_def);

    // quickcheck! macro - QuickCheck style shorthand
    let quickcheck_def = MacroDef {
        name: Spanned::new("quickcheck".to_string(), Span::new(0, 10)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![MacroPatternElement::MetaVar {
                name: "test".to_string(),
                kind: MetaVarKind::Expr,
            }]),
            template: MacroTemplate::Sequence(vec![
                MacroTemplateElement::Token(MacroToken::Ident("prop_check".to_string())),
                MacroTemplateElement::Group {
                    delimiter: Delimiter::Paren,
                    content: vec![
                        MacroTemplateElement::Token(MacroToken::Literal(MacroLiteral::String(
                            "quickcheck".to_string(),
                        ))),
                        MacroTemplateElement::Token(MacroToken::Punct(',')),
                        MacroTemplateElement::MetaVar("test".to_string()),
                        MacroTemplateElement::Token(MacroToken::Punct(',')),
                        MacroTemplateElement::Token(MacroToken::Ident("Generator".to_string())),
                        MacroTemplateElement::Token(MacroToken::Punct('.')),
                        MacroTemplateElement::Token(MacroToken::Ident("i64_any".to_string())),
                        MacroTemplateElement::Group {
                            delimiter: Delimiter::Paren,
                            content: vec![],
                        },
                    ],
                },
            ]),
        }],
        is_pub: true,
    };
    registry.register(quickcheck_def);
}

/// Process `#[prop]` attribute on functions
/// Transforms a function definition into a property test
pub fn process_prop_attribute(func: &Function, attr_args: &[String]) -> Option<Function> {
    // Parse attribute arguments (e.g., cases=100, seed=42)
    let mut num_cases = 100i64;
    let mut seed = 42i64;

    for arg in attr_args {
        if let Some(value) = arg.strip_prefix("cases=") {
            if let Ok(n) = value.parse() {
                num_cases = n;
            }
        } else if let Some(value) = arg.strip_prefix("seed=") {
            if let Ok(s) = value.parse() {
                seed = s;
            }
        }
    }

    // Generate wrapper function that runs the property test
    // This would need more complex AST manipulation
    // For now, we just mark the function for the test runner

    // The actual transformation would create:
    // F prop_<name>() -> PropertyResult {
    //     gen := infer_generator_from_params(...)
    //     prop := Property.new("<name>", @<name>, gen)
    //     prop.with_tests(num_cases).with_seed(seed).check()
    // }

    // Suppress unused variable warnings
    let _ = (num_cases, seed, func);

    // Return None for now - the actual implementation would require
    // more complex AST construction. The `#[prop]` attribute is recognized
    // by the test runner when scanning for property tests.
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MacroExpander, MacroRegistry};

    #[test]
    fn test_register_property_macros() {
        let mut registry = MacroRegistry::new();
        register_property_macros(&mut registry);

        assert!(registry.contains("forall"));
        assert!(registry.contains("check"));
        assert!(registry.contains("assert_prop"));
        assert!(registry.contains("quickcheck"));
    }

    #[test]
    fn test_forall_expansion() {
        let mut registry = MacroRegistry::new();
        register_property_macros(&mut registry);

        let expander = MacroExpander::new(&registry);

        // forall!(gen, test)
        let invoke = MacroInvoke {
            name: Spanned::new("forall".to_string(), Span::new(0, 6)),
            delimiter: Delimiter::Paren,
            tokens: vec![
                MacroToken::Ident("Generator".to_string()),
                MacroToken::Punct('.'),
                MacroToken::Ident("i64_any".to_string()),
                MacroToken::Group(Delimiter::Paren, vec![]),
                MacroToken::Punct(','),
                MacroToken::Ident("my_test".to_string()),
            ],
        };

        let result = expander.expand(&invoke);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let output = crate::tokens_to_string(&tokens);
        assert!(output.contains("forall"));
        assert!(output.contains("100")); // default count
    }

    #[test]
    fn test_forall_with_count_expansion() {
        let mut registry = MacroRegistry::new();
        register_property_macros(&mut registry);

        let expander = MacroExpander::new(&registry);

        // forall!(gen, test, 500)
        let invoke = MacroInvoke {
            name: Spanned::new("forall".to_string(), Span::new(0, 6)),
            delimiter: Delimiter::Paren,
            tokens: vec![
                MacroToken::Ident("Generator".to_string()),
                MacroToken::Punct('.'),
                MacroToken::Ident("i64_any".to_string()),
                MacroToken::Group(Delimiter::Paren, vec![]),
                MacroToken::Punct(','),
                MacroToken::Ident("my_test".to_string()),
                MacroToken::Punct(','),
                MacroToken::Literal(MacroLiteral::Int(500)),
            ],
        };

        let result = expander.expand(&invoke);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let output = crate::tokens_to_string(&tokens);
        assert!(output.contains("forall"));
        assert!(output.contains("500")); // custom count
    }
}
