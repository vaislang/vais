//! Expression code generation for Vais compiler
//!
//! This module contains helper functions for generating LLVM IR from Vais expressions.
//! The main generate_expr function remains in lib.rs but delegates to these helpers.

use crate::{CodeGenerator, CodegenResult, CodegenError};
use vais_ast::{Spanned, Expr, BinOp, UnaryOp};
use vais_types::ResolvedType;

#[allow(dead_code)]
impl CodeGenerator {
    /// Generate code for a literal expression (Int, Float, Bool, String, Unit)
    pub(crate) fn generate_literal(
        &mut self,
        expr: &Expr,
    ) -> CodegenResult<(String, String)> {
        match expr {
            Expr::Int(n) => Ok((n.to_string(), String::new())),
            Expr::Float(n) => Ok((crate::types::format_llvm_float(*n), String::new())),
            Expr::Bool(b) => Ok((if *b { "1" } else { "0" }.to_string(), String::new())),
            Expr::String(s) => {
                let name = format!(".str.{}", self.string_counter);
                self.string_counter += 1;
                self.string_constants.push((name.clone(), s.clone()));
                let len = s.len() + 1;
                Ok((
                    format!("getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)", len, len, name),
                    String::new(),
                ))
            }
            Expr::Unit => Ok(("void".to_string(), String::new())),
            _ => Err(CodegenError::Unsupported("Not a literal expression".to_string())),
        }
    }

