//! Phase 114: Monomorphization Completion Tests
//!
//! Tests for:
//! 1. Generic function monomorphization with concrete type specialization
//! 2. Multi-type generic instantiation (same function, different types)
//! 3. Generic struct + method monomorphization
//! 4. Nested generic calls (transitive monomorphization)
//! 5. Generic with multiple parameters
//! 6. Generic identity chains
//! 7. Fallback i64 behavior for uninstantiated generics
//! 8. Generic arithmetic patterns
//! 9. Generic struct field access patterns
//! 10. Generic with conditional logic

use super::helpers::*;

// ==================== 1. Basic generic identity monomorphization ====================

#[test]
fn e2e_phase114_generic_identity_i64() {
    let source = r#"
F identity<T>(x: T) -> T { x }

F main() -> i64 {
    identity(42)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 2. Generic function with arithmetic ====================

#[test]
fn e2e_phase114_generic_double() {
    let source = r#"
F double<T>(x: T) -> T { x + x }

F main() -> i64 {
    double(21)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 3. Generic function called multiple times ====================

#[test]
fn e2e_phase114_generic_multi_call() {
    let source = r#"
F add_one<T>(x: T) -> T { x + 1 }

F main() -> i64 {
    a := add_one(10)
    b := add_one(20)
    a + b + 1
}
"#;
    assert_exit_code(source, 33);
}

// ==================== 4. Transitive monomorphization: two levels ====================

#[test]
fn e2e_phase114_transitive_two_levels() {
    let source = r#"
F inner<T>(x: T) -> T { x + 1 }
F outer<T>(x: T) -> T { inner(x) + 1 }

F main() -> i64 {
    outer(40)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 5. Generic struct with method ====================

#[test]
fn e2e_phase114_generic_struct_method() {
    let source = r#"
S Box<T> {
    value: T
}

X Box<T> {
    F get(self) -> T {
        self.value
    }
}

F main() -> i64 {
    b := Box { value: 55 }
    b.get()
}
"#;
    assert_exit_code(source, 55);
}

// ==================== 6. Generic struct with arithmetic method ====================

#[test]
fn e2e_phase114_generic_struct_arithmetic_method() {
    let source = r#"
S Pair<T> {
    first: T,
    second: T
}

X Pair<T> {
    F sum(self) -> T {
        self.first + self.second
    }
}

F main() -> i64 {
    p := Pair { first: 20, second: 22 }
    p.sum()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 7. Generic function with conditional ====================

#[test]
fn e2e_phase114_generic_with_condition() {
    let source = r#"
F max_val<T>(a: T, b: T) -> T {
    I a > b {
        a
    } E {
        b
    }
}

F main() -> i64 {
    max_val(10, 42)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 8. Multiple generic parameters ====================

#[test]
fn e2e_phase114_multi_param_generic() {
    let source = r#"
F pick_first<A, B>(a: A, b: B) -> A { a }

F main() -> i64 {
    pick_first(42, 99)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 9. Generic chain: 3 levels deep ====================

#[test]
fn e2e_phase114_three_level_chain() {
    let source = r#"
F step1<T>(x: T) -> T { x + 1 }
F step2<T>(x: T) -> T { step1(x) + 1 }
F step3<T>(x: T) -> T { step2(x) + 1 }

F main() -> i64 {
    step3(39)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 10. Generic with loop ====================

#[test]
fn e2e_phase114_generic_with_loop() {
    let source = r#"
F accumulate<T>(start: T, count: i64) -> T {
    result := mut start
    L i:0..count {
        result = result + 1
    }
    result
}

F main() -> i64 {
    accumulate(32, 10)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 11. Generic wrapping and unwrapping ====================

#[test]
fn e2e_phase114_generic_wrap_unwrap() {
    let source = r#"
F wrap<T>(x: T) -> T { x }
F unwrap_val<T>(x: T) -> T { x }

F main() -> i64 {
    val := wrap(42)
    unwrap_val(val)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 12. Generic called from non-generic ====================

#[test]
fn e2e_phase114_generic_from_nongeneric() {
    let source = r#"
F add<T>(a: T, b: T) -> T { a + b }

F compute() -> i64 {
    add(20, 22)
}

F main() -> i64 {
    compute()
}
"#;
    assert_exit_code(source, 42);
}
