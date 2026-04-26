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

// -----------------------------------------------------------------------
// P1.4 iter 111 — R2 blocking test for the iter 107 ret-cast migration.
//
// The migrated site at `stmt_visitor.rs:708` now emits the bitcast through
// `TypedEmitter::emit_bitcast_with_prefix`, which automatically calls
// `record_emitted_type(name, dst_ty)` for the cast result. The legacy
// hand-rolled emission (pre-iter 107) skipped that registration, leaving
// the SSA in the i64 fallback path for any downstream consumer that
// queried `actual_llvm_type`.
//
// This test pins the migrated behavior in two ways:
//   1. The emitted IR for the ret-cast site uses the legacy SSA name
//      format `%ret.cast.{N}` byte-for-byte. If a future change switches
//      to the wrapper's default `%t{N}` allocator, the assertion fires.
//   2. The cast line is followed *immediately* by a `load` whose pointer
//      operand is that same `%ret.cast.{N}` SSA. If `record_emitted_type`
//      were skipped (or the cast were emitted in the wrong shape) the
//      pair would not match and downstream type lookup would fall back
//      to i64.
//
// This is an R2 (regression-blocking) test in the ADR 0002 sense: any
// future migration that breaks the wrapper's type-tagging contract for
// this site will fail here before reaching production.
// -----------------------------------------------------------------------

/// Find every `bitcast` line whose SSA has the legacy `%ret.cast.{N}`
/// name format. Returns (function_name, ssa_name, full_line).
fn find_ret_cast_bitcasts(ir: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let mut current_fn = String::from("<unknown>");
    let mut current_is_helper = false;
    for line in ir.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("define ") {
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
        // Match `  %ret.cast.{N} = bitcast ...`
        if let Some(after_pct) = trimmed.strip_prefix('%') {
            if after_pct.starts_with("ret.cast.") {
                if let Some(eq_idx) = after_pct.find(" =") {
                    let name = format!("%{}", &after_pct[..eq_idx]);
                    let rhs = &after_pct[eq_idx + 2..].trim_start();
                    if rhs.starts_with("bitcast ") {
                        out.push((current_fn.clone(), name, trimmed.to_string()));
                    }
                }
            }
        }
    }
    out
}

#[test]
fn ret_cast_migration_uses_legacy_prefix_format() {
    // Constructs a function whose body returns a specialized Result, which
    // hits the `base_ptr_ty != expected_ptr_ty` branch at
    // stmt_visitor.rs:708 and emits a `%ret.cast.{N}` bitcast.
    let ir = gen_ir(
        r#"
        F maybe(x: i64) -> Result<i64, str> {
            R Ok(x);
        }
        F main() -> i64 {
            R 0;
        }
        "#,
    );
    let casts = find_ret_cast_bitcasts(&ir);
    assert!(
        !casts.is_empty(),
        "P1.4 iter 107 migration regression: stmt_visitor.rs:708 must \
         continue to emit `%ret.cast.{{N}}` bitcasts on the \
         base_ptr_ty != expected_ptr_ty path. None were found.\nIR:\n{}",
        ir
    );
    // Every emitted bitcast at this site must keep the legacy prefix.
    for (fn_name, name, line) in &casts {
        assert!(
            name.starts_with("%ret.cast."),
            "fn `{}` emitted a ret-cast with non-legacy SSA name `{}`. \
             The TypedEmitter migration is required to preserve the \
             `%ret.cast.{{N}}` prefix byte-for-byte (iter 106\
             /107 commit message).\nLine: {}",
            fn_name,
            name,
            line
        );
    }
}

#[test]
fn ret_cast_is_immediately_followed_by_typed_load() {
    // Locates each `%ret.cast.{N}` bitcast and asserts the next non-empty
    // IR line consumes it as the pointer operand of a `load`. If
    // `record_emitted_type` were skipped on the cast result, downstream
    // emission could pick a different SSA shape for the load and the
    // pair would no longer match.
    let ir = gen_ir(
        r#"
        F maybe(x: i64) -> Result<i64, str> {
            R Ok(x);
        }
        F main() -> i64 {
            R 0;
        }
        "#,
    );
    let casts = find_ret_cast_bitcasts(&ir);
    assert!(!casts.is_empty(), "expected at least one ret-cast site");

    for (_, cast_name, _) in &casts {
        // Find the cast line, then scan forward for the next instruction
        // line (skipping blank lines).
        let mut found_pair = false;
        let mut saw_cast = false;
        for line in ir.lines() {
            let trimmed = line.trim();
            if !saw_cast {
                if let Some(rest) = trimmed.strip_prefix(cast_name) {
                    if rest.trim_start().starts_with("=") {
                        saw_cast = true;
                    }
                }
                continue;
            }
            if trimmed.is_empty() {
                continue;
            }
            // Expected shape: `  %ret.{N} = load <ty>, <ty>* %ret.cast.{N}`
            let active = strip_debug(line);
            if active.contains("= load ") && active.ends_with(cast_name) {
                found_pair = true;
            }
            break;
        }
        assert!(
            found_pair,
            "ret-cast `{}` was not immediately consumed by a `load <ty>, \
             <ty>* {}` instruction. The TypedEmitter migration at \
             stmt_visitor.rs:708 must keep the cast → load pairing \
             intact so that the auto-registered cast type drives \
             downstream emission.\nIR:\n{}",
            cast_name, cast_name, ir
        );
    }
}

#[test]
fn ret_cast_dst_type_is_pointer_to_specialized_form() {
    // The destination type of the bitcast must be `%<specialized>*`,
    // where `<specialized>` includes the generic monomorphization marker
    // (e.g., `%Result$i64$str*`). If the `format!("{}*", llvm_ty)`
    // construction at stmt_visitor.rs migration breaks (e.g., emits a
    // bare `%Result*` again), the cast becomes a no-op and downstream
    // typed-pointer load mismatches.
    let ir = gen_ir(
        r#"
        F maybe(x: i64) -> Result<i64, str> {
            R Ok(x);
        }
        F main() -> i64 {
            R 0;
        }
        "#,
    );
    let casts = find_ret_cast_bitcasts(&ir);
    assert!(!casts.is_empty(), "expected at least one ret-cast site");

    for (fn_name, name, line) in &casts {
        // `bitcast <src_ty> <val> to <dst_ty>` — pull <dst_ty>.
        let to_pos = line.rfind(" to ").unwrap_or_else(|| {
            panic!(
                "ret-cast in `{}` is missing ` to ` separator: {}",
                fn_name, line
            )
        });
        let dst_ty = line[to_pos + 4..].trim();
        assert!(
            dst_ty.ends_with('*'),
            "ret-cast `{}` in `{}` produced a non-pointer dst type `{}`. \
             The stmt_visitor.rs:708 migration must keep `format!(\"{{}}*\", \
             llvm_ty)` to maintain the typed-pointer load contract.\n\
             Line: {}",
            name,
            fn_name,
            dst_ty,
            line
        );
        assert!(
            dst_ty.contains('$'),
            "ret-cast `{}` in `{}` produced a non-specialized dst type \
             `{}`. The bitcast at this site is meaningful only when \
             casting `%Result*` (base) → `%Result$<args>*` (specialized). \
             A non-specialized dst means either the test setup no longer \
             reaches this branch or the migration accidentally bypasses \
             specialization.\nLine: {}",
            name,
            fn_name,
            dst_ty,
            line
        );
    }
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
