//! Phase 90 -- Enum Patterns and Matching
//!
//! Tests for enum definitions, variant construction, pattern matching,
//! and enum-based logic.

use super::helpers::*;

// ==================== Basic Enum ====================

#[test]
fn e2e_enum_simple_variant() {
    let source = r#"
enum Color { Red, Green, Blue }
fn main() -> i64 {
    c := Red
    match c {
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
enum Color { Red, Green, Blue }
fn main() -> i64 {
    c := Green
    match c {
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
enum Color { Red, Green, Blue }
fn main() -> i64 {
    c := Blue
    match c {
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
enum Dir { North, South, East, West }
fn main() -> i64 {
    d := West
    match d {
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
enum Value { Num(i64), Empty }
fn main() -> i64 {
    v := Num(42)
    match v {
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
enum Value { Num(i64), Empty }
fn main() -> i64 {
    v := Empty
    match v {
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
enum Shape { Circle, Square, Triangle }
fn sides(s: Shape) -> i64 {
    match s {
        Circle => 0,
        Square => 4,
        Triangle => 3
    }
}
fn main() -> i64 = sides(Square) * 10 + 2
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_returned_from_function() {
    let source = r#"
enum Outcome { Good(i64), Bad }
fn safe_div(a: i64, b: i64) -> Outcome {
    I b == 0 { return Bad }
    return Good(a / b)
}
fn main() -> i64 {
    r := safe_div(84, 2)
    match r {
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
enum Action { Add(i64), Sub(i64), Nop }
fn apply(acc: i64, action: Action) -> i64 {
    match action {
        Add(n) => acc + n,
        Sub(n) => acc - n,
        Nop => acc
    }
}
fn main() -> i64 {
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
enum Op { Inc, Dec, Set(i64) }
fn exec(val: i64, op: Op) -> i64 {
    match op {
        Inc => val + 1,
        Dec => val - 1,
        Set(n) => n
    }
}
fn main() -> i64 {
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
enum MyBool { Yes, No }
fn to_int(b: MyBool) -> i64 {
    match b {
        Yes => 1,
        No => 0
    }
}
fn main() -> i64 {
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
enum Season { Spring, Summer, Autumn, Winter }
fn temp(s: Season) -> i64 {
    match s {
        Spring => 15,
        Summer => 30,
        Autumn => 10,
        Winter => 0
    }
}
fn main() -> i64 = temp(Spring) + temp(Autumn) + 17
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_equality_uses_tag_not_address() {
    let source = r#"
enum State { Clean, Dirty }

struct Frame {
    state: State,
}

impl Frame {
    fn is_dirty(self) -> bool {
        self.state == State.Dirty
    }
}

fn main() -> i64 {
    dirty := Frame { state: State.Dirty }
    clean := Frame { state: State.Clean }
    I dirty.is_dirty() && !clean.is_dirty() { 42 } else { 1 }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Complex Enum Patterns ====================

#[test]
fn e2e_enum_chain_matching() {
    let source = r#"
enum Token { Num(i64), Plus, Minus }
fn eval_simple(a: Token, op: Token, b: Token) -> i64 {
    lhs := match a { Num(n) => n, _ => 0 }
    rhs := match b { Num(n) => n, _ => 0 }
    match op {
        Plus => lhs + rhs,
        Minus => lhs - rhs,
        _ => 0
    }
}
fn main() -> i64 = eval_simple(Num(20), Plus, Num(22))
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_nested_enum() {
    let source = r#"
enum Wrap { Val(i64), None }
fn main() -> i64 {
    v := Val(42)
    match v {
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
enum Digit { Zero, One, Two, Three, Four }
fn to_val(d: Digit) -> i64 {
    match d {
        Zero => 0,
        One => 1,
        Two => 2,
        Three => 3,
        Four => 4
    }
}
fn main() -> i64 {
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
enum QueryType {
    Select(str),
    Insert(str),
}

fn classify(q: QueryType) -> i64 {
    match q {
        Select(s) => 1,
        Insert(s) => 2
    }
}

fn main() -> i64 {
    q := Select("users")
    classify(q)
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_enum_str_field_insert_variant() {
    let source = r#"
enum QueryType {
    Select(str),
    Insert(str),
}

fn classify(q: QueryType) -> i64 {
    match q {
        Select(s) => 1,
        Insert(s) => 2
    }
}

fn main() -> i64 {
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
enum Msg {
    Hello(str),
    Bye(str),
}

fn main() -> i64 {
    m := Hello("world")
    match m {
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
    assert!(
        result.stdout.contains("world"),
        "stdout should contain 'world', got: {}",
        result.stdout
    );
}

#[test]
fn e2e_enum_str_field_mixed_with_unit() {
    // Enum with both str-carrying and unit variants
    let source = r#"
enum Event {
    Click(str),
    Hover(str),
    Close,
}

fn handle(e: Event) -> i64 {
    match e {
        Click(target) => 10,
        Hover(target) => 20,
        Close => 30
    }
}

fn main() -> i64 {
    a := handle(Click("button"))
    b := handle(Close)
    a + b + 2
}
"#;
    assert_exit_code(source, 42);
}
