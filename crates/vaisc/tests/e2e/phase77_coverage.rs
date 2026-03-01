//! Phase 77: Coverage-focused E2E tests
//!
//! These tests specifically target uncovered codegen, parser, and type checker paths
//! to increase overall coverage from ~68% toward 75%.

use super::helpers::{compile_and_run, compile_to_ir};

fn assert_exit_code(source: &str, expected: i32) {
    let result = compile_and_run(source).unwrap_or_else(|e| panic!("Compile/run failed: {}", e));
    assert_eq!(
        result.exit_code, expected,
        "Exit code mismatch. stdout: {}, stderr: {}",
        result.stdout, result.stderr
    );
}

fn assert_compiles(source: &str) {
    compile_to_ir(source).unwrap_or_else(|e| panic!("Compile failed: {}", e));
}

// ============================================================================
// Compound assignment operators (codegen paths)
// ============================================================================

#[test]
fn e2e_p77_plus_assign() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := mut 10
            x += 5
            x
        }
    "#,
        15,
    );
}

#[test]
fn e2e_p77_minus_assign() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := mut 20
            x -= 8
            x
        }
    "#,
        12,
    );
}

#[test]
fn e2e_p77_mul_assign() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := mut 6
            x *= 7
            x
        }
    "#,
        42,
    );
}

#[test]
fn e2e_p77_div_assign() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := mut 100
            x /= 4
            x
        }
    "#,
        25,
    );
}

#[test]
fn e2e_p77_mod_assign() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := mut 17
            x %= 5
            x
        }
    "#,
        2,
    );
}

#[test]
fn e2e_p77_bitwise_and_assign() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := mut 255
            x &= 15
            x
        }
    "#,
        15,
    );
}

#[test]
fn e2e_p77_bitwise_or_assign() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := mut 48
            x |= 3
            x
        }
    "#,
        51,
    );
}

#[test]
fn e2e_p77_bitwise_xor_assign() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := mut 255
            x ^= 240
            x
        }
    "#,
        15,
    );
}

#[test]
fn e2e_p77_shl_assign() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := mut 1
            x <<= 5
            x
        }
    "#,
        32,
    );
}

#[test]
fn e2e_p77_shr_assign() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := mut 128
            x >>= 3
            x
        }
    "#,
        16,
    );
}

// ============================================================================
// Bitwise operators (binary ops codegen)
// ============================================================================

#[test]
fn e2e_p77_bitwise_and() {
    assert_exit_code(
        r#"
        F main() -> i64 = 170 & 15
    "#,
        10,
    );
}

#[test]
fn e2e_p77_bitwise_or() {
    assert_exit_code(
        r#"
        F main() -> i64 = 48 | 3
    "#,
        51,
    );
}

#[test]
fn e2e_p77_bitwise_xor() {
    assert_exit_code(
        r#"
        F main() -> i64 = 85 ^ 34
    "#,
        119,
    );
}

#[test]
fn e2e_p77_shift_left() {
    assert_exit_code(
        r#"
        F main() -> i64 = 3 << 4
    "#,
        48,
    );
}

#[test]
fn e2e_p77_shift_right() {
    assert_exit_code(
        r#"
        F main() -> i64 = 255 >> 4
    "#,
        15,
    );
}

// ============================================================================
// Ternary expression codegen
// ============================================================================

#[test]
fn e2e_p77_ternary_true() {
    assert_exit_code("F main() -> i64 = true ? 42 : 0", 42);
}

#[test]
fn e2e_p77_ternary_false() {
    assert_exit_code("F main() -> i64 = false ? 0 : 33", 33);
}

#[test]
fn e2e_p77_ternary_comparison() {
    assert_exit_code("F main() -> i64 = 5 > 3 ? 1 : 0", 1);
}

#[test]
fn e2e_p77_nested_ternary() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := 10
            r := x > 20 ? 3 : 0
            I r == 0 {
                r = x > 5 ? 2 : 1
            }
            r
        }
    "#,
        2,
    );
}

// ============================================================================
// Cast expressions
// ============================================================================

