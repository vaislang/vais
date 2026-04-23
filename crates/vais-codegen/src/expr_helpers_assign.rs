//! Assignment and identifier expression helpers for CodeGenerator.
//!
//! Contains generate_assign_expr, generate_ident_expr, and generate_assign_op_expr.
//! Core binary/unary/cast helpers are in expr_helpers.

use crate::{format_did_you_mean, suggest_similar, CodeGenerator, CodegenError, CodegenResult};
use vais_ast::{BinOp, Expr, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Generate assign expression
    #[inline(never)]
    pub(crate) fn generate_assign_expr(
        &mut self,
        target: &Spanned<Expr>,
        value: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Phase 6.27b iteration 34: codegen-side mirror of TC's Never-promotion on
        // Expr::Assign. If the target local's type contains Never (from `mut None`
        // init) and the RHS has a concrete wrapper type, update the local's
        // recorded type so later field access through `Some(x) => x.field` sees
        // the promoted type rather than Optional(Named{Option, generics=[]}).
        if let Expr::Ident(name) = &target.node {
            let has_never = self
                .fn_ctx
                .locals
                .get(name)
                .map(|l| type_contains_never(&l.ty) || generics_all_empty_enum(&l.ty))
                .unwrap_or(false);
            if has_never {
                let value_ty = self.infer_expr_type(value);
                if !type_contains_never(&value_ty) && !generics_all_empty_enum(&value_ty) {
                    if let Some(local) = self.fn_ctx.locals.get_mut(name) {
                        local.ty = value_ty;
                    }
                }
            }
        }

        let (val, val_ir) = self.generate_expr(value, counter)?;
        let mut ir = val_ir;

        if let Expr::Ident(name) = &target.node {
            if let Some(local) = self.fn_ctx.locals.get(name).cloned() {
                if !local.is_param() {
                    if local.is_ssa() {
                        // SSA variable being reassigned: convert to alloca to support loops
                        // Without this, loop bodies would use stale SSA values
                        let local_ty = local.ty.clone();
                        let llvm_ty = self.type_to_llvm(&local_ty);
                        let old_ssa_val = local.llvm_name.clone();
                        let alloca_name = format!("{}.{}", name, counter);
                        *counter += 1;
                        // Determine if the old SSA value can be safely referenced
                        // from the entry block. Only literal immediate values
                        // (numbers) of scalar integer/float types are guaranteed
                        // safe — register references (%tN) may not dominate
                        // entry, and aggregate types cannot use a numeric literal.
                        let is_scalar_ty = llvm_ty.starts_with('i')
                            || llvm_ty == "double"
                            || llvm_ty == "float";
                        let is_immediate = old_ssa_val
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_digit() || c == '-');
                        let can_init_in_entry = is_scalar_ty && is_immediate;
                        if can_init_in_entry {
                            // Emit alloca + initial store (with original value) in entry.
                            // This guarantees the alloca is initialized on all paths,
                            // including paths that bypass the reassignment branch.
                            self.fn_ctx.entry_allocas.push(format!(
                                "  %{} = alloca {}\n  store {} {}, {}* %{}",
                                alloca_name,
                                llvm_ty,
                                llvm_ty,
                                old_ssa_val,
                                llvm_ty,
                                alloca_name
                            ));
                        } else {
                            // Fallback: alloca only; the reassignment store covers the
                            // reachable paths (legacy behavior). Non-immediate SSA values
                            // cannot safely be stored in the entry block because their
                            // definitions may not dominate it.
                            self.emit_entry_alloca(&format!("%{}", alloca_name), &llvm_ty);
                        }
                        // Now store the new (reassignment) value
                        let actual_val_ty = self.llvm_type_of(&val);
                        let coerced_val =
                            self.coerce_int_width(&val, &actual_val_ty, &llvm_ty, counter, &mut ir);
                        write_ir!(
                            ir,
                            "  store {} {}, {}* %{}",
                            llvm_ty,
                            coerced_val,
                            llvm_ty,
                            alloca_name
                        );
                        // Convert to alloca-based local
                        self.fn_ctx
                            .locals
                            .insert(name.clone(), crate::LocalVar::alloca(local_ty, alloca_name));
                    } else {
                        let llvm_ty = self.type_to_llvm(&local.ty);
                        // Coerce value width to match local variable type
                        let actual_val_ty = self.llvm_type_of(&val);
                        // Phase 17.H4.8b: if the local's type is Named (struct/enum)
                        // and the rhs `val` is actually a pointer to that struct
                        // (e.g., Some(x) → alloca %Option + stores, yielding %tN
                        // of type %Option*), load before storing so we store a
                        // value, not a pointer-as-value.
                        let needs_load = matches!(local.ty, ResolvedType::Named { .. })
                            && val.starts_with('%')
                            && actual_val_ty == format!("{}*", llvm_ty);
                        let coerced_val = if needs_load {
                            let loaded = self.next_temp(counter);
                            write_ir!(ir, "  {} = load {}, {}* {}", loaded, llvm_ty, llvm_ty, val);
                            loaded
                        } else {
                            self.coerce_int_width(&val, &actual_val_ty, &llvm_ty, counter, &mut ir)
                        };
                        // Store the value into the alloca.
                        write_ir!(
                            ir,
                            "  store {} {}, {}* %{}",
                            llvm_ty,
                            coerced_val,
                            llvm_ty,
                            local.llvm_name
                        );
                    }
                }
            } else if let Some(global_ty) = self.types.globals.get(name).map(|g| g._ty.clone()) {
                // Global variable assignment: store to @name
                let llvm_ty = self.type_to_llvm(&global_ty);
                write_ir!(ir, "  store {} {}, {}* @{}", llvm_ty, val, llvm_ty, name);
            }
        } else if let Expr::Field {
            expr: obj_expr,
            field,
        } = &target.node
        {
            let (obj_val, obj_ir) = self.generate_expr(obj_expr, counter)?;
            ir.push_str(&obj_ir);

            // Use infer_expr_type to support both simple (obj.field) and nested field assignment
            let obj_type = self.infer_expr_type(obj_expr);
            let resolved_type = match &obj_type {
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref().clone(),
                other => other.clone(),
            };

            if let ResolvedType::Named {
                name: orig_name, ..
            } = &resolved_type
            {
                let struct_name = self.resolve_struct_name(orig_name);
                if let Some(struct_info) = self.types.structs.get(&struct_name).cloned() {
                    if let Some(field_idx) = struct_info
                        .fields
                        .iter()
                        .position(|(n, _)| n == &field.node)
                    {
                        let field_ty = &struct_info.fields[field_idx].1;
                        let llvm_ty = self.type_to_llvm(field_ty);

                        let field_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}",
                            field_ptr,
                            struct_name,
                            struct_name,
                            obj_val,
                            field_idx
                        );
                        // Coerce value to match field type
                        let actual_val_ty = self.llvm_type_of(&val);
                        let coerced_val = if matches!(field_ty, ResolvedType::Named { .. })
                            && val.starts_with('%')
                        {
                            // Named field type: the value may be a pointer to the struct
                            // (e.g., SSA param spill %__severity_ptr). Find the local by
                            // matching llvm_name since the val is the LLVM name, not source name.
                            let is_ssa_named_ptr = self.fn_ctx.locals.values().any(|local| {
                                local.llvm_name == val
                                    && local.is_ssa()
                                    && matches!(local.ty, ResolvedType::Named { .. })
                            });
                            // Phase 17.H4.8: also load when the rhs is an
                            // alloca'd struct literal. A struct literal
                            // emits `%tN = alloca %T; store fields @ %tN`,
                            // so `val = %tN` is a pointer, not a value.
                            // Registered temp type tells us whether this
                            // SSA name is actually the named struct type.
                            let is_alloca_named_ptr =
                                self.fn_ctx
                                    .get_temp_type(&val)
                                    .is_some_and(|t| matches!(t, ResolvedType::Named { .. }))
                                    || actual_val_ty == format!("{}*", llvm_ty);
                            if is_ssa_named_ptr || is_alloca_named_ptr {
                                let loaded = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = load {}, {}* {}",
                                    loaded,
                                    llvm_ty,
                                    llvm_ty,
                                    val
                                );
                                loaded
                            } else {
                                self.coerce_int_width(
                                    &val,
                                    &actual_val_ty,
                                    &llvm_ty,
                                    counter,
                                    &mut ir,
                                )
                            }
                        } else {
                            self.coerce_int_width(&val, &actual_val_ty, &llvm_ty, counter, &mut ir)
                        };
                        write_ir!(
                            ir,
                            "  store {} {}, {}* {}",
                            llvm_ty,
                            coerced_val,
                            llvm_ty,
                            field_ptr
                        );
                    }
                }
            }
        } else if let Expr::Index {
            expr: arr_expr,
            index,
        } = &target.node
        {
            // Array/slice index assignment: arr[i] = value
            let (arr_val, arr_ir) = self.generate_expr(arr_expr, counter)?;
            let (idx_val, idx_ir) = self.generate_expr(index, counter)?;
            ir.push_str(&arr_ir);
            ir.push_str(&idx_ir);

            // Infer element type for correct GEP + store
            let arr_ty = self.infer_expr_type(arr_expr);
            // Strip Ref/RefMut — Vec indexed assign on a &Vec receiver still
            // uses the underlying Vec element type.
            let arr_ty_inner = match &arr_ty {
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref(),
                other => other,
            };
            enum AccessKind {
                /// Pointer math on a raw `T*` (array / pointer / typed slice
                /// with fat-pointer extraction).
                Direct,
                /// Fat-pointer slice `{ i8*, i64 }` — extract data, bitcast.
                FatSlice,
                /// Vec<T> — load data pointer from the first field of the
                /// Vec struct, then GEP/store through that pointer.
                VecData,
            }
            let (elem_llvm_ty, access) = match arr_ty_inner {
                ResolvedType::Pointer(elem) => (self.type_to_llvm(elem), AccessKind::Direct),
                ResolvedType::Array(elem) => (self.type_to_llvm(elem), AccessKind::Direct),
                ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => {
                    (self.type_to_llvm(elem), AccessKind::FatSlice)
                }
                ResolvedType::Named { name, generics }
                    if name == "Vec" && !generics.is_empty() =>
                {
                    (self.type_to_llvm(&generics[0]), AccessKind::VecData)
                }
                _ => ("i64".to_string(), AccessKind::Direct),
            };

            // Build the pointer we will GEP through.
            let base_ptr = match access {
                AccessKind::FatSlice => {
                    let data_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                        data_ptr,
                        arr_val
                    );
                    let typed_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = bitcast i8* {} to {}*",
                        typed_ptr,
                        data_ptr,
                        elem_llvm_ty
                    );
                    typed_ptr
                }
                AccessKind::VecData => {
                    // Vec layout is `{ i64, i64, i64, i64, i64 }` where field 0
                    // is the data pointer stored as an i64. Load it, then
                    // inttoptr to a typed T* for the GEP/store.
                    let vec_llvm_ty = self.type_to_llvm(arr_ty_inner);
                    let data_slot = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                        data_slot,
                        vec_llvm_ty,
                        vec_llvm_ty,
                        arr_val
                    );
                    let data_i64 = self.next_temp(counter);
                    write_ir!(ir, "  {} = load i64, i64* {}", data_i64, data_slot);
                    let typed_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = inttoptr i64 {} to {}*",
                        typed_ptr,
                        data_i64,
                        elem_llvm_ty
                    );
                    typed_ptr
                }
                AccessKind::Direct => arr_val.clone(),
            };

            let elem_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {}* {}, i64 {}",
                elem_ptr,
                elem_llvm_ty,
                elem_llvm_ty,
                base_ptr,
                idx_val
            );
            // Coerce value to match element type (e.g., i8 from trunc → i64 for Vec store)
            let val_ty = self.llvm_type_of(&val);
            let store_val = self.coerce_int_width(&val, &val_ty, &elem_llvm_ty, counter, &mut ir);
            write_ir!(
                ir,
                "  store {} {}, {}* {}",
                elem_llvm_ty,
                store_val,
                elem_llvm_ty,
                elem_ptr
            );
        }

        Ok((val, ir))
    }

    /// Generate identifier expression
    #[inline(never)]
    pub(crate) fn generate_ident_expr(
        &mut self,
        name: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        if let Some(local) = self.fn_ctx.locals.get(name).cloned() {
            if local.is_param() {
                // Parameters are SSA values, use directly
                Ok((format!("%{}", local.llvm_name), String::new()))
            } else if local.is_ssa() {
                // SSA variables: use the stored value directly, no load needed
                Ok((local.llvm_name.clone(), String::new()))
            } else if matches!(local.ty, ResolvedType::Named { .. }) {
                // Single-pointer layout: struct alloca locals are %Type* (%var = alloca %Type).
                // Return the alloca pointer directly — it IS the struct address.
                Ok((format!("%{}", local.llvm_name), String::new()))
            } else {
                // Local variables need to be loaded from alloca
                let tmp = self.next_temp(counter);
                let llvm_ty = self.type_to_llvm(&local.ty);
                let ir = format!(
                    "  {} = load {}, {}* %{}\n",
                    tmp, llvm_ty, llvm_ty, local.llvm_name
                );
                // Phase E.6: register the load result's type so downstream
                // passes (coerce, phi, cast) see the correct width instead
                // of falling back to i64. This is the single biggest source
                // of "i64 vs <N-bit>" mismatches across vaisdb tests.
                self.fn_ctx.register_temp_type(&tmp, local.ty.clone());
                Ok((tmp, ir))
            }
        } else if name == "self" {
            // Handle self reference
            Ok(("%self".to_string(), String::new()))
        } else if self.is_unit_enum_variant(name) {
            // Unit enum variant (e.g., None)
            // Create enum value on stack with just the tag
            // Clone enum info to avoid borrow conflict with self.next_temp/emit_entry_alloca
            let mut found = None;
            for enum_info in self.types.enums.values() {
                for (tag, variant) in enum_info.variants.iter().enumerate() {
                    if variant.name == name {
                        found = Some((enum_info.name.clone(), tag));
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            if let Some((enum_name, tag)) = found {
                let mut ir = String::new();
                let enum_ptr = self.next_temp(counter);
                self.emit_entry_alloca(&enum_ptr, &format!("%{}", enum_name));
                // Store tag
                let tag_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0",
                    tag_ptr,
                    enum_name,
                    enum_name,
                    enum_ptr
                );
                write_ir!(ir, "  store i32 {}, i32* {}", tag, tag_ptr);
                return Ok((enum_ptr, ir));
            }
            // Fallback if not found (shouldn't happen)
            Ok((format!("@{}", name), String::new()))
        } else if let Some(const_info) = self.types.constants.get(name).cloned() {
            // Constant reference - inline the constant value
            self.generate_expr(&const_info.value, counter)
        } else if let Some(global_info) = self.types.globals.get(name).cloned() {
            // Global variable access - load from @name
            let llvm_ty = self.type_to_llvm(&global_info._ty);
            let tmp = self.next_temp(counter);
            let ir = format!("  {} = load {}, {}* @{}\n", tmp, llvm_ty, llvm_ty, name);
            // Phase E.6: register load result type (see alloca-local comment).
            self.fn_ctx.register_temp_type(&tmp, global_info._ty.clone());
            Ok((tmp, ir))
        } else if let Some(fn_info) = self.types.functions.get(name).cloned() {
            // Function reference used as a value — convert function pointer to i64
            let ret_ty = self.type_to_llvm(&fn_info.signature.ret);
            let param_types: Vec<String> = fn_info
                .signature
                .params
                .iter()
                .map(|(_, ty, _)| self.type_to_llvm(ty))
                .collect();
            let fn_ptr_ty = format!("{} ({})*", ret_ty, param_types.join(", "));
            let tmp = self.next_temp(counter);
            let ir = format!("  {} = ptrtoint {} @{} to i64\n", tmp, fn_ptr_ty, name);
            Ok((tmp, ir))
        } else if let Some(self_local) = self.fn_ctx.locals.get("self").cloned() {
            // Implicit self: check if name is a field of the self struct
            let self_type = match &self_local.ty {
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref().clone(),
                other => other.clone(),
            };
            if let ResolvedType::Named {
                name: type_name, ..
            } = &self_type
            {
                let resolved_name = self.resolve_struct_name(type_name);
                if let Some(struct_info) = self.types.structs.get(&resolved_name).cloned() {
                    if let Some(field_idx) = struct_info.fields.iter().position(|(n, _)| n == name)
                    {
                        let field_ty = &struct_info.fields[field_idx].1;
                        let llvm_ty = self.type_to_llvm(field_ty);
                        let mut ir = String::new();
                        let field_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr %{}, %{}* %self, i32 0, i32 {}",
                            field_ptr,
                            resolved_name,
                            resolved_name,
                            field_idx
                        );
                        if matches!(field_ty, ResolvedType::Named { .. }) {
                            return Ok((field_ptr, ir));
                        } else {
                            let result = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = load {}, {}* {}",
                                result,
                                llvm_ty,
                                llvm_ty,
                                field_ptr
                            );
                            return Ok((result, ir));
                        }
                    }
                }
            }
            // Not a field — check if name is a known struct (used as a namespace for static calls)
            let resolved = self.resolve_struct_name(name);
            if self.types.structs.contains_key(&resolved)
                || self.generics.generated_structs.contains_key(&resolved)
            {
                // Struct name used as a namespace identifier (e.g., `StringMap.with_capacity()`).
                // Return a zero sentinel; the actual call will be handled by generate_expr_call.
                return Ok(("0".to_string(), String::new()));
            }
            // Generic type parameter (e.g., K, V, T, E) leaking as an identifier during
            // monomorphization — return a zero sentinel. This can happen when a generic method
            // body references a type parameter in a position that survives as an Expr::Ident
            // (e.g., synthesized default values for generic return types).
            if self.generics.substitutions.contains_key(name) {
                return Ok(("0".to_string(), String::new()));
            }
            let mut candidates: Vec<&str> = Vec::new();
            for var_name in self.fn_ctx.locals.keys() {
                candidates.push(var_name.as_str());
            }
            for func_name in self.types.functions.keys() {
                candidates.push(func_name.as_str());
            }
            let suggestions = suggest_similar(name, &candidates, 3);
            let suggestion_text = format_did_you_mean(&suggestions);
            Err(CodegenError::UndefinedVar(format!(
                "{}{}",
                name, suggestion_text
            )))
        } else {
            // Check if name is a known struct (used as a namespace for static method calls,
            // e.g., `StringMap.with_capacity(16)` in cross-module codegen).
            let resolved = self.resolve_struct_name(name);
            if self.types.structs.contains_key(&resolved)
                || self.generics.generated_structs.contains_key(&resolved)
            {
                // Struct name used as a namespace identifier — return zero sentinel.
                return Ok(("0".to_string(), String::new()));
            }
            // Generic type parameter (e.g., K, V, T, E) leaking as an identifier during
            // monomorphization — return a zero sentinel (same rationale as the branch above).
            if self.generics.substitutions.contains_key(name) {
                return Ok(("0".to_string(), String::new()));
            }

            // Undefined identifier - provide suggestions
            let mut candidates: Vec<&str> = Vec::new();

            // Add local variables
            for var_name in self.fn_ctx.locals.keys() {
                candidates.push(var_name.as_str());
            }

            // Add function names
            for func_name in self.types.functions.keys() {
                candidates.push(func_name.as_str());
            }

            // Add "self" if we're in a method context
            if self.fn_ctx.current_function.is_some() {
                candidates.push("self");
            }

            // Get suggestions
            let suggestions = suggest_similar(name, &candidates, 3);
            let suggestion_text = format_did_you_mean(&suggestions);
            Err(CodegenError::UndefinedVar(format!(
                "{}{}",
                name, suggestion_text
            )))
        }
    }

    /// Generate compound assignment expression
    #[inline(never)]
    pub(crate) fn generate_assign_op_expr(
        &mut self,
        op: &BinOp,
        target: &Spanned<Expr>,
        value: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (current_val, load_ir) = self.generate_expr(target, counter)?;
        let (rhs_val, rhs_ir) = self.generate_expr(value, counter)?;

        let mut ir = load_ir;
        ir.push_str(&rhs_ir);

        let op_str = match op {
            BinOp::Add => "add",
            BinOp::Sub => "sub",
            BinOp::Mul => "mul",
            BinOp::Div => "sdiv",
            BinOp::Mod => "srem",
            BinOp::BitAnd => "and",
            BinOp::BitOr => "or",
            BinOp::BitXor => "xor",
            BinOp::Shl => "shl",
            BinOp::Shr => "ashr",
            _ => return Err(CodegenError::Unsupported(format!("compound {:?}", op))),
        };

        // Determine the target's native integer width so the arithmetic is
        // performed in that width (e.g. `size: u32` += 8 uses `add i32`).
        let target_ty = self.infer_expr_type(target);
        let target_bits = self.get_integer_bits(&target_ty);
        let op_ty = if target_bits > 0 {
            format!("i{}", target_bits)
        } else {
            "i64".to_string()
        };

        // Coerce both operands to the target integer width.
        let coerce = |val: &str, bits: u32, cg: &mut Self, ir: &mut String, counter: &mut usize| -> String {
            if bits == target_bits || target_bits == 0 {
                val.to_string()
            } else if bits > target_bits {
                let t = cg.next_temp(counter);
                write_ir!(ir, "  {} = trunc i{} {} to i{}", t, bits, val, target_bits);
                t
            } else {
                let t = cg.next_temp(counter);
                write_ir!(ir, "  {} = sext i{} {} to i{}", t, bits, val, target_bits);
                t
            }
        };
        let current_bits = self.llvm_type_of(&current_val)
            .strip_prefix('i')
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(64);
        let rhs_bits = self.llvm_type_of(&rhs_val)
            .strip_prefix('i')
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(64);
        let current_coerced = coerce(&current_val, current_bits, self, &mut ir, counter);
        let rhs_coerced = coerce(&rhs_val, rhs_bits, self, &mut ir, counter);

        let result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = {} {} {}, {}",
            result,
            op_str,
            op_ty,
            current_coerced,
            rhs_coerced
        );

        if let Expr::Ident(name) = &target.node {
            if let Some(local) = self.fn_ctx.locals.get(name.as_str()).cloned() {
                if !local.is_param() {
                    let llvm_ty = self.type_to_llvm(&local.ty);
                    write_ir!(
                        ir,
                        "  store {} {}, {}* %{}",
                        llvm_ty,
                        result,
                        llvm_ty,
                        local.llvm_name
                    );
                }
            }
        } else if let Expr::Field {
            expr: obj_expr,
            field,
        } = &target.node
        {
            // Field compound assignment: self.field += value
            // Need to store the result back to the field
            let (obj_val, obj_ir) = self.generate_expr(obj_expr, counter)?;
            ir.push_str(&obj_ir);

            let obj_type = self.infer_expr_type(obj_expr);
            let resolved = match &obj_type {
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref().clone(),
                other => other.clone(),
            };

            if let ResolvedType::Named { name, .. } = &resolved {
                let type_name = self.resolve_struct_name(name);
                if let Some(struct_info) = self.types.structs.get(&type_name).cloned() {
                    if let Some(field_idx) = struct_info
                        .fields
                        .iter()
                        .position(|(n, _)| n == &field.node)
                    {
                        let field_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}",
                            field_ptr,
                            type_name,
                            type_name,
                            obj_val,
                            field_idx
                        );
                        let field_ty = &struct_info.fields[field_idx].1;
                        let llvm_ty = self.type_to_llvm(field_ty);
                        write_ir!(
                            ir,
                            "  store {} {}, {}* {}",
                            llvm_ty,
                            result,
                            llvm_ty,
                            field_ptr
                        );
                    }
                }
            }
        } else if let Expr::Index {
            expr: arr_expr,
            index: idx_expr,
        } = &target.node
        {
            // Array/Vec element compound assignment: arr[idx] += value
            let (arr_val, arr_ir) = self.generate_expr(arr_expr, counter)?;
            let (idx_val, idx_ir) = self.generate_expr(idx_expr, counter)?;
            ir.push_str(&arr_ir);
            ir.push_str(&idx_ir);
            // Use inferred element type instead of hardcoded i64
            let arr_type = self.infer_expr_type(arr_expr);
            let elem_llvm = match &arr_type {
                ResolvedType::Array(inner) | ResolvedType::Pointer(inner) => {
                    self.type_to_llvm(inner)
                }
                _ => self.llvm_type_of(&arr_val),
            };
            let elem_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {}* {}, i64 {}",
                elem_ptr,
                elem_llvm,
                elem_llvm,
                arr_val,
                idx_val
            );
            write_ir!(
                ir,
                "  store {} {}, {}* {}",
                elem_llvm,
                result,
                elem_llvm,
                elem_ptr
            );
        }

        Ok((result, ir))
    }
}

