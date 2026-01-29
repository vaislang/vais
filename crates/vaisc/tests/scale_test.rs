//! Scale testing infrastructure for the Vais compiler
//!
//! Tests compiler performance and correctness with large codebases (10,000+ lines).
//! These tests verify that the compiler can handle realistic large projects.
//!
//! Run with: `cargo test --test scale_test -- --ignored --nocapture`

use std::time::Instant;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

// ==================== Source Code Generation ====================

/// Generate large source files with specified number of items
struct SourceGenerator {
    struct_count: usize,
    function_count: usize,
    enum_count: usize,
}

impl SourceGenerator {
    fn new(struct_count: usize, function_count: usize, enum_count: usize) -> Self {
        Self {
            struct_count,
            function_count,
            enum_count,
        }
    }

    /// Generate simple structs
    fn generate_structs(&self) -> String {
        let mut source = String::new();
        for i in 0..self.struct_count {
            source.push_str(&format!(
                "S Struct{} {{ field_a: i64, field_b: i64, field_c: bool }}\n",
                i
            ));
        }
        source
    }

    /// Generate simple functions
    fn generate_functions(&self) -> String {
        let mut source = String::new();
        for i in 0..self.function_count {
            source.push_str(&format!("F func{}(x: i64) -> i64 = x + {}\n", i, i));
        }
        source
    }

    /// Generate enums
    fn generate_enums(&self) -> String {
        let mut source = String::new();
        for i in 0..self.enum_count {
            source.push_str(&format!(
                "E Enum{} {{ Variant{}A, Variant{}B(i64), Variant{}C{{ x: i64, y: i64 }} }}\n",
                i, i, i, i
            ));
        }
        source
    }

    /// Generate complete source with all items
    fn generate(&self) -> String {
        let mut source = String::new();
        source.push_str(&self.generate_structs());
        source.push_str(&self.generate_enums());
        source.push_str(&self.generate_functions());
        source
    }

    /// Generate source with realistic patterns - functions calling other functions
    fn generate_with_calls(&self) -> String {
        let mut source = String::new();

        // Generate structs
        source.push_str(&self.generate_structs());
        source.push_str(&self.generate_enums());

        // Generate functions that call each other
        for i in 0..self.function_count {
            if i == 0 {
                // Base case
                source.push_str(&format!("F func{}(x: i64) -> i64 = x + 1\n", i));
            } else {
                // Call previous function
                source.push_str(&format!(
                    "F func{}(x: i64) -> i64 = func{}(x) + {}\n",
                    i,
                    i - 1,
                    i
                ));
            }
        }

        source
    }

    /// Generate source with generic functions
    fn generate_with_generics(&self) -> String {
        let mut source = String::new();

        // Generate generic structs
        for i in 0..self.struct_count {
            source.push_str(&format!("S Struct{}<T> {{ value: T }}\n", i));
        }

        // Generate generic functions
        for i in 0..self.function_count {
            source.push_str(&format!("F func{}<T>(x: T) -> T = x\n", i));
        }

        source
    }

    /// Generate source with pattern matching
    fn generate_with_patterns(&self) -> String {
        let mut source = String::new();

        // Generate Option-like enum
        source.push_str("E Option<T> { None, Some(T) }\n");

        // Generate Result-like enum
        source.push_str("E Result<T, E> { Ok(T), Err(E) }\n");

        // Generate functions with pattern matching
        for i in 0..self.function_count {
            source.push_str(&format!(
                r#"F unwrap_or{}(opt: Option<i64>, default: i64) -> i64 = M opt {{
    Option::Some(x) -> x,
    Option::None -> default
}}
"#,
                i
            ));
        }

        source
    }

    /// Estimate total line count
    fn estimate_lines(&self) -> usize {
        // Structs: 1 line each
        // Enums: 1 line each
        // Functions: 1 line each (for simple functions)
        self.struct_count + self.function_count + self.enum_count
    }
}

// ==================== Performance Measurement ====================

