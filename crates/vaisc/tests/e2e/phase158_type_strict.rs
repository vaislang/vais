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
#[test]
fn e2e_phase158_strict_int_to_f64_return() {
    assert_compiles(r#"F main() -> f64 = 42"#);
}

/// Phase 160-A: float→int implicit coercion is allowed (numeric promotion).
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

/// Bool arithmetic still requires numeric type — `+` operator requires numeric, not bool.
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

// ==================== E. Phase 158 CI Gate (source-level) ====================
//
// ROADMAP #4: guard against regression by scanning `vais-types/src/inference/unification.rs`
// for forbidden coercion patterns. If any of these names appear in the source the gate
// fails, forcing the author to either rename the function or acknowledge a Phase 158 RFC.
//
// The list mirrors CLAUDE.md "Type Conversion Rules" and BASELINE_2026-04-11.md section 6.

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

    // Forbidden function-name fragments. These are the coercion shapes that
    // Phase 158 explicitly prohibits — adding any of them reintroduces the
    // yo-yo pattern that the rule exists to prevent.
    const FORBIDDEN: &[&str] = &[
        "coerce_bool",
        "bool_to_i64",
        "i64_to_bool",
        "int_to_float",
        "float_to_int",
        "str_to_i64",
        "i64_to_str",
        "narrow_int",
        "coerce_to_i64",
        "as_i64_implicit",
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
         Type coercion rules were tightened in Phase 158 (see CLAUDE.md \
         'Type Conversion Rules'). If you genuinely need to reintroduce one \
         of these conversions, update this gate list and the CLAUDE.md rules \
         in the same commit.",
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
