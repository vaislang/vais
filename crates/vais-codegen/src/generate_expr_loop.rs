//! Range-based for loop code generation for LLVM IR.

use vais_ast::{Expr, Pattern, Spanned, Stmt};
use vais_types::ResolvedType;

use crate::{CodeGenerator, CodegenError, CodegenResult, LocalVar, LoopLabels};

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
            _ => return Err(CodegenError::InternalError("generate_range_for_loop called with non-range iter".to_string())),
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
        ir.push_str(&format!("  {} = alloca i64\n", counter_var));
        ir.push_str(&format!(
            "  store i64 {}, i64* {}\n",
            start_val, counter_var
        ));

        let pattern_var = if let Pattern::Ident(name) = &pattern.node {
            let var_name = format!("{}.for", name);
            let llvm_name = format!("%{}", var_name);
            ir.push_str(&format!("  {} = alloca i64\n", llvm_name));
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
        self.fn_ctx.loop_stack.pop();

        Ok(("0".to_string(), ir))
    }
}
