//! Performance benchmarks for compiler optimizations
//!
//! Measures the impact of caching and memoization on type checking,
//! symbol resolution, and pattern matching.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;

use vais_parser::parse;
use vais_types::{TypeChecker, ExhaustivenessChecker, ResolvedType};
use vais_ast::{MatchArm, Pattern, Literal, Spanned, Span, Expr};

/// Load a fixture file
fn load_fixture(name: &str) -> String {
    let relative_path = format!("benches/fixtures/{}.vais", name);
    if let Ok(content) = fs::read_to_string(&relative_path) {
        return content;
    }

    let bench_relative = format!("fixtures/{}.vais", name);
    if let Ok(content) = fs::read_to_string(&bench_relative) {
        return content;
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = format!("{}/fixtures/{}.vais", manifest_dir, name);
        if let Ok(content) = fs::read_to_string(&manifest_path) {
            return content;
        }
    }

    panic!("Failed to load fixture: {}", name)
}

/// Benchmark: Type substitution with memoization
/// Tests the performance improvement from caching substitute_generics calls
fn bench_type_substitution(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_substitution");

    // Generate code with heavy generic usage
    let generic_code = r#"
        S Vec<T> { ptr: i64, len: i64, cap: i64 }
        S HashMap<K,V> { buckets: i64, size: i64 }
        S Option<T> { is_some: bool, value: T }

        F map<T,U>(vec: Vec<T>, f: fn(T)->U) -> Vec<U> = Vec { ptr: 0, len: 0, cap: 0 }
        F filter<T>(vec: Vec<T>, pred: fn(T)->bool) -> Vec<T> = vec
        F fold<T,U>(vec: Vec<T>, init: U, f: fn(U,T)->U) -> U = init

        F main() -> i64 = {
            L i: 0..100 {
                L j: 0..50 {
                    L k: 0..25 {
                        print_i64(i + j + k)
                    }
                }
            }
            0
        }
    "#;

    let ast = parse(generic_code).expect("Parse failed");

    group.bench_function("type_check_with_generics", |b| {
        b.iter(|| {
            let mut checker = TypeChecker::new();
            checker.check_module(black_box(&ast))
        })
    });

    // Benchmark repeated type checking (cache should help)
    group.bench_function("repeated_type_check", |b| {
        b.iter(|| {
            let mut checker = TypeChecker::new();
            // Type check multiple times (simulating IDE-like repeated checks)
            for _ in 0..10 {
                let _ = checker.check_module(black_box(&ast));
            }
        })
    });

    group.finish();
}

/// Benchmark: Exhaustiveness checking with cache
/// Tests pattern matching exhaustiveness performance improvements
fn bench_exhaustiveness_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("exhaustiveness_checking");

    // Create match arms for bool type
    let make_bool_arms = || {
        vec![
            MatchArm {
                pattern: Spanned::new(
                    Pattern::Literal(Literal::Bool(true)),
                    Span::default()
                ),
                guard: None,
                body: Box::new(Spanned::new(Expr::Int(1), Span::default())),
            },
            MatchArm {
                pattern: Spanned::new(
                    Pattern::Literal(Literal::Bool(false)),
                    Span::default()
                ),
                guard: None,
                body: Box::new(Spanned::new(Expr::Int(0), Span::default())),
            },
        ]
    };

    let arms = make_bool_arms();

    // Benchmark without cache (first call)
    group.bench_function("check_first_time", |b| {
        b.iter(|| {
            let mut checker = ExhaustivenessChecker::new();
            checker.check_match(black_box(&ResolvedType::Bool), black_box(&arms))
        })
    });

    // Benchmark with cache (repeated calls)
    group.bench_function("check_cached", |b| {
        let mut checker = ExhaustivenessChecker::new();
        b.iter(|| {
            checker.check_match(black_box(&ResolvedType::Bool), black_box(&arms))
        })
    });

    // Benchmark many different patterns to test cache effectiveness
    group.bench_function("check_varied_patterns", |b| {
        let mut checker = ExhaustivenessChecker::new();
        b.iter(|| {
            for i in 0..100 {
                let val = i % 2 == 0;
                let arms = vec![
                    MatchArm {
                        pattern: Spanned::new(
                            Pattern::Literal(Literal::Bool(val)),
                            Span::default()
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
                checker.check_match(&ResolvedType::Bool, &arms);
            }
        })
    });

    group.finish();
}

/// Benchmark: Type checking with complex patterns
fn bench_pattern_matching_heavy_code(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");

    let pattern_heavy_code = r#"
        E Option<T> { Some(T), None }
        E Result<T,E> { Ok(T), Err(E) }

        F process(opt: Option<i64>) -> i64 = M opt {
            Some(x) => M x {
                0 => 0,
                1 => 1,
                2 => 2,
                _ => x * 2
            },
            None => -1
        }

        F handle_result(res: Result<i64, i64>) -> i64 = M res {
            Ok(x) => x,
            Err(e) => e * -1
        }

        F main() -> i64 = {
            L i: 0..50 {
                L j: 0..50 {
                    process(Some(i + j))
                }
            }
            0
        }
    "#;

    let ast = parse(pattern_heavy_code).expect("Parse failed");

    group.bench_function("type_check_patterns", |b| {
        b.iter(|| {
            let mut checker = TypeChecker::new();
            checker.check_module(black_box(&ast))
        })
    });

    // Repeated checks (cache should help significantly)
    group.bench_function("repeated_pattern_checks", |b| {
        b.iter(|| {
            let mut checker = TypeChecker::new();
            for _ in 0..5 {
                let _ = checker.check_module(black_box(&ast));
            }
        })
    });

    group.finish();
}

/// Benchmark: Cache clearing overhead
fn bench_cache_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_management");

    let simple_code = r#"
        F add(x: i64, y: i64) -> i64 = x + y
        F main() -> i64 = add(1, 2)
    "#;

    let ast = parse(simple_code).expect("Parse failed");

    // Measure overhead of cache operations
    group.bench_function("with_cache_clear", |b| {
        b.iter(|| {
            let mut checker = TypeChecker::new();
            checker.check_module(black_box(&ast)).unwrap();
            // In practice, caches would be cleared on significant changes
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_type_substitution,
    bench_exhaustiveness_checking,
    bench_pattern_matching_heavy_code,
    bench_cache_management,
);

criterion_main!(benches);
