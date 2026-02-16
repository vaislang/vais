//! Auto-inferred contract checks (nonnull, safe_div, etc.).

use std::collections::HashSet;
use std::fmt::Write;
use vais_ast::{BinOp, Expr, Function, IfElse, Type};
use crate::{CodeGenerator, CodegenResult};

/// Options parsed from #[contract(...)] attribute
#[derive(Debug, Default)]
struct ContractOptions {
    /// Whether #[contract] is present
    enabled: bool,
    /// #[contract(pure)] - function has no side effects
    pure: bool,
    /// #[contract(nonnull)] - pointer/string params must be non-null
    nonnull: bool,
    /// #[contract(safe_div)] - check for division by zero
    safe_div: bool,
    /// #[contract(bounds)] - check array bounds
    bounds: bool,
}

impl CodeGenerator {
    /// Check if a function has any contract attributes
    pub(crate) fn _has_contracts(f: &Function) -> bool {
        f.attributes
            .iter()
            .any(|a| a.name == "requires" || a.name == "ensures" || a.name == "contract")
    }

    /// Check if a function has the #[contract] attribute with specific options
    fn get_contract_options(f: &Function) -> ContractOptions {
        let mut options = ContractOptions::default();

        for attr in &f.attributes {
            if attr.name == "contract" {
                options.enabled = true;
                for arg in &attr.args {
                    match arg.as_str() {
                        "pure" => options.pure = true,
                        "nonnull" => options.nonnull = true,
                        "safe_div" => options.safe_div = true,
                        "bounds" => options.bounds = true,
                        "all" => {
                            options.nonnull = true;
                            options.safe_div = true;
                            options.bounds = true;
                        }
                        _ => {}
                    }
                }
                // If no specific options, enable all auto-inference
                if attr.args.is_empty() {
                    options.nonnull = true;
                    options.safe_div = true;
                }
            }
        }

        options
    }

    /// Generate automatic contract checks based on #[contract] options
    pub(crate) fn generate_auto_contract_checks(
        &mut self,
        f: &Function,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        // Skip in release mode
        if self.release_mode {
            return Ok(String::new());
        }

        let options = Self::get_contract_options(f);
        if !options.enabled {
            return Ok(String::new());
        }

        let mut ir = String::new();

        // Generate nonnull checks for pointer/string parameters
        if options.nonnull {
            ir.push_str(&self.generate_nonnull_checks(f, counter)?);
        }

        // Generate safe_div checks by analyzing function body for division
        if options.safe_div {
            ir.push_str(&self.generate_safe_div_checks(f, counter)?);
        }

        Ok(ir)
    }

    /// Generate non-null checks for pointer and string parameters
    fn generate_nonnull_checks(
        &mut self,
        f: &Function,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        let mut ir = String::new();

        for param in &f.params {
            if self.is_nullable_type(&param.ty.node) {
                let param_name = &param.name.node;
                ir.push_str(&self.generate_nonnull_check_for_param(
                    param_name,
                    &f.name.node,
                    counter,
                )?);
            }
        }

        Ok(ir)
    }