#[test]
fn e2e_p77_cast_i64_to_i64() {
    assert_exit_code("F main() -> i64 { x := 42; x as i64 }", 42);
}

// ============================================================================
// Block expressions
// ============================================================================

#[test]
fn e2e_p77_block_expression() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := {
                a := 10
                b := 32
                a + b
            }
            x
        }
    "#,
        42,
    );
}

#[test]
fn e2e_p77_nested_block() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := {
                y := {
                    z := 7
                    z * 6
                }
                y
            }
            x
        }
    "#,
        42,
    );
}

// ============================================================================
// Self-recursion with @
// ============================================================================

#[test]
fn e2e_p77_self_recursion_factorial() {
    assert_exit_code(
        r#"
        F fact(n: i64) -> i64 {
            I n <= 1 { R 1 }
            n * @(n - 1)
        }
        F main() -> i64 = fact(5)
    "#,
        120,
    );
}

#[test]
fn e2e_p77_self_recursion_fibonacci() {
    assert_exit_code(
        r#"
        F fib(n: i64) -> i64 {
            I n <= 1 { R n }
            @(n - 1) + @(n - 2)
        }
        F main() -> i64 = fib(10)
    "#,
        55,
    );
}

// ============================================================================
// Enum construction and matching
// ============================================================================

#[test]
fn e2e_p77_enum_simple_match() {
    assert_exit_code(
        r#"
        E Color { Red, Green, Blue }
        F main() -> i64 {
            c := Red
            M c {
                Red => 1,
                Green => 2,
                Blue => 3,
                _ => 0
            }
        }
    "#,
        1,
    );
}

#[test]
fn e2e_p77_enum_match_green() {
    assert_exit_code(
        r#"
        E Color { Red, Green, Blue }
        F main() -> i64 {
            c := Green
            M c {
                Red => 10,
                Green => 20,
                Blue => 30,
                _ => 0
            }
        }
    "#,
        20,
    );
}

#[test]
fn e2e_p77_enum_with_data() {
    assert_exit_code(
        r#"
        E Shape {
            Circle(i64),
            Rect(i64, i64)
        }
        F area(s: Shape) -> i64 {
            M s {
                Circle(r) => r * r,
                Rect(w, h) => w * h,
                _ => 0
            }
        }
        F main() -> i64 = area(Circle(5))
    "#,
        25,
    );
}

#[test]
fn e2e_p77_enum_rect_area() {
    assert_exit_code(
        r#"
        E Shape {
            Circle(i64),
            Rect(i64, i64)
        }
        F area(s: Shape) -> i64 {
            M s {
                Circle(r) => r * r,
                Rect(w, h) => w * h,
                _ => 0
            }
        }
        F main() -> i64 = area(Rect(6, 7))
    "#,
        42,
    );
}

// ============================================================================
// Integer match patterns
// ============================================================================

#[test]
fn e2e_p77_match_int_first() {
    assert_exit_code(
        r#"
        F classify(n: i64) -> i64 {
            M n {
                0 => 10,
                1 => 20,
                2 => 30,
                _ => 99
            }
        }
        F main() -> i64 = classify(0)
    "#,
        10,
    );
}

#[test]
fn e2e_p77_match_int_wildcard() {
    assert_exit_code(
        r#"
        F classify(n: i64) -> i64 {
            M n {
                0 => 10,
                1 => 20,
                _ => 99
            }
        }
        F main() -> i64 = classify(42)
    "#,
        99,
    );
}

// ============================================================================
// Struct construction and field access
// ============================================================================

#[test]
fn e2e_p77_struct_field_access() {
    assert_exit_code(
        r#"
        S Point { x: i64, y: i64 }
        F main() -> i64 {
            p := Point { x: 10, y: 32 }
            p.x + p.y
        }
    "#,
        42,
    );
}

#[test]
fn e2e_p77_nested_struct() {
    assert_exit_code(
        r#"
        S Inner { val: i64 }
        S Outer { inner: Inner, extra: i64 }
        F main() -> i64 {
            o := Outer { inner: Inner { val: 40 }, extra: 2 }
            o.inner.val + o.extra
        }
    "#,
        42,
    );
}

