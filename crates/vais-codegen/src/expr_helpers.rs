//! Expression generation helper methods for CodeGenerator
//!
//! This module contains helper methods that are used by the ExprVisitor
//! implementation to generate specific expression types.

use crate::types::{ClosureInfo, LocalVar, LoopLabels};
use crate::{CodeGenerator, CodegenError, CodegenResult};
use vais_ast::{BinOp, Expr, Param, Span, Spanned, Stmt, Type, UnaryOp};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Generate a unit enum variant
    pub(crate) fn generate_unit_enum_variant(
        &mut self,
        name: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        for enum_info in self.enums.values() {
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
                _ => unreachable!(),
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
                    _ => unreachable!(),
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
                    _ => unreachable!(),
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
                    _ => unreachable!(),
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
                    _ => unreachable!(),
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
    pub(crate) fn generate_ternary_expr(
        &mut self,
        cond: &Spanned<Expr>,
        then: &Spanned<Expr>,
        else_: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Use proper branching for lazy evaluation
        let then_label = self.next_label("ternary.then");
        let else_label = self.next_label("ternary.else");
        let merge_label = self.next_label("ternary.merge");

        // Generate condition
        let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
        let mut ir = cond_ir;

        // Convert i64 to i1 for branch
        let cond_bool = self.next_temp(counter);
        ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_bool, cond_val));

        // Conditional branch
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cond_bool, then_label, else_label
        ));

        // Then branch
        ir.push_str(&format!("{}:\n", then_label));
        let (then_val, then_ir) = self.generate_expr(then, counter)?;
        ir.push_str(&then_ir);
        ir.push_str(&format!("  br label %{}\n", merge_label));

        // Else branch
        ir.push_str(&format!("{}:\n", else_label));
        let (else_val, else_ir) = self.generate_expr(else_, counter)?;
        ir.push_str(&else_ir);
        ir.push_str(&format!("  br label %{}\n", merge_label));

        // Merge with phi
        ir.push_str(&format!("{}:\n", merge_label));
        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = phi i64 [ {}, %{} ], [ {}, %{} ]\n",
            result, then_val, then_label, else_val, else_label
        ));

        Ok((result, ir))
    }

    /// Generate function call expression
    pub(crate) fn generate_call_expr(
        &mut self,
        func: &Spanned<Expr>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
        span: Span,
    ) -> CodegenResult<(String, String)> {
        // Check if this is an enum variant constructor (e.g., Some(42))
        if let Expr::Ident(name) = &func.node {
            if let Some((enum_name, tag)) = self.get_tuple_variant_info(name) {
                return self.generate_enum_variant_constructor(&enum_name, tag, args, counter);
            }

            // Check if this is a SIMD intrinsic - do this BEFORE regular function lookup
            if Self::is_simd_intrinsic(name) {
                return self.generate_simd_intrinsic(name, args, counter);
            }

            // sizeof(expr) — compile-time constant
            if name == "sizeof" {
                let size = if !args.is_empty() {
                    let arg_type = self.infer_expr_type(&args[0]);
                    self.compute_sizeof(&arg_type)
                } else {
                    8
                };
                return Ok((size.to_string(), String::new()));
            }

            // Handle print/println builtins with format string support
            if name == "print" || name == "println" {
                return self.generate_print_call(name, args, counter, span);
            }

            // Handle print_i64/print_f64 builtins
            if name == "print_i64" && args.len() == 1 {
                let has_user_fn = self
                    .functions
                    .get("print_i64")
                    .map(|f| !f.is_extern)
                    .unwrap_or(false);
                if !has_user_fn {
                    return self.generate_print_i64_builtin(args, counter);
                }
            }
            if name == "print_f64" && args.len() == 1 {
                let has_user_fn = self
                    .functions
                    .get("print_f64")
                    .map(|f| !f.is_extern)
                    .unwrap_or(false);
                if !has_user_fn {
                    return self.generate_print_f64_builtin(args, counter);
                }
            }

            // Handle format builtin
            if name == "format" {
                return self.generate_format_call(args, counter, span);
            }

            // Handle str_to_ptr builtin
            if name == "str_to_ptr" {
                return self.generate_str_to_ptr_builtin(args, counter);
            }

            // Handle ptr_to_str builtin
            if name == "ptr_to_str" {
                return self.generate_ptr_to_str_builtin(args, counter);
            }
        }

        // Check if this is a direct function call or indirect (lambda) call
        let (fn_name, is_indirect) = if let Expr::Ident(name) = &func.node {
            // Check if this is a generic function that needs monomorphization
            if let Some(instantiations_list) = self.generic_fn_instantiations.get(name) {
                let arg_types: Vec<vais_types::ResolvedType> =
                    args.iter().map(|a| self.infer_expr_type(a)).collect();
                let mangled = self.resolve_generic_call(name, &arg_types, instantiations_list);
                (mangled, false)
            } else if self.functions.contains_key(name) {
                (name.clone(), false)
            } else if self.locals.contains_key(name) {
                (name.clone(), true) // Lambda call
            } else {
                (name.clone(), false) // Assume it's a function
            }
        } else if let Expr::SelfCall = &func.node {
            (self.current_function.clone().unwrap_or_default(), false)
        } else {
            return Err(CodegenError::Unsupported(
                "complex indirect call".to_string(),
            ));
        };

        // Look up function info for parameter types
        let fn_info = if !is_indirect {
            self.functions.get(&fn_name).cloned()
        } else {
            None
        };

        let mut ir = String::new();
        let mut arg_vals = Vec::new();

        for (i, arg) in args.iter().enumerate() {
            let (mut val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);

            let param_ty = fn_info
                .as_ref()
                .and_then(|f| f.signature.params.get(i))
                .map(|(_, ty, _)| ty.clone());

            // Determine argument LLVM type - use parameter type if available, otherwise infer from expression
            let arg_ty = if let Some(ref pt) = param_ty {
                self.type_to_llvm(pt)
            } else {
                let inferred_ty = self.infer_expr_type(arg);
                self.type_to_llvm(&inferred_ty)
            };

            // Insert integer conversion if needed
            if let Some(param_type) = &param_ty {
                let src_bits = self.get_integer_bits_from_val(&val);
                let dst_bits = self.get_integer_bits(param_type);

                if src_bits > 0 && dst_bits > 0 && src_bits != dst_bits {
                    let conv_tmp = self.next_temp(counter);
                    let src_ty = format!("i{}", src_bits);
                    let dst_ty = format!("i{}", dst_bits);

                    if src_bits > dst_bits {
                        ir.push_str(&format!(
                            "  {} = trunc {} {} to {}\n",
                            conv_tmp, src_ty, val, dst_ty
                        ));
                    } else {
                        ir.push_str(&format!(
                            "  {} = sext {} {} to {}\n",
                            conv_tmp, src_ty, val, dst_ty
                        ));
                    }
                    val = conv_tmp;
                }
            }

            // For struct types, load the value from pointer if the expression produces a pointer
            // Struct literals return pointers (alloca), but function params expect values
            // This applies whether we have function info or not
            let type_to_check = param_ty
                .as_ref()
                .cloned()
                .unwrap_or_else(|| self.infer_expr_type(arg));
            let is_named = matches!(type_to_check, ResolvedType::Named { .. });
            let is_value = self.is_expr_value(arg);

            if is_named && !is_value {
                let loaded = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    loaded, arg_ty, arg_ty, val
                ));
                val = loaded;
            }

            arg_vals.push(format!("{} {}", arg_ty, val));
        }

        let ret_ty = fn_info
            .as_ref()
            .map(|f| self.type_to_llvm(&f.signature.ret))
            .unwrap_or_else(|| "i64".to_string());

        let actual_fn_name = fn_info
            .as_ref()
            .map(|f| f.signature.name.clone())
            .unwrap_or_else(|| fn_name.clone());

        // Generate the appropriate call based on function type
        self.generate_call_ir(
            &fn_name,
            &actual_fn_name,
            is_indirect,
            &ret_ty,
            &arg_vals,
            counter,
            span,
            &mut ir,
        )
    }

    /// Generate the IR for a function call
    #[allow(clippy::too_many_arguments)]
    fn generate_call_ir(
        &mut self,
        fn_name: &str,
        actual_fn_name: &str,
        is_indirect: bool,
        ret_ty: &str,
        arg_vals: &[String],
        counter: &mut usize,
        span: Span,
        ir: &mut String,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);

        if is_indirect {
            // Indirect call (lambda)
            let closure_info = self.closures.get(fn_name).cloned();

            let mut all_args = Vec::new();
            if let Some(ref info) = closure_info {
                for (_, capture_val) in &info.captures {
                    all_args.push(format!("i64 {}", capture_val));
                }
            }
            all_args.extend(arg_vals.iter().cloned());

            // If we have closure info, we know the exact function name - call directly
            if let Some(ref info) = closure_info {
                let tmp = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i64 @{}({}){}\n",
                    tmp,
                    info.func_name,
                    all_args.join(", "),
                    dbg_info
                ));
                return Ok((tmp, std::mem::take(ir)));
            }

            // Get the local variable info
            let local_info = self.locals.get(fn_name).cloned();
            let is_ssa_or_param = local_info
                .as_ref()
                .map(|l| l.is_ssa() || l.is_param())
                .unwrap_or(false);

            let ptr_tmp = if is_ssa_or_param {
                // SSA or param: the value IS the function pointer (as i64), no load needed
                let local = match local_info.as_ref() {
                    Some(l) => l,
                    None => {
                        return Err(CodegenError::TypeError(format!(
                            "missing local info for '{}'",
                            fn_name
                        )))
                    }
                };
                let val = &local.llvm_name;
                if local.is_ssa() {
                    // SSA values already include the % prefix (e.g., "%5")
                    val.clone()
                } else {
                    // Param names don't include % prefix
                    format!("%{}", val)
                }
            } else {
                let llvm_var_name = local_info
                    .as_ref()
                    .map(|l| l.llvm_name.clone())
                    .unwrap_or_else(|| fn_name.to_string());
                let tmp = self.next_temp(counter);
                ir.push_str(&format!("  {} = load i64, i64* %{}\n", tmp, llvm_var_name));
                tmp
            };

            let arg_types: Vec<String> = all_args
                .iter()
                .map(|a| a.split_whitespace().next().unwrap_or("i64").to_string())
                .collect();
            let fn_type = format!("i64 ({})*", arg_types.join(", "));

            let fn_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = inttoptr i64 {} to {}\n",
                fn_ptr, ptr_tmp, fn_type
            ));

            let tmp = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = call i64 {}({}){}\n",
                tmp,
                fn_ptr,
                all_args.join(", "),
                dbg_info
            ));
            Ok((tmp, std::mem::take(ir)))
        } else if fn_name == "malloc" {
            let ptr_tmp = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = call i8* @malloc({}){}\n",
                ptr_tmp,
                arg_vals.join(", "),
                dbg_info
            ));
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, ptr_tmp));
            Ok((result, std::mem::take(ir)))
        } else if fn_name == "free" {
            let ptr_tmp = self.next_temp(counter);
            let arg_val = arg_vals
                .first()
                .map(|s| s.split_whitespace().last().unwrap_or("0"))
                .unwrap_or("0");
            ir.push_str(&format!(
                "  {} = inttoptr i64 {} to i8*\n",
                ptr_tmp, arg_val
            ));
            ir.push_str(&format!("  call void @free(i8* {}){}\n", ptr_tmp, dbg_info));
            Ok(("void".to_string(), std::mem::take(ir)))
        } else if fn_name == "memcpy" || fn_name == "memcpy_str" {
            self.generate_memcpy_call(arg_vals, counter, span, ir)
        } else if fn_name == "strlen" {
            self.generate_strlen_call(arg_vals, counter, span, ir)
        } else if fn_name == "puts_ptr" {
            self.generate_puts_ptr_call(arg_vals, counter, span, ir)
        } else if ret_ty == "void" {
            let is_vararg = self
                .functions
                .get(fn_name)
                .map(|f| f.signature.is_vararg)
                .unwrap_or(false);
            if is_vararg {
                let param_types: Vec<String> = self
                    .functions
                    .get(fn_name)
                    .map(|f| {
                        f.signature
                            .params
                            .iter()
                            .map(|(_, ty, _)| self.type_to_llvm(ty))
                            .collect()
                    })
                    .unwrap_or_default();
                let sig = format!("void ({}, ...)", param_types.join(", "));
                ir.push_str(&format!(
                    "  call {} @{}({}){}\n",
                    sig,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            } else {
                ir.push_str(&format!(
                    "  call void @{}({}){}\n",
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            }
            Ok(("void".to_string(), std::mem::take(ir)))
        } else {
            let is_vararg = self
                .functions
                .get(fn_name)
                .map(|f| f.signature.is_vararg)
                .unwrap_or(false);
            let tmp = self.next_temp(counter);
            if is_vararg {
                let param_types: Vec<String> = self
                    .functions
                    .get(fn_name)
                    .map(|f| {
                        f.signature
                            .params
                            .iter()
                            .map(|(_, ty, _)| self.type_to_llvm(ty))
                            .collect()
                    })
                    .unwrap_or_default();
                let sig = format!("{} ({}, ...)", ret_ty, param_types.join(", "));
                ir.push_str(&format!(
                    "  {} = call {} @{}({}){}\n",
                    tmp,
                    sig,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            } else {
                ir.push_str(&format!(
                    "  {} = call {} @{}({}){}\n",
                    tmp,
                    ret_ty,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            }
            Ok((tmp, std::mem::take(ir)))
        }
    }

    /// Generate enum variant constructor
    fn generate_enum_variant_constructor(
        &mut self,
        enum_name: &str,
        tag: i32,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let mut arg_vals = Vec::new();

        for arg in args {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);
            arg_vals.push(val);
        }

        let enum_ptr = self.next_temp(counter);
        ir.push_str(&format!("  {} = alloca %{}\n", enum_ptr, enum_name));

        let tag_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0\n",
            tag_ptr, enum_name, enum_name, enum_ptr
        ));
        ir.push_str(&format!("  store i32 {}, i32* {}\n", tag, tag_ptr));

        // Store payload fields
        for (i, arg_val) in arg_vals.iter().enumerate() {
            let payload_field_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = getelementptr %{}, %{}* {}, i32 0, i32 1, i32 {}\n",
                payload_field_ptr, enum_name, enum_name, enum_ptr, i
            ));
            ir.push_str(&format!(
                "  store i64 {}, i64* {}\n",
                arg_val, payload_field_ptr
            ));
        }

        Ok((enum_ptr, ir))
    }

    /// Generate memcpy call
    fn generate_memcpy_call(
        &mut self,
        arg_vals: &[String],
        counter: &mut usize,
        span: Span,
        ir: &mut String,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let dest_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
        let src_full = arg_vals.get(1).map(|s| s.as_str()).unwrap_or("i64 0");
        let n_val = arg_vals
            .get(2)
            .map(|s| s.split_whitespace().last().unwrap_or("0"))
            .unwrap_or("0");

        // Handle dest pointer
        let dest_ptr = if dest_full.starts_with("i8*") {
            // Use everything after "i8* " to preserve complex expressions like getelementptr
            dest_full
                .strip_prefix("i8* ")
                .unwrap_or(dest_full.split_whitespace().last().unwrap_or("null"))
                .to_string()
        } else {
            let dest_val = dest_full.split_whitespace().last().unwrap_or("0");
            let ptr = self.next_temp(counter);
            ir.push_str(&format!("  {} = inttoptr i64 {} to i8*\n", ptr, dest_val));
            ptr
        };

        // Handle src pointer (can be i64 or i8* for memcpy_str)
        let src_ptr = if src_full.starts_with("i8*") {
            // Use everything after "i8* " to preserve complex expressions like getelementptr
            src_full
                .strip_prefix("i8* ")
                .unwrap_or(src_full.split_whitespace().last().unwrap_or("null"))
                .to_string()
        } else {
            let src_val = src_full.split_whitespace().last().unwrap_or("0");
            let ptr = self.next_temp(counter);
            ir.push_str(&format!("  {} = inttoptr i64 {} to i8*\n", ptr, src_val));
            ptr
        };

        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i8* @memcpy(i8* {}, i8* {}, i64 {}){}\n",
            result, dest_ptr, src_ptr, n_val, dbg_info
        ));
        let result_i64 = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = ptrtoint i8* {} to i64\n",
            result_i64, result
        ));
        Ok((result_i64, std::mem::take(ir)))
    }

    /// Generate strlen call
    fn generate_strlen_call(
        &mut self,
        arg_vals: &[String],
        counter: &mut usize,
        span: Span,
        ir: &mut String,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let arg_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
        let result = self.next_temp(counter);

        // Check if the argument is already i8* (str type) or i64 (pointer as integer)
        if arg_full.starts_with("i8*") {
            // Already a pointer, use directly
            let ptr_val = arg_full.split_whitespace().last().unwrap_or("null");
            ir.push_str(&format!(
                "  {} = call i64 @strlen(i8* {}){}\n",
                result, ptr_val, dbg_info
            ));
        } else {
            // Convert i64 to pointer
            let arg_val = arg_full.split_whitespace().last().unwrap_or("0");
            let ptr_tmp = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = inttoptr i64 {} to i8*\n",
                ptr_tmp, arg_val
            ));
            ir.push_str(&format!(
                "  {} = call i64 @strlen(i8* {}){}\n",
                result, ptr_tmp, dbg_info
            ));
        }
        Ok((result, std::mem::take(ir)))
    }

    /// Generate puts_ptr call
    fn generate_puts_ptr_call(
        &mut self,
        arg_vals: &[String],
        counter: &mut usize,
        span: Span,
        ir: &mut String,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let arg_val = arg_vals
            .first()
            .map(|s| s.split_whitespace().last().unwrap_or("0"))
            .unwrap_or("0");
        let ptr_tmp = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = inttoptr i64 {} to i8*\n",
            ptr_tmp, arg_val
        ));
        let i32_result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i32 @puts(i8* {}){}\n",
            i32_result, ptr_tmp, dbg_info
        ));
        // Convert i32 result to i64 for consistency
        let result = self.next_temp(counter);
        ir.push_str(&format!("  {} = sext i32 {} to i64\n", result, i32_result));
        Ok((result, std::mem::take(ir)))
    }

    /// Generate print/println call with format string support
    ///
    /// Converts Vais format strings like `print("x = {}", x)` to printf calls.
    /// `{}` placeholders are replaced with the appropriate C format specifier
    /// based on the inferred type of each argument.
    pub(crate) fn generate_print_call(
        &mut self,
        fn_name: &str,
        args: &[Spanned<Expr>],
        counter: &mut usize,
        span: Span,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let mut ir = String::new();

        if args.is_empty() {
            // print() with no args: do nothing
            return Ok(("void".to_string(), ir));
        }

        // If first arg is a StringInterp, flatten it into format string + args
        if let Expr::StringInterp(parts) = &args[0].node {
            let mut fmt_parts = Vec::new();
            let mut interp_args = Vec::new();
            for part in parts {
                match part {
                    vais_ast::StringInterpPart::Lit(s) => {
                        fmt_parts.push(s.clone());
                    }
                    vais_ast::StringInterpPart::Expr(e) => {
                        fmt_parts.push("{}".to_string());
                        interp_args.push(e.as_ref().clone());
                    }
                }
            }
            let fmt_string = fmt_parts.join("");
            let mut synthetic_args: Vec<Spanned<Expr>> = Vec::new();
            synthetic_args.push(Spanned::new(Expr::String(fmt_string), args[0].span));
            synthetic_args.extend(interp_args);
            // Also include any additional args after the interpolated string
            synthetic_args.extend_from_slice(&args[1..]);
            return self.generate_print_call(fn_name, &synthetic_args, counter, span);
        }

        // First argument must be a string literal (format string)
        let format_str = match &args[0].node {
            Expr::String(s) => s.clone(),
            _ => {
                // Non-literal first arg: treat as simple string output
                let (val, val_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&val_ir);
                if fn_name == "println" {
                    // For println with non-literal, use puts
                    let i32_result = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = call i32 @puts(i8* {}){}\n",
                        i32_result, val, dbg_info
                    ));
                } else {
                    // For print with non-literal, use printf with %s
                    let fmt_name = format!(".str.{}", self.string_counter);
                    self.string_counter += 1;
                    self.string_constants
                        .push((fmt_name.clone(), "%s".to_string()));
                    let fmt_len = 3; // "%s" + null
                    let fmt_ptr = format!(
                        "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                        fmt_len, fmt_len, fmt_name
                    );
                    let _result = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = call i32 (i8*, ...) @printf(i8* {}, i8* {}){}\n",
                        _result, fmt_ptr, val, dbg_info
                    ));
                }
                return Ok(("void".to_string(), ir));
            }
        };

        // Infer types of extra arguments (skip first format string arg)
        let extra_args = &args[1..];
        let mut arg_types: Vec<ResolvedType> = Vec::new();
        for arg in extra_args {
            arg_types.push(self.infer_expr_type(arg));
        }

        // Convert Vais format string `{}` to C printf format specifiers
        let mut c_format = String::new();
        let mut arg_idx = 0;
        let mut chars = format_str.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    // Escaped {{ -> literal {
                    chars.next();
                    c_format.push('{');
                } else if chars.peek() == Some(&'}') {
                    // {} -> format specifier based on type
                    chars.next();
                    if arg_idx < arg_types.len() {
                        let spec = match &arg_types[arg_idx] {
                            ResolvedType::I32 => "%d",
                            ResolvedType::I64 => "%ld",
                            ResolvedType::F32 | ResolvedType::F64 => "%f",
                            ResolvedType::Str => "%s",
                            ResolvedType::Bool => "%ld", // bools are i64 in codegen
                            // Char type not yet in Vais
                            _ => "%ld", // default to i64
                        };
                        c_format.push_str(spec);
                        arg_idx += 1;
                    } else {
                        return Err(CodegenError::TypeError(
                            "Too few arguments for format string".to_string(),
                        ));
                    }
                } else {
                    c_format.push(ch);
                }
            } else if ch == '}' {
                if chars.peek() == Some(&'}') {
                    // Escaped }} -> literal }
                    chars.next();
                    c_format.push('}');
                } else {
                    c_format.push(ch);
                }
            } else {
                c_format.push(ch);
            }
        }

        // For println, append newline
        if fn_name == "println" {
            c_format.push('\n');
        }

        // Create global string constant for the C format string
        let fmt_name = format!(".str.{}", self.string_counter);
        self.string_counter += 1;
        self.string_constants
            .push((fmt_name.clone(), c_format.clone()));
        let fmt_len = c_format.len() + 1; // +1 for null terminator
        let fmt_ptr = format!(
            "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
            fmt_len, fmt_len, fmt_name
        );

        // Evaluate extra arguments and build printf call
        let mut printf_args = vec![format!("i8* {}", fmt_ptr)];

        for (i, arg) in extra_args.iter().enumerate() {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);

            let llvm_ty = match &arg_types[i] {
                ResolvedType::I32 => "i32",
                ResolvedType::F32 => "float",
                ResolvedType::F64 => "double",
                ResolvedType::Str => "i8*",
                // Char type not yet in Vais
                _ => "i64", // i64, bool, etc.
            };

            // For i32 params, we may need to truncate from i64
            if llvm_ty == "i32" {
                let trunc_tmp = self.next_temp(counter);
                ir.push_str(&format!("  {} = trunc i64 {} to i32\n", trunc_tmp, val));
                printf_args.push(format!("i32 {}", trunc_tmp));
            } else {
                printf_args.push(format!("{} {}", llvm_ty, val));
            }
        }

        // If no extra args and println, use puts for efficiency
        if extra_args.is_empty() && fn_name == "println" {
            // Remove the printf format string (with \n) we added above
            self.string_constants.pop();
            self.string_counter -= 1;
            // Create puts string (without trailing \n, since puts adds one)
            let puts_str = &c_format[..c_format.len() - 1];
            let puts_name = format!(".str.{}", self.string_counter);
            self.string_counter += 1;
            self.string_constants
                .push((puts_name.clone(), puts_str.to_string()));
            let puts_len = puts_str.len() + 1;
            let puts_ptr = format!(
                "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                puts_len, puts_len, puts_name
            );
            let _result = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = call i32 @puts(i8* {}){}\n",
                _result, puts_ptr, dbg_info
            ));
            return Ok(("void".to_string(), ir));
        }

        // For print with no extra args, use printf with just the format string
        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i32 (i8*, ...) @printf({}){}\n",
            result,
            printf_args.join(", "),
            dbg_info
        ));

        Ok(("void".to_string(), ir))
    }

    /// Generate format() call - returns allocated formatted string (i8*)
    ///
    /// Uses snprintf(NULL, 0, fmt, ...) to measure, malloc to allocate,
    /// then snprintf(buf, len+1, fmt, ...) to write.
    pub(crate) fn generate_format_call(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
        span: Span,
    ) -> CodegenResult<(String, String)> {
        let _dbg_info = self.debug_info.dbg_ref_from_span(span);
        let mut ir = String::new();

        if args.is_empty() {
            return Err(CodegenError::TypeError(
                "format() requires at least a format string argument".to_string(),
            ));
        }

        // First argument must be a string literal (format string)
        let format_str = match &args[0].node {
            Expr::String(s) => s.clone(),
            _ => {
                // Non-literal: just return the string as-is
                let (val, val_ir) = self.generate_expr(&args[0], counter)?;
                return Ok((val, val_ir));
            }
        };

        // Infer types of extra arguments
        let extra_args = &args[1..];
        let mut arg_types: Vec<ResolvedType> = Vec::new();
        for arg in extra_args {
            arg_types.push(self.infer_expr_type(arg));
        }

        // Convert Vais format string `{}` to C printf format specifiers
        let mut c_format = String::new();
        let mut arg_idx = 0;
        let mut chars = format_str.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    c_format.push('{');
                } else if chars.peek() == Some(&'}') {
                    chars.next();
                    if arg_idx < arg_types.len() {
                        let spec = match &arg_types[arg_idx] {
                            ResolvedType::I32 => "%d",
                            ResolvedType::I64 => "%ld",
                            ResolvedType::F32 | ResolvedType::F64 => "%f",
                            ResolvedType::Str => "%s",
                            ResolvedType::Bool => "%ld",
                            _ => "%ld",
                        };
                        c_format.push_str(spec);
                        arg_idx += 1;
                    } else {
                        return Err(CodegenError::TypeError(
                            "Too few arguments for format string".to_string(),
                        ));
                    }
                } else {
                    c_format.push(ch);
                }
            } else if ch == '}' {
                if chars.peek() == Some(&'}') {
                    chars.next();
                    c_format.push('}');
                } else {
                    c_format.push(ch);
                }
            } else {
                c_format.push(ch);
            }
        }

        // Create global string constant for the C format string
        let fmt_name = format!(".str.{}", self.string_counter);
        self.string_counter += 1;
        self.string_constants
            .push((fmt_name.clone(), c_format.clone()));
        let fmt_len = c_format.len() + 1;
        let fmt_ptr = format!(
            "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
            fmt_len, fmt_len, fmt_name
        );

        // Evaluate extra arguments
        let mut snprintf_args = String::new();
        let mut arg_vals: Vec<String> = Vec::new();
        for (i, arg) in extra_args.iter().enumerate() {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);

            let llvm_ty = match &arg_types[i] {
                ResolvedType::I32 => "i32",
                ResolvedType::F32 => "float",
                ResolvedType::F64 => "double",
                ResolvedType::Str => "i8*",
                _ => "i64",
            };

            if llvm_ty == "i32" {
                let trunc_tmp = self.next_temp(counter);
                ir.push_str(&format!("  {} = trunc i64 {} to i32\n", trunc_tmp, val));
                arg_vals.push(format!("i32 {}", trunc_tmp));
            } else {
                arg_vals.push(format!("{} {}", llvm_ty, val));
            }
        }

        if !arg_vals.is_empty() {
            snprintf_args = format!(", {}", arg_vals.join(", "));
        }

        // If no extra args, just return the format string directly
        if extra_args.is_empty() {
            let str_name = format!(".str.{}", self.string_counter - 1);
            // Already pushed the constant above, reuse it
            let ptr = format!(
                "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                fmt_len, fmt_len, str_name
            );
            return Ok((ptr, ir));
        }

        // Step 1: snprintf(NULL, 0, fmt, ...) to get required length
        let len_i32 = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i32 (i8*, i64, i8*, ...) @snprintf(i8* null, i64 0, i8* {}{})\n",
            len_i32, fmt_ptr, snprintf_args
        ));

        // Convert i32 length to i64
        let len_i64 = self.next_temp(counter);
        ir.push_str(&format!("  {} = sext i32 {} to i64\n", len_i64, len_i32));

        // Step 2: malloc(len + 1)
        let buf_size = self.next_temp(counter);
        ir.push_str(&format!("  {} = add i64 {}, 1\n", buf_size, len_i64));

        let buf_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i8* @malloc(i64 {})\n",
            buf_ptr, buf_size
        ));

        // Step 3: snprintf(buf, len+1, fmt, ...)
        let _write_result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i32 (i8*, i64, i8*, ...) @snprintf(i8* {}, i64 {}, i8* {}{})\n",
            _write_result, buf_ptr, buf_size, fmt_ptr, snprintf_args
        ));

        // Mark that we need snprintf declaration
        self.needs_string_helpers = true;

        Ok((buf_ptr, ir))
    }

    /// Generate if expression
    pub(crate) fn generate_if_expr(
        &mut self,
        cond: &Spanned<Expr>,
        then: &[Spanned<Stmt>],
        else_: Option<&vais_ast::IfElse>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let then_label = self.next_label("then");
        let else_label = self.next_label("else");
        let merge_label = self.next_label("merge");

        let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
        let mut ir = cond_ir;

        let cond_bool = self.next_temp(counter);
        ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_bool, cond_val));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cond_bool, then_label, else_label
        ));

        // Then block
        ir.push_str(&format!("{}:\n", then_label));
        self.current_block.clone_from(&then_label);
        let (then_val, then_ir, then_terminated) = self.generate_block_stmts(then, counter)?;
        ir.push_str(&then_ir);
        let then_actual_block = self.current_block.clone();
        let then_from_label = if !then_terminated {
            ir.push_str(&format!("  br label %{}\n", merge_label));
            then_actual_block
        } else {
            String::new()
        };

        // Else block
        ir.push_str(&format!("{}:\n", else_label));
        self.current_block.clone_from(&else_label);
        let (else_val, else_ir, else_terminated, nested_last_block, has_else) =
            if let Some(else_branch) = else_ {
                let (v, i, t, last) =
                    self.generate_if_else_with_term(else_branch, counter, &merge_label)?;
                (v, i, t, last, true)
            } else {
                ("0".to_string(), String::new(), false, String::new(), false)
            };
        ir.push_str(&else_ir);
        let else_from_label = if !else_terminated {
            ir.push_str(&format!("  br label %{}\n", merge_label));
            if !nested_last_block.is_empty() {
                nested_last_block
            } else {
                self.current_block.clone()
            }
        } else {
            String::new()
        };

        // Merge block
        ir.push_str(&format!("{}:\n", merge_label));
        self.current_block.clone_from(&merge_label);
        let result = self.next_temp(counter);

        if !has_else {
            ir.push_str(&format!("  {} = add i64 0, 0\n", result));
        } else if !then_from_label.is_empty() && !else_from_label.is_empty() {
            ir.push_str(&format!(
                "  {} = phi i64 [ {}, %{} ], [ {}, %{} ]\n",
                result, then_val, then_from_label, else_val, else_from_label
            ));
        } else if !then_from_label.is_empty() {
            ir.push_str(&format!(
                "  {} = phi i64 [ {}, %{} ]\n",
                result, then_val, then_from_label
            ));
        } else if !else_from_label.is_empty() {
            ir.push_str(&format!(
                "  {} = phi i64 [ {}, %{} ]\n",
                result, else_val, else_from_label
            ));
        } else {
            ir.push_str(&format!("  {} = add i64 0, 0\n", result));
        }

        Ok((result, ir))
    }

    /// Generate loop expression
    pub(crate) fn generate_loop_expr(
        &mut self,
        iter: Option<&Spanned<Expr>>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let loop_start = self.next_label("loop.start");
        let loop_body = self.next_label("loop.body");
        let loop_end = self.next_label("loop.end");

        self.loop_stack.push(LoopLabels {
            continue_label: loop_start.to_string(),
            break_label: loop_end.to_string(),
        });

        let mut ir = String::new();

        if let Some(iter_expr) = iter {
            // Conditional loop
            ir.push_str(&format!("  br label %{}\n", loop_start));
            ir.push_str(&format!("{}:\n", loop_start));

            let (cond_val, cond_ir) = self.generate_expr(iter_expr, counter)?;
            ir.push_str(&cond_ir);

            let cond_bool = self.next_temp(counter);
            ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_bool, cond_val));
            ir.push_str(&format!(
                "  br i1 {}, label %{}, label %{}\n",
                cond_bool, loop_body, loop_end
            ));

            ir.push_str(&format!("{}:\n", loop_body));
            let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
            ir.push_str(&body_ir);
            if !body_terminated {
                ir.push_str(&format!("  br label %{}\n", loop_start));
            }
        } else {
            // Infinite loop
            ir.push_str(&format!("  br label %{}\n", loop_start));
            ir.push_str(&format!("{}:\n", loop_start));
            let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
            ir.push_str(&body_ir);
            if !body_terminated {
                ir.push_str(&format!("  br label %{}\n", loop_start));
            }
        }

        ir.push_str(&format!("{}:\n", loop_end));
        self.loop_stack.pop();

        Ok(("0".to_string(), ir))
    }

    /// Generate while loop expression
    pub(crate) fn generate_while_expr(
        &mut self,
        condition: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let loop_start = self.next_label("while.start");
        let loop_body = self.next_label("while.body");
        let loop_end = self.next_label("while.end");

        self.loop_stack.push(LoopLabels {
            continue_label: loop_start.to_string(),
            break_label: loop_end.to_string(),
        });

        let mut ir = String::new();

        // Jump to condition check
        ir.push_str(&format!("  br label %{}\n", loop_start));
        ir.push_str(&format!("{}:\n", loop_start));

        // Evaluate condition
        let (cond_val, cond_ir) = self.generate_expr(condition, counter)?;
        ir.push_str(&cond_ir);

        // Convert to i1 for branch
        let cond_bool = self.next_temp(counter);
        ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_bool, cond_val));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cond_bool, loop_body, loop_end
        ));

        // Loop body
        ir.push_str(&format!("{}:\n", loop_body));
        let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
        ir.push_str(&body_ir);

        // Jump back to condition if body doesn't terminate
        if !body_terminated {
            ir.push_str(&format!("  br label %{}\n", loop_start));
        }

        // Loop end
        ir.push_str(&format!("{}:\n", loop_end));
        self.loop_stack.pop();

        Ok(("0".to_string(), ir))
    }

    /// Generate cast expression
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
            if let Some(local) = self.locals.get(name).cloned() {
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
                if let Some(local) = self.locals.get(var_name.as_str()).cloned() {
                    if let ResolvedType::Named {
                        name: struct_name, ..
                    } = &local.ty
                    {
                        if let Some(struct_info) = self.structs.get(struct_name).cloned() {
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
            if let Some(local) = self.locals.get(name.as_str()).cloned() {
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

    /// Generate array literal expression
    pub(crate) fn generate_array_expr(
        &mut self,
        elements: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let len = elements.len();

        // Infer element type from first element (default to i64)
        let elem_ty = if let Some(first) = elements.first() {
            let resolved = self.infer_expr_type(first);
            self.type_to_llvm(&resolved)
        } else {
            "i64".to_string()
        };
        let arr_ty = format!("[{}  x {}]", len, elem_ty);

        let arr_ptr = self.next_temp(counter);
        ir.push_str(&format!("  {} = alloca {}\n", arr_ptr, arr_ty));

        for (i, elem) in elements.iter().enumerate() {
            let (val, elem_ir) = self.generate_expr(elem, counter)?;
            ir.push_str(&elem_ir);

            let elem_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = getelementptr {}, {}* {}, i64 0, i64 {}\n",
                elem_ptr, arr_ty, arr_ty, arr_ptr, i
            ));
            ir.push_str(&format!(
                "  store {} {}, {}* {}\n",
                elem_ty, val, elem_ty, elem_ptr
            ));
        }

        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {}, {}* {}, i64 0, i64 0\n",
            result, arr_ty, arr_ty, arr_ptr
        ));

        Ok((result, ir))
    }

    /// Generate tuple literal expression
    pub(crate) fn generate_tuple_expr(
        &mut self,
        elements: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let len = elements.len();

        let tuple_ty = format!("{{ {} }}", vec!["i64"; len].join(", "));

        let tuple_ptr = self.next_temp(counter);
        ir.push_str(&format!("  {} = alloca {}\n", tuple_ptr, tuple_ty));

        for (i, elem) in elements.iter().enumerate() {
            let (val, elem_ir) = self.generate_expr(elem, counter)?;
            ir.push_str(&elem_ir);

            let elem_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = getelementptr {}, {}* {}, i32 0, i32 {}\n",
                elem_ptr, tuple_ty, tuple_ty, tuple_ptr, i
            ));
            ir.push_str(&format!("  store i64 {}, i64* {}\n", val, elem_ptr));
        }

        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load {}, {}* {}\n",
            result, tuple_ty, tuple_ty, tuple_ptr
        ));

        Ok((result, ir))
    }

    /// Generate struct or union literal expression
    pub(crate) fn generate_struct_lit_expr(
        &mut self,
        name: &Spanned<String>,
        fields: &[(Spanned<String>, Spanned<Expr>)],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let type_name = &name.node;

        let resolved_name = self.resolve_struct_name(type_name);
        let type_name = &resolved_name;

        // First check if it's a struct
        if let Some(struct_info) = self.structs.get(type_name).cloned() {
            let mut ir = String::new();

            // Check if this struct has generic parameters
            // Collect generic parameters from struct fields
            let mut generic_params = Vec::new();
            for (_, field_ty) in &struct_info.fields {
                if let ResolvedType::Generic(param) = field_ty {
                    if !generic_params.contains(param) {
                        generic_params.push(param.clone());
                    }
                }
            }

            // If the struct is generic, infer concrete types from the field values
            let final_type_name = if !generic_params.is_empty() {
                let mut inferred_types = Vec::new();

                // For each generic parameter, find the first field that uses it and infer from the value
                for param in &generic_params {
                    let mut inferred = None;
                    for (field_name, field_expr) in fields {
                        // Find the field info
                        if let Some((_, ResolvedType::Generic(p))) = struct_info
                            .fields
                            .iter()
                            .find(|(name, _)| name == &field_name.node)
                        {
                            if p == param {
                                inferred = Some(self.infer_expr_type(field_expr));
                                break;
                            }
                        }
                    }
                    inferred_types.push(inferred.unwrap_or(ResolvedType::I64));
                }

                // Generate the mangled name with inferred types
                self.mangle_struct_name(type_name, &inferred_types)
            } else {
                type_name.to_string()
            };

            let struct_ptr = self.next_temp(counter);
            ir.push_str(&format!("  {} = alloca %{}\n", struct_ptr, final_type_name));

            for (field_name, field_expr) in fields {
                let field_idx = struct_info
                    .fields
                    .iter()
                    .position(|(n, _)| n == &field_name.node)
                    .ok_or_else(|| {
                        CodegenError::TypeError(format!(
                            "Unknown field '{}' in struct '{}'",
                            field_name.node, type_name
                        ))
                    })?;

                let (val, field_ir) = self.generate_expr(field_expr, counter)?;
                ir.push_str(&field_ir);

                let field_ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                    field_ptr, final_type_name, final_type_name, struct_ptr, field_idx
                ));

                let field_ty = &struct_info.fields[field_idx].1;
                let llvm_ty = self.type_to_llvm(field_ty);

                // For struct-typed fields, val might be a pointer that needs to be loaded
                let val_to_store = if matches!(field_ty, ResolvedType::Named { .. })
                    && !self.is_expr_value(field_expr)
                {
                    // Field value is a pointer to struct, need to load the value
                    let loaded = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        loaded, llvm_ty, llvm_ty, val
                    ));
                    loaded
                } else {
                    val
                };

                ir.push_str(&format!(
                    "  store {} {}, {}* {}\n",
                    llvm_ty, val_to_store, llvm_ty, field_ptr
                ));
            }

            Ok((struct_ptr, ir))
        // Then check if it's a union
        } else if let Some(union_info) = self.unions.get(type_name).cloned() {
            let mut ir = String::new();

            // Allocate union on stack
            let union_ptr = self.next_temp(counter);
            ir.push_str(&format!("  {} = alloca %{}\n", union_ptr, type_name));

            // Union should have exactly one field in the literal
            if fields.len() != 1 {
                return Err(CodegenError::TypeError(format!(
                    "Union literal should have exactly one field, got {}",
                    fields.len()
                )));
            }

            let (field_name, field_expr) = &fields[0];

            // Find field type
            let field_ty = union_info
                .fields
                .iter()
                .find(|(n, _)| n == &field_name.node)
                .map(|(_, ty)| ty.clone())
                .ok_or_else(|| {
                    CodegenError::TypeError(format!(
                        "Unknown field '{}' in union '{}'",
                        field_name.node, type_name
                    ))
                })?;

            let (val, field_ir) = self.generate_expr(field_expr, counter)?;
            ir.push_str(&field_ir);

            // Bitcast union pointer to field type pointer (all fields at offset 0)
            let field_llvm_ty = self.type_to_llvm(&field_ty);
            let field_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = bitcast %{}* {} to {}*\n",
                field_ptr, type_name, union_ptr, field_llvm_ty
            ));

            // Store the value
            ir.push_str(&format!(
                "  store {} {}, {}* {}\n",
                field_llvm_ty, val, field_llvm_ty, field_ptr
            ));

            Ok((union_ptr, ir))
        } else {
            Err(CodegenError::TypeError(format!(
                "Unknown struct or union: {}",
                type_name
            )))
        }
    }

    /// Generate index expression
    pub(crate) fn generate_index_expr(
        &mut self,
        array: &Spanned<Expr>,
        index: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (arr_val, arr_ir) = self.generate_expr(array, counter)?;
        let (idx_val, idx_ir) = self.generate_expr(index, counter)?;

        let mut ir = arr_ir;
        ir.push_str(&idx_ir);

        // Infer element type for correct LLVM IR generation
        let arr_ty = self.infer_expr_type(array);
        let elem_llvm_ty = match arr_ty {
            vais_types::ResolvedType::Pointer(ref elem) => self.type_to_llvm(elem),
            vais_types::ResolvedType::Array(ref elem) => self.type_to_llvm(elem),
            _ => "i64".to_string(),
        };

        let elem_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {}, {}* {}, i64 {}\n",
            elem_ptr, elem_llvm_ty, elem_llvm_ty, arr_val, idx_val
        ));

        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load {}, {}* {}\n",
            result, elem_llvm_ty, elem_llvm_ty, elem_ptr
        ));

        Ok((result, ir))
    }

    /// Generate field access expression
    pub(crate) fn generate_field_expr(
        &mut self,
        obj: &Spanned<Expr>,
        field: &Spanned<String>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (obj_val, obj_ir) = self.generate_expr(obj, counter)?;
        let mut ir = obj_ir;

        if let Expr::Ident(var_name) = &obj.node {
            if let Some(local) = self.locals.get(var_name.as_str()).cloned() {
                if let ResolvedType::Named {
                    name: orig_type_name,
                    ..
                } = &local.ty
                {
                    let type_name = &self.resolve_struct_name(orig_type_name);
                    // First check if it's a struct
                    if let Some(struct_info) = self.structs.get(type_name).cloned() {
                        let field_idx = struct_info
                            .fields
                            .iter()
                            .position(|(n, _)| n == &field.node)
                            .ok_or_else(|| {
                                CodegenError::TypeError(format!(
                                    "Unknown field '{}' in struct '{}'",
                                    field.node, type_name
                                ))
                            })?;

                        let field_ty = &struct_info.fields[field_idx].1;
                        let llvm_ty = self.type_to_llvm(field_ty);

                        let field_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                            field_ptr, type_name, type_name, obj_val, field_idx
                        ));

                        let result = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = load {}, {}* {}\n",
                            result, llvm_ty, llvm_ty, field_ptr
                        ));

                        return Ok((result, ir));
                    }
                    // Then check if it's a union
                    else if let Some(union_info) = self.unions.get(type_name).cloned() {
                        let field_ty = union_info
                            .fields
                            .iter()
                            .find(|(n, _)| n == &field.node)
                            .map(|(_, ty)| ty.clone())
                            .ok_or_else(|| {
                                CodegenError::TypeError(format!(
                                    "Unknown field '{}' in union '{}'",
                                    field.node, type_name
                                ))
                            })?;

                        let llvm_ty = self.type_to_llvm(&field_ty);

                        // For union field access, bitcast union pointer to field type pointer
                        // All fields share offset 0
                        let field_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = bitcast %{}* {} to {}*\n",
                            field_ptr, type_name, obj_val, llvm_ty
                        ));

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

        Err(CodegenError::Unsupported(
            "field access requires known struct or union type".to_string(),
        ))
    }

    /// Generate method call expression
    pub(crate) fn generate_method_call_expr(
        &mut self,
        receiver: &Spanned<Expr>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (recv_val, recv_ir, recv_type) = if matches!(&receiver.node, Expr::SelfCall) {
            if let Some(local) = self.locals.get("self") {
                let recv_type = local.ty.clone();
                ("%self".to_string(), String::new(), recv_type)
            } else {
                return Err(CodegenError::Unsupported(
                    "@.method() used outside of a method with self".to_string(),
                ));
            }
        } else {
            let (recv_val, recv_ir) = self.generate_expr(receiver, counter)?;
            let recv_type = self.infer_expr_type(receiver);
            (recv_val, recv_ir, recv_type)
        };
        let mut ir = recv_ir;

        let method_name = &method.node;

        // String method calls: str.len(), str.charAt(), str.contains(), etc.
        if matches!(recv_type, ResolvedType::Str) {
            return self.generate_string_method_call(&recv_val, &ir, method_name, args, counter);
        }

        // Use resolve_struct_name to match definition naming (e.g., Pair → Pair$i64)
        // For non-generic structs, this is a no-op (Vec → Vec)
        let full_method_name = if let ResolvedType::Named { name, .. } = &recv_type {
            let resolved = self.resolve_struct_name(name);
            format!("{}_{}", resolved, method_name)
        } else {
            method_name.clone()
        };

        let recv_llvm_ty = if matches!(&recv_type, ResolvedType::Named { .. }) {
            format!("{}*", self.type_to_llvm(&recv_type))
        } else {
            self.type_to_llvm(&recv_type)
        };

        let mut arg_vals = vec![format!("{} {}", recv_llvm_ty, recv_val)];

        for arg in args {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);
            let arg_type = self.infer_expr_type(arg);
            let arg_llvm_ty = self.type_to_llvm(&arg_type);
            arg_vals.push(format!("{} {}", arg_llvm_ty, val));
        }

        let ret_type = if let ResolvedType::Named { name, .. } = &recv_type {
            if let Some(_struct_info) = self.structs.get(name) {
                "i64"
            } else {
                "i64"
            }
        } else {
            "i64"
        };

        let tmp = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call {} @{}({})\n",
            tmp,
            ret_type,
            full_method_name,
            arg_vals.join(", ")
        ));

        Ok((tmp, ir))
    }

    /// Generate static method call expression
    pub(crate) fn generate_static_method_call_expr(
        &mut self,
        type_name: &Spanned<String>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();

        let full_method_name = format!("{}_{}", type_name.node, method.node);

        let mut arg_vals = Vec::new();
        for arg in args {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);
            let arg_type = self.infer_expr_type(arg);
            let arg_llvm_ty = self.type_to_llvm(&arg_type);
            arg_vals.push(format!("{} {}", arg_llvm_ty, val));
        }

        let ret_type = self
            .functions
            .get(&full_method_name)
            .map(|info| self.type_to_llvm(&info.signature.ret))
            .unwrap_or_else(|| "i64".to_string());

        let tmp = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call {} @{}({})\n",
            tmp,
            ret_type,
            full_method_name,
            arg_vals.join(", ")
        ));

        Ok((tmp, ir))
    }

    /// Generate await expression
    pub(crate) fn generate_await_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (future_ptr, future_ir) = self.generate_expr(inner, counter)?;
        let mut ir = future_ir;

        fn get_poll_func_name(expr: &Expr) -> String {
            match expr {
                Expr::Call { func, .. } => {
                    if let Expr::Ident(name) = &func.node {
                        format!("{}__poll", name)
                    } else {
                        "__async_poll".to_string()
                    }
                }
                Expr::MethodCall { method, .. } => {
                    format!("{}__poll", method.node)
                }
                Expr::Spawn(inner) => get_poll_func_name(&inner.node),
                _ => "__async_poll".to_string(),
            }
        }
        let poll_func = get_poll_func_name(&inner.node);

        let poll_start = self.next_label("await_poll");
        let poll_ready = self.next_label("await_ready");
        let poll_pending = self.next_label("await_pending");

        ir.push_str(&format!("  br label %{}\n\n", poll_start));
        ir.push_str(&format!("{}:\n", poll_start));

        let poll_result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call {{ i64, i64 }} @{}(i64 {})\n",
            poll_result, poll_func, future_ptr
        ));

        let status = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = extractvalue {{ i64, i64 }} {}, 0\n",
            status, poll_result
        ));

        let is_ready = self.next_temp(counter);
        ir.push_str(&format!("  {} = icmp eq i64 {}, 1\n", is_ready, status));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n\n",
            is_ready, poll_ready, poll_pending
        ));

        ir.push_str(&format!("{}:\n", poll_pending));
        // Yield CPU cooperatively instead of busy-waiting
        ir.push_str("  call i32 @sched_yield()\n");
        ir.push_str(&format!("  br label %{}\n\n", poll_start));

        ir.push_str(&format!("{}:\n", poll_ready));
        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = extractvalue {{ i64, i64 }} {}, 1\n",
            result, poll_result
        ));

        Ok((result, ir))
    }

    /// Generate lambda expression
    pub(crate) fn generate_lambda_expr(
        &mut self,
        params: &[Param],
        body: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let lambda_name = format!("__lambda_{}", self.label_counter);
        self.label_counter += 1;

        let capture_names = self.find_lambda_captures(params, body);

        let mut captured_vars: Vec<(String, ResolvedType, String)> = Vec::new();
        let mut capture_ir = String::new();

        for cap_name in &capture_names {
            if let Some(local) = self.locals.get(cap_name) {
                let ty = local.ty.clone();
                if local.is_param() {
                    captured_vars.push((cap_name.clone(), ty, format!("%{}", local.llvm_name)));
                } else if local.is_ssa() {
                    // SSA values are already the value itself, use directly
                    captured_vars.push((cap_name.clone(), ty, local.llvm_name.clone()));
                } else {
                    let tmp = self.next_temp(counter);
                    let llvm_ty = self.type_to_llvm(&ty);
                    capture_ir.push_str(&format!(
                        "  {} = load {}, {}* %{}\n",
                        tmp, llvm_ty, llvm_ty, local.llvm_name
                    ));
                    captured_vars.push((cap_name.clone(), ty, tmp));
                }
            }
        }

        let mut param_strs = Vec::new();
        let mut param_types = Vec::new();

        for (cap_name, cap_ty, _) in &captured_vars {
            let llvm_ty = self.type_to_llvm(cap_ty);
            param_strs.push(format!("{} %__cap_{}", llvm_ty, cap_name));
            param_types.push(llvm_ty);
        }

        for p in params {
            let ty = self.ast_type_to_resolved(&p.ty.node);
            let llvm_ty = self.type_to_llvm(&ty);
            param_strs.push(format!("{} %{}", llvm_ty, p.name.node));
            param_types.push(llvm_ty);
        }

        // SAFETY: if generate_expr below returns Err, the entire codegen aborts,
        // so empty self.locals after take is acceptable (never accessed post-error).
        let saved_function = self.current_function.take();
        let saved_locals = std::mem::take(&mut self.locals);

        self.current_function = Some(lambda_name.clone());

        for (cap_name, cap_ty, _) in &captured_vars {
            self.locals.insert(
                cap_name.clone(),
                LocalVar::param(cap_ty.clone(), format!("__cap_{}", cap_name)),
            );
        }

        for p in params {
            let ty = self.ast_type_to_resolved(&p.ty.node);
            self.locals.insert(
                p.name.node.clone(),
                LocalVar::param(ty, p.name.node.clone()),
            );
        }

        let mut lambda_counter = 0;
        let (body_val, body_ir) = self.generate_expr(body, &mut lambda_counter)?;

        let mut lambda_ir = format!(
            "define i64 @{}({}) {{\nentry:\n",
            lambda_name,
            param_strs.join(", ")
        );
        lambda_ir.push_str(&body_ir);
        lambda_ir.push_str(&format!("  ret i64 {}\n}}\n", body_val));

        self.lambda_functions.push(lambda_ir);

        self.current_function = saved_function;
        self.locals = saved_locals;

        // Emit ptrtoint as a proper instruction (not a constant expression)
        // so the result is a clean SSA temp that can be used anywhere
        let fn_ptr_tmp = self.next_temp(counter);
        capture_ir.push_str(&format!(
            "  {} = ptrtoint i64 ({})* @{} to i64\n",
            fn_ptr_tmp,
            param_types.join(", "),
            lambda_name
        ));

        if captured_vars.is_empty() {
            self.last_lambda_info = None;
            Ok((fn_ptr_tmp, capture_ir))
        } else {
            self.last_lambda_info = Some(ClosureInfo {
                func_name: lambda_name.clone(),
                captures: captured_vars
                    .iter()
                    .map(|(name, _, val)| (name.clone(), val.clone()))
                    .collect(),
            });
            Ok((fn_ptr_tmp, capture_ir))
        }
    }

    /// Generate try expression
    pub(crate) fn generate_try_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;
        let mut ir = inner_ir;

        let _tag_tmp = self.next_temp(counter);
        let result_ptr = self.next_temp(counter);
        let tag_ptr = self.next_temp(counter);
        let tag = self.next_temp(counter);

        ir.push_str("  ; Try expression\n");
        ir.push_str(&format!(
            "  {} = inttoptr i64 {} to {{i64, i64}}*\n",
            result_ptr, inner_val
        ));
        ir.push_str(&format!(
            "  {} = getelementptr {{i64, i64}}, {{i64, i64}}* {}, i32 0, i32 0\n",
            tag_ptr, result_ptr
        ));
        ir.push_str(&format!("  {} = load i64, i64* {}\n", tag, tag_ptr));

        let is_err = self.next_temp(counter);
        let err_label = self.next_label("try_err");
        let ok_label = self.next_label("try_ok");
        let merge_label = self.next_label("try_merge");

        ir.push_str(&format!("  {} = icmp eq i64 {}, 1\n", is_err, tag));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n\n",
            is_err, err_label, ok_label
        ));

        ir.push_str(&format!("{}:\n", err_label));
        ir.push_str(&format!(
            "  ret i64 {}  ; early return on Err\n\n",
            inner_val
        ));

        ir.push_str(&format!("{}:\n", ok_label));
        let value_ptr = self.next_temp(counter);
        let value = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {{i64, i64}}, {{i64, i64}}* {}, i32 0, i32 1\n",
            value_ptr, result_ptr
        ));
        ir.push_str(&format!("  {} = load i64, i64* {}\n", value, value_ptr));
        ir.push_str(&format!("  br label %{}\n\n", merge_label));

        ir.push_str(&format!("{}:\n", merge_label));

        Ok((value, ir))
    }

    /// Generate unwrap expression
    pub(crate) fn generate_unwrap_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;
        let mut ir = inner_ir;

        let result_ptr = self.next_temp(counter);
        let tag_ptr = self.next_temp(counter);
        let tag = self.next_temp(counter);

        ir.push_str("  ; Unwrap expression\n");
        ir.push_str(&format!(
            "  {} = inttoptr i64 {} to {{i64, i64}}*\n",
            result_ptr, inner_val
        ));
        ir.push_str(&format!(
            "  {} = getelementptr {{i64, i64}}, {{i64, i64}}* {}, i32 0, i32 0\n",
            tag_ptr, result_ptr
        ));
        ir.push_str(&format!("  {} = load i64, i64* {}\n", tag, tag_ptr));

        let is_err = self.next_temp(counter);
        let err_label = self.next_label("unwrap_err");
        let ok_label = self.next_label("unwrap_ok");

        ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", is_err, tag));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n\n",
            is_err, err_label, ok_label
        ));

        ir.push_str(&format!("{}:\n", err_label));
        ir.push_str("  call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.unwrap_panic_msg, i64 0, i64 0))\n");
        ir.push_str("  call void @abort()\n");
        ir.push_str("  unreachable\n\n");

        ir.push_str(&format!("{}:\n", ok_label));
        let value_ptr = self.next_temp(counter);
        let value = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {{i64, i64}}, {{i64, i64}}* {}, i32 0, i32 1\n",
            value_ptr, result_ptr
        ));
        ir.push_str(&format!("  {} = load i64, i64* {}\n", value, value_ptr));

        self.needs_unwrap_panic = true;

        Ok((value, ir))
    }

    // === SIMD Intrinsic Support ===

    /// Check if a function name is a SIMD intrinsic
    pub(crate) fn is_simd_intrinsic(name: &str) -> bool {
        name.starts_with("vec")
            && (name.ends_with("f32")
                || name.ends_with("f64")
                || name.ends_with("i32")
                || name.ends_with("i64"))
            || name.starts_with("simd_")
    }

    /// Generate SIMD intrinsic call
    pub(crate) fn generate_simd_intrinsic(
        &mut self,
        fn_name: &str,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let mut arg_vals = Vec::new();

        // Evaluate all arguments first
        for arg in args {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);
            arg_vals.push(val);
        }

        // Handle vector constructors
        if fn_name.starts_with("vec") && !fn_name.starts_with("vec_") {
            return self.generate_vector_constructor(fn_name, &arg_vals, counter, ir);
        }

        // Handle SIMD binary operations
        if fn_name.starts_with("simd_add_")
            || fn_name.starts_with("simd_sub_")
            || fn_name.starts_with("simd_mul_")
            || fn_name.starts_with("simd_div_")
        {
            return self.generate_simd_binop(fn_name, &arg_vals, counter, ir);
        }

        // Handle SIMD reduce operations
        if fn_name.starts_with("simd_reduce_add_") {
            return self.generate_simd_reduce_add(fn_name, &arg_vals, counter, ir);
        }

        Err(CodegenError::Unsupported(format!(
            "Unknown SIMD intrinsic: {}",
            fn_name
        )))
    }

    /// Generate vector constructor (e.g., vec4f32(x, y, z, w))
    fn generate_vector_constructor(
        &mut self,
        fn_name: &str,
        arg_vals: &[String],
        counter: &mut usize,
        mut ir: String,
    ) -> CodegenResult<(String, String)> {
        // Parse vector type from name (e.g., "vec4f32" -> lanes=4, element="float")
        let (lanes, elem_ty) = self.parse_vector_type_name(fn_name)?;

        // Build vector using insertelement instructions
        // Start with undef and insert each element
        let vec_ty = format!("<{} x {}>", lanes, elem_ty);
        let mut current_vec = "undef".to_string();

        for (i, val) in arg_vals.iter().enumerate() {
            let next_vec = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = insertelement {} {}, {} {}, i32 {}\n",
                next_vec, vec_ty, current_vec, elem_ty, val, i
            ));
            current_vec = next_vec;
        }

        Ok((current_vec, ir))
    }

    /// Generate SIMD binary operation (add, sub, mul, div)
    fn generate_simd_binop(
        &mut self,
        fn_name: &str,
        arg_vals: &[String],
        counter: &mut usize,
        mut ir: String,
    ) -> CodegenResult<(String, String)> {
        if arg_vals.len() != 2 {
            return Err(CodegenError::TypeError(format!(
                "SIMD binary operation {} requires 2 arguments",
                fn_name
            )));
        }

        // Parse operation and type from name (e.g., "simd_add_vec4f32")
        let (op, vec_suffix) = if let Some(suffix) = fn_name.strip_prefix("simd_add_") {
            ("add", suffix)
        } else if let Some(suffix) = fn_name.strip_prefix("simd_sub_") {
            ("sub", suffix)
        } else if let Some(suffix) = fn_name.strip_prefix("simd_mul_") {
            ("mul", suffix)
        } else if let Some(suffix) = fn_name.strip_prefix("simd_div_") {
            ("div", suffix)
        } else {
            return Err(CodegenError::Unsupported(format!(
                "Unknown SIMD op: {}",
                fn_name
            )));
        };

        let (lanes, elem_ty) = self.parse_vector_type_name(vec_suffix)?;
        let vec_ty = format!("<{} x {}>", lanes, elem_ty);

        // Determine LLVM instruction based on element type
        let llvm_op = match (op, elem_ty.as_str()) {
            ("add", "float") | ("add", "double") => "fadd",
            ("sub", "float") | ("sub", "double") => "fsub",
            ("mul", "float") | ("mul", "double") => "fmul",
            ("div", "float") | ("div", "double") => "fdiv",
            ("add", _) => "add",
            ("sub", _) => "sub",
            ("mul", _) => "mul",
            ("div", _) => "sdiv",
            _ => return Err(CodegenError::Unsupported(format!("Unknown op: {}", op))),
        };

        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = {} {} {}, {}\n",
            result, llvm_op, vec_ty, arg_vals[0], arg_vals[1]
        ));

        Ok((result, ir))
    }

    /// Generate SIMD reduce add operation
    fn generate_simd_reduce_add(
        &mut self,
        fn_name: &str,
        arg_vals: &[String],
        counter: &mut usize,
        mut ir: String,
    ) -> CodegenResult<(String, String)> {
        if arg_vals.len() != 1 {
            return Err(CodegenError::TypeError(format!(
                "SIMD reduce operation {} requires 1 argument",
                fn_name
            )));
        }

        // Parse type from name (e.g., "simd_reduce_add_vec4f32")
        let vec_suffix = &fn_name[16..]; // Skip "simd_reduce_add_"
        let (lanes, elem_ty) = self.parse_vector_type_name(vec_suffix)?;

        // Use LLVM vector reduce intrinsics
        let intrinsic = match elem_ty.as_str() {
            "float" => format!("@llvm.vector.reduce.fadd.v{}f32", lanes),
            "double" => format!("@llvm.vector.reduce.fadd.v{}f64", lanes),
            "i32" => format!("@llvm.vector.reduce.add.v{}i32", lanes),
            "i64" => format!("@llvm.vector.reduce.add.v{}i64", lanes),
            _ => {
                return Err(CodegenError::Unsupported(format!(
                    "Unknown element type: {}",
                    elem_ty
                )))
            }
        };

        let vec_ty = format!("<{} x {}>", lanes, elem_ty);
        let result = self.next_temp(counter);

        // For float/double, we need an initial value for ordered reduction
        if elem_ty == "float" || elem_ty == "double" {
            let zero = "0.0";
            ir.push_str(&format!(
                "  {} = call {} {}({} {}, {} {})\n",
                result, elem_ty, intrinsic, elem_ty, zero, vec_ty, arg_vals[0]
            ));
        } else {
            ir.push_str(&format!(
                "  {} = call {} {}({} {})\n",
                result, elem_ty, intrinsic, vec_ty, arg_vals[0]
            ));
        }

        Ok((result, ir))
    }

    /// Parse vector type name to get lanes and element type
    fn parse_vector_type_name(&self, name: &str) -> CodegenResult<(u32, String)> {
        // e.g., "vec4f32" -> (4, "float"), "vec2i64" -> (2, "i64")
        let (lanes, elem) = if let Some(rest) = name.strip_prefix("vec") {
            // Remove "vec" prefix
            if let Some(lanes_str) = rest.strip_suffix("f32") {
                (lanes_str.parse::<u32>().unwrap_or(4), "float".to_string())
            } else if let Some(lanes_str) = rest.strip_suffix("f64") {
                (lanes_str.parse::<u32>().unwrap_or(2), "double".to_string())
            } else if let Some(lanes_str) = rest.strip_suffix("i32") {
                (lanes_str.parse::<u32>().unwrap_or(4), "i32".to_string())
            } else if let Some(lanes_str) = rest.strip_suffix("i64") {
                (lanes_str.parse::<u32>().unwrap_or(2), "i64".to_string())
            } else {
                return Err(CodegenError::Unsupported(format!(
                    "Unknown vector type: {}",
                    name
                )));
            }
        } else {
            return Err(CodegenError::Unsupported(format!(
                "Invalid vector type name: {}",
                name
            )));
        };

        Ok((lanes, elem))
    }

    /// Generate print_i64 builtin call
    pub(crate) fn generate_print_i64_builtin(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (arg_val, arg_ir) = self.generate_expr(&args[0], counter)?;
        let mut ir = arg_ir;
        let fmt_str = "%ld";
        let fmt_name = self.make_string_name();
        self.string_counter += 1;
        self.string_constants
            .push((fmt_name.clone(), fmt_str.to_string()));
        let fmt_len = fmt_str.len() + 1;
        let fmt_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n",
            fmt_ptr, fmt_len, fmt_len, fmt_name
        ));
        let i32_result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i32 (i8*, ...) @printf(i8* {}, i64 {})\n",
            i32_result, fmt_ptr, arg_val
        ));
        let result = self.next_temp(counter);
        ir.push_str(&format!("  {} = sext i32 {} to i64\n", result, i32_result));
        Ok((result, ir))
    }

    /// Generate print_f64 builtin call
    pub(crate) fn generate_print_f64_builtin(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (arg_val, arg_ir) = self.generate_expr(&args[0], counter)?;
        let mut ir = arg_ir;
        let fmt_str = "%f";
        let fmt_name = self.make_string_name();
        self.string_counter += 1;
        self.string_constants
            .push((fmt_name.clone(), fmt_str.to_string()));
        let fmt_len = fmt_str.len() + 1;
        let fmt_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n",
            fmt_ptr, fmt_len, fmt_len, fmt_name
        ));
        let i32_result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i32 (i8*, ...) @printf(i8* {}, double {})\n",
            i32_result, fmt_ptr, arg_val
        ));
        let result = self.next_temp(counter);
        ir.push_str(&format!("  {} = sext i32 {} to i64\n", result, i32_result));
        Ok((result, ir))
    }

    /// Generate str_to_ptr builtin call
    fn generate_str_to_ptr_builtin(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        if args.len() != 1 {
            return Err(CodegenError::TypeError(
                "str_to_ptr expects 1 argument".to_string(),
            ));
        }
        let (str_val, str_ir) = self.generate_expr(&args[0], counter)?;
        let mut ir = str_ir;
        let result = self.next_temp(counter);
        ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, str_val));
        Ok((result, ir))
    }

    /// Generate ptr_to_str builtin call
    fn generate_ptr_to_str_builtin(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        if args.len() != 1 {
            return Err(CodegenError::TypeError(
                "ptr_to_str expects 1 argument".to_string(),
            ));
        }
        let (ptr_val, ptr_ir) = self.generate_expr(&args[0], counter)?;
        let mut ir = ptr_ir;
        let arg_type = self.infer_expr_type(&args[0]);
        if matches!(arg_type, vais_types::ResolvedType::I64) {
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = inttoptr i64 {} to i8*\n", result, ptr_val));
            return Ok((result, ir));
        }
        // Already a pointer type, no conversion needed
        Ok((ptr_val, ir))
    }
}
