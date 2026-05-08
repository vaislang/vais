//! Phase 47 — Struct/Enum methods and nested match E2E tests
//!
//! Tests covering:
//! - Struct methods with multiple parameters and return computations
//! - Enum methods via impl
//! - Nested match on enum variants
//! - Match on computed values and struct fields
//! - Match inside loops and functions
//! - Multi-arm match with guards
//! - Enum with multiple data variants

use super::helpers::*;

// ==================== 1. Enum with three data variants ====================

#[test]
fn e2e_p47_enum_three_data_variants() {
    // Enum with three data-carrying variants
    let source = r#"
enum Expr {
    Lit(i64),
    Add(i64, i64),
    Mul(i64, i64)
}
fn eval(e: Expr) -> i64 {
    match e {
        Lit(n) => n,
        Add(a, b) => a + b,
        Mul(a, b) => a * b
    }
}
fn main() -> i64 {
    eval(Mul(6, 7))
}
"#;
    // 6 * 7 = 42
    assert_exit_code(source, 42);
}

// ==================== 2. Enum: match Add variant ====================

#[test]
fn e2e_p47_enum_match_add_variant() {
    let source = r#"
enum Op {
    Lit(i64),
    Add(i64, i64),
    Sub(i64, i64)
}
fn eval(op: Op) -> i64 {
    match op {
        Lit(n) => n,
        Add(a, b) => a + b,
        Sub(a, b) => a - b
    }
}
fn main() -> i64 {
    eval(Add(10, 25))
}
"#;
    // 10 + 25 = 35
    assert_exit_code(source, 35);
}

// ==================== 3. Match with multiple literal arms ====================

#[test]
fn e2e_p47_match_many_literals() {
    // Match with 6 literal arms
    let source = r#"
fn day_code(d: i64) -> i64 {
    match d {
        1 => 10,
        2 => 20,
        3 => 30,
        4 => 40,
        5 => 50,
        _ => 0
    }
}
fn main() -> i64 { day_code(4) }
"#;
    assert_exit_code(source, 40);
}

// ==================== 4. Match with guard on variable ====================

#[test]
fn e2e_p47_match_guard_variable() {
    // Guard expression referencing matched variable
    let source = r#"
fn grade(score: i64) -> i64 {
    match score {
        x I x >= 90 => 5,
        x I x >= 80 => 4,
        x I x >= 70 => 3,
        x I x >= 60 => 2,
        _ => 1
    }
}
fn main() -> i64 { grade(85) }
"#;
    assert_exit_code(source, 4);
}

// ==================== 5. Match with block arm bodies ====================

#[test]
fn e2e_p47_match_block_arm_bodies() {
    // Match arms with block bodies that compute values
    let source = r#"
fn check(n: i64) -> i64 {
    match n {
        0 => {
            x := 50
            x * 2
        },
        1 => {
            y := 100
            y + 50
        },
        _ => 0
    }
}
fn main() -> i64 { check(0) }
"#;
    assert_exit_code(source, 100);
}

// ==================== 6. Match result assigned to variable ====================

#[test]
fn e2e_p47_match_result_assigned() {
    // Match expression result stored in variable
    let source = r#"
fn main() -> i64 {
    x := 3
    label := match x {
        1 => 10,
        2 => 20,
        3 => 30,
        _ => 0
    }
    label + 7
}
"#;
    // 30 + 7 = 37
    assert_exit_code(source, 37);
}

// ==================== 7. Nested if inside match arm ====================

#[test]
fn e2e_p47_nested_if_in_match() {
    // If-else inside a match arm body
    let source = r#"
fn process(n: i64) -> i64 {
    match n {
        x I x > 0 => {
            I x > 50 { 3 } else { 2 }
        },
        _ => 1
    }
}
fn main() -> i64 { process(25) }
"#;
    assert_exit_code(source, 2);
}

// ==================== 8. Match on function call result ====================

#[test]
fn e2e_p47_match_on_fn_result() {
    // Match applied to a function's return value
    let source = r#"
fn compute(a: i64, b: i64) -> i64 { a + b }
fn main() -> i64 {
    match compute(3, 7) {
        10 => 1,
        20 => 2,
        _ => 0
    }
}
"#;
    // compute(3,7) = 10, matches first arm
    assert_exit_code(source, 1);
}

// ==================== 9. Enum unit variants: all arms ====================

