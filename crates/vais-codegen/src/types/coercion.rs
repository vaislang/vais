//! Integer and float coercion utilities for LLVM IR generation

use crate::CodeGenerator;
use vais_types::ResolvedType;

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

    /// Coerce an IR value to a target numeric LLVM type.
    ///
    /// Returns the (possibly new) value name that has the target type.
    /// If the value already has the target type, returns it unchanged.
    /// Appends any needed conversion IR to `ir`.
    ///
    /// This is used to fix integer width mismatches and integer/float boundary
    /// coercions in binary ops, icmp, stores, phi values, and returns.
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

        let actual_width = Self::int_type_width(actual_ty);
        let target_width = Self::int_type_width(target_ty);

        if actual_width > 0 && (target_ty == "float" || target_ty == "double") {
            let is_float_literal =
                !val.starts_with('%') && (val.contains("e+") || val.contains("e-"));
            if is_float_literal {
                if target_ty == "float" {
                    if let Ok(d) = val.parse::<f64>() {
                        let f = d as f32;
                        let f_as_double = f as f64;
                        return format!("0x{:016X}", f_as_double.to_bits());
                    }
                }
                return val.to_string();
            }
            let tmp = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = sitofp {} {} to {}",
                tmp,
                actual_ty,
                val,
                target_ty
            );
            return tmp;
        }

        if (actual_ty == "float" || actual_ty == "double") && target_width > 0 {
            let tmp = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = fptosi {} {} to {}",
                tmp,
                actual_ty,
                val,
                target_ty
            );
            return tmp;
        }

        // Only coerce integer widths after handling int/float conversions.
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

    /// Build the explicit LLVM slice fat pointer for a `Vec<T>` argument.
    ///
    /// `&[T]` and `&mut [T]` lower to `{ i8*, i64 }`, while `Vec<T>` lowers to a
    /// named struct pointer. The type checker permits `Vec<T>` / `&Vec<T>` to
    /// flow into slice parameters, so text IR must materialize the ABI boundary:
    /// `{ data: inttoptr(vec.data), len: vec.len }`.
    pub(crate) fn coerce_vec_to_slice_fat_ptr(
        &mut self,
        val: &str,
        inferred_ty: &ResolvedType,
        expr_is_value: bool,
        counter: &mut usize,
        ir: &mut String,
    ) -> Option<String> {
        let (vec_ty, elem_ty, is_ref) = match inferred_ty {
            ResolvedType::Named { name, generics } if name == "Vec" => {
                let elem = generics.first().cloned().unwrap_or(ResolvedType::I64);
                (inferred_ty.clone(), elem, false)
            }
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => match inner.as_ref() {
                ResolvedType::Named { name, generics } if name == "Vec" => {
                    let elem = generics.first().cloned().unwrap_or(ResolvedType::I64);
                    (inner.as_ref().clone(), elem, true)
                }
                _ => return None,
            },
            _ => return None,
        };

        let vec_llvm = self.type_to_llvm(&vec_ty);
        let vec_ptr_llvm = format!("{}*", vec_llvm);
        let val_llvm = self.llvm_type_of(val);

        let vec_ptr = if is_ref || (!expr_is_value && val_llvm == vec_ptr_llvm) {
            val.to_string()
        } else if !expr_is_value && val_llvm == vec_llvm {
            // Pattern-bound Vec payloads are held as pointers, but the generic
            // temp-type registry can still record the semantic value type.
            // The value-ness check is the source of truth at this ABI boundary.
            val.to_string()
        } else if !expr_is_value && val_llvm.ends_with('*') {
            if val_llvm == vec_ptr_llvm {
                val.to_string()
            } else {
                let casted = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = bitcast {} {} to {}",
                    casted,
                    val_llvm,
                    val,
                    vec_ptr_llvm
                );
                casted
            }
        } else if val_llvm == "i64" && !expr_is_value {
            let ptr = self.next_temp(counter);
            write_ir!(ir, "  {} = inttoptr i64 {} to {}", ptr, val, vec_ptr_llvm);
            ptr
        } else {
            let alloca = self.next_temp(counter);
            self.emit_entry_alloca(&alloca, &vec_llvm);
            write_ir!(ir, "  store {} {}, {}* {}", vec_llvm, val, vec_llvm, alloca);
            alloca
        };

        let data_field = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
            data_field,
            vec_llvm,
            vec_llvm,
            vec_ptr
        );
        let data_i64 = self.next_temp(counter);
        write_ir!(ir, "  {} = load i64, i64* {}", data_i64, data_field);
        let data_ptr = self.next_temp(counter);
        write_ir!(ir, "  {} = inttoptr i64 {} to i8*", data_ptr, data_i64);

        let len_field = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = getelementptr {}, {}* {}, i32 0, i32 1",
            len_field,
            vec_llvm,
            vec_llvm,
            vec_ptr
        );
        let len_val = self.next_temp(counter);
        write_ir!(ir, "  {} = load i64, i64* {}", len_val, len_field);

        let fat0 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
            fat0,
            data_ptr
        );
        let fat1 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1",
            fat1,
            fat0,
            len_val
        );
        self.fn_ctx
            .register_temp_type(&fat1, ResolvedType::Slice(Box::new(elem_ty)));
        Some(fat1)
    }

    /// Load a `str` fat pointer from a reference/pointer argument when the callee
    /// expects the `str` ABI value (`{ i8*, i64 }`).
    pub(crate) fn coerce_ref_str_to_str_fat_ptr(
        &mut self,
        val: &str,
        inferred_ty: &ResolvedType,
        counter: &mut usize,
        ir: &mut String,
    ) -> Option<String> {
        let is_ref_str = matches!(
            inferred_ty,
            ResolvedType::Ref(inner)
                | ResolvedType::RefMut(inner)
                | ResolvedType::Pointer(inner)
                if matches!(inner.as_ref(), ResolvedType::Str)
        ) || matches!(
            self.fn_ctx.get_temp_type(val),
            Some(ResolvedType::Pointer(inner)) if matches!(inner.as_ref(), ResolvedType::Str)
        );
        if !is_ref_str {
            return None;
        }

        let str_llvm = self.type_to_llvm(&ResolvedType::Str);
        let val_llvm = self.llvm_type_of(val);
        if val_llvm == str_llvm {
            return None;
        }
        if val_llvm == format!("{}*", str_llvm) {
            let loaded = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = load {}, {}* {}",
                loaded,
                str_llvm,
                str_llvm,
                val
            );
            self.fn_ctx.register_temp_type(&loaded, ResolvedType::Str);
            Some(loaded)
        } else {
            None
        }
    }
}
