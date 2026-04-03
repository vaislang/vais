//! Control flow expression helpers for CodeGenerator
//!
//! Contains ternary, if, loop, and while expression generation.

use crate::types::LoopLabels;
use crate::{CodeGenerator, CodegenResult};
use vais_ast::{Expr, Spanned, Stmt};
use vais_types::ResolvedType;

impl CodeGenerator {
    #[inline(never)]
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

        // Convert to i1 for branch (type-aware: skip icmp for already-i1 bool)
        let (cond_bool, conv_ir) = self.generate_cond_to_i1(cond, &cond_val, counter);
        ir.push_str(&conv_ir);

        // Conditional branch
        write_ir!(
            ir,
            "  br i1 {}, label %{}, label %{}",
            cond_bool,
            then_label,
            else_label
        );

        // Then branch
        write_ir!(ir, "{}:", then_label);
        self.fn_ctx.current_block.clone_from(&then_label);
        let (then_val, then_ir) = self.generate_expr(then, counter)?;
        ir.push_str(&then_ir);
        let then_actual = self.fn_ctx.current_block.clone();
        write_ir!(ir, "  br label %{}", merge_label);

        // Else branch
        write_ir!(ir, "{}:", else_label);
        self.fn_ctx.current_block.clone_from(&else_label);
        let (else_val, else_ir) = self.generate_expr(else_, counter)?;
        ir.push_str(&else_ir);
        let else_actual = self.fn_ctx.current_block.clone();
        write_ir!(ir, "  br label %{}", merge_label);

        // Merge with phi — use actual blocks (may differ if body inserted labels)
        write_ir!(ir, "{}:", merge_label);
        self.fn_ctx.current_block.clone_from(&merge_label);
        let result = self.next_temp(counter);
        let phi_type = self.infer_expr_type(then);
        let phi_llvm = self.type_to_llvm(&phi_type);
        write_ir!(
            ir,
            "  {} = phi {} [ {}, %{} ], [ {}, %{} ]",
            result,
            phi_llvm,
            then_val,
            then_actual,
            else_val,
            else_actual
        );

