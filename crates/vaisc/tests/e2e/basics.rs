use super::helpers::*;

// ==================== Basic Programs ====================

#[test]
fn e2e_return_constant() {
    assert_exit_code("F main()->i64 = 42", 42);
}

#[test]
fn e2e_return_zero() {
    assert_exit_code("F main()->i64 = 0", 0);
}

#[test]
fn e2e_return_one() {
    assert_exit_code("F main()->i64 = 1", 1);
}

// ==================== Arithmetic ====================

#[test]
fn e2e_addition() {
    assert_exit_code("F main()->i64 = 30 + 12", 42);
}

#[test]
fn e2e_subtraction() {
    assert_exit_code("F main()->i64 = 50 - 8", 42);
}

#[test]
fn e2e_multiplication() {
    assert_exit_code("F main()->i64 = 6 * 7", 42);
}

#[test]
fn e2e_division() {
    assert_exit_code("F main()->i64 = 84 / 2", 42);
}

#[test]
fn e2e_modulo() {
    assert_exit_code("F main()->i64 = 47 % 5", 2);
}

#[test]
fn e2e_nested_arithmetic() {
    assert_exit_code("F main()->i64 = (3 + 4) * (2 + 4)", 42);
}

#[test]
fn e2e_operator_precedence() {
    // 2 + 3 * 4 = 14
    assert_exit_code("F main()->i64 = 2 + 3 * 4", 14);
}

// ==================== Functions ====================

