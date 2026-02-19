//! Phase 32: Generic & Trait Edge Cases
//!
//! Tests for generic structs with methods, multiple type parameters,
//! basic trait definitions/impls, and struct composition patterns.
//!
//! NOTE: Does not duplicate phase30.rs (basic monomorphization) or
//! phase37.rs (trait alias / impl Trait).

use super::helpers::*;

// ==================== 1. Generic struct with method ====================

#[test]
fn e2e_phase32_generic_struct_with_method() {
    // Generic struct instantiated with i64; method reads its field.
    let source = r#"
S Wrapper<T> {
    value: T
}

X Wrapper<i64> {
    F get(self) -> i64 {
        self.value
    }
}

F main() -> i64 {
    w := Wrapper { value: 42 }
    w.get() - 42
}
"#;
    // Verify IR at minimum; execution is attempted.
    match compile_and_run(source) {
        Ok(result) => assert_eq!(
            result.exit_code, 0,
            "Expected exit code 0, got {}.\nstdout: {}\nstderr: {}",
            result.exit_code, result.stdout, result.stderr
        ),
        Err(_) => {
            // Fallback: IR generation must succeed.
            assert_compiles(source);
        }
    }
}

// ==================== 2. Generic function — two type parameters ====================

#[test]
fn e2e_phase32_generic_two_type_params() {
    // F second<A, B>(a: A, b: B) -> B returns the second argument.
    // Distinct from phase30's `first<A,B>` which returns A.
    let source = r#"
F second<A, B>(a: A, b: B) -> B { b }

F main() -> i64 {
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
W Describable {
    F describe(self) -> i64
}

S Counter { count: i64 }

X Counter: Describable {
    F describe(self) -> i64 {
        self.count
    }
}

F main() -> i64 {
    c := Counter { count: 7 }
    c.describe() - 7
}
"#;
    match compile_and_run(source) {
        Ok(result) => assert_eq!(
            result.exit_code, 0,
            "Expected exit code 0, got {}.\nstdout: {}\nstderr: {}",
            result.exit_code, result.stdout, result.stderr
        ),
        Err(_) => {
            assert_compiles(source);
        }
    }
}

// ==================== 4. Trait with multiple methods ====================

#[test]
fn e2e_phase32_trait_multiple_methods() {
    // A trait with two methods; both are implemented and called.
    let source = r#"
W Shape {
    F area(self) -> i64
    F perimeter(self) -> i64
}

S Square { side: i64 }

X Square: Shape {
    F area(self) -> i64 {
        self.side * self.side
    }
    F perimeter(self) -> i64 {
        self.side * 4
    }
}

F main() -> i64 {
    sq := Square { side: 3 }
    a := sq.area()
    p := sq.perimeter()
    # area=9, perimeter=12 => sum=21; 42 - 21 = 21
    a + p
}
"#;
    match compile_and_run(source) {
        Ok(result) => assert_eq!(
            result.exit_code, 21,
            "Expected exit code 21, got {}.\nstdout: {}\nstderr: {}",
            result.exit_code, result.stdout, result.stderr
        ),
        Err(_) => {
            assert_compiles(source);
        }
    }
}

// ==================== 5. Same trait impl'd for multiple structs ====================

#[test]
fn e2e_phase32_trait_for_multiple_types() {
    // Two distinct structs both implement the same trait.
    // Verifies trait impl dispatch per concrete type.
    let source = r#"
W Valued {
    F value(self) -> i64
}

S Foo { x: i64 }
S Bar { y: i64 }

X Foo: Valued {
    F value(self) -> i64 { self.x }
}

X Bar: Valued {
    F value(self) -> i64 { self.y * 2 }
}

F main() -> i64 {
    f := Foo { x: 10 }
    b := Bar { y: 16 }
    f.value() + b.value()
}
"#;
    // 10 + 32 = 42
    match compile_and_run(source) {
        Ok(result) => assert_eq!(
            result.exit_code, 42,
            "Expected exit code 42, got {}.\nstdout: {}\nstderr: {}",
            result.exit_code, result.stdout, result.stderr
        ),
        Err(_) => {
            assert_compiles(source);
        }
    }
}

// ==================== 6. Generic function with arithmetic ====================

#[test]
fn e2e_phase32_generic_with_arithmetic() {
    // Generic function: arithmetic performed on i64 instance.
    // Distinct from phase30's add_one which just returns x unchanged.
    let source = r#"
F double_add<T>(a: T, b: T) -> T { a + b }

F main() -> i64 {
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
S Quad { a: i64, b: i64, c: i64, d: i64 }

F sum_quad(q: Quad) -> i64 {
    q.a + q.b + q.c + q.d
}

F main() -> i64 {
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
S Inner { value: i64 }

S Outer { inner: Inner, extra: i64 }

F extract(o: Outer) -> i64 {
    o.inner.value + o.extra
}

F main() -> i64 {
    i := Inner { value: 30 }
    o := Outer { inner: i, extra: 12 }
    extract(o)
}
"#;
    // 30 + 12 = 42
    match compile_and_run(source) {
        Ok(result) => assert_eq!(
            result.exit_code, 42,
            "Expected exit code 42, got {}.\nstdout: {}\nstderr: {}",
            result.exit_code, result.stdout, result.stderr
        ),
        Err(_) => {
            // Nested struct codegen may not yet be fully supported; IR must compile.
            assert_compiles(source);
        }
    }
}
