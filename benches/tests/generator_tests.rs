//! Tests for the large-scale project generator
//!
//! Validates that generated code is syntactically valid and parseable.

use vais_benches::utils::{
    generate_code, generate_distributed_project, generate_large_project,
    generate_multi_module_project,
};
use vais_parser::parse;

#[test]
fn test_generate_code_basic() {
    // Test the simple generator
    let code = generate_code(10);
    assert!(code.contains("F func0"));
    assert!(code.contains("F func9"));
    assert!(code.contains("F main()->i64 = func0(42)"));

    // Verify it parses
    let result = parse(&code);
    assert!(result.is_ok(), "Generated code should parse: {:?}", result.err());
}

#[test]
fn test_generate_large_project_10k() {
    // Generate 10K line project
    let code = generate_large_project(10_000);
    let line_count = code.lines().count();

    // Should be roughly 10K lines (within 20% tolerance)
    assert!(
        line_count >= 8_000 && line_count <= 12_000,
        "Expected ~10K lines, got {}",
        line_count
    );

    // Should contain expected constructs
    assert!(code.contains("# Module"));
    assert!(code.contains("S Point"));
    assert!(code.contains("E Result"));
    assert!(code.contains("F main()"));

    // Verify it parses
    let result = parse(&code);
    assert!(
        result.is_ok(),
        "10K-line generated code should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_generate_large_project_50k() {
    // Generate 50K line project
    let code = generate_large_project(50_000);
    let line_count = code.lines().count();

    // Should be roughly 50K lines (within 20% tolerance)
    assert!(
        line_count >= 40_000 && line_count <= 60_000,
        "Expected ~50K lines, got {}",
        line_count
    );

    // Verify it parses (this is the real test - can we parse 50K lines?)
    let result = parse(&code);
    assert!(
        result.is_ok(),
        "50K-line generated code should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_generate_large_project_100k() {
    // Generate 100K line project
    let code = generate_large_project(100_000);
    let line_count = code.lines().count();

    // Should be roughly 100K lines (within 20% tolerance)
    assert!(
        line_count >= 80_000 && line_count <= 120_000,
        "Expected ~100K lines, got {}",
        line_count
    );

    // Verify it parses
    let result = parse(&code);
    assert!(
        result.is_ok(),
        "100K-line generated code should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_generate_large_project_contains_variety() {
    // Test that generated code contains diverse constructs
    let code = generate_large_project(5_000);

    // Functions
    assert!(code.contains("_arithmetic_"));
    assert!(code.contains("_recursive_"));
    assert!(code.contains("_conditional_"));
    assert!(code.contains("_loop_"));
    assert!(code.contains("_match_"));
    assert!(code.contains("_ternary_"));
    assert!(code.contains("_struct_"));
    assert!(code.contains("_compute_"));

    // Structs
    assert!(code.contains("S Point"));
    assert!(code.contains("S Container"));

    // Enums
    assert!(code.contains("E Result"));
    assert!(code.contains("Ok(i64)"));
    assert!(code.contains("Err(i64)"));

    // Control flow
    assert!(code.contains("I ")); // if
    assert!(code.contains("L {"));  // loop
    assert!(code.contains("M ")); // match
    assert!(code.contains("R ")); // return

    // Operators
    assert!(code.contains("@(")); // self-recursion
    assert!(code.contains(" ? ")); // ternary
    assert!(code.contains(":= mut")); // mutable binding
}

#[test]
fn test_generate_multi_module_project() {
    // Generate multi-module project
    let modules = generate_multi_module_project(5, 1_000);

    // Should have 6 modules (5 + main)
    assert_eq!(modules.len(), 6, "Should have 5 modules + main");

    // Check filenames
    let filenames: Vec<_> = modules.iter().map(|(name, _)| name.as_str()).collect();
    assert!(filenames.contains(&"module0.vais"));
    assert!(filenames.contains(&"module4.vais"));
    assert!(filenames.contains(&"main.vais"));

    // Verify each module parses
    for (filename, code) in &modules {
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Module {} should parse: {:?}",
            filename,
            result.err()
        );

        // Check for public declarations (except main)
        if filename != "main.vais" {
            assert!(
                code.contains("P S ") || code.contains("P E ") || code.contains("P F "),
                "Module {} should have public declarations",
                filename
            );
        }
    }

    // Main should import other modules
    let main_module = modules
        .iter()
        .find(|(name, _)| name == "main.vais")
        .expect("Should have main.vais");
    assert!(main_module.1.contains("U module0"));
}

#[test]
fn test_generate_distributed_project() {
    // Generate distributed project
    let modules = generate_distributed_project(20_000, 4);

    // Should have 5 modules (4 + main)
    assert_eq!(modules.len(), 5, "Should have 4 modules + main");

    // Total line count should be roughly 20K (note: actual may be lower due to line limiting logic)
    let total_lines: usize = modules.iter().map(|(_, code)| code.lines().count()).sum();
    assert!(
        total_lines >= 10_000 && total_lines <= 25_000,
        "Expected ~10-25K total lines (target 20K), got {}",
        total_lines
    );

    // Verify each module parses
    for (filename, code) in &modules {
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Module {} should parse: {:?}",
            filename,
            result.err()
        );
    }
}

#[test]
fn test_generated_code_has_valid_syntax() {
    // Test specific syntax patterns are correct
    let code = generate_large_project(1_000);

    // Variable declarations should use :=
    assert!(code.contains(":="));

    // Struct field access should use .
    assert!(code.contains("p.x") || code.contains("p.y"));

    // Match syntax should be correct
    if code.contains("M result") {
        assert!(code.contains("=>"));
    }

    // Loop syntax should have braces
    assert!(code.contains("L {"));

    // Return statements should use R
    assert!(code.contains("R "));
}

#[test]
fn test_generator_performance() {
    // Ensure generators complete in reasonable time
    use std::time::Instant;

    let start = Instant::now();
    let _code = generate_large_project(10_000);
    let duration = start.elapsed();

    // Should complete within 1 second
    assert!(
        duration.as_secs() < 1,
        "10K-line generation should be fast, took {:?}",
        duration
    );
}

#[test]
fn test_multi_module_cross_references() {
    // Test that multi-module projects have proper cross-references
    let modules = generate_multi_module_project(3, 500);

    // Find module1 (should reference module0)
    let module1 = modules
        .iter()
        .find(|(name, _)| name == "module1.vais")
        .expect("Should have module1.vais");

    // Should import previous module
    assert!(
        module1.1.contains("U module0"),
        "Module1 should import module0"
    );

    // Should have call_previous function
    assert!(
        module1.1.contains("call_previous"),
        "Module1 should have call_previous function"
    );
}

#[test]
fn test_generate_code_empty() {
    // Edge case: 0 functions
    let code = generate_code(0);
    assert!(code.contains("F main()->i64 = func0(42)"));

    // Should still parse (even though func0 is missing, parser doesn't check semantics)
    let result = parse(&code);
    assert!(result.is_ok());
}

#[test]
fn test_large_project_min_size() {
    // Edge case: very small target
    let code = generate_large_project(10);
    assert!(code.contains("F main()"));

    // Should still parse
    let result = parse(&code);
    assert!(result.is_ok());
}
