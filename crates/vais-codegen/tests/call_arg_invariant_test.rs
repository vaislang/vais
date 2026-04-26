//! Call-argument invariant verification (ADR 0001 §1 R2).
//!
//! Invariant (Mini Pillar 1 extension, commit 041685e6):
//!     "When a function parameter expects `{ i8*, i64 }` (slice fat-ptr) and
//!      the argument's emitted LLVM type is `%Vec*` / `%Vec$T*`, the call
//!      site must emit Vec→fat-ptr coercion (load data + load len + 2x
//!      insertvalue) before passing."
//!
//! These tests block regressions of the structural call-arg guard added in
//! method_call.rs.
//!
//! Currently covers:
//!   - method calls where param is `Slice` and arg is `&Vec<T>`
//!
//! Known limitation (vaisdb Task #11): when type-inference erases the
//! element type at the indexing path (e.g. `Vec<Vec<u8>>` → `Vec<i64>`),
//! the indexing site emits raw `load i64` and the call-arg guard cannot
//! recover the lost type. That root cause lives in TC inference, not in
//! call-arg coercion. The `vec_of_vec_indexing_loses_element_type` test
//! below documents that case as `#[ignore]` until the deeper fix lands.

use vais_codegen::CodeGenerator;
use vais_parser::parse;

fn gen_ir(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed:\n{}\nErr: {}", source, e))
}

/// Find every `call ... @<fn>(...)` line and parse out (fn_name, args_text).
fn find_calls(ir: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in ir.lines() {
        let trimmed = line.trim();
        // Match both `%t = call ... @fn(...)` and `call void @fn(...)`.
        let payload = if let Some(idx) = trimmed.find(" = call ") {
            &trimmed[idx + " = call ".len()..]
        } else if let Some(rest) = trimmed.strip_prefix("call ") {
            rest
        } else {
            continue;
        };
        // Skip past return type up to `@`.
        let Some(at_idx) = payload.find('@') else {
            continue;
        };
        let after_at = &payload[at_idx + 1..];
        let Some(paren_idx) = after_at.find('(') else {
            continue;
        };
        let fn_name = after_at[..paren_idx].trim().to_string();
        // Skip runtime helpers (libc, intrinsics, vais runtime).
        if fn_name.starts_with("__")
            || fn_name.starts_with("llvm.")
            || matches!(
                fn_name.as_str(),
                "abort"
                    | "exit"
                    | "free"
                    | "malloc"
                    | "memcpy"
                    | "memset"
                    | "puts"
                    | "printf"
                    | "strcmp"
                    | "strlen"
            )
        {
            continue;
        }
        // Find the matching close-paren (track depth for nested aggregate types).
        let after_paren = &after_at[paren_idx + 1..];
        let mut depth = 1;
        let mut end_idx = None;
        for (i, c) in after_paren.char_indices() {
            match c {
                '(' | '{' | '[' => depth += 1,
                ')' | '}' | ']' => {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(end) = end_idx else {
            continue;
        };
        let args_text = after_paren[..end].to_string();
        out.push((fn_name, args_text));
    }
    out
}

/// Look for `call ... @<fn>(... { i8*, i64 } %scalar_named_value)` where
/// the named value is provably i64 in upstream IR. This is the bug shape
/// that was failing in vaisdb test_btree_node.ll:1848.
fn find_callarg_struct_scalar_mismatches(ir: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (fn_name, args) in find_calls(ir) {
        // Quick filter: only inspect calls whose arg list mentions the
        // fat-ptr type literally — we're hunting that specific class.
        if !args.contains("{ i8*, i64 }") {
            continue;
        }
        // For each `{ i8*, i64 } %name` arg, check if %name was emitted as
        // a non-aggregate (ie `load i64, ...` upstream).
        // We use a lazy heuristic: split args on `, ` *outside* braces.
        let mut depth = 0;
        let mut start = 0;
        let bytes = args.as_bytes();
        let mut chunks = Vec::new();
        for (i, &b) in bytes.iter().enumerate() {
            match b {
                b'{' | b'(' | b'[' => depth += 1,
                b'}' | b')' | b']' => depth -= 1,
                b',' if depth == 0 => {
                    chunks.push(args[start..i].trim().to_string());
                    start = i + 1;
                }
                _ => {}
            }
        }
        if start < args.len() {
            chunks.push(args[start..].trim().to_string());
        }
        for chunk in chunks {
            if let Some(rest) = chunk.strip_prefix("{ i8*, i64 } ") {
                let val = rest.trim();
                if !val.starts_with('%') {
                    continue;
                }
                // Look upstream for `<val> = load i64, ...` pattern.
                let load_pat = format!("{} = load i64,", val);
                if ir.contains(&load_pat) {
                    out.push((fn_name.clone(), chunk));
                }
            }
        }
    }
    out
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn callarg_invariant_holds_on_simple_vec_to_slice_push() {
    // Direct case where the type-inference path correctly identifies arg as
    // Ref(Vec<u8>) → Slice<u8>. method_call's existing
    // is_vec_to_slice_coercion path handles this.
    let ir = gen_ir(
        r#"
        F take_slice(s: &[u8]) -> i64 {
            R 0;
        }
        F main() -> i64 {
            v: Vec<u8> := mut Vec.with_capacity(4);
            v.push(1u8);
            R take_slice(&v);
        }
        "#,
    );
    let violations = find_callarg_struct_scalar_mismatches(&ir);
    assert!(
        violations.is_empty(),
        "call-arg invariant violated:\n{:#?}\nIR:\n{}",
        violations,
        ir
    );
}

#[test]
fn find_callarg_detector_works_on_synthetic_bad_ir() {
    // Hand-craft the bug shape and confirm the detector sees it.
    let bad_ir = r#"
define i64 @main() {
entry:
  %x = load i64, i64* %ptr
  %r = call i64 @foo({ i8*, i64 } %x)
  ret i64 %r
}
"#;
    let violations = find_callarg_struct_scalar_mismatches(bad_ir);
    assert!(
        !violations.is_empty(),
        "detector failed on synthetic bad IR:\n{}",
        bad_ir
    );
    assert_eq!(violations[0].0, "foo");
}

#[test]
#[ignore = "vaisdb Task #11: deeper TC type-inference fix needed (Vec<Vec<u8>> erasure)"]
fn vec_of_vec_indexing_loses_element_type() {
    // Direct repro of vaisdb test_btree_node.ll:1848 pattern.
    //
    // Vec<Vec<u8>> indexing path erases inner Vec<u8> to i64. After
    // `keys_owned[i]` returns a raw i64, `&keys_owned[i]` cannot recover
    // the original Vec<u8> shape, and the slice-fat-ptr call-arg coercion
    // has no way to fire.
    //
    // This test is currently #[ignore]'d — when the upstream TC inference
    // fix lands (or a codegen-local Vec<Vec<T>> indexing patch lands),
    // remove the #[ignore] and the test will pass.
    let ir = gen_ir(
        r#"
        F take_slice(s: &[u8]) -> i64 {
            R 0;
        }
        F main() -> i64 {
            keys: Vec<Vec<u8>> := mut Vec.with_capacity(2);
            inner: Vec<u8> := mut Vec.with_capacity(1);
            inner.push(42u8);
            keys.push(inner);
            R take_slice(&keys[0]);
        }
        "#,
    );
    let violations = find_callarg_struct_scalar_mismatches(&ir);
    assert!(
        violations.is_empty(),
        "Vec<Vec<u8>> indexing erasure regression:\n{:#?}\nIR:\n{}",
        violations,
        ir
    );
}
