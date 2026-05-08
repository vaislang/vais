//! Phase 32: Generic & Trait Edge Cases
//!
//! Tests for generic structs with methods, multiple type parameters,
//! basic trait definitions/impls, and struct composition patterns.
//!
//! NOTE: Does not duplicate phase30.rs (basic monomorphization) or
//! phase37.rs (trait alias).

use super::helpers::*;

// ==================== 1. Generic struct with method ====================

#[test]
fn e2e_phase32_generic_struct_with_method() {
    // Generic struct instantiated with i64; method reads its field.
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
    // w.get() = 42, 42 - 42 = 0, exit code 0.
    assert_exit_code(source, 0);
}

// ==================== 2. Generic function — two type parameters ====================

#[test]
fn e2e_phase32_generic_two_type_params() {
    // F second<A, B>(a: A, b: B) -> B returns the second argument.
    // Distinct from phase30's `first<A,B>` which returns A.
    let source = r#"
fn second<A, B>(a: A, b: B) -> B { b }

fn main() -> i64 {
    second(0, 99)
}
"#;
    assert_exit_code(source, 99);
}

// ==================== 3. Basic trait definition + impl + method call ====================

#[test]
fn e2e_phase32_trait_basic_impl() {
    // Define a trait, implement it on a concrete struct, call the method.
    let source = r#"
trait Describable {
    fn describe(self) -> i64
}

struct Counter { count: i64 }

impl Counter: Describable {
    fn describe(self) -> i64 {
        self.count
    }
}

fn main() -> i64 {
    c := Counter { count: 7 }
    c.describe() - 7
}
"#;
    // c.describe() = 7, 7 - 7 = 0, exit code 0.
    assert_exit_code(source, 0);
}

// ==================== 4. Trait with multiple methods ====================

#[test]
fn e2e_phase32_trait_multiple_methods() {
    // A trait with two methods; both are implemented and called.
    let source = r#"
trait Shape {
    fn area(self) -> i64
    fn perimeter(self) -> i64
}

struct Square { side: i64 }

impl Square: Shape {
    fn area(self) -> i64 {
        self.side * self.side
    }
    fn perimeter(self) -> i64 {
        self.side * 4
    }
}

fn main() -> i64 {
    sq := Square { side: 3 }
    a := sq.area()
    p := sq.perimeter()
    # area=9, perimeter=12 => sum=21; 42 - 21 = 21
    a + p
}
"#;
    // area=9, perimeter=12, 9+12=21, exit code 21.
    assert_exit_code(source, 21);
}

// ==================== 5. Same trait impl'd for multiple structs ====================

#[test]
fn e2e_phase32_trait_for_multiple_types() {
    // Two distinct structs both implement the same trait.
    // Verifies trait impl dispatch per concrete type.
    let source = r#"
trait Valued {
    fn value(self) -> i64
}

struct Foo { x: i64 }
struct Bar { y: i64 }

impl Foo: Valued {
    fn value(self) -> i64 { self.x }
}

impl Bar: Valued {
    fn value(self) -> i64 { self.y * 2 }
}

fn main() -> i64 {
    f := Foo { x: 10 }
    b := Bar { y: 16 }
    f.value() + b.value()
}
"#;
    // f.value()=10, b.value()=16*2=32, 10+32=42, exit code 42.
    assert_exit_code(source, 42);
}

// ==================== 6. Generic function with arithmetic ====================

#[test]
fn e2e_phase32_generic_with_arithmetic() {
    // Generic function: arithmetic performed on i64 instance.
    // Distinct from phase30's add_one which just returns x unchanged.
    let source = r#"
fn double_add<T>(a: T, b: T) -> type { a + b }

fn main() -> i64 {
    double_add(19, 23)
}
"#;
    // 19 + 23 = 42
    assert_exit_code(source, 42);
}

// ==================== 7. Struct with multiple fields ====================

#[test]
fn e2e_phase32_struct_multiple_fields() {
    // Struct with four i64 fields; verify each field is accessible and correct.
    let source = r#"
struct Quad { a: i64, b: i64, c: i64, d: i64 }

fn sum_quad(q: Quad) -> i64 {
    q.a + q.b + q.c + q.d
}

fn main() -> i64 {
    q := Quad { a: 10, b: 11, c: 12, d: 9 }
    sum_quad(q)
}
"#;
    // 10 + 11 + 12 + 9 = 42
    assert_exit_code(source, 42);
}

// ==================== 8. Nested struct (struct field of struct type) ====================

#[test]
fn e2e_phase32_nested_struct() {
    // Outer struct holds an Inner struct as a field.
    // Accesses nested field through two-level field lookup.
    let source = r#"
struct Inner { value: i64 }

struct Outer { inner: Inner, extra: i64 }

fn extract(o: Outer) -> i64 {
    o.inner.value + o.extra
}

fn main() -> i64 {
    i := Inner { value: 30 }
    o := Outer { inner: i, extra: 12 }
    extract(o)
}
"#;
    // inner.value=30, extra=12, 30+12=42, exit code 42.
    assert_exit_code(source, 42);
}
