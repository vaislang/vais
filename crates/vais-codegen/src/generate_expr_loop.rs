//! Range-based for loop code generation for LLVM IR.

use vais_ast::{Expr, Pattern, Spanned, Stmt};
use vais_types::ResolvedType;

use crate::{CodeGenerator, CodegenError, CodegenResult, LocalVar, LoopLabels};

impl CodeGenerator {
    #[inline(never)]
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
            _ => {
                return Err(CodegenError::InternalError(
                    "generate_range_for_loop called with non-range iter".to_string(),
                ))
            }
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

        let counter_var = format!("%loop_counter.{}", self.fn_ctx.label_counter);
        self.fn_ctx.label_counter += 1;
        self.emit_entry_alloca(&counter_var, "i64");
        write_ir!(ir, "  store i64 {}, i64* {}", start_val, counter_var);

        let pattern_var = if let Pattern::Ident(name) = &pattern.node {
            let var_name = format!("{}.for.{}", name, self.fn_ctx.label_counter);
            self.fn_ctx.label_counter += 1;
            let llvm_name = format!("%{}", var_name);
            self.emit_entry_alloca(&llvm_name, "i64");
            self.fn_ctx.locals.insert(
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

        self.fn_ctx.loop_stack.push(LoopLabels {
            continue_label: loop_inc.clone(),
            break_label: loop_end.clone(),
            scope_str_depth: self.fn_ctx.scope_str_stack.len(),
        });

        write_ir!(ir, "  br label %{}", loop_cond);

        write_ir!(ir, "{}:", loop_cond);
        let current_val = self.next_temp(counter);
        write_ir!(ir, "  {} = load i64, i64* {}", current_val, counter_var);

        let cmp_pred = if inclusive { "sle" } else { "slt" };
        let cond_result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = icmp {} i64 {}, {}",
            cond_result,
            cmp_pred,
            current_val,
            end_val
        );
        write_ir!(
            ir,
            "  br i1 {}, label %{}, label %{}",
            cond_result,
            loop_body_label,
            loop_end
        );

        write_ir!(ir, "{}:", loop_body_label);

        if let Some((_, llvm_name)) = &pattern_var {
            let bind_val = self.next_temp(counter);
            write_ir!(ir, "  {} = load i64, i64* {}", bind_val, counter_var);
            write_ir!(ir, "  store i64 {}, i64* {}", bind_val, llvm_name);
        }

        let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
        ir.push_str(&body_ir);

        if !body_terminated {
            write_ir!(ir, "  br label %{}", loop_inc);
        }

        write_ir!(ir, "{}:", loop_inc);
        let inc_load = self.next_temp(counter);
        write_ir!(ir, "  {} = load i64, i64* {}", inc_load, counter_var);
        let inc_result = self.next_temp(counter);
        write_ir!(ir, "  {} = add i64 {}, 1", inc_result, inc_load);
        write_ir!(ir, "  store i64 {}, i64* {}", inc_result, counter_var);
        write_ir!(ir, "  br label %{}", loop_cond);

        write_ir!(ir, "{}:", loop_end);
        self.fn_ctx.loop_stack.pop();

        Ok(("0".to_string(), ir))
    }
}
