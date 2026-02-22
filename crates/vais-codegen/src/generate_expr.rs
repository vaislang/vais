//! Expression code generation for LLVM IR.

use vais_ast::*;
use vais_types::ResolvedType;

use crate::{
    format_did_you_mean, suggest_similar, ClosureInfo, CodeGenerator, CodegenError, CodegenResult,
    LocalVar, LoopLabels,
};

impl CodeGenerator {
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
                self.strings.counter += 1;
                let len = s.len() + 1;
                let gep = format!(
                    "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                    len, len, name
                );
                self.strings.constants.push((name, s.clone())); // name moved after GEP creation

                // Return a getelementptr to the string constant
                Ok((gep, String::new()))
            }
            Expr::StringInterp(parts) => {
                // Desugar string interpolation into a format() call.
                // Build a format string with {} placeholders and collect expression args.
                let mut format_str_parts = Vec::with_capacity(parts.len());
                let mut interp_args = Vec::with_capacity(parts.len());
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
                let mut args: Vec<Spanned<Expr>> = Vec::with_capacity(interp_args.len() + 1);
                args.push(Spanned::new(Expr::String(fmt_string), expr.span));
                args.extend(interp_args);
                self.generate_format_call(&args, counter, expr.span)
            }
            Expr::Unit => Ok(("void".to_string(), String::new())),

            Expr::Ident(name) => self.generate_ident_expr(name, counter),

            Expr::SelfCall => {
                // @ refers to current function
                if let Some(fn_name) = &self.fn_ctx.current_function {
                    Ok((format!("@{}", fn_name), String::new()))
                } else {
                    Err(CodegenError::UndefinedFunction("@".to_string()))
                }
            }

            Expr::Binary { op, left, right } => {
                self.generate_binary_expr(op, left, right, counter, expr.span)
            }

            Expr::Unary { op, expr: inner } => {
                self.generate_unary_expr(op, inner, counter, expr.span)
            }

            Expr::Ternary { cond, then, else_ } => {
                self.generate_ternary_expr(cond, then, else_, counter)
            }

            Expr::Call { func, args } => self.generate_expr_call(func, args, expr.span, counter),

            // If/Else expression with basic blocks
            Expr::If { cond, then, else_ } => {
                self.generate_if_expr(cond, then, else_.as_ref(), counter)
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
                self.fn_ctx.loop_stack.push(LoopLabels {
                    continue_label: loop_start.clone(), // keep: used in continue stmt
                    break_label: loop_end.clone(),      // keep: used in break stmt
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
                    let (cond_bool, conv_ir) =
                        self.generate_cond_to_i1(iter_expr, &cond_val, counter);
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

                self.fn_ctx.loop_stack.pop();

                // Loop returns void by default (use break with value for expression)
                Ok(("0".to_string(), ir))
            }

            // While loop expression
            Expr::While { condition, body } => {
                let loop_start = self.next_label("while.start");
                let loop_body = self.next_label("while.body");
                let loop_end = self.next_label("while.end");

                // Push loop labels for break/continue
                self.fn_ctx.loop_stack.push(LoopLabels {
                    continue_label: loop_start.clone(), // keep: used in continue stmt
                    break_label: loop_end.clone(),      // keep: used in break stmt
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

                self.fn_ctx.loop_stack.pop();

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
                    if let Some(local) = self.fn_ctx.locals.get(name).cloned() {
                        if !local.is_param() {
                            let llvm_ty = self.type_to_llvm(&local.ty);

                            // If the variable was originally SSA, we need to upgrade it to alloca
                            // because SSA values cannot be reassigned with store.
                            if local.is_ssa() {
                                let alloca_name = format!("{}.alloca.{}", name, counter);
                                *counter += 1;
                                ir.push_str(&format!("  %{} = alloca {}\n", alloca_name, llvm_ty));
                                ir.push_str(&format!(
                                    "  store {} {}, {}* %{}\n",
                                    llvm_ty, val, llvm_ty, alloca_name
                                ));
                                // Upgrade from SSA to Alloca so future reads use load
                                self.fn_ctx.locals.insert(
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
                    // Infer the pointee type for correct store instruction
                    let ptr_type = self.infer_expr_type(inner);
                    let pointee_llvm = match &ptr_type {
                        ResolvedType::Pointer(inner) => self.type_to_llvm(inner),
                        ResolvedType::Ref(inner) => self.type_to_llvm(inner),
                        ResolvedType::RefMut(inner) => self.type_to_llvm(inner),
                        _ => "i64".to_string(),
                    };
                    // Store value at the pointed-to location
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        pointee_llvm, val, pointee_llvm, ptr_val
                    ));
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
                        if let Some(local) = self.fn_ctx.locals.get(var_name.as_str()).cloned() {
                            if let ResolvedType::Named {
                                name: type_name, ..
                            } = &local.ty
                            {
                                // First check struct
                                if let Some(struct_info) =
                                    self.types.structs.get(type_name).cloned()
                                {
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
                                else if let Some(union_info) =
                                    self.types.unions.get(type_name).cloned()
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

                // Detect float type for choosing float vs integer ops
                let target_type = self.infer_expr_type(target);
                let is_float = matches!(target_type, ResolvedType::F64 | ResolvedType::F32);
                let operand_llvm = if is_float {
                    self.type_to_llvm(&target_type)
                } else {
                    "i64".to_string()
                };

                let op_str = if is_float {
                    match op {
                        BinOp::Add => "fadd",
                        BinOp::Sub => "fsub",
                        BinOp::Mul => "fmul",
                        BinOp::Div => "fdiv",
                        BinOp::Mod => "frem",
                        _ => unreachable!("BinOp {:?} in {} codegen path", op, "compound_assign_float"),
                    }
                } else {
                    match op {
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
                        _ => unreachable!("BinOp {:?} in {} codegen path", op, "compound_assign"),
                    }
                };

                // Add division by zero check for sdiv and srem (only for integer ops)
                if !is_float && matches!(op, BinOp::Div | BinOp::Mod) {
                    let zero_check = self.next_temp(counter);
                    let div_ok_label = self.next_label("div_ok");
                    let div_zero_label = self.next_label("div_zero");

                    ir.push_str(&format!(
                        "  {} = icmp eq i64 {}, 0\n",
                        zero_check, rhs_val
                    ));
                    ir.push_str(&format!(
                        "  br i1 {}, label %{}, label %{}\n",
                        zero_check, div_zero_label, div_ok_label
                    ));
                    ir.push_str(&format!("{}:\n", div_zero_label));
                    ir.push_str("  call void @abort()\n");
                    ir.push_str("  unreachable\n");
                    ir.push_str(&format!("{}:\n", div_ok_label));
                    self.fn_ctx.current_block = div_ok_label;
                    self.needs_unwrap_panic = true;
                }

                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = {} {} {}, {}\n",
                    result, op_str, operand_llvm, current_val, rhs_val
                ));

                // Store back
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

            // Tuple literal: (a, b, c) — including nested tuples like (1, (2, 3)).
            // Infer each element's LLVM type to correctly handle non-i64 elements such as
            // inner tuples (which are struct types like { i64, i64 }).
            Expr::Tuple(elements) => {
                let mut ir = String::new();

                // Infer the LLVM type of each element before generating code.
                // This ensures nested tuples use the correct struct type instead of i64.
                let elem_resolved_types: Vec<ResolvedType> = elements
                    .iter()
                    .map(|e| self.infer_expr_type(e))
                    .collect();
                let elem_llvm_types: Vec<String> = elem_resolved_types
                    .iter()
                    .map(|t| self.type_to_llvm(t))
                    .collect();

                // Build tuple type string from actual element types.
                let tuple_ty = format!("{{ {} }}", elem_llvm_types.join(", "));

                // Allocate tuple on stack
                let tuple_ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = alloca {}\n", tuple_ptr, tuple_ty));

                // Store each element using its actual LLVM type.
                for (i, elem) in elements.iter().enumerate() {
                    let (val, elem_ir) = self.generate_expr(elem, counter)?;
                    ir.push_str(&elem_ir);

                    let elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i32 0, i32 {}\n",
                        elem_ptr, tuple_ty, tuple_ty, tuple_ptr, i
                    ));
                    let elem_ty = &elem_llvm_types[i];
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        elem_ty, val, elem_ty, elem_ptr
                    ));
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
                self.generate_expr_struct_lit(name, fields, counter)
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
                let result =
                    if matches!(arr_type, ResolvedType::Slice(_) | ResolvedType::SliceMut(_)) {
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
                    if let Some(struct_info) = self.types.structs.get(type_name).cloned() {
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
                    else if let Some(union_info) = self.types.unions.get(type_name).cloned() {
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
                    if let Some(local) = self.fn_ctx.locals.get("self") {
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
                    .types
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
                    .types
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
                    if let Some(local) = self.fn_ctx.locals.get(name.as_str()).cloned() {
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

                // Infer the pointee type from the pointer expression
                let ptr_type = self.infer_expr_type(inner);
                let pointee_llvm = match &ptr_type {
                    ResolvedType::Pointer(inner) => self.type_to_llvm(inner),
                    ResolvedType::Ref(inner) => self.type_to_llvm(inner),
                    ResolvedType::RefMut(inner) => self.type_to_llvm(inner),
                    _ => "i64".to_string(),
                };

                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    result, pointee_llvm, pointee_llvm, ptr_val
                ));

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
                        // Infer the source type to get the correct pointer type
                        let src_type = self.infer_expr_type(expr);
                        let src_llvm = self.type_to_llvm(&src_type);
                        // If source is already a pointer type, use it; otherwise assume i64*
                        let ptr_type = if src_llvm.ends_with('*') {
                            src_llvm
                        } else {
                            format!("{}*", src_llvm)
                        };
                        ir.push_str(&format!(
                            "  {} = ptrtoint {} {} to {}\n",
                            result, ptr_type, val, llvm_type
                        ));
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

            // Range expression: produce { i64 start, i64 end, i1 inclusive } struct
            Expr::Range {
                start,
                end,
                inclusive,
            } => {
                let mut ir = String::new();

                let (start_val, start_ir) = if let Some(s) = start {
                    self.generate_expr(s, counter)?
                } else {
                    ("0".to_string(), String::new())
                };
                ir.push_str(&start_ir);

                let (end_val, end_ir) = if let Some(e) = end {
                    self.generate_expr(e, counter)?
                } else {
                    (format!("{}", i64::MAX), String::new())
                };
                ir.push_str(&end_ir);

                let incl_val = if *inclusive { "1" } else { "0" };

                // Build struct via insertvalue chain
                let range_type = "{ i64, i64, i1 }";
                let t1 = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = insertvalue {} undef, i64 {}, 0\n",
                    t1, range_type, start_val
                ));
                let t2 = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = insertvalue {} {}, i64 {}, 1\n",
                    t2, range_type, t1, end_val
                ));
                let t3 = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = insertvalue {} {}, i1 {}, 2\n",
                    t3, range_type, t2, incl_val
                ));

                Ok((t3, ir))
            }

            // Await expression: poll the future until Ready
            Expr::Await(inner) => self.generate_await_expr(inner, counter),

            // Spawn expression: create a concurrent task
            // For async function calls, the inner expression already returns a state pointer
            // (from the create function generated by generate_async_function).
            // For sync expressions, spawn wraps the value in a Future struct:
            //   malloc {i64 state=-1, i64 result=value}, return pointer as i64.
            Expr::Spawn(inner) => {
                let inner_type = self.infer_expr_type(inner);
                let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;

                // If inner is already a Future (async call), pass through — the create
                // function already returned a state pointer.
                // Check both infer_expr_type (TC-registered Future type) and is_async
                // flag (direct async function lookup) for robustness.
                let is_async_call = if let Expr::Call { func, .. } = &inner.node {
                    if let Expr::Ident(name) = &func.node {
                        self.types
                            .functions
                            .get(name.as_str())
                            .is_some_and(|info| info.signature.is_async)
                    } else {
                        false
                    }
                } else {
                    false
                };
                if matches!(inner_type, vais_types::ResolvedType::Future(_)) || is_async_call {
                    return Ok((inner_val, inner_ir));
                }

                // Sync value: wrap in an immediate Future state struct {i64 state, i64 result}
                // state = -1 means "completed", result = the evaluated value.
                let mut ir = inner_ir;
                let state_ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i64 @malloc(i64 16)\n",
                    state_ptr
                ));
                let typed_ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = inttoptr i64 {} to {{i64, i64}}*\n",
                    typed_ptr, state_ptr
                ));
                // Store state = -1 (completed)
                let state_field = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr {{i64, i64}}, {{i64, i64}}* {}, i32 0, i32 0\n",
                    state_field, typed_ptr
                ));
                ir.push_str(&format!("  store i64 -1, i64* {}\n", state_field));
                // Store result value
                let result_field = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr {{i64, i64}}, {{i64, i64}}* {}, i32 0, i32 1\n",
                    result_field, typed_ptr
                ));
                ir.push_str(&format!("  store i64 {}, i64* {}\n", inner_val, result_field));

                self.needs_sync_spawn_poll = true;
                Ok((state_ptr, ir))
            }

            // Yield expression: yield a value from a generator.
            // Evaluates the inner expression and returns the yielded value.
            // In the current synchronous model, yield does not suspend execution;
            // coroutine-based state machine transformation is a future enhancement.
            Expr::Yield(inner) => {
                let (val, ir) = self.generate_expr(inner, counter)?;
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
                        self.strings.counter += 1;
                        self.strings.constants.push((name.clone(), s.clone()));
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
                if let Some(snapshot_var) = self.contracts.old_snapshots.get(&old_var_name) {
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
                capture_mode,
                ..
            } => {
                // Generate a unique function name for this lambda
                let lambda_name = format!("__lambda_{}", self.fn_ctx.label_counter);
                self.fn_ctx.label_counter += 1;

                // Find captured variables by analyzing free variables in lambda body
                let capture_names = self.find_lambda_captures(params, body);

                // Collect captured variable info from current scope
                let mut captured_vars: Vec<(String, ResolvedType, String)> = Vec::new();
                let mut capture_ir = String::new();

                let is_ref_capture = matches!(
                    capture_mode,
                    vais_ast::CaptureMode::ByRef | vais_ast::CaptureMode::ByMutRef
                );

                for cap_name in &capture_names {
                    if let Some(local) = self.fn_ctx.locals.get(cap_name) {
                        let ty = local.ty.clone();

                        if is_ref_capture {
                            // ByRef/ByMutRef: pass pointer to the captured variable
                            if local.is_param() || local.is_ssa() {
                                // Params and SSA values don't have an alloca address.
                                // Spill to alloca so we can take a pointer.
                                let llvm_ty = self.type_to_llvm(&ty);
                                let spill_name = format!("__refcap_{}", cap_name);
                                let spill_ptr = format!("%{}", spill_name);
                                capture_ir
                                    .push_str(&format!("  {} = alloca {}\n", spill_ptr, llvm_ty));
                                let val = if local.is_param() {
                                    format!("%{}", local.llvm_name)
                                } else {
                                    local.llvm_name.clone()
                                };
                                capture_ir.push_str(&format!(
                                    "  store {} {}, {}* {}\n",
                                    llvm_ty, val, llvm_ty, spill_ptr
                                ));
                                captured_vars.push((cap_name.clone(), ty, spill_ptr));
                            } else {
                                // Alloca: pass the pointer directly
                                captured_vars.push((
                                    cap_name.clone(),
                                    ty,
                                    format!("%{}", local.llvm_name),
                                ));
                            }
                        } else {
                            // ByValue/Move: load and pass the value
                            if local.is_param() {
                                captured_vars.push((
                                    cap_name.clone(),
                                    ty,
                                    format!("%{}", local.llvm_name),
                                ));
                            } else if local.is_ssa() {
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
                }

                // Build parameter list (original params + captured vars)
                let mut param_strs = Vec::new();
                let mut param_types = Vec::new();

                // First add captured variables as parameters (they come first)
                for (cap_name, cap_ty, _) in &captured_vars {
                    let llvm_ty = self.type_to_llvm(cap_ty);
                    if is_ref_capture {
                        // Reference capture: parameter is a pointer
                        param_strs.push(format!("{}* %__cap_{}", llvm_ty, cap_name));
                        param_types.push(format!("{}*", llvm_ty));
                    } else {
                        param_strs.push(format!("{} %__cap_{}", llvm_ty, cap_name));
                        param_types.push(llvm_ty);
                    }
                }

                // Then add original lambda parameters
                for p in params {
                    let ty = self.ast_type_to_resolved(&p.ty.node);
                    let llvm_ty = self.type_to_llvm(&ty);
                    param_strs.push(format!("{} %{}", llvm_ty, p.name.node));
                    param_types.push(llvm_ty);
                }

                // Store current function state (move instead of clone to avoid HashMap allocation).
                // SAFETY: if generate_expr below returns Err, the entire codegen aborts,
                // so empty self.fn_ctx.locals after take is acceptable (never accessed post-error).
                let saved_function = self.fn_ctx.current_function.take();
                let saved_locals = std::mem::take(&mut self.fn_ctx.locals);

                // Set up lambda context
                self.fn_ctx.current_function = Some(lambda_name.clone());

                // Register captured variables as locals (using capture parameter names)
                for (cap_name, cap_ty, _) in &captured_vars {
                    if is_ref_capture {
                        // ByRef/ByMutRef: parameter is a pointer, register as alloca
                        // so that reads generate load instructions from the pointer
                        self.fn_ctx.locals.insert(
                            cap_name.clone(),
                            LocalVar::alloca(cap_ty.clone(), format!("__cap_{}", cap_name)),
                        );
                    } else {
                        self.fn_ctx.locals.insert(
                            cap_name.clone(),
                            LocalVar::param(cap_ty.clone(), format!("__cap_{}", cap_name)),
                        );
                    }
                }

                // Register original parameters as locals
                for p in params {
                    let ty = self.ast_type_to_resolved(&p.ty.node);
                    self.fn_ctx.locals.insert(
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
                self.lambdas.generated_ir.push(lambda_ir);

                // Restore function context
                self.fn_ctx.current_function = saved_function;
                self.fn_ctx.locals = saved_locals;

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
                    self.lambdas.last_lambda_info = None;
                    Ok((fn_ptr_tmp, capture_ir))
                } else {
                    // Store closure info with captured variable values
                    self.lambdas.last_lambda_info = Some(ClosureInfo {
                        func_name: lambda_name.clone(),
                        captures: captured_vars
                            .iter()
                            .map(|(name, _, val)| (name.clone(), val.clone()))
                            .collect(),
                        is_ref_capture,
                    });
                    Ok((fn_ptr_tmp, capture_ir))
                }
            }

            // Try expression: expr? - propagate Err early, continue with Ok value
            Expr::Try(inner) => {
                // Determine the LLVM type name from the inner expression's type
                let inner_type = self.infer_expr_type(inner);
                let llvm_type = self.type_to_llvm(&inner_type);

                let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;
                let mut ir = inner_ir;

                ir.push_str("  ; Try expression (?)\n");

                // Determine the tag type based on the inner type:
                // - Optional/Result use i8 tag: { i8, T }
                // - User-defined enums use i32 tag: { i32, { T } }
                let (tag_type, extract_payload) = match &inner_type {
                    ResolvedType::Optional(_) | ResolvedType::Result(_, _) => {
                        ("i8", false) // { i8, T } — payload at field 1 directly
                    }
                    _ => {
                        ("i32", true) // { i32, { T } } — payload at field 1, then field 0
                    }
                };

                // Extract tag (field 0)
                let tag = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {} {}, 0\n",
                    tag, llvm_type, inner_val
                ));

                // Check if Err (tag != 0, i.e., not Ok/Some)
                let is_err = self.next_temp(counter);
                let err_label = self.next_label("try_err");
                let ok_label = self.next_label("try_ok");
                let merge_label = self.next_label("try_merge");

                ir.push_str(&format!("  {} = icmp ne {} {}, 0\n", is_err, tag_type, tag));
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

                // Ok branch: extract payload value
                ir.push_str(&format!("{}:\n", ok_label));
                let value = self.next_temp(counter);
                if extract_payload {
                    // User-defined enum: payload at field 1, then field 0 of payload struct
                    ir.push_str(&format!(
                        "  {} = extractvalue {} {}, 1, 0\n",
                        value, llvm_type, inner_val
                    ));
                } else {
                    // Optional/Result: payload at field 1 directly
                    ir.push_str(&format!(
                        "  {} = extractvalue {} {}, 1\n",
                        value, llvm_type, inner_val
                    ));
                }
                ir.push_str(&format!("  br label %{}\n\n", merge_label));

                // Merge block
                ir.push_str(&format!("{}:\n", merge_label));

                Ok((value, ir))
            }

            // Unwrap expression: expr! - panic on Err/None, continue with value
            Expr::Unwrap(inner) => {
                // Determine the LLVM type name from the inner expression's type
                let inner_type = self.infer_expr_type(inner);
                let llvm_type = self.type_to_llvm(&inner_type);

                let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;
                let mut ir = inner_ir;

                ir.push_str("  ; Unwrap expression\n");

                // Determine the tag type based on the inner type:
                // - Optional/Result use i8 tag: { i8, T }
                // - User-defined enums use i32 tag: { i32, { T } }
                let (tag_type, extract_payload) = match &inner_type {
                    ResolvedType::Optional(_) | ResolvedType::Result(_, _) => {
                        ("i8", false) // { i8, T } — payload at field 1 directly
                    }
                    _ => {
                        ("i32", true) // { i32, { T } } — payload at field 1, then field 0
                    }
                };

                // Extract tag (field 0)
                let tag = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {} {}, 0\n",
                    tag, llvm_type, inner_val
                ));

                // Check if Err/None (tag != 0)
                let is_err = self.next_temp(counter);
                let err_label = self.next_label("unwrap_err");
                let ok_label = self.next_label("unwrap_ok");

                ir.push_str(&format!("  {} = icmp ne {} {}, 0\n", is_err, tag_type, tag));
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n\n",
                    is_err, err_label, ok_label
                ));

                // Err branch: panic/abort
                ir.push_str(&format!("{}:\n", err_label));
                ir.push_str("  call i32 @puts(ptr getelementptr ([22 x i8], ptr @.unwrap_panic_msg, i64 0, i64 0))\n");
                ir.push_str("  call void @abort()\n");
                ir.push_str("  unreachable\n\n");

                // Ok branch: extract value
                ir.push_str(&format!("{}:\n", ok_label));
                let value = self.next_temp(counter);
                if extract_payload {
                    // User-defined enum: payload at field 1, then field 0 of payload struct
                    ir.push_str(&format!(
                        "  {} = extractvalue {} {}, 1, 0\n",
                        value, llvm_type, inner_val
                    ));
                } else {
                    // Optional/Result: payload at field 1 directly
                    ir.push_str(&format!(
                        "  {} = extractvalue {} {}, 1\n",
                        value, llvm_type, inner_val
                    ));
                }

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
