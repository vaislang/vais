//! Tests for compiler optimizations (caching, memoization)
//!
//! These tests verify that the optimization features work correctly
//! and produce the same results as without caching.

use vais_ast::{Expr, Literal, MatchArm, Pattern, Span, Spanned};
use vais_parser::parse;
use vais_types::{ExhaustivenessChecker, ResolvedType, TypeChecker};

#[test]
fn test_type_substitution_cache_correctness() {
    let code = r#"
        S Box<T> { value: i64 }

        F create_box<T>() -> Box<T> = Box { value: 42 }

        F main() -> i64 = 0
    "#;

    let ast = parse(code).expect("Parse failed");

    // First check should succeed
    let mut checker1 = TypeChecker::new();
    let result1 = checker1.check_module(&ast);
    assert!(result1.is_ok(), "First type check failed: {:?}", result1);

    // Second check with new checker should also succeed (tests cache correctness)
    let mut checker2 = TypeChecker::new();
    let result2 = checker2.check_module(&ast);
    assert!(result2.is_ok(), "Second type check failed: {:?}", result2);
}

#[test]
fn test_exhaustiveness_cache_correctness() {
    let mut checker = ExhaustivenessChecker::new();

    // Test bool exhaustiveness
    let bool_arms = vec![
        MatchArm {
            pattern: Spanned::new(Pattern::Literal(Literal::Bool(true)), Span::default()),
            guard: None,
            body: Box::new(Spanned::new(Expr::Int(1), Span::default())),
        },
        MatchArm {
            pattern: Spanned::new(Pattern::Literal(Literal::Bool(false)), Span::default()),
            guard: None,
            body: Box::new(Spanned::new(Expr::Int(0), Span::default())),
        },
    ];

    // First check
    let result1 = checker.check_match(&ResolvedType::Bool, &bool_arms);
    assert!(result1.is_exhaustive, "Bool match should be exhaustive");
    assert!(result1.missing_patterns.is_empty());

    // Cached check should return same result
    let result2 = checker.check_match(&ResolvedType::Bool, &bool_arms);
    assert!(
        result2.is_exhaustive,
        "Cached bool match should be exhaustive"
    );
    assert_eq!(result1.missing_patterns, result2.missing_patterns);
    assert_eq!(result1.unreachable_arms, result2.unreachable_arms);
}

#[test]
fn test_exhaustiveness_cache_with_non_exhaustive() {
    let mut checker = ExhaustivenessChecker::new();

    // Incomplete bool match
    let incomplete_arms = vec![MatchArm {
        pattern: Spanned::new(Pattern::Literal(Literal::Bool(true)), Span::default()),
        guard: None,
        body: Box::new(Spanned::new(Expr::Int(1), Span::default())),
    }];

    // First check
    let result1 = checker.check_match(&ResolvedType::Bool, &incomplete_arms);
    assert!(
        !result1.is_exhaustive,
        "Incomplete match should not be exhaustive"
    );
    assert!(!result1.missing_patterns.is_empty());

    // Cached check should return same non-exhaustive result
    let result2 = checker.check_match(&ResolvedType::Bool, &incomplete_arms);
    assert!(
        !result2.is_exhaustive,
        "Cached incomplete match should not be exhaustive"
    );
    assert_eq!(result1.missing_patterns, result2.missing_patterns);
}

#[test]
fn test_exhaustiveness_cache_invalidation() {
    let mut checker = ExhaustivenessChecker::new();

    let arms = vec![MatchArm {
        pattern: Spanned::new(Pattern::Wildcard, Span::default()),
        guard: None,
        body: Box::new(Spanned::new(Expr::Int(1), Span::default())),
    }];

    // Build cache
    let result1 = checker.check_match(&ResolvedType::I64, &arms);
    assert!(result1.is_exhaustive);

    // Clear cache
    checker.clear_cache();

    // Should still work after cache clear
    let result2 = checker.check_match(&ResolvedType::I64, &arms);
    assert!(result2.is_exhaustive);
    assert_eq!(result1.is_exhaustive, result2.is_exhaustive);
}

#[test]
fn test_complex_generic_substitution() {
    let code = r#"
        S Option<T> { is_some: bool, value: i64 }
        S Vec<T> { ptr: i64, len: i64, cap: i64 }

        F main() -> i64 = 0
    "#;

    let ast = parse(code).expect("Parse failed");
    let mut checker = TypeChecker::new();

    // Should handle nested generics correctly with caching
    let result = checker.check_module(&ast);
    assert!(
        result.is_ok(),
        "Complex generic substitution failed: {:?}",
        result
    );
}

#[test]
fn test_pattern_matching_with_generics() {
    let code = r#"
        E Option<T> { Some(T), None }

        F unwrap_or<T>(opt: Option<T>, default: T) -> T = M opt {
            Some(val) => val,
            None => default
        }

        F main() -> i64 = 0
    "#;

    let ast = parse(code).expect("Parse failed");
    let mut checker = TypeChecker::new();

    let result = checker.check_module(&ast);
    assert!(
        result.is_ok(),
        "Pattern matching with generics failed: {:?}",
        result
    );
}

#[test]
fn test_multiple_instantiations_same_type() {
    let code = r#"
        S Box<T> { value: T }

        F create<T>(val: T) -> Box<T> = Box { value: val }

        F main() -> i64 = 0
    "#;

    let ast = parse(code).expect("Parse failed");
    let mut checker = TypeChecker::new();

    // Multiple calls with same type should be efficiently cached
    let result = checker.check_module(&ast);
    assert!(
        result.is_ok(),
        "Multiple instantiations failed: {:?}",
        result
    );
}

#[test]
fn test_exhaustiveness_with_integer_ranges() {
    let mut checker = ExhaustivenessChecker::new();

    let range_arms = vec![
        MatchArm {
            pattern: Spanned::new(
                Pattern::Range {
                    start: Some(Box::new(Spanned::new(
                        Pattern::Literal(Literal::Int(0)),
                        Span::default(),
                    ))),
                    end: Some(Box::new(Spanned::new(
                        Pattern::Literal(Literal::Int(10)),
                        Span::default(),
                    ))),
                    inclusive: true,
                },
                Span::default(),
            ),
            guard: None,
            body: Box::new(Spanned::new(Expr::Int(1), Span::default())),
        },
        MatchArm {
            pattern: Spanned::new(Pattern::Wildcard, Span::default()),
            guard: None,
            body: Box::new(Spanned::new(Expr::Int(0), Span::default())),
        },
    ];

    let result = checker.check_match(&ResolvedType::I64, &range_arms);
    assert!(
        result.is_exhaustive,
        "Range pattern with wildcard should be exhaustive"
    );

    // Test cache with same patterns
    let result2 = checker.check_match(&ResolvedType::I64, &range_arms);
    assert_eq!(result.is_exhaustive, result2.is_exhaustive);
}

#[test]
fn test_cache_performance_consistency() {
    // Verify that results are consistent regardless of cache state
    let code = r#"
        F add(x: i64, y: i64) -> i64 = x + y
        F main() -> i64 = add(10, 20)
    "#;

    let ast = parse(code).expect("Parse failed");

    // Check multiple times with fresh checkers
    for _ in 0..5 {
        let mut checker = TypeChecker::new();
        let result = checker.check_module(&ast);
        assert!(result.is_ok(), "Consistency check failed: {:?}", result);
    }
}
