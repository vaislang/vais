use super::helpers::*;

// ==================== Phase 45: Language Basic Edge Cases ====================
// Tests for: lazy/force, comptime, union, match guard, or/range patterns,
// for/while loops, const, global, macro parse, defer parse, assert expr.

// ===== Lazy / Force =====

#[test]
fn e2e_phase45_lazy_basic() {
    // Basic lazy value creation and force evaluation
    let source = r#"
F main() -> i64 {
    x := lazy 42
    R force x
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase45_lazy_computation() {
    // Lazy expression capturing outer variables
    let source = r#"
F main() -> i64 {
    a := 10
    b := 20
    x := lazy (a + b)
    R force x
}
"#;
    assert_compiles(source);
}

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
    assert_compiles(source);
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
    assert_compiles(source);
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
    assert_compiles(source);
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

#[test]
fn e2e_phase45_match_or_pattern() {
    // Or patterns matching multiple literals
    let source = r#"
F kind(n: i64) -> i64 {
    M n {
        1 | 2 | 3 => 10,
        4 | 5 => 20,
        _ => 0
    }
}

F main() -> i64 {
    R kind(2)
}
"#;
    assert_exit_code(source, 10);
}

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
    assert_compiles(source);
}

// ===== Const =====

#[test]
fn e2e_phase45_const_literal() {
    // Const definition used in main
    let source = r#"
C MAX: i64 = 100

F main() -> i64 {
    R MAX
}
"#;
    assert_exit_code(source, 100);
}

// ===== Global Variable =====

#[test]
fn e2e_phase45_global_variable_parse() {
    // Global variable declaration — parser accepts G syntax
    let source = r#"
G counter: i64 = 0

F main() -> i64 {
    R 0
}
"#;
    assert_compiles(source);
}

// ===== Macro =====

#[test]
fn e2e_phase45_macro_parse() {
    // Macro definition should parse without error
    let source = r#"
macro vec_new! {
    () => { 0 }
}

F main() -> i64 {
    R 0
}
"#;
    assert_compiles(source);
}

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
    assert_compiles(source);
}