struct BenchResult {
    name: String,
    item_count: usize,
    estimated_lines: usize,
    lexer_time_ms: u128,
    parser_time_ms: u128,
    type_check_time_ms: u128,
    total_time_ms: u128,
}

impl BenchResult {
    fn print(&self) {
        println!("\n=== {} ===", self.name);
        println!("  Items: {}", self.item_count);
        println!("  Estimated lines: {}", self.estimated_lines);
        println!("  Lexer:      {:>6} ms", self.lexer_time_ms);
        println!("  Parser:     {:>6} ms", self.parser_time_ms);
        println!("  Type check: {:>6} ms", self.type_check_time_ms);
        println!("  Total:      {:>6} ms", self.total_time_ms);
        println!(
            "  Throughput: {:>6.1} lines/sec",
            if self.total_time_ms > 0 {
                (self.estimated_lines as f64 / self.total_time_ms as f64) * 1000.0
            } else {
                0.0
            }
        );
    }
}

fn benchmark_compilation(name: &str, source: &str, expected_items: usize, estimated_lines: usize) -> BenchResult {
    // Lexer
    let start = Instant::now();
    let _tokens = tokenize(source).expect("Lexer failed");
    let lexer_time = start.elapsed();

    // Parser
    let start = Instant::now();
    let module = parse(source).expect("Parser failed");
    let parser_time = start.elapsed();

    // Verify item count
    assert_eq!(
        module.items.len(),
        expected_items,
        "Expected {} items, got {}",
        expected_items,
        module.items.len()
    );

    // Type checker
    let start = Instant::now();
    let mut checker = TypeChecker::new();
    checker.check_module(&module).expect("Type checking failed");
    let type_check_time = start.elapsed();

    let total_time = lexer_time + parser_time + type_check_time;

    BenchResult {
        name: name.to_string(),
        item_count: expected_items,
        estimated_lines,
        lexer_time_ms: lexer_time.as_millis(),
        parser_time_ms: parser_time.as_millis(),
        type_check_time_ms: type_check_time.as_millis(),
        total_time_ms: total_time.as_millis(),
    }
}

// ==================== Scale Tests - Parsing ====================

#[test]
fn test_parse_100_items() {
    let gen = SourceGenerator::new(30, 60, 10);
    let source = gen.generate();
    let expected_items = 100;

    let result = benchmark_compilation(
        "Parse 100 items (baseline)",
        &source,
        expected_items,
        gen.estimate_lines(),
    );
    result.print();
}

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_parse_1000_items() {
    let gen = SourceGenerator::new(300, 600, 100);
    let source = gen.generate();
    let expected_items = 1000;

    let result = benchmark_compilation(
        "Parse 1,000 items",
        &source,
        expected_items,
        gen.estimate_lines(),
    );
    result.print();
}

#[test]
#[ignore]
fn test_parse_5000_items() {
    let gen = SourceGenerator::new(1500, 3000, 500);
    let source = gen.generate();
    let expected_items = 5000;

    let result = benchmark_compilation(
        "Parse 5,000 items",
        &source,
        expected_items,
        gen.estimate_lines(),
    );
    result.print();
}

#[test]
#[ignore]
fn test_parse_10000_items() {
    let gen = SourceGenerator::new(3000, 6000, 1000);
    let source = gen.generate();
    let expected_items = 10000;

    let result = benchmark_compilation(
        "Parse 10,000 items",
        &source,
        expected_items,
        gen.estimate_lines(),
    );
    result.print();
}

#[test]
#[ignore]
fn test_parse_50000_items() {
    let gen = SourceGenerator::new(15000, 30000, 5000);
    let source = gen.generate();
    let expected_items = 50000;

    let result = benchmark_compilation(
        "Parse 50,000 items",
        &source,
        expected_items,
        gen.estimate_lines(),
    );
    result.print();
}

// ==================== Scale Tests - Realistic Patterns ====================

#[test]
#[ignore]
fn test_parse_with_function_calls_1000() {
    let gen = SourceGenerator::new(100, 800, 100);
    let source = gen.generate_with_calls();
    let expected_items = 1000;

    let result = benchmark_compilation(
        "Parse 1,000 items with function calls",
        &source,
        expected_items,
        gen.estimate_lines(),
    );
    result.print();
}

