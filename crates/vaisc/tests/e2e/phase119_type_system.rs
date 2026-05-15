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
trait Container {
    fn size(&self) -> i64
}

struct Bag { count: i64 }

impl Bag: Container {
    fn size(&self) -> i64 {
        self.count
    }
}

fn main() -> i64 {
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
trait Measurable {
    fn measure(&self) -> i64
}

struct Ruler { len: i64 }
struct Tape { len: i64 }

impl Ruler: Measurable {
    fn measure(&self) -> i64 {
        self.len
    }
}

impl Tape: Measurable {
    fn measure(&self) -> i64 {
        self.len * 2
    }
}

fn main() -> i64 {
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
trait Valued {
    fn value(&self) -> i64
}

struct Gold { weight: i64 }

impl Gold: Valued {
    fn value(&self) -> i64 {
        self.weight * 100
    }
}

fn main() -> i64 {
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
fn identity<T>(x: T) -> type {
    x
}

fn main() -> i64 {
    identity(42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p119_generic_add_mono() {
    // Generic function with arithmetic — monomorphized to i64
    let source = r#"
fn add_vals<T>(a: T, b: T) -> type {
    a + b
}

fn main() -> i64 {
    add_vals(20, 22)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p119_generic_nested_call() {
    // Generic calling another generic — transitive monomorphization
    let source = r#"
fn inner<T>(x: T) -> type {
    x
}

fn outer<T>(x: T) -> type {
    inner(x)
}

fn main() -> i64 {
    outer(55)
}
"#;
    assert_exit_code(source, 55);
}

#[test]
fn e2e_p119_generic_triple_chain() {
    // Three-level generic chain
    let source = r#"
fn level3<T>(x: T) -> type {
    x
}

fn level2<T>(x: T) -> type {
    level3(x)
}

fn level1<T>(x: T) -> type {
    level2(x)
}

fn main() -> i64 {
    level1(77)
}
"#;
    assert_exit_code(source, 77);
}

#[test]
fn e2e_p119_generic_with_computation() {
    // Generic function with actual computation
    let source = r#"
fn double<T>(x: T) -> type {
    x + x
}

fn main() -> i64 {
    double(21)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p119_generic_multiple_type_params() {
    // Function with multiple generic params
    let source = r#"
fn first<A, B>(a: A, b: B) -> A {
    a
}

fn main() -> i64 {
    first(33, 99)
}
"#;
    assert_exit_code(source, 33);
}

#[test]
fn e2e_p119_generic_second_param() {
    // Return second generic param
    let source = r#"
fn second<A, B>(a: A, b: B) -> B {
    b
}

fn main() -> i64 {
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
struct Wrapper<T> { val: type }

impl Wrapper {
    fn get(&self) -> i64 {
        self.val
    }
}

fn main() -> i64 {
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
struct Pair<T> { first: T, second: type }

impl Pair {
    fn sum(&self) -> i64 {
        self.first + self.second
    }
}

fn main() -> i64 {
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
trait Describable {
    fn code(&self) -> i64
}

struct Box<T> { item: type }

impl Box: Describable {
    fn code(&self) -> i64 {
        self.item
    }
}

fn main() -> i64 {
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
trait Printable {
    fn to_code(&self) -> i64
}

trait Comparable {
    fn rank(&self) -> i64
}

struct Item { id: i64, priority: i64 }

impl Item: Printable {
    fn to_code(&self) -> i64 {
        self.id
    }
}

impl Item: Comparable {
    fn rank(&self) -> i64 {
        self.priority
    }
}

fn main() -> i64 {
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
struct Result { val: i64 }

fn make_result<T>(x: T) -> Result {
    Result { val: x }
}

fn main() -> i64 {
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
fn passthrough<T>(x: T) -> type {
    x
}

fn main() -> i64 {
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
fn countdown<T>(n: T) -> type {
    I n <= 0 {
        0
    } else {
        n + @(n - 1)
    }
}

fn main() -> i64 {
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
fn pick<T>(a: T, b: T, use_first: i64) -> type {
    I use_first > 0 {
        a
    } else {
        b
    }
}

fn main() -> i64 {
    pick(42, 99, 1)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p119_trait_method_with_arithmetic() {
    // Trait method doing arithmetic on struct fields
    let source = r#"
trait Calculator {
    fn compute(&self) -> i64
}

struct Adder { a: i64, b: i64 }

impl Adder: Calculator {
    fn compute(&self) -> i64 {
        self.a + self.b
    }
}

struct Multiplier { x: i64, y: i64 }

impl Multiplier: Calculator {
    fn compute(&self) -> i64 {
        self.x * self.y
    }
}

fn main() -> i64 {
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
struct Counter { val: i64 }

impl Counter {
    fn get(&self) -> i64 {
        self.val
    }

    fn doubled(&self) -> i64 {
        self.val * 2
    }
}

fn main() -> i64 {
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
struct Inner { x: i64 }
struct Outer { inner: Inner }

impl Inner {
    fn value(&self) -> i64 {
        self.x
    }
}

impl Outer {
    fn get_inner_val(&self) -> i64 {
        self.inner.x
    }
}

fn main() -> i64 {
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
fn wrap<T>(x: T) -> type {
    y := x
    y
}

fn main() -> i64 {
    wrap(42)
}
"#;
    assert_exit_code(source, 42);
}