        Ok((result, ir))
    }

    /// Generate function call expression
    #[inline(never)]
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

        // Convert to i1 for branch (type-aware: skip icmp for already-i1 bool)
        let (cond_bool, conv_ir) = self.generate_cond_to_i1(cond, &cond_val, counter);
        ir.push_str(&conv_ir);
        write_ir!(
            ir,
            "  br i1 {}, label %{}, label %{}",
            cond_bool,
            then_label,
            else_label
        );

        // Infer block type to detect struct/enum results that need loading
        let phi_type = self.infer_block_type(then);
        let phi_llvm = self.type_to_llvm(&phi_type);

        // Check each branch independently for struct pointer vs value
        let is_named_phi = matches!(&phi_type, ResolvedType::Named { .. });
        let then_is_ptr = is_named_phi && !self.is_block_result_value(then);

        // Then block
        write_ir!(ir, "{}:", then_label);
        self.fn_ctx.current_block.clone_from(&then_label);
        let (then_val, then_ir, then_terminated) = self.generate_block_stmts(then, counter)?;
        ir.push_str(&then_ir);

        // For struct/enum results, load the value from the alloca pointer before branch
        let then_val_for_phi = if then_is_ptr && !then_terminated {
            let loaded = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = load {}, {}* {}",
                loaded,
                phi_llvm,
                phi_llvm,
                then_val
            );
            loaded
        } else if !then_terminated {
            // Coerce integer width if the value type differs from the phi type
            let actual_ty = self.llvm_type_of(&then_val);
            let coerced =
                self.coerce_int_width(&then_val, &actual_ty, &phi_llvm, counter, &mut ir);
            // Also coerce float width (e.g., float→double or double→float for phi)
            let actual_after = self.llvm_type_of(&coerced);
            if actual_after != phi_llvm
                && (actual_after == "float" || actual_after == "double")
                && (phi_llvm == "float" || phi_llvm == "double")
            {
                let tmp = self.next_temp(counter);
                if actual_after == "float" {
                    write_ir!(ir, "  {} = fpext float {} to double", tmp, coerced);
                } else {
                    write_ir!(ir, "  {} = fptrunc double {} to float", tmp, coerced);
                }
                tmp
            } else {
                coerced
            }
        } else {
            then_val
        };

        let then_actual_block = self.fn_ctx.current_block.clone();
        let then_from_label = if !then_terminated {
            write_ir!(ir, "  br label %{}", merge_label);
            then_actual_block
        } else {
            String::new()
        };

        // Else block
        write_ir!(ir, "{}:", else_label);
        self.fn_ctx.current_block.clone_from(&else_label);
        let (else_val, else_ir, else_terminated, nested_last_block, has_else) =
            if let Some(else_branch) = else_ {
                let (v, i, t, last) =
                    self.generate_if_else_with_term(else_branch, counter, &merge_label)?;
                (v, i, t, last, true)
            } else {
                ("0".to_string(), String::new(), false, String::new(), false)
            };
        ir.push_str(&else_ir);

        // For struct/enum results, load the value from the alloca pointer before branch
        // But if else_val comes from a nested if-else (indicated by non-empty nested_last_block),
        // it's already a phi node value (not a pointer), so don't load it
        // Check else branch independently for struct pointer (may differ from then branch)
        let else_is_ptr = is_named_phi
            && has_else
            && nested_last_block.is_empty()
            && self.is_else_branch_ptr(else_);
        let else_val_for_phi =
            if else_is_ptr && !else_terminated && has_else && nested_last_block.is_empty() {
                let loaded = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = load {}, {}* {}",
                    loaded,
                    phi_llvm,
                    phi_llvm,
                    else_val
                );
                loaded
            } else if !else_terminated && has_else {
                // Coerce integer width if the value type differs from the phi type
                let actual_ty = self.llvm_type_of(&else_val);
                let coerced =
                    self.coerce_int_width(&else_val, &actual_ty, &phi_llvm, counter, &mut ir);
                // Also coerce float width (e.g., float→double or double→float for phi)
                let actual_after = self.llvm_type_of(&coerced);
                if actual_after != phi_llvm
                    && (actual_after == "float" || actual_after == "double")
                    && (phi_llvm == "float" || phi_llvm == "double")
                {
                    let tmp = self.next_temp(counter);
                    if actual_after == "float" {
                        write_ir!(ir, "  {} = fpext float {} to double", tmp, coerced);
                    } else {
                        write_ir!(ir, "  {} = fptrunc double {} to float", tmp, coerced);
                    }
                    tmp
                } else {
                    coerced
                }
            } else {
                else_val
            };

        let else_from_label = if !else_terminated {
            write_ir!(ir, "  br label %{}", merge_label);
            if !nested_last_block.is_empty() {
                nested_last_block
            } else {
                self.fn_ctx.current_block.clone()
            }
        } else {
            String::new()
        };

        // Merge block
        write_ir!(ir, "{}:", merge_label);
        self.fn_ctx.current_block.clone_from(&merge_label);
        let result = self.next_temp(counter);
        let is_void = crate::helpers::is_void_result(&phi_llvm, &phi_type);

        if is_void || !has_else {
            ir.push_str(&crate::helpers::void_placeholder_ir(&result));
        } else if !then_from_label.is_empty() && !else_from_label.is_empty() {
            write_ir!(
                ir,
                "  {} = phi {} [ {}, %{} ], [ {}, %{} ]",
                result,
                phi_llvm,
                then_val_for_phi,
                then_from_label,
                else_val_for_phi,
                else_from_label
            );
        } else if !then_from_label.is_empty() {
            write_ir!(
                ir,
                "  {} = phi {} [ {}, %{} ]",
                result,
                phi_llvm,
                then_val_for_phi,
                then_from_label
            );
        } else if !else_from_label.is_empty() {
            write_ir!(
                ir,
                "  {} = phi {} [ {}, %{} ]",
                result,
                phi_llvm,
                else_val_for_phi,
                else_from_label
            );
        } else {
            ir.push_str(&crate::helpers::void_placeholder_ir(&result));
        }

        Ok((result, ir))
    }

    /// Check if an else branch produces a pointer (struct literal) vs a value
    fn is_else_branch_ptr(&self, else_: Option<&vais_ast::IfElse>) -> bool {
        match else_ {
            Some(vais_ast::IfElse::Else(stmts)) => !self.is_block_result_value(stmts),
            Some(vais_ast::IfElse::ElseIf(..)) => false, // nested if-else produces phi value
            None => false,
        }
    }

    /// Generate loop expression
    #[inline(never)]
    pub(crate) fn generate_loop_expr(
        &mut self,
        iter: Option<&Spanned<Expr>>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let loop_start = self.next_label("loop.start");
        let loop_body = self.next_label("loop.body");
        let loop_end = self.next_label("loop.end");

        self.fn_ctx.loop_stack.push(LoopLabels {
            continue_label: loop_start.to_string(),
            break_label: loop_end.to_string(),
        });

        let mut ir = String::new();

        if let Some(iter_expr) = iter {
            // Conditional loop
            write_ir!(ir, "  br label %{}", loop_start);
            write_ir!(ir, "{}:", loop_start);

            let (cond_val, cond_ir) = self.generate_expr(iter_expr, counter)?;
            ir.push_str(&cond_ir);

            // Convert to i1 for branch (type-aware: skip icmp for already-i1 bool)
            let (cond_bool, conv_ir) = self.generate_cond_to_i1(iter_expr, &cond_val, counter);
            ir.push_str(&conv_ir);
            write_ir!(
                ir,
                "  br i1 {}, label %{}, label %{}",
                cond_bool,
                loop_body,
                loop_end
            );

            write_ir!(ir, "{}:", loop_body);
            let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
            ir.push_str(&body_ir);
            if !body_terminated {
                write_ir!(ir, "  br label %{}", loop_start);
            }
        } else {
            // Infinite loop
            write_ir!(ir, "  br label %{}", loop_start);
            write_ir!(ir, "{}:", loop_start);
            let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
            ir.push_str(&body_ir);
            if !body_terminated {
                write_ir!(ir, "  br label %{}", loop_start);
            }
        }

        write_ir!(ir, "{}:", loop_end);
        self.fn_ctx.loop_stack.pop();

        Ok(("0".to_string(), ir))
    }

    /// Generate while loop expression
    #[inline(never)]
    pub(crate) fn generate_while_expr(
        &mut self,
        condition: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let loop_start = self.next_label("while.start");
        let loop_body = self.next_label("while.body");
        let loop_end = self.next_label("while.end");

        self.fn_ctx.loop_stack.push(LoopLabels {
            continue_label: loop_start.to_string(),
            break_label: loop_end.to_string(),
        });

        let mut ir = String::new();

        // Jump to condition check
        write_ir!(ir, "  br label %{}", loop_start);
        write_ir!(ir, "{}:", loop_start);

        // Evaluate condition
        let (cond_val, cond_ir) = self.generate_expr(condition, counter)?;
        ir.push_str(&cond_ir);

        // Convert to i1 for branch (type-aware: skip icmp for already-i1 bool)
        let (cond_bool, conv_ir) = self.generate_cond_to_i1(condition, &cond_val, counter);
        ir.push_str(&conv_ir);
        write_ir!(
            ir,
            "  br i1 {}, label %{}, label %{}",
            cond_bool,
            loop_body,
            loop_end
        );

        // Loop body
        write_ir!(ir, "{}:", loop_body);
        let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
        ir.push_str(&body_ir);

        // Jump back to condition if body doesn't terminate
        if !body_terminated {
            write_ir!(ir, "  br label %{}", loop_start);
        }

        // Loop end
        write_ir!(ir, "{}:", loop_end);
        self.fn_ctx.current_block.clone_from(&loop_end);
        self.fn_ctx.loop_stack.pop();

        Ok(("0".to_string(), ir))
    }
}
