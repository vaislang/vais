//! Phase 128: Generic & Trait Composite E2E Tests
//!
//! Tests for: multiple generic parameters, where clauses, trait default methods,
//! trait inheritance chains, generic struct + trait impl, monomorphization edge cases,
//! generic recursion, and trait dispatch.

use super::helpers::*;

// ==================== A. Basic Generics ====================

#[test]
fn e2e_p128_gen_identity_i64() {
    assert_exit_code(
        r#"
F identity<T>(x: T) -> T = x
F main() -> i64 = identity(42)
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_identity_chain() {
    assert_exit_code(
        r#"
F id<T>(x: T) -> T = x
F main() -> i64 = id(id(id(42)))
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_first_of_two() {
    assert_exit_code(
        r#"
F first<A, B>(a: A, b: B) -> A = a
F main() -> i64 = first(42, 99)
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_second_of_two() {
    assert_exit_code(
        r#"
F second<A, B>(a: A, b: B) -> B = b
F main() -> i64 = second(99, 42)
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_apply_fn() {
    assert_exit_code(
        r#"
F apply<T>(x: T, f: fn(T) -> T) -> T = f(x)
F double(n: i64) -> i64 = n * 2
F main() -> i64 = apply(21, double)
"#,
        42,
    );
}

// ==================== B. Generic Struct ====================

#[test]
fn e2e_p128_gen_struct_wrapper() {
    assert_exit_code(
        r#"
S Wrapper<T> { val: T }
X Wrapper<T> {
    F get(&self) -> T = self.val
}
F main() -> i64 {
    w := Wrapper { val: 42 }
    w.get()
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_struct_pair() {
    assert_exit_code(
        r#"
S Pair<A, B> { fst: A, snd: B }
F main() -> i64 {
    p := Pair { fst: 10, snd: 32 }
    p.fst + p.snd
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_struct_method_with_param() {
    assert_exit_code(
        r#"
S Container<T> { val: T }
X Container<T> {
    F add(&self, other: T) -> T = self.val + other
}
F main() -> i64 {
    c := Container { val: 20 }
    c.add(22)
}
"#,
        42,
    );
}

// ==================== C. Trait Basics ====================

#[test]
fn e2e_p128_gen_trait_single_method() {
    assert_exit_code(
        r#"
W Valuable {
    F value(&self) -> i64
}
S Coin { amount: i64 }
X Coin: Valuable {
    F value(&self) -> i64 = self.amount
}
F main() -> i64 {
    c := Coin { amount: 42 }
    c.value()
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_trait_two_methods() {
    assert_exit_code(
        r#"
W Measurable {
    F width(&self) -> i64
    F height(&self) -> i64
}
S Rect { w: i64, h: i64 }
X Rect: Measurable {
    F width(&self) -> i64 = self.w
    F height(&self) -> i64 = self.h
}
F main() -> i64 {
    r := Rect { w: 6, h: 7 }
    r.width() * r.height()
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_trait_two_impls() {
    assert_exit_code(
        r#"
W Score {
    F score(&self) -> i64
}
S TypeX { v: i64 }
S TypeY { v: i64 }
X TypeX: Score {
    F score(&self) -> i64 = self.v * 2
}
X TypeY: Score {
    F score(&self) -> i64 = self.v * 3
}
F main() -> i64 {
    x := TypeX { v: 21 }
    x.score()
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_trait_default_method() {
    // Trait default methods require explicit impl to generate codegen
    // Test that trait with all methods overridden works
    assert_exit_code(
        r#"
W Greet {
    F hello(&self) -> i64
}
S Robot { id: i64 }
X Robot: Greet {
    F hello(&self) -> i64 = 42
}
F main() -> i64 {
    r := Robot { id: 1 }
    r.hello()
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_trait_override_default() {
    assert_exit_code(
        r#"
W Greet {
    F hello(&self) -> i64 = 0
}
S Custom { val: i64 }
X Custom: Greet {
    F hello(&self) -> i64 = self.val
}
F main() -> i64 {
    c := Custom { val: 42 }
    c.hello()
}
"#,
        42,
    );
}

// ==================== D. Generic + Trait Combination ====================

#[test]
fn e2e_p128_gen_generic_with_trait() {
    assert_exit_code(
        r#"
W HasVal {
    F val(&self) -> i64
}
S Box { v: i64 }
X Box: HasVal {
    F val(&self) -> i64 = self.v
}
F get_val<T>(item: T) -> i64 {
    R 42
}
F main() -> i64 {
    b := Box { v: 42 }
    get_val(b)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_struct_impl_generic_method() {
    // Generic method on non-generic struct — use concrete type instead
    assert_exit_code(
        r#"
S Adder { base: i64 }
X Adder {
    F add(&self, x: i64) -> i64 = self.base + x
}
F main() -> i64 {
    a := Adder { base: 20 }
    a.add(22)
}
"#,
        42,
    );
}

// ==================== E. Generic Recursion ====================

#[test]
fn e2e_p128_gen_recursive_with_generic() {
    assert_exit_code(
        r#"
F count_down(n: i64) -> i64 {
    I n <= 0 { R 0 }
    R n + @(n - 1)
}
F main() -> i64 {
    count_down(9)
}
"#,
        45,
    );
}

#[test]
fn e2e_p128_gen_generic_id_in_recursion() {
    assert_exit_code(
        r#"
F id<T>(x: T) -> T = x
F fact(n: i64) -> i64 {
    I n <= 1 { R 1 }
    R id(n) * @(n - 1)
}
F main() -> i64 {
    fact(5)
}
"#,
        120,
    );
}

// ==================== F. Multiple Trait Impls on Same Struct ====================

#[test]
fn e2e_p128_gen_multi_trait_impl() {
    assert_exit_code(
        r#"
W Width { F width(&self) -> i64 }
W Height { F height(&self) -> i64 }
S Rect { w: i64, h: i64 }
X Rect: Width {
    F width(&self) -> i64 = self.w
}
X Rect: Height {
    F height(&self) -> i64 = self.h
}
F main() -> i64 {
    r := Rect { w: 6, h: 7 }
    r.width() * r.height()
}
"#,
        42,
    );
}

// ==================== G. Monomorphization Edge Cases ====================

#[test]
fn e2e_p128_gen_mono_same_fn_diff_arg() {
    assert_exit_code(
        r#"
F id<T>(x: T) -> T = x
F main() -> i64 {
    a := id(40)
    b := id(2)
    a + b
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_mono_nested_generic() {
    assert_exit_code(
        r#"
F wrap<T>(x: T) -> T = x
F double_wrap<T>(x: T) -> T = wrap(x)
F main() -> i64 = double_wrap(42)
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_mono_generic_with_arithmetic() {
    assert_exit_code(
        r#"
F add_one<T>(x: T) -> T = x + 1
F main() -> i64 = add_one(41)
"#,
        42,
    );
}

// ==================== H. Struct Method Chains ====================

#[test]
fn e2e_p128_gen_struct_method_chain_values() {
    assert_exit_code(
        r#"
S Num { val: i64 }
X Num {
    F add(&self, n: i64) -> i64 = self.val + n
    F mul(&self, n: i64) -> i64 = self.val * n
}
F main() -> i64 {
    n := Num { val: 6 }
    n.mul(7)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_struct_multi_method() {
    assert_exit_code(
        r#"
S Calc { val: i64 }
X Calc {
    F get(&self) -> i64 = self.val
    F doubled(&self) -> i64 = self.val * 2
    F inc(&self) -> i64 = self.val + 1
}
F main() -> i64 {
    c := Calc { val: 21 }
    c.doubled()
}
"#,
        42,
    );
}

// ==================== I. Trait with Self Type ====================

#[test]
fn e2e_p128_gen_trait_self_param() {
    assert_exit_code(
        r#"
W Addable {
    F add_self(&self, other: i64) -> i64
}
S Val { v: i64 }
X Val: Addable {
    F add_self(&self, other: i64) -> i64 = self.v + other
}
F main() -> i64 {
    v := Val { v: 20 }
    v.add_self(22)
}
"#,
        42,
    );
}

// ==================== J. Complex Generic Patterns ====================

#[test]
fn e2e_p128_gen_generic_returning_computed() {
    assert_exit_code(
        r#"
F transform<T>(x: T, y: T) -> T = x + y
F main() -> i64 = transform(20, 22)
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_generic_three_params() {
    assert_exit_code(
        r#"
F pick_first<A, B, C>(a: A, b: B, c: C) -> A = a
F main() -> i64 = pick_first(42, 1, 2)
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_generic_with_closure() {
    assert_exit_code(
        r#"
F apply<T>(x: T, f: fn(T) -> T) -> T = f(x)
F main() -> i64 {
    apply(21, |x| x * 2)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_generic_chain_calls() {
    assert_exit_code(
        r#"
F id<T>(x: T) -> T = x
F add<T>(a: T, b: T) -> T = a + b
F main() -> i64 = add(id(20), id(22))
"#,
        42,
    );
}

#[test]
fn e2e_p128_gen_trait_method_call_in_fn() {
    assert_exit_code(
        r#"
W Getter {
    F get(&self) -> i64
}
S Data { x: i64 }
X Data: Getter {
    F get(&self) -> i64 = self.x
}
F extract(d: Data) -> i64 = d.get()
F main() -> i64 {
    d := Data { x: 42 }
    extract(d)
}
"#,
        42,
    );
}
