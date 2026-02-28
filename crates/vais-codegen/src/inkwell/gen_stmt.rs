//! Statement and control flow code generation.
//!
//! Handles blocks, statements, if/else, loops, break/continue,
//! and defer cleanup.

use inkwell::values::{BasicValue, BasicValueEnum};
use inkwell::IntPredicate;

use vais_ast::{Expr, IfElse, Pattern, Spanned, Stmt};
use vais_types::ResolvedType;

use super::generator::{InkwellCodeGenerator, LoopContext};
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn generate_block(
        &mut self,
        stmts: &[Spanned<Stmt>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let mut last_value: BasicValueEnum =
            self.context.struct_type(&[], false).const_zero().into();

        for stmt in stmts {
            // Stop generating after a terminator (return/break/continue)
            if let Some(block) = self.builder.get_insert_block() {
                if block.get_terminator().is_some() {
                    break;
                }
            }
            last_value = self.generate_stmt(&stmt.node)?;
        }

        Ok(last_value)
    }

    pub(super) fn generate_stmt(&mut self, stmt: &Stmt) -> CodegenResult<BasicValueEnum<'ctx>> {
        match stmt {
            Stmt::Let {
                name, ty, value, ..
            } => {
                // Track struct type from the value expression before generating
                let struct_type_name = self.infer_value_struct_type(&value.node);
                let is_lambda = matches!(&value.node, Expr::Lambda { .. });
                let is_lazy = matches!(&value.node, Expr::Lazy(_));

                // Clear last lambda/lazy info before generating
                self._last_lambda_info = None;
                self._last_lazy_info = None;
                let val = self.generate_expr(&value.node)?;

                // If this was a lambda binding, record the lambda info
                if is_lambda {
                    if let Some((lambda_fn_name, captures)) = self._last_lambda_info.take() {
                        self.lambda_bindings
                            .insert(name.node.clone(), (lambda_fn_name, captures));
                    }
                }
                // If this was a lazy binding, record the lazy thunk info
                if is_lazy {
                    if let Some((thunk_name, captures)) = self._last_lazy_info.take() {
                        self.lazy_bindings
                            .insert(name.node.clone(), (thunk_name, captures));
                    }
                }
                // Track resolved type for variables with type annotations
                // (used for element/pointee type inference in slice indexing and deref)
                if let Some(t) = ty.as_ref() {
                    let resolved = self.ast_type_to_resolved(&t.node);
                    self.var_resolved_types.insert(name.node.clone(), resolved);
                }

                let var_type = if let Some(t) = ty.as_ref() {
                    let resolved = self.ast_type_to_resolved(&t.node);
                    self.type_mapper.map_type(&resolved)
                } else if val.is_struct_value() {
                    // Use actual struct type for struct values
                    val.get_type()
                } else if val.is_float_value() {
                    // Keep float type
                    val.get_type()
                } else if val.is_pointer_value()
                    && matches!(
                        &value.node,
                        Expr::Array(_)
                            | Expr::Index { .. }
                            | Expr::String(_)
                            | Expr::StringInterp(_)
                    )
                {
                    // Keep pointer type for array allocations, slice results, and strings
                    val.get_type()
                } else if val.is_int_value() && val.into_int_value().get_type().get_bit_width() == 1
                {
                    // Keep i1 type for boolean values (from comparisons, bool literals)
                    val.get_type()
                } else {
                    // Default to i64 for non-struct values (backward compatible)
                    self.type_mapper.map_type(&ResolvedType::I64)
                };
                let alloca = self
                    .builder
                    .build_alloca(var_type, &name.node)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder
                    .build_store(alloca, val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.locals.insert(name.node.clone(), (alloca, var_type));

                // Record struct type for variable (from StructLit, function return type, or type annotation)
                if let Some(sn) = struct_type_name {
                    self.var_struct_types.insert(name.node.clone(), sn);
                } else if let Some(t) = ty.as_ref() {
                    if let Some(sn) = self.extract_struct_type_name(&t.node) {
                        self.var_struct_types.insert(name.node.clone(), sn);
                    }
                } else if val.is_struct_value() {
                    // Fallback: match the generated value's struct type against known structs
                    // Only use if unambiguous (exactly one struct matches the LLVM type)
                    let struct_type = val.into_struct_value().get_type();
                    let matches: Vec<_> = self
                        .generated_structs
                        .iter()
                        .filter(|(_, st)| **st == struct_type)
                        .collect();
                    if matches.len() == 1 {
                        self.var_struct_types
                            .insert(name.node.clone(), matches[0].0.clone());
                    }
                }

                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            Stmt::Expr(expr) => self.generate_expr(&expr.node),
            Stmt::Return(Some(expr)) => {
                let val = self.generate_expr(&expr.node)?;
                self.emit_defer_cleanup()?;
                self.builder
                    .build_return(Some(&val))
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            Stmt::Return(None) => {
                self.emit_defer_cleanup()?;
                self.builder
                    .build_return(None)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            Stmt::Break(value) => self.generate_break(value.as_ref().map(|v| &v.node)),
            Stmt::Continue => self.generate_continue(),
            Stmt::Defer(expr) => {
                // Add deferred expression to stack (will be executed in LIFO order before return)
                self.defer_stack.push(expr.node.clone());
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            Stmt::LetDestructure {
                pattern,
                value,
                is_mut: _,
            } => self.generate_let_destructure(&pattern.node, &value.node),
            _ => Err(CodegenError::Unsupported(format!("Statement: {:?}", stmt))),
        }
    }

    pub(super) fn generate_if_expr(
        &mut self,
        cond: &Expr,
        then_stmts: &[Spanned<Stmt>],
        else_branch: Option<&IfElse>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self.current_function.ok_or_else(|| {
            CodegenError::LlvmError("No current function for if expression".to_string())
        })?;

        // Generate condition
        let cond_val = self.generate_expr(cond)?;
        let cond_bool = if cond_val.is_int_value() {
            let int_val = cond_val.into_int_value();
            // Convert to i1 if needed
            if int_val.get_type().get_bit_width() > 1 {
                self.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        int_val,
                        int_val.get_type().const_int(0, false),
                        "cond_bool",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                int_val
            }
        } else {
            self.context.bool_type().const_int(1, false)
        };

        // Create blocks
        let then_block = self.context.append_basic_block(fn_value, "then");
        let else_block = self.context.append_basic_block(fn_value, "else");
        let merge_block = self.context.append_basic_block(fn_value, "merge");

        // Conditional branch
        self.builder
            .build_conditional_branch(cond_bool, then_block, else_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Then block
        self.builder.position_at_end(then_block);
        let then_val = self.generate_block(then_stmts)?;
        let then_end_block = self.builder.get_insert_block().unwrap();
        let then_terminated = then_end_block.get_terminator().is_some();
        if !then_terminated {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Else block
        self.builder.position_at_end(else_block);
        let else_val = if let Some(else_branch) = else_branch {
            self.generate_if_else(else_branch)?
        } else {
            self.context.struct_type(&[], false).const_zero().into()
        };
        let else_end_block = self.builder.get_insert_block().unwrap();
        let else_terminated = else_end_block.get_terminator().is_some();
        if !else_terminated {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Merge block with phi
        self.builder.position_at_end(merge_block);

        // If both branches are terminated (return/break), merge is unreachable
        if then_terminated && else_terminated {
            self.builder
                .build_unreachable()
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            return Ok(self.context.struct_type(&[], false).const_zero().into());
        }

        // Build phi node - only include non-terminated branches
        let mut incoming: Vec<(
            &dyn BasicValue<'ctx>,
            inkwell::basic_block::BasicBlock<'ctx>,
        )> = Vec::new();
        if !then_terminated {
            incoming.push((&then_val, then_end_block));
        }
        if !else_terminated {
            incoming.push((&else_val, else_end_block));
        }

        if incoming.len() == 1 {
            // Only one branch reaches merge - no phi needed
            Ok(incoming[0].0.as_basic_value_enum())
        } else if !incoming.is_empty() && then_val.get_type() == else_val.get_type() {
            let phi = self
                .builder
                .build_phi(then_val.get_type(), "if_result")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            for (val, block) in &incoming {
                phi.add_incoming(&[(*val, *block)]);
            }
            Ok(phi.as_basic_value())
        } else if !incoming.is_empty() {
            // Types differ: use the then-branch type as the canonical type
            // and attempt to coerce the other branch values
            let target_type = then_val.get_type();
            let phi = self
                .builder
                .build_phi(target_type, "if_result")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            for (val, block) in &incoming {
                let coerced = val.as_basic_value_enum();
                if coerced.get_type() == target_type {
                    phi.add_incoming(&[(&coerced, *block)]);
                } else {
                    // Coerce: try bitcast for same-size types, or use zero as fallback
                    let fallback = target_type.const_zero();
                    phi.add_incoming(&[(&fallback, *block)]);
                }
            }
            Ok(phi.as_basic_value())
        } else {
            Ok(self.context.struct_type(&[], false).const_zero().into())
        }
    }

    pub(super) fn generate_if_else(
        &mut self,
        if_else: &IfElse,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        match if_else {
            IfElse::Else(stmts) => self.generate_block(stmts),
            IfElse::ElseIf(cond, then_stmts, else_branch) => self.generate_if_expr(
                &cond.node,
                then_stmts,
                else_branch.as_ref().map(|b| b.as_ref()),
            ),
        }
    }

    // ========== Loop Expression ==========

    pub(super) fn generate_loop(
        &mut self,
        pattern: Option<&Spanned<Pattern>>,
        iter: Option<&Spanned<Expr>>,
        body: &[Spanned<Stmt>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self
            .current_function
            .ok_or_else(|| CodegenError::LlvmError("No current function for loop".to_string()))?;

        // Check if this is a range-based for loop
        let is_range_loop = iter
            .as_ref()
            .is_some_and(|it| matches!(&it.node, Expr::Range { .. }));

        if is_range_loop {
            if let (Some(pat), Some(it)) = (pattern, iter) {
                // Range-based for loop: L pattern : start..end { body }
                return self.generate_range_for_loop(fn_value, pat, it, body);
            }
        }
        // Condition-based or infinite loop
        self.generate_condition_loop(fn_value, pattern, iter, body)
    }

    pub(super) fn generate_range_for_loop(
        &mut self,
        fn_value: inkwell::values::FunctionValue<'ctx>,
        pattern: &Spanned<Pattern>,
        iter: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Extract range start, end, inclusive from the iter expression
        let (start_expr, end_expr, inclusive) = match &iter.node {
            Expr::Range {
                start,
                end,
                inclusive,
            } => (start.as_deref(), end.as_deref(), *inclusive),
            _ => return Err(CodegenError::InternalError("generate_range_for_loop called with non-range iter".to_string())),
        };

        // Generate start and end values
        let start_val = if let Some(s) = start_expr {
            self.generate_expr(&s.node)?.into_int_value()
        } else {
            self.context.i64_type().const_int(0, false)
        };

        let end_val = if let Some(e) = end_expr {
            self.generate_expr(&e.node)?.into_int_value()
        } else {
            self.context.i64_type().const_int(i64::MAX as u64, false)
        };

        // Create counter variable
        let counter_alloca = self
            .builder
            .build_alloca(self.context.i64_type(), "loop_counter")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(counter_alloca, start_val)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let loop_cond = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("for.cond"));
        let loop_body = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("for.body"));
        let loop_inc = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("for.inc"));
        let loop_end = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("for.end"));

        // Push loop context: continue goes to increment, break goes to end
        self.loop_stack.push(LoopContext {
            break_block: loop_end,
            continue_block: loop_inc,
        });

        // Branch to condition check
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Condition block: counter < end (or counter <= end for inclusive)
        self.builder.position_at_end(loop_cond);
        let current_val = self
            .builder
            .build_load(self.context.i64_type(), counter_alloca, "counter_val")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();

        let cmp_pred = if inclusive {
            IntPredicate::SLE
        } else {
            IntPredicate::SLT
        };
        let cond = self
            .builder
            .build_int_compare(cmp_pred, current_val, end_val, "for_cond")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        self.builder
            .build_conditional_branch(cond, loop_body, loop_end)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Body block: bind pattern to current counter value, execute body
        self.builder.position_at_end(loop_body);

        // Load current counter and bind to pattern
        let counter_for_bind = self
            .builder
            .build_load(self.context.i64_type(), counter_alloca, "bind_val")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.generate_pattern_bindings(pattern, &counter_for_bind)?;

        // Generate body
        let _body_val = self.generate_block(body)?;

        // Branch to increment (if not terminated by break/return)
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(loop_inc)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Increment block: counter += 1
        self.builder.position_at_end(loop_inc);
        let inc_val = self
            .builder
            .build_load(self.context.i64_type(), counter_alloca, "inc_load")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();
        let next_val = self
            .builder
            .build_int_add(
                inc_val,
                self.context.i64_type().const_int(1, false),
                "inc_val",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(counter_alloca, next_val)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // End block
        self.builder.position_at_end(loop_end);
        self.loop_stack.pop();

        // For loops return unit
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    pub(super) fn generate_condition_loop(
        &mut self,
        fn_value: inkwell::values::FunctionValue<'ctx>,
        pattern: Option<&Spanned<Pattern>>,
        iter: Option<&Spanned<Expr>>,
        body: &[Spanned<Stmt>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let loop_start = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("loop.start"));
        let loop_body = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("loop.body"));
        let loop_end = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("loop.end"));

        // Push loop context for break/continue
        self.loop_stack.push(LoopContext {
            break_block: loop_end,
            continue_block: loop_start,
        });

        // Allocate an alloca to cache the iterator value when both a pattern and an
        // iterator expression are present.  This avoids evaluating the iterator
        // expression a second time in the loop body (fix for double-evaluation bug).
        // We allocate it in the entry block (current position, before branch) so that
        // LLVM's mem2reg can promote it.
        let iter_alloca: Option<inkwell::values::PointerValue<'ctx>> =
            if pattern.is_some() && iter.is_some() {
                let alloca = self
                    .builder
                    .build_alloca(self.context.i64_type(), "iter_cache")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Some(alloca)
            } else {
                None
            };

        // Branch to loop start
        self.builder
            .build_unconditional_branch(loop_start)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Loop start - check condition if present
        self.builder.position_at_end(loop_start);

        if let Some(iter_expr) = iter {
            // Conditional loop (while-like): evaluate the iterator once per iteration.
            let cond_val = self.generate_expr(&iter_expr.node)?;

            // Cache the raw iterator value so the loop body can bind it without
            // re-evaluating the expression (which could have side effects).
            if let Some(alloca) = iter_alloca {
                let store_val = if cond_val.is_int_value() {
                    // Widen to i64 if necessary so the alloca type matches
                    let iv = cond_val.into_int_value();
                    if iv.get_type().get_bit_width() < 64 {
                        self.builder
                            .build_int_z_extend(iv, self.context.i64_type(), "iter_zext")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                            .into()
                    } else {
                        cond_val
                    }
                } else {
                    // Non-integer: store a placeholder zero
                    self.context.i64_type().const_int(0, false).into()
                };
                self.builder
                    .build_store(alloca, store_val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            }

            let cond_bool = if cond_val.is_int_value() {
                let int_val = cond_val.into_int_value();
                if int_val.get_type().get_bit_width() > 1 {
                    self.builder
                        .build_int_compare(
                            IntPredicate::NE,
                            int_val,
                            int_val.get_type().const_int(0, false),
                            "loop_cond",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                } else {
                    int_val
                }
            } else {
                self.context.bool_type().const_int(1, false)
            };

            self.builder
                .build_conditional_branch(cond_bool, loop_body, loop_end)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        } else {
            // Infinite loop
            self.builder
                .build_unconditional_branch(loop_body)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Loop body
        self.builder.position_at_end(loop_body);

        // Bind pattern if present: load the cached iterator value from the alloca
        // instead of re-evaluating the iterator expression.
        if let (Some(pat), Some(_iter_expr)) = (pattern, iter) {
            if let Some(cached_alloca) = iter_alloca {
                let iter_val = self
                    .builder
                    .build_load(self.context.i64_type(), cached_alloca, "iter_cached")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.generate_pattern_bindings(pat, &iter_val)?;
            }
        }

        let _body_val = self.generate_block(body)?;

        // Branch back to loop start (if not terminated by break/return)
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(loop_start)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Loop end
        self.builder.position_at_end(loop_end);
        self.loop_stack.pop();

        // Loops return unit by default
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    pub(super) fn generate_while_loop(
        &mut self,
        condition: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self.current_function.ok_or_else(|| {
            CodegenError::LlvmError("No current function for while loop".to_string())
        })?;

        let loop_cond = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("while.cond"));
        let loop_body = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("while.body"));
        let loop_end = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("while.end"));

        // Push loop context for break/continue
        self.loop_stack.push(LoopContext {
            break_block: loop_end,
            continue_block: loop_cond,
        });

        // Branch to condition check
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Condition block
        self.builder.position_at_end(loop_cond);
        let cond_val = self.generate_expr(&condition.node)?;
        let cond_bool = if cond_val.is_int_value() {
            let int_val = cond_val.into_int_value();
            if int_val.get_type().get_bit_width() > 1 {
                self.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        int_val,
                        int_val.get_type().const_int(0, false),
                        "while_cond",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                int_val
            }
        } else {
            self.context.bool_type().const_int(1, false)
        };

        self.builder
            .build_conditional_branch(cond_bool, loop_body, loop_end)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Loop body
        self.builder.position_at_end(loop_body);
        let _body_val = self.generate_block(body)?;

        // Branch back to condition (if not terminated by break/return)
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(loop_cond)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Loop end
        self.builder.position_at_end(loop_end);
        self.loop_stack.pop();

        // While loops return unit
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    /// Emit deferred expressions in LIFO order (before function return).
    pub(super) fn emit_defer_cleanup(&mut self) -> CodegenResult<()> {
        let deferred: Vec<Expr> = self.defer_stack.iter().rev().cloned().collect();
        for expr in deferred {
            self.generate_expr(&expr)?;
        }
        Ok(())
    }

    pub(super) fn generate_break(
        &mut self,
        value: Option<&Expr>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let break_block = self
            .loop_stack
            .last()
            .ok_or_else(|| CodegenError::Unsupported("break outside of loop".to_string()))?
            .break_block;

        // Generate value if present (for loop with value)
        if let Some(val_expr) = value {
            let _val = self.generate_expr(val_expr)?;
            // In a full implementation, this would be used for loop-with-value
        }

        self.builder
            .build_unconditional_branch(break_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    pub(super) fn generate_continue(&mut self) -> CodegenResult<BasicValueEnum<'ctx>> {
        let loop_ctx = self
            .loop_stack
            .last()
            .ok_or_else(|| CodegenError::Unsupported("continue outside of loop".to_string()))?;

        let continue_block = loop_ctx.continue_block;
        self.builder
            .build_unconditional_branch(continue_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    // ========== Array/Tuple/Index ==========
}
