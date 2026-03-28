//! Integer and float coercion utilities for LLVM IR generation

use crate::CodeGenerator;

impl CodeGenerator {
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
    pub(crate) fn llvm_type_of(&self, val: &str) -> String {
        // 1. Check temporaries first (most common for generate_expr results)
        if let Some(ty) = self.fn_ctx.get_temp_type(val) {
            return self.type_to_llvm(ty);
        }
        // 2. Check local variables (strip leading '%' if present for lookup)
        let local_name = val.strip_prefix('%').unwrap_or(val);
        if let Some(local) = self.fn_ctx.locals.get(local_name) {
            return self.type_to_llvm(&local.ty);
        }
        // 3. Legacy fallback — assume i64 (generic erasure default)
        String::from("i64")
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
            write_ir!(ir, "  {} = sext {} {} to {}", tmp, actual_ty, val, target_ty);
        } else {
            // Truncate to narrower type
            write_ir!(ir, "  {} = trunc {} {} to {}", tmp, actual_ty, val, target_ty);
        }
        tmp
    }

    /// Get the bit width of an integer LLVM type string. Returns 0 for non-integer types.
    fn int_type_width(ty: &str) -> u32 {
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
        // Only handle float<->double coercions
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
            _ => {
                // Not a float coercion — return unchanged
                val.to_string()
            }
        }
    }
}