    /// Check if a type is nullable (pointer or string)
    fn is_nullable_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Pointer(_) => true,
            Type::Ref(_) | Type::RefMut(_) => true,
            Type::Named { name, .. } => name == "str",
            _ => false,
        }
    }

    /// Generate a non-null check for a single parameter
    fn generate_nonnull_check_for_param(
        &mut self,
        param_name: &str,
        func_name: &str,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        let mut ir = String::new();

        let ok_label = format!("nonnull_ok_{}_{}", param_name, *counter);
        let fail_label = format!("nonnull_fail_{}_{}", param_name, *counter);
        *counter += 1;

        // Load the parameter value (it's stored as i8* or i64 depending on type)
        // For simplicity, we compare against null (0)
        let param_ptr = format!("%{}", param_name);

        // Convert to i64 for comparison (strings and pointers are i8*)
        let cond_i1 = format!("%nonnull_cond_{}", *counter);
        *counter += 1;

        writeln!(ir, "  {} = icmp ne i8* {}, null", cond_i1, param_ptr).unwrap();

        writeln!(
            ir,
            "  br i1 {}, label %{}, label %{}",
            cond_i1, ok_label, fail_label
        )
        .unwrap();

        // Failure block
        writeln!(ir, "{}:", fail_label).unwrap();

        let kind_value = 1; // CONTRACT_REQUIRES
        let condition_str = self.get_or_create_contract_string(&format!("{} != null", param_name));
        let file_name = self
            .fn_ctx
            .current_file
            .as_deref()
            .unwrap_or("unknown")
            .to_string();
        let file_str = self.get_or_create_contract_string(&file_name);
        let func_str = self.get_or_create_contract_string(func_name);

        writeln!(
            ir,
            "  call i64 @__contract_fail(i64 {}, i8* {}, i8* {}, i64 0, i8* {})",
            kind_value, condition_str, file_str, func_str
        )
        .unwrap();
        ir.push_str("  unreachable\n");

        // Success block
        writeln!(ir, "{}:", ok_label).unwrap();

        Ok(ir)
    }

    /// Generate safe division checks by analyzing the function body
    fn generate_safe_div_checks(
        &mut self,
        f: &Function,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        let mut ir = String::new();

        // Find all integer parameters that are used as divisors
        let divisor_params = self.find_divisor_params(f);

        for param_name in divisor_params {
            ir.push_str(&self.generate_nonzero_check_for_param(
                &param_name,
                &f.name.node,
                counter,
            )?);
        }

        Ok(ir)
    }

    /// Find parameters that are used as divisors in division operations
    fn find_divisor_params(&self, f: &Function) -> Vec<String> {
        // Use HashSet for O(1) duplicate detection instead of O(n) Vec search
        let mut divisors = HashSet::new();

        // Collect integer parameter names (using &str to avoid clone)
        let int_params: Vec<&str> = f
            .params
            .iter()
            .filter(|p| self.is_integer_type(&p.ty.node))
            .map(|p| p.name.node.as_str())
            .collect();

        // Analyze function body for division operations
        match &f.body {
            vais_ast::FunctionBody::Expr(expr) => {
                self.find_divisors_in_expr(&expr.node, &int_params, &mut divisors);
            }
            vais_ast::FunctionBody::Block(stmts) => {
                for stmt in stmts {
                    self.find_divisors_in_stmt(&stmt.node, &int_params, &mut divisors);
                }
            }
        }

        // Convert HashSet to Vec for return
        divisors.into_iter().collect()
    }

    /// Check if a type is an integer type
    fn is_integer_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Named { name, .. } => {
                matches!(
                    name.as_str(),
                    "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize"
                )
            }
            _ => false,
        }
    }

    /// Find divisor variables in an expression
    fn find_divisors_in_expr(&self, expr: &Expr, params: &[&str], divisors: &mut HashSet<String>) {
        match expr {
            Expr::Binary {
                op, right, left, ..
            } => {
                if matches!(op, BinOp::Div | BinOp::Mod) {
                    // Check if right-hand side is a parameter
                    if let Expr::Ident(name) = &right.node {
                        if params.contains(&name.as_str()) {
                            // HashSet::insert automatically handles duplicates - O(1) check
                            divisors.insert(name.clone());
                        }
                    }
                }
                self.find_divisors_in_expr(&left.node, params, divisors);
                self.find_divisors_in_expr(&right.node, params, divisors);
            }
            Expr::Unary { expr: inner, .. } => {
                self.find_divisors_in_expr(&inner.node, params, divisors);
            }
            Expr::Call { func, args, .. } => {
                self.find_divisors_in_expr(&func.node, params, divisors);
                for arg in args {
                    self.find_divisors_in_expr(&arg.node, params, divisors);
                }
            }
            Expr::If {
                cond, then, else_, ..
            } => {
                self.find_divisors_in_expr(&cond.node, params, divisors);
                for stmt in then {
                    self.find_divisors_in_stmt(&stmt.node, params, divisors);
                }
                if let Some(else_branch) = else_ {
                    self.find_divisors_in_if_else(else_branch, params, divisors);
                }
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.find_divisors_in_stmt(&stmt.node, params, divisors);
                }
            }
            _ => {}
        }
    }

    /// Find divisor variables in a statement
    fn find_divisors_in_stmt(
        &self,
        stmt: &vais_ast::Stmt,
        params: &[&str],
        divisors: &mut HashSet<String>,
    ) {
        match stmt {
            vais_ast::Stmt::Let { value, .. } => {
                self.find_divisors_in_expr(&value.node, params, divisors);
            }
            vais_ast::Stmt::Expr(expr) => {
                self.find_divisors_in_expr(&expr.node, params, divisors);
            }
            vais_ast::Stmt::Return(Some(expr)) => {
                self.find_divisors_in_expr(&expr.node, params, divisors);
            }
            _ => {}
        }
    }

    /// Find divisors in if-else branches
    fn find_divisors_in_if_else(
        &self,
        branch: &IfElse,
        params: &[&str],
        divisors: &mut HashSet<String>,
    ) {
        match branch {
            IfElse::ElseIf(cond, then, else_) => {
                self.find_divisors_in_expr(&cond.node, params, divisors);
                for stmt in then {
                    self.find_divisors_in_stmt(&stmt.node, params, divisors);
                }
                if let Some(next) = else_ {
                    self.find_divisors_in_if_else(next, params, divisors);
                }
            }
            IfElse::Else(stmts) => {
                for stmt in stmts {
                    self.find_divisors_in_stmt(&stmt.node, params, divisors);
                }
            }
        }
    }

    /// Generate a non-zero check for a parameter
    fn generate_nonzero_check_for_param(
        &mut self,
        param_name: &str,
        func_name: &str,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        let mut ir = String::new();

        let ok_label = format!("nonzero_ok_{}_{}", param_name, *counter);
        let fail_label = format!("nonzero_fail_{}_{}", param_name, *counter);
        *counter += 1;

        // Parameters are passed by value for integers - use directly
        let param_ref = format!("%{}", param_name);

        // Check if not zero
        let cond_i1 = format!("%nonzero_cond_{}", *counter);
        *counter += 1;
        writeln!(ir, "  {} = icmp ne i64 {}, 0", cond_i1, param_ref).unwrap();

        writeln!(
            ir,
            "  br i1 {}, label %{}, label %{}",
            cond_i1, ok_label, fail_label
        )
        .unwrap();

        // Failure block
        writeln!(ir, "{}:", fail_label).unwrap();

        let kind_value = 1; // CONTRACT_REQUIRES
        let condition_str =
            self.get_or_create_contract_string(&format!("{} != 0 (division by zero)", param_name));
        let file_name = self
            .fn_ctx
            .current_file
            .as_deref()
            .unwrap_or("unknown")
            .to_string();
        let file_str = self.get_or_create_contract_string(&file_name);
        let func_str = self.get_or_create_contract_string(func_name);

        writeln!(
            ir,
            "  call i64 @__contract_fail(i64 {}, i8* {}, i8* {}, i64 0, i8* {})",
            kind_value, condition_str, file_str, func_str
        )
        .unwrap();
        ir.push_str("  unreachable\n");

        // Success block
        writeln!(ir, "{}:", ok_label).unwrap();

        Ok(ir)
    }
}