#[test]
fn e2e_p47_enum_all_unit_arms() {
    // Exercise every arm of a 4-variant enum
    let source = r#"
enum Season {
    Spring,
    Summer,
    Autumn,
    Winter
}
fn temp(s: Season) -> i64 {
    match s {
        Spring => 15,
        Summer => 30,
        Autumn => 10,
        Winter => 0
    }
}
fn main() -> i64 {
    temp(Summer) + temp(Winter)
}
"#;
    // 30 + 0 = 30
    assert_exit_code(source, 30);
}

// ==================== 10. Struct method calling free function ====================

#[test]
fn e2e_p47_struct_method_calls_fn() {
    // Method body calls a free (non-method) function
    let source = r#"
fn square(x: i64) -> i64 { x * x }
struct Num { val: i64 }
impl Num {
    fn squared(self) -> i64 { square(self.val) }
}
fn main() -> i64 {
    n := Num { val: 8 }
    n.squared()
}
"#;
    // 8 * 8 = 64
    assert_exit_code(source, 64);
}

// ==================== 11. Struct method with 3 params ====================

#[test]
fn e2e_p47_struct_method_three_params() {
    // Method with three extra parameters beyond self
    let source = r#"
struct Acc { base: i64 }
impl Acc {
    fn add3(self, a: i64, b: i64, c: i64) -> i64 {
        self.base + a + b + c
    }
}
fn main() -> i64 {
    a := Acc { base: 10 }
    a.add3(1, 2, 3)
}
"#;
    // 10 + 1 + 2 + 3 = 16
    assert_exit_code(source, 16);
}

// ==================== 12. Enum data variant: arithmetic on extracted data ====================

#[test]
fn e2e_p47_enum_data_arithmetic() {
    // Arithmetic performed on data extracted from enum variant
    let source = r#"
enum Item {
    Priced(i64),
    Free
}
fn cost(item: Item) -> i64 {
    match item {
        Priced(p) => p * 2,
        Free => 0
    }
}
fn main() -> i64 { cost(Priced(25)) }
"#;
    // 25 * 2 = 50
    assert_exit_code(source, 50);
}

// ==================== 13. Match wildcard with computation ====================

#[test]
fn e2e_p47_match_wildcard_computation() {
    // Wildcard arm does computation rather than constant
    let source = r#"
fn transform(n: i64) -> i64 {
    match n {
        0 => 100,
        1 => 50,
        x => x * 3
    }
}
fn main() -> i64 { transform(9) }
"#;
    // 9 * 3 = 27
    assert_exit_code(source, 27);
}

// ==================== 14. Enum method: is_some pattern ====================

#[test]
fn e2e_p47_enum_is_some_pattern() {
    // Enum method that checks if a variant has data
    let source = r#"
enum Maybe {
    Just(i64),
    Empty
}
impl Maybe {
    fn is_just(self) -> i64 {
        match self {
            Just(_) => 1,
            Empty => 0
        }
    }
}
fn main() -> i64 {
    a := Just(42)
    b := Empty
    a.is_just() + b.is_just()
}
"#;
    // 1 + 0 = 1
    assert_exit_code(source, 1);
}

// ==================== 15. Struct impl: method returns conditional ====================

#[test]
fn e2e_p47_struct_method_returns_conditional() {
    // Method returns result of if-else
    let source = r#"
struct Limit { max: i64 }
impl Limit {
    fn clamp(self, val: i64) -> i64 {
        I val > self.max { self.max } else { val }
    }
}
fn main() -> i64 {
    lim := Limit { max: 50 }
    lim.clamp(30) + lim.clamp(100)
}
"#;
    // clamp(30)=30, clamp(100)=50, 30+50=80
    assert_exit_code(source, 80);
}

// ==================== 16. Match inside loop body ====================

#[test]
fn e2e_p47_match_in_loop() {
    // Match expression evaluated each iteration
    let source = r#"
fn classify(n: i64) -> i64 {
    match n {
        x I x > 3 => 10,
        _ => 1
    }
}
fn main() -> i64 {
    total := mut 0
    L i:0..6 {
        total = total + classify(i)
    }
    total
}
"#;
    // i=0:1, 1:1, 2:1, 3:1, 4:10, 5:10 => 1+1+1+1+10+10 = 24
    assert_exit_code(source, 24);
}

// ==================== 17. Enum two-field data match ====================

#[test]
fn e2e_p47_enum_two_field_match() {
    // Enum variant with two fields, both destructured
    let source = r#"
enum Pair {
    Both(i64, i64),
    Single(i64)
}
fn sum_pair(p: Pair) -> i64 {
    match p {
        Both(a, b) => a + b,
        Single(x) => x
    }
}
fn main() -> i64 { sum_pair(Both(17, 28)) }
"#;
    // 17 + 28 = 45
    assert_exit_code(source, 45);
}