/// Check whether a ResolvedType contains ResolvedType::Never anywhere.
/// Mirror of vais-types checker_expr/special.rs `type_contains_never`.
fn type_contains_never(ty: &ResolvedType) -> bool {
    match ty {
        ResolvedType::Never => true,
        ResolvedType::Array(inner)
        | ResolvedType::Optional(inner)
        | ResolvedType::Pointer(inner)
        | ResolvedType::Ref(inner)
        | ResolvedType::RefMut(inner)
        | ResolvedType::Slice(inner)
        | ResolvedType::SliceMut(inner)
        | ResolvedType::Range(inner)
        | ResolvedType::Future(inner) => type_contains_never(inner),
        ResolvedType::ConstArray { element, .. } => type_contains_never(element),
        ResolvedType::Map(k, v) => type_contains_never(k) || type_contains_never(v),
        ResolvedType::Result(ok, err) => type_contains_never(ok) || type_contains_never(err),
        ResolvedType::Tuple(elems) => elems.iter().any(type_contains_never),
        ResolvedType::Named { generics, .. } => generics.iter().any(type_contains_never),
        ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
            params.iter().any(type_contains_never) || type_contains_never(ret)
        }
        _ => false,
    }
}

/// Detect the `Named { name: "Option"|"Result", generics: [] }` pattern that
/// codegen-local `infer_expr_type` emits for bare `None` / `Err(...)` idents.
/// This is functionally equivalent to `Optional(Never)` but erased one level,
/// so we must promote its scope binding when a concrete wrapper arrives.
fn generics_all_empty_enum(ty: &ResolvedType) -> bool {
    if let ResolvedType::Named { name, generics } = ty {
        if generics.is_empty() && matches!(name.as_str(), "Option" | "Result") {
            return true;
        }
    }
    false
}
