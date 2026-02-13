//! Control flow expression helpers for CodeGenerator
//!
//! Contains ternary, if, loop, and while expression generation.

use crate::types::LoopLabels;
use crate::{CodeGenerator, CodegenResult};
use vais_ast::{Expr, Spanned, Stmt};

impl CodeGenerator {
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
        self.fn_ctx.current_block.clone_from(&then_label);
        let (then_val, then_ir, then_terminated) = self.generate_block_stmts(then, counter)?;
        ir.push_str(&then_ir);
        let then_actual_block = self.fn_ctx.current_block.clone();
        let then_from_label = if !then_terminated {
            ir.push_str(&format!("  br label %{}\n", merge_label));
            then_actual_block
        } else {
            String::new()
        };

        // Else block
        ir.push_str(&format!("{}:\n", else_label));
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
        let else_from_label = if !else_terminated {
            ir.push_str(&format!("  br label %{}\n", merge_label));
            if !nested_last_block.is_empty() {
                nested_last_block
            } else {
                self.fn_ctx.current_block.clone()
            }
        } else {
            String::new()
        };

        // Merge block
        ir.push_str(&format!("{}:\n", merge_label));
        self.fn_ctx.current_block.clone_from(&merge_label);
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

        self.fn_ctx.loop_stack.push(LoopLabels {
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
        self.fn_ctx.loop_stack.pop();

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

        self.fn_ctx.loop_stack.push(LoopLabels {
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
        self.fn_ctx.loop_stack.pop();

        Ok(("0".to_string(), ir))
    }

}
