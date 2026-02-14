//! Phase 32: Language feature extensions
//!
//! Tests for three new language features:
//! - Closure capture modes (move)
//! - Where clause syntax
//! - Pattern alias (@)

use super::helpers::*;

// ===== Closure Capture Modes =====

#[test]
fn e2e_closure_default_capture() {
    // Default capture (ByValue) - verify existing behavior
    let source = r#"
F main() -> i64 {
    x := 10
    f := |y| x + y
    R f(5)
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_closure_move_capture() {
    // Explicit move closure
    let source = r#"
F main() -> i64 {
    x := 42
    f := move |y| x + y
    R f(0)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_move_multiple_captures() {
    // Move closure with multiple captured variables
    let source = r#"
F main() -> i64 {
    a := 10
    b := 20
    f := move |x| a + b + x
    R f(3)
}
"#;
    assert_exit_code(source, 33);
}

#[test]
fn e2e_closure_move_nested() {
    // Nested closures with move
    let source = r#"
F main() -> i64 {
    x := 5
    outer := move |y| {
        inner := move |z| x + y + z
        inner(10)
    }
    R outer(3)
}
"#;
    assert_exit_code(source, 18);
}

#[test]
fn e2e_closure_move_compile_only() {
    // Test that move syntax parses and compiles correctly
    let source = r#"
F main() -> i64 {
    value := 100
    f := move || value
    0
}
"#;
    // Just verify it compiles - we're testing syntax parsing
    compile_to_ir(source).expect("should compile");
}

// ===== Where Clause =====

#[test]
fn e2e_where_clause_basic() {
    // Basic where clause on function (trait bounds not enforced yet)
    let source = r#"
F identity<T>(x: T) -> T where T: Clone {
    R x
}

F main() -> i64 {
    R identity(42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_where_clause_multiple_bounds() {
    // Where clause with multiple trait bounds
    let source = r#"
F process<T, U>(a: T, b: U) -> i64 where T: Clone, U: Clone {
    R 99
}

F main() -> i64 {
    R process(1, 2)
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn e2e_where_clause_compile_only() {
    // Test where clause parsing without execution
    let source = r#"
F test<T>(x: T) -> T where T: Display {
    R x
}

F main() -> i64 {
    0
}
"#;
    // Just verify it compiles - where clause is parsed but not enforced
    compile_to_ir(source).expect("should compile");
}

#[test]
fn e2e_where_clause_mixed_bounds() {
    // Inline bounds + where clause together
    let source = r#"
F mixed<T: Clone>(x: T) -> T where T: Display {
    R x
}

F main() -> i64 {
    R mixed(42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_where_clause_multiple_params() {
    // Where clause with multiple type parameters
    let source = r#"
F combine<A, B, C>(a: A, b: B, c: C) -> i64 where A: Clone, B: Display, C: Debug {
    R 77
}

F main() -> i64 {
    R combine(1, 2, 3)
}
"#;
    assert_exit_code(source, 77);
}

// ===== Pattern Alias (@) =====

#[test]
fn e2e_pattern_alias_basic() {
    // Basic pattern alias with literal
    let source = r#"
F main() -> i64 {
    x := 42
    M x {
        n @ 42 => n,
        _ => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_pattern_alias_wildcard() {
    // Pattern alias with wildcard
    let source = r#"
F main() -> i64 {
    x := 99
    M x {
        n @ _ => n
    }
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn e2e_pattern_alias_with_literal() {
    // Pattern alias with different literals
    let source = r#"
F main() -> i64 {
    x := 2
    M x {
        a @ 1 => a * 10,
        n @ 2 => n * 10,
        b @ 3 => b * 10,
        _ => 0
    }
}
"#;
    assert_exit_code(source, 20);
}

#[test]
fn e2e_pattern_alias_multiple_cases() {
    // Multiple pattern aliases in different branches
    let source = r#"
F main() -> i64 {
    x := 5
    M x {
        a @ 1 => a,
        b @ 5 => b * 2,
        c @ _ => c + 100
    }
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_pattern_alias_range() {
    // Pattern alias with range pattern
    let source = r#"
F main() -> i64 {
    x := 7
    M x {
        n @ 5..=8 => n,
        _ => 0
    }
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn e2e_pattern_alias_nested_match() {
    // Nested match with pattern alias - simplified
    let source = r#"
F main() -> i64 {
    x := 5
    result := M x {
        outer @ 5 => outer * 2,
        _ => 0
    }
    R result
}
"#;
    assert_exit_code(source, 10);
}

// ===== Combined Features =====

#[test]
fn e2e_move_closure_with_pattern_alias() {
    // Combine move closure and pattern alias
    let source = r#"
F main() -> i64 {
    x := 10
    f := move |y| {
        M y {
            n @ 5 => x + n,
            _ => 0
        }
    }
    R f(5)
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_where_clause_with_pattern_alias() {
    // Combine where clause and pattern alias
    let source = r#"
F process<T>(val: T) -> i64 where T: Clone {
    M val {
        n @ 42 => n,
        _ => 0
    }
}

F main() -> i64 {
    R process(42)
}
"#;
    assert_exit_code(source, 42);
}