    /// Generate code for identifier access
    pub(crate) fn generate_ident(
        &mut self,
        name: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        if let Some(local) = self.locals.get(name).cloned() {
            if local.is_param() {
                Ok((format!("%{}", local.llvm_name), String::new()))
            } else if matches!(local.ty, ResolvedType::Named { .. }) {
                let tmp = self.next_temp(counter);
                let llvm_ty = self.type_to_llvm(&local.ty);
                let ir = format!(
                    "  {} = load {}*, {}** %{}\n",
                    tmp, llvm_ty, llvm_ty, local.llvm_name
                );
                Ok((tmp, ir))
            } else {
                let tmp = self.next_temp(counter);
                let llvm_ty = self.type_to_llvm(&local.ty);
                let ir = format!(
                    "  {} = load {}, {}* %{}\n",
                    tmp, llvm_ty, llvm_ty, local.llvm_name
                );
                Ok((tmp, ir))
            }
        } else if name == "self" {
            Ok(("%self".to_string(), String::new()))
        } else if self.functions.contains_key(name) {
            Ok((format!("@{}", name), String::new()))
        } else if let Some(self_local) = self.locals.get("self").cloned() {
            // Implicit self: check if name is a field of the self struct
            let self_type = match &self_local.ty {
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref().clone(),
                other => other.clone(),
            };
            if let ResolvedType::Named { name: type_name, .. } = &self_type {
                let resolved_name = self.resolve_struct_name(type_name);
                if let Some(struct_info) = self.structs.get(&resolved_name).cloned() {
                    if let Some(field_idx) = struct_info.fields.iter().position(|(n, _)| n == name) {
                        let field_ty = &struct_info.fields[field_idx].1;
                        let llvm_ty = self.type_to_llvm(field_ty);
                        let mut ir = String::new();
                        // Generate: getelementptr from %self
                        let field_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr %{}, %{}* %self, i32 0, i32 {}\n",
                            field_ptr, resolved_name, resolved_name, field_idx
                        ));
                        if matches!(field_ty, ResolvedType::Named { .. }) {
                            return Ok((field_ptr, ir));
                        } else {
                            let result = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = load {}, {}* {}\n",
                                result, llvm_ty, llvm_ty, field_ptr
                            ));
                            return Ok((result, ir));
                        }
                    }
                }
            }
            // Not a field, fall through to error
            let mut candidates: Vec<&str> = Vec::new();
            for var_name in self.locals.keys() {
                candidates.push(var_name.as_str());
            }
            for func_name in self.functions.keys() {
                candidates.push(func_name.as_str());
            }
            candidates.push("self");
            let suggestions = crate::suggest_similar(name, &candidates, 3);
            let suggestion_text = crate::format_did_you_mean(&suggestions);
            Err(CodegenError::UndefinedVar(format!("{}{}", name, suggestion_text)))
        } else {
            // Collect all available symbols for suggestions
            let mut candidates: Vec<&str> = Vec::new();

            // Add local variables
            for var_name in self.locals.keys() {
                candidates.push(var_name.as_str());
            }

            // Add function names
            for func_name in self.functions.keys() {
                candidates.push(func_name.as_str());
            }

            // Add "self" if we're in a method context
            if self.current_function.is_some() {
                candidates.push("self");
            }

            // Get suggestions
            let suggestions = crate::suggest_similar(name, &candidates, 3);
            let suggestion_text = crate::format_did_you_mean(&suggestions);

            Err(CodegenError::UndefinedVar(format!("{}{}", name, suggestion_text)))
        }
    }

    /// Generate code for binary operations
    pub(crate) fn generate_binary_op(
        &mut self,
        op: &BinOp,
        left: &Spanned<Expr>,
        right: &Spanned<Expr>,
        counter: &mut usize,
        span: vais_ast::Span,
    ) -> CodegenResult<(String, String)> {
        let (left_val, left_ir) = self.generate_expr(left, counter)?;
        let (right_val, right_ir) = self.generate_expr(right, counter)?;

        let left_type = self.infer_expr_type(left);

        // Handle string operations
        if matches!(left_type, ResolvedType::Str) {
            let mut ir = left_ir;
            ir.push_str(&right_ir);
            return self.generate_string_binary_op(op, &left_val, &right_val, ir, counter);
        }

        let tmp = self.next_temp(counter);
        let is_float = matches!(left_type, ResolvedType::F32 | ResolvedType::F64);
        let bits = self.get_integer_bits(&left_type);

        let (op_str, is_cmp) = self.get_binary_op_str(op, is_float, bits);

        let result_type = if is_cmp {
            "i1".to_string()
        } else if is_float {
            if matches!(left_type, ResolvedType::F32) { "float" } else { "double" }.to_string()
        } else {
            format!("i{}", bits)
        };

        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let ir = format!(
            "{}{}  {} = {} {} {}, {}{}\n",
            left_ir, right_ir, tmp, op_str, result_type, left_val, right_val, dbg_info
        );
        Ok((tmp, ir))
    }

    /// Get the LLVM instruction for a binary operator
    fn get_binary_op_str(&self, op: &BinOp, is_float: bool, _bits: u32) -> (&'static str, bool) {
        match op {
            // Arithmetic
            BinOp::Add => (if is_float { "fadd" } else { "add" }, false),
            BinOp::Sub => (if is_float { "fsub" } else { "sub" }, false),
            BinOp::Mul => (if is_float { "fmul" } else { "mul" }, false),
            BinOp::Div => (if is_float { "fdiv" } else { "sdiv" }, false),
            BinOp::Mod => (if is_float { "frem" } else { "srem" }, false),
            // Comparison
            BinOp::Eq => (if is_float { "fcmp oeq" } else { "icmp eq" }, true),
            BinOp::Neq => (if is_float { "fcmp one" } else { "icmp ne" }, true),
            BinOp::Lt => (if is_float { "fcmp olt" } else { "icmp slt" }, true),
            BinOp::Lte => (if is_float { "fcmp ole" } else { "icmp sle" }, true),
            BinOp::Gt => (if is_float { "fcmp ogt" } else { "icmp sgt" }, true),
            BinOp::Gte => (if is_float { "fcmp oge" } else { "icmp sge" }, true),
            // Logical (integer only)
            BinOp::And => ("and", false),
            BinOp::Or => ("or", false),
            // Bitwise
            BinOp::BitAnd => ("and", false),
            BinOp::BitOr => ("or", false),
            BinOp::BitXor => ("xor", false),
            BinOp::Shl => ("shl", false),
            BinOp::Shr => ("ashr", false), // Arithmetic shift right for signed
        }
    }

    /// Generate code for unary operations
    pub(crate) fn generate_unary_op(
        &mut self,
        op: &UnaryOp,
        expr: &Spanned<Expr>,
        counter: &mut usize,
        span: vais_ast::Span,
    ) -> CodegenResult<(String, String)> {
        let (val, ir) = self.generate_expr(expr, counter)?;
        let tmp = self.next_temp(counter);

        let expr_type = self.infer_expr_type(expr);
        let is_float = matches!(expr_type, ResolvedType::F32 | ResolvedType::F64);
        let bits = self.get_integer_bits(&expr_type);

        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let result_ir = match op {
            UnaryOp::Neg => {
                if is_float {
                    let float_ty = if matches!(expr_type, ResolvedType::F32) { "float" } else { "double" };
                    format!("{}  {} = fneg {} {}{}\n", ir, tmp, float_ty, val, dbg_info)
                } else {
                    format!("{}  {} = sub i{} 0, {}{}\n", ir, tmp, bits, val, dbg_info)
                }
            }
            UnaryOp::Not => {
                format!("{}  {} = xor i{} {}, -1{}\n", ir, tmp, bits, val, dbg_info)
            }
            UnaryOp::BitNot => {
                format!("{}  {} = xor i{} {}, -1{}\n", ir, tmp, bits, val, dbg_info)
            }
        };

        Ok((tmp, result_ir))
    }

    /// Generate code for assignment expressions
    pub(crate) fn generate_assignment(
        &mut self,
        target: &Spanned<Expr>,
        value: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (val, val_ir) = self.generate_expr(value, counter)?;

        match &target.node {
            Expr::Ident(name) => {
                if let Some(local) = self.locals.get(name).cloned() {
                    let llvm_ty = self.type_to_llvm(&local.ty);
                    let ir = format!(
                        "{}  store {} {}, {}* %{}\n",
                        val_ir, llvm_ty, val, llvm_ty, local.llvm_name
                    );
                    Ok((val.clone(), ir))
                } else {
                    // Collect all available symbols for suggestions
                    let candidates: Vec<&str> = self.locals.keys()
                        .map(|s| s.as_str())
                        .collect();

                    let suggestions = crate::suggest_similar(name, &candidates, 3);
                    let suggestion_text = crate::format_did_you_mean(&suggestions);

                    Err(CodegenError::UndefinedVar(format!("{}{}", name, suggestion_text)))
                }
            }
            Expr::Index { expr: arr_expr, index } => {
                self.generate_index_assignment(arr_expr, index, &val, &val_ir, counter)
            }
            Expr::Field { expr: obj_expr, field } => {
                self.generate_field_assignment(obj_expr, field, &val, &val_ir, counter)
            }
            _ => Err(CodegenError::Unsupported("Invalid assignment target".to_string())),
        }
    }

    /// Generate code for index assignment (arr[i] = val)
    fn generate_index_assignment(
        &mut self,
        arr_expr: &Spanned<Expr>,
        index: &Spanned<Expr>,
        val: &str,
        val_ir: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (arr_val, arr_ir) = self.generate_expr(arr_expr, counter)?;
        let (idx_val, idx_ir) = self.generate_expr(index, counter)?;

        let arr_type = self.infer_expr_type(arr_expr);

        match &arr_type {
            ResolvedType::Array(elem_ty) => {
                let data_ptr = self.next_temp(counter);
                let elem_ptr = self.next_temp(counter);
                let elem_llvm = self.type_to_llvm(elem_ty);

                let ir = format!(
                    "{}{}{}  {} = extractvalue {{ i64, {}* }} {}, 1\n  {} = getelementptr {}, {}* {}, i64 {}\n  store {} {}, {}* {}\n",
                    val_ir, arr_ir, idx_ir,
                    data_ptr, elem_llvm, arr_val,
                    elem_ptr, elem_llvm, elem_llvm, data_ptr, idx_val,
                    elem_llvm, val, elem_llvm, elem_ptr
                );
                Ok((val.to_string(), ir))
            }
            _ => {
                let type_name = format!("{:?}", arr_type);
                Err(CodegenError::TypeError(format!(
                    "Cannot index non-array type (found {})",
                    type_name
                )))
            }
        }
    }

    /// Generate code for field assignment (obj.field = val)
    fn generate_field_assignment(
        &mut self,
        obj_expr: &Spanned<Expr>,
        field: &Spanned<String>,
        val: &str,
        val_ir: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (obj_val, obj_ir) = self.generate_expr(obj_expr, counter)?;
        let obj_type = self.infer_expr_type(obj_expr);

        if let ResolvedType::Named { name: struct_name, .. } = &obj_type {
            if let Some(struct_info) = self.structs.get(struct_name).cloned() {
                let field_idx = struct_info.fields.iter()
                    .position(|(n, _)| n == &field.node);

                let field_idx = if let Some(idx) = field_idx {
                    idx
                } else {
                    // Suggest similar field names
                    let candidates: Vec<&str> = struct_info.fields.iter()
                        .map(|(name, _)| name.as_str())
                        .collect();

                    let suggestions = crate::suggest_similar(&field.node, &candidates, 3);
                    let suggestion_text = crate::format_did_you_mean(&suggestions);

                    return Err(CodegenError::UndefinedVar(format!("field {}{}", field.node, suggestion_text)));
                };

                let struct_llvm = self.type_to_llvm(&obj_type);
                let (_, field_ty) = &struct_info.fields[field_idx];
                let field_llvm = self.type_to_llvm(field_ty);

                let field_ptr = self.next_temp(counter);
                let ir = format!(
                    "{}{}  {} = getelementptr {}, {}* {}, i32 0, i32 {}\n  store {} {}, {}* {}\n",
                    val_ir, obj_ir, field_ptr, struct_llvm, struct_llvm, obj_val, field_idx,
                    field_llvm, val, field_llvm, field_ptr
                );
                Ok((val.to_string(), ir))
            } else {
                Err(CodegenError::UndefinedVar(format!("struct {} not found", struct_name)))
            }
        } else {
            let type_name = format!("{:?}", obj_type);
            Err(CodegenError::TypeError(format!(
                "Cannot access field of non-struct type (found {})",
                type_name
            )))
        }
    }

    /// Generate code for compound assignment (+=, -=, etc.)
    pub(crate) fn generate_compound_assignment(
        &mut self,
        op: &BinOp,
        target: &Spanned<Expr>,
        value: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // First, read the current value
        let (current_val, current_ir) = self.generate_expr(target, counter)?;
        let (add_val, add_ir) = self.generate_expr(value, counter)?;

        let tmp = self.next_temp(counter);
        let target_type = self.infer_expr_type(target);
        let is_float = matches!(target_type, ResolvedType::F32 | ResolvedType::F64);
        let bits = self.get_integer_bits(&target_type);

        let (op_str, _) = self.get_binary_op_str(op, is_float, bits);
        let result_type = if is_float {
            if matches!(target_type, ResolvedType::F32) { "float" } else { "double" }.to_string()
        } else {
            format!("i{}", bits)
        };

        let compute_ir = format!(
            "{}{}  {} = {} {} {}, {}\n",
            current_ir, add_ir, tmp, op_str, result_type, current_val, add_val
        );

        // Now store back
        match &target.node {
            Expr::Ident(name) => {
                if let Some(local) = self.locals.get(name).cloned() {
                    let llvm_ty = self.type_to_llvm(&local.ty);
                    let ir = format!(
                        "{}  store {} {}, {}* %{}\n",
                        compute_ir, llvm_ty, tmp, llvm_ty, local.llvm_name
                    );
                    Ok((tmp, ir))
                } else {
                    Err(CodegenError::UndefinedVar(name.clone()))
                }
            }
            _ => Err(CodegenError::Unsupported("Compound assignment to complex target".to_string())),
        }
    }
}
