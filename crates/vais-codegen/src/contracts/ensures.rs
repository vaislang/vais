//! Postcondition (ensures) checks.

use crate::{CodeGenerator, CodegenResult};
use std::fmt::Write;
use vais_ast::Function;
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Generate ensures (postcondition) checks for a function
    ///
    /// Inserts condition checks before function return, calling __contract_fail
    /// if any postcondition fails. The return value is available as `return`.
    pub(crate) fn generate_ensures_checks(
        &mut self,
        f: &Function,
        return_value: &str,
        ret_type: &ResolvedType,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        // Skip contract checks in release mode
        if self.release_mode {
            return Ok(String::new());
        }

        let mut ir = String::new();

        // Register 'return' as a local variable for ensures expressions
        if !f.attributes.iter().any(|a| a.name == "ensures") {
            return Ok(ir);
        }

        // Store the return value in a local for reference in ensures expressions
        let return_llvm = self.type_to_llvm(ret_type);
        // Note: llvm_name should NOT include the % prefix - generate_ident adds it
        let return_var_name = format!("__contract_return.{}", *counter);
        *counter += 1;

        writeln!(ir, "  %{} = alloca {}", return_var_name, return_llvm).unwrap();
        writeln!(
            ir,
            "  store {} {}, {}* %{}",
            return_llvm, return_value, return_llvm, return_var_name
        )
        .unwrap();

        // Register 'return' in locals for expression generation
        // Use alloca since we stored the return value at return_var_name
        self.fn_ctx.locals.insert(
            "return".to_string(),
            crate::types::LocalVar::alloca(ret_type.clone(), return_var_name),
        );

        for (idx, attr) in f.attributes.iter().enumerate() {
            if attr.name == "ensures" {
                if let Some(expr) = &attr.expr {
                    let check_ir =
                        self.generate_contract_check(expr, &f.name.node, "ensures", idx, counter)?;
                    ir.push_str(&check_ir);
                }
            }
        }

        // Remove 'return' from locals
        self.fn_ctx.locals.remove("return");

        Ok(ir)
    }
}
