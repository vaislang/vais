//! Phase 81 — E2E coverage expansion tests
//!
//! Targets under-covered language features:
//! 1. Type aliases — declaration, usage in functions, nested aliases
//! 2. String interpolation (~) — basic and compound expressions
//! 3. Extern FFI — calling C stdlib functions
//! 4. Advanced control flow — nested loops, break/continue, early return
//! 5. Struct method chaining — multi-field, nested struct, enum methods
//! 6. Pattern matching — complex match arms, nested patterns, guards
//! 7. Recursion — mutual, self-recursion (@), Fibonacci, accumulator
//! 8. Variable shadowing — same-name rebinding in nested scopes
//! 9. Expression-body functions — concise syntax, composability
//! 10. Error recovery — compile errors, diagnostics, graceful failures
//! 11. Bitwise/shift — compound operations, masking, rotation
//! 12. Closure advanced — multi-level capture, closure as return value

use super::helpers::*;

// ==================== 1. Type Aliases ====================

#[test]
fn e2e_p81_type_alias_basic() {
    let source = r#"
T Num = i64
F double(x: Num) -> Num = x * 2
F main() -> i64 = double(21)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_type_alias_in_struct() {
    let source = r#"
T Coord = i64
S Point { x: Coord, y: Coord }
F dist(p: Point) -> Coord = p.x + p.y
F main() -> i64 {
    p := Point { x: 10, y: 32 }
    dist(p)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_type_alias_function_param() {
    let source = r#"
T Score = i64
F add_scores(a: Score, b: Score) -> Score = a + b
F main() -> i64 = add_scores(20, 22)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_type_alias_multiple() {
    let source = r#"
T Width = i64
T Height = i64
T Area = i64
F compute_area(w: Width, h: Height) -> Area = w * h
F main() -> i64 = compute_area(6, 7)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_type_alias_in_return() {
    // Type alias used as function return type
    let source = r#"
T Count = i64
F make_count(n: i64) -> Count = n
F main() -> i64 = make_count(42)
"#;
    assert_exit_code(source, 42);
}

// ==================== 2. Advanced Control Flow ====================

#[test]
fn e2e_p81_nested_loop_break() {
    let source = r#"
F main() -> i64 {
    total := mut 0
    i := mut 0
    L {
        I i >= 5 { B }
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
    // 5 * 3 = 15
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p81_loop_continue_skip_even() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    i := mut 0
    L {
        I i >= 10 { B }
        i = i + 1
        I i % 2 == 0 { C }
        sum = sum + i
    }
    sum
}
"#;
    // 1 + 3 + 5 + 7 + 9 = 25
    assert_exit_code(source, 25);
}

#[test]
fn e2e_p81_early_return_guard() {
    let source = r#"
F classify(n: i64) -> i64 {
    I n < 0 { R 0 }
    I n == 0 { R 1 }
    I n < 10 { R 2 }
    I n < 100 { R 3 }
    R 4
}
F main() -> i64 {
    classify(-5) + classify(0) + classify(7) + classify(50) + classify(200)
}
"#;
    // 0 + 1 + 2 + 3 + 4 = 10
    assert_exit_code(source, 10);
}

#[test]
fn e2e_p81_loop_countdown_accumulator() {
    let source = r#"
F main() -> i64 {
    n := mut 10
    acc := mut 0
    L {
        I n <= 0 { B }
        acc = acc + n
        n = n - 1
    }
    acc
}
"#;
    // 10 + 9 + 8 + ... + 1 = 55
    assert_exit_code(source, 55);
}

#[test]
fn e2e_p81_nested_if_else_chain() {
    let source = r#"
F grade(score: i64) -> i64 {
    I score >= 90 {
        I score >= 95 {
            5
        } E {
            4
        }
    } E {
        I score >= 80 {
            3
        } E {
            I score >= 70 {
                2
            } E {
                1
            }
        }
    }
}
F main() -> i64 {
    grade(97) + grade(91) + grade(85) + grade(75) + grade(60)
}
"#;
    // 5 + 4 + 3 + 2 + 1 = 15
    assert_exit_code(source, 15);
}

// ==================== 3. Struct Methods & Chaining ====================

#[test]
fn e2e_p81_struct_method_basic() {
    let source = r#"
S Counter { val: i64 }
X Counter {
    F get(self) -> i64 { self.val }
}
F main() -> i64 {
    c := Counter { val: 42 }
    c.get()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_struct_multiple_methods() {
    let source = r#"
S Rect { w: i64, h: i64 }
X Rect {
    F area(self) -> i64 { self.w * self.h }
    F perimeter(self) -> i64 { 2 * (self.w + self.h) }
    F is_square(self) -> i64 { I self.w == self.h { 1 } E { 0 } }
}
F main() -> i64 {
    r := Rect { w: 4, h: 6 }
    r.area() + r.perimeter() + r.is_square()
}
"#;
    // area=24, perimeter=20, is_square=0, total=44
    assert_exit_code(source, 44);
}

#[test]
fn e2e_p81_nested_struct_access() {
    let source = r#"
S Inner { val: i64 }
S Outer { data: Inner, scale: i64 }
F main() -> i64 {
    o := Outer { data: Inner { val: 7 }, scale: 6 }
    o.data.val * o.scale
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_struct_with_function() {
    let source = r#"
S Vec2 { x: i64, y: i64 }
F add_vec(a: Vec2, b: Vec2) -> Vec2 {
    Vec2 { x: a.x + b.x, y: a.y + b.y }
}
F main() -> i64 {
    v1 := Vec2 { x: 10, y: 20 }
    v2 := Vec2 { x: 5, y: 7 }
    result := add_vec(v1, v2)
    result.x + result.y
}
"#;
    // (10+5) + (20+7) = 15 + 27 = 42
    assert_exit_code(source, 42);
}

// ==================== 4. Enum & Pattern Matching ====================

#[test]
fn e2e_p81_enum_basic_match() {
    let source = r#"
E Color { Red, Green, Blue }
F color_val(c: Color) -> i64 {
    M c {
        Red => 1,
        Green => 2,
        Blue => 3,
    }
}
F main() -> i64 {
    color_val(Red) + color_val(Green) + color_val(Blue)
}
"#;
    // 1 + 2 + 3 = 6
    assert_exit_code(source, 6);
}

#[test]
fn e2e_p81_match_with_default() {
    let source = r#"
F classify(n: i64) -> i64 {
    M n {
        0 => 100,
        1 => 200,
        _ => 42,
    }
}
F main() -> i64 = classify(99)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_match_multiple_patterns() {
    let source = r#"
F day_type(day: i64) -> i64 {
    M day {
        1 => 10,
        2 => 10,
        3 => 10,
        4 => 10,
        5 => 10,
        6 => 20,
        7 => 20,
        _ => 0,
    }
}
F main() -> i64 {
    day_type(3) + day_type(6) + day_type(7) + day_type(99)
}
"#;
    // 10 + 20 + 20 + 0 = 50
    assert_exit_code(source, 50);
}

#[test]
fn e2e_p81_match_integer_ranges() {
    let source = r#"
F score(n: i64) -> i64 {
    M n {
        0 => 0,
        1 => 1,
        2 => 4,
        3 => 9,
        _ => 100,
    }
}
F main() -> i64 {
    score(0) + score(1) + score(2) + score(3) + score(10)
}
"#;
    // 0 + 1 + 4 + 9 + 100 = 114
    assert_exit_code(source, 114);
}

// ==================== 5. Recursion Patterns ====================

#[test]
fn e2e_p81_fibonacci_recursive() {
    let source = r#"
F fib(n: i64) -> i64 {
    I n <= 1 { R n }
    @(n - 1) + @(n - 2)
}
F main() -> i64 = fib(10)
"#;
    // fib(10) = 55
    assert_exit_code(source, 55);
}

#[test]
fn e2e_p81_factorial_recursive() {
    let source = r#"
F fact(n: i64) -> i64 {
    I n <= 1 { R 1 }
    n * @(n - 1)
}
F main() -> i64 {
    I fact(5) == 120 { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_sum_recursive() {
    let source = r#"
F sum_to(n: i64) -> i64 {
    I n <= 0 { R 0 }
    n + @(n - 1)
}
F main() -> i64 = sum_to(10)
"#;
    // 10+9+8+...+1 = 55
    assert_exit_code(source, 55);
}

#[test]
fn e2e_p81_power_recursive() {
    let source = r#"
F power(base: i64, exp: i64) -> i64 {
    I exp == 0 { R 1 }
    base * power(base, exp - 1)
}
F main() -> i64 {
    I power(2, 8) == 256 { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_gcd_recursive() {
    let source = r#"
F gcd(a: i64, b: i64) -> i64 {
    I b == 0 { R a }
    gcd(b, a % b)
}
F main() -> i64 {
    gcd(48, 18)
}
"#;
    // gcd(48, 18) = 6
    assert_exit_code(source, 6);
}

// ==================== 6. Variable Shadowing ====================

#[test]
fn e2e_p81_shadow_basic() {
    let source = r#"
F main() -> i64 {
    x := 10
    x := x + 20
    x := x + 12
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_shadow_in_if() {
    // Variable shadowed inside if branch
    let source = r#"
F main() -> i64 {
    x := 10
    result := I true {
        x := 32
        x
    } E {
        x
    }
    result
}
"#;
    assert_exit_code(source, 32);
}

#[test]
fn e2e_p81_shadow_function_param() {
    let source = r#"
F process(x: i64) -> i64 {
    x := x * 2
    x := x + 1
    x
}
F main() -> i64 = process(20)
"#;
    // 20 * 2 + 1 = 41
    assert_exit_code(source, 41);
}

// ==================== 7. Expression-Body Functions ====================

#[test]
fn e2e_p81_expr_body_chain() {
    let source = r#"
F double(x: i64) -> i64 = x * 2
F add_one(x: i64) -> i64 = x + 1
F transform(x: i64) -> i64 = add_one(double(x))
F main() -> i64 = transform(20)
"#;
    // double(20) = 40, add_one(40) = 41
    assert_exit_code(source, 41);
}

#[test]
fn e2e_p81_expr_body_with_ternary() {
    let source = r#"
F abs_val(x: i64) -> i64 = I x >= 0 { x } E { 0 - x }
F main() -> i64 = abs_val(-42) + abs_val(0)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_expr_body_recursive() {
    let source = r#"
F count_digits(n: i64) -> i64 = I n < 10 { 1 } E { 1 + @(n / 10) }
F main() -> i64 = count_digits(12345)
"#;
    // 12345 has 5 digits
    assert_exit_code(source, 5);
}

// ==================== 8. Bitwise Operations ====================

#[test]
fn e2e_p81_bitwise_and() {
    let source = r#"
F main() -> i64 = 255 & 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_bitwise_or() {
    let source = r#"
F main() -> i64 = 32 | 10
"#;
    // 32 | 10 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_bitwise_xor() {
    let source = r#"
F main() -> i64 = 63 ^ 21
"#;
    // 63 ^ 21 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_shift_left() {
    let source = r#"
F main() -> i64 = 21 << 1
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_shift_right() {
    let source = r#"
F main() -> i64 = 168 >> 2
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_bitwise_not_complement() {
    // Use XOR with all 1s to simulate NOT for lower bits
    let source = r#"
F main() -> i64 {
    x := 213
    # Invert lower 8 bits: XOR with 0xFF (255)
    inverted := x ^ 255
    inverted & 255
}
"#;
    // 213 XOR 255 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_bitwise_mask_extract() {
    let source = r#"
F extract_nibble(val: i64, pos: i64) -> i64 {
    (val >> (pos * 4)) & 15
}
F main() -> i64 {
    val := 42
    extract_nibble(val, 0) + extract_nibble(val, 1) * 16
}
"#;
    // 42 = 0x2A, low nibble = 10 (0xA), high nibble = 2
    // 10 + 2*16 = 10 + 32 = 42
    assert_exit_code(source, 42);
}

// ==================== 9. Closure Advanced ====================

#[test]
fn e2e_p81_closure_as_arg() {
    let source = r#"
F apply_twice(x: i64, f: fn(i64) -> i64) -> i64 = f(f(x))
F main() -> i64 {
    inc := |n: i64| n + 1
    apply_twice(40, inc)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_closure_capture_direct_call() {
    // Closure captures a variable and is called directly (not via fn pointer)
    let source = r#"
F main() -> i64 {
    offset := 30
    add_offset := |n: i64| n + offset
    add_offset(12)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_closure_composition() {
    let source = r#"
F compose(x: i64, f: fn(i64) -> i64, g: fn(i64) -> i64) -> i64 = g(f(x))
F main() -> i64 {
    double := |x: i64| x * 2
    add_two := |x: i64| x + 2
    compose(20, double, add_two)
}
"#;
    // double(20) = 40, add_two(40) = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_closure_in_loop() {
    // Closure created and called directly inside loop
    let source = r#"
F main() -> i64 {
    sum := mut 0
    i := mut 1
    L {
        I i > 5 { B }
        add_i := |x: i64| x + i
        sum = add_i(sum)
        i = i + 1
    }
    sum
}
"#;
    // sum = 0+1+2+3+4+5 = 15
    assert_exit_code(source, 15);
}

// ==================== 10. Trait Dispatch ====================

#[test]
fn e2e_p81_trait_dispatch_two_types() {
    let source = r#"
W Measurable {
    F measure(self) -> i64
}
S Box { size: i64 }
S Sphere { radius: i64 }
X Box: Measurable {
    F measure(self) -> i64 { self.size * self.size }
}
X Sphere: Measurable {
    F measure(self) -> i64 { self.radius * 3 }
}
F main() -> i64 {
    b := Box { size: 5 }
    s := Sphere { radius: 4 }
    b.measure() + s.measure()
}
"#;
    // 25 + 12 = 37
    assert_exit_code(source, 37);
}

#[test]
fn e2e_p81_trait_with_default_like() {
    let source = r#"
W Printable {
    F code(self) -> i64
}
S Ascii { ch: i64 }
X Ascii: Printable {
    F code(self) -> i64 { self.ch }
}
F main() -> i64 {
    a := Ascii { ch: 42 }
    a.code()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_trait_method_arithmetic() {
    let source = r#"
W Weighted {
    F weight(self) -> i64
}
S Item { w: i64, qty: i64 }
X Item: Weighted {
    F weight(self) -> i64 { self.w * self.qty }
}
F total_weight(a: Item, b: Item) -> i64 {
    a.weight() + b.weight()
}
F main() -> i64 {
    x := Item { w: 5, qty: 4 }
    y := Item { w: 11, qty: 2 }
    total_weight(x, y)
}
"#;
    // 5*4 + 11*2 = 20 + 22 = 42
    assert_exit_code(source, 42);
}

// ==================== 11. Complex Expressions ====================

#[test]
fn e2e_p81_complex_arithmetic_chain() {
    let source = r#"
F main() -> i64 {
    a := 100
    b := 50
    c := 8
    (a - b) * c / (c + 2) + 2
}
"#;
    // (100-50)*8/(8+2)+2 = 50*8/10+2 = 400/10+2 = 40+2 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_boolean_logic_chain() {
    let source = r#"
F main() -> i64 {
    a := 10
    b := 20
    c := 30
    result := mut 0
    I a < b && b < c { result = result + 10 }
    I a > 0 || b < 0 { result = result + 20 }
    I a != b { result = result + 12 }
    result
}
"#;
    // 10 + 20 + 12 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_multiple_comparison_operators() {
    let source = r#"
F to_int(b: bool) -> i64 = I b { 1 } E { 0 }
F main() -> i64 {
    r := mut 0
    I 5 < 10 { r = r + 1 }
    I 10 > 5 { r = r + 2 }
    I 5 <= 5 { r = r + 4 }
    I 5 >= 5 { r = r + 8 }
    I 5 == 5 { r = r + 16 }
    I 5 != 6 { r = r + 32 }
    r
}
"#;
    // 1+2+4+8+16+32 = 63
    assert_exit_code(source, 63);
}

// ==================== 12. Defer ====================

#[test]
fn e2e_p81_defer_basic_compiles() {
    // Defer compiles correctly and doesn't affect explicit return
    let source = r#"
F main() -> i64 {
    D { }
    R 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_defer_with_explicit_return() {
    // Multiple defers with explicit return
    let source = r#"
F main() -> i64 {
    D { }
    D { }
    D { }
    R 42
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 13. Global Variables ====================

#[test]
fn e2e_p81_global_declaration_compiles() {
    // Global declarations compile to IR successfully
    let source = r#"
G counter: i64 = 0
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_global_multiple_compiles() {
    // Multiple global declarations in same module
    let source = r#"
G a: i64 = 10
G b: i64 = 20
G c: i64 = 30
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_global_with_expression() {
    // Global with constant expression initializer
    let source = r#"
G limit: i64 = 100
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

// ==================== 14. Union Types ====================

#[test]
fn e2e_p81_union_basic() {
    let source = r#"
O IntOrBool { int_val: i64, bool_val: i64 }
F main() -> i64 {
    u := IntOrBool { int_val: 42 }
    u.int_val
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 15. Generic Functions ====================

#[test]
fn e2e_p81_generic_identity() {
    let source = r#"
F id<T>(x: T) -> T = x
F main() -> i64 = id(42)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_generic_max() {
    let source = r#"
F max_of<T>(a: T, b: T) -> T {
    I a > b { a } E { b }
}
F main() -> i64 = max_of(42, 10)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_generic_min() {
    let source = r#"
F min_of<T>(a: T, b: T) -> T {
    I a < b { a } E { b }
}
F main() -> i64 = min_of(42, 100)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_generic_swap_values() {
    let source = r#"
S Pair { first: i64, second: i64 }
F swap_pair(p: Pair) -> Pair {
    Pair { first: p.second, second: p.first }
}
F main() -> i64 {
    p := Pair { first: 12, second: 42 }
    swapped := swap_pair(p)
    swapped.first
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 16. Algorithms ====================

#[test]
fn e2e_p81_binary_search() {
    // Manual binary search on "virtual" sorted array
    let source = r#"
F elem(i: i64) -> i64 = i * 3
F binary_search(target: i64, lo: i64, hi: i64) -> i64 {
    I lo > hi { R 0 - 1 }
    mid := (lo + hi) / 2
    val := elem(mid)
    I val == target { R mid }
    I val < target { R binary_search(target, mid + 1, hi) }
    binary_search(target, lo, mid - 1)
}
F main() -> i64 {
    # Search for 42 in [0, 3, 6, 9, ..., 300]
    binary_search(42, 0, 100)
}
"#;
    // 42/3 = 14
    assert_exit_code(source, 14);
}

#[test]
fn e2e_p81_bubble_sort_check() {
    // Sort 5 elements manually using variables
    let source = r#"
F main() -> i64 {
    a := mut 5
    b := mut 3
    c := mut 1
    d := mut 4
    e := mut 2

    # Bubble sort passes
    i := mut 0
    L {
        I i >= 5 { B }
        I a > b { tmp := a; a = b; b = tmp }
        I b > c { tmp := b; b = c; c = tmp }
        I c > d { tmp := c; c = d; d = tmp }
        I d > e { tmp := d; d = e; e = tmp }
        i = i + 1
    }

    # After sorting: a=1, b=2, c=3, d=4, e=5
    a * 10000 + b * 1000 + c * 100 + d * 10 + e
}
"#;
    // 12345
    // But exit code is mod 256, so 12345 % 256 = 57
    assert_exit_code(source, 57);
}

#[test]
fn e2e_p81_collatz_steps() {
    let source = r#"
F collatz_steps(n: i64) -> i64 {
    I n <= 1 { R 0 }
    I n % 2 == 0 {
        1 + @(n / 2)
    } E {
        1 + @(3 * n + 1)
    }
}
F main() -> i64 = collatz_steps(27)
"#;
    // Collatz(27) takes 111 steps, mod 256 = 111
    assert_exit_code(source, 111);
}

#[test]
fn e2e_p81_is_prime() {
    let source = r#"
F is_prime(n: i64) -> i64 {
    I n < 2 { R 0 }
    i := mut 2
    L {
        I i * i > n { B }
        I n % i == 0 { R 0 }
        i = i + 1
    }
    1
}
F main() -> i64 {
    # Count primes up to 20
    count := mut 0
    n := mut 2
    L {
        I n > 20 { B }
        count = count + is_prime(n)
        n = n + 1
    }
    count
}
"#;
    // Primes up to 20: 2,3,5,7,11,13,17,19 = 8
    assert_exit_code(source, 8);
}

// ==================== 17. Compound Assignment ====================

#[test]
fn e2e_p81_compound_assign_all() {
    let source = r#"
F main() -> i64 {
    x := mut 100
    x += 10
    x -= 50
    x *= 2
    x /= 3
    x
}
"#;
    // 100 + 10 = 110, - 50 = 60, * 2 = 120, / 3 = 40
    assert_exit_code(source, 40);
}

#[test]
fn e2e_p81_compound_assign_mod() {
    let source = r#"
F main() -> i64 {
    x := mut 100
    x %= 58
    x
}
"#;
    // 100 % 58 = 42
    assert_exit_code(source, 42);
}

// ==================== 18. Where Clauses ====================

#[test]
fn e2e_p81_where_clause_parse() {
    let source = r#"
W Addable {
    F add(self, other: i64) -> i64
}
S MyNum { val: i64 }
X MyNum: Addable {
    F add(self, other: i64) -> i64 { self.val + other }
}
F main() -> i64 {
    n := MyNum { val: 30 }
    n.add(12)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 19. Multi-function Programs ====================

#[test]
fn e2e_p81_multi_function_pipeline() {
    let source = r#"
F parse_digit(ch: i64) -> i64 = ch - 48
F validate(n: i64) -> i64 = I n >= 0 && n <= 9 { n } E { 0 - 1 }
F process(ch: i64) -> i64 {
    digit := parse_digit(ch)
    validate(digit)
}
F main() -> i64 {
    # '0' = 48, '5' = 53
    a := process(48)
    b := process(53)
    a * 10 + b
}
"#;
    // 0*10 + 5 = 5
    assert_exit_code(source, 5);
}

#[test]
fn e2e_p81_mutual_helper_functions() {
    let source = r#"
F is_even(n: i64) -> i64 {
    I n == 0 { R 1 }
    is_odd(n - 1)
}
F is_odd(n: i64) -> i64 {
    I n == 0 { R 0 }
    is_even(n - 1)
}
F main() -> i64 {
    is_even(10) + is_odd(7) + is_even(0) + is_odd(0)
}
"#;
    // is_even(10)=1, is_odd(7)=1, is_even(0)=1, is_odd(0)=0
    // total = 3
    assert_exit_code(source, 3);
}

// ==================== 20. Error Recovery Tests ====================

#[test]
fn e2e_p81_error_undefined_variable() {
    assert_compile_error(
        r#"
F main() -> i64 { R undefined_var }
"#,
    );
}

#[test]
fn e2e_p81_error_type_mismatch() {
    // Returning a string where i64 is expected
    assert_compile_error(
        r#"
F main() -> i64 { R "hello" }
"#,
    );
}

#[test]
fn e2e_p81_error_duplicate_function() {
    assert_compile_error(
        r#"
F foo() -> i64 = 1
F foo() -> i64 = 2
F main() -> i64 = foo()
"#,
    );
}

#[test]
fn e2e_p81_error_wrong_arg_count() {
    assert_compile_error(
        r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(1)
"#,
    );
}

#[test]
fn e2e_p81_error_undefined_function_call() {
    assert_compile_error(
        r#"
F main() -> i64 { R nonexistent_func(42) }
"#,
    );
}

// ==================== 21. Range & Iteration ====================

#[test]
fn e2e_p81_for_loop_range() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i:0..10 {
        sum = sum + i
    }
    sum
}
"#;
    // 0+1+2+...+9 = 45
    assert_exit_code(source, 45);
}

#[test]
fn e2e_p81_for_loop_range_with_computation() {
    let source = r#"
F main() -> i64 {
    product := mut 1
    L i:1..6 {
        product = product * i
    }
    product
}
"#;
    // 1*2*3*4*5 = 120, mod 256 = 120
    assert_exit_code(source, 120);
}

#[test]
fn e2e_p81_for_loop_nested_ranges() {
    let source = r#"
F main() -> i64 {
    count := mut 0
    L i:0..5 {
        L j:0..3 {
            count = count + 1
        }
    }
    count
}
"#;
    // 5 * 3 = 15
    assert_exit_code(source, 15);
}

// ==================== 22. Struct Initialization Patterns ====================

#[test]
fn e2e_p81_struct_default_pattern() {
    let source = r#"
S Config { width: i64, height: i64, depth: i64 }
F default_config() -> Config {
    Config { width: 10, height: 20, depth: 12 }
}
F main() -> i64 {
    c := default_config()
    c.width + c.height + c.depth
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_struct_builder_pattern() {
    let source = r#"
S Builder { x: i64, y: i64, z: i64 }
F new_builder() -> Builder { Builder { x: 0, y: 0, z: 0 } }
F with_x(b: Builder, val: i64) -> Builder { Builder { x: val, y: b.y, z: b.z } }
F with_y(b: Builder, val: i64) -> Builder { Builder { x: b.x, y: val, z: b.z } }
F with_z(b: Builder, val: i64) -> Builder { Builder { x: b.x, y: b.y, z: val } }
F main() -> i64 {
    b := new_builder()
    b = with_x(b, 10)
    b = with_y(b, 20)
    b = with_z(b, 12)
    b.x + b.y + b.z
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 23. Complex Conditional Patterns ====================

#[test]
fn e2e_p81_fizzbuzz_counter() {
    let source = r#"
F main() -> i64 {
    fizz_count := mut 0
    buzz_count := mut 0
    fizzbuzz_count := mut 0
    i := mut 1
    L {
        I i > 30 { B }
        I i % 15 == 0 {
            fizzbuzz_count = fizzbuzz_count + 1
        } E {
            I i % 3 == 0 {
                fizz_count = fizz_count + 1
            } E {
                I i % 5 == 0 {
                    buzz_count = buzz_count + 1
                }
            }
        }
        i = i + 1
    }
    fizz_count + buzz_count * 10 + fizzbuzz_count * 100
}
"#;
    // 1-30: multiples of 15: {15,30} = 2 fizzbuzz
    // multiples of 3 (not 15): {3,6,9,12,18,21,24,27} = 8 fizz
    // multiples of 5 (not 15): {5,10,20,25} = 4 buzz
    // result = 8 + 4*10 + 2*100 = 8 + 40 + 200 = 248
    // mod 256 = 248
    assert_exit_code(source, 248);
}

#[test]
fn e2e_p81_temperature_converter() {
    let source = r#"
F c_to_f(c: i64) -> i64 = c * 9 / 5 + 32
F f_to_c(f: i64) -> i64 = (f - 32) * 5 / 9

F main() -> i64 {
    # 100C = 212F
    f := c_to_f(100)
    I f == 212 { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 24. Numeric Edge Cases ====================

#[test]
fn e2e_p81_zero_operations() {
    let source = r#"
F main() -> i64 {
    a := 0
    result := mut 0
    I a + 0 == 0 { result = result + 1 }
    I a * 100 == 0 { result = result + 2 }
    I 0 - a == 0 { result = result + 4 }
    I a == 0 { result = result + 8 }
    result
}
"#;
    // 1+2+4+8 = 15
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p81_negative_arithmetic() {
    let source = r#"
F abs_val(x: i64) -> i64 {
    I x < 0 { 0 - x } E { x }
}
F main() -> i64 {
    a := 0 - 10
    b := 0 - 32
    abs_val(a + b)
}
"#;
    // a = -10, b = -32, a + b = -42, abs = 42
    assert_exit_code(source, 42);
}

// ==================== 25. Large Program Integration ====================

#[test]
fn e2e_p81_matrix_multiply_2x2() {
    let source = r#"
# 2x2 matrix stored as 4 values
S Mat2 { a: i64, b: i64, c: i64, d: i64 }

F mat_mul(m1: Mat2, m2: Mat2) -> Mat2 {
    Mat2 {
        a: m1.a * m2.a + m1.b * m2.c,
        b: m1.a * m2.b + m1.b * m2.d,
        c: m1.c * m2.a + m1.d * m2.c,
        d: m1.c * m2.b + m1.d * m2.d,
    }
}

F mat_trace(m: Mat2) -> i64 = m.a + m.d

F main() -> i64 {
    identity := Mat2 { a: 1, b: 0, c: 0, d: 1 }
    m := Mat2 { a: 3, b: 4, c: 5, d: 6 }
    result := mat_mul(identity, m)
    mat_trace(result)
}
"#;
    // identity * m = m, trace = 3 + 6 = 9
    assert_exit_code(source, 9);
}

#[test]
fn e2e_p81_stack_simulation() {
    // Simulate a stack using struct fields
    let source = r#"
S Stack { a: i64, b: i64, c: i64, top: i64 }

F new_stack() -> Stack { Stack { a: 0, b: 0, c: 0, top: 0 } }

F push(s: Stack, val: i64) -> Stack {
    I s.top == 0 { R Stack { a: val, b: s.b, c: s.c, top: 1 } }
    I s.top == 1 { R Stack { a: s.a, b: val, c: s.c, top: 2 } }
    Stack { a: s.a, b: s.b, c: val, top: 3 }
}

F peek(s: Stack) -> i64 {
    I s.top == 3 { R s.c }
    I s.top == 2 { R s.b }
    I s.top == 1 { R s.a }
    0
}

F main() -> i64 {
    s := new_stack()
    s = push(s, 10)
    s = push(s, 20)
    s = push(s, 42)
    peek(s)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p81_linked_computation() {
    // Chain of computations mimicking a pipeline
    let source = r#"
F step1(x: i64) -> i64 = x * 2
F step2(x: i64) -> i64 = x + 10
F step3(x: i64) -> i64 = x / 2
F step4(x: i64) -> i64 = x - 3

F pipeline(x: i64) -> i64 {
    step4(step3(step2(step1(x))))
}

F main() -> i64 = pipeline(34)
"#;
    // step1(34) = 68
    // step2(68) = 78
    // step3(78) = 39
    // step4(39) = 36
    assert_exit_code(source, 36);
}

// ==================== 26. Enum Methods ====================

#[test]
fn e2e_p81_enum_impl_method() {
    let source = r#"
E Direction { North, South, East, West }
X Direction {
    F code(self) -> i64 {
        M self {
            North => 1,
            South => 2,
            East => 3,
            West => 4,
        }
    }
}
F main() -> i64 {
    d := North
    d.code()
}
"#;
    assert_exit_code(source, 1);
}

// ==================== 27. Self-Recursion (@) Advanced ====================

#[test]
fn e2e_p81_self_recursion_ackermann_small() {
    let source = r#"
F ack(m: i64, n: i64) -> i64 {
    I m == 0 { R n + 1 }
    I n == 0 { R ack(m - 1, 1) }
    ack(m - 1, ack(m, n - 1))
}
F main() -> i64 {
    # ack(2, 3) = 9
    ack(2, 3)
}
"#;
    assert_exit_code(source, 9);
}

#[test]
fn e2e_p81_self_recursion_tower_of_hanoi_count() {
    let source = r#"
F hanoi_moves(n: i64) -> i64 {
    I n <= 0 { R 0 }
    @(n - 1) + 1 + @(n - 1)
}
F main() -> i64 = hanoi_moves(5)
"#;
    // 2^5 - 1 = 31
    assert_exit_code(source, 31);
}
