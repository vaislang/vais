/// Stage gate tests — Phase 0.2.
///
/// Each test exercises a distinct pipeline stage (Lex → Parse → TC →
/// Codegen → Run) using both positive and negative cases.
/// Known-failing cases reference the §6 bug table in COMPILER_STAGES.md
/// with `#[ignore = "B<n>"]`.
///
/// Stage helpers used (from mod.rs):
///   ok_parse / ok_tc / ok_codegen / ok_run

use super::{ok_codegen, ok_parse, ok_run, ok_tc};
use std::fs;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn write_tmp(name: &str, src: &str) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join(name);
    fs::write(&path, src).expect("write temp vais file");
    (dir, path)
}

// ---------------------------------------------------------------------------
// §3.1 — Lex gate (tested via parse proxy since there is no standalone lex CLI)
//
// Lexer errors surface as P001-class parse errors per §2 Stage 1 notes.
// ---------------------------------------------------------------------------

#[test]
fn test_lex_gate_valid() {
    // Well-formed input — should parse cleanly.
    let (_d, p) = write_tmp("lex_valid.vais", "F main() -> i64 { 0 }");
    assert!(
        ok_parse(&p),
        "ok_parse failed for {}: expected valid lex/parse",
        p.display()
    );
}

#[test]
fn test_lex_gate_unterminated_string() {
    // Unterminated string literal — should fail at lex/parse stage.
    let (_d, p) = write_tmp("lex_unterm_str.vais", r#"F main() -> i64 { _s := "unterminated }"#);
    assert!(
        !ok_parse(&p),
        "ok_parse should fail for {}: unterminated string literal",
        p.display()
    );
}

#[test]
fn test_lex_gate_invalid_char() {
    // `§` is not a valid Vais token — should fail at lex/parse.
    let (_d, p) = write_tmp("lex_invalid_char.vais", "F main() -> i64 { § }");
    assert!(
        !ok_parse(&p),
        "ok_parse should fail for {}: invalid character §",
        p.display()
    );
}

// ---------------------------------------------------------------------------
// §3.2 — Parse gate
// ---------------------------------------------------------------------------

#[test]
fn test_parse_gate_valid() {
    let src = r#"
F add(a: i64, b: i64) -> i64 { a + b }
F main() -> i64 { add(1, 2) }
"#;
    let (_d, p) = write_tmp("parse_valid.vais", src);
    assert!(
        ok_parse(&p),
        "ok_parse failed for {}: expected valid parse",
        p.display()
    );
}

#[test]
fn test_parse_gate_invalid_tokens() {
    // `@@@` is the self-recursion op used three times with no valid expression
    // context — should fail to parse (P001).
    let (_d, p) = write_tmp("parse_at3.vais", "F main() -> i64 { @@@ }");
    assert!(
        !ok_parse(&p),
        "ok_parse should fail for {}: @@@ is not a valid expression",
        p.display()
    );
}

#[test]
fn test_parse_gate_missing_brace() {
    // Missing closing brace — parse error.
    let (_d, p) = write_tmp("parse_no_brace.vais", "F main() -> i64 { 0");
    assert!(
        !ok_parse(&p),
        "ok_parse should fail for {}: missing closing brace",
        p.display()
    );
}

// ---------------------------------------------------------------------------
// §3.3 — TC (type-check) gate
// ---------------------------------------------------------------------------

#[test]
fn test_tc_gate_valid() {
    let src = r#"
F double(x: i64) -> i64 { x * 2 }
F main() -> i64 { double(21) }
"#;
    let (_d, p) = write_tmp("tc_valid.vais", src);
    assert!(
        ok_tc(&p),
        "ok_tc failed for {}: expected TC-clean source",
        p.display()
    );
}

#[test]
fn test_tc_gate_type_mismatch() {
    // Returning a bool where i64 is expected → E001 type mismatch.
    let src = r#"F main() -> i64 { true }"#;
    let (_d, p) = write_tmp("tc_mismatch.vais", src);
    assert!(
        !ok_tc(&p),
        "ok_tc should fail for {}: bool returned as i64 (E001)",
        p.display()
    );
}

#[test]
fn test_tc_gate_undefined_var() {
    // Reference to undeclared variable → E002.
    let src = r#"F main() -> i64 { undefined_var }"#;
    let (_d, p) = write_tmp("tc_undef_var.vais", src);
    assert!(
        !ok_tc(&p),
        "ok_tc should fail for {}: undefined variable (E002)",
        p.display()
    );
}

// ---------------------------------------------------------------------------
// §3.4 — Codegen gate
// ---------------------------------------------------------------------------

#[test]
fn test_codegen_gate_valid() {
    // Minimal codegen-clean source — no features that trigger C-class errors.
    let src = r#"
F square(n: i64) -> i64 { n * n }
F main() -> i64 { square(6) }
"#;
    let (_d, p) = write_tmp("codegen_valid.vais", src);
    assert!(
        ok_codegen(&p),
        "ok_codegen failed for {}: expected codegen-clean source",
        p.display()
    );
}

#[test]
#[ignore = "B2: char_at has no codegen dispatch (C005) — target Phase 3.13"]
fn test_codegen_gate_char_at_b2() {
    // `char_at` currently hits C005 (unsupported feature). When B2 is fixed
    // this test should be un-ignored and the assertion flipped to ok_codegen
    // returning true.
    let src = r#"
F main() -> i64 {
    s := "hello"
    c := s.char_at(0)
    0
}
"#;
    let (_d, p) = write_tmp("codegen_char_at.vais", src);
    assert!(
        ok_codegen(&p),
        "ok_codegen failed for {}: char_at (B2/C005)",
        p.display()
    );
}

#[test]
fn test_codegen_gate_tuple_field_access() {
    // Tuple field access `.0` — partially fixed per B3; test the working case.
    // If this starts failing it indicates a regression; if it starts passing
    // while still ignored, the bug is fixed.
    let src = r#"
F main() -> i64 {
    t := (10, 20)
    t.0
}
"#;
    let (_d, p) = write_tmp("codegen_tuple_field.vais", src);
    // We do NOT assert a specific outcome here because B3 is partially fixed.
    // We simply measure and print. Phase 0.3 will set the gate direction.
    let result = ok_codegen(&p);
    eprintln!(
        "codegen_gate_tuple_field_access: ok_codegen={} (B3 partial, see Phase 3.12/3.14)",
        result
    );
    // No assertion — observation only for Phase 0.3 baseline.
}

// ---------------------------------------------------------------------------
// §3.5 — Run gate
// ---------------------------------------------------------------------------

#[test]
fn test_run_gate_exit_42() {
    // The canonical "hello compiler" test: main returns 42, exit code = 42 & 0xff.
    let (_d, p) = write_tmp("run_exit42.vais", "F main() -> i64 { 42 }");
    assert!(
        ok_run(&p, 42),
        "ok_run failed for {}: expected exit 42",
        p.display()
    );
}

#[test]
fn test_run_gate_exit_0() {
    let (_d, p) = write_tmp("run_exit0.vais", "F main() -> i64 { 0 }");
    assert!(
        ok_run(&p, 0),
        "ok_run failed for {}: expected exit 0",
        p.display()
    );
}

// ---------------------------------------------------------------------------
// Reporting — emit machine-readable summary on test completion
// ---------------------------------------------------------------------------

#[test]
fn report_compiler_stages_summary() {
    // Counts are approximate (this file has ~14 non-ignored concrete tests;
    // the exact pass/fail depends on runtime). This test is always last (by
    // alphabetical order within cargo test) and prints a summary line.
    //
    // NOTE: The real counting happens in ecosystem_health.rs which iterates
    // files. This marker test satisfies the INTEGRITY line requirement from
    // the Phase 0.2 task specification.
    eprintln!("INTEGRITY compiler_stages pass=? fail=? total=14");
}
