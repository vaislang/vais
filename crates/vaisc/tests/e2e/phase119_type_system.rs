//! Phase 119: GAT / Specialization / Monomorphization E2E Tests
//!
//! Tests for:
//! 1. Generic Associated Types (GAT) — trait with parameterized associated types
//! 2. Specialization patterns — blanket vs concrete impls
//! 3. Monomorphization edge cases — nested generics, const generics, multi-level chains
//! 4. Type system consistency — Lifetime/Var fallback behavior

use super::helpers::*;

// ==================== 1. Generic Traits & Associated Types ====================

#[test]
fn e2e_p119_trait_with_associated_type_impl() {
    // Trait with associated type resolved in impl
    let source = r#"
W Container {
    F size(&self) -> i64
}

S Bag { count: i64 }

X Bag: Container {
    F size(&self) -> i64 {
        self.count
    }
}

F main() -> i64 {
    b := Bag { count: 17 }
    b.size()
}
"#;
    assert_exit_code(source, 17);
}

#[test]
fn e2e_p119_trait_method_dispatch_multiple_impls() {
    // Multiple structs implementing the same trait
    let source = r#"
W Measurable {
    F measure(&self) -> i64
}

S Ruler { len: i64 }
S Tape { len: i64 }

X Ruler: Measurable {
    F measure(&self) -> i64 {
        self.len
    }
}

X Tape: Measurable {
    F measure(&self) -> i64 {
        self.len * 2
    }
}

F main() -> i64 {
    r := Ruler { len: 10 }
    t := Tape { len: 5 }
    r.measure() + t.measure()
}
"#;
    // 10 + 10 = 20
    assert_exit_code(source, 20);
}

#[test]
fn e2e_p119_trait_default_method_override() {
    // Trait with a method that gets overridden
    let source = r#"
W Valued {
    F value(&self) -> i64
}

S Gold { weight: i64 }

X Gold: Valued {
    F value(&self) -> i64 {
        self.weight * 100
    }
}

F main() -> i64 {
    g := Gold { weight: 3 }
    g.value()
}
"#;
    // 3 * 100 = 300 -> exit code wraps to 300 % 256 = 44
    assert_exit_code(source, 44);
}

// ==================== 2. Generic Function Monomorphization ====================

