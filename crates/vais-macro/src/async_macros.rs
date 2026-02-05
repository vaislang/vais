//! Async-specific built-in macros for Vais
//!
//! This module provides select! and join! macros for concurrent async operations.
//!
//! # Macros
//!
//! - `select!`: Runs multiple futures concurrently, returns when the first completes
//! - `join!`: Runs multiple futures concurrently, waits for all to complete
//!
//! # Example
//!
//! ```ignore
//! // select! - returns first completed future's result
//! result := select! {
//!     a <- future1() => a * 2,
//!     b <- future2() => b + 1,
//! };
//!
//! // join! - returns tuple of all results
//! (a, b, c) := join!(future1(), future2(), future3());
//! ```

use crate::{MacroError, MacroRegistry, MacroResult};
use vais_ast::*;

/// Built-in async macro names
pub const SELECT_MACRO: &str = "select";
pub const JOIN_MACRO: &str = "join";
pub const TIMEOUT_MACRO: &str = "timeout";

/// Register built-in async macros in the registry
pub fn register_async_macros(registry: &mut MacroRegistry) {
    // select! macro definition
    let select_def = create_select_macro_def();
    registry.register(select_def);

    // join! macro definition
    let join_def = create_join_macro_def();
    registry.register(join_def);

    // timeout! macro definition
    let timeout_def = create_timeout_macro_def();
    registry.register(timeout_def);
}

/// Create the select! macro definition
///
/// ```ignore
/// select! {
///     $pat <- $future => $body,
///     ...
/// }
/// ```
fn create_select_macro_def() -> MacroDef {
    // Pattern: { $($pat:pat <- $future:expr => $body:expr),+ }
    // Template: Creates SelectFuture chains

    MacroDef {
        name: Spanned::new(SELECT_MACRO.to_string(), Span::new(0, 6)),
        rules: vec![
            // Rule 1: Two arms
            MacroRule {
                pattern: MacroPattern::Sequence(vec![MacroPatternElement::Group {
                    delimiter: Delimiter::Brace,
                    content: vec![
                        MacroPatternElement::MetaVar {
                            name: "pat1".to_string(),
                            kind: MetaVarKind::Pat,
                        },
                        MacroPatternElement::Token(MacroToken::Punct('<')),
                        MacroPatternElement::Token(MacroToken::Punct('-')),
                        MacroPatternElement::MetaVar {
                            name: "future1".to_string(),
                            kind: MetaVarKind::Expr,
                        },
                        MacroPatternElement::Token(MacroToken::Punct('=')),
                        MacroPatternElement::Token(MacroToken::Punct('>')),
                        MacroPatternElement::MetaVar {
                            name: "body1".to_string(),
                            kind: MetaVarKind::Expr,
                        },
                        MacroPatternElement::Token(MacroToken::Punct(',')),
                        MacroPatternElement::MetaVar {
                            name: "pat2".to_string(),
                            kind: MetaVarKind::Pat,
                        },
                        MacroPatternElement::Token(MacroToken::Punct('<')),
                        MacroPatternElement::Token(MacroToken::Punct('-')),
                        MacroPatternElement::MetaVar {
                            name: "future2".to_string(),
                            kind: MetaVarKind::Expr,
                        },
                        MacroPatternElement::Token(MacroToken::Punct('=')),
                        MacroPatternElement::Token(MacroToken::Punct('>')),
                        MacroPatternElement::MetaVar {
                            name: "body2".to_string(),
                            kind: MetaVarKind::Expr,
                        },
                    ],
                }]),
                template: MacroTemplate::Sequence(vec![
                    // Generate: select_race($future1, $future2).await |> match { ... }
                    MacroTemplateElement::Token(MacroToken::Ident("select".to_string())),
                    MacroTemplateElement::Group {
                        delimiter: Delimiter::Paren,
                        content: vec![
                            MacroTemplateElement::MetaVar("future1".to_string()),
                            MacroTemplateElement::Token(MacroToken::Punct(',')),
                            MacroTemplateElement::MetaVar("future2".to_string()),
                        ],
                    },
                    MacroTemplateElement::Token(MacroToken::Punct('.')),
                    MacroTemplateElement::Token(MacroToken::Ident("await".to_string())),
                ]),
            },
        ],
        is_pub: true,
    }
}