// ==================== 18. Multiple structs with methods ====================

#[test]
fn e2e_p47_multiple_structs_methods() {
    // Two different structs, each with methods
    let source = r#"
struct Foo { x: i64 }
struct Bar { y: i64 }
impl Foo {
    fn val(self) -> i64 { self.x }
}
impl Bar {
    fn val(self) -> i64 { self.y * 2 }
}
fn main() -> i64 {
    f := Foo { x: 5 }
    b := Bar { y: 10 }
    f.val() + b.val()
}
"#;
    // 5 + 10*2 = 25
    assert_exit_code(source, 25);
}

// ==================== 19. Match returning from each arm consistently ====================

#[test]
fn e2e_p47_match_all_arms_return() {
    // Verify all match arms return consistent types
    let source = r#"
fn pick(n: i64) -> i64 {
    match n {
        0 => 10,
        1 => 20,
        2 => 30,
        3 => 40,
        _ => 50
    }
}
fn main() -> i64 { pick(2) + pick(99) }
"#;
    // pick(2)=30, pick(99)=50, 30+50=80
    assert_exit_code(source, 80);
}

// ==================== 20. Struct field update and method ====================

#[test]
fn e2e_p47_struct_field_update_method() {
    // Mutable struct field updated, then method called
    let source = r#"
struct Count { n: i64 }
impl Count {
    fn get(self) -> i64 { self.n }
}
fn main() -> i64 {
    c := mut Count { n: 0 }
    c.n = 42
    c.get()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 21. Enum: match with or-pattern ====================

#[test]
fn e2e_p47_enum_match_or_pattern() {
    // Or-pattern matching multiple enum variants
    let source = r#"
enum Light {
    Red,
    Yellow,
    Green
}
fn should_stop(l: Light) -> i64 {
    match l {
        Red | Yellow => 1,
        Green => 0
    }
}
fn main() -> i64 {
    should_stop(Yellow) + should_stop(Green)
}
"#;
    // Yellow->1, Green->0, 1+0=1
    assert_exit_code(source, 1);
}

// ==================== 22. Match on boolean value ====================

#[test]
fn e2e_p47_match_on_bool() {
    // Match on a boolean expression
    let source = r#"
fn main() -> i64 {
    x := 10
    match x > 5 {
        true => 1,
        false => 0
    }
}
"#;
    assert_exit_code(source, 1);
}

// ==================== 23. Struct method with loop inside ====================

#[test]
fn e2e_p47_struct_method_with_loop() {
    // Method body contains a loop
    let source = r#"
struct Range { n: i64 }
impl Range {
    fn sum_to(self) -> i64 {
        total := mut 0
        L i:1..self.n+1 {
            total = total + i
        }
        total
    }
}
fn main() -> i64 {
    r := Range { n: 5 }
    r.sum_to()
}
"#;
    // 1+2+3+4+5 = 15
    assert_exit_code(source, 15);
}

// ==================== 24. Enum variant passed to function ====================

#[test]
fn e2e_p47_enum_variant_to_fn() {
    // Enum variant constructed inline and passed to function
    let source = r#"
enum Cmd {
    Inc(i64),
    Dec(i64),
    Reset
}
fn apply(cmd: Cmd, val: i64) -> i64 {
    match cmd {
        Inc(n) => val + n,
        Dec(n) => val - n,
        Reset => 0
    }
}
fn main() -> i64 {
    v := apply(Inc(5), 10)
    apply(Dec(3), v)
}
"#;
    // apply(Inc(5), 10) = 15, apply(Dec(3), 15) = 12
    assert_exit_code(source, 12);
}

// ==================== 25. Multiple match expressions in sequence ====================

#[test]
fn e2e_p47_multiple_matches() {
    // Two match expressions computed sequentially
    let source = r#"
fn main() -> i64 {
    a := match 3 {
        1 => 10,
        2 => 20,
        3 => 30,
        _ => 0
    }
    b := match 5 {
        x I x > 4 => 50,
        _ => 0
    }
    a + b
}
"#;
    // 30 + 50 = 80
    assert_exit_code(source, 80);
}

// ==================== 26. Struct method returning match ====================

#[test]
fn e2e_p47_struct_method_returns_match() {
    // Method body is a match expression
    let source = r#"
struct Selector { choice: i64 }
impl Selector {
    fn select(self) -> i64 {
        match self.choice {
            1 => 100,
            2 => 200,
            _ => 0
        }
    }
}
fn main() -> i64 {
    s := Selector { choice: 2 }
    s.select()
}
"#;
    assert_exit_code(source, 200);
}
