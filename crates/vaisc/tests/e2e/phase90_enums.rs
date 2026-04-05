//! Phase 90 -- Enum Patterns and Matching
//!
//! Tests for enum definitions, variant construction, pattern matching,
//! and enum-based logic.

use super::helpers::*;

// ==================== Basic Enum ====================

#[test]
fn e2e_enum_simple_variant() {
    let source = r#"
E Color { Red, Green, Blue }
F main() -> i64 {
    c := Red
    M c {
        Red => 42,
        Green => 1,
        Blue => 2
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_second_variant() {
    let source = r#"
E Color { Red, Green, Blue }
F main() -> i64 {
    c := Green
    M c {
        Red => 1,
        Green => 42,
        Blue => 3
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_third_variant() {
    let source = r#"
E Color { Red, Green, Blue }
F main() -> i64 {
    c := Blue
    M c {
        Red => 1,
        Green => 2,
        Blue => 42
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_wildcard_match() {
    let source = r#"
E Dir { North, South, East, West }
F main() -> i64 {
    d := West
    M d {
        North => 1,
        _ => 42
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Enum with Data ====================

#[test]
fn e2e_enum_with_i64_data() {
    let source = r#"
E Value { Num(i64), Empty }
F main() -> i64 {
    v := Num(42)
    M v {
        Num(n) => n,
        Empty => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_empty_variant_match() {
    let source = r#"
E Value { Num(i64), Empty }
F main() -> i64 {
    v := Empty
    M v {
        Num(n) => n,
        Empty => 42
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Enum in Functions ====================

#[test]
fn e2e_enum_as_parameter() {
    let source = r#"
E Shape { Circle, Square, Triangle }
F sides(s: Shape) -> i64 {
    M s {
        Circle => 0,
        Square => 4,
        Triangle => 3
    }
}
F main() -> i64 = sides(Square) * 10 + 2
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_returned_from_function() {
    let source = r#"
E Outcome { Good(i64), Bad }
F safe_div(a: i64, b: i64) -> Outcome {
    I b == 0 { R Bad }
    R Good(a / b)
}
F main() -> i64 {
    r := safe_div(84, 2)
    M r {
        Good(v) => v,
        Bad => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Multiple Enum Usage ====================

#[test]
fn e2e_enum_loop_matching() {
    let source = r#"
E Action { Add(i64), Sub(i64), Nop }
F apply(acc: i64, action: Action) -> i64 {
    M action {
        Add(n) => acc + n,
        Sub(n) => acc - n,
        Nop => acc
    }
}
F main() -> i64 {
    result := mut 0
    result = apply(result, Add(50))
    result = apply(result, Sub(10))
    result = apply(result, Nop)
    result = apply(result, Add(2))
    result
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_two_data_variants() {
    let source = r#"
E Op { Inc, Dec, Set(i64) }
F exec(val: i64, op: Op) -> i64 {
    M op {
        Inc => val + 1,
        Dec => val - 1,
        Set(n) => n
    }
}
F main() -> i64 {
    v := mut 0
    v = exec(v, Set(40))
    v = exec(v, Inc)
    v = exec(v, Inc)
    v
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Bool-like Enums ====================

#[test]
fn e2e_enum_bool_like() {
    let source = r#"
E MyBool { Yes, No }
F to_int(b: MyBool) -> i64 {
    M b {
        Yes => 1,
        No => 0
    }
}
F main() -> i64 {
    a := to_int(Yes)
    b := to_int(No)
    a * 42 + b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_four_variants() {
    let source = r#"
E Season { Spring, Summer, Autumn, Winter }
F temp(s: Season) -> i64 {
    M s {
        Spring => 15,
        Summer => 30,
        Autumn => 10,
        Winter => 0
    }
}
F main() -> i64 = temp(Spring) + temp(Autumn) + 17
"#;
    assert_exit_code(source, 42);
}

// ==================== Complex Enum Patterns ====================

#[test]
fn e2e_enum_chain_matching() {
    let source = r#"
E Token { Num(i64), Plus, Minus }
F eval_simple(a: Token, op: Token, b: Token) -> i64 {
    lhs := M a { Num(n) => n, _ => 0 }
    rhs := M b { Num(n) => n, _ => 0 }
    M op {
        Plus => lhs + rhs,
        Minus => lhs - rhs,
        _ => 0
    }
}
F main() -> i64 = eval_simple(Num(20), Plus, Num(22))
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_nested_enum() {
    let source = r#"
E Wrap { Val(i64), None }
F main() -> i64 {
    v := Val(42)
    M v {
        Val(n) => n,
        None => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_five_variants() {
    let source = r#"
E Digit { Zero, One, Two, Three, Four }
F to_val(d: Digit) -> i64 {
    M d {
        Zero => 0,
        One => 1,
        Two => 2,
        Three => 3,
        Four => 4
    }
}
F main() -> i64 {
    a := to_val(Four)
    b := to_val(Two)
    a * 10 + b
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Enum with str Fields (Issue #67) ====================

#[test]
fn e2e_enum_str_field_select_variant() {
    // Enum variant with str field: match and extract str, use its presence to branch
    let source = r#"
E QueryType {
    Select(str),
    Insert(str),
}

F classify(q: QueryType) -> i64 {
    M q {
        Select(s) => 1,
        Insert(s) => 2
    }
}

F main() -> i64 {
    q := Select("users")
    classify(q)
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_enum_str_field_insert_variant() {
    let source = r#"
E QueryType {
    Select(str),
    Insert(str),
}

F classify(q: QueryType) -> i64 {
    M q {
        Select(s) => 1,
        Insert(s) => 2
    }
}

F main() -> i64 {
    q := Insert("data")
    classify(q)
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_enum_str_field_extract_and_print() {
    // Extract the str value from enum variant and use it with println
    let source = r#"
E Msg {
    Hello(str),
    Bye(str),
}

F main() -> i64 {
    m := Hello("world")
    M m {
        Hello(s) => {
            println("{s}")
            42
        },
        Bye(s) => {
            println("{s}")
            0
        }
    }
}
"#;
    let ir = compile_to_ir(source).expect("compile to ir");
    // Dump the main function IR for debugging
    std::fs::write("/tmp/vais_debug_enum_str.ll", &ir).ok();
    let result = compile_and_run(source).expect("compile and run");
    assert_eq!(result.exit_code, 42);
    assert!(result.stdout.contains("world"), "stdout should contain 'world', got: {}", result.stdout);
}

#[test]
fn e2e_enum_str_field_mixed_with_unit() {
    // Enum with both str-carrying and unit variants
    let source = r#"
E Event {
    Click(str),
    Hover(str),
    Close,
}

F handle(e: Event) -> i64 {
    M e {
        Click(target) => 10,
        Hover(target) => 20,
        Close => 30
    }
}

F main() -> i64 {
    a := handle(Click("button"))
    b := handle(Close)
    a + b + 2
}
"#;
    assert_exit_code(source, 42);
}
