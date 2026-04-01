//! Expression generation helper methods for CodeGenerator
//!
//! This module contains core expression helpers: enum variants,
//! binary/unary operations, casts, and assignment operations.

use crate::{format_did_you_mean, suggest_similar, CodeGenerator, CodegenError, CodegenResult};
use vais_ast::{BinOp, Expr, Span, Spanned, Type, UnaryOp};
use vais_types::ResolvedType;

impl CodeGenerator {
    #[inline(never)]
    pub(crate) fn generate_unit_enum_variant(
        &mut self,
        name: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
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
    }

    /// Generate binary expression
    #[inline(never)]
    pub(crate) fn generate_binary_expr(
        &mut self,
        op: &BinOp,
        left: &Spanned<Expr>,
        right: &Spanned<Expr>,
        counter: &mut usize,
        span: Span,
    ) -> CodegenResult<(String, String)> {
        let (left_val, left_ir) = self.generate_expr(left, counter)?;
        let (right_val, right_ir) = self.generate_expr(right, counter)?;

        let mut ir = left_ir;
        ir.push_str(&right_ir);

        // Handle string operations
        let left_type = self.infer_expr_type(left);
        if matches!(left_type, ResolvedType::Str) {
            return self.generate_string_binary_op(op, &left_val, &right_val, ir, counter);
        }

        // Handle comparison and logical operations (result is i1)
        let is_comparison = matches!(
            op,
            BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte | BinOp::Eq | BinOp::Neq
        );
        let is_logical = matches!(op, BinOp::And | BinOp::Or);

        if is_logical {
            // For logical And/Or, convert operands to i1 first, then perform operation
            // If operand is already i1 (bool), skip the conversion
            let left_bool = if matches!(left_type, ResolvedType::Bool) {
                left_val.clone()
            } else {
                let tmp = self.next_temp(counter);
                let left_llvm = self.type_to_llvm(&left_type);
                write_ir!(ir, "  {} = icmp ne {} {}, 0", tmp, left_llvm, left_val);
                tmp
            };
            let right_type = self.infer_expr_type(right);
            let right_bool = if matches!(right_type, ResolvedType::Bool) {
                right_val.clone()
            } else {
                let tmp = self.next_temp(counter);
                let right_llvm = self.type_to_llvm(&right_type);
                write_ir!(ir, "  {} = icmp ne {} {}, 0", tmp, right_llvm, right_val);
                tmp
            };

            let op_str = match op {
                BinOp::And => "and",
                BinOp::Or => "or",
                _ => {
                    return Err(CodegenError::InternalError(format!(
                        "BinOp {:?} in logical codegen path",
                        op
                    )))
                }
            };

            let result_bool = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            write_ir!(
                ir,
                "  {} = {} i1 {}, {}{}",
                result_bool,
                op_str,
                left_bool,
                right_bool,
                dbg_info
            );

            // Extend back to i64 for consistency
            let result = self.next_temp(counter);
            write_ir!(ir, "  {} = zext i1 {} to i64", result, result_bool);
            Ok((result, ir))
        } else if is_comparison {
            // Comparison returns i1, extend to i64
            let right_type = self.infer_expr_type(right);
            let is_float_cmp = matches!(left_type, ResolvedType::F64 | ResolvedType::F32)
                || matches!(right_type, ResolvedType::F64 | ResolvedType::F32);

            let cmp_tmp = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);

            if is_float_cmp {
                let op_str = match op {
                    BinOp::Lt => "fcmp olt",
                    BinOp::Lte => "fcmp ole",
                    BinOp::Gt => "fcmp ogt",
                    BinOp::Gte => "fcmp oge",
                    BinOp::Eq => "fcmp oeq",
                    BinOp::Neq => "fcmp one",
                    _ => {
                        return Err(CodegenError::InternalError(format!(
                            "BinOp {:?} in float_cmp codegen path",
                            op
                        )))
                    }
                };
                // Handle mixed f32/f64 comparisons
                let left_is_f32 = matches!(left_type, ResolvedType::F32);
                let right_is_f32 = matches!(right_type, ResolvedType::F32);
                let left_is_f64 = matches!(left_type, ResolvedType::F64);
                let right_is_f64 = matches!(right_type, ResolvedType::F64);

                let float_llvm;
                let mut actual_left = left_val.clone();
                let mut actual_right = right_val.clone();

                if (left_is_f32 && right_is_f64) || (left_is_f64 && right_is_f32) {
                    float_llvm = "double";
                    if left_is_f32 {
                        let ext = self.next_temp(counter);
                        write_ir!(ir, "  {} = fpext float {} to double", ext, left_val);
                        actual_left = ext;
                    }
                    if right_is_f32 {
                        let ext = self.next_temp(counter);
                        write_ir!(ir, "  {} = fpext float {} to double", ext, right_val);
                        actual_right = ext;
                    }
                } else if left_is_f32 || right_is_f32 {
                    float_llvm = "float";
                } else {
                    float_llvm = "double";
                }

                write_ir!(
                    ir,
                    "  {} = {} {} {}, {}{}",
                    cmp_tmp,
                    op_str,
                    float_llvm,
                    actual_left,
                    actual_right,
                    dbg_info
                );
            } else {
                let op_str = match op {
                    BinOp::Lt => "icmp slt",
                    BinOp::Lte => "icmp sle",
                    BinOp::Gt => "icmp sgt",
                    BinOp::Gte => "icmp sge",
                    BinOp::Eq => "icmp eq",
                    BinOp::Neq => "icmp ne",
                    _ => {
                        return Err(CodegenError::InternalError(format!(
                            "BinOp {:?} in int_cmp codegen path",
                            op
                        )))
                    }
                };
                // Use inferred type for integer comparison width
                let cmp_llvm = match &left_type {
                    ResolvedType::I8 | ResolvedType::U8 => "i8",
                    ResolvedType::I16 | ResolvedType::U16 => "i16",
                    ResolvedType::I32 | ResolvedType::U32 => "i32",
                    ResolvedType::I128 | ResolvedType::U128 => "i128",
                    ResolvedType::Bool => "i1",
                    _ => "i64",
                };
                // Coerce operands to the comparison width if they differ
                let actual_left_ty = self.llvm_type_of(&left_val);
                let actual_right_ty = self.llvm_type_of(&right_val);
                let coerced_left =
                    self.coerce_int_width(&left_val, &actual_left_ty, cmp_llvm, counter, &mut ir);
                let coerced_right =
                    self.coerce_int_width(&right_val, &actual_right_ty, cmp_llvm, counter, &mut ir);
                write_ir!(
                    ir,
                    "  {} = {} {} {}, {}{}",
                    cmp_tmp,
                    op_str,
                    cmp_llvm,
                    coerced_left,
                    coerced_right,
                    dbg_info
                );
            }

            // Extend i1 to i64
            let result = self.next_temp(counter);
            write_ir!(ir, "  {} = zext i1 {} to i64", result, cmp_tmp);
            Ok((result, ir))
        } else {
            // Arithmetic and bitwise operations
            let tmp = self.next_temp(counter);

            // Check if either operand is a float type
            let right_type = self.infer_expr_type(right);
            let is_float = matches!(left_type, ResolvedType::F64 | ResolvedType::F32)
                || matches!(right_type, ResolvedType::F64 | ResolvedType::F32);

            let dbg_info = self.debug_info.dbg_ref_from_span(span);

            if is_float
                && matches!(
                    op,
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod
                )
            {
                let op_str = match op {
                    BinOp::Add => "fadd",
                    BinOp::Sub => "fsub",
                    BinOp::Mul => "fmul",
                    BinOp::Div => "fdiv",
                    BinOp::Mod => "frem",
                    _ => {
                        return Err(CodegenError::InternalError(format!(
                            "BinOp {:?} in float_arith codegen path",
                            op
                        )))
                    }
                };
                // Determine target float type
                let left_is_f32 = matches!(left_type, ResolvedType::F32);
                let right_is_f32 = matches!(right_type, ResolvedType::F32);
                let left_is_f64 = matches!(left_type, ResolvedType::F64);
                let right_is_f64 = matches!(right_type, ResolvedType::F64);

                let float_llvm;
                let mut actual_left = left_val.clone();
                let mut actual_right = right_val.clone();

                if (left_is_f32 && right_is_f64) || (left_is_f64 && right_is_f32) {
                    // Mixed f32/f64 — promote f32 to f64
                    float_llvm = "double";
                    if left_is_f32 {
                        let ext = self.next_temp(counter);
                        write_ir!(ir, "  {} = fpext float {} to double", ext, left_val);
                        actual_left = ext;
                    }
                    if right_is_f32 {
                        let ext = self.next_temp(counter);
                        write_ir!(ir, "  {} = fpext float {} to double", ext, right_val);
                        actual_right = ext;
                    }
                } else if left_is_f32 || right_is_f32 {
                    float_llvm = "float";
                } else {
                    float_llvm = "double";
                }

                write_ir!(
                    ir,
                    "  {} = {} {} {}, {}{}",
                    tmp,
                    op_str,
                    float_llvm,
                    actual_left,
                    actual_right,
                    dbg_info
                );
            } else {
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
                    _ => {
                        return Err(CodegenError::InternalError(format!(
                            "BinOp {:?} in int_arith codegen path",
                            op
                        )))
                    }
                };
                // Use MAX of both operand widths as the target so that narrower
                // operands are promoted before the instruction (fixes P3: e.g.,
                // shl i16 %t20, %t23 where %t20 is actually i8).
                let left_bits = self.get_integer_bits(&left_type);
                let right_bits = self.get_integer_bits(&right_type);
                let target_bits = if left_bits > 0 && right_bits > 0 {
                    std::cmp::max(left_bits, right_bits)
                } else if left_bits > 0 {
                    left_bits
                } else if right_bits > 0 {
                    right_bits
                } else {
                    64 // default
                };
                let int_llvm_owned = format!("i{}", target_bits);
                let int_llvm: &str = &int_llvm_owned;

                // Coerce both operands to the target width (using inferred types, not llvm_type_of)
                let left_ty_str = format!("i{}", if left_bits > 0 { left_bits } else { 64 });
                let right_ty_str = format!("i{}", if right_bits > 0 { right_bits } else { 64 });
                let coerced_left =
                    self.coerce_int_width(&left_val, &left_ty_str, int_llvm, counter, &mut ir);
                let coerced_right =
                    self.coerce_int_width(&right_val, &right_ty_str, int_llvm, counter, &mut ir);
                write_ir!(
                    ir,
                    "  {} = {} {} {}, {}{}",
                    tmp,
                    op_str,
                    int_llvm,
                    coerced_left,
                    coerced_right,
                    dbg_info
                );
            }
            Ok((tmp, ir))
        }
    }

    /// Generate unary expression
    #[inline(never)]
    pub(crate) fn generate_unary_expr(
        &mut self,
        op: &UnaryOp,
        expr: &Spanned<Expr>,
        counter: &mut usize,
        span: Span,
    ) -> CodegenResult<(String, String)> {
        let (val, val_ir) = self.generate_expr(expr, counter)?;
        let tmp = self.next_temp(counter);

        let mut ir = val_ir;
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let expr_type = self.infer_expr_type(expr);
        let int_llvm = match &expr_type {
            ResolvedType::I8 | ResolvedType::U8 => "i8",
            ResolvedType::I16 | ResolvedType::U16 => "i16",
            ResolvedType::I32 | ResolvedType::U32 => "i32",
            ResolvedType::I128 | ResolvedType::U128 => "i128",
            ResolvedType::Bool => "i1",
            _ => "i64",
        };
        match op {
            UnaryOp::Neg => {
                write_ir!(ir, "  {} = sub {} 0, {}{}", tmp, int_llvm, val, dbg_info);
            }
            UnaryOp::Not => {
                write_ir!(ir, "  {} = xor i1 {}, 1{}", tmp, val, dbg_info);
            }
            UnaryOp::BitNot => {
                write_ir!(ir, "  {} = xor {} {}, -1{}", tmp, int_llvm, val, dbg_info);
            }
        }

        Ok((tmp, ir))
    }

    /// Generate ternary expression
    #[inline(never)]
    pub(crate) fn generate_cast_expr(
        &mut self,
        expr: &Spanned<Expr>,
        ty: &Spanned<Type>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (val, val_ir) = self.generate_expr(expr, counter)?;
        let mut ir = val_ir;

        let target_type = self.ast_type_to_resolved(&ty.node);
        let llvm_type = self.type_to_llvm(&target_type);

        // Check source type for str→i64 cast: extract data pointer from fat pointer
        let src_llvm_ty = self.llvm_type_of(&val);
        if src_llvm_ty == "{ i8*, i64 }" && llvm_type == "i64" {
            // str → i64: extract the data pointer (field 0) and ptrtoint
            let ptr_val = self.next_temp(counter);
            let result = self.next_temp(counter);
            write_ir!(ir, "  {} = extractvalue {{ i8*, i64 }} {}, 0", ptr_val, val);
            write_ir!(ir, "  {} = ptrtoint i8* {} to i64", result, ptr_val);
            return Ok((result, ir));
        }

        // i64 → str cast: convert pointer-as-i64 to fat pointer { i8*, i64 }
        if src_llvm_ty == "i64" && llvm_type == "{ i8*, i64 }" {
            let ptr_val = self.next_temp(counter);
            let fat1 = self.next_temp(counter);
            let result = self.next_temp(counter);
            write_ir!(ir, "  {} = inttoptr i64 {} to i8*", ptr_val, val);
            write_ir!(ir, "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0", fat1, ptr_val);
            write_ir!(ir, "  {} = insertvalue {{ i8*, i64 }} {}, i64 0, 1", result, fat1);
            return Ok((result, ir));
        }

        // pointer/struct → i64 cast: ptrtoint
        if llvm_type == "i64" && (src_llvm_ty.ends_with('*') || src_llvm_ty.starts_with('%')) {
            let result = self.next_temp(counter);
            if src_llvm_ty.ends_with('*') {
                write_ir!(ir, "  {} = ptrtoint {} {} to i64", result, src_llvm_ty, val);
            } else {
                // Named struct type — the SSA value is actually a pointer (from alloca)
                write_ir!(ir, "  {} = ptrtoint {}* {} to i64", result, src_llvm_ty, val);
            }
            return Ok((result, ir));
        }

        // Simple cast - in many cases just bitcast or pass through
        let result = self.next_temp(counter);
        match (&target_type, llvm_type.as_str()) {
            // Integer to pointer cast
            (ResolvedType::Pointer(_), _)
            | (ResolvedType::Ref(_), _)
            | (ResolvedType::RefMut(_), _) => {
                write_ir!(ir, "  {} = inttoptr i64 {} to {}", result, val, llvm_type);
            }
            // Default: just use the value as-is (same size types)
            _ => {
                return Ok((val, ir));
            }
        }

        Ok((result, ir))
    }

    /// Generate assign expression
    #[inline(never)]
    pub(crate) fn generate_assign_expr(
        &mut self,
        target: &Spanned<Expr>,
        value: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
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
                        let alloca_name = format!("{}.{}", name, counter);
                        *counter += 1;
                        // Create alloca, store current value (coerce width if mismatched)
                        let actual_val_ty = self.llvm_type_of(&val);
                        let coerced_val =
                            self.coerce_int_width(&val, &actual_val_ty, &llvm_ty, counter, &mut ir);
                        self.emit_entry_alloca(&format!("%{}", alloca_name), &llvm_ty);
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
                        let coerced_val =
                            self.coerce_int_width(&val, &actual_val_ty, &llvm_ty, counter, &mut ir);
                        // For struct types (Named), the local is a single pointer (%Type*).
                        // Store the struct value directly into the alloca.
                        if matches!(&local.ty, ResolvedType::Named { .. }) && local.is_alloca() {
                            write_ir!(
                                ir,
                                "  store {} {}, {}* %{}",
                                llvm_ty,
                                coerced_val,
                                llvm_ty,
                                local.llvm_name
                            );
                        } else {
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
                        // Coerce value width to match field type
                        let actual_val_ty = self.llvm_type_of(&val);
                        let coerced_val =
                            self.coerce_int_width(&val, &actual_val_ty, &llvm_ty, counter, &mut ir);
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
            let (elem_llvm_ty, is_fat_ptr) = match &arr_ty {
                ResolvedType::Pointer(elem) => (self.type_to_llvm(elem), false),
                ResolvedType::Array(elem) => (self.type_to_llvm(elem), false),
                ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => {
                    (self.type_to_llvm(elem), true)
                }
                _ => ("i64".to_string(), false),
            };

            // For fat pointer slices { i8*, i64 }, extract data pointer and bitcast
            let base_ptr = if is_fat_ptr {
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
            } else {
                arr_val.clone()
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
            write_ir!(
                ir,
                "  store {} {}, {}* {}",
                elem_llvm_ty,
                val,
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
            // Not a field, fall through to error
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

        let result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = {} i64 {}, {}",
            result,
            op_str,
            current_val,
            rhs_val
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
