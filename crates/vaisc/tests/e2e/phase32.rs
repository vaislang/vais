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
fn main() -> i64 {
    x := 10
    f := |y| x + y
    return f(5)
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_closure_move_capture() {
    // Explicit move closure
    let source = r#"
fn main() -> i64 {
    x := 42
    f := move |y| x + y
    return f(0)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_move_multiple_captures() {
    // Move closure with multiple captured variables
    let source = r#"
fn main() -> i64 {
    a := 10
    b := 20
    f := move |x| a + b + x
    return f(3)
}
"#;
    assert_exit_code(source, 33);
}

#[test]
fn e2e_closure_move_nested() {
    // Nested closures with move
    let source = r#"
fn main() -> i64 {
    x := 5
    outer := move |y| {
        inner := move |z| x + y + z
        inner(10)
    }
    return outer(3)
}
"#;
    assert_exit_code(source, 18);
}

#[test]
fn e2e_closure_move_compile_only() {
    // Test that move syntax parses and compiles correctly
    let source = r#"
fn main() -> i64 {
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
fn identity<T>(x: T) -> type where T: Clone {
    return x
}

fn main() -> i64 {
    return identity(42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_where_clause_multiple_bounds() {
    // Where clause with multiple trait bounds
    let source = r#"
fn process<T, U>(a: T, b: U) -> i64 where T: Clone, U: Clone {
    return 99
}

fn main() -> i64 {
    return process(1, 2)
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn e2e_where_clause_compile_only() {
    // Test where clause parsing without execution
    let source = r#"
fn test<T>(x: T) -> type where T: Display {
    return x
}

fn main() -> i64 {
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
fn mixed<T: Clone>(x: T) -> type where T: Display {
    return x
}

fn main() -> i64 {
    return mixed(42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_where_clause_multiple_params() {
    // Where clause with multiple type parameters
    let source = r#"
fn combine<A, B, C>(a: A, b: B, c: C) -> i64 where A: Clone, B: Display, C: Debug {
    return 77
}

fn main() -> i64 {
    return combine(1, 2, 3)
}
"#;
    assert_exit_code(source, 77);
}

// ===== Pattern Alias (@) =====

#[test]
fn e2e_pattern_alias_basic() {
    // Basic pattern alias with literal
    let source = r#"
fn main() -> i64 {
    x := 42
    match x {
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
fn main() -> i64 {
    x := 99
    match x {
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
fn main() -> i64 {
    x := 2
    match x {
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
fn main() -> i64 {
    x := 5
    match x {
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
fn main() -> i64 {
    x := 7
    match x {
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
fn main() -> i64 {
    x := 5
    result := match x {
        outer @ 5 => outer * 2,
        _ => 0
    }
    return result
}
"#;
    assert_exit_code(source, 10);
}

// ===== Combined Features =====

#[test]
fn e2e_move_closure_with_pattern_alias() {
    // Combine move closure and pattern alias
    let source = r#"
fn main() -> i64 {
    x := 10
    f := move |y| {
        match y {
            n @ 5 => x + n,
            _ => 0
        }
    }
    return f(5)
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_where_clause_with_pattern_alias() {
    // Combine where clause and pattern alias
    let source = r#"
fn process<T>(val: T) -> i64 where T: Clone {
    match val {
        n @ 42 => n,
        _ => 0
    }
}

fn main() -> i64 {
    return process(42)
}
"#;
    assert_exit_code(source, 42);
}

// ===== Phase 34: Codegen Bug Fixes =====

// REGRESSION(phase-34): enum unit variant tag mismatch — {i32} vs {i8,i64}
#[test]
fn e2e_enum_unit_variant_matching() {
    // Tests that enum unit variants correctly match their respective arms
    let source = r#"
enum Color { Red, Green, Blue }
fn color_to_int(c: Color) -> i64 {
    match c {
        Red => 1,
        Green => 2,
        Blue => 3
    }
}
fn main() -> i64 {
    r := color_to_int(Red)
    g := color_to_int(Green)
    b := color_to_int(Blue)
    I r == 1 && g == 2 && b == 3 { 0 } else { 1 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_enum_unit_variant_three_way() {
    // Verifies each unit variant arm returns distinct values
    let source = r#"
enum Status { Pending, Running, Done }
fn status_code(s: Status) -> i64 {
    match s { Pending => 10, Running => 20, Done => 30 }
}
fn main() -> i64 {
    a := status_code(Pending)
    b := status_code(Running)
    c := status_code(Done)
    a + b + c
}
"#;
    assert_exit_code(source, 60);
}

// REGRESSION(phase-34): struct by-value parameter field access ABI
#[test]
fn e2e_struct_by_value_parameter() {
    // Tests passing struct by value and accessing multiple fields
    let source = r#"
struct Point { x: i64, y: i64 }
fn sum_point(p: Point) -> i64 {
    p.x + p.y
}
fn main() -> i64 {
    pt := Point { x: 10, y: 32 }
    sum_point(pt)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_by_value_nested_fields() {
    // Tests struct by-value with arithmetic on multiple fields
    let source = r#"
struct Rect { w: i64, h: i64 }
fn area(r: Rect) -> i64 {
    r.w * r.h
}
fn main() -> i64 {
    rect := Rect { w: 6, h: 7 }
    area(rect)
}
"#;
    assert_exit_code(source, 42);
}

// REGRESSION(phase-34): match arm phi node type mismatch for enum returns
#[test]
fn e2e_enum_return_from_match() {
    // Tests returning enum variants from match arms (phi node fix)
    let source = r#"
enum Result { Ok(i64), Err(i64) }
fn transform(r: Result) -> Result {
    match r {
        Ok(v) => Ok(v * 2),
        Err(e) => Err(e + 1)
    }
}
fn get_val(r: Result) -> i64 {
    match r { Ok(v) => v, Err(_) => 0 - 1 }
}
fn main() -> i64 {
    r := Ok(21)
    t := transform(r)
    get_val(t)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_return_err_path() {
    // Tests that Err path also works correctly for enum return
    let source = r#"
enum Result { Ok(i64), Err(i64) }
fn transform(r: Result) -> Result {
    match r {
        Ok(v) => Ok(v * 2),
        Err(e) => Err(e + 100)
    }
}
fn get_err(r: Result) -> i64 {
    match r { Ok(_) => 0, Err(e) => e }
}
fn main() -> i64 {
    r := Err(5)
    t := transform(r)
    get_err(t)
}
"#;
    assert_exit_code(source, 105);
}