/// Create the join! macro definition
///
/// ```ignore
/// join!($future1, $future2, ...)
/// ```
fn create_join_macro_def() -> MacroDef {
    // Pattern: ($($future:expr),+)
    // Template: Creates JoinFuture chain

    MacroDef {
        name: Spanned::new(JOIN_MACRO.to_string(), Span::new(0, 4)),
        rules: vec![
            // Rule 1: Two futures
            MacroRule {
                pattern: MacroPattern::Sequence(vec![
                    MacroPatternElement::MetaVar {
                        name: "f1".to_string(),
                        kind: MetaVarKind::Expr,
                    },
                    MacroPatternElement::Token(MacroToken::Punct(',')),
                    MacroPatternElement::MetaVar {
                        name: "f2".to_string(),
                        kind: MetaVarKind::Expr,
                    },
                ]),
                template: MacroTemplate::Sequence(vec![
                    MacroTemplateElement::Token(MacroToken::Ident("join".to_string())),
                    MacroTemplateElement::Group {
                        delimiter: Delimiter::Paren,
                        content: vec![
                            MacroTemplateElement::MetaVar("f1".to_string()),
                            MacroTemplateElement::Token(MacroToken::Punct(',')),
                            MacroTemplateElement::MetaVar("f2".to_string()),
                        ],
                    },
                    MacroTemplateElement::Token(MacroToken::Punct('.')),
                    MacroTemplateElement::Token(MacroToken::Ident("await".to_string())),
                ]),
            },
            // Rule 2: Three futures
            MacroRule {
                pattern: MacroPattern::Sequence(vec![
                    MacroPatternElement::MetaVar {
                        name: "f1".to_string(),
                        kind: MetaVarKind::Expr,
                    },
                    MacroPatternElement::Token(MacroToken::Punct(',')),
                    MacroPatternElement::MetaVar {
                        name: "f2".to_string(),
                        kind: MetaVarKind::Expr,
                    },
                    MacroPatternElement::Token(MacroToken::Punct(',')),
                    MacroPatternElement::MetaVar {
                        name: "f3".to_string(),
                        kind: MetaVarKind::Expr,
                    },
                ]),
                template: MacroTemplate::Sequence(vec![
                    MacroTemplateElement::Token(MacroToken::Ident("join".to_string())),
                    MacroTemplateElement::Group {
                        delimiter: Delimiter::Paren,
                        content: vec![
                            MacroTemplateElement::Token(MacroToken::Ident("join".to_string())),
                            MacroTemplateElement::Group {
                                delimiter: Delimiter::Paren,
                                content: vec![
                                    MacroTemplateElement::MetaVar("f1".to_string()),
                                    MacroTemplateElement::Token(MacroToken::Punct(',')),
                                    MacroTemplateElement::MetaVar("f2".to_string()),
                                ],
                            },
                            MacroTemplateElement::Token(MacroToken::Punct(',')),
                            MacroTemplateElement::MetaVar("f3".to_string()),
                        ],
                    },
                    MacroTemplateElement::Token(MacroToken::Punct('.')),
                    MacroTemplateElement::Token(MacroToken::Ident("await".to_string())),
                ]),
            },
        ],
        is_pub: true,
    }
}

/// Create the timeout! macro definition
///
/// ```ignore
/// timeout!(duration_ms, $future)
/// ```
fn create_timeout_macro_def() -> MacroDef {
    MacroDef {
        name: Spanned::new(TIMEOUT_MACRO.to_string(), Span::new(0, 7)),
        rules: vec![MacroRule {
            pattern: MacroPattern::Sequence(vec![
                MacroPatternElement::MetaVar {
                    name: "ms".to_string(),
                    kind: MetaVarKind::Expr,
                },
                MacroPatternElement::Token(MacroToken::Punct(',')),
                MacroPatternElement::MetaVar {
                    name: "future".to_string(),
                    kind: MetaVarKind::Expr,
                },
            ]),
            template: MacroTemplate::Sequence(vec![
                MacroTemplateElement::Token(MacroToken::Ident("select".to_string())),
                MacroTemplateElement::Group {
                    delimiter: Delimiter::Paren,
                    content: vec![
                        MacroTemplateElement::MetaVar("future".to_string()),
                        MacroTemplateElement::Token(MacroToken::Punct(',')),
                        MacroTemplateElement::Token(MacroToken::Ident("delay".to_string())),
                        MacroTemplateElement::Group {
                            delimiter: Delimiter::Paren,
                            content: vec![MacroTemplateElement::MetaVar("ms".to_string())],
                        },
                    ],
                },
                MacroTemplateElement::Token(MacroToken::Punct('.')),
                MacroTemplateElement::Token(MacroToken::Ident("await".to_string())),
            ]),
        }],
        is_pub: true,
    }
}

/// Async macro expander with specialized handling
pub struct AsyncMacroExpander {
    /// Counter for generating unique names
    counter: usize,
}

impl AsyncMacroExpander {
    /// Create a new async macro expander
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    /// Generate a unique temporary variable name
    fn fresh_name(&mut self, prefix: &str) -> String {
        let name = format!("__{}_tmp_{}", prefix, self.counter);
        self.counter += 1;
        name
    }