// ============================================================================
// Impl methods
// ============================================================================

#[test]
fn e2e_p77_impl_method_call() {
    assert_exit_code(
        r#"
        S Counter { value: i64 }
        X Counter {
            F get(self) -> i64 = self.value
            F add(self, n: i64) -> i64 = self.value + n
        }
        F main() -> i64 {
            c := Counter { value: 40 }
            c.add(2)
        }
    "#,
        42,
    );
}

#[test]
fn e2e_p77_static_method() {
    assert_exit_code(
        r#"
        S Pair { a: i64, b: i64 }
        X Pair {
            F new(a: i64, b: i64) -> Pair = Pair { a: a, b: b }
            F sum(self) -> i64 = self.a + self.b
        }
        F main() -> i64 {
            p := Pair::new(20, 22)
            p.sum()
        }
    "#,
        42,
    );
}

// ============================================================================
// Trait impl
// ============================================================================

#[test]
fn e2e_p77_trait_impl_basic() {
    assert_exit_code(
        r#"
        W Evaluate { F eval(self) -> i64 }
        S Literal { value: i64 }
        X Literal: Evaluate {
            F eval(self) -> i64 = self.value
        }
        F main() -> i64 {
            lit := Literal { value: 42 }
            lit.eval()
        }
    "#,
        42,
    );
}

// ============================================================================
// Lambda / closures
// ============================================================================

#[test]
fn e2e_p77_lambda_simple() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            f := |x: i64| x * 2
            f(21)
        }
    "#,
        42,
    );
}

#[test]
fn e2e_p77_lambda_multi_param() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            add := |a: i64, b: i64| a + b
            add(20, 22)
        }
    "#,
        42,
    );
}

// ============================================================================
// Higher order functions
// ============================================================================

#[test]
fn e2e_p77_higher_order_apply() {
    assert_exit_code(
        r#"
        F apply(x: i64, f: fn(i64) -> i64) -> i64 = f(x)
        F double(x: i64) -> i64 = x * 2
        F main() -> i64 = apply(21, double)
    "#,
        42,
    );
}

// ============================================================================
// Generic functions
// ============================================================================

#[test]
fn e2e_p77_generic_identity() {
    assert_exit_code(
        r#"
        F id<T>(x: T) -> T = x
        F main() -> i64 = id(42)
    "#,
        42,
    );
}

// ============================================================================
// Type alias
// ============================================================================

#[test]
fn e2e_p77_type_alias() {
    assert_exit_code(
        r#"
        T Num = i64
        F double(x: Num) -> Num = x * 2
        F main() -> i64 = double(21)
    "#,
        42,
    );
}

// ============================================================================
// Constants
// ============================================================================

#[test]
fn e2e_p77_const_declaration() {
    assert_exit_code(
        r#"
        C ANSWER: i64 = 42
        F main() -> i64 = ANSWER
    "#,
        42,
    );
}

#[test]
fn e2e_p77_const_in_expression() {
    assert_exit_code(
        r#"
        C BASE: i64 = 40
        F main() -> i64 = BASE + 2
    "#,
        42,
    );
}

// ============================================================================
// Global variables
// ============================================================================

#[test]
fn e2e_p77_global_variable() {
    assert_compiles(
        r#"
        G counter: i64 = 42
        F main() -> i64 = 42
    "#,
    );
}

// ============================================================================
// Loops with break/continue
// ============================================================================

#[test]
fn e2e_p77_loop_break() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := mut 0
            L {
                x = x + 1
                I x >= 42 { B }
            }
            x
        }
    "#,
        42,
    );
}

#[test]
fn e2e_p77_for_loop_sum() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            sum := mut 0
            L i:0..10 {
                sum = sum + i
            }
            sum
        }
    "#,
        45,
    );
}

#[test]
fn e2e_p77_while_loop() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := mut 0
            L x < 42 {
                x = x + 1
            }
            x
        }
    "#,
        42,
    );
}

#[test]
fn e2e_p77_loop_continue() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            sum := mut 0
            L i:0..10 {
                I i % 2 == 0 { C }
                sum = sum + i
            }
            sum
        }
    "#,
        25,
    );
}

