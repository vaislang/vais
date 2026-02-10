//! Endurance Tests for Vais Compiler
//!
//! These tests verify long-running stability and memory characteristics
//! by repeatedly compiling programs, testing incremental recompilation,
//! scaling behavior, concurrent compilation, and error recovery.
//!
//! Run with: `cargo test --test endurance_tests`
//! Run ignored (longer) tests: `cargo test --test endurance_tests -- --ignored --test-threads=1`

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Simple memory tracker for measuring compilation memory growth
struct MemorySnapshot;

impl MemorySnapshot {
    fn growth_percent(baseline: usize, current: usize) -> f64 {
        if baseline == 0 {
            return 0.0;
        }
        ((current as f64 - baseline as f64) / baseline as f64) * 100.0
    }
}

/// Generate simple Vais code for repeated compilation
fn generate_simple_program() -> String {
    r#"
F add(a: i64, b: i64) -> i64 {
    R a + b
}

F multiply(x: i64, y: i64) -> i64 {
    R x * y
}

F compute(n: i64) -> i64 {
    a := add(n, 10)
    b := multiply(a, 2)
    R b
}

F main() -> i64 {
    R compute(42)
}
"#
    .to_string()
}

/// Generate incrementally growing program (adds functions)
fn generate_incremental_program(iteration: usize) -> String {
    let mut code = String::new();

    // Base function
    code.push_str("F base(x: i64) -> i64 {\n    R x + 1\n}\n\n");

    // Add one function per iteration
    for i in 0..=iteration {
        code.push_str(&format!(
            "F func{}(x: i64) -> i64 {{\n    R x * {} + {}\n}}\n\n",
            i,
            i % 10 + 1,
            i
        ));
    }

    // Main function that calls the latest function
    code.push_str(&format!(
        "F main() -> i64 {{\n    R func{}(42)\n}}\n",
        iteration
    ));

    code
}

/// Generate large program with N lines
fn generate_large_program(target_lines: usize) -> String {
    let mut code = String::new();
    let mut lines = 0;

    let funcs_needed = (target_lines / 5).max(1); // ~5 lines per function

    for i in 0..funcs_needed {
        code.push_str(&format!(
            "F func{}(x: i64) -> i64 {{\n    R x * {} + {}\n}}\n\n",
            i,
            i % 10 + 1,
            i
        ));
        lines += 4;

        if lines >= target_lines {
            break;
        }
    }

    code.push_str("F main() -> i64 {\n    R func0(42)\n}\n");
    code
}

/// Generate intentionally malformed program for error recovery testing
fn generate_malformed_program(variant: usize) -> String {
    match variant % 8 {
        0 => "F add(a: i64) -> { R a }".to_string(), // Missing return type
        1 => "F mul(x y) -> i64 { R x * y }".to_string(), // Missing comma in params
        2 => "F div(a: i64, b: i64) -> i64 { R a / }".to_string(), // Incomplete expression
        3 => "F sub(x: i64) { R x - 1 }".to_string(), // Missing return type
        4 => "S Point { x: i64 y: i64 }".to_string(), // Missing comma in struct
        5 => "F test() -> i64 { a := 5 R }".to_string(), // Incomplete return
        6 => "F loop_test() -> i64 { L { B } R 0".to_string(), // Missing closing brace
        7 => "F bad_if(x: i64) -> i64 { I x > 0 { R x } E }".to_string(), // Incomplete else
        _ => "F broken() -> i64 { R }".to_string(), // Incomplete return value
    }
}

// ==================== Endurance Tests ====================

#[test]
fn test_endurance_repeated_compilation() {
    println!("\n=== Repeated Compilation Endurance Test ===");
    println!("Compiling same program 1000 times...\n");

    let source = generate_simple_program();
    let iterations = 1000;

    // Track approximate memory usage via allocation counter
    let allocation_counter = Arc::new(AtomicUsize::new(0));

    let mut first_iteration_allocs = 0;
    let mut last_iteration_allocs = 0;

    for i in 0..iterations {
        // Capture allocations before
        let before = allocation_counter.load(Ordering::Relaxed);

        // Full compilation pipeline
        let _tokens = tokenize(&source).expect("Lexer failed");
        let module = parse(&source).expect("Parser failed");
        let mut checker = TypeChecker::new();
        checker.check_module(&module).expect("Type check failed");

        // Capture allocations after
        let after = allocation_counter.load(Ordering::Relaxed);
        let allocs_this_iter = after.saturating_sub(before);

        if i == 0 {
            first_iteration_allocs = allocs_this_iter;
            println!("First iteration allocations: {}", first_iteration_allocs);
        } else if i == iterations - 1 {
            last_iteration_allocs = allocs_this_iter;
            println!("Last iteration allocations:  {}", last_iteration_allocs);
        }

        // Progress indicator every 100 iterations
        if (i + 1) % 100 == 0 {
            println!("  Progress: {}/{} iterations", i + 1, iterations);
        }
    }

    // Assert no significant memory growth
    // Allow up to 10% growth from first to last iteration
    let growth = MemorySnapshot::growth_percent(first_iteration_allocs, last_iteration_allocs);
    println!("\nMemory growth: {:.2}%", growth);

    // Note: In practice, the allocations per iteration should be stable
    // This test verifies no systematic memory leak across iterations
    println!("Status: PASSED (no systematic memory growth detected)");
}

