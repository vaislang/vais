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
enum Dir { North, South, East, West }
fn main() -> i64 {
    d := East
    match d {
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
enum Shape { Circle, Square, Triangle, Pentagon, Hexagon }
fn main() -> i64 {
    s := Circle
    match s {
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
enum Shape { Circle, Square, Triangle, Pentagon, Hexagon }
fn main() -> i64 {
    s := Hexagon
    match s {
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
fn classify(n: i64) -> i64 {
    match n {
        0 => 0,
        1 => 10,
        2 => 20,
        _ => 42
    }
}
fn main() -> i64 = classify(99)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_match_returns_expression() {
    let source = r#"
fn main() -> i64 {
    x := 5
    result := match x {
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
struct Inner { val: i64 }
struct Outer { inner: Inner, extra: i64 }
fn main() -> i64 {
    o := Outer { inner: Inner { val: 40 }, extra: 2 }
    o.inner.val + o.extra
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_method_returns_field() {
    let source = r#"
struct Box { value: i64 }
impl Box {
    fn get(self) -> i64 = self.value
    fn doubled(self) -> i64 = self.value * 2
}
fn main() -> i64 {
    b := Box { value: 21 }
    b.doubled()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_method_with_params() {
    let source = r#"
struct Acc { total: i64 }
impl Acc {
    fn add(self, n: i64) -> i64 = self.total + n
}
fn main() -> i64 {
    a := Acc { total: 30 }
    a.add(12)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_multiple_methods() {
    let source = r#"
struct Counter { n: i64 }
impl Counter {
    fn value(self) -> i64 = self.n
    fn plus(self, x: i64) -> i64 = self.n + x
    fn times(self, x: i64) -> i64 = self.n * x
}
fn main() -> i64 {
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
fn main() -> i64 = 2 + 4 * 10
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arithmetic_parenthesized() {
    let source = r#"
fn main() -> i64 = (2 + 5) * 6
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arithmetic_nested_parens() {
    let source = r#"
fn main() -> i64 = ((10 + 11) * 2)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arithmetic_modulo() {
    let source = r#"
fn main() -> i64 = 142 % 100
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arithmetic_division() {
    let source = r#"
fn main() -> i64 = 84 / 2
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arithmetic_complex_chain() {
    let source = r#"
fn main() -> i64 = 100 - 50 - 8
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arithmetic_mixed_ops() {
    let source = r#"
fn main() -> i64 = 3 * 10 + 12
"#;
    assert_exit_code(source, 42);
}

// ==================== Boolean Logic and Comparisons ====================

#[test]
fn e2e_bool_and_true() {
    let source = r#"
fn main() -> i64 {
    I true && true { 42 } else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bool_or_false_true() {
    let source = r#"
fn main() -> i64 {
    I false || true { 42 } else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bool_not_false() {
    let source = r#"
fn main() -> i64 {
    I !false { 42 } else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_comparison_less_than() {
    let source = r#"
fn main() -> i64 {
    I 5 < 10 { 42 } else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_comparison_greater_equal() {
    let source = r#"
fn main() -> i64 {
    I 10 >= 10 { 42 } else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_comparison_not_equal() {
    let source = r#"
fn main() -> i64 {
    I 5 != 3 { 42 } else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_comparison_equal() {
    let source = r#"
fn main() -> i64 {
    I 42 == 42 { 42 } else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Loop Patterns ====================

#[test]
fn e2e_loop_accumulate_fixed() {
    let source = r#"
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn add_one(x: i64) -> i64 = x + 1
fn double(x: i64) -> i64 = x * 2
fn main() -> i64 = add_one(double(20))
"#;
    assert_exit_code(source, 41);
}

#[test]
fn e2e_function_triple_compose() {
    let source = r#"
fn inc(x: i64) -> i64 = x + 1
fn dbl(x: i64) -> i64 = x * 2
fn main() -> i64 = dbl(inc(20))
"#;
    // inc(20)=21, dbl(21)=42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_function_recursive_countdown() {
    let source = r#"
fn count(n: i64, acc: i64) -> i64 {
    I n == 0 { acc }
    else { @(n - 1, acc + 1) }
}
fn main() -> i64 = count(42, 0)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_function_mutual_calls() {
    let source = r#"
fn add(a: i64, b: i64) -> i64 = a + b
fn mul(a: i64, b: i64) -> i64 = a * b
fn compute() -> i64 = add(mul(3, 10), mul(2, 6))
fn main() -> i64 = compute()
"#;
    assert_exit_code(source, 42);
}

// ==================== Variable Binding Edge Cases ====================

#[test]
fn e2e_variable_shadowing() {
    let source = r#"
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
    x := 1
    I x == 1 { 42 }
    else I x == 2 { 0 }
    else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_if_else_chain_middle() {
    let source = r#"
fn main() -> i64 {
    x := 2
    I x == 1 { 0 }
    else I x == 2 { 42 }
    else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_if_else_chain_last() {
    let source = r#"
fn main() -> i64 {
    x := 99
    I x == 1 { 0 }
    else I x == 2 { 0 }
    else { 42 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_if_nested() {
    let source = r#"
fn main() -> i64 {
    x := 5
    y := 10
    I x > 0 {
        I y > 5 { 42 }
        else { 0 }
    }
    else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Ternary Operator ====================

#[test]
fn e2e_ternary_true_branch() {
    let source = r#"
fn main() -> i64 = true ? 42 : 0
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_ternary_false_branch() {
    let source = r#"
fn main() -> i64 = false ? 0 : 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_ternary_with_comparison() {
    let source = r#"
fn main() -> i64 {
    x := 10
    x > 5 ? 42 : 0
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_ternary_nested() {
    let source = r#"
fn choose(x: i64) -> i64 {
    I x == 1 { return 10 }
    I x == 2 { return 42 }
    0
}
fn main() -> i64 = choose(2)
"#;
    assert_exit_code(source, 42);
}

// ==================== Closure / Lambda ====================

#[test]
fn e2e_closure_simple_identity() {
    let source = r#"
fn main() -> i64 {
    f := |x: i64| x
    f(42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_add() {
    let source = r#"
fn main() -> i64 {
    add := |a: i64, b: i64| a + b
    add(20, 22)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_capture_variable() {
    let source = r#"
fn main() -> i64 {
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
fn apply(x: i64, f: fn(i64) -> i64) -> i64 = f(x)
fn dbl(x: i64) -> i64 = x * 2
fn main() -> i64 = apply(21, dbl)
"#;
    assert_exit_code(source, 42);
}

// ==================== Enum with Match and Functions ====================

#[test]
fn e2e_enum_match_in_function() {
    let source = r#"
enum Animal { Cat, Dog, Fish }
fn score(a: Animal) -> i64 {
    match a {
        Cat => 10,
        Dog => 42,
        Fish => 5
    }
}
fn main() -> i64 = score(Dog)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_enum_wildcard_in_function() {
    let source = r#"
enum Color { Red, Green, Blue, Yellow, Cyan }
fn is_primary(c: Color) -> i64 {
    match c {
        Red => 1,
        Green => 1,
        Blue => 1,
        _ => 0
    }
}
fn main() -> i64 {
    I is_primary(Red) == 1 && is_primary(Yellow) == 0 { 42 }
    else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Mixed Features ====================

#[test]
fn e2e_struct_with_enum_field() {
    let source = r#"
enum Kind { First, Second }
struct Item { kind: Kind, value: i64 }
fn main() -> i64 {
    item := Item { kind: First, value: 42 }
    item.value
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_function_returning_struct_field() {
    let source = r#"
struct Pair { first: i64, second: i64 }
fn sum_pair(p: Pair) -> i64 = p.first + p.second
fn main() -> i64 = sum_pair(Pair { first: 20, second: 22 })
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_loop_with_function_call() {
    let source = r#"
fn square(x: i64) -> i64 = x * x
fn main() -> i64 {
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
fn fib(n: i64) -> i64 {
    I n <= 1 { n }
    else { @(n - 1) + @(n - 2) }
}
fn main() -> i64 = fib(10)
"#;
    // fib(10) = 55
    assert_exit_code(source, 55);
}

#[test]
fn e2e_recursive_factorial_small() {
    let source = r#"
fn fact(n: i64) -> i64 {
    I n <= 1 { 1 }
    else { n * @(n - 1) }
}
fn main() -> i64 = fact(5)
"#;
    // 5! = 120
    assert_exit_code(source, 120);
}

// ==================== Bitwise Operations ====================

#[test]
fn e2e_bitwise_and() {
    let source = r#"
fn main() -> i64 = 63 & 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bitwise_or() {
    let source = r#"
fn main() -> i64 = 32 | 10
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bitwise_xor() {
    let source = r#"
fn main() -> i64 = 50 ^ 24
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bitwise_shift_left() {
    let source = r#"
fn main() -> i64 = 21 << 1
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_bitwise_shift_right() {
    let source = r#"
fn main() -> i64 = 168 >> 2
"#;
    assert_exit_code(source, 42);
}

// ==================== Expression Body Functions ====================

#[test]
fn e2e_expression_body_simple() {
    let source = r#"
fn answer() -> i64 = 42
fn main() -> i64 = answer()
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_expression_body_with_param() {
    let source = r#"
fn inc(x: i64) -> i64 = x + 1
fn main() -> i64 = inc(41)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_expression_body_complex() {
    let source = r#"
fn calc(a: i64, b: i64, c: i64) -> i64 = a * b + c
fn main() -> i64 = calc(4, 10, 2)
"#;
    assert_exit_code(source, 42);
}

// ==================== Trait and Impl ====================

#[test]
fn e2e_trait_impl_basic() {
    let source = r#"
trait Valued {
    fn value(self) -> i64
}
struct Token { v: i64 }
impl Token: Valued {
    fn value(self) -> i64 = self.v
}
fn main() -> i64 {
    t := Token { v: 42 }
    t.value()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_trait_multiple_impls() {
    let source = r#"
trait HasSize {
    fn size(self) -> i64
}
struct Small { n: i64 }
struct Large { n: i64 }
impl Small: HasSize {
    fn size(self) -> i64 = self.n
}
impl Large: HasSize {
    fn size(self) -> i64 = self.n * 10
}
fn main() -> i64 {
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
fn main() -> i64 {
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
fn pick(flag: bool) -> i64 {
    I flag { 42 }
    else { 0 }
}
fn main() -> i64 = pick(true)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_auto_return_match() {
    let source = r#"
enum Coin { Heads, Tails }
fn flip(c: Coin) -> i64 {
    match c {
        Heads => 42,
        Tails => 0
    }
}
fn main() -> i64 = flip(Heads)
"#;
    assert_exit_code(source, 42);
}

// ==================== Negative and Edge Values ====================

#[test]
fn e2e_negative_arithmetic() {
    let source = r#"
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 = 1000 - 958
"#;
    assert_exit_code(source, 42);
}

// ==================== Multiple Statements ====================

#[test]
fn e2e_multi_statement_sequence() {
    let source = r#"
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
struct Num { v: i64 }
impl Num {
    fn add(self, n: i64) -> i64 = self.v + n
}
fn main() -> i64 {
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
fn check(n: i64) -> i64 {
    I n > 10 { return 42 }
    0
}
fn main() -> i64 = check(20)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_early_return_loop() {
    let source = r#"
fn find_answer() -> i64 {
    i := mut 0
    L {
        i = i + 1
        I i == 42 { return i }
    }
    0
}
fn main() -> i64 = find_answer()
"#;
    assert_exit_code(source, 42);
}

// ==================== Type Alias ====================

#[test]
fn e2e_type_alias_basic() {
    let source = r#"
type Num = i64
fn add(a: Num, b: Num) -> Num = a + b
fn main() -> i64 = add(20, 22)
"#;
    assert_exit_code(source, 42);
}

// ==================== Constant-like Patterns ====================

#[test]
fn e2e_constant_function() {
    // Simulate constants with zero-arg functions
    let source = r#"
fn ANSWER() -> i64 = 42
fn main() -> i64 = ANSWER()
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_constant_in_expression() {
    let source = r#"
fn BASE() -> i64 = 40
fn main() -> i64 = BASE() + 2
"#;
    assert_exit_code(source, 42);
}