// ============================================================================
// Nested if/else
// ============================================================================

#[test]
fn e2e_p77_nested_if() {
    assert_exit_code(
        r#"
        F classify(n: i64) -> i64 {
            I n > 100 { R 3 }
            I n > 50 { R 2 }
            I n > 0 { R 1 }
            R 0
        }
        F main() -> i64 = classify(75)
    "#,
        2,
    );
}

#[test]
fn e2e_p77_if_else_expression() {
    assert_exit_code(
        r#"
        F abs(x: i64) -> i64 {
            I x < 0 { R 0 - x }
            x
        }
        F main() -> i64 = abs(-42)
    "#,
        42,
    );
}

// ============================================================================
// Mutual recursion
// ============================================================================

#[test]
fn e2e_p77_mutual_recursion() {
    assert_exit_code(
        r#"
        F is_even(n: i64) -> i64 {
            I n == 0 { R 1 }
            is_odd(n - 1)
        }
        F is_odd(n: i64) -> i64 {
            I n == 0 { R 0 }
            is_even(n - 1)
        }
        F main() -> i64 = is_even(10)
    "#,
        1,
    );
}

// ============================================================================
// Complex expressions
// ============================================================================

#[test]
fn e2e_p77_complex_arithmetic() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            a := 10
            b := 3
            c := a * b + a / b - a % b
            c
        }
    "#,
        32,
    );
}

#[test]
fn e2e_p77_bool_logic() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            a := true
            b := false
            I a && !b { R 42 }
            R 0
        }
    "#,
        42,
    );
}

#[test]
fn e2e_p77_comparison_chain() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := 5
            I x > 0 && x < 10 && x != 3 { R 42 }
            R 0
        }
    "#,
        42,
    );
}

// ============================================================================
// Defer statement
// ============================================================================

#[test]
fn e2e_p77_defer_basic() {
    assert_compiles(
        r#"
        F main() -> i64 {
            x := mut 0
            D { x = x + 1 }
            x = 41
            x
        }
    "#,
    );
}

// ============================================================================
// Multiple functions calling each other
// ============================================================================

#[test]
fn e2e_p77_call_chain() {
    assert_exit_code(
        r#"
        F step1(x: i64) -> i64 = x + 10
        F step2(x: i64) -> i64 = step1(x) * 2
        F step3(x: i64) -> i64 = step2(x) - 16
        F main() -> i64 = step3(3)
    "#,
        10,
    );
}

// ============================================================================
// Void function (no return value)
// ============================================================================

#[test]
fn e2e_p77_void_function() {
    assert_exit_code(
        r#"
        F noop() -> i64 = 0
        F main() -> i64 {
            noop()
            42
        }
    "#,
        42,
    );
}

// ============================================================================
// Nested match
// ============================================================================

#[test]
fn e2e_p77_nested_match() {
    assert_exit_code(
        r#"
        F inner(y: i64) -> i64 {
            M y {
                3 => 42,
                _ => 29
            }
        }
        F main() -> i64 {
            x := 2
            y := 3
            M x {
                1 => 19,
                2 => inner(y),
                _ => 99
            }
        }
    "#,
        42,
    );
}

// ============================================================================
// Expression body functions
// ============================================================================

#[test]
fn e2e_p77_expression_body_arithmetic() {
    assert_exit_code("F main() -> i64 = 6 * 7", 42);
}

#[test]
fn e2e_p77_expression_body_call() {
    assert_exit_code(
        r#"
        F add(a: i64, b: i64) -> i64 = a + b
        F main() -> i64 = add(20, 22)
    "#,
        42,
    );
}

// ============================================================================
// Multiple struct instances
// ============================================================================

#[test]
fn e2e_p77_multiple_structs() {
    assert_exit_code(
        r#"
        S Vec2 { x: i64, y: i64 }
        F dot(a: Vec2, b: Vec2) -> i64 = a.x * b.x + a.y * b.y
        F main() -> i64 {
            a := Vec2 { x: 3, y: 4 }
            b := Vec2 { x: 5, y: 6 }
            dot(a, b)
        }
    "#,
        39,
    );
}

