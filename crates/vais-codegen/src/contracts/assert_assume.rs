//! Assert and assume expression generation.

use crate::{CodeGenerator, CodegenResult};
use std::fmt::Write;
use vais_ast::{Expr, Spanned};

impl CodeGenerator {
    /// Generate assert expression
    ///
    /// assert(condition) or assert(condition, message)
    /// Generates runtime check that panics if condition is false.
    pub(crate) fn generate_assert(
        &mut self,
        condition: &Spanned<Expr>,
        message: Option<&Spanned<Expr>>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // In release mode, assert is still checked (unlike assume)
        let mut ir = String::new();

        // Generate the condition expression
        let (cond_value, cond_ir) = self.generate_expr(condition, counter)?;
        ir.push_str(&cond_ir);

        // Generate unique labels
        let ok_label = format!("assert_ok_{}", *counter);
        let fail_label = format!("assert_fail_{}", *counter);
        *counter += 1;

        // Convert condition to i1
        let cond_i1 = format!("%assert_cond_i1_{}", *counter);
        *counter += 1;
        writeln!(ir, "  {} = icmp ne i64 {}, 0", cond_i1, cond_value).unwrap();

        // Branch based on condition
        writeln!(
            ir,
            "  br i1 {}, label %{}, label %{}",
            cond_i1, ok_label, fail_label
        )
        .unwrap();

        // Failure block
        writeln!(ir, "{}:", fail_label).unwrap();

        // Generate error message
        let msg_str = if let Some(msg_expr) = message {
            // User-provided message
            let (msg_val, msg_ir) = self.generate_expr(msg_expr, counter)?;
            ir.push_str(&msg_ir);
            msg_val
        } else {
            // Default message
            let default_msg = format!(
                "Assertion failed at {}:{}",
                self.fn_ctx.current_file.as_deref().unwrap_or("unknown"),
                self.debug_info.offset_to_line(condition.span.start)
            );

            self.get_or_create_contract_string(&default_msg)
        };

        // Call __panic to terminate
        writeln!(ir, "  call i64 @__panic(i8* {})", msg_str).unwrap();
        ir.push_str("  unreachable\n");

        // Success block
        writeln!(ir, "{}:", ok_label).unwrap();

        // Assert returns unit (0)
        Ok(("0".to_string(), ir))
    }

    /// Generate assume expression
    ///
    /// assume(condition) tells the verifier/optimizer that condition is true.
    /// In debug mode, acts like assert. In release mode, generates llvm.assume intrinsic.
    pub(crate) fn generate_assume(
        &mut self,
        condition: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();

        // Generate the condition expression
        let (cond_value, cond_ir) = self.generate_expr(condition, counter)?;
        ir.push_str(&cond_ir);

        // Convert condition to i1
        let cond_i1 = format!("%assume_cond_i1_{}", *counter);
        *counter += 1;
        writeln!(ir, "  {} = icmp ne i64 {}, 0", cond_i1, cond_value).unwrap();

        if self.release_mode {
            // In release mode, use LLVM assume intrinsic for optimization hints
            writeln!(ir, "  call void @llvm.assume(i1 {})", cond_i1).unwrap();
        } else {
            // In debug mode, check the assumption
            let ok_label = format!("assume_ok_{}", *counter);
            let fail_label = format!("assume_fail_{}", *counter);
            *counter += 1;

            writeln!(
                ir,
                "  br i1 {}, label %{}, label %{}",
                cond_i1, ok_label, fail_label
            )
            .unwrap();

            // Failure block
            writeln!(ir, "{}:", fail_label).unwrap();

            let fail_msg = format!(
                "Assumption violated at {}:{}",
                self.fn_ctx.current_file.as_deref().unwrap_or("unknown"),
                self.debug_info.offset_to_line(condition.span.start)
            );
            let msg_const = self.get_or_create_contract_string(&fail_msg);

            writeln!(ir, "  call i64 @__panic(i8* {})", msg_const).unwrap();
            ir.push_str("  unreachable\n");

            // Success block
            writeln!(ir, "{}:", ok_label).unwrap();
        }

        // Assume returns unit (0)
        Ok(("0".to_string(), ir))
    }
}
