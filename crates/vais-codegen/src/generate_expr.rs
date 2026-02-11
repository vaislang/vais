//! Expression code generation for LLVM IR.

use vais_ast::*;
use vais_types::ResolvedType;

use crate::{
    CodeGenerator, CodegenResult, CodegenError, LocalVar, LoopLabels, ClosureInfo,
    suggest_similar, format_did_you_mean,
};

impl CodeGenerator {
    pub(crate) fn generate_range_for_loop(
        &mut self,
        pattern: &Spanned<Pattern>,
        iter: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (start_expr, end_expr, inclusive) = match &iter.node {
            Expr::Range {
                start,
                end,
                inclusive,
            } => (start.as_deref(), end.as_deref(), *inclusive),
            _ => unreachable!("generate_range_for_loop called with non-range iter"),
        };

        let mut ir = String::new();

        let (start_val, start_ir) = if let Some(s) = start_expr {
            self.generate_expr(s, counter)?
        } else {
            ("0".to_string(), String::new())
        };
        ir.push_str(&start_ir);

        let (end_val, end_ir) = if let Some(e) = end_expr {
            self.generate_expr(e, counter)?
        } else {
            (format!("{}", i64::MAX), String::new())
        };
        ir.push_str(&end_ir);

        let counter_var = format!("%loop_counter.{}", self.label_counter);
        self.label_counter += 1;
        ir.push_str(&format!("  {} = alloca i64\n", counter_var));
        ir.push_str(&format!(
            "  store i64 {}, i64* {}\n",
            start_val, counter_var
        ));

        let pattern_var = if let Pattern::Ident(name) = &pattern.node {
            let var_name = format!("{}.for", name);
            let llvm_name = format!("%{}", var_name);
            ir.push_str(&format!("  {} = alloca i64\n", llvm_name));
            self.locals.insert(
                name.clone(),
                LocalVar::alloca(ResolvedType::I64, var_name.clone()),
            );
            Some((name.clone(), llvm_name))
        } else {
            None
        };

        let loop_cond = self.next_label("for.cond");
        let loop_body_label = self.next_label("for.body");
        let loop_inc = self.next_label("for.inc");
        let loop_end = self.next_label("for.end");

        self.loop_stack.push(LoopLabels {
            continue_label: loop_inc.clone(),
            break_label: loop_end.clone(),
        });

        ir.push_str(&format!("  br label %{}\n", loop_cond));

        ir.push_str(&format!("{}:\n", loop_cond));
        let current_val = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load i64, i64* {}\n",
            current_val, counter_var
        ));

        let cmp_pred = if inclusive { "sle" } else { "slt" };
        let cond_result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = icmp {} i64 {}, {}\n",
            cond_result, cmp_pred, current_val, end_val
        ));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cond_result, loop_body_label, loop_end
        ));

        ir.push_str(&format!("{}:\n", loop_body_label));

        if let Some((_, llvm_name)) = &pattern_var {
            let bind_val = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = load i64, i64* {}\n",
                bind_val, counter_var
            ));
            ir.push_str(&format!("  store i64 {}, i64* {}\n", bind_val, llvm_name));
        }

        let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
        ir.push_str(&body_ir);

        if !body_terminated {
            ir.push_str(&format!("  br label %{}\n", loop_inc));
        }

        ir.push_str(&format!("{}:\n", loop_inc));
        let inc_load = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load i64, i64* {}\n",
            inc_load, counter_var
        ));
        let inc_result = self.next_temp(counter);
        ir.push_str(&format!("  {} = add i64 {}, 1\n", inc_result, inc_load));
        ir.push_str(&format!(
            "  store i64 {}, i64* {}\n",
            inc_result, counter_var
        ));
        ir.push_str(&format!("  br label %{}\n", loop_cond));

        ir.push_str(&format!("{}:\n", loop_end));
        self.loop_stack.pop();

        Ok(("0".to_string(), ir))
    }

    pub(crate) fn generate_expr(
        &mut self,
        expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &expr.node {
            Expr::Int(n) => Ok((n.to_string(), String::new())),
            Expr::Float(n) => Ok((crate::types::format_llvm_float(*n), String::new())),
            Expr::Bool(b) => Ok((if *b { "1" } else { "0" }.to_string(), String::new())),
            Expr::String(s) => {
                // Create a global string constant
                let name = self.make_string_name();
                self.string_counter += 1;
                self.string_constants.push((name.clone(), s.clone()));

                // Return a getelementptr to the string constant
                let len = s.len() + 1;
                Ok((
                    format!(
                        "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                        len, len, name
                    ),
                    String::new(),
                ))
            }
            Expr::StringInterp(parts) => {
                // Desugar string interpolation into a format() call.
                // Build a format string with {} placeholders and collect expression args.
                let mut format_str_parts = Vec::new();
                let mut interp_args = Vec::new();
                for part in parts {
                    match part {
                        vais_ast::StringInterpPart::Lit(s) => {
                            format_str_parts.push(s.clone());
                        }
                        vais_ast::StringInterpPart::Expr(e) => {
                            format_str_parts.push("{}".to_string());
                            interp_args.push(e.as_ref().clone());
                        }
                    }
                }
                let fmt_string = format_str_parts.join("");
                // Build synthetic args: format string + expression args
                let mut args: Vec<Spanned<Expr>> = Vec::new();
                args.push(Spanned::new(Expr::String(fmt_string), expr.span));
                args.extend(interp_args);
                self.generate_format_call(&args, counter, expr.span)
            }
            Expr::Unit => Ok(("void".to_string(), String::new())),

            Expr::Ident(name) => {
                if let Some(local) = self.locals.get(name.as_str()).cloned() {
                    if local.is_param() {
                        // Parameters are SSA values, use directly
                        Ok((format!("%{}", local.llvm_name), String::new()))
                    } else if local.is_ssa() {
                        // SSA variables: use the stored value directly, no load needed
                        Ok((local.llvm_name.clone(), String::new()))
                    } else if matches!(local.ty, ResolvedType::Named { .. }) {
                        // Struct variables store a pointer to the struct
                        // Load the pointer (the struct address)
                        let tmp = self.next_temp(counter);
                        let llvm_ty = self.type_to_llvm(&local.ty);
                        let ir = format!(
                            "  {} = load {}*, {}** %{}\n",
                            tmp, llvm_ty, llvm_ty, local.llvm_name
                        );
                        Ok((tmp, ir))
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
                    for enum_info in self.enums.values() {
                        for (tag, variant) in enum_info.variants.iter().enumerate() {
                            if variant.name == *name {
                                let mut ir = String::new();
                                let enum_ptr = self.next_temp(counter);
                                ir.push_str(&format!(
                                    "  {} = alloca %{}\n",
                                    enum_ptr, enum_info.name
                                ));
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
                } else if let Some(const_info) = self.constants.get(name).cloned() {
                    // Constant reference - inline the constant value
                    self.generate_expr(&const_info.value, counter)
                } else if self.functions.contains_key(name.as_str()) {
                    // Function reference
                    Ok((format!("@{}", name), String::new()))
                } else if let Some(self_local) = self.locals.get("self").cloned() {
                    // Implicit self: check if name is a field of the self struct
                    let self_type = match &self_local.ty {
                        ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                            inner.as_ref().clone()
                        }
                        other => other.clone(),
                    };
                    if let ResolvedType::Named {
                        name: type_name, ..
                    } = &self_type
                    {
                        let resolved_name = self.resolve_struct_name(type_name);
                        if let Some(struct_info) = self.structs.get(&resolved_name).cloned() {
                            if let Some(field_idx) =
                                struct_info.fields.iter().position(|(n, _)| n == name)
                            {
                                let field_ty = &struct_info.fields[field_idx].1;
                                let llvm_ty = self.type_to_llvm(field_ty);
                                let mut ir = String::new();
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
                    let suggestions = suggest_similar(name, &candidates, 3);
                    let suggestion_text = format_did_you_mean(&suggestions);
                    Err(CodegenError::UndefinedVar(format!(
                        "{}{}",
                        name, suggestion_text
                    )))
                }
            }

            Expr::SelfCall => {
                // @ refers to current function
                if let Some(fn_name) = &self.current_function {
                    Ok((format!("@{}", fn_name), String::new()))
                } else {
                    Err(CodegenError::UndefinedFunction("@".to_string()))
                }
            }

            Expr::Binary { op, left, right } => {
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
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
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
                    let is_float_cmp = matches!(left_type, ResolvedType::F64)
                        || matches!(right_type, ResolvedType::F64);

                    let cmp_tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);

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
                    let is_float = matches!(left_type, ResolvedType::F64)
                        || matches!(right_type, ResolvedType::F64);

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

                        let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
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

                        let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                        ir.push_str(&format!(
                            "  {} = {} i64 {}, {}{}\n",
                            tmp, op_str, left_val, right_val, dbg_info
                        ));
                    }
                    Ok((tmp, ir))
                }
            }

            Expr::Unary { op, expr: inner } => {
                let (val, val_ir) = self.generate_expr(inner, counter)?;
                let tmp = self.next_temp(counter);

                let mut ir = val_ir;
                let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
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

            Expr::Ternary { cond, then, else_ } => {
                // Use proper branching for lazy evaluation
                let then_label = self.next_label("ternary.then");
                let else_label = self.next_label("ternary.else");
                let merge_label = self.next_label("ternary.merge");

                // Generate condition
                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let mut ir = cond_ir;

                // Convert to i1 for branch (type-aware: skips for bool/i1)
                let (cond_bool, conv_ir) = self.generate_cond_to_i1(cond, &cond_val, counter);
                ir.push_str(&conv_ir);

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

            Expr::Call { func, args } => {
                // Check if this is an enum variant constructor or builtin
                if let Expr::Ident(name) = &func.node {
                    // Struct tuple literal: `Response(200, 1)` → desugar to StructLit
                    let resolved = self.resolve_struct_name(name);
                    if self.structs.contains_key(&resolved) && !self.functions.contains_key(name) {
                        if let Some(struct_info) = self.structs.get(&resolved).cloned() {
                            let fields: Vec<_> = struct_info
                                .fields
                                .iter()
                                .zip(args.iter())
                                .map(|((fname, _), val)| {
                                    (
                                        vais_ast::Spanned::new(fname.clone(), val.span),
                                        val.clone(),
                                    )
                                })
                                .collect();
                            let struct_lit = vais_ast::Spanned::new(
                                Expr::StructLit {
                                    name: vais_ast::Spanned::new(name.clone(), func.span),
                                    fields,
                                },
                                expr.span,
                            );
                            return self.generate_expr(&struct_lit, counter);
                        }
                    }

                    // Handle print/println builtins with format string support
                    if name == "print" || name == "println" {
                        return self.generate_print_call(name, args, counter, expr.span);
                    }

                    // Handle format builtin: returns formatted string
                    if name == "format" {
                        return self.generate_format_call(args, counter, expr.span);
                    }

                    // Handle str_to_ptr builtin: convert string pointer to i64
                    if name == "str_to_ptr" {
                        if args.len() != 1 {
                            return Err(CodegenError::TypeError(
                                "str_to_ptr expects 1 argument".to_string(),
                            ));
                        }
                        let (str_val, str_ir) = self.generate_expr(&args[0], counter)?;
                        let mut ir = str_ir;
                        let result = self.next_temp(counter);
                        ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, str_val));
                        return Ok((result, ir));
                    }

                    // Handle ptr_to_str builtin: convert i64 to string pointer (i8*)
                    // If value is already a pointer type, pass through directly
                    if name == "ptr_to_str" {
                        if args.len() != 1 {
                            return Err(CodegenError::TypeError(
                                "ptr_to_str expects 1 argument".to_string(),
                            ));
                        }
                        let (ptr_val, ptr_ir) = self.generate_expr(&args[0], counter)?;
                        let mut ir = ptr_ir;
                        // Only do inttoptr if the arg is actually an integer (i64)
                        // If it's already a pointer (str, malloc result, etc.), pass through
                        let arg_type = self.infer_expr_type(&args[0]);
                        if matches!(arg_type, vais_types::ResolvedType::I64) {
                            let result = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = inttoptr i64 {} to i8*\n",
                                result, ptr_val
                            ));
                            return Ok((result, ir));
                        }
                        // Already a pointer/str, no conversion needed
                        return Ok((ptr_val, ir));
                    }

                    // sizeof(expr) — compile-time constant size query
                    // Also supports sizeof(T) where T is a generic type parameter
                    if name == "sizeof" && !args.is_empty() {
                        let arg_type = if let Expr::Ident(ident) = &args[0].node {
                            if let Some(concrete) = self.get_generic_substitution(ident) {
                                concrete
                            } else {
                                self.infer_expr_type(&args[0])
                            }
                        } else {
                            self.infer_expr_type(&args[0])
                        };
                        let size = self.compute_sizeof(&arg_type);
                        return Ok((size.to_string(), String::new()));
                    }

                    // alignof(expr) — compile-time constant alignment query
                    // Also supports alignof(T) where T is a generic type parameter
                    if name == "alignof" && !args.is_empty() {
                        let arg_type = if let Expr::Ident(ident) = &args[0].node {
                            if let Some(concrete) = self.get_generic_substitution(ident) {
                                concrete
                            } else {
                                self.infer_expr_type(&args[0])
                            }
                        } else {
                            self.infer_expr_type(&args[0])
                        };
                        let align = self.compute_alignof(&arg_type);
                        return Ok((align.to_string(), String::new()));
                    }

                    // type_size() — compile-time size of current generic type T
                    // Returns sizeof(T) where T is resolved from generic_substitutions
                    // Used in generic containers like Vec<T> to get element size
                    if name == "type_size" && args.is_empty() {
                        let resolved_t = self
                            .get_generic_substitution("T")
                            .unwrap_or(ResolvedType::I64);
                        let size = self.compute_sizeof(&resolved_t);
                        return Ok((size.to_string(), String::new()));
                    }

                    // load_typed(ptr) -> T — type-aware memory load
                    // Dispatches to correct LLVM load based on generic type T
                    if name == "load_typed" && !args.is_empty() {
                        let (ptr_val, ptr_ir) = self.generate_expr(&args[0], counter)?;
                        let mut ir = ptr_ir;
                        // Resolve T from generic substitutions
                        let resolved_t = self
                            .get_generic_substitution("T")
                            .unwrap_or(ResolvedType::I64);
                        let size = self.compute_sizeof(&resolved_t);
                        let result = self.next_temp(counter);
                        match size {
                            1 => {
                                let tmp1 = self.next_temp(counter);
                                let tmp2 = self.next_temp(counter);
                                ir.push_str(&format!("  {} = inttoptr i64 {} to i8*\n", tmp1, ptr_val));
                                ir.push_str(&format!("  {} = load i8, i8* {}\n", tmp2, tmp1));
                                ir.push_str(&format!("  {} = zext i8 {} to i64\n", result, tmp2));
                            }
                            2 => {
                                let tmp1 = self.next_temp(counter);
                                let tmp2 = self.next_temp(counter);
                                ir.push_str(&format!("  {} = inttoptr i64 {} to i16*\n", tmp1, ptr_val));
                                ir.push_str(&format!("  {} = load i16, i16* {}\n", tmp2, tmp1));
                                ir.push_str(&format!("  {} = zext i16 {} to i64\n", result, tmp2));
                            }
                            4 if matches!(resolved_t, ResolvedType::F32) => {
                                let tmp1 = self.next_temp(counter);
                                let tmp2 = self.next_temp(counter);
                                let tmp3 = self.next_temp(counter);
                                ir.push_str(&format!("  {} = inttoptr i64 {} to float*\n", tmp1, ptr_val));
                                ir.push_str(&format!("  {} = load float, float* {}\n", tmp2, tmp1));
                                ir.push_str(&format!("  {} = fpext float {} to double\n", tmp3, tmp2));
                                ir.push_str(&format!("  {} = bitcast double {} to i64\n", result, tmp3));
                            }
                            4 => {
                                let tmp1 = self.next_temp(counter);
                                let tmp2 = self.next_temp(counter);
                                ir.push_str(&format!("  {} = inttoptr i64 {} to i32*\n", tmp1, ptr_val));
                                ir.push_str(&format!("  {} = load i32, i32* {}\n", tmp2, tmp1));
                                ir.push_str(&format!("  {} = zext i32 {} to i64\n", result, tmp2));
                            }
                            _ if matches!(resolved_t, ResolvedType::F64) => {
                                let tmp1 = self.next_temp(counter);
                                let tmp2 = self.next_temp(counter);
                                ir.push_str(&format!("  {} = inttoptr i64 {} to double*\n", tmp1, ptr_val));
                                ir.push_str(&format!("  {} = load double, double* {}\n", tmp2, tmp1));
                                ir.push_str(&format!("  {} = bitcast double {} to i64\n", result, tmp2));
                            }
                            _ => {
                                let tmp1 = self.next_temp(counter);
                                ir.push_str(&format!("  {} = inttoptr i64 {} to i64*\n", tmp1, ptr_val));
                                ir.push_str(&format!("  {} = load i64, i64* {}\n", result, tmp1));
                            }
                        }
                        return Ok((result, ir));
                    }

                    // store_typed(ptr, val) — type-aware memory store
                    // Dispatches to correct LLVM store based on generic type T
                    if name == "store_typed" && args.len() >= 2 {
                        let (ptr_val, ptr_ir) = self.generate_expr(&args[0], counter)?;
                        let (val_val, val_ir) = self.generate_expr(&args[1], counter)?;
                        let mut ir = ptr_ir;
                        ir.push_str(&val_ir);
                        let resolved_t = self
                            .get_generic_substitution("T")
                            .unwrap_or(ResolvedType::I64);
                        let size = self.compute_sizeof(&resolved_t);
                        match size {
                            1 => {
                                let tmp1 = self.next_temp(counter);
                                let tmp2 = self.next_temp(counter);
                                ir.push_str(&format!("  {} = inttoptr i64 {} to i8*\n", tmp1, ptr_val));
                                ir.push_str(&format!("  {} = trunc i64 {} to i8\n", tmp2, val_val));
                                ir.push_str(&format!("  store i8 {}, i8* {}\n", tmp2, tmp1));
                            }
                            2 => {
                                let tmp1 = self.next_temp(counter);
                                let tmp2 = self.next_temp(counter);
                                ir.push_str(&format!("  {} = inttoptr i64 {} to i16*\n", tmp1, ptr_val));
                                ir.push_str(&format!("  {} = trunc i64 {} to i16\n", tmp2, val_val));
                                ir.push_str(&format!("  store i16 {}, i16* {}\n", tmp2, tmp1));
                            }
                            4 if matches!(resolved_t, ResolvedType::F32) => {
                                let tmp1 = self.next_temp(counter);
                                let tmp2 = self.next_temp(counter);
                                let tmp3 = self.next_temp(counter);
                                ir.push_str(&format!("  {} = inttoptr i64 {} to float*\n", tmp1, ptr_val));
                                ir.push_str(&format!("  {} = bitcast i64 {} to double\n", tmp2, val_val));
                                ir.push_str(&format!("  {} = fptrunc double {} to float\n", tmp3, tmp2));
                                ir.push_str(&format!("  store float {}, float* {}\n", tmp3, tmp1));
                            }
                            4 => {
                                let tmp1 = self.next_temp(counter);
                                let tmp2 = self.next_temp(counter);
                                ir.push_str(&format!("  {} = inttoptr i64 {} to i32*\n", tmp1, ptr_val));
                                ir.push_str(&format!("  {} = trunc i64 {} to i32\n", tmp2, val_val));
                                ir.push_str(&format!("  store i32 {}, i32* {}\n", tmp2, tmp1));
                            }
                            _ if matches!(resolved_t, ResolvedType::F64) => {
                                let tmp1 = self.next_temp(counter);
                                let tmp2 = self.next_temp(counter);
                                ir.push_str(&format!("  {} = inttoptr i64 {} to double*\n", tmp1, ptr_val));
                                ir.push_str(&format!("  {} = bitcast i64 {} to double\n", tmp2, val_val));
                                ir.push_str(&format!("  store double {}, double* {}\n", tmp2, tmp1));
                            }
                            _ => {
                                let tmp1 = self.next_temp(counter);
                                ir.push_str(&format!("  {} = inttoptr i64 {} to i64*\n", tmp1, ptr_val));
                                ir.push_str(&format!("  store i64 {}, i64* {}\n", val_val, tmp1));
                            }
                        }
                        return Ok(("0".to_string(), ir));
                    }

                    if let Some((enum_name, tag)) = self.get_tuple_variant_info(name) {
                        // This is a tuple enum variant constructor
                        let mut ir = String::new();

                        // Generate argument values
                        let mut arg_vals = Vec::new();
                        for arg in args {
                            let (val, arg_ir) = self.generate_expr(arg, counter)?;
                            ir.push_str(&arg_ir);
                            arg_vals.push(val);
                        }

                        // Create enum value on stack: { i32 tag, i64 payload }
                        let enum_ptr = self.next_temp(counter);
                        ir.push_str(&format!("  {} = alloca %{}\n", enum_ptr, enum_name));

                        // Store tag
                        let tag_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0\n",
                            tag_ptr, enum_name, enum_name, enum_ptr
                        ));
                        ir.push_str(&format!("  store i32 {}, i32* {}\n", tag, tag_ptr));

                        // Store payload fields into the payload sub-struct
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

                        // Return pointer to the enum
                        return Ok((enum_ptr, ir));
                    }

                    // Check if this is a SIMD intrinsic
                    if Self::is_simd_intrinsic(name) {
                        return self.generate_simd_intrinsic(name, args, counter);
                    }

                    // Handle print_i64/print_f64 builtins: emit printf call
                    // Skip if user defined their own function with the same name
                    let has_user_print_i64 = self
                        .functions
                        .get("print_i64")
                        .map(|f| !f.is_extern)
                        .unwrap_or(false);
                    if name == "print_i64" && args.len() == 1 && !has_user_print_i64 {
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
                        return Ok((result, ir));
                    }

                    let has_user_print_f64 = self
                        .functions
                        .get("print_f64")
                        .map(|f| !f.is_extern)
                        .unwrap_or(false);
                    if name == "print_f64" && args.len() == 1 && !has_user_print_f64 {
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
                        return Ok((result, ir));
                    }
                }

                // Check if this is a direct function call or indirect (lambda) call
                let (fn_name, is_indirect) = if let Expr::Ident(name) = &func.node {
                    // Check if this is a generic function that needs monomorphization
                    if let Some(instantiations_list) = self.generic_fn_instantiations.get(name) {
                        // Infer argument types to select the right specialization
                        let arg_types: Vec<ResolvedType> =
                            args.iter().map(|a| self.infer_expr_type(a)).collect();

                        // Find the matching instantiation based on argument types
                        let mangled =
                            self.resolve_generic_call(name, &arg_types, instantiations_list);
                        (mangled, false)
                    } else if self.functions.contains_key(name) {
                        (name.clone(), false)
                    } else if self.locals.contains_key(name) {
                        (name.clone(), true) // Lambda call
                    } else if self.declared_functions.contains(name) {
                        // Function declared in module (may be generic, will instantiate later)
                        (name.clone(), false)
                    } else {
                        // Unknown function - provide suggestions
                        let mut candidates: Vec<&str> = Vec::new();

                        // Add declared function names (including generics)
                        for func_name in &self.declared_functions {
                            candidates.push(func_name.as_str());
                        }

                        // Add instantiated function names
                        for func_name in self.functions.keys() {
                            candidates.push(func_name.as_str());
                        }

                        // Add local variables (could be lambdas)
                        for var_name in self.locals.keys() {
                            candidates.push(var_name.as_str());
                        }

                        let suggestions = suggest_similar(name, &candidates, 3);
                        let suggestion_text = format_did_you_mean(&suggestions);
                        return Err(CodegenError::UndefinedFunction(format!(
                            "{}{}",
                            name, suggestion_text
                        )));
                    }
                } else if let Expr::SelfCall = &func.node {
                    (self.current_function.clone().unwrap_or_default(), false)
                } else {
                    return Err(CodegenError::Unsupported(
                        "complex indirect call".to_string(),
                    ));
                };

                // Look up function info for parameter types (only for direct calls)
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

                    // Get parameter type from function info if available
                    let param_ty = fn_info
                        .as_ref()
                        .and_then(|f| f.signature.params.get(i))
                        .map(|(_, ty, _)| ty.clone());

                    let arg_ty = if let Some(ref ty) = param_ty {
                        self.type_to_llvm(ty)
                    } else {
                        // For vararg arguments, infer the type from the expression
                        let inferred_ty = self.infer_expr_type(arg);
                        self.type_to_llvm(&inferred_ty)
                    };

                    // For struct arguments, load the value if we have a pointer
                    // (struct literals generate alloca+stores, returning pointers)
                    if let Some(ResolvedType::Named { .. }) = &param_ty {
                        // Check if val looks like a pointer (starts with %)
                        if val.starts_with('%') {
                            let loaded = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = load {}, {}* {}\n",
                                loaded, arg_ty, arg_ty, val
                            ));
                            val = loaded;
                        }
                    }

                    // Trait object coercion: &ConcreteType -> &dyn Trait
                    // When parameter expects &dyn Trait and argument is a concrete type reference,
                    // create a fat pointer { data_ptr, vtable_ptr }
                    if let Some(ref param_type) = param_ty {
                        let dyn_trait = match param_type {
                            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                                if let ResolvedType::DynTrait { trait_name, .. } = inner.as_ref() {
                                    Some(trait_name.clone())
                                } else {
                                    None
                                }
                            }
                            ResolvedType::DynTrait { trait_name, .. } => Some(trait_name.clone()),
                            _ => None,
                        };

                        if let Some(trait_name) = dyn_trait {
                            // Get the concrete type of the argument
                            let arg_expr_type = self.infer_expr_type(arg);
                            let concrete_type_name = match &arg_expr_type {
                                ResolvedType::Named { name, .. } => Some(name.clone()),
                                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                                    if let ResolvedType::Named { name, .. } = inner.as_ref() {
                                        Some(name.clone())
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };

                            if let Some(concrete_name) = concrete_type_name {
                                // Generate vtable for this concrete type + trait
                                let vtable_info =
                                    self.get_or_generate_vtable(&concrete_name, &trait_name);

                                if let Some(vtable) = vtable_info {
                                    // Load the actual struct pointer if we have a pointer-to-pointer
                                    // (Ref expressions return the address of the storage, not the struct)
                                    let struct_ptr = self.next_temp(counter);
                                    ir.push_str(&format!(
                                        "  {} = load %{}*, %{}** {}\n",
                                        struct_ptr, concrete_name, concrete_name, val
                                    ));
                                    // Cast data pointer to i8*
                                    let data_ptr = self.next_temp(counter);
                                    ir.push_str(&format!(
                                        "  {} = bitcast %{}* {} to i8*\n",
                                        data_ptr, concrete_name, struct_ptr
                                    ));

                                    // Create fat pointer { i8*, i8* }
                                    let trait_obj_1 = self.next_temp(counter);
                                    ir.push_str(&format!(
                                        "  {} = insertvalue {{ i8*, i8* }} undef, i8* {}, 0\n",
                                        trait_obj_1, data_ptr
                                    ));
                                    let vtable_cast = self.next_temp(counter);
                                    ir.push_str(&format!(
                                        "  {} = bitcast {{ i8*, i64, i64, i64(i8*)* }}* {} to i8*\n",
                                        vtable_cast, vtable.global_name
                                    ));
                                    let trait_obj_2 = self.next_temp(counter);
                                    ir.push_str(&format!(
                                        "  {} = insertvalue {{ i8*, i8* }} {}, i8* {}, 1\n",
                                        trait_obj_2, trait_obj_1, vtable_cast
                                    ));

                                    val = trait_obj_2;
                                }
                            }
                        }
                    }

                    // Insert integer conversion if needed (trunc for narrowing, sext for widening)
                    if let Some(param_type) = &param_ty {
                        let src_bits = self.get_integer_bits_from_val(&val);
                        let dst_bits = self.get_integer_bits(param_type);

                        if src_bits > 0 && dst_bits > 0 && src_bits != dst_bits {
                            let conv_tmp = self.next_temp(counter);
                            let src_ty = format!("i{}", src_bits);
                            let dst_ty = format!("i{}", dst_bits);

                            if src_bits > dst_bits {
                                // Truncate
                                ir.push_str(&format!(
                                    "  {} = trunc {} {} to {}\n",
                                    conv_tmp, src_ty, val, dst_ty
                                ));
                            } else {
                                // Sign extend
                                ir.push_str(&format!(
                                    "  {} = sext {} {} to {}\n",
                                    conv_tmp, src_ty, val, dst_ty
                                ));
                            }
                            val = conv_tmp;
                        }
                    }

                    // Convert i64 to i8* when parameter expects str/i8* but arg is i64
                    if let Some(ref param_type) = param_ty {
                        if matches!(param_type, ResolvedType::Str) {
                            let actual_ty = self.infer_expr_type(arg);
                            if matches!(actual_ty, ResolvedType::I64) {
                                let ptr_tmp = self.next_temp(counter);
                                ir.push_str(&format!(
                                    "  {} = inttoptr i64 {} to i8*\n",
                                    ptr_tmp, val
                                ));
                                val = ptr_tmp;
                            }
                        }
                    }

                    arg_vals.push(format!("{} {}", arg_ty, val));
                }

                // Get return type and actual function name (may differ for builtins)
                let ret_ty = fn_info
                    .as_ref()
                    .map(|f| self.type_to_llvm(&f.signature.ret))
                    .unwrap_or_else(|| "i64".to_string());

                let actual_fn_name = fn_info
                    .as_ref()
                    .map(|f| f.signature.name.clone())
                    .unwrap_or_else(|| fn_name.clone());

                let is_vararg = fn_info
                    .as_ref()
                    .map(|f| f.signature.is_vararg)
                    .unwrap_or(false);

                if is_indirect {
                    // Check if this is a closure with captured variables
                    let closure_info = self.closures.get(&fn_name).cloned();

                    // Prepend captured values to arguments if this is a closure
                    let mut all_args = Vec::new();
                    if let Some(ref info) = closure_info {
                        for (_, capture_val) in &info.captures {
                            all_args.push(format!("i64 {}", capture_val));
                        }
                    }
                    all_args.extend(arg_vals);

                    // If we have closure info, we know the exact function name - call directly
                    if let Some(ref info) = closure_info {
                        let tmp = self.next_temp(counter);
                        let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                        ir.push_str(&format!(
                            "  {} = call i64 @{}({}){}\n",
                            tmp,
                            info.func_name,
                            all_args.join(", "),
                            dbg_info
                        ));
                        return Ok((tmp, ir));
                    }

                    // Get the local variable info
                    let local_info = self.locals.get(&fn_name).cloned();
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
                        // Alloca: load the function pointer from the stack slot
                        let llvm_var_name = local_info
                            .as_ref()
                            .map(|l| l.llvm_name.clone())
                            .unwrap_or_else(|| fn_name.clone());
                        let tmp = self.next_temp(counter);
                        ir.push_str(&format!("  {} = load i64, i64* %{}\n", tmp, llvm_var_name));
                        tmp
                    };

                    // Build function type signature for indirect call (including captures)
                    let arg_types: Vec<String> = all_args
                        .iter()
                        .map(|a| a.split_whitespace().next().unwrap_or("i64").to_string())
                        .collect();
                    let fn_type = format!("i64 ({})*", arg_types.join(", "));

                    // Cast i64 to function pointer
                    let fn_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = inttoptr i64 {} to {}\n",
                        fn_ptr, ptr_tmp, fn_type
                    ));

                    // Make indirect call with all arguments
                    let tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i64 {}({}){}\n",
                        tmp,
                        fn_ptr,
                        all_args.join(", "),
                        dbg_info
                    ));
                    Ok((tmp, ir))
                } else if fn_name == "malloc" {
                    // Special handling for malloc: call returns i8*, convert to i64
                    let ptr_tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i8* @malloc({}){}\n",
                        ptr_tmp,
                        arg_vals.join(", "),
                        dbg_info
                    ));
                    let result = self.next_temp(counter);
                    ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, ptr_tmp));
                    Ok((result, ir))
                } else if fn_name == "free" {
                    // Special handling for free: convert i64 to i8*
                    let ptr_tmp = self.next_temp(counter);
                    // Extract the i64 value from arg_vals
                    let arg_val = arg_vals
                        .first()
                        .map(|s| s.split_whitespace().last().unwrap_or("0"))
                        .unwrap_or("0");
                    ir.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        ptr_tmp, arg_val
                    ));
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!("  call void @free(i8* {}){}\n", ptr_tmp, dbg_info));
                    Ok(("void".to_string(), ir))
                } else if fn_name == "memcpy" || fn_name == "memcpy_str" {
                    // Special handling for memcpy/memcpy_str: convert pointers as needed
                    let dest_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
                    let src_full = arg_vals.get(1).map(|s| s.as_str()).unwrap_or("i64 0");
                    let n_val = arg_vals
                        .get(2)
                        .map(|s| s.split_whitespace().last().unwrap_or("0"))
                        .unwrap_or("0");

                    // Handle dest pointer
                    let dest_ptr = if dest_full.starts_with("i8*") {
                        // Use everything after "i8* " to preserve complex expressions
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
                        // Use everything after "i8* " to preserve complex expressions
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
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i8* @memcpy(i8* {}, i8* {}, i64 {}){}\n",
                        result, dest_ptr, src_ptr, n_val, dbg_info
                    ));
                    // Convert result back to i64
                    let result_i64 = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = ptrtoint i8* {} to i64\n",
                        result_i64, result
                    ));
                    Ok((result_i64, ir))
                } else if fn_name == "strlen" {
                    // Special handling for strlen: convert i64 to i8* if needed
                    let arg_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
                    let result = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);

                    // Check if the argument is already i8* (str type) or i64 (pointer as integer)
                    if arg_full.starts_with("i8*") {
                        // Already a pointer, use directly
                        // Use everything after "i8* " to preserve complex expressions like getelementptr
                        let ptr_val = arg_full
                            .strip_prefix("i8* ")
                            .unwrap_or(arg_full.split_whitespace().last().unwrap_or("null"));
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
                    Ok((result, ir))
                } else if fn_name == "puts_ptr" {
                    // Special handling for puts_ptr: convert i64 to i8*
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
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i32 @puts(i8* {}){}\n",
                        i32_result, ptr_tmp, dbg_info
                    ));
                    // Convert i32 result to i64 for consistency
                    let result = self.next_temp(counter);
                    ir.push_str(&format!("  {} = sext i32 {} to i64\n", result, i32_result));
                    Ok((result, ir))
                } else if ret_ty == "void" {
                    // Check for recursive call with decreases clause
                    if self.is_recursive_call(&fn_name) {
                        let check_ir = self.generate_recursive_decreases_check(args, counter)?;
                        ir.push_str(&check_ir);
                    }

                    // Direct void function call
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    if is_vararg {
                        let param_types: Vec<String> = fn_info
                            .as_ref()
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
                    Ok(("void".to_string(), ir))
                } else if ret_ty == "i32" {
                    // Check for recursive call with decreases clause
                    if self.is_recursive_call(&fn_name) {
                        let check_ir = self.generate_recursive_decreases_check(args, counter)?;
                        ir.push_str(&check_ir);
                    }

                    // i32 return function call - convert to i64 for consistency
                    let i32_tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    if is_vararg {
                        // Variadic functions need explicit signature in LLVM IR call
                        let param_types: Vec<String> = fn_info
                            .as_ref()
                            .map(|f| {
                                f.signature
                                    .params
                                    .iter()
                                    .map(|(_, ty, _)| self.type_to_llvm(ty))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let sig = format!("i32 ({}, ...)", param_types.join(", "));
                        ir.push_str(&format!(
                            "  {} = call {} @{}({}){}\n",
                            i32_tmp,
                            sig,
                            actual_fn_name,
                            arg_vals.join(", "),
                            dbg_info
                        ));
                    } else {
                        ir.push_str(&format!(
                            "  {} = call i32 @{}({}){}\n",
                            i32_tmp,
                            actual_fn_name,
                            arg_vals.join(", "),
                            dbg_info
                        ));
                    }
                    let tmp = self.next_temp(counter);
                    ir.push_str(&format!("  {} = sext i32 {} to i64\n", tmp, i32_tmp));
                    Ok((tmp, ir))
                } else {
                    // Check for recursive call with decreases clause
                    if self.is_recursive_call(&fn_name) {
                        let check_ir = self.generate_recursive_decreases_check(args, counter)?;
                        ir.push_str(&check_ir);
                    }

                    // Direct function call with return value
                    let tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    if is_vararg {
                        let param_types: Vec<String> = fn_info
                            .as_ref()
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
                    Ok((tmp, ir))
                }
            }

            // If/Else expression with basic blocks
            Expr::If { cond, then, else_ } => {
                let then_label = self.next_label("then");
                let else_label = self.next_label("else");
                let merge_label = self.next_label("merge");

                // Infer the type of the then block for phi node
                let block_type = self.infer_block_type(then);
                let llvm_type = self.type_to_llvm(&block_type);

                // Check if the result is a struct type (returned as pointer from struct literals)
                let is_struct_result = matches!(&block_type, ResolvedType::Named { .. })
                    && !self.is_block_result_value(then);

                // Generate condition
                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let mut ir = cond_ir;

                // Convert to i1 for branch condition (type-aware: skips for bool/i1)
                let (cond_bool, conv_ir) = self.generate_cond_to_i1(cond, &cond_val, counter);
                ir.push_str(&conv_ir);

                // Conditional branch
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_bool, then_label, else_label
                ));

                // Then block
                ir.push_str(&format!("{}:\n", then_label));
                self.current_block = then_label.clone();
                let (then_val, then_ir, then_terminated) =
                    self.generate_block_stmts(then, counter)?;
                ir.push_str(&then_ir);

                // For struct results, load the value before branch if it's a pointer
                let then_val_for_phi = if is_struct_result && !then_terminated {
                    let loaded = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        loaded, llvm_type, llvm_type, then_val
                    ));
                    loaded
                } else {
                    then_val.clone()
                };

                let then_actual_block = self.current_block.clone();
                // Only emit branch to merge if block is not terminated
                let then_from_label = if !then_terminated {
                    ir.push_str(&format!("  br label %{}\n", merge_label));
                    then_actual_block
                } else {
                    String::new() // Block is terminated, won't contribute to phi
                };

                // Else block
                ir.push_str(&format!("{}:\n", else_label));
                self.current_block = else_label.clone();
                let (else_val, else_ir, else_terminated, nested_last_block, has_else) =
                    if let Some(else_branch) = else_ {
                        let (v, i, t, last) =
                            self.generate_if_else_with_term(else_branch, counter, &merge_label)?;
                        (v, i, t, last, true)
                    } else {
                        ("0".to_string(), String::new(), false, String::new(), false)
                    };
                ir.push_str(&else_ir);

                // For struct results, load the value before branch if it's a pointer
                // But if else_val comes from a nested if-else (indicated by non-empty nested_last_block),
                // it's already a phi node value (not a pointer), so don't load it
                let else_val_for_phi = if is_struct_result
                    && !else_terminated
                    && has_else
                    && nested_last_block.is_empty()
                {
                    let loaded = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        loaded, llvm_type, llvm_type, else_val
                    ));
                    loaded
                } else {
                    else_val.clone()
                };

                // Only emit branch to merge if block is not terminated
                let else_from_label = if !else_terminated {
                    ir.push_str(&format!("  br label %{}\n", merge_label));
                    // If there was a nested if-else, use its merge block as the predecessor
                    if !nested_last_block.is_empty() {
                        nested_last_block
                    } else {
                        self.current_block.clone()
                    }
                } else {
                    String::new()
                };

                // Merge block with phi node
                ir.push_str(&format!("{}:\n", merge_label));
                self.current_block = merge_label.clone();
                let result = self.next_temp(counter);

                // Check if the block type is void/unit - if so, don't generate phi nodes
                // (phi nodes cannot have void type in LLVM IR)
                let is_void_type = matches!(block_type, ResolvedType::Unit);

                // If there's no else branch, don't use phi - the value is not meaningful
                // This avoids type mismatches when then branch returns i32 (e.g., putchar)
                if !has_else || is_void_type {
                    // If-only statement or void type: value is not used, just use 0
                    ir.push_str(&format!("  {} = add i64 0, 0\n", result));
                } else if !then_from_label.is_empty() && !else_from_label.is_empty() {
                    // Both branches reach merge - use the inferred type
                    ir.push_str(&format!(
                        "  {} = phi {} [ {}, %{} ], [ {}, %{} ]\n",
                        result,
                        llvm_type,
                        then_val_for_phi,
                        then_from_label,
                        else_val_for_phi,
                        else_from_label
                    ));
                } else if !then_from_label.is_empty() {
                    // Only then branch reaches merge
                    ir.push_str(&format!(
                        "  {} = phi {} [ {}, %{} ]\n",
                        result, llvm_type, then_val_for_phi, then_from_label
                    ));
                } else if !else_from_label.is_empty() {
                    // Only else branch reaches merge
                    ir.push_str(&format!(
                        "  {} = phi {} [ {}, %{} ]\n",
                        result, llvm_type, else_val_for_phi, else_from_label
                    ));
                } else {
                    // Neither branch reaches merge (both break/continue)
                    // This merge block is actually unreachable, but we still need a value
                    ir.push_str(&format!("  {} = add i64 0, 0\n", result));
                }

                Ok((result, ir))
            }

            // Loop expression
            Expr::Loop {
                pattern,
                iter,
                body,
            } => {
                // Check if this is a range-based for loop
                let is_range_loop = iter
                    .as_ref()
                    .is_some_and(|it| matches!(&it.node, Expr::Range { .. }));

                if is_range_loop {
                    if let (Some(pat), Some(it)) = (pattern.as_ref(), iter.as_ref()) {
                        // Range-based for loop: L pattern : start..end { body }
                        return self.generate_range_for_loop(pat, it, body, counter);
                    }
                }

                // Conditional or infinite loop
                let loop_start = self.next_label("loop.start");
                let loop_body = self.next_label("loop.body");
                let loop_end = self.next_label("loop.end");

                // Push loop labels for break/continue
                self.loop_stack.push(LoopLabels {
                    continue_label: loop_start.clone(),
                    break_label: loop_end.clone(),
                });

                let mut ir = String::new();

                // Check if this is a conditional loop (L cond { body }) or infinite loop
                if let Some(iter_expr) = iter {
                    // Conditional loop: L condition { body }
                    ir.push_str(&format!("  br label %{}\n", loop_start));
                    ir.push_str(&format!("{}:\n", loop_start));

                    // Evaluate condition
                    let (cond_val, cond_ir) = self.generate_expr(iter_expr, counter)?;
                    ir.push_str(&cond_ir);

                    // Convert to i1 for branch (type-aware: skips for bool/i1)
                    let (cond_bool, conv_ir) = self.generate_cond_to_i1(iter_expr, &cond_val, counter);
                    ir.push_str(&conv_ir);
                    ir.push_str(&format!(
                        "  br i1 {}, label %{}, label %{}\n",
                        cond_bool, loop_body, loop_end
                    ));

                    // Loop body
                    ir.push_str(&format!("{}:\n", loop_body));
                    let (_body_val, body_ir, body_terminated) =
                        self.generate_block_stmts(body, counter)?;
                    ir.push_str(&body_ir);
                    // Only emit loop back if body doesn't terminate
                    if !body_terminated {
                        ir.push_str(&format!("  br label %{}\n", loop_start));
                    }
                } else {
                    // Infinite loop: L { body } - must use break to exit
                    ir.push_str(&format!("  br label %{}\n", loop_start));
                    ir.push_str(&format!("{}:\n", loop_start));
                    let (_body_val, body_ir, body_terminated) =
                        self.generate_block_stmts(body, counter)?;
                    ir.push_str(&body_ir);
                    // Only emit loop back if body doesn't terminate
                    if !body_terminated {
                        ir.push_str(&format!("  br label %{}\n", loop_start));
                    }
                }

                // Loop end
                ir.push_str(&format!("{}:\n", loop_end));

                self.loop_stack.pop();

                // Loop returns void by default (use break with value for expression)
                Ok(("0".to_string(), ir))
            }

            // While loop expression
            Expr::While { condition, body } => {
                let loop_start = self.next_label("while.start");
                let loop_body = self.next_label("while.body");
                let loop_end = self.next_label("while.end");

                // Push loop labels for break/continue
                self.loop_stack.push(LoopLabels {
                    continue_label: loop_start.clone(),
                    break_label: loop_end.clone(),
                });

                let mut ir = String::new();

                // Jump to condition check
                ir.push_str(&format!("  br label %{}\n", loop_start));
                ir.push_str(&format!("{}:\n", loop_start));

                // Evaluate condition
                let (cond_val, cond_ir) = self.generate_expr(condition, counter)?;
                ir.push_str(&cond_ir);

                // Convert to i1 for branch (type-aware: skips for bool/i1)
                let (cond_bool, conv_ir) = self.generate_cond_to_i1(condition, &cond_val, counter);
                ir.push_str(&conv_ir);
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_bool, loop_body, loop_end
                ));

                // Loop body
                ir.push_str(&format!("{}:\n", loop_body));
                let (_body_val, body_ir, body_terminated) =
                    self.generate_block_stmts(body, counter)?;
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

            // Block expression
            Expr::Block(stmts) => {
                let (val, ir, _terminated) = self.generate_block_stmts(stmts, counter)?;
                Ok((val, ir))
            }

            // Assignment expression
            Expr::Assign { target, value } => {
                let (val, val_ir) = self.generate_expr(value, counter)?;
                let mut ir = val_ir;

                if let Expr::Ident(name) = &target.node {
                    if let Some(local) = self.locals.get(name).cloned() {
                        if !local.is_param() {
                            let llvm_ty = self.type_to_llvm(&local.ty);

                            // If the variable was originally SSA, we need to upgrade it to alloca
                            // because SSA values cannot be reassigned with store.
                            if local.is_ssa() {
                                let alloca_name =
                                    format!("{}.alloca.{}", name, counter);
                                *counter += 1;
                                ir.push_str(&format!(
                                    "  %{} = alloca {}\n",
                                    alloca_name, llvm_ty
                                ));
                                ir.push_str(&format!(
                                    "  store {} {}, {}* %{}\n",
                                    llvm_ty, val, llvm_ty, alloca_name
                                ));
                                // Upgrade from SSA to Alloca so future reads use load
                                self.locals.insert(
                                    name.clone(),
                                    LocalVar::alloca(local.ty.clone(), alloca_name),
                                );
                            } else if matches!(&local.ty, ResolvedType::Named { .. })
                                && local.is_alloca()
                            {
                                // For struct types (Named), the local is a double pointer (%Type**).
                                // We need to alloca a new struct, store the value, then update the pointer.
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
                } else if let Expr::Deref(inner) = &target.node {
                    // Pointer dereference assignment: *ptr = value
                    let (ptr_val, ptr_ir) = self.generate_expr(inner, counter)?;
                    ir.push_str(&ptr_ir);
                    // Store value at the pointed-to location
                    ir.push_str(&format!("  store i64 {}, i64* {}\n", val, ptr_val));
                } else if let Expr::Index {
                    expr: arr_expr,
                    index,
                } = &target.node
                {
                    // Array index assignment: arr[i] = value
                    let (arr_val, arr_ir) = self.generate_expr(arr_expr, counter)?;
                    ir.push_str(&arr_ir);
                    let (idx_val, idx_ir) = self.generate_expr(index, counter)?;
                    ir.push_str(&idx_ir);

                    // Determine element type from array type
                    let arr_type = self.infer_expr_type(arr_expr);
                    let elem_llvm_type = match &arr_type {
                        ResolvedType::Array(inner) => self.type_to_llvm(inner),
                        ResolvedType::ConstArray { element, .. } => self.type_to_llvm(element),
                        ResolvedType::Pointer(inner) => self.type_to_llvm(inner),
                        _ => "i64".to_string(),
                    };

                    let elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i64 {}\n",
                        elem_ptr, elem_llvm_type, elem_llvm_type, arr_val, idx_val
                    ));
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        elem_llvm_type, val, elem_llvm_type, elem_ptr
                    ));
                } else if let Expr::Field {
                    expr: obj_expr,
                    field,
                } = &target.node
                {
                    // Field assignment: obj.field = value
                    let (obj_val, obj_ir) = self.generate_expr(obj_expr, counter)?;
                    ir.push_str(&obj_ir);

                    // Get struct or union info
                    if let Expr::Ident(var_name) = &obj_expr.node {
                        if let Some(local) = self.locals.get(var_name.as_str()).cloned() {
                            if let ResolvedType::Named {
                                name: type_name, ..
                            } = &local.ty
                            {
                                // First check struct
                                if let Some(struct_info) = self.structs.get(type_name).cloned() {
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
                                            field_ptr, type_name, type_name, obj_val, field_idx
                                        ));
                                        ir.push_str(&format!(
                                            "  store {} {}, {}* {}\n",
                                            llvm_ty, val, llvm_ty, field_ptr
                                        ));
                                    }
                                }
                                // Then check union
                                else if let Some(union_info) = self.unions.get(type_name).cloned()
                                {
                                    if let Some((_, field_ty)) =
                                        union_info.fields.iter().find(|(n, _)| n == &field.node)
                                    {
                                        let llvm_ty = self.type_to_llvm(field_ty);

                                        // For union, bitcast to field type pointer
                                        let field_ptr = self.next_temp(counter);
                                        ir.push_str(&format!(
                                            "  {} = bitcast %{}* {} to {}*\n",
                                            field_ptr, type_name, obj_val, llvm_ty
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

            // Compound assignment (+=, -=, etc.)
            Expr::AssignOp { op, target, value } => {
                // First load current value
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

                // Store back
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

            // Array literal: [a, b, c]
            Expr::Array(elements) => {
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

                // Allocate array on stack
                let arr_ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = alloca {}\n", arr_ptr, arr_ty));

                // Store each element
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

                // Return pointer to first element
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr {}, {}* {}, i64 0, i64 0\n",
                    result, arr_ty, arr_ty, arr_ptr
                ));

                Ok((result, ir))
            }

            // Map literal: {k: v, ...}
            // Stored as parallel arrays of keys and values on the stack
            Expr::MapLit(pairs) => {
                let mut ir = String::new();
                let len = pairs.len();

                // Infer key/value types
                let (key_ty, val_ty) = if let Some((k, v)) = pairs.first() {
                    let kt = self.type_to_llvm(&self.infer_expr_type(k));
                    let vt = self.type_to_llvm(&self.infer_expr_type(v));
                    (kt, vt)
                } else {
                    ("i64".to_string(), "i64".to_string())
                };

                let keys_arr_ty = format!("[{} x {}]", len, key_ty);
                let vals_arr_ty = format!("[{} x {}]", len, val_ty);

                // Allocate key and value arrays on stack
                let keys_ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = alloca {}\n", keys_ptr, keys_arr_ty));
                let vals_ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = alloca {}\n", vals_ptr, vals_arr_ty));

                // Store each key-value pair
                for (i, (k, v)) in pairs.iter().enumerate() {
                    let (kval, k_ir) = self.generate_expr(k, counter)?;
                    ir.push_str(&k_ir);
                    let k_elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i64 0, i64 {}\n",
                        k_elem_ptr, keys_arr_ty, keys_arr_ty, keys_ptr, i
                    ));
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        key_ty, kval, key_ty, k_elem_ptr
                    ));

                    let (vval, v_ir) = self.generate_expr(v, counter)?;
                    ir.push_str(&v_ir);
                    let v_elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i64 0, i64 {}\n",
                        v_elem_ptr, vals_arr_ty, vals_arr_ty, vals_ptr, i
                    ));
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        val_ty, vval, val_ty, v_elem_ptr
                    ));
                }

                // Return pointer to keys array (map is represented as parallel arrays)
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr {}, {}* {}, i64 0, i64 0\n",
                    result, keys_arr_ty, keys_arr_ty, keys_ptr
                ));

                Ok((result, ir))
            }

            // Tuple literal: (a, b, c)
            Expr::Tuple(elements) => {
                let mut ir = String::new();
                let len = elements.len();

                // Build tuple type string
                let tuple_ty = format!("{{ {} }}", vec!["i64"; len].join(", "));

                // Allocate tuple on stack
                let tuple_ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = alloca {}\n", tuple_ptr, tuple_ty));

                // Store each element
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

                // Load and return tuple value
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    result, tuple_ty, tuple_ty, tuple_ptr
                ));

                Ok((result, ir))
            }

            // Struct literal: Point{x:1, y:2}
            // Also handles union literal: IntOrFloat{as_int: 42}
            Expr::StructLit { name, fields } => {
                let resolved_name = self.resolve_struct_name(&name.node);
                let type_name = &resolved_name;

                // First check if it's a struct
                if let Some(struct_info) = self.structs.get(type_name).cloned() {
                    let mut ir = String::new();

                    // Allocate struct on stack
                    let struct_ptr = self.next_temp(counter);
                    ir.push_str(&format!("  {} = alloca %{}\n", struct_ptr, type_name));

                    // Store each field
                    for (field_name, field_expr) in fields {
                        // Find field index
                        let field_idx = struct_info
                            .fields
                            .iter()
                            .position(|(n, _)| n == &field_name.node)
                            .ok_or_else(|| {
                                let candidates: Vec<&str> = struct_info
                                    .fields
                                    .iter()
                                    .map(|(name, _)| name.as_str())
                                    .collect();
                                let suggestions = suggest_similar(&field_name.node, &candidates, 3);
                                let suggestion_text = format_did_you_mean(&suggestions);
                                CodegenError::TypeError(format!(
                                    "Unknown field '{}' in struct '{}'{}",
                                    field_name.node, type_name, suggestion_text
                                ))
                            })?;

                        let (val, field_ir) = self.generate_expr(field_expr, counter)?;
                        ir.push_str(&field_ir);

                        let field_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                            field_ptr, type_name, type_name, struct_ptr, field_idx
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

                    // Return pointer to struct
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

                    // Return pointer to union
                    Ok((union_ptr, ir))
                } else {
                    Err(CodegenError::TypeError(format!(
                        "Unknown struct or union: {}",
                        type_name
                    )))
                }
            }

            // Index: arr[idx] or slice: arr[start..end]
            Expr::Index {
                expr: array_expr,
                index,
            } => {
                // Check if this is a slice operation (index is a Range expression)
                if let Expr::Range {
                    start,
                    end,
                    inclusive,
                } = &index.node
                {
                    return self.generate_slice(
                        array_expr,
                        start.as_deref(),
                        end.as_deref(),
                        *inclusive,
                        counter,
                    );
                }

                let (arr_val, arr_ir) = self.generate_expr(array_expr, counter)?;

                // Check if the type is actually indexable
                let arr_type = self.infer_expr_type(array_expr);
                match arr_type {
                    ResolvedType::Array(_)
                    | ResolvedType::ConstArray { .. }
                    | ResolvedType::Pointer(_)
                    | ResolvedType::Slice(_)
                    | ResolvedType::SliceMut(_) => {
                        // OK - indexable type
                    }
                    _ => {
                        let type_name = format!("{:?}", arr_type);
                        return Err(CodegenError::TypeError(format!(
                            "Cannot index non-array type (found {})",
                            type_name
                        )));
                    }
                }
                let (idx_val, idx_ir) = self.generate_expr(index, counter)?;

                let mut ir = arr_ir;
                ir.push_str(&idx_ir);

                // Determine element type from array type
                let elem_llvm_type = match &arr_type {
                    ResolvedType::Array(inner) => self.type_to_llvm(inner),
                    ResolvedType::ConstArray { element, .. } => self.type_to_llvm(element),
                    ResolvedType::Pointer(inner) => self.type_to_llvm(inner),
                    ResolvedType::Slice(inner) | ResolvedType::SliceMut(inner) => {
                        self.type_to_llvm(inner)
                    }
                    _ => "i64".to_string(),
                };

                // Handle indexing differently for Slice types vs regular arrays/pointers
                let result = if matches!(arr_type, ResolvedType::Slice(_) | ResolvedType::SliceMut(_))
                {
                    // For Slice types, extract data pointer from fat pointer struct
                    // Slice is { i8* data, i64 length }
                    let data_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = extractvalue {{ i8*, i64 }} {}, 0\n",
                        data_ptr, arr_val
                    ));

                    // Bitcast i8* to typed pointer
                    let typed_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = bitcast i8* {} to {}*\n",
                        typed_ptr, data_ptr, elem_llvm_type
                    ));

                    // Get element pointer
                    let elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i64 {}\n",
                        elem_ptr, elem_llvm_type, elem_llvm_type, typed_ptr, idx_val
                    ));

                    // Load element
                    let result = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        result, elem_llvm_type, elem_llvm_type, elem_ptr
                    ));

                    result
                } else {
                    // For regular arrays/pointers, use direct GEP
                    let elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i64 {}\n",
                        elem_ptr, elem_llvm_type, elem_llvm_type, arr_val, idx_val
                    ));

                    // Load element
                    let result = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        result, elem_llvm_type, elem_llvm_type, elem_ptr
                    ));

                    result
                };

                Ok((result, ir))
            }

            // Field access: obj.field
            Expr::Field {
                expr: obj_expr,
                field,
            } => {
                let (obj_val, obj_ir) = self.generate_expr(obj_expr, counter)?;
                let mut ir = obj_ir;

                // Use type inference to get the type of the object expression
                // This handles both simple identifiers and nested field accesses
                let obj_type = self.infer_expr_type(obj_expr);

                if let ResolvedType::Named {
                    name: orig_type_name,
                    ..
                } = &obj_type
                {
                    let type_name = &self.resolve_struct_name(orig_type_name);
                    // First check if it's a struct
                    if let Some(struct_info) = self.structs.get(type_name).cloned() {
                        let field_idx = struct_info
                            .fields
                            .iter()
                            .position(|(n, _)| n == &field.node)
                            .ok_or_else(|| {
                                let candidates: Vec<&str> = struct_info
                                    .fields
                                    .iter()
                                    .map(|(name, _)| name.as_str())
                                    .collect();
                                let suggestions = suggest_similar(&field.node, &candidates, 3);
                                let suggestion_text = format_did_you_mean(&suggestions);
                                CodegenError::TypeError(format!(
                                    "Unknown field '{}' in struct '{}'{}",
                                    field.node, type_name, suggestion_text
                                ))
                            })?;

                        let field_ty = &struct_info.fields[field_idx].1;
                        let llvm_ty = self.type_to_llvm(field_ty);

                        // Generate field pointer
                        let field_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                            field_ptr, type_name, type_name, obj_val, field_idx
                        ));

                        // Only load if the field is not itself a struct (to support nested access)
                        // For nested field access like o.a.val, we want o.a to return a pointer to Inner,
                        // not the Inner value itself
                        if matches!(field_ty, ResolvedType::Named { .. }) {
                            // Field is a struct - return pointer for nested access
                            return Ok((field_ptr, ir));
                        } else {
                            // Field is a primitive - load the value
                            let result = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = load {}, {}* {}\n",
                                result, llvm_ty, llvm_ty, field_ptr
                            ));
                            return Ok((result, ir));
                        }
                    }
                    // Then check if it's a union
                    else if let Some(union_info) = self.unions.get(type_name).cloned() {
                        let field_ty = union_info
                            .fields
                            .iter()
                            .find(|(n, _)| n == &field.node)
                            .map(|(_, ty)| ty.clone())
                            .ok_or_else(|| {
                                let candidates: Vec<&str> = union_info
                                    .fields
                                    .iter()
                                    .map(|(name, _)| name.as_str())
                                    .collect();
                                let suggestions = suggest_similar(&field.node, &candidates, 3);
                                let suggestion_text = format_did_you_mean(&suggestions);
                                CodegenError::TypeError(format!(
                                    "Unknown field '{}' in union '{}'{}",
                                    field.node, type_name, suggestion_text
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

                Err(CodegenError::Unsupported(
                    "field access requires known struct or union type".to_string(),
                ))
            }

            // Method call: obj.method(args)
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                // Special case: @.method() means self.method() (call another method on self)
                let (recv_val, recv_ir, recv_type) = if matches!(&receiver.node, Expr::SelfCall) {
                    // When receiver is @, use %self instead of the function pointer
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
                    return self.generate_string_method_call(
                        &recv_val,
                        &ir,
                        method_name,
                        args,
                        counter,
                    );
                }

                // Check for dynamic trait dispatch (dyn Trait)
                let dyn_trait_name = match &recv_type {
                    ResolvedType::DynTrait { trait_name, .. } => Some(trait_name.clone()),
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                        if let ResolvedType::DynTrait { trait_name, .. } = inner.as_ref() {
                            Some(trait_name.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(trait_name) = dyn_trait_name {
                    // Dynamic dispatch through vtable
                    // Generate additional arguments (just values, vtable generator adds types)
                    let mut extra_arg_vals = Vec::new();
                    for arg in args {
                        let (val, arg_ir) = self.generate_expr(arg, counter)?;
                        ir.push_str(&arg_ir);
                        extra_arg_vals.push(val);
                    }

                    let (dyn_ir, result) = self.generate_dyn_method_call(
                        &recv_val,
                        &trait_name,
                        method_name,
                        &extra_arg_vals,
                        counter,
                    )?;
                    ir.push_str(&dyn_ir);
                    return Ok((result, ir));
                }

                // Build full method name: ResolvedStructName_methodName
                // Use resolve_struct_name to match definition naming (e.g., Pair → Pair$i64)
                // For non-generic structs, this is a no-op (Vec → Vec)
                let full_method_name = if let ResolvedType::Named { name, .. } = &recv_type {
                    let resolved = self.resolve_struct_name(name);
                    format!("{}_{}", resolved, method_name)
                } else {
                    method_name.clone()
                };

                // Get struct type for receiver (add * for pointer)
                let recv_llvm_ty = if matches!(&recv_type, ResolvedType::Named { .. }) {
                    format!("{}*", self.type_to_llvm(&recv_type))
                } else {
                    self.type_to_llvm(&recv_type)
                };

                // Generate arguments (receiver is implicit first arg)
                let mut arg_vals = vec![format!("{} {}", recv_llvm_ty, recv_val)];

                for arg in args {
                    let (val, arg_ir) = self.generate_expr(arg, counter)?;
                    ir.push_str(&arg_ir);
                    let arg_type = self.infer_expr_type(arg);
                    let arg_llvm_ty = self.type_to_llvm(&arg_type);
                    arg_vals.push(format!("{} {}", arg_llvm_ty, val));
                }

                // Determine return type from function registry
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

            // Static method call: Type.method(args)
            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => {
                let mut ir = String::new();

                // Build full method name: TypeName_methodName
                let full_method_name = format!("{}_{}", type_name.node, method.node);

                // Generate arguments (no receiver for static methods)
                let mut arg_vals = Vec::new();

                for arg in args {
                    let (val, arg_ir) = self.generate_expr(arg, counter)?;
                    ir.push_str(&arg_ir);
                    let arg_type = self.infer_expr_type(arg);
                    let arg_llvm_ty = self.type_to_llvm(&arg_type);
                    arg_vals.push(format!("{} {}", arg_llvm_ty, val));
                }

                // Get return type from method signature
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

            // Spread: ..expr (handled within array generation; standalone generates inner)
            Expr::Spread(inner) => self.generate_expr(inner, counter),

            // Reference: &expr
            Expr::Ref(inner) => {
                // For simple references, just return the address
                if let Expr::Ident(name) = &inner.node {
                    if let Some(local) = self.locals.get(name.as_str()).cloned() {
                        if local.is_alloca() {
                            // Alloca variables already have an address
                            return Ok((format!("%{}", local.llvm_name), String::new()));
                        } else {
                            // SSA/Param values need to be spilled to stack to take their address
                            let mut ir = String::new();
                            let llvm_ty = self.type_to_llvm(&local.ty);
                            let (val, val_ir) = self.generate_expr(inner, counter)?;
                            ir.push_str(&val_ir);
                            let tmp_alloca = self.next_temp(counter);
                            ir.push_str(&format!("  {} = alloca {}\n", tmp_alloca, llvm_ty));
                            ir.push_str(&format!(
                                "  store {} {}, {}* {}\n",
                                llvm_ty, val, llvm_ty, tmp_alloca
                            ));
                            return Ok((tmp_alloca, ir));
                        }
                    }
                }
                // For complex expressions, evaluate and return
                self.generate_expr(inner, counter)
            }

            // Dereference: *expr
            Expr::Deref(inner) => {
                let (ptr_val, ptr_ir) = self.generate_expr(inner, counter)?;
                let mut ir = ptr_ir;

                let result = self.next_temp(counter);
                ir.push_str(&format!("  {} = load i64, i64* {}\n", result, ptr_val));

                Ok((result, ir))
            }

            // Type cast: expr as Type
            Expr::Cast { expr, ty } => {
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
                    // Pointer to integer cast
                    _ if val.starts_with('%') || val.parse::<i64>().is_err() => {
                        // Might be a pointer, try to cast
                        ir.push_str(&format!("  {} = ptrtoint i64* {} to i64\n", result, val));
                    }
                    // Default: just use the value as-is (same size types)
                    _ => {
                        return Ok((val, ir));
                    }
                }

                Ok((result, ir))
            }

            // Match expression: M expr { pattern => body, ... }
            Expr::Match {
                expr: match_expr,
                arms,
            } => self.generate_match(match_expr, arms, counter),

            // Range expression (for now just return start value)
            Expr::Range { start, .. } => {
                if let Some(start_expr) = start {
                    self.generate_expr(start_expr, counter)
                } else {
                    Ok(("0".to_string(), String::new()))
                }
            }

            // Await expression: poll the future until Ready
            Expr::Await(inner) => {
                let (future_ptr, future_ir) = self.generate_expr(inner, counter)?;
                let mut ir = future_ir;

                // Get the function name being awaited (for poll function lookup)
                // Helper to extract poll function name from an expression
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
                        Expr::Spawn(inner) => {
                            // For spawn, look at the inner expression
                            get_poll_func_name(&inner.node)
                        }
                        _ => "__async_poll".to_string(),
                    }
                }
                let poll_func = get_poll_func_name(&inner.node);

                // Generate blocking poll loop
                let poll_start = self.next_label("await_poll");
                let poll_ready = self.next_label("await_ready");
                let poll_pending = self.next_label("await_pending");

                ir.push_str(&format!("  br label %{}\n\n", poll_start));
                ir.push_str(&format!("{}:\n", poll_start));

                // Call poll function: returns {i64 status, i64 result}
                let poll_result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call {{ i64, i64 }} @{}(i64 {})\n",
                    poll_result, poll_func, future_ptr
                ));

                // Extract status (0 = Pending, 1 = Ready)
                let status = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {{ i64, i64 }} {}, 0\n",
                    status, poll_result
                ));

                // Check if Ready
                let is_ready = self.next_temp(counter);
                ir.push_str(&format!("  {} = icmp eq i64 {}, 1\n", is_ready, status));
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n\n",
                    is_ready, poll_ready, poll_pending
                ));

                // Pending: yield and retry (for now just spin)
                ir.push_str(&format!("{}:\n", poll_pending));
                ir.push_str(&format!("  br label %{}\n\n", poll_start));

                // Ready: extract result
                ir.push_str(&format!("{}:\n", poll_ready));
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {{ i64, i64 }} {}, 1\n",
                    result, poll_result
                ));

                Ok((result, ir))
            }

            // Spawn expression: create a new task for the runtime
            Expr::Spawn(inner) => {
                let (future_ptr, future_ir) = self.generate_expr(inner, counter)?;
                let mut ir = future_ir;

                // Spawn returns the task/future handle for later awaiting
                // For now, just return the future pointer directly
                ir.push_str(&format!("; Spawned task at {}\n", future_ptr));

                Ok((future_ptr, ir))
            }

            // Yield expression: yield a value from generator
            // For now, treat yield as returning the value (simplified generator support)
            Expr::Yield(inner) => {
                let (val, ir) = self.generate_expr(inner, counter)?;
                // In a full implementation, yield would save state and return.
                // For now, it just evaluates and returns the yielded value.
                Ok((val, ir))
            }

            // Comptime expression: evaluate at compile time and emit constant
            Expr::Comptime { body } => {
                // Evaluate at compile time
                let mut evaluator = vais_types::ComptimeEvaluator::new();
                let value = evaluator.eval(body).map_err(|e| {
                    CodegenError::TypeError(format!("Comptime evaluation failed: {}", e))
                })?;

                // Return the evaluated constant
                match value {
                    vais_types::ComptimeValue::Int(n) => Ok((n.to_string(), String::new())),
                    vais_types::ComptimeValue::Float(f) => {
                        Ok((crate::types::format_llvm_float(f), String::new()))
                    }
                    vais_types::ComptimeValue::Bool(b) => {
                        Ok((if b { "1" } else { "0" }.to_string(), String::new()))
                    }
                    vais_types::ComptimeValue::String(s) => {
                        // Create a global string constant
                        let name = self.make_string_name();
                        self.string_counter += 1;
                        self.string_constants.push((name.clone(), s.clone()));
                        let len = s.len() + 1;
                        Ok((
                            format!(
                                "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                                len, len, name
                            ),
                            String::new(),
                        ))
                    }
                    vais_types::ComptimeValue::Array(arr) => {
                        // Generate array literal from comptime array
                        let mut elements = Vec::new();
                        let mut ir = String::new();

                        for elem in arr {
                            match elem {
                                vais_types::ComptimeValue::Int(n) => elements.push(n.to_string()),
                                vais_types::ComptimeValue::Float(f) => {
                                    elements.push(crate::types::format_llvm_float(f))
                                }
                                vais_types::ComptimeValue::Bool(b) => {
                                    elements.push(if b { "1" } else { "0" }.to_string())
                                }
                                _ => {
                                    return Err(CodegenError::TypeError(
                                        "Comptime arrays can only contain simple values (int, float, bool)".to_string()
                                    ));
                                }
                            }
                        }

                        // Create array on the stack
                        let array_name = format!("%comptime_array_{}", counter);
                        *counter += 1;
                        let len = elements.len();

                        // For now, assume i64 elements (we'd need better type inference for mixed types)
                        ir.push_str(&format!("  {} = alloca [{} x i64]\n", array_name, len));

                        for (i, elem_val) in elements.iter().enumerate() {
                            let elem_ptr = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = getelementptr [{} x i64], [{} x i64]* {}, i64 0, i64 {}\n",
                                elem_ptr, len, len, array_name, i
                            ));
                            ir.push_str(&format!("  store i64 {}, i64* {}\n", elem_val, elem_ptr));
                        }

                        Ok((array_name, ir))
                    }
                    vais_types::ComptimeValue::Unit => Ok(("void".to_string(), String::new())),
                }
            }

            // Macro invocation (should be expanded before codegen)
            Expr::MacroInvoke(invoke) => Err(CodegenError::TypeError(format!(
                "Unexpanded macro invocation: {}! - macros must be expanded before code generation",
                invoke.name.node
            ))),

            // Old expression for contract ensures clauses
            Expr::Old(inner) => {
                // old(expr) references a pre-snapshot value
                // In codegen, we generate a load from the pre-snapshot storage
                let old_var_name = format!("__old_{}", counter);
                *counter += 1;

                // Check if we have a pre-snapshot for this expression
                if let Some(snapshot_var) = self.old_snapshots.get(&old_var_name) {
                    let ty = self.infer_expr_type(inner);
                    let llvm_ty = self.type_to_llvm(&ty);
                    let result = self.next_temp(counter);
                    let ir = format!(
                        "  {} = load {}, {}* %{}\n",
                        result, llvm_ty, llvm_ty, snapshot_var
                    );
                    Ok((result, ir))
                } else {
                    // Fallback: just evaluate the expression (for non-ensures contexts)
                    self.generate_expr(inner, counter)
                }
            }

            // Assert expression
            Expr::Assert { condition, message } => {
                self.generate_assert(condition, message.as_deref(), counter)
            }

            // Assume expression (verification hint, no runtime effect in release)
            Expr::Assume(inner) => {
                if self.release_mode {
                    // In release mode, assume is a no-op
                    Ok(("0".to_string(), String::new()))
                } else {
                    // In debug mode, assume acts like assert but with different error message
                    self.generate_assume(inner, counter)
                }
            }

            // Lambda expression with captures
            Expr::Lambda {
                params,
                body,
                captures: _,
            } => {
                // Generate a unique function name for this lambda
                let lambda_name = format!("__lambda_{}", self.label_counter);
                self.label_counter += 1;

                // Find captured variables by analyzing free variables in lambda body
                let capture_names = self.find_lambda_captures(params, body);

                // Collect captured variable info from current scope
                let mut captured_vars: Vec<(String, ResolvedType, String)> = Vec::new();
                let mut capture_ir = String::new();

                for cap_name in &capture_names {
                    if let Some(local) = self.locals.get(cap_name) {
                        let ty = local.ty.clone();
                        // Load captured value if it's a local variable
                        if local.is_param() {
                            // Parameters are already values, use directly
                            captured_vars.push((
                                cap_name.clone(),
                                ty,
                                format!("%{}", local.llvm_name),
                            ));
                        } else if local.is_ssa() {
                            // SSA values are already the value itself, use directly
                            // llvm_name for SSA includes % prefix (e.g., "%5") or is a literal (e.g., "10")
                            captured_vars.push((cap_name.clone(), ty, local.llvm_name.clone()));
                        } else {
                            // Load from alloca
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

                // Build parameter list (original params + captured vars)
                let mut param_strs = Vec::new();
                let mut param_types = Vec::new();

                // First add captured variables as parameters (they come first)
                for (cap_name, cap_ty, _) in &captured_vars {
                    let llvm_ty = self.type_to_llvm(cap_ty);
                    param_strs.push(format!("{} %__cap_{}", llvm_ty, cap_name));
                    param_types.push(llvm_ty);
                }

                // Then add original lambda parameters
                for p in params {
                    let ty = self.ast_type_to_resolved(&p.ty.node);
                    let llvm_ty = self.type_to_llvm(&ty);
                    param_strs.push(format!("{} %{}", llvm_ty, p.name.node));
                    param_types.push(llvm_ty);
                }

                // Store current function state
                let saved_function = self.current_function.clone();
                let saved_locals = self.locals.clone();

                // Set up lambda context
                self.current_function = Some(lambda_name.clone());
                self.locals.clear();

                // Register captured variables as locals (using capture parameter names)
                for (cap_name, cap_ty, _) in &captured_vars {
                    self.locals.insert(
                        cap_name.clone(),
                        LocalVar::param(cap_ty.clone(), format!("__cap_{}", cap_name)),
                    );
                }

                // Register original parameters as locals
                for p in params {
                    let ty = self.ast_type_to_resolved(&p.ty.node);
                    self.locals.insert(
                        p.name.node.clone(),
                        LocalVar::param(ty, p.name.node.clone()),
                    );
                }

                // Generate lambda body
                let mut lambda_counter = 0;
                let (body_val, body_ir) = self.generate_expr(body, &mut lambda_counter)?;

                // Build lambda function IR
                let mut lambda_ir = format!(
                    "define i64 @{}({}) {{\nentry:\n",
                    lambda_name,
                    param_strs.join(", ")
                );
                lambda_ir.push_str(&body_ir);
                lambda_ir.push_str(&format!("  ret i64 {}\n}}\n", body_val));

                // Store lambda function for later emission
                self.lambda_functions.push(lambda_ir);

                // Restore function context
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

                // Store lambda info for Let statement to pick up
                if captured_vars.is_empty() {
                    self.last_lambda_info = None;
                    Ok((fn_ptr_tmp, capture_ir))
                } else {
                    // Store closure info with captured variable values
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

            // Try expression: expr? - propagate Err early, continue with Ok value
            // User-defined enum layout: %EnumName = type { i32 tag, { i64 } payload }
            Expr::Try(inner) => {
                // Determine the LLVM type name from the inner expression's type
                let inner_type = self.infer_expr_type(inner);
                let llvm_type = self.type_to_llvm(&inner_type);

                let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;
                let mut ir = inner_ir;

                ir.push_str("  ; Try expression (?)\n");

                // Extract tag (field 0, i32)
                let tag = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {} {}, 0\n",
                    tag, llvm_type, inner_val
                ));

                // Check if Err (tag != 0, i.e., not Ok)
                let is_err = self.next_temp(counter);
                let err_label = self.next_label("try_err");
                let ok_label = self.next_label("try_ok");
                let merge_label = self.next_label("try_merge");

                ir.push_str(&format!("  {} = icmp ne i32 {}, 0\n", is_err, tag));
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n\n",
                    is_err, err_label, ok_label
                ));

                // Err branch: return the whole enum value as-is (early return)
                ir.push_str(&format!("{}:\n", err_label));
                ir.push_str(&format!(
                    "  ret {} {}  ; early return on Err\n\n",
                    llvm_type, inner_val
                ));

                // Ok branch: extract payload value (field 1, then field 0 of the payload struct)
                ir.push_str(&format!("{}:\n", ok_label));
                let value = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {} {}, 1, 0\n",
                    value, llvm_type, inner_val
                ));
                ir.push_str(&format!("  br label %{}\n\n", merge_label));

                // Merge block
                ir.push_str(&format!("{}:\n", merge_label));

                Ok((value, ir))
            }

            // Unwrap expression: expr! - panic on Err/None, continue with value
            // User-defined enum layout: %EnumName = type { i32 tag, { i64 } payload }
            Expr::Unwrap(inner) => {
                // Determine the LLVM type name from the inner expression's type
                let inner_type = self.infer_expr_type(inner);
                let llvm_type = self.type_to_llvm(&inner_type);

                let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;
                let mut ir = inner_ir;

                ir.push_str("  ; Unwrap expression\n");

                // Extract tag (field 0, i32)
                let tag = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {} {}, 0\n",
                    tag, llvm_type, inner_val
                ));

                // Check if Err/None (tag != 0)
                let is_err = self.next_temp(counter);
                let err_label = self.next_label("unwrap_err");
                let ok_label = self.next_label("unwrap_ok");

                ir.push_str(&format!("  {} = icmp ne i32 {}, 0\n", is_err, tag));
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n\n",
                    is_err, err_label, ok_label
                ));

                // Err branch: panic/abort
                ir.push_str(&format!("{}:\n", err_label));
                ir.push_str("  call i32 @puts(ptr getelementptr ([22 x i8], ptr @.unwrap_panic_msg, i64 0, i64 0))\n");
                ir.push_str("  call void @abort()\n");
                ir.push_str("  unreachable\n\n");

                // Ok branch: extract value (field 1, field 0 of payload struct)
                ir.push_str(&format!("{}:\n", ok_label));
                let value = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {} {}, 1, 0\n",
                    value, llvm_type, inner_val
                ));

                // Track that we need the panic message and abort declaration
                self.needs_unwrap_panic = true;

                Ok((value, ir))
            }

            // Error nodes should not reach codegen
            Expr::Error { message, .. } => Err(CodegenError::Unsupported(format!(
                "Parse error in expression: {}",
                message
            ))),

            // Lazy and Force expressions - delegate to visitor
            Expr::Lazy(inner) => {
                use crate::visitor::ExprVisitor;
                self.visit_lazy(inner, counter)
            }
            Expr::Force(inner) => {
                use crate::visitor::ExprVisitor;
                self.visit_force(inner, counter)
            }
        }
    }
}
