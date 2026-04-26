//! ret invariant verification (ADR 0001 §1 R2).
//!
//! Invariant (Mini Pillar 1, commit 7cfc5caf):
//!     "Every emitted `ret <ty> <val>` instruction has val's LLVM type == <ty>."
//!
//! These tests block regressions of `coerce_ret_value`. If a future change
//! breaks the invariant for one of the migrated emit sites, the relevant
//! test panics with the offending function name and IR fragment.
//!
//! Currently covers:
//!   - `stmt_visitor::Stmt::Return` path (only site migrated to coerce_ret_value)
//!   - i32→i64 / float→double / %Vec*→{i8*,i64} coercion classes
//!
//! Migration tracker: 30+ other ret emit sites still use inline coercion
//! (function_gen/codegen.rs ~20, string_ops.rs 6, emit.rs 1, stmt.rs 1,
//! stmt_visitor poll/async 2). As each migrates, add a test here.

use vais_codegen::CodeGenerator;
use vais_parser::parse;

/// Compile a small Vais program and return the emitted IR.
fn gen_ir(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed:\n{}\nErr: {}", source, e))
}

/// Strip `; ...` IR debug comment from a line, returning the active
/// instruction tokens. Helps when emit appends `, !dbg !N` etc.
fn strip_debug(line: &str) -> String {
    let line = line.trim();
    let cut = line.find(", !").unwrap_or(line.len());
    line[..cut].to_string()
}

/// Return true if this function name is a runtime helper / lib intrinsic
/// (not user code emitted by codegen). These have hand-rolled IR that uses
/// pre-coercion conventions and is not subject to coerce_ret_value.
fn is_runtime_helper(name: &str) -> bool {
    // Convention: runtime helpers start with `__` (vais runtime), have known
    // libc names (stat, fopen, malloc, ...), or are intrinsics.
    name.starts_with("__")
        || name.starts_with("llvm.")
        || matches!(
            name,
            "stat"
                | "fopen_ptr"
                | "fopen"
                | "fclose"
                | "fread"
                | "fwrite"
                | "fseek"
                | "ftell"
                | "malloc"
                | "free"
                | "memcpy"
                | "memset"
                | "abort"
                | "exit"
                | "printf"
                | "puts"
                | "strlen"
                | "strcmp"
                | "strncmp"
                | "strcpy"
                | "strncpy"
        )
}

/// Find every `ret <ty> <val>` line in the IR. Returns (function_name, ret_line).
/// Skips runtime helper functions whose IR is hand-rolled (not subject to
/// coerce_ret_value).
fn find_ret_instructions(ir: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut current_fn = String::from("<unknown>");
    let mut current_is_helper = false;
    for line in ir.lines() {
        let trimmed = line.trim();
        // Track current function so panic messages are debuggable.
        if let Some(rest) = trimmed.strip_prefix("define ") {
            // e.g. `define i64 @main() {` or `define { i8*, i64 } @foo() {`
            if let Some(at_idx) = rest.find('@') {
                let after_at = &rest[at_idx + 1..];
                let end = after_at
                    .find('(')
                    .unwrap_or_else(|| after_at.find(' ').unwrap_or(after_at.len()));
                current_fn = after_at[..end].to_string();
                current_is_helper = is_runtime_helper(&current_fn);
            }
        }
        if current_is_helper {
            continue;
        }
        let active = strip_debug(line);
        if active.starts_with("ret ") && active != "ret void" {
            out.push((current_fn.clone(), active));
        }
    }
    out
}

/// Extract a leading LLVM type from a string. Handles aggregates with
/// internal commas (`{ i8*, i64 }`) by tracking brace depth, then returns
/// the type and the remainder.
fn extract_leading_type(s: &str) -> Option<(String, &str)> {
    let s = s.trim_start();
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    if bytes[0] == b'{' {
        // Aggregate — find matching `}`.
        let mut depth = 0;
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'{' {
                depth += 1;
            } else if b == b'}' {
                depth -= 1;
                if depth == 0 {
                    let mut end = i + 1;
                    // Include trailing `*`s (pointer-to-aggregate).
                    while end < bytes.len() && bytes[end] == b'*' {
                        end += 1;
                    }
                    return Some((s[..end].to_string(), &s[end..]));
                }
            }
        }
        None
    } else {
        // Simple type — token until first whitespace or comma.
        let end = bytes
            .iter()
            .position(|&b| b == b' ' || b == b',')
            .unwrap_or(bytes.len());
        Some((s[..end].to_string(), &s[end..]))
    }
}