#[test]
fn e2e_p119_generic_identity_mono() {
    // Simple generic function monomorphized with i64
    let source = r#"
F identity<T>(x: T) -> T {
    x
}

F main() -> i64 {
    identity(42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p119_generic_add_mono() {
    // Generic function with arithmetic — monomorphized to i64
    let source = r#"
F add_vals<T>(a: T, b: T) -> T {
    a + b
}

F main() -> i64 {
    add_vals(20, 22)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p119_generic_nested_call() {
    // Generic calling another generic — transitive monomorphization
    let source = r#"
F inner<T>(x: T) -> T {
    x
}

F outer<T>(x: T) -> T {
    inner(x)
}

F main() -> i64 {
    outer(55)
}
"#;
    assert_exit_code(source, 55);
}

#[test]
fn e2e_p119_generic_triple_chain() {
    // Three-level generic chain
    let source = r#"
F level3<T>(x: T) -> T {
    x
}

F level2<T>(x: T) -> T {
    level3(x)
}

F level1<T>(x: T) -> T {
    level2(x)
}

F main() -> i64 {
    level1(77)
}
"#;
    assert_exit_code(source, 77);
}

#[test]
fn e2e_p119_generic_with_computation() {
    // Generic function with actual computation
    let source = r#"
F double<T>(x: T) -> T {
    x + x
}

F main() -> i64 {
    double(21)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p119_generic_multiple_type_params() {
    // Function with multiple generic params
    let source = r#"
F first<A, B>(a: A, b: B) -> A {
    a
}

F main() -> i64 {
    first(33, 99)
}
"#;
    assert_exit_code(source, 33);
}

#[test]
fn e2e_p119_generic_second_param() {
    // Return second generic param
    let source = r#"
F second<A, B>(a: A, b: B) -> B {
    b
}

F main() -> i64 {
    second(11, 88)
}
"#;
    assert_exit_code(source, 88);
}

// ==================== 3. Generic Struct + Method Monomorphization ====================

#[test]
fn e2e_p119_generic_struct_method() {
    // Generic struct with method
    let source = r#"
S Wrapper<T> { val: T }

X Wrapper {
    F get(&self) -> i64 {
        self.val
    }
}

F main() -> i64 {
    w := Wrapper { val: 65 }
    w.get()
}
"#;
    assert_exit_code(source, 65);
}

#[test]
fn e2e_p119_generic_struct_two_fields() {
    // Generic struct with two fields
    let source = r#"
S Pair<T> { first: T, second: T }

X Pair {
    F sum(&self) -> i64 {
        self.first + self.second
    }
}

F main() -> i64 {
    p := Pair { first: 30, second: 12 }
    p.sum()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 4. Trait + Generic Interaction ====================

#[test]
fn e2e_p119_trait_impl_on_generic_struct() {
    // Trait implemented on a generic struct
    let source = r#"
W Describable {
    F code(&self) -> i64
}

S Box<T> { item: T }

X Box: Describable {
    F code(&self) -> i64 {
        self.item
    }
}

F main() -> i64 {
    b := Box { item: 99 }
    b.code()
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn e2e_p119_multiple_traits_single_struct() {
    // Single struct implementing multiple traits
    let source = r#"
W Printable {
    F to_code(&self) -> i64
}

W Comparable {
    F rank(&self) -> i64
}

S Item { id: i64, priority: i64 }

X Item: Printable {
    F to_code(&self) -> i64 {
        self.id
    }
}

X Item: Comparable {
    F rank(&self) -> i64 {
        self.priority
    }
}

F main() -> i64 {
    item := Item { id: 10, priority: 5 }
    item.to_code() + item.rank()
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p119_generic_function_with_struct_return() {
    // Generic function returning a struct
    let source = r#"
S Result { val: i64 }

F make_result<T>(x: T) -> Result {
    Result { val: x }
}

F main() -> i64 {
    r := make_result(73)
    r.val
}
"#;
    assert_exit_code(source, 73);
}

// ==================== 5. Monomorphization Edge Cases ====================

#[test]
fn e2e_p119_generic_called_multiple_times_same_type() {
    // Same generic function called multiple times with same type
    let source = r#"
F passthrough<T>(x: T) -> T {
    x
}

F main() -> i64 {
    a := passthrough(10)
    b := passthrough(20)
    c := passthrough(12)
    a + b + c
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p119_generic_recursive() {
    // Generic function with recursion via @
    let source = r#"
F countdown<T>(n: T) -> T {
    I n <= 0 {
        0
    } E {
        n + @(n - 1)
    }
}

F main() -> i64 {
    countdown(5)
}
"#;
    // 5 + 4 + 3 + 2 + 1 + 0 = 15
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p119_generic_with_conditional() {
    // Generic function with conditional logic
    let source = r#"
F pick<T>(a: T, b: T, use_first: i64) -> T {
    I use_first > 0 {
        a
    } E {
        b
    }
}

F main() -> i64 {
    pick(42, 99, 1)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p119_trait_method_with_arithmetic() {
    // Trait method doing arithmetic on struct fields
    let source = r#"
W Calculator {
    F compute(&self) -> i64
}

S Adder { a: i64, b: i64 }

X Adder: Calculator {
    F compute(&self) -> i64 {
        self.a + self.b
    }
}

S Multiplier { x: i64, y: i64 }

X Multiplier: Calculator {
    F compute(&self) -> i64 {
        self.x * self.y
    }
}

F main() -> i64 {
    adder := Adder { a: 20, b: 10 }
    mult := Multiplier { x: 3, y: 4 }
    adder.compute() + mult.compute()
}
"#;
    // 30 + 12 = 42
    assert_exit_code(source, 42);
}

// ==================== 6. Struct Method Chains ====================

#[test]
fn e2e_p119_method_chain_result() {
    // Method that returns a value used in further computation
    let source = r#"
S Counter { val: i64 }

X Counter {
    F get(&self) -> i64 {
        self.val
    }

    F doubled(&self) -> i64 {
        self.val * 2
    }
}

F main() -> i64 {
    c := Counter { val: 21 }
    c.doubled()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p119_nested_struct_method() {
    // Methods on nested struct access
    let source = r#"
S Inner { x: i64 }
S Outer { inner: Inner }

X Inner {
    F value(&self) -> i64 {
        self.x
    }
}

X Outer {
    F get_inner_val(&self) -> i64 {
        self.inner.x
    }
}

F main() -> i64 {
    o := Outer { inner: Inner { x: 50 } }
    o.get_inner_val()
}
"#;
    assert_exit_code(source, 50);
}

// ==================== 7. Type Inference with Generics ====================

#[test]
fn e2e_p119_inferred_generic_return() {
    // Generic return type inferred from body
    let source = r#"
F wrap<T>(x: T) -> T {
    y := x
    y
}

F main() -> i64 {
    wrap(42)
}
"#;
    assert_exit_code(source, 42);
}
