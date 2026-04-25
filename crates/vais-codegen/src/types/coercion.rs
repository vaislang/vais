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
