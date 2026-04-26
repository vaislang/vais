//! Integer and float coercion utilities for LLVM IR generation

use crate::CodeGenerator;

impl CodeGenerator {
    /// Look up the LLVM type string for an IR value, **only** returning a type
    /// when we have reliable evidence. Returns `None` for unregistered SSA
    /// temporaries — callers can then decide whether to skip coercion instead
    /// of trusting a default that may be wrong.
    ///
    /// Known-type evidence (resolution order, highest fidelity first):
    /// 1. `actual_llvm_type` — ground-truth recorded at emission time (Phase 17.H4
    ///    iter 26+ refactor; see compiler/docs/refactor/llvm-ground-truth.md)
    /// 2. `temp_var_types` — ResolvedType registered during `generate_expr`,
    ///    projected via `type_to_llvm` (legacy, may disagree with emission)
    /// 3. `locals` — named local variables (alloca/ssa/param)
    /// 4. Numeric literals (`42`, `-3`) — integer by construction
    /// 5. Float literals (`1.0e+00`) — detected by `.` or scientific notation
    /// 6. `null` — pointer by construction
    ///
    /// Prefer this over `llvm_type_of` when a wrong guess would produce invalid
    /// IR (e.g., spurious `inttoptr` on a value that was already a pointer).
    pub(crate) fn llvm_type_of_checked(&self, val: &str) -> Option<String> {
        // 1. Ground-truth (recorded at emission time) wins over all other tracks.
        if let Some(actual) = self.fn_ctx.get_emitted_type(val) {
            return Some(actual.to_string());
        }
        // 2. Legacy: ResolvedType → type_to_llvm projection.
        if let Some(ty) = self.fn_ctx.get_temp_type(val) {
            return Some(self.type_to_llvm(ty));
        }
        // 3. Check local variables (strip leading '%' if present for lookup)
        let local_name = val.strip_prefix('%').unwrap_or(val);
        if let Some(local) = self.fn_ctx.locals.get(local_name) {
            return Some(self.type_to_llvm(&local.ty));
        }
        // 3. Literal forms we can type by inspection
        if val == "null" {
            return None; // ambiguous pointer type — caller infers from context
        }
        if let Some(first) = val.chars().next() {
            if first.is_ascii_digit() || first == '-' {
                // Float literal? contains '.' or 'e' (but not plain minus)
                let looks_like_float = val.contains('.')
                    || (val.len() > 1 && (val.contains('e') || val.contains('E')));
                if looks_like_float {
                    return Some("double".to_string());
                }
                return Some("i64".to_string());
            }
        }
        None
    }

    /// Look up the LLVM type string for an IR value (temporary or local variable).
    ///
    /// Resolution order:
    /// 1. `temp_var_types` — temporaries registered during `generate_expr`
    /// 2. `locals` — named local variables (alloca/ssa/param)
    /// 3. Fallback — `"i64"` (legacy behaviour, to be narrowed over time)
    ///
    /// This is the primary entry point for downstream passes (store, binary op,
    /// icmp, call) that need to know the LLVM type of an operand produced by an
    /// earlier instruction.
    ///
    /// NOTE: prefer `llvm_type_of_checked` when a wrong answer would produce
    /// invalid IR — the "i64" fallback here can mislead pointer/float paths.
    pub(crate) fn llvm_type_of(&self, val: &str) -> String {
        // Special-case the literal "void" return marker — this isn't an SSA
        // value, but some callers route void-returning expressions through
        // llvm_type_of (e.g., fall-through if-expr branches). Returning "void"
        // is correct ground-truth and stops the probe from flagging it.
        if val == "void" {
            return String::from("void");
        }
        self.llvm_type_of_checked(val).unwrap_or_else(|| {
            // Wave 4a probe: report missing ground-truth registrations to stderr
            // so callers/iterations can extend coverage. Falls back to "i64" for
            // graceful degradation — does NOT panic.
            if std::env::var("VAIS_GROUND_TRUTH_PROBE").is_ok() {
                let fn_name = self.fn_ctx.current_function.as_deref().unwrap_or("?");
                eprintln!("[ground-truth-miss] fn={} val={}", fn_name, val);
            }
            String::from("i64")
        })
    }

