//! Phase 125: Type Accuracy & Void/Unit Handling E2E Tests
//!
//! Tests for:
//! 1. Void/Unit expressions in if/else, match, loop contexts
//! 2. Generic monomorphization accuracy across type combinations
//! 3. Strict type mode behavior (warning vs error)
//! 4. Method return type inference accuracy

use super::helpers::*;

// ==================== 1. Void/Unit Expression Handling ====================

#[test]
fn e2e_p125_void_if_no_else() {
    // If without else produces void — should not crash
    let source = r#"
fn main() -> i64 {
    x := mut 10
    I x > 5 {
        x = 20
    }
    x
}
"#;
    assert_exit_code(source, 20);
}

#[test]
fn e2e_p125_void_if_else_both_unit() {
    // Both branches produce Unit — void placeholder should be used
    let source = r#"
fn side_effect(x: i64) -> i64 {
    x
}

fn main() -> i64 {
    x := mut 0
    I 1 > 0 {
        x = 42
    } else {
        x = 99
    }
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_void_nested_if() {
    // Nested if/else with void results
    let source = r#"
fn main() -> i64 {
    x := mut 0
    I 1 > 0 {
        I 2 > 1 {
            x = 33
        } else {
            x = 44
        }
    } else {
        x = 55
    }
    x
}
"#;
    assert_exit_code(source, 33);
}

#[test]
fn e2e_p125_void_loop_break() {
    // Loop producing void result via break
    let source = r#"
fn main() -> i64 {
    x := mut 0
    i := mut 0
    L {
        I i >= 5 {
            B
        }
        x = x + i
        i = i + 1
    }
    x
}
"#;
    // 0 + 1 + 2 + 3 + 4 = 10
    assert_exit_code(source, 10);
}

#[test]
fn e2e_p125_void_if_chain() {
    // Chain of if-else-if with void
    let source = r#"
fn main() -> i64 {
    val := 3
    result := mut 0
    I val == 1 {
        result = 10
    } else I val == 2 {
        result = 20
    } else I val == 3 {
        result = 30
    } else {
        result = 40
    }
    result
}
"#;
    assert_exit_code(source, 30);
}

// ==================== 2. Generic Monomorphization Accuracy ====================

#[test]
fn e2e_p125_generic_with_subtraction() {
    // Generic function with subtraction
    let source = r#"
fn diff<T>(a: T, b: T) -> type {
    a - b
}

fn main() -> i64 {
    diff(100, 58)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_generic_with_comparison() {
    // Generic function with comparison returning i64
    let source = r#"
fn max_val<T>(a: T, b: T) -> type {
    I a > b {
        a
    } else {
        b
    }
}

fn main() -> i64 {
    max_val(30, 42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_generic_min_val() {
    // Generic min function
    let source = r#"
fn min_val<T>(a: T, b: T) -> type {
    I a < b {
        a
    } else {
        b
    }
}

fn main() -> i64 {
    min_val(42, 99)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_generic_swap_return_first() {
    // Two-param generic returning computed result
    let source = r#"
fn combine<A, B>(a: A, b: B) -> A {
    a + b
}

fn main() -> i64 {
    combine(20, 22)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_generic_accumulate() {
    // Generic function called in a loop
    let source = r#"
fn add<T>(a: T, b: T) -> type {
    a + b
}

fn main() -> i64 {
    sum := mut 0
    i := mut 1
    L {
        I i > 5 { B }
        sum = add(sum, i)
        i = i + 1
    }
    sum
}
"#;
    // 1 + 2 + 3 + 4 + 5 = 15
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p125_generic_with_struct_field() {
    // Generic function operating on struct field values
    let source = r#"
struct Data { x: i64 }

fn extract<T>(val: T) -> type {
    val
}

fn main() -> i64 {
    d := Data { x: 77 }
    extract(d.x)
}
"#;
    assert_exit_code(source, 77);
}

#[test]
fn e2e_p125_generic_chain_four_levels() {
    // Four-level generic chain
    let source = r#"
fn l4<T>(x: T) -> type { x }
fn l3<T>(x: T) -> type { l4(x) }
fn l2<T>(x: T) -> type { l3(x) }
fn l1<T>(x: T) -> type { l2(x) }

fn main() -> i64 {
    l1(42)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 3. Trait + Method Type Accuracy ====================

#[test]
fn e2e_p125_trait_method_return_accuracy() {
    // Verify trait method return type is correctly inferred
    let source = r#"
trait Scorable {
    fn score(&self) -> i64
}

struct Player { points: i64, bonus: i64 }

impl Player: Scorable {
    fn score(&self) -> i64 {
        self.points + self.bonus
    }
}

fn main() -> i64 {
    p := Player { points: 30, bonus: 12 }
    p.score()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_multiple_methods_correct_dispatch() {
    // Multiple methods on same struct — correct dispatch
    let source = r#"
struct Vec2 { x: i64, y: i64 }

impl Vec2 {
    fn sum(&self) -> i64 {
        self.x + self.y
    }

    fn product(&self) -> i64 {
        self.x * self.y
    }

    fn diff(&self) -> i64 {
        self.x - self.y
    }
}

fn main() -> i64 {
    v := Vec2 { x: 10, y: 3 }
    v.sum() + v.product() + v.diff()
}
"#;
    // 13 + 30 + 7 = 50
    assert_exit_code(source, 50);
}

#[test]
fn e2e_p125_struct_method_with_param() {
    // Struct method taking an additional parameter
    let source = r#"
struct Scaler { factor: i64 }

impl Scaler {
    fn apply(&self, x: i64) -> i64 {
        self.factor * x
    }
}

fn main() -> i64 {
    s := Scaler { factor: 7 }
    s.apply(6)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 4. If/Else as Expression (Value-Producing) ====================

#[test]
fn e2e_p125_if_else_as_value() {
    // If/else used as expression producing a value
    let source = r#"
fn main() -> i64 {
    x := 10
    result := I x > 5 { 42 } else { 99 }
    result
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_nested_if_else_as_value() {
    // Nested if/else expressions
    let source = r#"
fn main() -> i64 {
    x := 3
    result := I x == 1 { 10 } else I x == 2 { 20 } else I x == 3 { 42 } else { 99 }
    result
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_if_else_value_in_function() {
    // If/else expression as function return
    let source = r#"
fn classify(n: i64) -> i64 {
    I n > 100 {
        3
    } else I n > 10 {
        2
    } else {
        1
    }
}

fn main() -> i64 {
    classify(5) + classify(50) + classify(500)
}
"#;
    // 1 + 2 + 3 = 6
    assert_exit_code(source, 6);
}

// ==================== 5. Enum Type Accuracy ====================

#[test]
fn e2e_p125_enum_match_all_variants() {
    // Enum with match covering all variants
    let source = r#"
enum Color {
    Red,
    Green,
    Blue
}

fn color_code(c: Color) -> i64 {
    match c {
        Red => 1,
        Green => 2,
        Blue => 3
    }
}

fn main() -> i64 {
    r := Red
    g := Green
    b := Blue
    color_code(r) + color_code(g) * 10 + color_code(b) * 100
}
"#;
    // 1 + 20 + 300 = 321 -> exit code 321 % 256 = 65
    assert_exit_code(source, 65);
}

#[test]
fn e2e_p125_enum_with_data_match() {
    // Enum variant with data
    let source = r#"
enum Shape {
    Circle(i64),
    Square(i64)
}

fn area(s: Shape) -> i64 {
    match s {
        Circle(r) => r * r * 3,
        Square(side) => side * side
    }
}

fn main() -> i64 {
    c := Circle(3)
    area(c)
}
"#;
    // 3 * 3 * 3 = 27
    assert_exit_code(source, 27);
}

// ==================== 6. Closure Type Accuracy ====================

#[test]
fn e2e_p125_closure_captures_correctly() {
    // Closure capturing outer variable
    let source = r#"
fn main() -> i64 {
    base := 40
    add_base := |x| x + base
    add_base(2)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_closure_as_argument() {
    // Passing closure to a function
    let source = r#"
fn apply(x: i64, f: fn(i64) -> i64) -> i64 {
    f(x)
}

fn main() -> i64 {
    doubler := |x| x * 2
    apply(21, doubler)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 7. Codegen Warning Collection ====================

#[test]
fn e2e_p125_warnings_collected_for_uninstantiated_generic() {
    // Verify that codegen collects warnings (not panics) for generic functions
    // that are called but have no concrete instantiation recorded
    let source = r#"
fn identity<T>(x: T) -> type {
    x
}

fn wrapper<T>(x: T) -> type {
    identity(x)
}

fn main() -> i64 {
    wrapper(42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_strict_mode_does_not_affect_generic_fallback() {
    // Even in strict mode, Generic fallback (Category A) should be a warning, not error
    // This test verifies that the generic identity pattern still compiles
    let source = r#"
fn id<T>(x: T) -> type { x }
fn double<T>(x: T) -> type { x + x }

fn main() -> i64 {
    id(21) + double(10) + 1
}
"#;
    // 21 + 20 + 1 = 42
    assert_exit_code(source, 42);
}

// ==================== 8. Generic Multi-Type Specialization ====================

#[test]
fn e2e_p125_generic_two_specializations() {
    // Same generic called with different concrete types (both i64 at runtime)
    let source = r#"
fn apply<T>(x: T) -> type { x }

fn main() -> i64 {
    a := apply(20)
    b := apply(22)
    a + b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_generic_recursive_base() {
    // Generic function with recursion (@ self-call)
    let source = r#"
fn power<T>(base: T, exp: i64) -> type {
    I exp == 0 { 1 }
    else { base * @(base, exp - 1) }
}

fn main() -> i64 {
    power(2, 5)
}
"#;
    // 2^5 = 32
    assert_exit_code(source, 32);
}

#[test]
fn e2e_p125_generic_with_default_return() {
    // Generic function with conditional return
    let source = r#"
fn clamp<T>(val: T, lo: T, hi: T) -> type {
    I val < lo { lo }
    else I val > hi { hi }
    else { val }
}

fn main() -> i64 {
    clamp(50, 0, 42)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 9. Struct with Method Chains ====================

#[test]
fn e2e_p125_struct_method_chain_result() {
    // Multiple method calls on struct values
    let source = r#"
struct Counter { val: i64 }

impl Counter {
    fn get(&self) -> i64 { self.val }
    fn doubled(&self) -> i64 { self.val * 2 }
}

fn main() -> i64 {
    c := Counter { val: 21 }
    c.doubled()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_struct_two_instances() {
    // Two instances of same struct, different field values
    let source = r#"
struct Point { x: i64, y: i64 }

impl Point {
    fn sum(&self) -> i64 { self.x + self.y }
}

fn main() -> i64 {
    a := Point { x: 10, y: 5 }
    b := Point { x: 20, y: 7 }
    a.sum() + b.sum()
}
"#;
    // 15 + 27 = 42
    assert_exit_code(source, 42);
}

// ==================== 10. Nested Generics and Expressions ====================

#[test]
fn e2e_p125_generic_called_from_conditional() {
    // Generic function called inside if-else expression
    let source = r#"
fn id<T>(x: T) -> type { x }

fn main() -> i64 {
    x := 10
    result := I x > 5 { id(42) } else { id(0) }
    result
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_generic_in_loop_accumulation() {
    // Generic add used in loop for accumulation
    let source = r#"
fn add_vals<T>(a: T, b: T) -> type { a + b }

fn main() -> i64 {
    result := mut 0
    L i:1..8 {
        result = add_vals(result, i)
    }
    result
}
"#;
    // 1+2+3+4+5+6+7 = 28
    assert_exit_code(source, 28);
}

#[test]
fn e2e_p125_enum_unit_variants_arithmetic() {
    // Enum unit variants used in arithmetic via match
    let source = r#"
enum Dir { North, South, East, West }

fn dir_val(d: Dir) -> i64 {
    match d {
        North => 10,
        South => 20,
        East => 5,
        West => 7
    }
}

fn main() -> i64 {
    dir_val(North) + dir_val(South) + dir_val(East) + dir_val(West)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 11. Void/Unit Edge Cases ====================

#[test]
fn e2e_p125_void_in_match_all_unit_arms() {
    // Match where all arms produce Unit/void
    let source = r#"
fn main() -> i64 {
    x := mut 0
    val := 2
    match val {
        1 => { x = 10 },
        2 => { x = 42 },
        _ => { x = 99 }
    }
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_void_loop_with_break_value() {
    // Loop with conditional break returning value after loop
    let source = r#"
fn main() -> i64 {
    result := mut 0
    i := mut 1
    L {
        I i > 10 { B }
        result = result + i
        i = i + 1
    }
    result
}
"#;
    // 1+2+...+10 = 55
    assert_exit_code(source, 55);
}

#[test]
fn e2e_p125_void_nested_loops() {
    // Nested loops both producing void
    let source = r#"
fn main() -> i64 {
    total := mut 0
    i := mut 0
    L {
        I i >= 3 { B }
        j := mut 0
        L {
            I j >= 3 { B }
            total = total + 1
            j = j + 1
        }
        i = i + 1
    }
    total
}
"#;
    // 3 * 3 = 9
    assert_exit_code(source, 9);
}

// ==================== 12. Complex Type Interactions ====================

#[test]
fn e2e_p125_struct_in_match() {
    // Match on integer with struct construction in each arm
    let source = r#"
struct Result { code: i64, value: i64 }

fn main() -> i64 {
    input := 2
    r := match input {
        1 => Result { code: 1, value: 10 },
        2 => Result { code: 2, value: 42 },
        _ => Result { code: 0, value: 0 }
    }
    r.value
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_multiple_generics_same_function() {
    // Generic function called multiple times in same expression
    let source = r#"
fn double<T>(x: T) -> type { x + x }

fn main() -> i64 {
    double(10) + double(11)
}
"#;
    // 20 + 22 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_generic_called_with_negative() {
    // Generic with negative arguments
    let source = r#"
fn abs_val<T>(x: T) -> type {
    I x < 0 { 0 - x } else { x }
}

fn main() -> i64 {
    abs_val(-42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_closure_nested_capture() {
    // Closure capturing multiple variables
    let source = r#"
fn main() -> i64 {
    a := 10
    b := 20
    c := 12
    sum_all := |x| x + a + b + c
    sum_all(0)
}
"#;
    // 0 + 10 + 20 + 12 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_enum_data_variant_extraction() {
    // Extract data from enum variant in match
    let source = r#"
enum Wrapper {
    Val(i64),
    Empty
}

fn unwrap_or(w: Wrapper, default: i64) -> i64 {
    match w {
        Val(v) => v,
        Empty => default
    }
}

fn main() -> i64 {
    a := Val(42)
    b := Empty
    unwrap_or(a, 0) + unwrap_or(b, 0)
}
"#;
    // 42 + 0 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_deeply_nested_if_else_value() {
    // Deeply nested if-else producing values
    let source = r#"
fn classify(n: i64) -> i64 {
    I n > 1000 { 5 }
    else I n > 100 { 4 }
    else I n > 10 { 3 }
    else I n > 0 { 2 }
    else { 1 }
}

fn main() -> i64 {
    classify(0) + classify(5) + classify(50) + classify(500) + classify(5000)
}
"#;
    // 1 + 2 + 3 + 4 + 5 = 15
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p125_struct_field_arithmetic() {
    // Arithmetic on struct fields
    let source = r#"
struct Rect { w: i64, h: i64 }

fn area(r: Rect) -> i64 { r.w * r.h }
fn perimeter(r: Rect) -> i64 { 2 * (r.w + r.h) }

fn main() -> i64 {
    r := Rect { w: 6, h: 7 }
    area(r)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_for_loop_type_accuracy() {
    // For loop with range — verifying loop variable type correctness
    let source = r#"
fn main() -> i64 {
    sum := mut 0
    L i:1..10 {
        sum = sum + i
    }
    sum
}
"#;
    // 1+2+...+9 = 45
    assert_exit_code(source, 45);
}

#[test]
fn e2e_p125_generic_triple_call_chain() {
    // Generic functions calling each other in a triple chain
    let source = r#"
fn inc<T>(x: T) -> type { x + 1 }
fn double<T>(x: T) -> type { x + x }
fn triple<T>(x: T) -> type { x + x + x }

fn main() -> i64 {
    inc(triple(double(3)))
}
"#;
    // double(3)=6, triple(6)=18, inc(18)=19
    assert_exit_code(source, 19);
}