    /// Expand select! macro with multiple arms
    ///
    /// Transforms:
    /// ```ignore
    /// select! {
    ///     a <- future1() => expr1,
    ///     b <- future2() => expr2,
    /// }
    /// ```
    ///
    /// Into:
    /// ```ignore
    /// {
    ///     let __sel = select(future1(), future2());
    ///     let __result = __sel.await;
    ///     match __result {
    ///         Either::Left(a) => expr1,
    ///         Either::Right(b) => expr2,
    ///     }
    /// }
    /// ```
    pub fn expand_select(
        &mut self,
        arms: &[(String, Vec<MacroToken>, Vec<MacroToken>)], // (pat, future, body)
    ) -> MacroResult<Vec<MacroToken>> {
        if arms.len() < 2 {
            return Err(MacroError::InvalidFragment {
                expected: "at least 2 select arms".to_string(),
                found: format!("{} arms", arms.len()),
            });
        }

        let mut result = Vec::new();

        // Opening brace
        result.push(MacroToken::Punct('{'));

        // Let bindings for futures
        let sel_var = self.fresh_name("sel");
        let result_var = self.fresh_name("result");

        // let __sel = select(future1, future2);
        result.push(MacroToken::Ident("let".to_string()));
        result.push(MacroToken::Ident(sel_var.clone()));
        result.push(MacroToken::Punct('='));
        result.push(MacroToken::Ident("select".to_string()));
        result.push(MacroToken::Punct('('));

        // Add futures separated by commas
        for (i, (_, future_tokens, _)) in arms.iter().enumerate() {
            if i > 0 {
                result.push(MacroToken::Punct(','));
            }
            result.extend(future_tokens.clone());
        }

        result.push(MacroToken::Punct(')'));
        result.push(MacroToken::Punct(';'));

        // let __result = __sel.await;
        result.push(MacroToken::Ident("let".to_string()));
        result.push(MacroToken::Ident(result_var.clone()));
        result.push(MacroToken::Punct('='));
        result.push(MacroToken::Ident(sel_var));
        result.push(MacroToken::Punct('.'));
        result.push(MacroToken::Ident("await".to_string()));
        result.push(MacroToken::Punct(';'));

        // match __result { ... }
        result.push(MacroToken::Ident("M".to_string())); // Vais match keyword
        result.push(MacroToken::Ident(result_var));
        result.push(MacroToken::Punct('{'));

        // Match arms for Either::Left / Either::Right
        for (i, (pat, _, body_tokens)) in arms.iter().enumerate() {
            if i == 0 {
                result.push(MacroToken::Ident("Left".to_string()));
            } else {
                result.push(MacroToken::Ident("Right".to_string()));
            }
            result.push(MacroToken::Punct('('));
            result.push(MacroToken::Ident(pat.clone()));
            result.push(MacroToken::Punct(')'));
            result.push(MacroToken::Punct('='));
            result.push(MacroToken::Punct('>'));
            result.extend(body_tokens.clone());
            result.push(MacroToken::Punct(','));
        }

        result.push(MacroToken::Punct('}')); // match
        result.push(MacroToken::Punct('}')); // block

        Ok(result)
    }

    /// Expand join! macro with multiple futures
    ///
    /// Transforms:
    /// ```ignore
    /// join!(future1(), future2(), future3())
    /// ```
    ///
    /// Into:
    /// ```ignore
    /// join(join(future1(), future2()), future3()).await
    /// ```
    pub fn expand_join(&mut self, futures: &[Vec<MacroToken>]) -> MacroResult<Vec<MacroToken>> {
        if futures.is_empty() {
            return Err(MacroError::InvalidFragment {
                expected: "at least 1 future".to_string(),
                found: "0 futures".to_string(),
            });
        }

        if futures.len() == 1 {
            // Single future, just await it
            let mut result = futures[0].clone();
            result.push(MacroToken::Punct('.'));
            result.push(MacroToken::Ident("await".to_string()));
            return Ok(result);
        }

        // Build nested join calls: join(join(f1, f2), f3)...
        let mut result = futures[0].clone();

        for future_tokens in futures.iter().skip(1) {
            let mut new_result = Vec::new();
            new_result.push(MacroToken::Ident("join".to_string()));
            new_result.push(MacroToken::Punct('('));
            new_result.extend(result);
            new_result.push(MacroToken::Punct(','));
            new_result.extend(future_tokens.clone());
            new_result.push(MacroToken::Punct(')'));
            result = new_result;
        }

        // Add .await at the end
        result.push(MacroToken::Punct('.'));
        result.push(MacroToken::Ident("await".to_string()));

        Ok(result)
    }

