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
E Expr {
    Lit(i64),
    Add(i64, i64),
    Mul(i64, i64)
}
F eval(e: Expr) -> i64 {
    M e {
        Lit(n) => n,
        Add(a, b) => a + b,
        Mul(a, b) => a * b
    }
}
F main() -> i64 {
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
E Op {
    Lit(i64),
    Add(i64, i64),
    Sub(i64, i64)
}
F eval(op: Op) -> i64 {
    M op {
        Lit(n) => n,
        Add(a, b) => a + b,
        Sub(a, b) => a - b
    }
}
F main() -> i64 {
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
F day_code(d: i64) -> i64 {
    M d {
        1 => 10,
        2 => 20,
        3 => 30,
        4 => 40,
        5 => 50,
        _ => 0
    }
}
F main() -> i64 { day_code(4) }
"#;
    assert_exit_code(source, 40);
}

// ==================== 4. Match with guard on variable ====================

#[test]
fn e2e_p47_match_guard_variable() {
    // Guard expression referencing matched variable
    let source = r#"
F grade(score: i64) -> i64 {
    M score {
        x I x >= 90 => 5,
        x I x >= 80 => 4,
        x I x >= 70 => 3,
        x I x >= 60 => 2,
        _ => 1
    }
}
F main() -> i64 { grade(85) }
"#;
    assert_exit_code(source, 4);
}

// ==================== 5. Match with block arm bodies ====================

#[test]
fn e2e_p47_match_block_arm_bodies() {
    // Match arms with block bodies that compute values
    let source = r#"
F check(n: i64) -> i64 {
    M n {
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
F main() -> i64 { check(0) }
"#;
    assert_exit_code(source, 100);
}

// ==================== 6. Match result assigned to variable ====================

#[test]
fn e2e_p47_match_result_assigned() {
    // Match expression result stored in variable
    let source = r#"
F main() -> i64 {
    x := 3
    label := M x {
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
F process(n: i64) -> i64 {
    M n {
        x I x > 0 => {
            I x > 50 { 3 } E { 2 }
        },
        _ => 1
    }
}
F main() -> i64 { process(25) }
"#;
    assert_exit_code(source, 2);
}

// ==================== 8. Match on function call result ====================

#[test]
fn e2e_p47_match_on_fn_result() {
    // Match applied to a function's return value
    let source = r#"
F compute(a: i64, b: i64) -> i64 { a + b }
F main() -> i64 {
    M compute(3, 7) {
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
E Season {
    Spring,
    Summer,
    Autumn,
    Winter
}
F temp(s: Season) -> i64 {
    M s {
        Spring => 15,
        Summer => 30,
        Autumn => 10,
        Winter => 0
    }
}
F main() -> i64 {
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
F square(x: i64) -> i64 { x * x }
S Num { val: i64 }
X Num {
    F squared(self) -> i64 { square(self.val) }
}
F main() -> i64 {
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
S Acc { base: i64 }
X Acc {
    F add3(self, a: i64, b: i64, c: i64) -> i64 {
        self.base + a + b + c
    }
}
F main() -> i64 {
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
E Item {
    Priced(i64),
    Free
}
F cost(item: Item) -> i64 {
    M item {
        Priced(p) => p * 2,
        Free => 0
    }
}
F main() -> i64 { cost(Priced(25)) }
"#;
    // 25 * 2 = 50
    assert_exit_code(source, 50);
}

// ==================== 13. Match wildcard with computation ====================

#[test]
fn e2e_p47_match_wildcard_computation() {
    // Wildcard arm does computation rather than constant
    let source = r#"
F transform(n: i64) -> i64 {
    M n {
        0 => 100,
        1 => 50,
        x => x * 3
    }
}
F main() -> i64 { transform(9) }
"#;
    // 9 * 3 = 27
    assert_exit_code(source, 27);
}

// ==================== 14. Enum method: is_some pattern ====================

#[test]
fn e2e_p47_enum_is_some_pattern() {
    // Enum method that checks if a variant has data
    let source = r#"
E Maybe {
    Just(i64),
    Empty
}
X Maybe {
    F is_just(self) -> i64 {
        M self {
            Just(_) => 1,
            Empty => 0
        }
    }
}
F main() -> i64 {
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
S Limit { max: i64 }
X Limit {
    F clamp(self, val: i64) -> i64 {
        I val > self.max { self.max } E { val }
    }
}
F main() -> i64 {
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
F classify(n: i64) -> i64 {
    M n {
        x I x > 3 => 10,
        _ => 1
    }
}
F main() -> i64 {
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
E Pair {
    Both(i64, i64),
    Single(i64)
}
F sum_pair(p: Pair) -> i64 {
    M p {
        Both(a, b) => a + b,
        Single(x) => x
    }
}
F main() -> i64 { sum_pair(Both(17, 28)) }
"#;
    // 17 + 28 = 45
    assert_exit_code(source, 45);
}

// ==================== 18. Multiple structs with methods ====================

#[test]
fn e2e_p47_multiple_structs_methods() {
    // Two different structs, each with methods
    let source = r#"
S Foo { x: i64 }
S Bar { y: i64 }
X Foo {
    F val(self) -> i64 { self.x }
}
X Bar {
    F val(self) -> i64 { self.y * 2 }
}
F main() -> i64 {
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
F pick(n: i64) -> i64 {
    M n {
        0 => 10,
        1 => 20,
        2 => 30,
        3 => 40,
        _ => 50
    }
}
F main() -> i64 { pick(2) + pick(99) }
"#;
    // pick(2)=30, pick(99)=50, 30+50=80
    assert_exit_code(source, 80);
}

// ==================== 20. Struct field update and method ====================

#[test]
fn e2e_p47_struct_field_update_method() {
    // Mutable struct field updated, then method called
    let source = r#"
S Count { n: i64 }
X Count {
    F get(self) -> i64 { self.n }
}
F main() -> i64 {
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
E Light {
    Red,
    Yellow,
    Green
}
F should_stop(l: Light) -> i64 {
    M l {
        Red | Yellow => 1,
        Green => 0
    }
}
F main() -> i64 {
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
F main() -> i64 {
    x := 10
    M x > 5 {
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
S Range { n: i64 }
X Range {
    F sum_to(self) -> i64 {
        total := mut 0
        L i:1..self.n+1 {
            total = total + i
        }
        total
    }
}
F main() -> i64 {
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
E Cmd {
    Inc(i64),
    Dec(i64),
    Reset
}
F apply(cmd: Cmd, val: i64) -> i64 {
    M cmd {
        Inc(n) => val + n,
        Dec(n) => val - n,
        Reset => 0
    }
}
F main() -> i64 {
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
F main() -> i64 {
    a := M 3 {
        1 => 10,
        2 => 20,
        3 => 30,
        _ => 0
    }
    b := M 5 {
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
S Selector { choice: i64 }
X Selector {
    F select(self) -> i64 {
        M self.choice {
            1 => 100,
            2 => 200,
            _ => 0
        }
    }
}
F main() -> i64 {
    s := Selector { choice: 2 }
    s.select()
}
"#;
    assert_exit_code(source, 200);
}
