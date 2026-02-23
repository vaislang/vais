//! Expression code generation for LLVM IR.
//!
//! This module implements the main expression dispatcher (`generate_expr`) which routes
//! each expression type to the appropriate code generation method.
//!
//! # Submodules
//!
//! - `special` — Spawn, Comptime, Range (extracted from inline match arms)
//!
//! Most expression types delegate to pre-existing helper modules:
//! - `expr_helpers` — Binary, Unary, Ident, Assign, AssignOp, Cast
//! - `expr_helpers_control` — Ternary, If
//! - `expr_helpers_data` — Array, Tuple, Index, Field
//! - `expr_helpers_misc` — Lambda, Try, Unwrap, Await
//! - `expr_helpers_call/method_call` — MethodCall, StaticMethodCall
//! - `generate_expr_call` — Call
//! - `generate_expr_struct` — StructLit
//! - `generate_expr_loop` — RangeForLoop
//! - `control_flow/match_gen` — Match
//! - `contracts/assert_assume` — Assert, Assume
//! - `string_ops` — String methods
//! - `helpers` — Slice

use vais_ast::*;
use vais_types::ResolvedType;

use crate::{CodeGenerator, CodegenError, CodegenResult, LoopLabels};

mod special;

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
                self.strings.constants.push((name, s.clone()));
                Ok((gep, String::new()))
            }
            Expr::StringInterp(parts) => {
                // Desugar string interpolation into a format() call.
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
                let mut args: Vec<Spanned<Expr>> = Vec::with_capacity(interp_args.len() + 1);
                args.push(Spanned::new(Expr::String(fmt_string), expr.span));
                args.extend(interp_args);
                self.generate_format_call(&args, counter, expr.span)
            }
            Expr::Unit => Ok(("void".to_string(), String::new())),

            Expr::Ident(name) => self.generate_ident_expr(name, counter),

            Expr::SelfCall => {
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
                self.generate_assign_expr(target, value, counter)
            }

            // Compound assignment (+=, -=, etc.)
            Expr::AssignOp { op, target, value } => {
                self.generate_assign_op_expr(op, target, value, counter)
            }

            // Array literal: [a, b, c]
            Expr::Array(elements) => self.generate_array_expr(elements, counter),

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
            Expr::Tuple(elements) => self.generate_tuple_expr(elements, counter),

            // Struct literal: Point{x:1, y:2}
            Expr::StructLit { name, fields } => {
                self.generate_expr_struct_lit(name, fields, counter)
            }

            // Index: arr[idx] or slice: arr[start..end]
            Expr::Index {
                expr: array_expr,
                index,
            } => self.generate_index_expr(array_expr, index, counter),

            // Field access: obj.field
            Expr::Field {
                expr: obj_expr,
                field,
            } => self.generate_field_expr(obj_expr, field, counter),

            // Method call: obj.method(args)
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => self.generate_method_call_expr(receiver, method, args, counter),

            // Static method call: Type.method(args)
            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => self.generate_static_method_call_expr(type_name, method, args, counter),

            // Spread: ..expr (handled within array generation; standalone generates inner)
            Expr::Spread(inner) => self.generate_expr(inner, counter),

            // Reference: &expr
            Expr::Ref(inner) => {
                // Special case: &[elem, ...] array literal -> slice fat pointer { i8*, i64 }
                if let Expr::Array(elements) = &inner.node {
                    let len = elements.len();
                    let mut ir = String::new();

                    // Infer element type
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

                    // Get pointer to first element
                    let data_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i64 0, i64 0\n",
                        data_ptr, arr_ty, arr_ty, arr_ptr
                    ));

                    // Bitcast to i8*
                    let data_i8 = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = bitcast {}* {} to i8*\n",
                        data_i8, elem_ty, data_ptr
                    ));

                    // Build fat pointer: { i8*, i64 }
                    let fat1 = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0\n",
                        fat1, data_i8
                    ));
                    let fat2 = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1\n",
                        fat2, fat1, len
                    ));

                    return Ok((fat2, ir));
                }

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
            Expr::Cast { expr, ty } => self.generate_cast_expr(expr, ty, counter),

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
            } => self.generate_range_expr(start, end, *inclusive, counter),

            // Await expression: poll the future until Ready
            Expr::Await(inner) => self.generate_await_expr(inner, counter),

            // Spawn expression: create a concurrent task
            Expr::Spawn(inner) => self.generate_spawn_expr(inner, counter),

            // Yield expression: yield a value from a generator.
            Expr::Yield(inner) => {
                let (val, ir) = self.generate_expr(inner, counter)?;
                Ok((val, ir))
            }

            // Comptime expression: evaluate at compile time and emit constant
            Expr::Comptime { body } => self.generate_comptime_expr(body, counter),

            // Macro invocation (should be expanded before codegen)
            Expr::MacroInvoke(invoke) => Err(CodegenError::TypeError(format!(
                "Unexpanded macro invocation: {}! - macros must be expanded before code generation",
                invoke.name.node
            ))),

            // Old expression for contract ensures clauses
            Expr::Old(inner) => {
                // old(expr) references a pre-snapshot value
                let old_var_name = format!("__old_{}", counter);
                *counter += 1;

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
            } => self.generate_lambda_expr(params, body, capture_mode, counter),

            // Try expression: expr? - propagate Err early, continue with Ok value
            Expr::Try(inner) => self.generate_try_expr(inner, counter),

            // Unwrap expression: expr! - panic on Err/None, continue with value
            Expr::Unwrap(inner) => self.generate_unwrap_expr(inner, counter),

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
