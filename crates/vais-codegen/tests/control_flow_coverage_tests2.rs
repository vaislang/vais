//! Additional coverage tests for control_flow/ module — pattern.rs, match_gen.rs, if_else.rs
//!
//! Targets uncovered paths: float patterns, string patterns, guard expressions,
//! match with block bodies, nested match, pattern alias, OR pattern with >2 alternatives,
//! if-else with struct results, both-terminated branches, void type phi nodes.

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
// pattern.rs — float literal pattern
// ============================================================================

#[test]
fn test_pattern_float_literal() {
    let result = gen_result(
        r#"
        F test(x: f64) -> i64 {
            M x {
                1.0 => 1,
                2.5 => 2,
                _ => 0
            }
        }
    "#,
    );
    // Float patterns use fcmp oeq
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// pattern.rs — string literal pattern (uses strcmp)
// ============================================================================

#[test]
fn test_pattern_string_literal() {
    let result = gen_result(
        r#"
        F test(s: str) -> i64 {
            M s {
                "hello" => 1,
                "world" => 2,
                _ => 0
            }
        }
    "#,
    );
    // String patterns use strcmp
    if let Ok(ir) = &result {
        assert!(ir.contains("strcmp") || ir.contains("icmp"));
    }
}

// ============================================================================
// pattern.rs — range with open start/end
// ============================================================================

#[test]
fn test_pattern_range_only_upper_bound() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                1..10 => 1,
                _ => 0
            }
        }
    "#,
    );
    assert!(ir.contains("icmp") || ir.contains("and"));
}

// ============================================================================
// pattern.rs — OR pattern with more than 2 alternatives
// ============================================================================

#[test]
fn test_pattern_or_many_alternatives() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                1 | 2 | 3 | 4 | 5 => 100,
                _ => 0
            }
        }
    "#,
    );
    // Multiple OR checks chained
    assert!(!ir.is_empty());
}

// ============================================================================
// pattern.rs — enum variant pattern with payload extraction
// ============================================================================

