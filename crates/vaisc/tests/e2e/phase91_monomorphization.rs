//! Phase 91: Monomorphization Enhancement Tests
//!
//! Tests for:
//! 1. Generic struct methods via impl blocks
//! 2. Generic identity and composition patterns
//! 3. Generic method instantiation with concrete types
//! 4. Multi-field generic struct methods

use super::helpers::*;

// ==================== 1. Generic struct with generic impl method ====================

#[test]
fn e2e_phase91_generic_struct_impl_method() {
    // Generic struct with impl block where T falls back to i64
    let source = r#"
struct Container<T> {
    value: T
}

impl Container<T> {
    fn get(self) -> type {
        self.value
    }
}

fn main() -> i64 {
    c := Container { value: 42 }
    c.get()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 2. Generic struct method with arithmetic ====================

#[test]
fn e2e_phase91_generic_struct_method_arithmetic() {
    // Generic struct method that performs arithmetic on the field
    let source = r#"
struct Wrapper<T> {
    val: T
}

impl Wrapper<T> {
    fn doubled(self) -> type {
        self.val + self.val
    }
}

fn main() -> i64 {
    w := Wrapper { val: 21 }
    w.doubled()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 3. Generic function identity pattern ====================

#[test]
fn e2e_phase91_generic_identity_i64() {
    let source = r#"
fn identity<T>(x: T) -> type { x }

fn main() -> i64 {
    identity(77)
}
"#;
    assert_exit_code(source, 77);
}

// ==================== 4. Generic function with two params ====================

#[test]
fn e2e_phase91_generic_add() {
    let source = r#"
fn add<T>(a: T, b: T) -> type { a + b }

fn main() -> i64 {
    add(20, 22)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 5. Generic struct multiple methods ====================

#[test]
fn e2e_phase91_generic_struct_multiple_methods() {
    let source = r#"
struct Pair<T> {
    first: T,
    second: T
}

impl Pair<T> {
    fn sum(self) -> type {
        self.first + self.second
    }

    fn get_first(self) -> type {
        self.first
    }
}

fn main() -> i64 {
    p := Pair { first: 30, second: 12 }
    p.sum()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 6. Generic struct method returns modified value ====================

#[test]
fn e2e_phase91_generic_struct_method_computation() {
    let source = r#"
struct Box<T> {
    data: T
}

impl Box<T> {
    fn value(self) -> type {
        self.data
    }

    fn plus(self, n: T) -> type {
        self.data + n
    }
}

fn main() -> i64 {
    b := Box { data: 40 }
    b.plus(2)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 7. Generic function called from non-generic ====================

#[test]
fn e2e_phase91_generic_fn_called_from_non_generic() {
    let source = r#"
fn double<T>(x: T) -> type { x + x }

fn compute() -> i64 {
    double(21)
}

fn main() -> i64 {
    compute()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 8. Generic struct with concrete impl ====================

#[test]
fn e2e_phase91_generic_struct_concrete_impl() {
    // Impl for a concrete type param (Wrapper<i64>)
    let source = r#"
struct Wrapper<T> {
    value: T
}

impl Wrapper<i64> {
    fn get(self) -> i64 {
        self.value
    }
}

fn main() -> i64 {
    w := Wrapper { value: 42 }
    w.get() - 42
}
"#;
    assert_exit_code(source, 0);
}

// ==================== 9. Generic struct with generic function composition ====================

#[test]
fn e2e_phase91_generic_struct_with_fn() {
    let source = r#"
struct Cell<T> {
    inner: T
}

fn extract<T>(c: Cell<T>) -> type {
    c.inner
}

fn main() -> i64 {
    cell := Cell { inner: 99 }
    extract(cell)
}
"#;
    assert_exit_code(source, 99);
}

// ==================== 10. Generic function chain with struct ====================

#[test]
fn e2e_phase91_generic_fn_chain() {
    let source = r#"
fn step1<T>(x: T) -> type { x + 1 }
fn step2<T>(x: T) -> type { step1(x) + 1 }

fn main() -> i64 {
    step2(40)
}
"#;
    // step2(40) = step1(40) + 1 = (40+1) + 1 = 42
    assert_exit_code(source, 42);
}

// ==================== 11. Generic struct method chaining (via intermediate vars) ====================

#[test]
fn e2e_phase91_generic_method_chain_vars() {
    let source = r#"
struct Num<T> {
    v: T
}

impl Num<T> {
    fn val(self) -> type {
        self.v
    }
}

fn main() -> i64 {
    a := Num { v: 20 }
    b := Num { v: 22 }
    a.val() + b.val()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 12. Trait impl on generic struct ====================

#[test]
fn e2e_phase91_trait_on_generic_struct() {
    let source = r#"
trait Valuable {
    fn worth(self) -> i64
}

struct Item<T> {
    price: T
}

impl Item<i64>: Valuable {
    fn worth(self) -> i64 {
        self.price
    }
}

fn main() -> i64 {
    item := Item { price: 42 }
    item.worth()
}
"#;
    assert_exit_code(source, 42);
}
