//! Type conversion E2E tests.
//!
//! Verifies that forbidden structural conversions are rejected at compile time,
//! and that explicitly permitted numeric adaptation and explicit `as` casts
//! continue to work.
//!
//! Current rules (see LANGUAGE_SPEC.md "Type Conversion Rules"):
//!   Allowed (implicit):  numeric adaptation across integer widths/signs,
//!                        int↔float numeric promotion, f32↔f64 literal inference
//!   Forbidden:           bool↔i64, str↔i64, typed pointer↔i64
//!   All other structural conversions require an explicit conversion boundary.

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

/// Phase 158: bool→i64 implicit coercion is FORBIDDEN (requires `as i64`).
#[test]
fn e2e_phase158_strict_bool_to_i64_return() {
    assert_error_contains(r#"F main() -> i64 = true"#, "mismatch");
}

/// Phase 158: i64→bool implicit coercion is FORBIDDEN (requires comparison).
#[test]
fn e2e_phase158_strict_i64_to_bool_return() {
    assert_error_contains(r#"F main() -> bool = 42"#, "mismatch");
}

/// Phase 160-A: int→float implicit coercion is allowed (numeric promotion).
/// Phase 191: main() float return with integer body now passes through as i64.
#[test]
fn e2e_phase158_strict_int_to_f64_return() {
    assert_exit_code(r#"F main() -> f64 = 42"#, 42);
}

/// Phase 160-A: float→int implicit coercion is allowed (numeric promotion).
/// Phase 191: float literal in i64 return now emits fptosi.
#[test]
fn e2e_phase158_strict_f64_to_i64_return() {
    assert_exit_code(r#"F main() -> i64 = 3.14"#, 3);
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
    assert_exit_code(source, 1);
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

/// Typed pointer values do not implicitly unify with raw i64 addresses.
#[test]
fn e2e_phase158_strict_pointer_to_i64_return() {
    assert_error_contains(
        r#"
F addr(p: *i64) -> i64 = p
F main() -> i64 = 0
"#,
        "mismatch",
    );
}

/// Bool arithmetic still requires numeric type — `+` operator requires numeric, not bool.
#[test]
fn e2e_phase158_strict_bool_in_arithmetic() {
    let source = r#"
F main() -> i64 { x := true; x + 1 }
"#;
    assert_error_contains(source, "numeric");
}

// ==================== B. Permitted Conversions (must succeed) ====================

/// Integer numeric unification permits i32→i64 in this return context.
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

/// Integer numeric unification permits i8→i64 in this return context.
#[test]
fn e2e_phase158_strict_i8_to_i64_widening() {
    assert_exit_code(
        r#"
F main() -> i64 { x:i8 = 5; x }
"#,
        5,
    );
}

/// Integer numeric unification permits u8→i64 in this return context.
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

/// Typed integer literal (i32 suffix) is valid and adapts to i64 context.
#[test]
fn e2e_phase158_strict_typed_int_literal_i32() {
    // `1i32` is a typed integer literal; adaptation to i64 return is permitted.
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
/// Phase 191: fptosi now correctly handles float literal → i64 cast.
#[test]
fn e2e_phase158_strict_explicit_cast_f64_to_i64() {
    assert_exit_code(
        r#"
F main() -> i64 { x := 3.14; y := x as i64; y }
"#,
        3,
    );
}

/// Sanity check: assert_compiles helper works for a trivially valid program.
#[test]
fn e2e_phase158_strict_trivial_compile() {
    assert_exit_code(r#"F main() -> i64 = 0"#, 0);
}

// ==================== E. Phase 158 CI Gate (source-level) ====================
//
// ROADMAP #4: guard against regression by scanning `vais-types/src/inference/unification.rs`
// for forbidden structural coercion patterns. If any of these names appear in
// the source the gate fails, forcing the author to either rename the function or
// update the language conversion rules in the same change.

/// Locate the unification source file regardless of where `cargo test` is run from.
fn find_unification_rs() -> Option<std::path::PathBuf> {
    // Start from CARGO_MANIFEST_DIR (crates/vaisc) and walk up to the workspace root.
    let manifest = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    let mut dir = std::path::PathBuf::from(manifest);
    for _ in 0..5 {
        let candidate = dir.join("crates/vais-types/src/inference/unification.rs");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

#[test]
fn e2e_phase158_ci_gate_no_forbidden_coercions() {
    let path = match find_unification_rs() {
        Some(p) => p,
        None => {
            // If we genuinely can't find the source (e.g. running from a stripped
            // cargo package), skip loudly rather than fail silently. This keeps the
            // gate meaningful when the file is present and skipped otherwise.
            eprintln!(
                "[phase158-gate] skipped: unable to locate vais-types/src/inference/unification.rs"
            );
            return;
        }
    };
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));

    // Forbidden function-name fragments. These are structural coercion shapes
    // that the language explicitly prohibits as implicit unification.
    const FORBIDDEN: &[&str] = &[
        "coerce_bool",
        "bool_to_i64",
        "i64_to_bool",
        "str_to_i64",
        "i64_to_str",
        "coerce_to_i64",
        "as_i64_implicit",
        "pointer_to_i64",
        "i64_to_pointer",
    ];

    let mut hits: Vec<&str> = Vec::new();
    for name in FORBIDDEN {
        if src.contains(name) {
            hits.push(name);
        }
    }
    assert!(
        hits.is_empty(),
        "Phase 158 CI gate: forbidden coercion identifier(s) found in \
         vais-types/src/inference/unification.rs: {:?}. \
         If you genuinely need to reintroduce one of these conversions, update \
         this gate list and LANGUAGE_SPEC.md in the same commit.",
        hits
    );
}

#[test]
fn e2e_phase158_ci_gate_no_vais_tc_nonfatal_escape_hatch() {
    // Phase 158/ROADMAP #4: VAIS_TC_NONFATAL was removed in iter 12. Guard against
    // it creeping back in — any occurrence under `crates/vaisc/src/commands/build/`
    // is a regression.
    let manifest = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(m) => std::path::PathBuf::from(m),
        Err(_) => return, // skip silently — gate only meaningful with CARGO_MANIFEST_DIR
    };
    let build_dir = manifest.join("src/commands/build");
    if !build_dir.exists() {
        eprintln!(
            "[phase158-gate] skipped: build/ dir not found at {}",
            build_dir.display()
        );
        return;
    }
    let mut offenders: Vec<String> = Vec::new();
    let entries = match std::fs::read_dir(&build_dir) {
        Ok(e) => e,
        Err(e) => panic!("failed to read {}: {}", build_dir.display(), e),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if let Ok(src) = std::fs::read_to_string(&path) {
            // Match identifier-level occurrences; a comment that merely mentions
            // the flag inside a prose sentence is fine, but a `std::env::var("VAIS_TC_NONFATAL")`
            // call is not.
            if src.contains("std::env::var(\"VAIS_TC_NONFATAL\"")
                || src.contains("env::var(\"VAIS_TC_NONFATAL\"")
            {
                offenders.push(path.display().to_string());
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "Phase 158 CI gate: `VAIS_TC_NONFATAL` env-var escape hatch reappeared in: {:?}. \
         This hatch was removed in ROADMAP #4 — TC errors must always be fatal.",
        offenders
    );
}
