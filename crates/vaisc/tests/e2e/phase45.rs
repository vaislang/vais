use super::helpers::*;

// ==================== Phase 45: Language Basic Edge Cases ====================
// Tests for: lazy/force, comptime, union, match guard, or/range patterns,
// for/while loops, const, global, macro parse, defer parse, assert expr.

// ===== Lazy / Force =====
// Note: lazy_basic (lazy 42, force → 42) covered in phase42.rs (e2e_phase42_lazy_force_basic)
// and execution_tests.rs (exec_lazy_basic).
// Note: lazy_computation (lazy (a+b), force → 30) covered in phase42.rs (e2e_phase42_lazy_force_with_capture).

#[test]
fn e2e_phase45_lazy_multiple_force() {
    // Forcing the same lazy value twice should be valid
    let source = r#"
F main() -> i64 {
    x := lazy 21
    a := force x
    b := force x
    R a + b
}
"#;
    assert_exit_code(source, 42);
}

// ===== Comptime =====

#[test]
fn e2e_phase45_comptime_basic() {
    // comptime block evaluated at compile time
    let source = r#"
F main() -> i64 {
    x := comptime { 2 + 3 }
    R x
}
"#;
    assert_exit_code(source, 5);
}

// ===== Union =====

#[test]
fn e2e_phase45_union_parse() {
    // Union type declaration should parse and compile
    let source = r#"
O Value {
    int_val: i64,
    flt_val: f64
}

F main() -> i64 {
    R 0
}
"#;
    assert_exit_code(source, 0);
}

// ===== Match Guard =====

#[test]
fn e2e_phase45_match_guard_basic() {
    // Match arms with guard conditions
    let source = r#"
F classify(n: i64) -> i64 {
    M n {
        x I x > 100 => 3,
        x I x > 50 => 2,
        x I x > 0 => 1,
        _ => 0
    }
}

F main() -> i64 {
    R classify(75)
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_phase45_match_guard_with_literal() {
    // Guard on a specific literal arm
    let source = r#"
F check(n: i64) -> i64 {
    M n {
        0 => 100,
        x I x < 0 => 50,
        _ => 1
    }
}

F main() -> i64 {
    R check(0)
}
"#;
    assert_exit_code(source, 100);
}

// ===== Or Pattern =====
// Note: match_or_pattern covered by phase32_patterns.rs (e2e_phase32_pattern_or_simple)

// ===== Range Pattern =====

#[test]
fn e2e_phase45_match_range_pattern() {
    // Exclusive range patterns in match
    let source = r#"
F bucket(n: i64) -> i64 {
    M n {
        0..10 => 1,
        10..20 => 2,
        _ => 0
    }
}

F main() -> i64 {
    R bucket(15)
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_phase45_match_inclusive_range() {
    // Inclusive range pattern (..=)
    let source = r#"
F check(n: i64) -> i64 {
    M n {
        0..=5 => 1,
        6..=10 => 2,
        _ => 0
    }
}

F main() -> i64 {
    R check(5)
}
"#;
    assert_exit_code(source, 1);
}

// ===== For Loop =====

#[test]
fn e2e_phase45_for_loop_sum() {
    // Exclusive range for-loop summing 0..10
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i:0..10 {
        sum = sum + i
    }
    R sum
}
"#;
    // 0+1+2+...+9 = 45
    assert_exit_code(source, 45);
}

#[test]
fn e2e_phase45_for_loop_nested() {
    // Nested for-loops counting iterations
    let source = r#"
F main() -> i64 {
    count := mut 0
    L i:0..5 {
        L j:0..3 {
            count = count + 1
        }
    }
    R count
}
"#;
    // 5 * 3 = 15
    assert_exit_code(source, 15);
}

// ===== While Loop =====

#[test]
fn e2e_phase45_while_loop_countdown() {
    // Condition-based loop counts down to zero
    let source = r#"
F main() -> i64 {
    n := mut 10
    L n > 0 {
        n = n - 1
    }
    R n
}
"#;
    assert_exit_code(source, 0);
}

// ===== Defer =====

#[test]
fn e2e_phase45_defer_parse() {
    // Defer statement should parse and compile (execution verified separately)
    let source = r#"
F main() -> i64 {
    D puts("cleanup")
    R 42
}
"#;
    assert_exit_code(source, 42);
}

// Note: const_literal covered by phase37_union_const.rs (e2e_p37_const_basic_usage)
// Note: global_variable_parse covered by phase37_union_const.rs (e2e_p37_global_single)
// Note: macro_parse covered by phase37_comptime_defer.rs (e2e_p37_macro_simple_parse)

// ===== Assert Expression =====

#[test]
fn e2e_phase45_assert_expr() {
    // assert() builtin should compile without error
    let source = r#"
F main() -> i64 {
    x := 10
    assert(x > 0)
    R x
}
"#;
    assert_exit_code(source, 10);
}
