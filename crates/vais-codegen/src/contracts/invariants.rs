//! Struct invariant checks.

use std::fmt::Write;
use vais_types::ResolvedType;
use crate::{CodeGenerator, CodegenResult};

impl CodeGenerator {
    /// Generate invariant checks for a struct type
    ///
    /// Called after struct construction/modification to verify invariants.
    pub(crate) fn _generate_invariant_checks(
        &mut self,
        struct_name: &str,
        struct_ptr: &str,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        // Skip in release mode
        if self.release_mode {
            return Ok(String::new());
        }

        // Look up struct's invariant attributes
        let struct_info = self.types.structs.get(struct_name).cloned();
        let invariants = struct_info
            .as_ref()
            .map(|s| s._invariants.clone())
            .unwrap_or_default();

        if invariants.is_empty() {
            return Ok(String::new());
        }

        let mut ir = String::new();

        for (idx, invariant_expr) in invariants.iter().enumerate() {
            // Generate the invariant condition
            // Note: The invariant expression can reference struct fields via 'self'
            // We need to set up 'self' to point to struct_ptr
            let saved_self = self.fn_ctx.locals.get("self").cloned();

            if let Some(_si) = &struct_info {
                self.fn_ctx.locals.insert(
                    "self".to_string(),
                    crate::types::LocalVar::param(
                        ResolvedType::Named {
                            name: struct_name.to_string(),
                            generics: Vec::new(),
                        },
                        struct_ptr.trim_start_matches('%').to_string(),
                    ),
                );
            }

            let (cond_value, cond_ir) = self.generate_expr(invariant_expr, counter)?;
            ir.push_str(&cond_ir);

            // Restore self
            if let Some(prev) = saved_self {
                self.fn_ctx.locals.insert("self".to_string(), prev);
            } else {
                self.fn_ctx.locals.remove("self");
            }

            // Generate check
            let ok_label = format!("invariant_ok_{}_{}", struct_name, idx);
            let fail_label = format!("invariant_fail_{}_{}", struct_name, idx);

            let cond_i1 = format!("%invariant_cond_i1_{}", *counter);
            *counter += 1;
            writeln!(ir, "  {} = icmp ne i64 {}, 0", cond_i1, cond_value).unwrap();

            writeln!(
                ir,
                "  br i1 {}, label %{}, label %{}",
                cond_i1, ok_label, fail_label
            )
            .unwrap();

            // Failure block
            writeln!(ir, "{}:", fail_label).unwrap();

            let kind_value = 3; // CONTRACT_INVARIANT
            let condition_str = self
                .get_or_create_contract_string(&format!("invariant #{} of {}", idx, struct_name));
            let file_name = self
                .fn_ctx
                .current_file
                .as_deref()
                .unwrap_or("unknown")
                .to_string();
            let func_name = self
                .fn_ctx
                .current_function
                .as_deref()
                .unwrap_or("unknown")
                .to_string();
            let file_str = self.get_or_create_contract_string(&file_name);
            let func_str = self.get_or_create_contract_string(&func_name);
            let line = self.debug_info.offset_to_line(invariant_expr.span.start) as i64;

            writeln!(
                ir,
                "  call i64 @__contract_fail(i64 {}, i8* {}, i8* {}, i64 {}, i8* {})",
                kind_value, condition_str, file_str, line, func_str
            )
            .unwrap();
            ir.push_str("  unreachable\n");

            // Success block
            writeln!(ir, "{}:", ok_label).unwrap();
        }

        Ok(ir)
    }
}