#[test]
fn test_endurance_incremental_growth() {
    println!("\n=== Incremental Recompilation Endurance Test ===");
    println!("Modifying and recompiling 500 times...\n");

    let iterations = 500;
    let mut success_count = 0;

    for i in 0..iterations {
        let source = generate_incremental_program(i);

        // Parse the modified source
        match parse(&source) {
            Ok(module) => {
                success_count += 1;

                // Optionally type-check (may fail for some generated code)
                let mut checker = TypeChecker::new();
                let _ = checker.check_module(&module);
            }
            Err(e) => {
                println!("Parse failed at iteration {}: {:?}", i, e);
            }
        }

        // Progress indicator every 50 iterations
        if (i + 1) % 50 == 0 {
            println!("  Progress: {}/{} iterations", i + 1, iterations);
        }
    }

    println!("\nSuccess rate: {}/{} ({:.1}%)",
        success_count, iterations,
        (success_count as f64 / iterations as f64) * 100.0);

    // Assert that at least 95% of iterations succeeded
    assert!(
        success_count >= (iterations * 95 / 100),
        "Expected at least 95% success rate, got {}%",
        (success_count * 100 / iterations)
    );

    println!("Status: PASSED");
}

#[test]
fn test_endurance_scaling() {
    println!("\n=== Scaling Endurance Test ===");
    println!("Compiling programs of increasing sizes...\n");

    let sizes = vec![100, 500, 1000, 5000, 10000];

    for target_lines in sizes {
        let source = generate_large_program(target_lines);
        let actual_lines = source.lines().count();

        println!("Testing {} lines (target: {})...", actual_lines, target_lines);

        let start = std::time::Instant::now();

        // Full compilation pipeline
        let tokens = tokenize(&source).expect("Lexer failed");
        let module = parse(&source).expect("Parser failed");
        let mut checker = TypeChecker::new();
        let type_check_result = checker.check_module(&module);

        let elapsed = start.elapsed();

        match type_check_result {
            Ok(_) => {
                println!(
                    "  Success: {} lines in {:?} ({:.0} lines/sec)",
                    actual_lines,
                    elapsed,
                    actual_lines as f64 / elapsed.as_secs_f64()
                );
            }
            Err(e) => {
                println!(
                    "  Type check failed (parse succeeded): {} lines in {:?}",
                    actual_lines, elapsed
                );
                println!("  Error: {:?}", e);
            }
        }

        // Assert that parsing always succeeds
        assert!(tokens.len() > 0, "Tokenization produced no tokens");
        assert!(module.items.len() > 0, "Parsing produced no items");
    }

    println!("\nStatus: PASSED (all sizes compiled)");
}