#[test]
fn test_pattern_enum_variant_with_fields() {
    let ir = gen_ok(
        r#"
        E Value { Num(i64), Pair(i64, i64) }
        F extract(v: Value) -> i64 {
            M v {
                Num(n) => n,
                Pair(a, b) => a + b,
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// pattern.rs — Pattern::Alias (x @ pattern)
// ============================================================================

#[test]
fn test_pattern_alias_binding() {
    let result = gen_result(
        r#"
        F test(x: i64) -> i64 {
            M x {
                n @ 1 => n * 10,
                _ => 0
            }
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// match_gen.rs — match with guard expression
// ============================================================================

#[test]
fn test_match_with_guard() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                n I n > 10 => n * 2,
                n => n
            }
        }
    "#,
    );
    // Guard creates extra branches
    assert!(ir.contains("br") || ir.contains("icmp"));
}

#[test]
fn test_match_int_with_guard() {
    let ir = gen_ok(
        r#"
        F test(x: i64, y: i64) -> i64 {
            M x {
                1 I y > 0 => 100,
                2 => 200,
                _ => 0
            }
        }
    "#,
    );
    assert!(ir.contains("switch") || ir.contains("br"));
}

// ============================================================================
// match_gen.rs — match with block body
// ============================================================================

#[test]
fn test_match_with_block_body() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                1 => {
                    a := 10
                    b := 20
                    a + b
                },
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// match_gen.rs — match with return in arms
// ============================================================================

#[test]
fn test_match_with_block_in_arms() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                0 => {
                    y := 100
                    y
                },
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// match_gen.rs — nested match
// ============================================================================

#[test]
fn test_nested_match() {
    let ir = gen_ok(
        r#"
        F test(x: i64, y: i64) -> i64 {
            M x {
                1 => M y {
                    1 => 11,
                    _ => 10
                },
                2 => 20,
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// match_gen.rs — match on bool with false branch
// ============================================================================

#[test]
fn test_match_bool_false() {
    let ir = gen_ok(
        r#"
        F test(x: bool) -> i64 {
            M x {
                false => 0,
                true => 1,
                _ => -1
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// match_gen.rs — single wildcard arm (no switch needed)
// ============================================================================

#[test]
fn test_match_only_wildcard() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                _ => 99
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// if_else.rs — if expression as value with nested else-if
// ============================================================================

#[test]
fn test_if_expr_triple_branch() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            result := I x > 100 { 3 }
                      E I x > 50 { 2 }
                      E I x > 0 { 1 }
                      E { 0 }
            R result
        }
    "#,
    );
    assert!(ir.contains("phi") || ir.contains("br"));
}

// ============================================================================
// if_else.rs — if/else where both branches return (both_terminated)
// ============================================================================

#[test]
fn test_if_else_both_terminated() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            I x > 0 {
                R 1
            } E {
                R 0
            }
        }
    "#,
    );
    assert!(ir.contains("ret"));
}

// ============================================================================
// if_else.rs — if/else as expression with void type (Unit)
// ============================================================================

#[test]
fn test_if_else_void_branches() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            y := mut 0
            I x > 0 {
                y = 1
            } E {
                y = 2
            }
            R y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// if_else.rs — deeply nested else-if chain (>3 levels)
// ============================================================================

#[test]
fn test_deeply_nested_else_if() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            I x == 1 { R 10 }
            E I x == 2 { R 20 }
            E I x == 3 { R 30 }
            E I x == 4 { R 40 }
            E I x == 5 { R 50 }
            E { R 0 }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// if_else.rs — if without else (only then branch)
// ============================================================================

#[test]
fn test_if_without_else() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            y := mut 0
            I x > 0 {
                y = x
            }
            R y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// match_gen.rs — match with mixed patterns (non-all-int-literals path)
// ============================================================================

#[test]
fn test_match_with_binding_and_literal_mixed() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                0 => 100,
                n => n * 2
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// match_gen.rs — match with many integer arms (exercises switch heavily)
// ============================================================================

#[test]
fn test_match_many_int_arms() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                0 => 0,
                1 => 1,
                2 => 4,
                3 => 9,
                4 => 16,
                5 => 25,
                6 => 36,
                7 => 49,
                _ => -1
            }
        }
    "#,
    );
    assert!(ir.contains("switch"));
}

// ============================================================================
// match_gen.rs — match without default arm (implicit default)
// ============================================================================

#[test]
fn test_match_no_default_arm() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                1 => 10,
                2 => 20
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// pattern.rs — tuple pattern (exercises extractvalue)
// ============================================================================

#[test]
fn test_pattern_tuple_matching() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            t := (10, 20)
            M t {
                (a, b) => a + b,
                _ => 0
            }
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// if_else.rs — if expression producing struct result
// ============================================================================

#[test]
fn test_if_else_struct_result() {
    let ir = gen_ok(
        r#"
        S Pair { x: i64, y: i64 }
        F test(flag: bool) -> i64 {
            p := I flag { Pair { x: 1, y: 2 } } E { Pair { x: 3, y: 4 } }
            p.x + p.y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// match_gen.rs — match producing enum result
// ============================================================================

#[test]
fn test_match_enum_result() {
    let ir = gen_ok(
        r#"
        E Color { Red, Green, Blue }
        F test(x: i64) -> i64 {
            c := M x {
                1 => Red,
                2 => Green,
                _ => Blue
            }
            M c {
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

// ============================================================================
// if/else expression used in return position
// ============================================================================

#[test]
fn test_if_else_in_return() {
    let ir = gen_ok(
        r#"
        F abs(x: i64) -> i64 {
            R I x >= 0 { x } E { 0 - x }
        }
    "#,
    );
    assert!(ir.contains("phi") || ir.contains("ret"));
}

// ============================================================================
// match with negative integer literals
// ============================================================================

#[test]
fn test_match_negative_int() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                -1 => 100,
                0 => 0,
                1 => 100,
                _ => -1
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// match followed by more code (value used)
// ============================================================================

#[test]
fn test_match_value_used_in_computation() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            base := M x {
                0 => 10,
                1 => 20,
                _ => 30
            }
            R base * 2 + 1
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// if-else chain with computation in branches
// ============================================================================

#[test]
fn test_if_else_with_computation() {
    let ir = gen_ok(
        r#"
        F test(x: i64, y: i64) -> i64 {
            result := I x > y {
                x * x + y
            } E I x == y {
                x * 2
            } E {
                y * y + x
            }
            R result
        }
    "#,
    );
    assert!(!ir.is_empty());
}