    /// Coerce an IR value to match a function return type before `ret`.
    ///
    /// **Mini Pillar 1 (ADR 0001 §1)** — single coerce point for all `ret`
    /// emit sites. Currently invoked from `stmt_visitor::generate_return_stmt_visitor`;
    /// other ret sites (`function_gen/codegen.rs`, `string_ops.rs`, `emit.rs`,
    /// `stmt.rs`, `stmt_visitor` async/poll) will migrate incrementally as
    /// part of Pillar 1.
    ///
    /// **Invariant (ADR 0001 R1)**: After this call, the returned value's
    /// LLVM type equals `ret_llvm` byte-for-byte (or the value is unchanged
    /// when already matching).
    ///
    /// **Conversions handled** (kept narrow on purpose — only verified cases):
    /// - `iN → iM` (different widths) — sext/trunc via `coerce_int_width`
    /// - `float ↔ double` — fpext/fptrunc via `coerce_float_width`
    /// - `%Vec*` / `%Vec$T*` → `{ i8*, i64 }` — load Vec.data + Vec.len + insertvalue
    ///
    /// Returns `(coerced_value, ir_to_emit)`. Caller appends `ir_to_emit` to
    /// the function's IR buffer before emitting the `ret` instruction.
    ///
    /// **Tracker**: `vaisdb_iter74_pillar3_pillar2_landed_2026-04-26.md`
    /// **Tests**: `crates/vais-codegen/tests/ret_invariant_test.rs` (TBD)
    pub(crate) fn coerce_ret_value(
        &mut self,
        val: &str,
        val_ty: &str,
        ret_llvm: &str,
        counter: &mut usize,
    ) -> (String, String) {
        if val_ty == ret_llvm {
            return (val.to_string(), String::new());
        }
        let mut ir = String::new();

        // Case 1: integer width mismatch (iN → iM).
        if val_ty.starts_with('i')
            && ret_llvm.starts_with('i')
            && Self::int_type_width(val_ty) > 0
            && Self::int_type_width(ret_llvm) > 0
        {
            let coerced = self.coerce_int_width(val, val_ty, ret_llvm, counter, &mut ir);
            return (coerced, ir);
        }

        // Case 2: float width mismatch (float ↔ double).
        if (val_ty == "float" || val_ty == "double")
            && (ret_llvm == "float" || ret_llvm == "double")
        {
            let coerced = self.coerce_float_width(val, val_ty, ret_llvm, counter, &mut ir);
            return (coerced, ir);
        }

        // Case 3: Vec struct pointer → slice fat-ptr.
        // Pattern: function returns &[T] (slice) but body returns &self.field
        // where self.field: Vec<T>. Construct fat-ptr from Vec.data (field 0,
        // i8*) + Vec.len (field 1, i64).
        //
        // Tracker: vaisdb Task #6 (test_btree_node.ll:1736).
        // ADR 0001 §1 R3 same-class audit: ret emit sites scattered across 31+
        // locations; this coerce_ret_value is the migration target. Initial
        // adoption in stmt_visitor.rs only — others migrate incrementally.
        if val_ty.starts_with("%Vec")
            && val_ty.ends_with('*')
            && ret_llvm == "{ i8*, i64 }"
        {
            let vec_struct_ty = val_ty.trim_end_matches('*');
            let data_ptr_field = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {} {}, i32 0, i32 0",
                data_ptr_field,
                vec_struct_ty,
                val_ty,
                val
            );
            let data_i64 = self.next_temp(counter);
            write_ir!(ir, "  {} = load i64, i64* {}", data_i64, data_ptr_field);
            let data_i8ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = inttoptr i64 {} to i8*",
                data_i8ptr,
                data_i64
            );
            let len_field = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {} {}, i32 0, i32 1",
                len_field,
                vec_struct_ty,
                val_ty,
                val
            );
            let len_val = self.next_temp(counter);
            write_ir!(ir, "  {} = load i64, i64* {}", len_val, len_field);
            let fp1 = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
                fp1,
                data_i8ptr
            );
            let fp2 = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1",
                fp2,
                fp1,
                len_val
            );
            self.fn_ctx.record_emitted_type(&fp2, "{ i8*, i64 }");
            return (fp2, ir);
        }

        // Case 4: i64 void-placeholder → str fat-ptr (zeroinitializer fallback).
        // Body produced an i64 placeholder for a void/Unit expression but
        // function ret is `str` (fat pointer). Use zero fat-ptr.
        //
        // SAFETY: Same gating as Case 5 — only fire when val_ty is
        // ground-truth i64 to avoid clobbering unregistered struct loads.
        if val_ty == "i64" && ret_llvm == "{ i8*, i64 }" {
            let actual = self.llvm_type_of_checked(val);
            if actual.as_deref() == Some("i64") {
                let zinit = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = insertvalue {{ i8*, i64 }} {{ i8* null, i64 0 }}, i64 0, 1",
                    zinit
                );
                self.fn_ctx.record_emitted_type(&zinit, "{ i8*, i64 }");
                return (zinit, ir);
            }
        }

        // Case 5: i64 → %Struct (specialized generic erasure).
        // Specialized generic functions: body uses i64 (generic erasure)
        // but signature declares a concrete struct. Reinterpret-cast via
        // inttoptr + load.
        //
        // SAFETY: Only fire when val_ty is *known* to be i64 (ground-truth
        // registered). The fallback in llvm_type_of returns "i64" for
        // unregistered SSA temps, which would falsely trigger this branch
        // for values that are actually struct loads (e.g. `%ret.8 = load
        // %Vec$T, %Vec$T* %ptr`). Use llvm_type_of_checked to gate.
        if val_ty == "i64" && ret_llvm.starts_with('%') && !ret_llvm.ends_with('*') {
            // Only if ground-truth confirms i64 (not fallback)
            let actual = self.llvm_type_of_checked(val);
            if actual.as_deref() == Some("i64") {
                let tmp_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = inttoptr i64 {} to {}*",
                    tmp_ptr,
                    val,
                    ret_llvm
                );
                let loaded = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = load {}, {}* {}",
                    loaded,
                    ret_llvm,
                    ret_llvm,
                    tmp_ptr
                );
                return (loaded, ir);
            }
        }

        // No conversion known — return value as-is. Caller will emit `ret <ret_llvm> <val>`
        // which may produce a clang IR-verify error. This is the signal that a new
        // (val_ty, ret_llvm) pair needs to be added here, NOT to a new emit-site
        // if-branch (CLAUDE.md 규칙 8 — ADR 0001 forbids site-fix without R3 audit).
        (val.to_string(), ir)
    }

    /// Coerce an IR value to a target integer LLVM type, emitting sext/trunc if needed.
    ///
    /// Returns the (possibly new) value name that has the target type.
    /// If the value already has the target type, returns it unchanged.
    /// Appends any needed conversion IR to `ir`.
    ///
    /// This is used to fix integer width mismatches in binary ops, icmp, and store.
    pub(crate) fn coerce_int_width(
        &self,
        val: &str,
        actual_ty: &str,
        target_ty: &str,
        counter: &mut usize,
        ir: &mut String,
    ) -> String {
        if actual_ty == target_ty {
            return val.to_string();
        }
        // Only coerce integer types (iN)
        let actual_width = Self::int_type_width(actual_ty);
        let target_width = Self::int_type_width(target_ty);
        if actual_width == 0 || target_width == 0 {
            // Non-integer types (float, struct, ptr, etc.) — no coercion
            return val.to_string();
        }
        let tmp = self.next_temp(counter);
        if actual_width < target_width {
            // Sign-extend to wider type
            write_ir!(
                ir,
                "  {} = sext {} {} to {}",
                tmp,
                actual_ty,
                val,
                target_ty
            );
        } else {
            // Truncate to narrower type
            write_ir!(
                ir,
                "  {} = trunc {} {} to {}",
                tmp,
                actual_ty,
                val,
                target_ty
            );
        }
        tmp
    }

    /// Get the bit width of an integer LLVM type string. Returns 0 for non-integer types.
    pub(crate) fn int_type_width(ty: &str) -> u32 {
        match ty {
            "i1" => 1,
            "i8" => 8,
            "i16" => 16,
            "i32" => 32,
            "i64" => 64,
            "i128" => 128,
            _ => 0,
        }
    }

    /// Coerce an IR value to a target float LLVM type, emitting fpext/fptrunc if needed.
    ///
    /// Returns the (possibly new) value name that has the target type.
    /// If the value already has the target type, returns it unchanged.
    /// Appends any needed conversion IR to `ir`.
    ///
    /// - float → double: `fpext float %val to double`
    /// - double → float: `fptrunc double %val to float`
    ///
    /// This helper centralises float-width coercion that was previously
    /// duplicated across expr_helpers.rs and generate_expr_call.rs.
    #[allow(dead_code)]
    pub(crate) fn coerce_float_width(
        &self,
        val: &str,
        actual_ty: &str,
        target_ty: &str,
        counter: &mut usize,
        ir: &mut String,
    ) -> String {
        if actual_ty == target_ty {
            return val.to_string();
        }
        // Handle float<->double and int→float coercions.
        match (actual_ty, target_ty) {
            ("float", "double") => {
                let tmp = self.next_temp(counter);
                write_ir!(ir, "  {} = fpext float {} to double", tmp, val);
                tmp
            }
            ("double", "float") => {
                let tmp = self.next_temp(counter);
                write_ir!(ir, "  {} = fptrunc double {} to float", tmp, val);
                tmp
            }
            (src, dst) if (dst == "float" || dst == "double") && src.starts_with('i') => {
                // Integer value passed where a float param is expected. This
                // happens for bare integer literals (e.g. `2` passed into a
                // `f32` Vec) where codegen preserves the i-type. sitofp both
                // SSA temps and literal constants — LLVM accepts the same
                // syntax for either.
                let tmp = self.next_temp(counter);
                write_ir!(ir, "  {} = sitofp {} {} to {}", tmp, src, val, dst);
                tmp
            }
            _ => {
                // Not a float coercion — return unchanged
                val.to_string()
            }
        }
    }
}
