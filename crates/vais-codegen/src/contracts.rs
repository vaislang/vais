//! Contract code generation for Design by Contract
//!
//! Generates LLVM IR for requires (preconditions) and ensures (postconditions).
//! Contract checks are only generated in debug builds.
//!
//! ## #[contract] Attribute Macro
//!
//! The `#[contract]` attribute provides automatic contract inference based on
//! function signatures and common patterns:
//!
//! - `#[contract]` - Enable auto-inference for common patterns
//! - `#[contract(pure)]` - Mark as pure function (no side effects)
//! - `#[contract(nonnull)]` - All pointer/str params are non-null
//! - `#[contract(safe_div)]` - Division by zero checks
//! - `#[contract(bounds)]` - Array bounds checking
//!
//! Example:
//! ```vais
//! #[contract(safe_div)]
//! F div(a: i64, b: i64) -> i64 { a / b }  // Auto: requires(b != 0)
//!
//! #[contract(nonnull)]
//! F strlen(s: str) -> i64 { ... }  // Auto: requires(s != null)
//! ```

use crate::{CodeGenerator, CodegenResult};
use vais_ast::{BinOp, Expr, Function, IfElse, Spanned, Type};
use vais_types::ResolvedType;

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

        ir.push_str(&format!(
            "  %{} = alloca {}\n",
            return_var_name, return_llvm
        ));
        ir.push_str(&format!(
            "  store {} {}, {}* %{}\n",
            return_llvm, return_value, return_llvm, return_var_name
        ));

        // Register 'return' in locals for expression generation
        // Use alloca since we stored the return value at return_var_name
        self.locals.insert(
            "return".to_string(),
            crate::types::LocalVar::alloca(ret_type.clone(), return_var_name.clone()),
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
        self.locals.remove("return");

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
    fn generate_contract_check(
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
        ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_i1, cond_value));

        // Branch based on condition
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cond_i1, ok_label, fail_label
        ));

        // Failure block
        ir.push_str(&format!("{}:\n", fail_label));

        // Contract kind: 1 = requires, 2 = ensures
        let kind_value = if kind == "requires" { 1 } else { 2 };

        // Create string constants for error message
        let condition_str =
            self.get_or_create_contract_string(&format!("{} condition #{}", kind, idx));
        let file_name = self
            .current_file
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let file_str = self.get_or_create_contract_string(&file_name);
        let func_str = self.get_or_create_contract_string(func_name);

        // Get line number from span
        let line = self.debug_info.offset_to_line(expr.span.start) as i64;

        // Call __contract_fail
        ir.push_str(&format!(
            "  call i64 @__contract_fail(i64 {}, i8* {}, i8* {}, i64 {}, i8* {})\n",
            kind_value, condition_str, file_str, line, func_str
        ));
        ir.push_str("  unreachable\n");

        // Success block
        ir.push_str(&format!("{}:\n", ok_label));

        Ok(ir)
    }

    /// Get or create a contract string constant, returning an i8* GEP expression
    fn get_or_create_contract_string(&mut self, s: &str) -> String {
        // Check if we already have this string
        if let Some(name) = self.contract_string_constants.get(s) {
            return format!(
                "getelementptr inbounds ([{} x i8], [{} x i8]* {}, i64 0, i64 0)",
                s.len() + 1,
                s.len() + 1,
                name
            );
        }

        // Create a new string constant
        let const_name = format!("@.str.contract.{}", self.contract_string_counter);
        self.contract_string_counter += 1;

        self.contract_string_constants
            .insert(s.to_string(), const_name.clone());

        format!(
            "getelementptr inbounds ([{} x i8], [{} x i8]* {}, i64 0, i64 0)",
            s.len() + 1,
            s.len() + 1,
            const_name
        )
    }

    /// Generate declarations for contract runtime functions
    pub(crate) fn generate_contract_declarations(&self) -> String {
        // Only generate if we have any contracts
        if self.contract_string_constants.is_empty() && self.release_mode {
            return String::new();
        }

        let mut ir = String::new();
        ir.push_str("; Contract runtime declarations\n");
        // Note: __contract_fail and __panic are now defined in generate_helper_functions()
        // LLVM assume intrinsic for optimization hints (used by assume() in release mode)
        ir.push_str("declare void @llvm.assume(i1)\n");
        ir.push('\n');

        ir
    }

    /// Generate string constants for contract messages
    pub(crate) fn generate_contract_string_constants(&self) -> String {
        let mut ir = String::new();

        for (s, name) in &self.contract_string_constants {
            let escaped = escape_string_for_llvm(s);
            ir.push_str(&format!(
                "{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
                name,
                s.len() + 1,
                escaped
            ));
        }

        ir
    }

    /// Check if a function has any contract attributes
    #[allow(dead_code)]
    pub(crate) fn has_contracts(f: &Function) -> bool {
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

        ir.push_str(&format!(
            "  {} = icmp ne i8* {}, null\n",
            cond_i1, param_ptr
        ));

        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cond_i1, ok_label, fail_label
        ));

        // Failure block
        ir.push_str(&format!("{}:\n", fail_label));

        let kind_value = 1; // CONTRACT_REQUIRES
        let condition_str = self.get_or_create_contract_string(&format!("{} != null", param_name));
        let file_name = self
            .current_file
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let file_str = self.get_or_create_contract_string(&file_name);
        let func_str = self.get_or_create_contract_string(func_name);

        ir.push_str(&format!(
            "  call i64 @__contract_fail(i64 {}, i8* {}, i8* {}, i64 0, i8* {})\n",
            kind_value, condition_str, file_str, func_str
        ));
        ir.push_str("  unreachable\n");

        // Success block
        ir.push_str(&format!("{}:\n", ok_label));

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
        let mut divisors = Vec::new();

        // Collect integer parameter names
        let int_params: Vec<String> = f
            .params
            .iter()
            .filter(|p| self.is_integer_type(&p.ty.node))
            .map(|p| p.name.node.clone())
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

        divisors
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
    fn find_divisors_in_expr(&self, expr: &Expr, params: &[String], divisors: &mut Vec<String>) {
        match expr {
            Expr::Binary {
                op, right, left, ..
            } => {
                if matches!(op, BinOp::Div | BinOp::Mod) {
                    // Check if right-hand side is a parameter
                    if let Expr::Ident(name) = &right.node {
                        if params.contains(name) && !divisors.contains(name) {
                            divisors.push(name.clone());
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
        params: &[String],
        divisors: &mut Vec<String>,
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
        params: &[String],
        divisors: &mut Vec<String>,
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
        ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_i1, param_ref));

        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cond_i1, ok_label, fail_label
        ));

        // Failure block
        ir.push_str(&format!("{}:\n", fail_label));

        let kind_value = 1; // CONTRACT_REQUIRES
        let condition_str =
            self.get_or_create_contract_string(&format!("{} != 0 (division by zero)", param_name));
        let file_name = self
            .current_file
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let file_str = self.get_or_create_contract_string(&file_name);
        let func_str = self.get_or_create_contract_string(func_name);

        ir.push_str(&format!(
            "  call i64 @__contract_fail(i64 {}, i8* {}, i8* {}, i64 0, i8* {})\n",
            kind_value, condition_str, file_str, func_str
        ));
        ir.push_str("  unreachable\n");

        // Success block
        ir.push_str(&format!("{}:\n", ok_label));

        Ok(ir)
    }
}

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

