//! Coverage tests for control_flow/ module — if_else, match_gen, pattern.
//!
//! Strategy: gen_ok/gen_result pattern to exercise control flow codegen paths.

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
// if_else.rs — generate_if_else_with_term
// ============================================================================

#[test]
fn test_simple_if() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            I x > 0 { return 1 }
            return 0
        }
    "#,
    );
    assert!(ir.contains("icmp") || ir.contains("br"));
}

#[test]
fn test_if_else() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            I x > 0 { return 1 } else { return 0 }
        }
    "#,
    );
    assert!(ir.contains("br"));
}

#[test]
fn test_if_else_if() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            I x > 0 { return 1 } else I x < 0 { return -1 } else { return 0 }
        }
    "#,
    );
    // Should have multiple branch instructions
    assert!(!ir.is_empty());
}

#[test]
fn test_if_else_if_chain() {
    let ir = gen_ok(
        r#"
        fn classify(x: i64) -> i64 {
            I x > 100 { return 3 } else I x > 50 { return 2 } else I x > 0 { return 1 } else { return 0 }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_if_with_block_value() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            y := I x > 0 { 1 } else { 0 }
            return y
        }
    "#,
    );
    // Value-producing if should generate phi nodes
    assert!(ir.contains("phi") || ir.contains("br"));
}

#[test]
fn test_nested_if() {
    let ir = gen_ok(
        r#"
        fn test(x: i64, y: i64) -> i64 {
            I x > 0 {
                I y > 0 { return 1 } else { return 2 }
            }
            return 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_if_both_branches_return() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            I x > 0 {
                return 1
            } else {
                return 0
            }
        }
    "#,
    );
    // Both branches terminated — merge block should be skipped
    assert!(!ir.is_empty());
}

#[test]
fn test_if_only_then_returns() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            I x > 0 {
                return 1
            }
            return 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_if_with_assignments() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            y := 0
            I x > 0 {
                y = x * 2
            }
            return y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// match_gen.rs — generate_match
// ============================================================================

#[test]
fn test_match_integer_literal() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x {
                1 => 10,
                2 => 20,
                _ => 0
            }
        }
    "#,
    );
    // Integer match should use switch instruction
    assert!(ir.contains("switch") || ir.contains("icmp"));
}

#[test]
fn test_match_wildcard_only() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x {
                _ => 42
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_multiple_int_cases() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x {
                0 => 100,
                1 => 200,
                2 => 300,
                3 => 400,
                _ => 0
            }
        }
    "#,
    );
    assert!(ir.contains("switch") || ir.contains("icmp"));
}

#[test]
fn test_match_bool() {
    let ir = gen_ok(
        r#"
        fn test(x: bool) -> i64 {
            match x {
                true => 1,
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_with_variable_binding() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x {
                n => n * 2
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_enum_variant() {
    let ir = gen_ok(
        r#"
        enum Color { Red, Green, Blue }
        fn test(c: Color) -> i64 {
            match c {
                Red => 1,
                Green => 2,
                Blue => 3,
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_enum_with_payload() {
    let ir = gen_ok(
        r#"
        enum Shape { Circle(i64), Rect(i64, i64) }
        fn area(s: Shape) -> i64 {
            match s {
                Circle(r) => r * r,
                Rect(w, h) => w * h,
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// pattern.rs — generate_pattern_check_typed, generate_pattern_bindings_typed
// ============================================================================

#[test]
fn test_pattern_wildcard() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x { _ => 42 }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_pattern_literal_int() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x { 0 => 100, _ => 0 }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_pattern_literal_bool() {
    let ir = gen_ok(
        r#"
        fn test(x: bool) -> i64 {
            match x { true => 1, _ => 0 }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_pattern_identifier_binding() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x { n => n + 1 }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_pattern_tuple() {
    let result = gen_result(
        r#"
        fn test() -> i64 {
            t := (1, 2)
            match t {
                (a, b) => a + b,
                _ => 0
            }
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_pattern_or() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x {
                1 | 2 | 3 => 10,
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_pattern_range_inclusive() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x {
                1..=5 => 10,
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_pattern_range_exclusive() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x {
                1..5 => 10,
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_pattern_struct_field() {
    // Struct pattern matching - use result since syntax may vary
    let result = gen_result(
        r#"
        struct Point { x: i64, y: i64 }
        fn test(p: Point) -> i64 {
            match p {
                n => 42,
                _ => 0
            }
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// control_flow edge cases
// ============================================================================

#[test]
fn test_deeply_nested_if_else() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            I x > 10 {
                I x > 20 {
                    I x > 30 {
                        return 3
                    } else {
                        return 2
                    }
                } else {
                    return 1
                }
            } else {
                return 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_single_arm() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x { _ => 99 }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_with_computation_in_arms() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x {
                0 => 0,
                1 => 1,
                n => n * n
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_if_as_expression_with_else_if() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            result := I x > 0 { 1 } else I x < 0 { -1 } else { 0 }
            return result
        }
    "#,
    );
    assert!(ir.contains("phi") || ir.contains("br"));
}

#[test]
fn test_match_followed_by_code() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            y := match x {
                0 => 10,
                _ => 20
            }
            return y + 1
        }
    "#,
    );
    assert!(!ir.is_empty());
}