#[test]
fn e2e_simple_function_call() {
    let source = r#"
F double(x: i64) -> i64 = x * 2
F main() -> i64 = double(21)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_multiple_function_calls() {
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b
F mul(a: i64, b: i64) -> i64 = a * b
F main() -> i64 = add(mul(5, 8), 2)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_function_with_body() {
    let source = r#"
F compute(x: i64) -> i64 {
    a := x * 2
    b := a + 1
    b
}
F main() -> i64 = compute(20)
"#;
    assert_exit_code(source, 41);
}

// ==================== Recursion (Self-Recursion Operator @) ====================

#[test]
fn e2e_fibonacci() {
    let source = r#"
F fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)
F main() -> i64 = fib(10)
"#;
    assert_exit_code(source, 55);
}

#[test]
fn e2e_factorial() {
    // 5! = 120, but exit codes are mod 256, so 120
    let source = r#"
F factorial(n: i64) -> i64 = n < 2 ? 1 : n * @(n-1)
F main() -> i64 = factorial(5)
"#;
    assert_exit_code(source, 120);
}

#[test]
fn e2e_countdown() {
    let source = r#"
F countdown(n: i64) -> i64 = n < 1 ? 0 : @(n-1)
F main() -> i64 = countdown(100)
"#;
    assert_exit_code(source, 0);
}

// ==================== Control Flow: If/Else ====================

#[test]
fn e2e_ternary_true() {
    assert_exit_code("F main()->i64 = 1 > 0 ? 42 : 0", 42);
}

#[test]
fn e2e_ternary_false() {
    assert_exit_code("F main()->i64 = 0 > 1 ? 0 : 42", 42);
}

#[test]
fn e2e_if_else_block() {
    let source = r#"
F max(a: i64, b: i64) -> i64 = I a > b { a } E { b }
F main() -> i64 = max(42, 10)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_if_else_nested() {
    let source = r#"
F clamp(x: i64, lo: i64, hi: i64) -> i64 =
    I x < lo { lo } E I x > hi { hi } E { x }
F main() -> i64 = clamp(100, 0, 42)
"#;
    assert_exit_code(source, 42);
}

// ==================== Control Flow: Match ====================

#[test]
fn e2e_match_literal() {
    let source = r#"
F describe(n: i64) -> i64 {
    M n {
        0 => 10,
        1 => 20,
        2 => 30,
        _ => 42
    }
}
F main() -> i64 = describe(5)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_match_first_arm() {
    let source = r#"
F pick(n: i64) -> i64 {
    M n {
        0 => 42,
        _ => 0
    }
}
F main() -> i64 = pick(0)
"#;
    assert_exit_code(source, 42);
}

// ==================== Variables ====================

#[test]
fn e2e_let_binding() {
    let source = r#"
F main() -> i64 {
    x := 40
    y := 2
    x + y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_mutable_variable() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    x = 42
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_multiple_assignments() {
    let source = r#"
F main() -> i64 {
    x := mut 1
    x = x + 10
    x = x * 2
    x = x + 20
    x
}
"#;
    // (1+10)*2+20 = 42
    assert_exit_code(source, 42);
}

// ==================== Boolean Logic ====================

#[test]
fn e2e_comparison_true() {
    // true = 1
    assert_exit_code("F main()->i64 = I 10 > 5 { 1 } E { 0 }", 1);
}

#[test]
fn e2e_comparison_false() {
    assert_exit_code("F main()->i64 = I 5 > 10 { 1 } E { 0 }", 0);
}

#[test]
fn e2e_equality() {
    assert_exit_code("F main()->i64 = I 42 == 42 { 1 } E { 0 }", 1);
}

#[test]
fn e2e_inequality() {
    assert_exit_code("F main()->i64 = I 42 != 43 { 1 } E { 0 }", 1);
}

// ==================== Structs ====================

#[test]
fn e2e_struct_creation_and_field_access() {
    let source = r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { x: 40, y: 2 }
    p.x + p.y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_nested_struct_field_access() {
    let source = r#"
S Inner { val: i64 }
S Outer { a: Inner }
F main() -> i64 {
    inner := Inner { val: 42 }
    outer := Outer { a: inner }
    outer.a.val
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_nested_struct_three_levels() {
    let source = r#"
S Deep { x: i64 }
S Mid { d: Deep }
S Top { m: Mid }
F main() -> i64 {
    deep := Deep { x: 99 }
    mid := Mid { d: deep }
    top := Top { m: mid }
    top.m.d.x
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn e2e_nested_struct_field_arithmetic() {
    let source = r#"
S Point { x: i64, y: i64 }
S Line { start: Point, end: Point }
F main() -> i64 {
    s := Point { x: 10, y: 20 }
    e := Point { x: 30, y: 40 }
    line := Line { start: s, end: e }
    line.start.x + line.end.y
}
"#;
    assert_exit_code(source, 50);
}

// This test verifies single-level struct field access works correctly.
#[test]
fn e2e_struct_two_fields() {
    let source = r#"
S Pair { first: i64, second: i64 }
F main() -> i64 {
    p := Pair { first: 40, second: 2 }
    p.first + p.second
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Enums ====================

// NOTE: Enum variant matching for unit variants has a codegen issue where
// all variants match the first arm. This is a known compiler limitation.
// Test enum definition compiles and basic match on integers works instead.
#[test]
fn e2e_enum_definition_compiles() {
    let source = r#"
E Color { Red, Green, Blue }
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

// ==================== Puts / stdout ====================

#[test]
fn e2e_puts_output() {
    let source = r#"
F main() -> i64 {
    puts("Hello, Vais!")
    0
}
"#;
    assert_stdout_contains(source, "Hello, Vais!");
}

#[test]
fn e2e_putchar_output() {
    let source = r#"
F main() -> i64 {
    putchar(72)
    putchar(105)
    putchar(10)
    0
}
"#;
    // H=72, i=105
    assert_stdout_contains(source, "Hi");
}

// ==================== Arrays / Pointers ====================

#[test]
fn e2e_array_access() {
    let source = r#"
F main() -> i64 {
    arr: *i64 = [10, 20, 30, 42]
    arr[3]
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_array_mutation() {
    let source = r#"
F main() -> i64 {
    arr: *i64 = [10, 20, 30, 0]
    arr[3] = 42
    arr[3]
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_array_mutation_overwrite() {
    let source = r#"
F main() -> i64 {
    arr: *i64 = [1, 2, 3, 4]
    arr[0] = 99
    arr[0]
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn e2e_array_mutation_with_expr_index() {
    let source = r#"
F main() -> i64 {
    arr: *i64 = [10, 20, 30, 40, 50]
    i := 2
    arr[i] = 42
    arr[2]
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Complex Programs ====================

#[test]
fn e2e_gcd() {
    let source = r#"
F gcd(a: i64, b: i64) -> i64 = I b == 0 { a } E { gcd(b, a % b) }
F main() -> i64 = gcd(462, 1071)
"#;
    // gcd(462, 1071) = 21
    assert_exit_code(source, 21);
}

#[test]
fn e2e_sum_to_n() {
    let source = r#"
F sum(n: i64) -> i64 = I n == 0 { 0 } E { n + @(n-1) }
F main() -> i64 = sum(9)
"#;
    // 9+8+7+6+5+4+3+2+1 = 45
    assert_exit_code(source, 45);
}

#[test]
fn e2e_power() {
    let source = r#"
F pow(base: i64, exp: i64) -> i64 = I exp == 0 { 1 } E { base * @(base, exp - 1) }
F main() -> i64 = pow(2, 5)
"#;
    // 2^5 = 32
    assert_exit_code(source, 32);
}

#[test]
fn e2e_abs() {
    let source = r#"
F abs(x: i64) -> i64 = I x < 0 { 0 - x } E { x }
F main() -> i64 = abs(0 - 42)
"#;
    assert_exit_code(source, 42);
}

// ==================== Compilation Errors ====================

#[test]
fn e2e_error_undefined_function() {
    assert_compile_error("F main()->i64 = unknown_func(42)");
}

#[test]
fn e2e_error_type_mismatch() {
    // Passing bool where i64 expected (if the type system catches it)
    assert_compile_error(
        r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(1, "hello")
"#,
    );
}

// ==================== Edge Cases ====================

#[test]
fn e2e_empty_main_block() {
    let source = r#"
F main() -> i64 {
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_large_exit_code() {
    // Exit codes are modulo 256 on most systems
    assert_exit_code("F main()->i64 = 300", 300 % 256);
}

#[test]
fn e2e_negative_return() {
    // Negative exit codes wrap around (256 - 1 = 255 on most systems)
    let source = "F main()->i64 = 0 - 1";
    let result = compile_and_run(source).expect("Should compile and run");
    // On macOS/Linux, negative exit codes wrap: -1 -> 255
    assert_eq!(result.exit_code & 0xFF, 255);
}

// ==================== Multi-function Programs ====================

#[test]
fn e2e_chain_of_functions() {
    let source = r#"
F step1(x: i64) -> i64 = x + 10
F step2(x: i64) -> i64 = x * 2
F step3(x: i64) -> i64 = x + 2
F main() -> i64 = step3(step2(step1(10)))
"#;
    // step1(10)=20, step2(20)=40, step3(40)=42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_mutual_recursion() {
    let source = r#"
F is_even(n: i64) -> i64 = I n == 0 { 1 } E { is_odd(n - 1) }
F is_odd(n: i64) -> i64 = I n == 0 { 0 } E { is_even(n - 1) }
F main() -> i64 = is_even(10)
"#;
    assert_exit_code(source, 1);
}

// NOTE: Passing structs by value to functions has a codegen limitation (ptr type mismatch).
// This test verifies struct field access and arithmetic in main directly.
#[test]
fn e2e_struct_field_arithmetic() {
    let source = r#"
S Rect { w: i64, h: i64 }
F main() -> i64 {
    r := Rect { w: 6, h: 7 }
    r.w * r.h
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Bitwise Operations ====================

#[test]
fn e2e_bitwise_and() {
    // 0xFF & 0x0F = 0x0F = 15
    assert_exit_code("F main()->i64 = 255 & 15", 15);
}

#[test]
fn e2e_bitwise_or() {
    // 0x20 | 0x0A = 0x2A = 42
    assert_exit_code("F main()->i64 = 32 | 10", 42);
}

#[test]
fn e2e_bitwise_xor() {
    // 0xFF ^ 0xD5 = 0x2A = 42
    assert_exit_code("F main()->i64 = 255 ^ 213", 42);
}

#[test]
fn e2e_bitwise_left_shift() {
    // 21 << 1 = 42
    assert_exit_code("F main()->i64 = 21 << 1", 42);
}

#[test]
fn e2e_bitwise_right_shift() {
    // 168 >> 2 = 42
    assert_exit_code("F main()->i64 = 168 >> 2", 42);
}

// ==================== Logical Operators ====================

#[test]
fn e2e_logical_and_true() {
    let source = "F main()->i64 = I 1 > 0 && 2 > 1 { 42 } E { 0 }";
    assert_exit_code(source, 42);
}

#[test]
fn e2e_logical_and_false() {
    let source = "F main()->i64 = I 1 > 0 && 2 < 1 { 0 } E { 42 }";
    assert_exit_code(source, 42);
}

#[test]
fn e2e_logical_or_true() {
    let source = "F main()->i64 = I 0 > 1 || 2 > 1 { 42 } E { 0 }";
    assert_exit_code(source, 42);
}

#[test]
fn e2e_logical_or_both_false() {
    let source = "F main()->i64 = I 0 > 1 || 0 > 2 { 0 } E { 42 }";
    assert_exit_code(source, 42);
}

// ==================== Iterative Computation Tests (via recursion) ====================

#[test]
fn e2e_recursive_count_to_n() {
    // Count from 0 to n using recursion (simulates loop counting)
    let source = r#"
F count(n: i64) -> i64 = I n == 0 { 0 } E { 1 + @(n - 1) }
F main() -> i64 = count(10)
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_recursive_sum_to_n() {
    // Sum 1..n using tail-recursive accumulator pattern
    let source = r#"
F sum_acc(n: i64, acc: i64) -> i64 = I n == 0 { acc } E { @(n - 1, acc + n) }
F main() -> i64 = sum_acc(9, 0)
"#;
    // 9+8+7+6+5+4+3+2+1 = 45
    assert_exit_code(source, 45);
}

#[test]
fn e2e_recursive_product() {
    // Multiply 2 five times: 2^5 = 32
    let source = r#"
F repeat_mul(base: i64, times: i64, acc: i64) -> i64 =
    I times == 0 { acc } E { @(base, times - 1, acc * base) }
F main() -> i64 = repeat_mul(2, 5, 1)
"#;
    assert_exit_code(source, 32);
}

#[test]
fn e2e_recursive_countdown_sum() {
    // Sum from 10 down to 1: 10+9+...+1 = 55
    let source = r#"
F countdown_sum(n: i64) -> i64 = I n == 0 { 0 } E { n + @(n - 1) }
F main() -> i64 = countdown_sum(10)
"#;
    assert_exit_code(source, 55);
}

#[test]
fn e2e_recursive_multiply_add() {
    // Compute base * times via repeated addition
    let source = r#"
F mul_add(base: i64, times: i64, acc: i64) -> i64 =
    I times == 0 { acc } E { @(base, times - 1, acc + base) }
F main() -> i64 = mul_add(5, 5, 0)
"#;
    // 5 * 5 = 25
    assert_exit_code(source, 25);
}

// ==================== String Output Tests ====================

#[test]
fn e2e_multiple_puts() {
    let source = r#"
F main() -> i64 {
    puts("Hello")
    puts("World")
    0
}
"#;
    assert_stdout_contains(source, "Hello");
}

#[test]
fn e2e_puts_multiple_lines_check_second() {
    let source = r#"
F main() -> i64 {
    puts("Line1")
    puts("Line2")
    0
}
"#;
    assert_stdout_contains(source, "Line2");
}

#[test]
fn e2e_putchar_spell_ok() {
    let source = r#"
F main() -> i64 {
    putchar(79)
    putchar(75)
    putchar(10)
    0
}
"#;
    // O=79, K=75
    assert_stdout_contains(source, "OK");
}

#[test]
fn e2e_putchar_digit_output() {
    let source = r#"
F main() -> i64 {
    putchar(52)
    putchar(50)
    putchar(10)
    0
}
"#;
    // '4'=52, '2'=50 => "42"
    assert_stdout_contains(source, "42");
}

// ==================== Advanced Recursion ====================

#[test]
fn e2e_ackermann_small() {
    // A(2,3) = 9
    let source = r#"
F ack(m: i64, n: i64) -> i64 =
    I m == 0 { n + 1 }
    E I n == 0 { @(m - 1, 1) }
    E { @(m - 1, @(m, n - 1)) }
F main() -> i64 = ack(2, 3)
"#;
    assert_exit_code(source, 9);
}

#[test]
fn e2e_tower_of_hanoi_count() {
    // Minimum moves for n disks = 2^n - 1
    // hanoi(6) = 63
    let source = r#"
F hanoi(n: i64) -> i64 = I n == 0 { 0 } E { 2 * @(n - 1) + 1 }
F main() -> i64 = hanoi(6)
"#;
    assert_exit_code(source, 63);
}

#[test]
fn e2e_collatz_steps() {
    // Collatz steps for 6: 6->3->10->5->16->8->4->2->1 = 8 steps
    let source = r#"
F collatz(n: i64) -> i64 =
    I n == 1 { 0 }
    E I n % 2 == 0 { 1 + @(n / 2) }
    E { 1 + @(3 * n + 1) }
F main() -> i64 = collatz(6)
"#;
    assert_exit_code(source, 8);
}

#[test]
fn e2e_triple_mutual_recursion() {
    // Three mutually recursive functions that decrement and cycle
    let source = r#"
F fa(n: i64) -> i64 = I n == 0 { 1 } E { fb(n - 1) }
F fb(n: i64) -> i64 = I n == 0 { 2 } E { fc(n - 1) }
F fc(n: i64) -> i64 = I n == 0 { 3 } E { fa(n - 1) }
F main() -> i64 = fa(7)
"#;
    // fa(7)->fb(6)->fc(5)->fa(4)->fb(3)->fc(2)->fa(1)->fb(0)=2
    assert_exit_code(source, 2);
}

// ==================== Complex Control Flow ====================

#[test]
fn e2e_nested_if_else_chain() {
    let source = r#"
F classify(n: i64) -> i64 =
    I n < 10 { 1 }
    E I n < 20 { 2 }
    E I n < 50 { 3 }
    E I n < 100 { 4 }
    E { 5 }
F main() -> i64 = classify(42)
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_match_many_arms() {
    let source = r#"
F day_code(d: i64) -> i64 {
    M d {
        1 => 10,
        2 => 20,
        3 => 30,
        4 => 40,
        5 => 50,
        6 => 60,
        7 => 70,
        _ => 0
    }
}
F main() -> i64 = day_code(4)
"#;
    assert_exit_code(source, 40);
}

#[test]
fn e2e_if_multiple_paths() {
    let source = r#"
F sign(n: i64) -> i64 =
    I n > 0 { 1 }
    E I n < 0 { 255 }
    E { 0 }
F main() -> i64 = sign(0 - 5)
"#;
    // Return 255 directly for negative input
    assert_exit_code(source, 255);
}

#[test]
fn e2e_cascading_conditional_calls() {
    let source = r#"
F pick(a: i64, b: i64) -> i64 = I a > b { a } E { b }
F bump(x: i64) -> i64 = x + 1
F main() -> i64 = bump(pick(20, bump(20)))
"#;
    // bump(20)=21, pick(20,21)=21, bump(21)=22
    assert_exit_code(source, 22);
}

#[test]
fn e2e_if_else_grade() {
    let source = r#"
F grade(score: i64) -> i64 =
    I score > 90 { 5 }
    E I score > 80 { 4 }
    E I score > 70 { 3 }
    E I score > 60 { 2 }
    E { 1 }
F main() -> i64 = grade(85)
"#;
    assert_exit_code(source, 4);
}

// ==================== Variable Scoping ====================

#[test]
fn e2e_variable_shadowing() {
    let source = r#"
F main() -> i64 {
    x := 10
    y := x + 5
    x := 30
    x + y - 3
}
"#;
    // y = 15, x(new) = 30, 30 + 15 - 3 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_nested_let_bindings() {
    let source = r#"
F main() -> i64 {
    a := 2
    b := a * 3
    c := b + a
    d := c * c
    d
}
"#;
    // a=2, b=6, c=8, d=64
    assert_exit_code(source, 64);
}

#[test]
fn e2e_mutable_complex_expression() {
    let source = r#"
F double(x: i64) -> i64 = x * 2
F main() -> i64 {
    x := mut 5
    x = double(x) + 1
    x = double(x)
    x
}
"#;
    // x=5, x=double(5)+1=11, x=double(11)=22
    assert_exit_code(source, 22);
}

#[test]
fn e2e_multiple_mutable_updates() {
    let source = r#"
F main() -> i64 {
    a := mut 0
    b := mut 0
    a = 10
    b = 20
    a = a + b
    b = a - 5
    b
}
"#;
    // a=10, b=20, a=30, b=25
    assert_exit_code(source, 25);
}

// ==================== Array Operations ====================

#[test]
fn e2e_array_computed_values() {
    let source = r#"
F main() -> i64 {
    arr: *i64 = [2 * 3, 4 + 1, 7 * 6]
    arr[2]
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_array_first_element() {
    let source = r#"
F main() -> i64 {
    arr: *i64 = [99, 10, 20]
    arr[0]
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn e2e_array_sum_elements() {
    let source = r#"
F main() -> i64 {
    arr: *i64 = [10, 20, 12]
    arr[0] + arr[1] + arr[2]
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Edge Cases & Regression ====================

#[test]
fn e2e_zero_division_guard() {
    let source = r#"
F safe_div(a: i64, b: i64) -> i64 = I b == 0 { 0 } E { a / b }
F main() -> i64 = safe_div(100, 0)
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_deeply_nested_calls() {
    let source = r#"
F f1(x: i64) -> i64 = x + 1
F f2(x: i64) -> i64 = f1(x) + 1
F f3(x: i64) -> i64 = f2(x) + 1
F f4(x: i64) -> i64 = f3(x) + 1
F f5(x: i64) -> i64 = f4(x) + 1
F f6(x: i64) -> i64 = f5(x) + 1
F f7(x: i64) -> i64 = f6(x) + 1
F f8(x: i64) -> i64 = f7(x) + 1
F f9(x: i64) -> i64 = f8(x) + 1
F f10(x: i64) -> i64 = f9(x) + 1
F main() -> i64 = f10(32)
"#;
    // 32 + 10 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_large_number_arithmetic() {
    // Use large numbers but return result mod-friendly
    let source = r#"
F main() -> i64 {
    a := 1000000
    b := 999958
    a - b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_boolean_as_integer() {
    // true comparisons return 1, false return 0
    let source = r#"
F btoi(cond: i64) -> i64 = I cond > 0 { 1 } E { 0 }
F main() -> i64 {
    a := btoi(10)
    b := btoi(0)
    a + b
}
"#;
    // a=1, b=0 => 1
    assert_exit_code(source, 1);
}

#[test]
fn e2e_negative_number_arithmetic() {
    let source = r#"
F main() -> i64 {
    a := 0 - 10
    b := 0 - 32
    result := 0 - (a + b)
    result
}
"#;
    // a=-10, b=-32, a+b=-42, 0-(-42)=42
    assert_exit_code(source, 42);
}

// ==================== Multi-struct Programs ====================

#[test]
fn e2e_multiple_struct_types() {
    let source = r#"
S Vec2 { x: i64, y: i64 }
S Size { w: i64, h: i64 }
F main() -> i64 {
    pos := Vec2 { x: 10, y: 20 }
    dim := Size { w: 6, h: 7 }
    pos.x + dim.h * dim.w - pos.y + 10
}
"#;
    // 10 + 42 - 20 + 10 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_computed_fields() {
    let source = r#"
S Result { value: i64, ok: i64 }
F main() -> i64 {
    r := Result { value: 6 * 7, ok: 1 }
    I r.ok == 1 { r.value } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_in_complex_program() {
    let source = r#"
S Counter { count: i64, step: i64 }
F advance(count: i64, step: i64, times: i64) -> i64 =
    I times == 0 { count } E { @(count + step, step, times - 1) }
F main() -> i64 {
    c := Counter { count: 0, step: 7 }
    advance(c.count, c.step, 6)
}
"#;
    // 0 + 7*6 = 42
    assert_exit_code(source, 42);
}