/// Escape a string for LLVM IR constant
fn escape_string_for_llvm(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\5C"),
            '"' => result.push_str("\\22"),
            '\n' => result.push_str("\\0A"),
            '\r' => result.push_str("\\0D"),
            '\t' => result.push_str("\\09"),
            c if c.is_ascii_graphic() || c == ' ' => result.push(c),
            c => {
                // Escape non-printable characters as hex
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("\\{:02X}", byte));
                }
            }
        }
    }
    result
}

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
        ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_i1, cond_value));

        // Branch based on condition
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cond_i1, ok_label, fail_label
        ));

        // Failure block
        ir.push_str(&format!("{}:\n", fail_label));

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
                self.current_file.as_deref().unwrap_or("unknown"),
                self.debug_info.offset_to_line(condition.span.start)
            );

            self.get_or_create_contract_string(&default_msg)
        };

        // Call __panic to terminate
        ir.push_str(&format!("  call i64 @__panic(i8* {})\n", msg_str));
        ir.push_str("  unreachable\n");

        // Success block
        ir.push_str(&format!("{}:\n", ok_label));

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
        ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_i1, cond_value));

        if self.release_mode {
            // In release mode, use LLVM assume intrinsic for optimization hints
            ir.push_str(&format!("  call void @llvm.assume(i1 {})\n", cond_i1));
        } else {
            // In debug mode, check the assumption
            let ok_label = format!("assume_ok_{}", *counter);
            let fail_label = format!("assume_fail_{}", *counter);
            *counter += 1;

            ir.push_str(&format!(
                "  br i1 {}, label %{}, label %{}\n",
                cond_i1, ok_label, fail_label
            ));

            // Failure block
            ir.push_str(&format!("{}:\n", fail_label));

            let fail_msg = format!(
                "Assumption violated at {}:{}",
                self.current_file.as_deref().unwrap_or("unknown"),
                self.debug_info.offset_to_line(condition.span.start)
            );
            let msg_const = self.get_or_create_contract_string(&fail_msg);

            ir.push_str(&format!("  call i64 @__panic(i8* {})\n", msg_const));
            ir.push_str("  unreachable\n");

            // Success block
            ir.push_str(&format!("{}:\n", ok_label));
        }

        // Assume returns unit (0)
        Ok(("0".to_string(), ir))
    }

    /// Generate invariant checks for a struct type
    ///
    /// Called after struct construction/modification to verify invariants.
    #[allow(dead_code)]
    pub(crate) fn generate_invariant_checks(
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
        let struct_info = self.structs.get(struct_name).cloned();
        let invariants = struct_info
            .as_ref()
            .map(|s| s.invariants.clone())
            .unwrap_or_default();

        if invariants.is_empty() {
            return Ok(String::new());
        }

        let mut ir = String::new();

        for (idx, invariant_expr) in invariants.iter().enumerate() {
            // Generate the invariant condition
            // Note: The invariant expression can reference struct fields via 'self'
            // We need to set up 'self' to point to struct_ptr
            let saved_self = self.locals.get("self").cloned();

            if let Some(_si) = &struct_info {
                self.locals.insert(
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
                self.locals.insert("self".to_string(), prev);
            } else {
                self.locals.remove("self");
            }

            // Generate check
            let ok_label = format!("invariant_ok_{}_{}", struct_name, idx);
            let fail_label = format!("invariant_fail_{}_{}", struct_name, idx);

            let cond_i1 = format!("%invariant_cond_i1_{}", *counter);
            *counter += 1;
            ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_i1, cond_value));

            ir.push_str(&format!(
                "  br i1 {}, label %{}, label %{}\n",
                cond_i1, ok_label, fail_label
            ));

            // Failure block
            ir.push_str(&format!("{}:\n", fail_label));

            let kind_value = 3; // CONTRACT_INVARIANT
            let condition_str = self
                .get_or_create_contract_string(&format!("invariant #{} of {}", idx, struct_name));
            let file_name = self
                .current_file
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let file_str = self.get_or_create_contract_string(&file_name);
            let func_str = self.get_or_create_contract_string(
                &self
                    .current_function
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
            );
            let line = self.debug_info.offset_to_line(invariant_expr.span.start) as i64;

            ir.push_str(&format!(
                "  call i64 @__contract_fail(i64 {}, i8* {}, i8* {}, i64 {}, i8* {})\n",
                kind_value, condition_str, file_str, line, func_str
            ));
            ir.push_str("  unreachable\n");

            // Success block
            ir.push_str(&format!("{}:\n", ok_label));
        }

        Ok(ir)
    }

    /// Generate old() snapshots for ensures clauses
    ///
    /// Called at function entry to capture pre-state values for old() references.
    #[allow(dead_code)]
    pub(crate) fn generate_old_snapshots(
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
                    ir.push_str(&self.capture_old_expressions(expr, counter)?);
                }
            }
        }

        Ok(ir)
    }

    /// Recursively find and capture old() expressions
    #[allow(dead_code)]
    fn capture_old_expressions(
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

                ir.push_str(&format!("  %{} = alloca {}\n", snapshot_name, llvm_ty));
                ir.push_str(&format!(
                    "  store {} {}, {}* %{}\n",
                    llvm_ty, value, llvm_ty, snapshot_name
                ));

                // Register the snapshot
                let old_var_name = format!("__old_{}", *counter);
                self.old_snapshots.insert(old_var_name, snapshot_name);
                *counter += 1;
            }

            // Recurse into sub-expressions
            Expr::Binary { left, right, .. } => {
                ir.push_str(&self.capture_old_expressions(left, counter)?);
                ir.push_str(&self.capture_old_expressions(right, counter)?);
            }
            Expr::Unary { expr: inner, .. } => {
                ir.push_str(&self.capture_old_expressions(inner, counter)?);
            }
            Expr::Call { func, args, .. } => {
                ir.push_str(&self.capture_old_expressions(func, counter)?);
                for arg in args {
                    ir.push_str(&self.capture_old_expressions(arg, counter)?);
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                ir.push_str(&self.capture_old_expressions(receiver, counter)?);
                for arg in args {
                    ir.push_str(&self.capture_old_expressions(arg, counter)?);
                }
            }
            Expr::Field { expr: inner, .. } => {
                ir.push_str(&self.capture_old_expressions(inner, counter)?);
            }
            Expr::Index { expr: inner, index } => {
                ir.push_str(&self.capture_old_expressions(inner, counter)?);
                ir.push_str(&self.capture_old_expressions(index, counter)?);
            }
            Expr::If { cond, then, else_ } => {
                ir.push_str(&self.capture_old_expressions(cond, counter)?);
                for stmt in then {
                    if let vais_ast::Stmt::Expr(e) = &stmt.node {
                        ir.push_str(&self.capture_old_expressions(e, counter)?);
                    }
                }
                if let Some(else_branch) = else_ {
                    ir.push_str(&self.capture_old_in_if_else(else_branch, counter)?);
                }
            }
            Expr::Ternary { cond, then, else_ } => {
                ir.push_str(&self.capture_old_expressions(cond, counter)?);
                ir.push_str(&self.capture_old_expressions(then, counter)?);
                ir.push_str(&self.capture_old_expressions(else_, counter)?);
            }
            _ => {}
        }

        Ok(ir)
    }

    #[allow(dead_code)]
    fn capture_old_in_if_else(
        &mut self,
        branch: &IfElse,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        let mut ir = String::new();
        match branch {
            IfElse::ElseIf(cond, then, else_) => {
                ir.push_str(&self.capture_old_expressions(cond, counter)?);
                for stmt in then {
                    if let vais_ast::Stmt::Expr(e) = &stmt.node {
                        ir.push_str(&self.capture_old_expressions(e, counter)?);
                    }
                }
                if let Some(next) = else_ {
                    ir.push_str(&self.capture_old_in_if_else(next, counter)?);
                }
            }
            IfElse::Else(stmts) => {
                for stmt in stmts {
                    if let vais_ast::Stmt::Expr(e) = &stmt.node {
                        ir.push_str(&self.capture_old_expressions(e, counter)?);
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
                    ir.push_str(&format!("  %{} = alloca i64\n", storage_name));
                    ir.push_str(&format!("  store i64 {}, i64* %{}\n", value, storage_name));

                    // Store decreases info for recursive call checking
                    self.current_decreases_info = Some(crate::DecreasesInfo {
                        storage_name: storage_name.clone(),
                        expr: expr.clone(),
                        function_name: f.name.node.clone(),
                    });

                    // Check that value is non-negative
                    let ok_label = format!("decreases_nonneg_ok_{}_{}", idx, *counter);
                    let fail_label = format!("decreases_nonneg_fail_{}_{}", idx, *counter);
                    *counter += 1;

                    let cmp_result = format!("%decreases_cmp_{}", *counter);
                    *counter += 1;
                    ir.push_str(&format!("  {} = icmp sge i64 {}, 0\n", cmp_result, value));
                    ir.push_str(&format!(
                        "  br i1 {}, label %{}, label %{}\n",
                        cmp_result, ok_label, fail_label
                    ));

                    // Failure block
                    ir.push_str(&format!("{}:\n", fail_label));

                    let fail_msg = format!(
                        "decreases expression must be non-negative in function '{}'",
                        f.name.node
                    );
                    let msg_const = self.get_or_create_contract_string(&fail_msg);
                    ir.push_str(&format!("  call i64 @__panic(i8* {})\n", msg_const));
                    ir.push_str("  unreachable\n");

                    // Success block
                    ir.push_str(&format!("{}:\n", ok_label));
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

        let decreases_info = match &self.current_decreases_info {
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

        // First, save current locals state
        let saved_locals = self.locals.clone();

        // Get the function info to map parameters
        let func_name = &decreases_info.function_name;
        let func_info = self.functions.get(func_name).cloned();

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
                    ir.push_str(&format!("  %{} = alloca {}\n", temp_var, ty));
                    ir.push_str(&format!(
                        "  store {} {}, {}* %{}\n",
                        ty, arg_val, ty, temp_var
                    ));

                    // Register in locals so generate_expr can find it
                    self.locals.insert(
                        param_name.clone(),
                        crate::types::LocalVar::alloca(param_type.clone(), temp_var),
                    );
                }
            }
        }

        // Generate the new decreases value (using call arguments)
        let (new_value, new_ir) = self.generate_expr(&decreases_info.expr, counter)?;
        ir.push_str(&new_ir);

        // Restore saved locals
        self.locals = saved_locals;

        // Load the original decreases value
        let old_value = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load i64, i64* %{}\n",
            old_value, decreases_info.storage_name
        ));

        // Check that new_value < old_value (strictly decreasing)
        let ok_label = format!("decreases_check_ok_{}", *counter);
        let fail_label = format!("decreases_check_fail_{}", *counter);
        *counter += 1;

        let cmp_result = format!("%decreases_strict_cmp_{}", *counter);
        *counter += 1;
        ir.push_str(&format!(
            "  {} = icmp slt i64 {}, {}\n",
            cmp_result, new_value, old_value
        ));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cmp_result, ok_label, fail_label
        ));

        // Failure block
        ir.push_str(&format!("{}:\n", fail_label));

        let fail_msg = format!(
            "decreases expression must strictly decrease on recursive call in '{}'",
            decreases_info.function_name
        );
        let msg_const = self.get_or_create_contract_string(&fail_msg);
        ir.push_str(&format!("  call i64 @__panic(i8* {})\n", msg_const));
        ir.push_str("  unreachable\n");

        // Success block
        ir.push_str(&format!("{}:\n", ok_label));

        Ok(ir)
    }

    /// Clear decreases info (called when leaving function scope)
    pub(crate) fn clear_decreases_info(&mut self) {
        self.current_decreases_info = None;
    }

    /// Check if current function has a decreases clause
    #[allow(dead_code)]
    pub(crate) fn has_decreases_clause(&self) -> bool {
        self.current_decreases_info.is_some()
    }

    /// Get the function name with decreases clause (if any)
    #[allow(dead_code)]
    pub(crate) fn get_decreases_function_name(&self) -> Option<String> {
        self.current_decreases_info
            .as_ref()
            .map(|info| info.function_name.clone())
    }
}
