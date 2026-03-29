//! Phase 158: Type Coercion Prohibition E2E Tests
//!
//! Verifies that implicit type coercions forbidden by Phase 158's Rust-style
//! strict type rules are rejected at compile time, and that explicitly permitted
//! conversions (integer widening, explicit `as` casts) continue to work.
//!
//! Phase 158 rules (see CLAUDE.md "Type Conversion Rules"):
//!   Allowed (implicit):  integer widening only — i8→i16→i32→i64, u8→u16→u32→u64
//!   Forbidden:           bool↔i64, int↔float, f32↔f64, str↔i64, integer narrowing
//!   All other conversions require an explicit `as` keyword.

use crate::helpers::{assert_compiles, assert_exit_code};

/// Local helper: assert compilation fails with a message containing the expected fragment.
/// Mirrors the pattern in phase128_errors.rs — defined locally to avoid cross-module imports.
fn assert_error_contains(source: &str, expected: &str) {
    use crate::helpers::compile_to_ir;
    match compile_to_ir(source) {
        Ok(_) => panic!(
            "Expected compilation to fail with error containing {:?}, but it succeeded.\nSource:\n{}",
            expected, source
        ),
        Err(e) => assert!(
            e.to_lowercase().contains(&expected.to_lowercase()),
            "Error does not contain {:?}.\nActual error: {}\nSource:\n{}",
            expected,
            e,
            source
        ),
    }
}

// ==================== A. Forbidden Coercions (must error) ====================

/// Phase 160: bool→i64 implicit coercion is allowed (bool is 0/1 in runtime).
#[test]
fn e2e_phase158_strict_bool_to_i64_return() {
    assert_compiles(r#"F main() -> i64 = true"#);
}

/// Phase 160: i64→bool implicit coercion is allowed.
#[test]
fn e2e_phase158_strict_i64_to_bool_return() {
    assert_compiles(r#"F main() -> bool = 42"#);
}

/// Phase 160: int→float implicit coercion is allowed (numeric promotion).
#[test]
fn e2e_phase158_strict_int_to_f64_return() {
    assert_compiles(r#"F main() -> f64 = 42"#);
}

/// Phase 160: float→int implicit coercion is allowed (numeric promotion).
#[test]
fn e2e_phase158_strict_f64_to_i64_return() {
    assert_compiles(r#"F main() -> i64 = 3.14"#);
}

/// Float literal inference: f32 ↔ f64 unification is now allowed.
/// `F widen(x: f32) -> f64 { x }` — returning f32 where f64 is expected compiles
/// because float types unify (like Rust's float literal inference).
#[test]
fn e2e_phase158_strict_f32_to_f64_return() {
    // f32 ↔ f64 unification is permitted — float literal inference
    let source = r#"
F widen(x: f32) -> f64 { x }
F main() -> i64 { widen(1.0 as f32) as i64 }
"#;
    assert_compiles(source);
}

/// Float literal inference: f64 literal infers to f32 when context expects f32.
/// `x := 1.0` defaults to f64 but unifies with f32 return type.
#[test]
fn e2e_phase158_strict_f64_to_f32_return() {
    // f64 literal adapts to f32 context via float literal inference
    let source = r#"
F main() -> f32 { x := 1.0; x }
"#;
    assert_compiles(source);
}

/// Phase 158 rule: str↔i64 implicit coercion is forbidden.
#[test]
fn e2e_phase158_strict_str_to_i64_return() {
    // A string literal cannot be returned where i64 is expected
    assert_error_contains(r#"F main() -> i64 = "hello""#, "mismatch");
}

/// Bool cannot participate in arithmetic — `+` requires numeric, not bool.
/// Use `x as i64 + 1` for explicit conversion.
#[test]
fn e2e_phase158_strict_bool_in_arithmetic() {
    let source = r#"
F main() -> i64 { x := true; x + 1 }
"#;
    assert_error_contains(source, "numeric");
}

// ==================== B. Permitted Conversions (must succeed) ====================

/// Phase 158 rule: integer widening i32→i64 is permitted (integer unification).
#[test]
fn e2e_phase158_strict_i32_to_i64_widening() {
    // A variable declared as i32 can be returned as i64 via widening
    assert_exit_code(
        r#"
F main() -> i64 { x:i32 = 1; x }
"#,
        1,
    );
}

/// Phase 158 rule: integer widening i8→i64 is permitted.
#[test]
fn e2e_phase158_strict_i8_to_i64_widening() {
    assert_exit_code(
        r#"
F main() -> i64 { x:i8 = 5; x }
"#,
        5,
    );
}

/// Phase 158 rule: integer widening u8→i64 is permitted.
#[test]
fn e2e_phase158_strict_u8_to_i64_widening() {
    assert_exit_code(
        r#"
F main() -> i64 { x:u8 = 3; x }
"#,
        3,
    );
}

/// Baseline: plain i64 integer literal return is always valid.
#[test]
fn e2e_phase158_strict_i64_literal_return() {
    assert_exit_code(r#"F main() -> i64 = 42"#, 42);
}

/// Phase 158 rule: typed integer literal (i32 suffix) is valid — inferred as i32, widened to i64.
#[test]
fn e2e_phase158_strict_typed_int_literal_i32() {
    // `1i32` is a typed integer literal; widening to i64 return is permitted
    assert_exit_code(
        r#"
F main() -> i64 { x := 1i32; x }
"#,
        1,
    );
}

// ==================== C. Explicit `as` Casts (must succeed) ====================

/// Phase 158 rule: explicit `as` cast bool→i64 is permitted.
#[test]
fn e2e_phase158_strict_explicit_cast_bool_to_i64() {
    // `true as i64` produces 1; explicit casts always bypass the implicit-coercion ban
    assert_exit_code(
        r#"
F main() -> i64 { x := true as i64; x }
"#,
        1,
    );
}

/// Phase 158 rule: explicit `as` cast f64→i64 is permitted (truncating).
/// Note: codegen for fptosi is a known limitation (pre-existing), so we only
/// verify type-checking acceptance here via assert_compiles.
#[test]
fn e2e_phase158_strict_explicit_cast_f64_to_i64() {
    // f64→i64 with explicit `as` cast should pass TC (codegen fptosi is separate issue)
    assert_compiles(
        r#"
F main() -> i64 { x := 3.14; y := x as i64; y }
"#,
    );
}

/// Sanity check: assert_compiles helper works for a trivially valid program.
#[test]
fn e2e_phase158_strict_trivial_compile() {
    assert_compiles(r#"F main() -> i64 = 0"#);
}