/// Like extract_leading_type but returns just the type (caller doesn't need
/// remainder). Falls back to the whole string if parsing fails.
fn extract_aggregate_or_simple_type(s: &str) -> String {
    extract_leading_type(s)
        .map(|(ty, _)| ty)
        .unwrap_or_else(|| s.trim().to_string())
}

/// Parse a `ret <ty> <val>` line into (ty, val).
fn parse_ret_line(line: &str) -> Option<(String, String)> {
    let body = line.strip_prefix("ret ")?.trim();
    let (ty, rest) = extract_leading_type(body)?;
    let val = rest.trim().to_string();
    if val.is_empty() {
        return None;
    }
    Some((ty, val))
}

/// Reverse-scan the IR to find what type `val` was last assigned. Returns the
/// LHS-declared LLVM type (string before `=`-RHS keyword we recognize).
///
/// This is a coarse heuristic but sufficient for Mini Pillar 1 verification:
/// we only need to detect `ret { i8*, i64 } %v` where `%v` was actually
/// declared as `i8*` or `i64` (the bug we just fixed).
fn last_assigned_type(ir: &str, val: &str) -> Option<String> {
    // Find lines like `  %v = <op> <type-stuff> ...`.
    for line in ir.lines().rev() {
        let line = strip_debug(line);
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(val) {
            if let Some(rhs) = rest.trim_start().strip_prefix('=') {
                let rhs = rhs.trim_start();
                // Recognize a few common RHS forms.
                if let Some(rest) = rhs.strip_prefix("insertvalue ") {
                    // `insertvalue { i8*, i64 } undef, ...` → result type is
                    // the aggregate before the value-list. The aggregate may
                    // itself contain `,` (e.g. `{ i8*, i64 }`), so we must
                    // close-paren the brace level before splitting.
                    return Some(extract_aggregate_or_simple_type(rest));
                }
                if let Some(rest) = rhs.strip_prefix("load ") {
                    // `load <ty>, <ty>* %ptr` → result is <ty>.
                    let ty_end = rest.find(",").unwrap_or(rest.len());
                    return Some(rest[..ty_end].trim().to_string());
                }
                if let Some(rest) = rhs.strip_prefix("getelementptr ") {
                    // GEP returns a pointer to the indexed type. We approximate
                    // by saying the result is a pointer (any aggregate result
                    // here would be a regression).
                    let _ = rest;
                    return Some("<gep-ptr>".to_string());
                }
                if let Some(rest) = rhs.strip_prefix("call ") {
                    // `call <ret-ty> @<fn>(...)` → result is <ret-ty>.
                    let ty_end = rest
                        .find(" @")
                        .or_else(|| rest.find(" %"))
                        .unwrap_or(rest.len());
                    return Some(rest[..ty_end].trim().to_string());
                }
                if let Some(rest) = rhs.strip_prefix("trunc ")
                    .or_else(|| rhs.strip_prefix("sext "))
                    .or_else(|| rhs.strip_prefix("zext "))
                    .or_else(|| rhs.strip_prefix("fpext "))
                    .or_else(|| rhs.strip_prefix("fptrunc "))
                    .or_else(|| rhs.strip_prefix("fptosi "))
                    .or_else(|| rhs.strip_prefix("sitofp "))
                    .or_else(|| rhs.strip_prefix("inttoptr "))
                    .or_else(|| rhs.strip_prefix("ptrtoint "))
                    .or_else(|| rhs.strip_prefix("bitcast "))
                {
                    // `<op> <src-ty> %v to <dst-ty>` → result is <dst-ty>.
                    let to_idx = rest.rfind(" to ")?;
                    return Some(rest[to_idx + 4..].trim().to_string());
                }
                // Other RHS forms: return None (caller treats as "unknown,
                // skip strict check").
                return None;
            }
        }
    }
    None
}

