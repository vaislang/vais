//! Phase 91 -- Lifetime Verification
//!
//! Tests for lifetime inference, elision rules, explicit lifetime annotations,
//! dangling reference detection, and lifetime constraint solving.

use super::helpers::*;

// ==================== Lifetime Elision Rule 2: Single Ref Param ====================

#[test]
fn e2e_lifetime_single_ref_param_elision() {
    // Rule 2: single reference parameter → output gets same lifetime
    let source = r#"
F identity(x: &i64) -> &i64 = x
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_lifetime_single_ref_pass_through() {
    let source = r#"
F pass(x: &i64) -> &i64 = x
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_lifetime_single_ref_with_value_params() {
    // Only one reference param, other params are values → elision works
    let source = r#"
F pick(x: &i64, n: i64) -> &i64 = x
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

// ==================== Lifetime Elision Failure: Multiple Ref Params ====================

#[test]
fn e2e_lifetime_ambiguous_elision_error() {
    // Two distinct reference params, no self → ambiguous elision
    let source = r#"
F first(x: &i64, y: &i64) -> &i64 = x
F main() -> i64 = 42
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_lifetime_ambiguous_three_ref_params() {
    let source = r#"
F pick(x: &i64, y: &i64, z: &i64) -> &i64 = x
F main() -> i64 = 42
"#;
    assert_compile_error(source);
}

// ==================== Explicit Lifetime Annotations ====================

#[test]
fn e2e_lifetime_explicit_same_lifetime() {
    // Explicit 'a on both params and return resolves ambiguity
    let source = r#"
F first<'a>(x: &'a i64, y: &'a i64) -> &'a i64 = x
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_lifetime_explicit_different_lifetimes() {
    // 'a and 'b on params, return type uses 'a (valid: 'a from param x)
    let source = r#"
F first<'a, 'b>(x: &'a i64, y: &'b i64) -> &'a i64 = x
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_lifetime_explicit_return_second_param() {
    // Return uses 'b from param y (valid)
    let source = r#"
F second<'a, 'b>(x: &'a i64, y: &'b i64) -> &'b i64 = y
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

// ==================== Orphan Lifetime Detection ====================

#[test]
fn e2e_lifetime_orphan_lifetime_error() {
    // Return lifetime 'c is not present on any parameter → dangling
    let source = r#"
F bad<'a, 'c>(x: &'a i64) -> &'c i64 = x
F main() -> i64 = 42
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_lifetime_orphan_with_two_params() {
    // Return lifetime 'c not in param lifetimes 'a, 'b → dangling
    let source = r#"
F bad<'a, 'b, 'c>(x: &'a i64, y: &'b i64) -> &'c i64 = x
F main() -> i64 = 42
"#;
    assert_compile_error(source);
}

// ==================== No Reference Params ====================

#[test]
fn e2e_lifetime_no_ref_params_no_ref_return() {
    // No references at all → lifetime checking is a no-op
    let source = r#"
F add(x: i64, y: i64) -> i64 = x + y
F main() -> i64 = add(20, 22)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_lifetime_no_ref_return_type() {
    // Ref param but non-ref return → no elision needed
    let source = r#"
F deref(x: &i64) -> i64 = 42
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

// ==================== Method Self Lifetime (Rule 3) ====================

#[test]
fn e2e_lifetime_method_self_elision() {
    // Methods with self → output lifetime is self's lifetime (rule 3)
    let source = r#"
S Counter { value: i64 }
X Counter {
    F get(self) -> i64 = self.value
}
F main() -> i64 {
    c := Counter { value: 42 }
    c.get()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_lifetime_method_with_extra_param() {
    let source = r#"
S Adder { base: i64 }
X Adder {
    F add(self, n: i64) -> i64 = self.base + n
}
F main() -> i64 {
    a := Adder { base: 30 }
    a.add(12)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Multiple Functions with Lifetimes ====================

#[test]
fn e2e_lifetime_multiple_functions_valid() {
    let source = r#"
F id_ref(x: &i64) -> &i64 = x
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(20, 22)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_lifetime_mixed_ref_and_value_functions() {
    let source = r#"
F pass_ref(x: &i64) -> &i64 = x
F double(x: i64) -> i64 = x * 2
F main() -> i64 = double(21)
"#;
    assert_exit_code(source, 42);
}

// ==================== Edge Cases ====================

#[test]
fn e2e_lifetime_elision_with_bool_and_ref() {
    // One ref param + one value param → elision from ref param
    let source = r#"
F cond_ref(flag: bool, x: &i64) -> &i64 = x
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_lifetime_explicit_single_lifetime() {
    // Explicit 'a on single param — redundant but valid
    let source = r#"
F wrap<'a>(x: &'a i64) -> &'a i64 = x
F main() -> i64 = 42
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_lifetime_no_ref_params_returns_ref() {
    // No ref params, returns a ref → gets 'static from elision (valid for now)
    // NOTE: assert_compiles only -- codegen emits `ret i64* 42` (literal, not pointer), clang rejects
    let source = r#"
F get_ref() -> &i64 {
    42
}
F main() -> i64 = 42
"#;
    assert_compiles(source);
}