#[test]
#[ignore]
fn test_parse_with_function_calls_5000() {
    let gen = SourceGenerator::new(500, 4000, 500);
    let source = gen.generate_with_calls();
    let expected_items = 5000;

    let result = benchmark_compilation(
        "Parse 5,000 items with function calls",
        &source,
        expected_items,
        gen.estimate_lines(),
    );
    result.print();
}

#[test]
#[ignore]
fn test_parse_with_generics_1000() {
    let gen = SourceGenerator::new(300, 700, 0);
    let source = gen.generate_with_generics();
    let expected_items = 1000;

    let result = benchmark_compilation(
        "Parse 1,000 generic items",
        &source,
        expected_items,
        gen.estimate_lines(),
    );
    result.print();
}

#[test]
#[ignore]
fn test_parse_with_generics_5000() {
    let gen = SourceGenerator::new(1500, 3500, 0);
    let source = gen.generate_with_generics();
    let expected_items = 5000;

    let result = benchmark_compilation(
        "Parse 5,000 generic items",
        &source,
        expected_items,
        gen.estimate_lines(),
    );
    result.print();
}

#[test]
#[ignore]
fn test_parse_with_patterns_1000() {
    // 2 enum definitions + 1000 functions = 1002 items
    let gen = SourceGenerator::new(0, 1000, 0);
    let source = gen.generate_with_patterns();
    let expected_items = 1002; // 2 enums + 1000 functions

    let result = benchmark_compilation(
        "Parse 1,000 functions with pattern matching",
        &source,
        expected_items,
        gen.estimate_lines() + 2, // Add 2 for the enum definitions
    );
    result.print();
}

#[test]
#[ignore]
fn test_parse_with_patterns_5000() {
    // 2 enum definitions + 5000 functions = 5002 items
    let gen = SourceGenerator::new(0, 5000, 0);
    let source = gen.generate_with_patterns();
    let expected_items = 5002;

    let result = benchmark_compilation(
        "Parse 5,000 functions with pattern matching",
        &source,
        expected_items,
        gen.estimate_lines() + 2,
    );
    result.print();
}

// ==================== Stress Tests - Deep Nesting ====================

#[test]
#[ignore]
fn test_deep_call_chain() {
    // Test deeply nested function calls (potential stack overflow)
    let mut source = String::new();
    let depth = 100;

    for i in 0..depth {
        if i == 0 {
            source.push_str("F func0(x: i64) -> i64 = x + 1\n");
        } else {
            source.push_str(&format!("F func{}(x: i64) -> i64 = func{}(x + 1)\n", i, i - 1));
        }
    }

    let result = benchmark_compilation(
        "Deep call chain (100 levels)",
        &source,
        depth,
        depth,
    );
    result.print();
}

#[test]
#[ignore]
fn test_wide_match_expression() {
    // Test pattern matching with many branches
    let mut source = String::new();

    // Generate enum with many variants
    source.push_str("E Color { ");
    for i in 0..100 {
        source.push_str(&format!("Color{}", i));
        if i < 99 {
            source.push_str(", ");
        }
    }
    source.push_str(" }\n");

    // Generate function with match on all variants
    source.push_str("F color_to_num(c: Color) -> i64 = M c {\n");
    for i in 0..100 {
        source.push_str(&format!("    Color::Color{} -> {},\n", i, i));
    }
    source.push_str("}\n");

    let result = benchmark_compilation(
        "Wide match expression (100 branches)",
        &source,
        2, // 1 enum + 1 function
        102, // Approximate line count
    );
    result.print();
}

// ==================== Memory Tests ====================