/// Scan IR for ret/value type mismatches that the invariant forbids.
/// Returns the list of (function, ret_line, val, ret_ty, val_ty) violations.
fn find_ret_violations(ir: &str) -> Vec<(String, String, String, String, String)> {
    let mut violations = Vec::new();
    for (fn_name, line) in find_ret_instructions(ir) {
        let Some((ret_ty, val)) = parse_ret_line(&line) else {
            continue;
        };
        // Skip literals (no `%`/`@`) — type can't disagree with itself.
        if !val.starts_with('%') && !val.starts_with('@') {
            continue;
        }
        // We can only check when we recognize the assignment form.
        let Some(val_ty) = last_assigned_type(ir, &val) else {
            continue;
        };
        // <gep-ptr> is opaque — we record-only when ret expects a non-pointer
        // aggregate (the exact bug Task #6 fixed).
        if val_ty == "<gep-ptr>" {
            if ret_ty == "{ i8*, i64 }" {
                violations.push((fn_name, line, val, ret_ty, val_ty));
            }
            continue;
        }
        if val_ty != ret_ty {
            violations.push((fn_name, line, val, ret_ty, val_ty));
        }
    }
    violations
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn ret_invariant_holds_on_simple_int_return() {
    let ir = gen_ir(
        r#"
        F main() -> i64 {
            R 42;
        }
        "#,
    );
    let violations = find_ret_violations(&ir);
    assert!(
        violations.is_empty(),
        "ret invariant violated:\n{:#?}\nIR:\n{}",
        violations,
        ir
    );
}

#[test]
fn ret_invariant_holds_on_int_width_coerce() {
    // Function returns i32 but body produces i64 (sext path) — coerce_int_width
    // must trunc back to i32 before ret.
    let ir = gen_ir(
        r#"
        F narrow() -> i32 {
            x: i32 := 42;
            R x;
        }
        "#,
    );
    let violations = find_ret_violations(&ir);
    assert!(
        violations.is_empty(),
        "ret i32 must not get an i64 value:\n{:#?}\nIR:\n{}",
        violations,
        ir
    );
}

#[test]
fn ret_invariant_holds_on_float_width_coerce() {
    // f32 declared, body produces double — coerce_float_width must fptrunc.
    let ir = gen_ir(
        r#"
        F narrow_float() -> f32 {
            x: f32 := 3.14;
            R x;
        }
        "#,
    );
    let violations = find_ret_violations(&ir);
    assert!(
        violations.is_empty(),
        "ret f32 must not get a double value:\n{:#?}\nIR:\n{}",
        violations,
        ir
    );
}

#[test]
fn ret_invariant_blocks_vec_ptr_to_fat_ptr_regression() {
    // Direct reproduction of vaisdb test_btree_node.ll:1736 pattern.
    // Function returns &[u8] (-> { i8*, i64 }); body is `R &self.data` where
    // self.data: Vec<u8>. Without coerce_ret_value, the ret would emit
    // `ret { i8*, i64 } %vec_ptr` with %vec_ptr typed as %Vec*.
    //
    // Mini Pillar 1 (commit 7cfc5caf) makes coerce_ret_value:
    //   1. detect val_ty == "%Vec$T*" && ret_ty == "{ i8*, i64 }"
    //   2. emit load Vec.data + load Vec.len + insertvalue 2-step chain
    //   3. update val to the `insertvalue` result (typed { i8*, i64 })
    //
    // If anyone deletes that branch or breaks the path, this test fires.
    let ir = gen_ir(
        r#"
        S Holder {
            data: Vec<u8>,
        }
        X Holder {
            F view(self) -> &[u8] {
                R &self.data;
            }
        }
        F main() -> i64 {
            R 0;
        }
        "#,
    );
    let violations = find_ret_violations(&ir);
    assert!(
        violations.is_empty(),
        "Vec→fat-ptr coerce regression: ret {{ i8*, i64 }} emitted with non-fat-ptr value:\n{:#?}\nIR:\n{}",
        violations,
        ir
    );
}

#[test]
fn find_ret_violations_detects_synthetic_mismatch() {
    // Sanity check: hand-craft IR with a known violation and confirm we catch it.
    let bad_ir = r#"
define { i8*, i64 } @bad() {
entry:
  %p = getelementptr i8, i8* null, i64 0
  ret { i8*, i64 } %p
}
"#;
    let violations = find_ret_violations(bad_ir);
    assert!(
        !violations.is_empty(),
        "violation detector failed on synthetic bad IR:\n{}",
        bad_ir
    );
    let (fn_name, _, _, ret_ty, _) = &violations[0];
    assert_eq!(fn_name, "bad");
    assert_eq!(ret_ty, "{ i8*, i64 }");
}
