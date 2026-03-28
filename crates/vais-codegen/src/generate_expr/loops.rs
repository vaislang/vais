//! Loop expression code generation.
//!
//! Extracted from `generate_expr_inner` match arms for `Expr::Loop` and
//! `Expr::While` to reduce the parent function's stack frame size.
//! Each handler is `#[inline(never)]` so Rust allocates its locals independently.

use vais_ast::*;

use crate::{CodeGenerator, CodegenResult, LoopLabels};

impl CodeGenerator {
    /// Generate code for a loop expression (`L` keyword) with pattern support.
    /// Handles range-based for loops (`L pattern : start..end { body }`),
    /// then falls through to conditional/infinite loops.
    /// Extracted from `generate_expr_inner` to reduce stack frame size.
    #[inline(never)]
    pub(crate) fn generate_loop_with_pattern(
        &mut self,
        pattern: Option<&Spanned<Pattern>>,
        iter: Option<&Box<Spanned<Expr>>>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Check if this is a range-based for loop
        let is_range_loop = iter
            .as_ref()
            .is_some_and(|it| matches!(&it.node, Expr::Range { .. }));

        if is_range_loop {
            if let (Some(pat), Some(it)) = (pattern, iter) {
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
            write_ir!(ir, "  br label %{}", loop_start);
            write_ir!(ir, "{}:", loop_start);

            // Evaluate condition
            let (cond_val, cond_ir) = self.generate_expr(iter_expr, counter)?;
            ir.push_str(&cond_ir);

            // Convert to i1 for branch (type-aware: skips for bool/i1)
            let (cond_bool, conv_ir) =
                self.generate_cond_to_i1(iter_expr, &cond_val, counter);
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
            let (_body_val, body_ir, body_terminated) =
                self.generate_block_stmts(body, counter)?;
            ir.push_str(&body_ir);
            // Only emit loop back if body doesn't terminate
            if !body_terminated {
                write_ir!(ir, "  br label %{}", loop_start);
            }
        } else {
            // Infinite loop: L { body } - must use break to exit
            write_ir!(ir, "  br label %{}", loop_start);
            write_ir!(ir, "{}:", loop_start);
            let (_body_val, body_ir, body_terminated) =
                self.generate_block_stmts(body, counter)?;
            ir.push_str(&body_ir);
            // Only emit loop back if body doesn't terminate
            if !body_terminated {
                write_ir!(ir, "  br label %{}", loop_start);
            }
        }

        // Loop end
        write_ir!(ir, "{}:", loop_end);
        self.fn_ctx.current_block.clone_from(&loop_end);

        self.fn_ctx.loop_stack.pop();

        // Loop returns void by default (use break with value for expression)
        Ok(("0".to_string(), ir))
    }

    /// Generate code for a while loop expression.
    /// Extracted from `generate_expr_inner` to reduce stack frame size.
    #[inline(never)]
    pub(crate) fn generate_while_loop_expr(
        &mut self,
        condition: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
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
        write_ir!(ir, "  br label %{}", loop_start);
        write_ir!(ir, "{}:", loop_start);

        // Evaluate condition
        let (cond_val, cond_ir) = self.generate_expr(condition, counter)?;
        ir.push_str(&cond_ir);

        // Convert to i1 for branch (type-aware: skips for bool/i1)
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
        let (_body_val, body_ir, body_terminated) =
            self.generate_block_stmts(body, counter)?;
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
