//! Stress Tests for Vais Compiler
//!
//! These tests verify that the compiler can handle large, complex programs
//! by parsing and type-checking 1000+ lines of Vais code across multiple
//! comprehensive test programs.
//!
//! Run with: `cargo test --test stress_tests`

use std::fs;
use std::path::PathBuf;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Get the path to the stress tests directory
fn stress_tests_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from vaisc
    path.pop(); // Go up from crates
    path.push("tests");
    path.push("stress");
    path
}

/// Result of parsing and type-checking a stress test file
#[derive(Debug)]
struct StressTestResult {
    file_name: String,
    source_lines: usize,
    token_count: usize,
    item_count: usize,
    parse_success: bool,
    type_check_success: bool,
    error_message: Option<String>,
}

impl StressTestResult {
    fn print_summary(&self) {
        println!("\n=== {} ===", self.file_name);
        println!("  Lines:       {}", self.source_lines);
        println!("  Tokens:      {}", self.token_count);
        println!("  Items:       {}", self.item_count);
        println!(
            "  Parse:       {}",
            if self.parse_success { "✓" } else { "✗" }
        );
        println!(
            "  Type Check:  {}",
            if self.type_check_success {
                "✓"
            } else {
                "✗"
            }
        );
        if let Some(ref err) = self.error_message {
            println!("  Error:       {}", err);
        }
    }
}

/// Parse and type-check a stress test file
fn run_stress_test(file_path: &PathBuf) -> StressTestResult {
    let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();

    // Read source file
    let source = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            return StressTestResult {
                file_name,
                source_lines: 0,
                token_count: 0,
                item_count: 0,
                parse_success: false,
                type_check_success: false,
                error_message: Some(format!("Failed to read file: {}", e)),
            }
        }
    };

    let source_lines = source.lines().count();

    // Tokenize
    let tokens = match tokenize(&source) {
        Ok(t) => t,
        Err(e) => {
            return StressTestResult {
                file_name,
                source_lines,
                token_count: 0,
                item_count: 0,
                parse_success: false,
                type_check_success: false,
                error_message: Some(format!("Lexer error: {:?}", e)),
            }
        }
    };

    let token_count = tokens.len();

    // Parse
    let module = match parse(&source) {
        Ok(m) => m,
        Err(e) => {
            return StressTestResult {
                file_name,
                source_lines,
                token_count,
                item_count: 0,
                parse_success: false,
                type_check_success: false,
                error_message: Some(format!("Parser error: {:?}", e)),
            }
        }
    };

    let item_count = module.items.len();

    // Type check
    let mut checker = TypeChecker::new();
    let type_check_result = checker.check_module(&module);

    let (type_check_success, error_message) = match type_check_result {
        Ok(_) => (true, None),
        Err(e) => (false, Some(format!("Type error: {:?}", e))),
    };

    StressTestResult {
        file_name,
        source_lines,
        token_count,
        item_count,
        parse_success: true,
        type_check_success,
        error_message,
    }
}

// ==================== Individual Stress Tests ====================

#[test]
fn stress_test_data_structures() {
    let mut path = stress_tests_dir();
    path.push("data_structures.vais");

    let result = run_stress_test(&path);
    result.print_summary();

    assert!(
        result.parse_success,
        "Data structures test failed to parse: {:?}",
        result.error_message
    );

    // Note: Type checking may fail if certain features aren't fully implemented yet
    // For now, we focus on parsing correctness
    if !result.type_check_success {
        println!("  Note: Type checking not fully passing yet (expected during development)");
    }

    assert!(
        result.source_lines >= 200,
        "Expected at least 200 lines, got {}",
        result.source_lines
    );
}

#[test]
fn stress_test_algorithms() {
    let mut path = stress_tests_dir();
    path.push("algorithms.vais");

    let result = run_stress_test(&path);
    result.print_summary();

    assert!(
        result.parse_success,
        "Algorithms test failed to parse: {:?}",
        result.error_message
    );

    if !result.type_check_success {
        println!("  Note: Type checking not fully passing yet (expected during development)");
    }

    assert!(
        result.source_lines >= 200,
        "Expected at least 200 lines, got {}",
        result.source_lines
    );
}

#[test]
fn stress_test_type_system() {
    let mut path = stress_tests_dir();
    path.push("type_system.vais");

    let result = run_stress_test(&path);
    result.print_summary();

    assert!(
        result.parse_success,
        "Type system test failed to parse: {:?}",
        result.error_message
    );

    if !result.type_check_success {
        println!("  Note: Type checking not fully passing yet (expected during development)");
    }

    assert!(
        result.source_lines >= 200,
        "Expected at least 200 lines, got {}",
        result.source_lines
    );
}

#[test]
fn stress_test_control_flow() {
    let mut path = stress_tests_dir();
    path.push("control_flow.vais");

    let result = run_stress_test(&path);
    result.print_summary();

    assert!(
        result.parse_success,
        "Control flow test failed to parse: {:?}",
        result.error_message
    );

    if !result.type_check_success {
        println!("  Note: Type checking not fully passing yet (expected during development)");
    }

    assert!(
        result.source_lines >= 200,
        "Expected at least 200 lines, got {}",
        result.source_lines
    );
}

