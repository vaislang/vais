//! Termination proofs and decreases checks.

use std::fmt::Write;
use vais_ast::{Function, Spanned, Expr, IfElse};
use crate::{CodeGenerator, CodegenResult};

impl CodeGenerator {

    /// Generate old() snapshots for ensures clauses
    ///
    /// Called at function entry to capture pre-state values for old() references.
    pub(crate) fn _generate_old_snapshots(
        &mut self,
        f: &Function,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        // Skip if no ensures clauses with old()
        if self.release_mode {
            return Ok(String::new());
        }

        let mut ir = String::new();

        // Find all old() expressions in ensures clauses
        for attr in &f.attributes {
            if attr.name == "ensures" {
                if let Some(expr) = &attr.expr {
                    ir.push_str(&self._capture_old_expressions(expr, counter)?);
                }
            }
        }

        Ok(ir)
    }

    /// Recursively find and capture old() expressions
    fn _capture_old_expressions(
        &mut self,
        expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        let mut ir = String::new();

        match &expr.node {
            Expr::Old(inner) => {
                // Capture this expression's value
                let (value, value_ir) = self.generate_expr(inner, counter)?;
                ir.push_str(&value_ir);

                // Allocate storage for the snapshot
                let snapshot_name = format!("__old_snapshot_{}", *counter);
                let ty = self.infer_expr_type(inner);
                let llvm_ty = self.type_to_llvm(&ty);

                writeln!(ir, "  %{} = alloca {}", snapshot_name, llvm_ty).unwrap();
                writeln!(
                    ir,
                    "  store {} {}, {}* %{}",
                    llvm_ty, value, llvm_ty, snapshot_name
                )
                .unwrap();

                // Register the snapshot
                let old_var_name = format!("__old_{}", *counter);
                self.contracts
                    .old_snapshots
                    .insert(old_var_name, snapshot_name);
                *counter += 1;
            }

            // Recurse into sub-expressions
            Expr::Binary { left, right, .. } => {
                ir.push_str(&self._capture_old_expressions(left, counter)?);
                ir.push_str(&self._capture_old_expressions(right, counter)?);
            }
            Expr::Unary { expr: inner, .. } => {
                ir.push_str(&self._capture_old_expressions(inner, counter)?);
            }
            Expr::Call { func, args, .. } => {
                ir.push_str(&self._capture_old_expressions(func, counter)?);
                for arg in args {
                    ir.push_str(&self._capture_old_expressions(arg, counter)?);
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                ir.push_str(&self._capture_old_expressions(receiver, counter)?);
                for arg in args {
                    ir.push_str(&self._capture_old_expressions(arg, counter)?);
                }
            }
            Expr::Field { expr: inner, .. } => {
                ir.push_str(&self._capture_old_expressions(inner, counter)?);
            }
            Expr::Index { expr: inner, index } => {
                ir.push_str(&self._capture_old_expressions(inner, counter)?);
                ir.push_str(&self._capture_old_expressions(index, counter)?);
            }
            Expr::If { cond, then, else_ } => {
                ir.push_str(&self._capture_old_expressions(cond, counter)?);
                for stmt in then {
                    if let vais_ast::Stmt::Expr(e) = &stmt.node {
                        ir.push_str(&self._capture_old_expressions(e, counter)?);
                    }
                }
                if let Some(else_branch) = else_ {
                    ir.push_str(&self._capture_old_in_if_else(else_branch, counter)?);
                }
            }
            Expr::Ternary { cond, then, else_ } => {
                ir.push_str(&self._capture_old_expressions(cond, counter)?);
                ir.push_str(&self._capture_old_expressions(then, counter)?);
                ir.push_str(&self._capture_old_expressions(else_, counter)?);
            }
            _ => {}
        }

        Ok(ir)
    }

    fn _capture_old_in_if_else(
        &mut self,
        branch: &IfElse,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        let mut ir = String::new();
        match branch {
            IfElse::ElseIf(cond, then, else_) => {
                ir.push_str(&self._capture_old_expressions(cond, counter)?);
                for stmt in then {
                    if let vais_ast::Stmt::Expr(e) = &stmt.node {
                        ir.push_str(&self._capture_old_expressions(e, counter)?);
                    }
                }
                if let Some(next) = else_ {
                    ir.push_str(&self._capture_old_in_if_else(next, counter)?);
                }
            }
            IfElse::Else(stmts) => {
                for stmt in stmts {
                    if let vais_ast::Stmt::Expr(e) = &stmt.node {
                        ir.push_str(&self._capture_old_expressions(e, counter)?);
                    }
                }
            }
        }
        Ok(ir)
    }

    /// Generate decreases check for termination proofs
    ///
    /// Verifies that the decreases expression is non-negative and strictly
    /// decreasing on recursive calls.
    pub(crate) fn generate_decreases_checks(
        &mut self,
        f: &Function,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        // Skip in release mode
        if self.release_mode {
            return Ok(String::new());
        }

        let mut ir = String::new();

        for (idx, attr) in f.attributes.iter().enumerate() {
            if attr.name == "decreases" {
                if let Some(expr) = &attr.expr {
                    // Generate the decreases expression value
                    let (value, value_ir) = self.generate_expr(expr, counter)?;
                    ir.push_str(&value_ir);

                    // Store the initial value for comparison in recursive calls
                    let storage_name = format!("__decreases_{}_{}", f.name.node, idx);
                    writeln!(ir, "  %{} = alloca i64", storage_name).unwrap();
                    writeln!(ir, "  store i64 {}, i64* %{}", value, storage_name).unwrap();

                    // Store decreases info for recursive call checking
                    self.contracts.current_decreases_info = Some(crate::DecreasesInfo {
                        storage_name,
                        expr: expr.clone(),
                        function_name: f.name.node.clone(),
                    });

                    // Check that value is non-negative
                    let ok_label = format!("decreases_nonneg_ok_{}_{}", idx, *counter);
                    let fail_label = format!("decreases_nonneg_fail_{}_{}", idx, *counter);
                    *counter += 1;

                    let cmp_result = format!("%decreases_cmp_{}", *counter);
                    *counter += 1;
                    writeln!(ir, "  {} = icmp sge i64 {}, 0", cmp_result, value).unwrap();
                    writeln!(
                        ir,
                        "  br i1 {}, label %{}, label %{}",
                        cmp_result, ok_label, fail_label
                    )
                    .unwrap();

                    // Failure block
                    writeln!(ir, "{}:", fail_label).unwrap();

                    let fail_msg = format!(
                        "decreases expression must be non-negative in function '{}'",
                        f.name.node
                    );
                    let msg_const = self.get_or_create_contract_string(&fail_msg);
                    writeln!(ir, "  call i64 @__panic(i8* {})", msg_const).unwrap();
                    ir.push_str("  unreachable\n");

                    // Success block
                    writeln!(ir, "{}:", ok_label).unwrap();
                }
            }
        }

        Ok(ir)
    }

    /// Generate decreases check before a recursive call
    ///
    /// This verifies that the decreases expression is strictly less than
    /// the value stored at function entry.
    pub(crate) fn generate_recursive_decreases_check(
        &mut self,
        args: &[vais_ast::Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<String> {
        // Skip in release mode
        if self.release_mode {
            return Ok(String::new());
        }

        let decreases_info = match &self.contracts.current_decreases_info {
            Some(info) => info.clone(),
            None => return Ok(String::new()),
        };

        let mut ir = String::new();

        // Substitute parameters in decreases expression with call arguments
        // For now, we assume the decreases expression uses simple parameter references
        // and we need to evaluate the decreases expression with the call arguments

        // Create temporary bindings for call arguments
        // This is a simplified version - assumes decreases expression uses first param
        // A more complete solution would analyze the expression and substitute properly

        // For the common case of `#[decreases(n)]` where n is the first parameter,
        // we evaluate the decreases expression using the call arguments

        // First, save current locals state.
        // NOTE: This is a full HashMap clone - required because we need to:
        // 1. Generate expressions for call arguments (needs current scope)
        // 2. Temporarily rebind parameters to new values
        // 3. Restore original bindings after decreases check
        // This is expensive but necessary for correctness in recursive call analysis.
        let saved_locals = self.fn_ctx.locals.clone();

        // Get the function info to map parameters
        let func_name = &decreases_info.function_name;

        // NOTE: Cannot use direct reference here - borrow checker conflict.
        // We need to modify self.fn_ctx.locals (line 1087) while holding a reference
        // to self.types.functions, which would violate Rust's borrowing rules.
        // The clone is necessary to avoid simultaneous mutable and immutable borrows of self.
        let func_info = self.types.functions.get(func_name).cloned();

        if let Some(ref info) = func_info {
            // Temporarily rebind parameters to call argument values
            let mut arg_values = Vec::new();
            for arg in args {
                let (val, arg_ir) = self.generate_expr(arg, counter)?;
                ir.push_str(&arg_ir);
                arg_values.push(val);
            }

            // Create temporary locals for parameter names with argument values
            for (i, (param_name, param_type, _)) in info.signature.params.iter().enumerate() {
                if let Some(arg_val) = arg_values.get(i) {
                    // Create a temporary alloca for the argument value
                    let temp_var = format!("__decreases_arg_{}_{}", i, *counter);
                    *counter += 1;
                    let ty = self.type_to_llvm(param_type);
                    writeln!(ir, "  %{} = alloca {}", temp_var, ty).unwrap();
                    writeln!(ir, "  store {} {}, {}* %{}", ty, arg_val, ty, temp_var).unwrap();

                    // Register in locals so generate_expr can find it
                    self.fn_ctx.locals.insert(
                        param_name.to_string(),
                        crate::types::LocalVar::alloca(param_type.clone(), temp_var),
                    );
                }
            }
        }

        // Generate the new decreases value (using call arguments)
        let (new_value, new_ir) = self.generate_expr(&decreases_info.expr, counter)?;
        ir.push_str(&new_ir);

        // Restore saved locals
        self.fn_ctx.locals = saved_locals;

        // Load the original decreases value
        let old_value = self.next_temp(counter);
        writeln!(
            ir,
            "  {} = load i64, i64* %{}",
            old_value, decreases_info.storage_name
        )
        .unwrap();

        // Check that new_value < old_value (strictly decreasing)
        let ok_label = format!("decreases_check_ok_{}", *counter);
        let fail_label = format!("decreases_check_fail_{}", *counter);
        *counter += 1;

        let cmp_result = format!("%decreases_strict_cmp_{}", *counter);
        *counter += 1;
        writeln!(
            ir,
            "  {} = icmp slt i64 {}, {}",
            cmp_result, new_value, old_value
        )
        .unwrap();
        writeln!(
            ir,
            "  br i1 {}, label %{}, label %{}",
            cmp_result, ok_label, fail_label
        )
        .unwrap();

        // Failure block
        writeln!(ir, "{}:", fail_label).unwrap();

        let fail_msg = format!(
            "decreases expression must strictly decrease on recursive call in '{}'",
            decreases_info.function_name
        );
        let msg_const = self.get_or_create_contract_string(&fail_msg);
        writeln!(ir, "  call i64 @__panic(i8* {})", msg_const).unwrap();
        ir.push_str("  unreachable\n");

        // Success block
        writeln!(ir, "{}:", ok_label).unwrap();

        Ok(ir)
    }

    /// Clear decreases info (called when leaving function scope)
    pub(crate) fn clear_decreases_info(&mut self) {
        self.contracts.current_decreases_info = None;
    }

    /// Check if current function has a decreases clause
    pub(crate) fn _has_decreases_clause(&self) -> bool {
        self.contracts.current_decreases_info.is_some()
    }

    /// Get the function name with decreases clause (if any)
    pub(crate) fn _get_decreases_function_name(&self) -> Option<&str> {
        self.contracts
            .current_decreases_info
            .as_ref()
            .map(|info| info.function_name.as_str())
    }
}
