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
//! Also covers the no-annotation `Vec<Vec<u8>>` path: a `Vec.push(inner)`
//! call must propagate the pushed element type back into the receiver local
//! before later indexing chooses the element LLVM load type.

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
fn vec_of_vec_no_annotation_loses_inner_type() {
    // Closer reduction of vaisdb test_btree_node.ll:1848 shape.
    //
    // Vais source (line 241-253 of vaisdb btree/node.vais):
    //   keys_owned := mut Vec.with_capacity(...)  <-- no annotation
    //   keys_owned.push(key_copy)                  <-- key_copy is Vec<u8>
    //   key_refs.push(&keys_owned[i])              <-- &Vec<u8> as &[u8]
    //
    // Without explicit `keys_owned: Vec<Vec<u8>>`, codegen-local inference
    // must still learn the inner element type from the push site. Otherwise
    // indexing later loads the inner Vec as raw i64.
    let ir = gen_ir(
        r#"
        F take_slice(s: &[u8]) -> i64 {
            R 0;
        }
        F main() -> i64 {
            keys := mut Vec.with_capacity(2);
            inner := mut Vec.with_capacity(1);
            inner.push(42u8);
            keys.push(inner);
            R take_slice(&keys[0]);
        }
        "#,
    );
    let violations = find_callarg_struct_scalar_mismatches(&ir);
    assert!(
        violations.is_empty(),
        "Vec<Vec<u8>> no-annotation erasure:\n{:#?}\nIR:\n{}",
        violations,
        ir
    );
}

#[test]
fn vec_of_vec_indexing_simple_repro_passes() {
    // Smaller standalone repro of the vaisdb test_btree_node.ll:1848
    // shape: Vec<Vec<u8>> with explicit type annotations.
    //
    // PASSES — the compiler correctly resolves the inner type when (a) the
    // outer Vec is type-annotated and (b) explicit `inner: Vec<u8>` is
    // declared before push.
    //
    // The vaisdb failure occurs in a more complex context where:
    //   - keys_owned has no explicit type annotation
    //   - inner key_copy is constructed via Vec.with_capacity inside a loop
    //   - cross-module + generic instantiation produce inference path that
    //     loses the inner Vec<u8> shape
    //
    // Reduction of the actual failing vaisdb shape is a separate task
    // (vaisdb Task #11 follow-up). For now this test guards the simple case.
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