// ============================================================================
// Negative numbers
// ============================================================================

#[test]
fn e2e_p77_negative_literal() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := -10
            0 - x
        }
    "#,
        10,
    );
}

// ============================================================================
// Chained method calls
// ============================================================================

#[test]
fn e2e_p77_chained_methods() {
    assert_exit_code(
        r#"
        S Builder { value: i64 }
        X Builder {
            F new() -> Builder = Builder { value: 0 }
            F add(self, n: i64) -> Builder = Builder { value: self.value + n }
            F build(self) -> i64 = self.value
        }
        F main() -> i64 {
            b := Builder::new()
            b2 := b.add(20)
            b3 := b2.add(22)
            b3.build()
        }
    "#,
        42,
    );
}

// ============================================================================
// Multiple enum variants
// ============================================================================

#[test]
fn e2e_p77_enum_four_variants() {
    assert_exit_code(
        r#"
        E Dir { North, South, East, West }
        F score(d: Dir) -> i64 {
            M d {
                North => 1,
                South => 2,
                East => 3,
                West => 4,
                _ => 0
            }
        }
        F main() -> i64 = score(West) * 10 + score(East)
    "#,
        43,
    );
}

// ============================================================================
// Early return
// ============================================================================

#[test]
fn e2e_p77_early_return() {
    assert_exit_code(
        r#"
        F check(x: i64) -> i64 {
            I x == 0 { R 99 }
            I x == 1 { R 42 }
            R 0
        }
        F main() -> i64 = check(1)
    "#,
        42,
    );
}

// ============================================================================
// Extern functions
// ============================================================================

#[test]
fn e2e_p77_extern_function() {
    assert_compiles(
        r#"
        N "C" {
            F puts(s: i64) -> i64
        }
        F main() -> i64 = 42
    "#,
    );
}

// ============================================================================
// Complex control flow
// ============================================================================

#[test]
fn e2e_p77_fizzbuzz_count() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            count := mut 0
            L i:1..21 {
                I i % 15 == 0 { count = count + 3 }
                E I i % 3 == 0 { count = count + 1 }
                E I i % 5 == 0 { count = count + 2 }
            }
            count
        }
    "#,
        14,
    );
}

// ============================================================================
// Array-like patterns
// ============================================================================

#[test]
fn e2e_p77_pair_sum() {
    // Tuple index access (.0, .1) may not be supported - use struct instead
    assert_exit_code(
        r#"
        S Pair { a: i64, b: i64 }
        F main() -> i64 {
            p := Pair { a: 20, b: 22 }
            p.a + p.b
        }
    "#,
        42,
    );
}

// ============================================================================
// Boolean expressions as conditions
// ============================================================================

#[test]
fn e2e_p77_bool_and_short_circuit() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            I false && true { R 0 }
            42
        }
    "#,
        42,
    );
}

#[test]
fn e2e_p77_bool_or_short_circuit() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            I true || false { R 42 }
            0
        }
    "#,
        42,
    );
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn e2e_p77_zero_division_guard() {
    assert_exit_code(
        r#"
        F safe_div(a: i64, b: i64) -> i64 {
            I b == 0 { R 0 }
            a / b
        }
        F main() -> i64 = safe_div(84, 2)
    "#,
        42,
    );
}

#[test]
fn e2e_p77_many_params() {
    assert_exit_code(
        r#"
        F add5(a: i64, b: i64, c: i64, d: i64, e: i64) -> i64 = a + b + c + d + e
        F main() -> i64 = add5(8, 8, 8, 9, 9)
    "#,
        42,
    );
}

#[test]
fn e2e_p77_deeply_nested_if() {
    assert_exit_code(
        r#"
        F main() -> i64 {
            x := 42
            I x > 0 {
                I x > 10 {
                    I x > 20 {
                        I x > 30 {
                            I x > 40 {
                                R x
                            }
                        }
                    }
                }
            }
            R 0
        }
    "#,
        42,
    );
}

#[test]
fn e2e_p77_unused_function_present() {
    assert_exit_code(
        r#"
        F unused() -> i64 = 99
        F main() -> i64 = 42
    "#,
        42,
    );
}
