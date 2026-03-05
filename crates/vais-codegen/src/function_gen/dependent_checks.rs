//! Runtime dependent type (refinement type) assertion generation.
//!
//! Generates LLVM IR assertions for dependent type predicates at function entry.
//! When a function has parameters with dependent types (e.g., `{x: i64 | x > 0}`),
//! this module inserts runtime checks that abort if the predicate is violated.
//!
//! The predicate is evaluated by temporarily binding the predicate's bound variable
//! to the parameter value, then using `generate_expr` to compile the boolean predicate.
//!
//! ## Release mode behavior
//!
//! When `release_mode` is enabled (e.g., `vaisc --release`), all dependent type
//! assertions are **completely omitted** from the generated IR, just like contract
//! checks (`#[requires]`, `#[ensures]`, `#[invariant]`). This is intentional for
//! performance but means refinement type safety is **not enforced at runtime** in
//! release builds. Users relying on dependent types for safety-critical invariants
//! should be aware of this trade-off.
//!
//! ## Interaction with LLVM optimizations
//!
//! The generated assertion pattern (`br i1 %cond, label %ok, label %fail` where
//! `%fail` calls `@__panic` + `unreachable`) is generally preserved by LLVM
//! optimizations because `@__panic` is an external call with observable side effects,
//! and `unreachable` signals that the fail path is a noreturn. However, if the LLVM
//! optimizer can statically prove the predicate is always true (e.g., constant
//! propagation), it may legally remove the branch — which is correct behavior.
//! The `@__panic` call acts as an optimization barrier for non-trivially-provable
//! predicates.

use crate::types::LocalVar;
use crate::{CodeGenerator, CodegenResult};
use vais_ast::{Param, Type};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Generate runtime assertions for dependent type parameters.
    ///
    /// For each parameter with a dependent type, emits:
    /// ```llvm
    /// ; Bind predicate variable to parameter value
    /// %pred_val = <evaluate predicate with bound var = param value>
    /// %pred_i1 = icmp ne i64 %pred_val, 0
    /// br i1 %pred_i1, label %dep_ok_N, label %dep_fail_N
    ///
    /// dep_fail_N:
    ///   call i64 @__panic(i8* <message>)
    ///   unreachable
    ///
    /// dep_ok_N:
    /// ```
    pub(crate) fn generate_dependent_type_assertions(
        &mut self,
        params: &[Param],
        registered_param_types: &[ResolvedType],
        counter: &mut usize,
    ) -> CodegenResult<String> {
        // In release mode, skip all dependent type assertions for performance.
        // See module-level documentation for the safety implications of this behavior.
        if self.release_mode {
            return Ok(String::new());
        }

        let mut ir = String::new();

        for (i, p) in params.iter().enumerate() {
            // Check the AST type for Dependent variant
            if let Type::Dependent {
                var_name,
                predicate,
                ..
            } = &p.ty.node
            {
                // Get the resolved type for this parameter
                let resolved_ty = if i < registered_param_types.len() {
                    registered_param_types[i].clone()
                } else {
                    self.ast_type_to_resolved(&p.ty.node)
                };

                // Extract the base type (unwrap Dependent wrapper)
                let base_ty = resolved_ty.base_type().clone();

                // Get the predicate string from resolved type for error message
                let predicate_display = if let ResolvedType::Dependent {
                    predicate: pred_str,
                    ..
                } = &resolved_ty
                {
                    pred_str.clone()
                } else {
                    format!("{:?}", predicate.node)
                };

                // Get the parameter's local info to determine its LLVM name and kind
                let param_local = if let Some(local) = self.fn_ctx.locals.get(&p.name.node) {
                    local.clone()
                } else {
                    let sanitized = crate::helpers::sanitize_param_name(&p.name.node);
                    LocalVar::param(base_ty.clone(), sanitized)
                };

                // Temporarily bind the predicate's bound variable to the parameter value.
                // We mirror the parameter's storage kind (Param/Ssa/Alloca) so that
                // visit_ident generates the correct LLVM IR access pattern.
                let old_binding = self.fn_ctx.locals.remove(var_name);
                let temp_local = LocalVar {
                    ty: base_ty.clone(),
                    kind: param_local.kind,
                    llvm_name: param_local.llvm_name.clone(),
                };
                self.fn_ctx.locals.insert(var_name.clone(), temp_local);

                // Generate the predicate expression as IR
                let (pred_value, pred_ir) = self.generate_expr(predicate, counter)?;
                ir.push_str(&pred_ir);

                // Restore the old binding (or remove if there wasn't one)
                self.fn_ctx.locals.remove(var_name);
                if let Some(old) = old_binding {
                    self.fn_ctx.locals.insert(var_name.clone(), old);
                }

                // Generate unique labels
                let ok_label = format!("dep_ok_{}", *counter);
                let fail_label = format!("dep_fail_{}", *counter);
                *counter += 1;

                // Convert the predicate result to i1 for branch
                // VAIS uses i64 for bool, but LLVM branch needs i1
                let cond_i1 = format!("%dep_cond_i1_{}", *counter);
                *counter += 1;
                write_ir!(ir, "  {} = icmp ne i64 {}, 0", cond_i1, pred_value);

                // Branch based on predicate result
                write_ir!(
                    ir,
                    "  br i1 {}, label %{}, label %{}",
                    cond_i1,
                    ok_label,
                    fail_label
                );

                // Failure block -- call panic with a descriptive message
                write_ir!(ir, "{}:", fail_label);

                // Create descriptive error message
                let fail_msg = format!(
                    "refinement type violation: parameter '{}' failed predicate '{}'",
                    p.name.node, predicate_display
                );
                let msg_const = self.get_or_create_contract_string(&fail_msg);

                write_ir!(ir, "  call i64 @__panic(i8* {})", msg_const);
                ir.push_str("  unreachable\n");

                // Success block
                write_ir!(ir, "{}:", ok_label);
            }
        }

        Ok(ir)
    }
}
