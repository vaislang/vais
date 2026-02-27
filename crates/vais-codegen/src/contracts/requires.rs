//! Precondition (requires) checks.

use crate::{CodeGenerator, CodegenResult};
use std::fmt::Write;
use vais_ast::{Expr, Function, Spanned};

impl CodeGenerator {
    /// Generate requires (precondition) checks for a function
    ///
    /// Inserts condition checks after function entry, calling __contract_fail
    /// if any precondition fails.
    pub(crate) fn generate_requires_checks(
        &mut self,
        f: &Function,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        // Skip contract checks in release mode
        if self.release_mode {
            return Ok(String::new());
        }

        let mut ir = String::new();

        for (idx, attr) in f.attributes.iter().enumerate() {
            if attr.name == "requires" {
                if let Some(expr) = &attr.expr {
                    let check_ir =
                        self.generate_contract_check(expr, &f.name.node, "requires", idx, counter)?;
                    ir.push_str(&check_ir);
                }
            }
        }

        Ok(ir)
    }

    /// Generate a single contract check
    ///
    /// Generates:
    /// ```llvm
    /// %cond = <evaluate expression>
    /// br i1 %cond, label %contract_ok_N, label %contract_fail_N
    ///
    /// contract_fail_N:
    ///   call i64 @__contract_fail(i64 <kind>, i8* <condition>, i8* <file>, i64 <line>, i8* <func>)
    ///   unreachable
    ///
    /// contract_ok_N:
    /// ```
    pub(super) fn generate_contract_check(
        &mut self,
        expr: &Spanned<Expr>,
        func_name: &str,
        kind: &str, // "requires" or "ensures"
        idx: usize,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        let mut ir = String::new();

        // Generate the condition expression
        let (cond_value, cond_ir) = self.generate_expr(expr, counter)?;
        ir.push_str(&cond_ir);

        // Generate unique labels
        let ok_label = format!("contract_ok_{}_{}", kind, idx);
        let fail_label = format!("contract_fail_{}_{}", kind, idx);

        // Convert the condition to i1 for branch
        // VAIS uses i64 for bool, but LLVM branch needs i1
        let cond_i1 = format!("%contract_cond_i1_{}", *counter);
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

        // Contract kind: 1 = requires, 2 = ensures
        let kind_value = if kind == "requires" { 1 } else { 2 };

        // Create string constants for error message
        let condition_str =
            self.get_or_create_contract_string(&format!("{} condition #{}", kind, idx));
        let file_name = self
            .fn_ctx
            .current_file
            .as_deref()
            .unwrap_or("unknown")
            .to_string();
        let file_str = self.get_or_create_contract_string(&file_name);
        let func_str = self.get_or_create_contract_string(func_name);

        // Get line number from span
        let line = self.debug_info.offset_to_line(expr.span.start) as i64;

        // Call __contract_fail
        writeln!(
            ir,
            "  call i64 @__contract_fail(i64 {}, i8* {}, i8* {}, i64 {}, i8* {})",
            kind_value, condition_str, file_str, line, func_str
        )
        .unwrap();
        ir.push_str("  unreachable\n");

        // Success block
        writeln!(ir, "{}:", ok_label).unwrap();

        Ok(ir)
    }
}
