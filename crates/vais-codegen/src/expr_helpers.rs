//! Expression generation helper methods for CodeGenerator
//!
//! This module contains core expression helpers: enum variants,
//! binary/unary operations, casts, and assignment operations.

use crate::{CodeGenerator, CodegenError, CodegenResult};
use vais_ast::{BinOp, Expr, Span, Spanned, Type, UnaryOp};
use vais_types::ResolvedType;

impl CodeGenerator {
    pub(crate) fn generate_unit_enum_variant(
        &mut self,
        name: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        for enum_info in self.types.enums.values() {
            for (tag, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == name {
                    let mut ir = String::new();
                    let enum_ptr = self.next_temp(counter);
                    ir.push_str(&format!("  {} = alloca %{}\n", enum_ptr, enum_info.name));
                    // Store tag
                    let tag_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0\n",
                        tag_ptr, enum_info.name, enum_info.name, enum_ptr
                    ));
                    ir.push_str(&format!("  store i32 {}, i32* {}\n", tag, tag_ptr));
                    return Ok((enum_ptr, ir));
                }
            }
        }
        // Fallback if not found (shouldn't happen)
        Ok((format!("@{}", name), String::new()))
    }

    /// Generate binary expression
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
            let left_bool = self.next_temp(counter);
            ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", left_bool, left_val));
            let right_bool = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = icmp ne i64 {}, 0\n",
                right_bool, right_val
            ));

            let op_str = match op {
                BinOp::And => "and",
                BinOp::Or => "or",
                _ => {
                    eprintln!(
                        "[ICE] unexpected BinOp variant in logical operation: {:?}",
                        op
                    );
                    return Err(CodegenError::Unsupported(format!(
                        "unexpected logical operator {:?}",
                        op
                    )));
                }
            };

            let result_bool = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            ir.push_str(&format!(
                "  {} = {} i1 {}, {}{}\n",
                result_bool, op_str, left_bool, right_bool, dbg_info
            ));

            // Extend back to i64 for consistency
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = zext i1 {} to i64\n", result, result_bool));
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
                        eprintln!(
                            "[ICE] unexpected BinOp variant in float comparison: {:?}",
                            op
                        );
                        return Err(CodegenError::Unsupported(format!(
                            "unexpected float comparison operator {:?}",
                            op
                        )));
                    }
                };
                ir.push_str(&format!(
                    "  {} = {} double {}, {}{}\n",
                    cmp_tmp, op_str, left_val, right_val, dbg_info
                ));
            } else {
                let op_str = match op {
                    BinOp::Lt => "icmp slt",
                    BinOp::Lte => "icmp sle",
                    BinOp::Gt => "icmp sgt",
                    BinOp::Gte => "icmp sge",
                    BinOp::Eq => "icmp eq",
                    BinOp::Neq => "icmp ne",
                    _ => {
                        eprintln!(
                            "[ICE] unexpected BinOp variant in integer comparison: {:?}",
                            op
                        );
                        return Err(CodegenError::Unsupported(format!(
                            "unexpected integer comparison operator {:?}",
                            op
                        )));
                    }
                };
                ir.push_str(&format!(
                    "  {} = {} i64 {}, {}{}\n",
                    cmp_tmp, op_str, left_val, right_val, dbg_info
                ));
            }

            // Extend i1 to i64
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = zext i1 {} to i64\n", result, cmp_tmp));
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
                        eprintln!(
                            "[ICE] unexpected BinOp variant in float arithmetic: {:?}",
                            op
                        );
                        return Err(CodegenError::Unsupported(format!(
                            "unexpected float arithmetic operator {:?}",
                            op
                        )));
                    }
                };
                ir.push_str(&format!(
                    "  {} = {} double {}, {}{}\n",
                    tmp, op_str, left_val, right_val, dbg_info
                ));
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
                        eprintln!(
                            "[ICE] unexpected BinOp variant in integer arithmetic: {:?}",
                            op
                        );
                        return Err(CodegenError::Unsupported(format!(
                            "unexpected integer arithmetic operator {:?}",
                            op
                        )));
                    }
                };
                ir.push_str(&format!(
                    "  {} = {} i64 {}, {}{}\n",
                    tmp, op_str, left_val, right_val, dbg_info
                ));
            }
            Ok((tmp, ir))
        }
    }

    /// Generate unary expression
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
        match op {
            UnaryOp::Neg => {
                ir.push_str(&format!("  {} = sub i64 0, {}{}\n", tmp, val, dbg_info));
            }
            UnaryOp::Not => {
                ir.push_str(&format!("  {} = xor i1 {}, 1{}\n", tmp, val, dbg_info));
            }
            UnaryOp::BitNot => {
                ir.push_str(&format!("  {} = xor i64 {}, -1{}\n", tmp, val, dbg_info));
            }
        }

        Ok((tmp, ir))
    }

    /// Generate ternary expression
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

        // Simple cast - in many cases just bitcast or pass through
        let result = self.next_temp(counter);
        match (&target_type, llvm_type.as_str()) {
            // Integer to pointer cast
            (ResolvedType::Pointer(_), _)
            | (ResolvedType::Ref(_), _)
            | (ResolvedType::RefMut(_), _) => {
                ir.push_str(&format!(
                    "  {} = inttoptr i64 {} to {}\n",
                    result, val, llvm_type
                ));
            }
            // Default: just use the value as-is (same size types)
            _ => {
                return Ok((val, ir));
            }
        }

        Ok((result, ir))
    }

    /// Generate assign expression
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
                    let llvm_ty = self.type_to_llvm(&local.ty);
                    // For struct types (Named), the local is a double pointer (%Type**).
                    // We need to alloca a new struct, store the value, then update the pointer.
                    if matches!(&local.ty, ResolvedType::Named { .. }) && local.is_alloca() {
                        let tmp_ptr = self.next_temp(counter);
                        ir.push_str(&format!("  {} = alloca {}\n", tmp_ptr, llvm_ty));
                        ir.push_str(&format!(
                            "  store {} {}, {}* {}\n",
                            llvm_ty, val, llvm_ty, tmp_ptr
                        ));
                        ir.push_str(&format!(
                            "  store {}* {}, {}** %{}\n",
                            llvm_ty, tmp_ptr, llvm_ty, local.llvm_name
                        ));
                    } else {
                        ir.push_str(&format!(
                            "  store {} {}, {}* %{}\n",
                            llvm_ty, val, llvm_ty, local.llvm_name
                        ));
                    }
                }
            }
        } else if let Expr::Field {
            expr: obj_expr,
            field,
        } = &target.node
        {
            let (obj_val, obj_ir) = self.generate_expr(obj_expr, counter)?;
            ir.push_str(&obj_ir);

            if let Expr::Ident(var_name) = &obj_expr.node {
                if let Some(local) = self.fn_ctx.locals.get(var_name.as_str()).cloned() {
                    if let ResolvedType::Named {
                        name: struct_name, ..
                    } = &local.ty
                    {
                        if let Some(struct_info) = self.types.structs.get(struct_name).cloned() {
                            if let Some(field_idx) = struct_info
                                .fields
                                .iter()
                                .position(|(n, _)| n == &field.node)
                            {
                                let field_ty = &struct_info.fields[field_idx].1;
                                let llvm_ty = self.type_to_llvm(field_ty);

                                let field_ptr = self.next_temp(counter);
                                ir.push_str(&format!(
                                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                                    field_ptr, struct_name, struct_name, obj_val, field_idx
                                ));
                                ir.push_str(&format!(
                                    "  store {} {}, {}* {}\n",
                                    llvm_ty, val, llvm_ty, field_ptr
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok((val, ir))
    }

    /// Generate compound assignment expression
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
        ir.push_str(&format!(
            "  {} = {} i64 {}, {}\n",
            result, op_str, current_val, rhs_val
        ));

        if let Expr::Ident(name) = &target.node {
            if let Some(local) = self.fn_ctx.locals.get(name.as_str()).cloned() {
                if !local.is_param() {
                    let llvm_ty = self.type_to_llvm(&local.ty);
                    ir.push_str(&format!(
                        "  store {} {}, {}* %{}\n",
                        llvm_ty, result, llvm_ty, local.llvm_name
                    ));
                }
            }
        }

        Ok((result, ir))
    }
}