    /// Expand timeout! macro
    ///
    /// Transforms:
    /// ```ignore
    /// timeout!(1000, future())
    /// ```
    ///
    /// Into:
    /// ```ignore
    /// select(future(), delay(1000)).await
    /// ```
    pub fn expand_timeout(
        &mut self,
        duration_tokens: &[MacroToken],
        future_tokens: &[MacroToken],
    ) -> MacroResult<Vec<MacroToken>> {
        let mut result = Vec::new();

        result.push(MacroToken::Ident("select".to_string()));
        result.push(MacroToken::Punct('('));
        result.extend(future_tokens.iter().cloned());
        result.push(MacroToken::Punct(','));
        result.push(MacroToken::Ident("delay".to_string()));
        result.push(MacroToken::Punct('('));
        result.extend(duration_tokens.iter().cloned());
        result.push(MacroToken::Punct(')'));
        result.push(MacroToken::Punct(')'));
        result.push(MacroToken::Punct('.'));
        result.push(MacroToken::Ident("await".to_string()));

        Ok(result)
    }
}

impl Default for AsyncMacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens_to_string;

    #[test]
    fn test_register_async_macros() {
        let mut registry = MacroRegistry::new();
        register_async_macros(&mut registry);

        assert!(registry.contains(SELECT_MACRO));
        assert!(registry.contains(JOIN_MACRO));
        assert!(registry.contains(TIMEOUT_MACRO));
    }

    #[test]
    fn test_async_expander_join_two() {
        let mut expander = AsyncMacroExpander::new();

        let futures = vec![
            vec![
                MacroToken::Ident("f1".to_string()),
                MacroToken::Punct('('),
                MacroToken::Punct(')'),
            ],
            vec![
                MacroToken::Ident("f2".to_string()),
                MacroToken::Punct('('),
                MacroToken::Punct(')'),
            ],
        ];

        let result = expander.expand_join(&futures).unwrap();
        let s = tokens_to_string(&result);

        assert!(s.contains("join"));
        assert!(s.contains("f1()"));
        assert!(s.contains("f2()"));
        assert!(s.contains(".await"));
    }

    #[test]
    fn test_async_expander_join_three() {
        let mut expander = AsyncMacroExpander::new();

        let futures = vec![
            vec![MacroToken::Ident("a".to_string())],
            vec![MacroToken::Ident("b".to_string())],
            vec![MacroToken::Ident("c".to_string())],
        ];

        let result = expander.expand_join(&futures).unwrap();
        let s = tokens_to_string(&result);

        // Should be nested: join(join(a, b), c).await
        assert!(s.contains("join(join(a,b),c).await"));
    }

    #[test]
    fn test_async_expander_timeout() {
        let mut expander = AsyncMacroExpander::new();

        let duration = vec![MacroToken::Literal(MacroLiteral::Int(1000))];
        let future = vec![MacroToken::Ident("my_future".to_string())];

        let result = expander.expand_timeout(&duration, &future).unwrap();
        let s = tokens_to_string(&result);

        assert!(s.contains("select(my_future,delay(1000)).await"));
    }

    #[test]
    fn test_async_expander_select() {
        let mut expander = AsyncMacroExpander::new();

        let arms = vec![
            (
                "a".to_string(),
                vec![MacroToken::Ident("f1".to_string())],
                vec![MacroToken::Ident("a".to_string())],
            ),
            (
                "b".to_string(),
                vec![MacroToken::Ident("f2".to_string())],
                vec![MacroToken::Ident("b".to_string())],
            ),
        ];

        let result = expander.expand_select(&arms).unwrap();
        let s = tokens_to_string(&result);

        assert!(s.contains("select"));
        assert!(s.contains("M")); // match keyword
        assert!(s.contains("Left"));
        assert!(s.contains("Right"));
    }

    #[test]
    fn test_join_single_future() {
        let mut expander = AsyncMacroExpander::new();

        let futures = vec![vec![MacroToken::Ident("single".to_string())]];

        let result = expander.expand_join(&futures).unwrap();
        let s = tokens_to_string(&result);

        // Single future should just be awaited
        assert_eq!(s, "single.await");
    }

    #[test]
    fn test_join_empty_error() {
        let mut expander = AsyncMacroExpander::new();
        let futures: Vec<Vec<MacroToken>> = vec![];

        let result = expander.expand_join(&futures);
        assert!(result.is_err());
    }

    #[test]
    fn test_select_too_few_arms() {
        let mut expander = AsyncMacroExpander::new();

        let arms = vec![(
            "a".to_string(),
            vec![MacroToken::Ident("f1".to_string())],
            vec![MacroToken::Ident("a".to_string())],
        )];

        let result = expander.expand_select(&arms);
        assert!(result.is_err());
    }
}