#[test]
#[ignore]
fn test_large_struct_definitions() {
    let mut source = String::new();
    let num_structs = 1000;

    for i in 0..num_structs {
        source.push_str(&format!("S Struct{} {{\n", i));
        // Each struct has 50 fields
        for j in 0..50 {
            source.push_str(&format!("    field_{}: i64,\n", j));
        }
        source.push_str("}\n");
    }

    let result = benchmark_compilation(
        "1,000 structs with 50 fields each",
        &source,
        num_structs,
        num_structs * 52, // Estimate: 1 open + 50 fields + 1 close per struct
    );
    result.print();
}

// ==================== Lexer-Specific Tests ====================

#[test]
fn test_lexer_only_100_items() {
    let gen = SourceGenerator::new(30, 60, 10);
    let source = gen.generate();

    let start = Instant::now();
    let tokens = tokenize(&source).expect("Lexer failed");
    let elapsed = start.elapsed();

    println!("\n=== Lexer Only - 100 items ===");
    println!("  Tokens: {}", tokens.len());
    println!("  Time:   {} ms", elapsed.as_millis());
    println!("  Rate:   {:.0} tokens/sec", (tokens.len() as f64 / elapsed.as_secs_f64()));
}

#[test]
#[ignore]
fn test_lexer_only_10000_items() {
    let gen = SourceGenerator::new(3000, 6000, 1000);
    let source = gen.generate();

    let start = Instant::now();
    let tokens = tokenize(&source).expect("Lexer failed");
    let elapsed = start.elapsed();

    println!("\n=== Lexer Only - 10,000 items ===");
    println!("  Tokens: {}", tokens.len());
    println!("  Time:   {} ms", elapsed.as_millis());
    println!("  Rate:   {:.0} tokens/sec", (tokens.len() as f64 / elapsed.as_secs_f64()));
}

// ==================== Parser-Specific Tests ====================

#[test]
#[ignore]
fn test_parser_only_10000_items() {
    let gen = SourceGenerator::new(3000, 6000, 1000);
    let source = gen.generate();

    let start = Instant::now();
    let module = parse(&source).expect("Parser failed");
    let elapsed = start.elapsed();

    assert_eq!(module.items.len(), 10000);

    println!("\n=== Parser Only - 10,000 items ===");
    println!("  Items:  {}", module.items.len());
    println!("  Time:   {} ms", elapsed.as_millis());
    println!("  Rate:   {:.0} items/sec", (module.items.len() as f64 / elapsed.as_secs_f64()));
}

// ==================== Type Checker-Specific Tests ====================

#[test]
#[ignore]
fn test_type_checker_only_10000_items() {
    let gen = SourceGenerator::new(3000, 6000, 1000);
    let source = gen.generate();

    // Pre-parse
    let module = parse(&source).expect("Parser failed");

    // Time type checking only
    let start = Instant::now();
    let mut checker = TypeChecker::new();
    checker.check_module(&module).expect("Type checking failed");
    let elapsed = start.elapsed();

    println!("\n=== Type Checker Only - 10,000 items ===");
    println!("  Items:  {}", module.items.len());
    println!("  Time:   {} ms", elapsed.as_millis());
    println!("  Rate:   {:.0} items/sec", (module.items.len() as f64 / elapsed.as_secs_f64()));
}

// ==================== Combined Stress Test ====================

#[test]
#[ignore]
fn test_full_scale_stress() {
    println!("\n========================================");
    println!("    FULL SCALE STRESS TEST SUITE");
    println!("========================================");

    let test_cases = vec![
        ("100 items", SourceGenerator::new(30, 60, 10)),
        ("500 items", SourceGenerator::new(150, 300, 50)),
        ("1,000 items", SourceGenerator::new(300, 600, 100)),
        ("5,000 items", SourceGenerator::new(1500, 3000, 500)),
        ("10,000 items", SourceGenerator::new(3000, 6000, 1000)),
    ];

    for (name, gen) in test_cases {
        let source = gen.generate();
        let expected = gen.struct_count + gen.function_count + gen.enum_count;
        let result = benchmark_compilation(name, &source, expected, gen.estimate_lines());
        result.print();
    }

    println!("\n========================================");
    println!("    STRESS TEST COMPLETE");
    println!("========================================");
}