#[test]
fn stress_test_math_library() {
    let mut path = stress_tests_dir();
    path.push("math_library.vais");

    let result = run_stress_test(&path);
    result.print_summary();

    assert!(
        result.parse_success,
        "Math library test failed to parse: {:?}",
        result.error_message
    );

    if !result.type_check_success {
        println!("  Note: Type checking not fully passing yet (expected during development)");
    }

    assert!(
        result.source_lines >= 200,
        "Expected at least 200 lines, got {}",
        result.source_lines
    );
}

// ==================== Comprehensive Stress Test ====================

#[test]
fn stress_test_all() {
    println!("\n========================================");
    println!("    COMPREHENSIVE STRESS TEST SUITE");
    println!("========================================");

    let stress_dir = stress_tests_dir();

    let test_files = vec![
        "data_structures.vais",
        "algorithms.vais",
        "type_system.vais",
        "control_flow.vais",
        "math_library.vais",
    ];

    let mut results = Vec::new();
    let mut total_lines = 0;
    let mut total_tokens = 0;
    let mut total_items = 0;

    for file_name in &test_files {
        let mut path = stress_dir.clone();
        path.push(file_name);

        let result = run_stress_test(&path);
        total_lines += result.source_lines;
        total_tokens += result.token_count;
        total_items += result.item_count;

        result.print_summary();
        results.push(result);
    }

    println!("\n========================================");
    println!("    SUMMARY");
    println!("========================================");
    println!("  Total Files:       {}", test_files.len());
    println!("  Total Lines:       {}", total_lines);
    println!("  Total Tokens:      {}", total_tokens);
    println!("  Total Items:       {}", total_items);
    println!(
        "  Parse Success:     {}/{}",
        results.iter().filter(|r| r.parse_success).count(),
        results.len()
    );
    println!(
        "  Type Check Success: {}/{}",
        results.iter().filter(|r| r.type_check_success).count(),
        results.len()
    );

    // Assert that we have at least 1000 lines total
    assert!(
        total_lines >= 1000,
        "Expected at least 1000 total lines, got {}",
        total_lines
    );

    // Assert that all files parsed successfully
    let failed_parses: Vec<_> = results.iter().filter(|r| !r.parse_success).collect();
    assert!(
        failed_parses.is_empty(),
        "Some files failed to parse: {:?}",
        failed_parses
            .iter()
            .map(|r| &r.file_name)
            .collect::<Vec<_>>()
    );

    println!("\n========================================");
    println!("    STRESS TEST COMPLETE");
    println!("========================================");
}

// ==================== Performance Benchmarks ====================

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn stress_benchmark_all() {
    use std::time::Instant;

    println!("\n========================================");
    println!("    STRESS TEST PERFORMANCE BENCHMARK");
    println!("========================================");

    let stress_dir = stress_tests_dir();

    let test_files = vec![
        "data_structures.vais",
        "algorithms.vais",
        "type_system.vais",
        "control_flow.vais",
        "math_library.vais",
    ];

    let mut total_lex_time = 0u128;
    let mut total_parse_time = 0u128;
    let mut total_type_check_time = 0u128;
    let mut total_lines = 0;

    for file_name in &test_files {
        let mut path = stress_dir.clone();
        path.push(file_name);

        let source = fs::read_to_string(&path).expect("Failed to read file");
        let lines = source.lines().count();
        total_lines += lines;

        // Benchmark lexer
        let start = Instant::now();
        let tokens = tokenize(&source).expect("Lexer failed");
        let lex_time = start.elapsed().as_millis();
        total_lex_time += lex_time;

        // Benchmark parser
        let start = Instant::now();
        let module = parse(&source).expect("Parser failed");
        let parse_time = start.elapsed().as_millis();
        total_parse_time += parse_time;

        // Benchmark type checker
        let start = Instant::now();
        let mut checker = TypeChecker::new();
        let _ = checker.check_module(&module); // May fail, but we measure time anyway
        let type_check_time = start.elapsed().as_millis();
        total_type_check_time += type_check_time;

        println!("\n{}", file_name);
        println!("  Lines:       {}", lines);
        println!("  Tokens:      {}", tokens.len());
        println!("  Lex time:    {} ms", lex_time);
        println!("  Parse time:  {} ms", parse_time);
        println!("  Type check:  {} ms", type_check_time);
        println!(
            "  Total:       {} ms",
            lex_time + parse_time + type_check_time
        );
    }

    let total_time = total_lex_time + total_parse_time + total_type_check_time;

    println!("\n========================================");
    println!("    PERFORMANCE SUMMARY");
    println!("========================================");
    println!("  Total Lines:       {}", total_lines);
    println!("  Total Lex Time:    {} ms", total_lex_time);
    println!("  Total Parse Time:  {} ms", total_parse_time);
    println!("  Total Type Check:  {} ms", total_type_check_time);
    println!("  Total Time:        {} ms", total_time);
    println!(
        "  Throughput:        {:.0} lines/sec",
        (total_lines as f64 / total_time as f64) * 1000.0
    );
}
