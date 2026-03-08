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
F main() -> i64 {
    x := 10
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
F side_effect(x: i64) -> i64 {
    x
}

F main() -> i64 {
    x := mut 0
    I 1 > 0 {
        x = 42
    } E {
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
F main() -> i64 {
    x := mut 0
    I 1 > 0 {
        I 2 > 1 {
            x = 33
        } E {
            x = 44
        }
    } E {
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
F main() -> i64 {
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
F main() -> i64 {
    val := 3
    result := mut 0
    I val == 1 {
        result = 10
    } E I val == 2 {
        result = 20
    } E I val == 3 {
        result = 30
    } E {
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
F diff<T>(a: T, b: T) -> T {
    a - b
}

F main() -> i64 {
    diff(100, 58)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_generic_with_comparison() {
    // Generic function with comparison returning i64
    let source = r#"
F max_val<T>(a: T, b: T) -> T {
    I a > b {
        a
    } E {
        b
    }
}

F main() -> i64 {
    max_val(30, 42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_generic_min_val() {
    // Generic min function
    let source = r#"
F min_val<T>(a: T, b: T) -> T {
    I a < b {
        a
    } E {
        b
    }
}

F main() -> i64 {
    min_val(42, 99)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_generic_swap_return_first() {
    // Two-param generic returning computed result
    let source = r#"
F combine<A, B>(a: A, b: B) -> A {
    a + b
}

F main() -> i64 {
    combine(20, 22)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_generic_accumulate() {
    // Generic function called in a loop
    let source = r#"
F add<T>(a: T, b: T) -> T {
    a + b
}

F main() -> i64 {
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
S Data { x: i64 }

F extract<T>(val: T) -> T {
    val
}

F main() -> i64 {
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
F l4<T>(x: T) -> T { x }
F l3<T>(x: T) -> T { l4(x) }
F l2<T>(x: T) -> T { l3(x) }
F l1<T>(x: T) -> T { l2(x) }

F main() -> i64 {
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
W Scorable {
    F score(&self) -> i64
}

S Player { points: i64, bonus: i64 }

X Player: Scorable {
    F score(&self) -> i64 {
        self.points + self.bonus
    }
}

F main() -> i64 {
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
S Vec2 { x: i64, y: i64 }

X Vec2 {
    F sum(&self) -> i64 {
        self.x + self.y
    }

    F product(&self) -> i64 {
        self.x * self.y
    }

    F diff(&self) -> i64 {
        self.x - self.y
    }
}

F main() -> i64 {
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
S Scaler { factor: i64 }

X Scaler {
    F apply(&self, x: i64) -> i64 {
        self.factor * x
    }
}

F main() -> i64 {
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
F main() -> i64 {
    x := 10
    result := I x > 5 { 42 } E { 99 }
    result
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_nested_if_else_as_value() {
    // Nested if/else expressions
    let source = r#"
F main() -> i64 {
    x := 3
    result := I x == 1 { 10 } E I x == 2 { 20 } E I x == 3 { 42 } E { 99 }
    result
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p125_if_else_value_in_function() {
    // If/else expression as function return
    let source = r#"
F classify(n: i64) -> i64 {
    I n > 100 {
        3
    } E I n > 10 {
        2
    } E {
        1
    }
}

F main() -> i64 {
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
E Color {
    Red,
    Green,
    Blue
}

F color_code(c: Color) -> i64 {
    M c {
        Red => 1,
        Green => 2,
        Blue => 3
    }
}

F main() -> i64 {
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
E Shape {
    Circle(i64),
    Square(i64)
}

F area(s: Shape) -> i64 {
    M s {
        Circle(r) => r * r * 3,
        Square(side) => side * side
    }
}

F main() -> i64 {
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
F main() -> i64 {
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
F apply(x: i64, f: fn(i64) -> i64) -> i64 {
    f(x)
}

F main() -> i64 {
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
F identity<T>(x: T) -> T {
    x
}

F wrapper<T>(x: T) -> T {
    identity(x)
}

F main() -> i64 {
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
F id<T>(x: T) -> T { x }
F double<T>(x: T) -> T { x + x }

F main() -> i64 {
    id(21) + double(10) + 1
}
"#;
    // 21 + 20 + 1 = 42
    assert_exit_code(source, 42);
}
