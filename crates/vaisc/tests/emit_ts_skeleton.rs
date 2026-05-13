//! Skeleton tests for `vaisc emit-ts` (Master Plan v16 Order Step 8).
//!
//! Three tests:
//!   1. `emit_ts_basic_struct`         — happy path: pub struct with primitive fields
//!   2. `emit_ts_unsupported_field_errors` — raw pointer `*i64` triggers EMIT_TS_009 (Stage 2)
//!   3. `emit_ts_skips_private_struct` — only pub structs appear in the output

use std::path::PathBuf;
use std::process::Command;

// ---------------------------------------------------------------------------
// Helper: path to the compiled vaisc binary
// ---------------------------------------------------------------------------

fn vaisc_binary() -> PathBuf {
    // Cargo injects CARGO_BIN_EXE_vaisc when running `cargo test -p vaisc`.
    let built = env!("CARGO_BIN_EXE_vaisc");
    if !built.is_empty() {
        return PathBuf::from(built);
    }
    if let Ok(p) = std::env::var("VAISC") {
        return PathBuf::from(p);
    }
    PathBuf::from("target/debug/vaisc")
}

// ---------------------------------------------------------------------------
// Test 1 — happy path: pub struct with all primitive fields
// ---------------------------------------------------------------------------

#[test]
fn emit_ts_basic_struct() {
    let vaisc = vaisc_binary();
    let dir = tempfile::tempdir().expect("tempdir");

    // Write a minimal schema with a pub struct containing all four primitive kinds.
    let input_path = dir.path().join("user.vais");
    let output_path = dir.path().join("user.d.ts");

    std::fs::write(
        &input_path,
        "P S User {\n  id: i64,\n  name: str,\n  active: bool,\n}\n",
    )
    .expect("write input");

    let output = Command::new(&vaisc)
        .args([
            "emit-ts",
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .env("VAIS_NO_UPDATE_CHECK", "1")
        .output()
        .expect("failed to run vaisc emit-ts");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code().unwrap_or(1),
        0,
        "expected exit 0 for basic struct; stderr:\n{}\nstdout:\n{}",
        stderr,
        stdout
    );

    // The output file must exist.
    assert!(
        output_path.exists(),
        "expected .d.ts file to be written at {}",
        output_path.display()
    );

    let dts = std::fs::read_to_string(&output_path).expect("read .d.ts");

    // Must contain the TS interface declaration.
    assert!(
        dts.contains("export interface User {"),
        "expected 'export interface User {{' in .d.ts; got:\n{}",
        dts
    );
    // Stage 4: integer types lower to width-branded TS aliases (VaisI64, etc.)
    // so that an i64→f64 schema change is a typed change in TS.
    assert!(
        dts.contains("readonly id: VaisI64;"),
        "expected 'readonly id: VaisI64;'; got:\n{}",
        dts
    );
    assert!(
        dts.contains("readonly name: string;"),
        "expected 'readonly name: string;'; got:\n{}",
        dts
    );
    assert!(
        dts.contains("readonly active: boolean;"),
        "expected 'readonly active: boolean;'; got:\n{}",
        dts
    );
    // The brand prelude must be present so consumers can type the field.
    assert!(
        dts.contains("export type VaisI64"),
        "expected brand prelude 'export type VaisI64' in .d.ts; got:\n{}",
        dts
    );
}

// ---------------------------------------------------------------------------
// Test 2 — raw pointer `*i64` triggers EMIT_TS_009 (Stage 2 specific code)
// ---------------------------------------------------------------------------

#[test]
fn emit_ts_unsupported_field_errors() {
    let vaisc = vaisc_binary();
    let dir = tempfile::tempdir().expect("tempdir");

    let input_path = dir.path().join("bad.vais");
    let output_path = dir.path().join("bad.d.ts");

    // Stage 2 routes raw pointer `*i64` to EMIT_TS_009 specifically.
    std::fs::write(&input_path, "P S X {\n  v: *i64,\n}\n").expect("write input");

    let output = Command::new(&vaisc)
        .args([
            "emit-ts",
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .env("VAIS_NO_UPDATE_CHECK", "1")
        .output()
        .expect("failed to run vaisc emit-ts");

    // Must exit non-zero.
    assert_ne!(
        output.status.code().unwrap_or(0),
        0,
        "expected non-zero exit when an unsupported field type is present"
    );

    // Stage 2: the error output must contain EMIT_TS_009 (raw pointer specific code).
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("EMIT_TS_009"),
        "expected 'EMIT_TS_009' in stderr (Stage 2 specific code for raw pointer); got:\n{}",
        stderr
    );
    assert!(
        stderr.contains('v'),
        "expected field name 'v' in stderr; got:\n{}",
        stderr
    );
}

// ---------------------------------------------------------------------------
// Test 3 — private structs are skipped; only pub structs appear in .d.ts
// ---------------------------------------------------------------------------

#[test]
fn emit_ts_skips_private_struct() {
    let vaisc = vaisc_binary();
    let dir = tempfile::tempdir().expect("tempdir");

    let input_path = dir.path().join("mixed.vais");
    let output_path = dir.path().join("mixed.d.ts");

    // Two structs: one public, one private.
    std::fs::write(
        &input_path,
        "P S Public {\n  x: i64,\n}\nS Private {\n  y: i64,\n}\n",
    )
    .expect("write input");

    let output = Command::new(&vaisc)
        .args([
            "emit-ts",
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .env("VAIS_NO_UPDATE_CHECK", "1")
        .output()
        .expect("failed to run vaisc emit-ts");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code().unwrap_or(1),
        0,
        "expected exit 0 for mixed public/private struct schema; stderr:\n{}\nstdout:\n{}",
        stderr,
        stdout
    );

    assert!(
        output_path.exists(),
        ".d.ts not written at {}",
        output_path.display()
    );

    let dts = std::fs::read_to_string(&output_path).expect("read .d.ts");

    // The public struct must be present.
    assert!(
        dts.contains("export interface Public {"),
        "expected 'Public' interface in .d.ts; got:\n{}",
        dts
    );

    // The private struct must NOT be present.
    assert!(
        !dts.contains("Private"),
        "expected 'Private' to be absent from .d.ts; got:\n{}",
        dts
    );
}