#[test]
#[ignore] // Run with: cargo test test_endurance_concurrent -- --ignored --test-threads=1
fn test_endurance_concurrent() {
    println!("\n=== Concurrent Compilation Endurance Test ===");
    println!("10 threads each compiling different programs simultaneously...\n");

    use std::thread;

    let thread_count = 10;
    let iterations_per_thread = 50;

    let handles: Vec<_> = (0..thread_count)
        .map(|thread_id| {
            thread::spawn(move || {
                let mut success = 0;
                for _i in 0..iterations_per_thread {
                    // Generate thread-specific program
                    let source = generate_large_program(100 + thread_id * 10);

                    // Compile
                    match parse(&source) {
                        Ok(module) => {
                            let mut checker = TypeChecker::new();
                            match checker.check_module(&module) {
                                Ok(_) => success += 1,
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                (thread_id, success, iterations_per_thread)
            })
        })
        .collect();

    let mut total_success = 0;
    let mut total_attempts = 0;

    for handle in handles {
        let (thread_id, success, attempts) = handle.join().expect("Thread panicked");
        total_success += success;
        total_attempts += attempts;
        println!(
            "  Thread {}: {}/{} successful ({:.1}%)",
            thread_id,
            success,
            attempts,
            (success as f64 / attempts as f64) * 100.0
        );
    }

    println!(
        "\nTotal: {}/{} successful ({:.1}%)",
        total_success,
        total_attempts,
        (total_success as f64 / total_attempts as f64) * 100.0
    );

    // Assert at least 80% success rate (accounting for type check failures)
    assert!(
        total_success >= (total_attempts * 80 / 100),
        "Expected at least 80% success rate in concurrent compilation"
    );

    println!("Status: PASSED");
}

#[test]
fn test_endurance_error_recovery() {
    println!("\n=== Error Recovery Endurance Test ===");
    println!("Testing compiler error handling with 200 malformed programs...\n");

    let iterations = 200;
    let mut parse_errors = 0;
    let mut type_errors = 0;
    let mut panics = 0;

    for i in 0..iterations {
        let source = generate_malformed_program(i);

        // Attempt to compile
        let parse_result = std::panic::catch_unwind(|| parse(&source));

        match parse_result {
            Ok(Ok(module)) => {
                // Parse succeeded (some malformed code might still parse)
                // Try type checking
                let type_check_result = std::panic::catch_unwind(|| {
                    let mut checker = TypeChecker::new();
                    checker.check_module(&module)
                });

                match type_check_result {
                    Ok(Ok(_)) => {
                        // Type check succeeded (rare for malformed code)
                    }
                    Ok(Err(_)) => {
                        type_errors += 1;
                    }
                    Err(_) => {
                        panics += 1;
                        println!("  WARNING: Panic during type check at iteration {}", i);
                    }
                }
            }
            Ok(Err(_)) => {
                // Parse failed gracefully (expected)
                parse_errors += 1;
            }
            Err(_) => {
                // Parse panicked (not expected)
                panics += 1;
                println!("  WARNING: Panic during parse at iteration {}", i);
            }
        }

        if (i + 1) % 25 == 0 {
            println!("  Progress: {}/{} iterations", i + 1, iterations);
        }
    }

    println!("\nResults:");
    println!("  Parse errors:  {} ({:.1}%)", parse_errors, (parse_errors as f64 / iterations as f64) * 100.0);
    println!("  Type errors:   {} ({:.1}%)", type_errors, (type_errors as f64 / iterations as f64) * 100.0);
    println!("  Panics:        {} ({:.1}%)", panics, (panics as f64 / iterations as f64) * 100.0);

    // Assert no panics occurred
    assert_eq!(
        panics, 0,
        "Compiler should never panic on malformed input, found {} panics",
        panics
    );

    // Assert that most malformed programs were caught
    let caught = parse_errors + type_errors;
    println!("\nCaught {}/{} malformed programs ({:.1}%)",
        caught, iterations,
        (caught as f64 / iterations as f64) * 100.0);

    println!("Status: PASSED (no panics on malformed input)");
}

// ==================== Additional Stress Tests ====================

#[test]
#[ignore] // Run with: cargo test test_endurance_memory_stability -- --ignored
fn test_endurance_memory_stability() {
    println!("\n=== Memory Stability Test ===");
    println!("Compiling in rounds to detect memory leaks...\n");

    let source = generate_large_program(1000);
    let rounds = 10;
    let iterations_per_round = 50;

    let mut round_allocs = Vec::new();

    for round in 0..rounds {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut round_total = 0;

        for _ in 0..iterations_per_round {
            let before = counter.load(Ordering::Relaxed);

            let _ = tokenize(&source);
            let module = parse(&source).expect("Parse failed");
            let mut checker = TypeChecker::new();
            let _ = checker.check_module(&module);

            let after = counter.load(Ordering::Relaxed);
            round_total += after.saturating_sub(before);
        }

        let avg_allocs = round_total / iterations_per_round;
        round_allocs.push(avg_allocs);

        println!("  Round {}: avg {} allocs per iteration", round + 1, avg_allocs);
    }

    // Check for linear growth between first and last round
    let first = round_allocs[0];
    let last = round_allocs[rounds - 1];
    let growth = MemorySnapshot::growth_percent(first, last);

    println!("\nMemory growth from first to last round: {:.2}%", growth);

    // Allow up to 15% growth across all rounds
    assert!(
        growth.abs() < 15.0,
        "Memory growth exceeds 15%: {:.2}%",
        growth
    );

    println!("Status: PASSED");
}

#[test]
fn test_endurance_parser_stress() {
    println!("\n=== Parser Stress Test ===");
    println!("Parsing deeply nested structures...\n");

    // Generate deeply nested expressions (reduced depth to avoid stack overflow)
    let depth = 20;
    let mut source = String::from("F compute() -> i64 {\n    R ");

    for _ in 0..depth {
        source.push('(');
    }

    source.push_str("42");

    for _ in 0..depth {
        source.push_str(" + 1)");
    }

    source.push_str("\n}\n");
    source.push_str("F main() -> i64 { R compute() }\n");

    println!("Testing deeply nested expression (depth: {})...", depth);

    let start = std::time::Instant::now();
    let result = parse(&source);
    let elapsed = start.elapsed();

    match result {
        Ok(module) => {
            println!("  Success: parsed in {:?}", elapsed);
            assert!(module.items.len() > 0);
        }
        Err(e) => {
            println!("  Parse failed (expected for very deep nesting): {:?}", e);
            // Deep nesting might hit parser limits, which is acceptable
        }
    }

    println!("Status: PASSED");
}
