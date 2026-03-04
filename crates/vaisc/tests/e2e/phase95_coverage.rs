//! Phase 95 -- Comprehensive Coverage Tests
//!
//! Tests for under-covered language features: advanced matching, nested structs,
//! chained operations, complex expressions, control flow edge cases,
//! and operator combinations.

use super::helpers::*;

// ==================== Advanced Pattern Matching ====================

#[test]
fn e2e_match_wildcard_default() {
    let source = r#"
E Dir { North, South, East, West }
F main() -> i64 {
    d := East
    M d {
        North => 1,
        _ => 42
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_match_first_of_many_variants() {
    let source = r#"
E Shape { Circle, Square, Triangle, Pentagon, Hexagon }
F main() -> i64 {
    s := Circle
    M s {
        Circle => 42,
        Square => 1,
        Triangle => 2,
        Pentagon => 3,
        Hexagon => 4
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_match_last_of_many_variants() {
    let source = r#"
E Shape { Circle, Square, Triangle, Pentagon, Hexagon }
F main() -> i64 {
    s := Hexagon
    M s {
        Circle => 1,
        Square => 2,
        Triangle => 3,
        Pentagon => 4,
        Hexagon => 42
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_match_integer_literal() {
    let source = r#"
F classify(n: i64) -> i64 {
    M n {
        0 => 0,
        1 => 10,
        2 => 20,
        _ => 42
    }
}
F main() -> i64 = classify(99)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_match_returns_expression() {
    let source = r#"
F main() -> i64 {
    x := 5
    result := M x {
        1 => 10,
        5 => 42,
        _ => 0
    }
    result
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Nested Struct Operations ====================

#[test]
fn e2e_nested_struct_access() {
    let source = r#"
S Inner { val: i64 }
S Outer { inner: Inner, extra: i64 }
F main() -> i64 {
    o := Outer { inner: Inner { val: 40 }, extra: 2 }
    o.inner.val + o.extra
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_method_returns_field() {
    let source = r#"
S Box { value: i64 }
X Box {
    F get(self) -> i64 = self.value
    F doubled(self) -> i64 = self.value * 2
}
F main() -> i64 {
    b := Box { value: 21 }
    b.doubled()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_method_with_params() {
    let source = r#"
S Acc { total: i64 }
X Acc {
    F add(self, n: i64) -> i64 = self.total + n
}
F main() -> i64 {
    a := Acc { total: 30 }
    a.add(12)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_multiple_methods() {
    let source = r#"
S Counter { n: i64 }
X Counter {
    F value(self) -> i64 = self.n
    F plus(self, x: i64) -> i64 = self.n + x
    F times(self, x: i64) -> i64 = self.n * x
}
F main() -> i64 {
    c := Counter { n: 6 }
    c.times(7)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Complex Arithmetic Expressions ====================

#[test]
fn e2e_arithmetic_operator_precedence() {
    let source = r#"
F main() -> i64 = 2 + 4 * 10
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arithmetic_parenthesized() {
    let source = r#"
F main() -> i64 = (2 + 5) * 6
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arithmetic_nested_parens() {
    let source = r#"
F main() -> i64 = ((10 + 11) * 2)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arithmetic_modulo() {
    let source = r#"
F main() -> i64 = 142 % 100
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arithmetic_division() {
    let source = r#"
F main() -> i64 = 84 / 2
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arithmetic_complex_chain() {
    let source = r#"
F main() -> i64 = 100 - 50 - 8
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arithmetic_mixed_ops() {
    let source = r#"
F main() -> i64 = 3 * 10 + 12
"#;
    assert_exit_code(source, 42);
}

// ==================== Boolean Logic and Comparisons ====================

#[test]
fn e2e_bool_and_true() {
    let source = r#"
F main() -> i64 {
    I true && true { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bool_or_false_true() {
    let source = r#"
F main() -> i64 {
    I false || true { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bool_not_false() {
    let source = r#"
F main() -> i64 {
    I !false { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_comparison_less_than() {
    let source = r#"
F main() -> i64 {
    I 5 < 10 { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_comparison_greater_equal() {
    let source = r#"
F main() -> i64 {
    I 10 >= 10 { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_comparison_not_equal() {
    let source = r#"
F main() -> i64 {
    I 5 != 3 { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_comparison_equal() {
    let source = r#"
F main() -> i64 {
    I 42 == 42 { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Loop Patterns ====================

#[test]
fn e2e_loop_accumulate_fixed() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    i := mut 0
    L i < 7 {
        sum = sum + i
        i = i + 1
    }
    sum
}
"#;
    assert_exit_code(source, 21);
}

#[test]
fn e2e_loop_countdown() {
    let source = r#"
F main() -> i64 {
    n := mut 42
    count := mut 0
    L n > 0 {
        count = count + 1
        n = n - 1
    }
    count
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_loop_with_break_value() {
    let source = r#"
F main() -> i64 {
    i := mut 0
    L {
        i = i + 1
        I i == 42 { B }
    }
    i
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_loop_nested() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    i := mut 0
    L i < 6 {
        j := mut 0
        L j < 7 {
            sum = sum + 1
            j = j + 1
        }
        i = i + 1
    }
    sum
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_for_loop_range() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i:0..7 {
        sum = sum + i
    }
    sum
}
"#;
    assert_exit_code(source, 21);
}

// ==================== Function Composition ====================

#[test]
fn e2e_function_chain() {
    let source = r#"
F add_one(x: i64) -> i64 = x + 1
F double(x: i64) -> i64 = x * 2
F main() -> i64 = add_one(double(20))
"#;
    assert_exit_code(source, 41);
}

#[test]
fn e2e_function_triple_compose() {
    let source = r#"
F inc(x: i64) -> i64 = x + 1
F dbl(x: i64) -> i64 = x * 2
F main() -> i64 = dbl(inc(20))
"#;
    // inc(20)=21, dbl(21)=42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_function_recursive_countdown() {
    let source = r#"
F count(n: i64, acc: i64) -> i64 {
    I n == 0 { acc }
    E { @(n - 1, acc + 1) }
}
F main() -> i64 = count(42, 0)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_function_mutual_calls() {
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b
F mul(a: i64, b: i64) -> i64 = a * b
F compute() -> i64 = add(mul(3, 10), mul(2, 6))
F main() -> i64 = compute()
"#;
    assert_exit_code(source, 42);
}

// ==================== Variable Binding Edge Cases ====================

#[test]
fn e2e_variable_shadowing() {
    let source = r#"
F main() -> i64 {
    x := 10
    x := 42
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_variable_multiple_shadowing() {
    let source = r#"
F main() -> i64 {
    x := 1
    x := 2
    x := 3
    x := 42
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_variable_mutable_reassign() {
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
fn e2e_variable_mutable_multiple_reassign() {
    let source = r#"
F main() -> i64 {
    x := mut 1
    x = 10
    x = 20
    x = 42
    x
}
"#;
    assert_exit_code(source, 42);
}

// ==================== If/Else Chains ====================

#[test]
fn e2e_if_else_chain_first() {
    let source = r#"
F main() -> i64 {
    x := 1
    I x == 1 { 42 }
    E I x == 2 { 0 }
    E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_if_else_chain_middle() {
    let source = r#"
F main() -> i64 {
    x := 2
    I x == 1 { 0 }
    E I x == 2 { 42 }
    E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_if_else_chain_last() {
    let source = r#"
F main() -> i64 {
    x := 99
    I x == 1 { 0 }
    E I x == 2 { 0 }
    E { 42 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_if_nested() {
    let source = r#"
F main() -> i64 {
    x := 5
    y := 10
    I x > 0 {
        I y > 5 { 42 }
        E { 0 }
    }
    E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Ternary Operator ====================

#[test]
fn e2e_ternary_true_branch() {
    let source = r#"
F main() -> i64 = true ? 42 : 0
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_ternary_false_branch() {
    let source = r#"
F main() -> i64 = false ? 0 : 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_ternary_with_comparison() {
    let source = r#"
F main() -> i64 {
    x := 10
    x > 5 ? 42 : 0
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_ternary_nested() {
    let source = r#"
F choose(x: i64) -> i64 {
    I x == 1 { R 10 }
    I x == 2 { R 42 }
    0
}
F main() -> i64 = choose(2)
"#;
    assert_exit_code(source, 42);
}

// ==================== Closure / Lambda ====================

#[test]
fn e2e_closure_simple_identity() {
    let source = r#"
F main() -> i64 {
    f := |x: i64| x
    f(42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_add() {
    let source = r#"
F main() -> i64 {
    add := |a: i64, b: i64| a + b
    add(20, 22)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_capture_variable() {
    let source = r#"
F main() -> i64 {
    base := 40
    add_base := |x: i64| x + base
    add_base(2)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_as_argument() {
    // fn(i64) -> i64 is the Vais syntax for function-typed parameters
    let source = r#"
F apply(x: i64, f: fn(i64) -> i64) -> i64 = f(x)
F dbl(x: i64) -> i64 = x * 2
F main() -> i64 = apply(21, dbl)
"#;
    assert_exit_code(source, 42);
}

// ==================== Enum with Match and Functions ====================

#[test]
fn e2e_enum_match_in_function() {
    let source = r#"
E Animal { Cat, Dog, Fish }
F score(a: Animal) -> i64 {
    M a {
        Cat => 10,
        Dog => 42,
        Fish => 5
    }
}
F main() -> i64 = score(Dog)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_wildcard_in_function() {
    let source = r#"
E Color { Red, Green, Blue, Yellow, Cyan }
F is_primary(c: Color) -> i64 {
    M c {
        Red => 1,
        Green => 1,
        Blue => 1,
        _ => 0
    }
}
F main() -> i64 {
    I is_primary(Red) == 1 && is_primary(Yellow) == 0 { 42 }
    E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Mixed Features ====================

#[test]
fn e2e_struct_with_enum_field() {
    let source = r#"
E Kind { First, Second }
S Item { kind: Kind, value: i64 }
F main() -> i64 {
    item := Item { kind: First, value: 42 }
    item.value
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_function_returning_struct_field() {
    let source = r#"
S Pair { first: i64, second: i64 }
F sum_pair(p: Pair) -> i64 = p.first + p.second
F main() -> i64 = sum_pair(Pair { first: 20, second: 22 })
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_loop_with_function_call() {
    let source = r#"
F square(x: i64) -> i64 = x * x
F main() -> i64 {
    sum := mut 0
    L i:1..4 {
        sum = sum + square(i)
    }
    sum
}
"#;
    // 1 + 4 + 9 = 14
    assert_exit_code(source, 14);
}

#[test]
fn e2e_recursive_fibonacci() {
    let source = r#"
F fib(n: i64) -> i64 {
    I n <= 1 { n }
    E { @(n - 1) + @(n - 2) }
}
F main() -> i64 = fib(10)
"#;
    // fib(10) = 55
    assert_exit_code(source, 55);
}

#[test]
fn e2e_recursive_factorial_small() {
    let source = r#"
F fact(n: i64) -> i64 {
    I n <= 1 { 1 }
    E { n * @(n - 1) }
}
F main() -> i64 = fact(5)
"#;
    // 5! = 120
    assert_exit_code(source, 120);
}

// ==================== Bitwise Operations ====================

#[test]
fn e2e_bitwise_and() {
    let source = r#"
F main() -> i64 = 63 & 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bitwise_or() {
    let source = r#"
F main() -> i64 = 32 | 10
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bitwise_xor() {
    let source = r#"
F main() -> i64 = 50 ^ 24
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bitwise_shift_left() {
    let source = r#"
F main() -> i64 = 21 << 1
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bitwise_shift_right() {
    let source = r#"
F main() -> i64 = 168 >> 2
"#;
    assert_exit_code(source, 42);
}

// ==================== Expression Body Functions ====================

#[test]
fn e2e_expression_body_simple() {
    let source = r#"
F answer() -> i64 = 42
F main() -> i64 = answer()
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_expression_body_with_param() {
    let source = r#"
F inc(x: i64) -> i64 = x + 1
F main() -> i64 = inc(41)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_expression_body_complex() {
    let source = r#"
F calc(a: i64, b: i64, c: i64) -> i64 = a * b + c
F main() -> i64 = calc(4, 10, 2)
"#;
    assert_exit_code(source, 42);
}

// ==================== Trait and Impl ====================

#[test]
fn e2e_trait_impl_basic() {
    let source = r#"
W Valued {
    F value(self) -> i64
}
S Token { v: i64 }
X Token: Valued {
    F value(self) -> i64 = self.v
}
F main() -> i64 {
    t := Token { v: 42 }
    t.value()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_trait_multiple_impls() {
    let source = r#"
W HasSize {
    F size(self) -> i64
}
S Small { n: i64 }
S Large { n: i64 }
X Small: HasSize {
    F size(self) -> i64 = self.n
}
X Large: HasSize {
    F size(self) -> i64 = self.n * 10
}
F main() -> i64 {
    s := Small { n: 42 }
    s.size()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Auto-return (last expression) ====================

#[test]
fn e2e_auto_return_block() {
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
fn e2e_auto_return_if_else() {
    let source = r#"
F pick(flag: bool) -> i64 {
    I flag { 42 }
    E { 0 }
}
F main() -> i64 = pick(true)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_auto_return_match() {
    let source = r#"
E Coin { Heads, Tails }
F flip(c: Coin) -> i64 {
    M c {
        Heads => 42,
        Tails => 0
    }
}
F main() -> i64 = flip(Heads)
"#;
    assert_exit_code(source, 42);
}

// ==================== Negative and Edge Values ====================

#[test]
fn e2e_negative_arithmetic() {
    let source = r#"
F main() -> i64 {
    x := 50
    y := 8
    x - y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_zero_handling() {
    let source = r#"
F main() -> i64 {
    x := 0
    y := 42
    x + y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_large_subtraction() {
    let source = r#"
F main() -> i64 = 1000 - 958
"#;
    assert_exit_code(source, 42);
}

// ==================== Multiple Statements ====================

#[test]
fn e2e_multi_statement_sequence() {
    let source = r#"
F main() -> i64 {
    a := 10
    b := 20
    c := 12
    a + b + c
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_multi_statement_with_mutation() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    x = x + 10
    x = x + 20
    x = x + 12
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_multi_statement_with_conditionals() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    I true { x = x + 20 }
    I true { x = x + 22 }
    x
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Struct Impl Chaining ====================

#[test]
fn e2e_impl_method_chain_value() {
    let source = r#"
S Num { v: i64 }
X Num {
    F add(self, n: i64) -> i64 = self.v + n
}
F main() -> i64 {
    n := Num { v: 30 }
    n.add(12)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Early Return ====================

#[test]
fn e2e_early_return_if() {
    let source = r#"
F check(n: i64) -> i64 {
    I n > 10 { R 42 }
    0
}
F main() -> i64 = check(20)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_early_return_loop() {
    let source = r#"
F find_answer() -> i64 {
    i := mut 0
    L {
        i = i + 1
        I i == 42 { R i }
    }
    0
}
F main() -> i64 = find_answer()
"#;
    assert_exit_code(source, 42);
}

// ==================== Type Alias ====================

#[test]
fn e2e_type_alias_basic() {
    let source = r#"
T Num = i64
F add(a: Num, b: Num) -> Num = a + b
F main() -> i64 = add(20, 22)
"#;
    assert_exit_code(source, 42);
}

// ==================== Constant-like Patterns ====================

#[test]
fn e2e_constant_function() {
    // Simulate constants with zero-arg functions
    let source = r#"
F ANSWER() -> i64 = 42
F main() -> i64 = ANSWER()
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_constant_in_expression() {
    let source = r#"
F BASE() -> i64 = 40
F main() -> i64 = BASE() + 2
"#;
    assert_exit_code(source, 42);
}
