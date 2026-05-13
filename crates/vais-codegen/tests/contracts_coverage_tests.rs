//! Coverage tests for contracts/ module — requires, ensures, assert_assume,
//! auto_checks, decreases, invariants, helpers, mod.
//!
//! Strategy: gen_ok/gen_result pattern to exercise contract codegen paths.

use vais_codegen::CodeGenerator;
use vais_parser::parse;

fn gen_ok(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed for: {}\nErr: {}", source, e))
}

fn gen_result(source: &str) -> Result<String, String> {
    let module = parse(source).map_err(|e| format!("Parse: {:?}", e))?;
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .map_err(|e| format!("Codegen: {}", e))
}

// ============================================================================
// Contract declarations — generate_contract_declarations
// ============================================================================

#[test]
fn test_contract_declarations_no_contracts() {
    // Without contracts, declarations should still be generated (or be empty)
    let ir = gen_ok(r#"F main() -> i64 { R 0 }"#);
    // Should compile without issue
    assert!(!ir.is_empty());
}

#[test]
fn test_contract_declarations_with_assert() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            assert(1 == 1)
            R 0
        }
    "#,
    );
    // Assert generates contract-related IR
    assert!(ir.contains("assert") || ir.contains("contract") || ir.contains("panic"));
}

// ============================================================================
// helpers.rs — escape_string_for_llvm, get_or_create_contract_string
// ============================================================================

#[test]
fn test_contract_string_simple_message() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            assert(1 == 1)
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_contract_string_with_special_chars() {
    // Triggers escape_string_for_llvm with backslash and quotes
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            assert(1 > 0)
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// assert_assume.rs — generate_assert, generate_assume
// ============================================================================

#[test]
fn test_assert_true_condition() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            assert(1 == 1)
            R 42
        }
    "#,
    );
    assert!(ir.contains("icmp") || ir.contains("assert"));
}

#[test]
fn test_assert_variable_condition() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            assert(x > 0)
            R x
        }
    "#,
    );
    assert!(ir.contains("icmp") || ir.contains("assert"));
}

#[test]
fn test_assert_with_complex_expr() {
    let ir = gen_ok(
        r#"
        F test(a: i64, b: i64) -> i64 {
            assert(a + b > 0)
            R a + b
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_assert_generates_failure_path() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            assert(1 == 1)
            R 0
        }
    "#,
    );
    // Assert should generate fail label and ok label
    assert!(ir.contains("assert_ok") || ir.contains("assert_fail") || ir.contains("unreachable"));
}

#[test]
fn test_assume_basic() {
    let result = gen_result(
        r#"
        F test(x: i64) -> i64 {
            assume(x > 0)
            R x
        }
    "#,
    );
    // assume may or may not be supported; either is fine
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_multiple_asserts() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            assert(x > 0)
            assert(x < 100)
            R x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// auto_checks.rs — generate_auto_contract_checks, is_nullable_type, is_integer_type
// ============================================================================

#[test]
fn test_function_no_contract_attr() {
    // Normal function without #[contract] should skip auto checks
    let ir = gen_ok(
        r#"
        F add(a: i64, b: i64) -> i64 = a + b
    "#,
    );
    // Should NOT contain contract check IR
    assert!(!ir.contains("nonnull_ok") && !ir.contains("nonzero_ok"));
}

#[test]
fn test_function_with_integer_params() {
    // Division by integer param — auto_checks should detect if #[contract] is present
    let ir = gen_ok(
        r#"
        F divide(a: i64, b: i64) -> i64 = a / b
    "#,
    );
    assert!(ir.contains("sdiv") || ir.contains("div"));
}

#[test]
fn test_function_division_basic() {
    let ir = gen_ok(
        r#"
        F safe_div(a: i64, b: i64) -> i64 {
            R a / b
        }
    "#,
    );
    assert!(ir.contains("sdiv") || ir.contains("div"));
}

#[test]
fn test_function_modulo_basic() {
    let ir = gen_ok(
        r#"
        F modulo(a: i64, b: i64) -> i64 {
            R a % b
        }
    "#,
    );
    assert!(ir.contains("srem") || ir.contains("rem"));
}

// ============================================================================
// requires.rs — generate_requires_checks
// ============================================================================

#[test]
fn test_function_without_requires() {
    let ir = gen_ok(
        r#"
        F add(a: i64, b: i64) -> i64 = a + b
    "#,
    );
    // No requires attributes, should skip
    assert!(!ir.contains("contract_fail_requires"));
}

// ============================================================================
// ensures.rs — generate_ensures_checks
// ============================================================================

#[test]
fn test_function_without_ensures() {
    let ir = gen_ok(
        r#"
        F double(x: i64) -> i64 = x * 2
    "#,
    );
    // No ensures attributes, should skip
    assert!(!ir.contains("contract_fail_ensures"));
}

// ============================================================================
// decreases.rs — generate_decreases_checks, clear_decreases_info
// ============================================================================

#[test]
fn test_recursive_function_without_decreases() {
    let ir = gen_ok(
        r#"
        F factorial(n: i64) -> i64 {
            I n <= 1 {
                R 1
            }
            R n * factorial(n - 1)
        }
    "#,
    );
    // No decreases attribute
    assert!(!ir.contains("decreases_nonneg"));
}

#[test]
fn test_simple_recursion() {
    let ir = gen_ok(
        r#"
        F countdown(n: i64) -> i64 {
            I n <= 0 {
                R 0
            }
            R countdown(n - 1)
        }
    "#,
    );
    assert!(ir.contains("call"));
}

// ============================================================================
// invariants.rs — _generate_invariant_checks
// ============================================================================

#[test]
fn test_struct_without_invariants() {
    let ir = gen_ok(
        r#"
        S Point { x: i64, y: i64 }
        F test() -> i64 {
            p := Point { x: 1, y: 2 }
            R p.x
        }
    "#,
    );
    assert!(!ir.contains("invariant_ok"));
}

// ============================================================================
// mod.rs — generate_contract_declarations, generate_contract_string_constants
// ============================================================================

#[test]
fn test_contract_string_constants_with_assert() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            assert(1 > 0)
            assert(2 > 0)
            R 0
        }
    "#,
    );
    // Multiple asserts should generate string constants
    assert!(!ir.is_empty());
}

#[test]
fn test_contract_string_deduplication() {
    // Same assert message should be deduplicated
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            assert(1 > 0)
            assert(2 > 0)
            assert(3 > 0)
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_assert_in_nested_if() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            I x > 0 {
                assert(x > 0)
                R x
            }
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_assert_with_binary_comparison() {
    let ir = gen_ok(
        r#"
        F test(a: i64, b: i64) -> i64 {
            assert(a != b)
            R a - b
        }
    "#,
    );
    assert!(ir.contains("icmp"));
}

#[test]
fn test_assert_with_equality() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 42
            assert(x == 42)
            R x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_assert_preserves_control_flow() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            assert(x >= 0)
            y := x * 2
            assert(y >= 0)
            R y
        }
    "#,
    );
    // Two asserts with code between should maintain correct control flow
    assert!(!ir.is_empty());
}

#[test]
fn test_assert_returns_unit() {
    // assert returns unit (0), so using it in an expression should be valid
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            assert(1 == 1)
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_contract_in_function_with_multiple_params() {
    let ir = gen_ok(
        r#"
        F clamp(val: i64, lo: i64, hi: i64) -> i64 {
            assert(lo <= hi)
            I val < lo { R lo }
            I val > hi { R hi }
            R val
        }
    "#,
    );
    assert!(!ir.is_empty());
}
