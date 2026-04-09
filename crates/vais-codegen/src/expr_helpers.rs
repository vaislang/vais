//! Expression generation helper methods for CodeGenerator
//!
//! This module contains core expression helpers: enum variants,
//! binary/unary operations, and cast operations.
//! Assignment and identifier expression helpers are in expr_helpers_assign.

use crate::{CodeGenerator, CodegenError, CodegenResult};
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

                // Check for int operands that need sitofp conversion
                let left_is_int = !left_is_f32 && !left_is_f64;
                let right_is_int = !right_is_f32 && !right_is_f64;

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

                // Convert int operands to float (sitofp) for mixed int*float arithmetic
                if left_is_int {
                    let conv = self.next_temp(counter);
                    let int_llvm = self.type_to_llvm(&left_type);
                    write_ir!(ir, "  {} = sitofp {} {} to {}", conv, int_llvm, actual_left, float_llvm);
                    actual_left = conv;
                }
                if right_is_int {
                    let conv = self.next_temp(counter);
                    let int_llvm = self.type_to_llvm(&right_type);
                    write_ir!(ir, "  {} = sitofp {} {} to {}", conv, int_llvm, actual_right, float_llvm);
                    actual_right = conv;
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
            write_ir!(
                ir,
                "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
                fat1,
                ptr_val
            );
            write_ir!(
                ir,
                "  {} = insertvalue {{ i8*, i64 }} {}, i64 0, 1",
                result,
                fat1
            );
            return Ok((result, ir));
        }

        // pointer/struct → i64 cast: ptrtoint
        if llvm_type == "i64" && (src_llvm_ty.ends_with('*') || src_llvm_ty.starts_with('%')) {
            let result = self.next_temp(counter);
            if src_llvm_ty.ends_with('*') {
                write_ir!(ir, "  {} = ptrtoint {} {} to i64", result, src_llvm_ty, val);
            } else {
                // Named struct type — the SSA value is actually a pointer (from alloca)
                write_ir!(
                    ir,
                    "  {} = ptrtoint {}* {} to i64",
                    result,
                    src_llvm_ty,
                    val
                );
            }
            return Ok((result, ir));
        }

        // Integer width coercion: only apply when source type is reliably known
        // (not fallback i64). The "everything-is-i64" body convention means that
        // i64→i32 truncation would break downstream code that expects i64 values.
        // Only widen (i8/i16/i32 → i64) from known types, not narrow.
        {
            let has_known_type = self.fn_ctx.get_temp_type(&val).is_some()
                || self
                    .fn_ctx
                    .locals
                    .contains_key(val.strip_prefix('%').unwrap_or(&val));
            if has_known_type
                && src_llvm_ty.starts_with('i')
                && llvm_type.starts_with('i')
                && src_llvm_ty != llvm_type
            {
                let src_bits: u32 = src_llvm_ty[1..].parse().unwrap_or(0);
                let dst_bits: u32 = llvm_type[1..].parse().unwrap_or(0);
                if src_bits > 0 && dst_bits > 0 && src_bits != dst_bits {
                    let result = self.next_temp(counter);
                    if src_bits > dst_bits {
                        write_ir!(
                            ir,
                            "  {} = trunc {} {} to {}",
                            result,
                            src_llvm_ty,
                            val,
                            llvm_type
                        );
                    } else {
                        write_ir!(
                            ir,
                            "  {} = sext {} {} to {}",
                            result,
                            src_llvm_ty,
                            val,
                            llvm_type
                        );
                    }
                    return Ok((result, ir));
                }
            }
        }

        // Float width coercion: f32 ↔ f64 (fpext/fptrunc)
        if (src_llvm_ty == "float" && llvm_type == "double")
            || (src_llvm_ty == "double" && llvm_type == "float")
        {
            let result = self.next_temp(counter);
            if src_llvm_ty == "float" {
                write_ir!(ir, "  {} = fpext float {} to double", result, val);
            } else {
                write_ir!(ir, "  {} = fptrunc double {} to float", result, val);
            }
            return Ok((result, ir));
        }

        // Integer ↔ float coercion (as f64, as f32 from int, as i64 from float)
        if src_llvm_ty.starts_with('i') && (llvm_type == "float" || llvm_type == "double") {
            // Check if the value is actually a float literal (e.g., "5.000000e+00")
            // that was given i64 type by the "everything is i64" fallback.
            // Float literals don't start with '%' and contain 'e+' or 'e-' (scientific notation).
            let is_float_literal =
                !val.starts_with('%') && (val.contains("e+") || val.contains("e-"));
            if is_float_literal {
                // Return the literal directly — it's already a valid LLVM float constant
                return Ok((val.clone(), ir));
            }
            let result = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = sitofp {} {} to {}",
                result,
                src_llvm_ty,
                val,
                llvm_type
            );
            return Ok((result, ir));
        }
        if (src_llvm_ty == "float" || src_llvm_ty == "double") && llvm_type.starts_with('i') {
            let result = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = fptosi {} {} to {}",
                result,
                src_llvm_ty,
                val,
                llvm_type
            );
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
}
