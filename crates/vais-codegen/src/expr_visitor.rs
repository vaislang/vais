//! Expression Visitor implementation for CodeGenerator
//!
//! This module implements the ExprVisitor trait for CodeGenerator,
//! providing a clean separation of expression code generation logic.

use crate::visitor::{ExprVisitor, GenResult};
use crate::{CodeGenerator, CodegenError};
use vais_ast::{Spanned, Expr, Stmt, BinOp, UnaryOp, MatchArm, Param, Span, Type};
use vais_types::ResolvedType;

impl ExprVisitor for CodeGenerator {
    fn visit_expr(&mut self, expr: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        match &expr.node {
            Expr::Int(n) => self.visit_int(*n),
            Expr::Float(n) => self.visit_float(*n),
            Expr::Bool(b) => self.visit_bool(*b),
            Expr::String(s) => self.visit_string(s, counter),
            Expr::Unit => self.visit_unit(),
            Expr::Ident(name) => self.visit_ident(name, counter),
            Expr::SelfCall => self.visit_self_call(),
            Expr::Binary { op, left, right } => {
                self.visit_binary(op, left, right, counter, expr.span)
            }
            Expr::Unary { op, expr: inner } => {
                self.visit_unary(op, inner, counter, expr.span)
            }
            Expr::Ternary { cond, then, else_ } => {
                self.visit_ternary(cond, then, else_, counter)
            }
            Expr::Call { func, args } => {
                self.visit_call(func, args, counter, expr.span)
            }
            Expr::If { cond, then, else_ } => {
                self.visit_if(cond, then, else_.as_ref(), counter)
            }
            Expr::Loop { pattern: _, iter, body } => {
                self.visit_loop(iter.as_ref().map(|e| e.as_ref()), body, counter)
            }
            Expr::While { condition, body } => {
                self.visit_while(condition, body, counter)
            }
            Expr::Block(stmts) => self.visit_block(stmts, counter),
            Expr::Assign { target, value } => {
                self.visit_assign(target, value, counter)
            }
            Expr::AssignOp { op, target, value } => {
                self.visit_assign_op(op, target, value, counter)
            }
            Expr::Array(elements) => self.visit_array(elements, counter),
            Expr::Tuple(elements) => self.visit_tuple(elements, counter),
            Expr::StructLit { name, fields } => {
                self.visit_struct_lit(name, fields, counter)
            }
            Expr::Index { expr: array, index } => {
                // Check if this is a slice operation (index is a Range expression)
                if let Expr::Range { start, end, inclusive } = &index.node {
                    return self.generate_slice(array, start.as_deref(), end.as_deref(), *inclusive, counter);
                }
                self.visit_index(array, index, counter)
            }
            Expr::Field { expr: obj, field } => {
                self.visit_field(obj, field, counter)
            }
            Expr::MethodCall { receiver, method, args } => {
                self.visit_method_call(receiver, method, args, counter)
            }
            Expr::StaticMethodCall { type_name, method, args } => {
                self.visit_static_method_call(type_name, method, args, counter)
            }
            Expr::Ref(inner) => self.visit_ref(inner, counter),
            Expr::Deref(inner) => self.visit_deref(inner, counter),
            Expr::Cast { expr, ty } => self.visit_cast(expr, ty, counter),
            Expr::Match { expr: match_expr, arms } => {
                self.visit_match(match_expr, arms, counter)
            }
            Expr::Range { start, end, inclusive } => {
                self.visit_range(start.as_deref(), end.as_deref(), *inclusive, counter)
            }
            Expr::Await(inner) => self.visit_await(inner, counter),
            Expr::Spawn(inner) => self.visit_spawn(inner, counter),
            Expr::Lambda { params, body, captures: _ } => {
                self.visit_lambda(params, body, counter)
            }
            Expr::Try(inner) => self.visit_try(inner, counter),
            Expr::Unwrap(inner) => self.visit_unwrap(inner, counter),
            Expr::Comptime { body } => self.visit_comptime(body, counter),
            Expr::MacroInvoke(invoke) => self.visit_macro_invoke(invoke),
            Expr::Old(inner) => self.visit_old(inner, counter),
            Expr::Assert { condition, message } => {
                self.visit_assert(condition, message.as_deref(), counter)
            }
            Expr::Assume(inner) => self.visit_assume(inner, counter),
            Expr::Lazy(inner) => self.visit_lazy(inner, counter),
            Expr::Force(inner) => self.visit_force(inner, counter),
            Expr::Error { message, .. } => {
                // Error nodes should not reach codegen - they indicate parsing failures
                Err(CodegenError::Unsupported(format!(
                    "Parse error in expression: {}",
                    message
                )))
            }
        }
    }

    fn visit_int(&mut self, n: i64) -> GenResult {
        Ok((n.to_string(), String::new()))
    }

    fn visit_float(&mut self, n: f64) -> GenResult {
        Ok((format!("{:e}", n), String::new()))
    }

    fn visit_bool(&mut self, b: bool) -> GenResult {
        Ok((if b { "1" } else { "0" }.to_string(), String::new()))
    }

    fn visit_string(&mut self, s: &str, _counter: &mut usize) -> GenResult {
        let name = format!(".str.{}", self.string_counter);
        self.string_counter += 1;
        self.string_constants.push((name.clone(), s.to_string()));

        let len = s.len() + 1;
        Ok((
            format!("getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)", len, len, name),
            String::new(),
        ))
    }

    fn visit_unit(&mut self) -> GenResult {
        Ok(("void".to_string(), String::new()))
    }

    fn visit_ident(&mut self, name: &str, counter: &mut usize) -> GenResult {
        if let Some(local) = self.locals.get(name).cloned() {
            if local.is_param() {
                Ok((format!("%{}", local.llvm_name), String::new()))
            } else if matches!(local.ty, ResolvedType::Named { .. }) {
                let tmp = self.next_temp(counter);
                let llvm_ty = self.type_to_llvm(&local.ty);
                let ir = format!(
                    "  {} = load {}*, {}** %{}\n",
                    tmp, llvm_ty, llvm_ty, local.llvm_name
                );
                Ok((tmp, ir))
            } else {
                let tmp = self.next_temp(counter);
                let llvm_ty = self.type_to_llvm(&local.ty);
                let ir = format!(
                    "  {} = load {}, {}* %{}\n",
                    tmp, llvm_ty, llvm_ty, local.llvm_name
                );
                Ok((tmp, ir))
            }
        } else if name == "self" {
            Ok(("%self".to_string(), String::new()))
        } else if self.is_unit_enum_variant(name) {
            self.generate_unit_enum_variant(name, counter)
        } else {
            Ok((format!("@{}", name), String::new()))
        }
    }

    fn visit_self_call(&mut self) -> GenResult {
        if let Some(fn_name) = &self.current_function {
            Ok((format!("@{}", fn_name), String::new()))
        } else {
            Err(CodegenError::UndefinedFunction("@".to_string()))
        }
    }

    fn visit_binary(
        &mut self,
        op: &BinOp,
        left: &Spanned<Expr>,
        right: &Spanned<Expr>,
        counter: &mut usize,
        span: Span,
    ) -> GenResult {
        // Delegate to existing implementation
        self.generate_binary_expr(op, left, right, counter, span)
    }

    fn visit_unary(
        &mut self,
        op: &UnaryOp,
        expr: &Spanned<Expr>,
        counter: &mut usize,
        span: Span,
    ) -> GenResult {
        self.generate_unary_expr(op, expr, counter, span)
    }

    fn visit_ternary(
        &mut self,
        cond: &Spanned<Expr>,
        then: &Spanned<Expr>,
        else_: &Spanned<Expr>,
        counter: &mut usize,
    ) -> GenResult {
        self.generate_ternary_expr(cond, then, else_, counter)
    }

    fn visit_call(
        &mut self,
        func: &Spanned<Expr>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
        span: Span,
    ) -> GenResult {
        self.generate_call_expr(func, args, counter, span)
    }

    fn visit_if(
        &mut self,
        cond: &Spanned<Expr>,
        then: &[Spanned<Stmt>],
        else_: Option<&vais_ast::IfElse>,
        counter: &mut usize,
    ) -> GenResult {
        self.generate_if_expr(cond, then, else_, counter)
    }

    fn visit_loop(
        &mut self,
        iter: Option<&Spanned<Expr>>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> GenResult {
        self.generate_loop_expr(iter, body, counter)
    }

    fn visit_while(
        &mut self,
        condition: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> GenResult {
        self.generate_while_expr(condition, body, counter)
    }

    fn visit_block(&mut self, stmts: &[Spanned<Stmt>], counter: &mut usize) -> GenResult {
        let (val, ir, _terminated) = self.generate_block_stmts(stmts, counter)?;
        Ok((val, ir))
    }

    fn visit_assign(
        &mut self,
        target: &Spanned<Expr>,
        value: &Spanned<Expr>,
        counter: &mut usize,
    ) -> GenResult {
        self.generate_assign_expr(target, value, counter)
    }

    fn visit_assign_op(
        &mut self,
        op: &BinOp,
        target: &Spanned<Expr>,
        value: &Spanned<Expr>,
        counter: &mut usize,
    ) -> GenResult {
        self.generate_assign_op_expr(op, target, value, counter)
    }

    fn visit_array(&mut self, elements: &[Spanned<Expr>], counter: &mut usize) -> GenResult {
        self.generate_array_expr(elements, counter)
    }

    fn visit_tuple(&mut self, elements: &[Spanned<Expr>], counter: &mut usize) -> GenResult {
        self.generate_tuple_expr(elements, counter)
    }

    fn visit_struct_lit(
        &mut self,
        name: &Spanned<String>,
        fields: &[(Spanned<String>, Spanned<Expr>)],
        counter: &mut usize,
    ) -> GenResult {
        self.generate_struct_lit_expr(name, fields, counter)
    }

    fn visit_index(
        &mut self,
        array: &Spanned<Expr>,
        index: &Spanned<Expr>,
        counter: &mut usize,
    ) -> GenResult {
        self.generate_index_expr(array, index, counter)
    }

    fn visit_field(
        &mut self,
        obj: &Spanned<Expr>,
        field: &Spanned<String>,
        counter: &mut usize,
    ) -> GenResult {
        self.generate_field_expr(obj, field, counter)
    }

    fn visit_method_call(
        &mut self,
        receiver: &Spanned<Expr>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> GenResult {
        self.generate_method_call_expr(receiver, method, args, counter)
    }

    fn visit_static_method_call(
        &mut self,
        type_name: &Spanned<String>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> GenResult {
        self.generate_static_method_call_expr(type_name, method, args, counter)
    }

    fn visit_ref(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        if let Expr::Ident(name) = &inner.node {
            if self.locals.contains_key(name.as_str()) {
                return Ok((format!("%{}", name), String::new()));
            }
        }
        // For complex expressions, evaluate and return
        self.visit_expr(inner, counter)
    }

    fn visit_deref(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        let (ptr_val, ptr_ir) = self.visit_expr(inner, counter)?;
        let mut ir = ptr_ir;

        let result = self.next_temp(counter);
        ir.push_str(&format!("  {} = load i64, i64* {}\n", result, ptr_val));

        Ok((result, ir))
    }

    fn visit_cast(
        &mut self,
        expr: &Spanned<Expr>,
        ty: &Spanned<Type>,
        counter: &mut usize,
    ) -> GenResult {
        self.generate_cast_expr(expr, ty, counter)
    }

    fn visit_match(
        &mut self,
        expr: &Spanned<Expr>,
        arms: &[MatchArm],
        counter: &mut usize,
    ) -> GenResult {
        self.generate_match(expr, arms, counter)
    }

    fn visit_range(
        &mut self,
        start: Option<&Spanned<Expr>>,
        _end: Option<&Spanned<Expr>>,
        _inclusive: bool,
        counter: &mut usize,
    ) -> GenResult {
        if let Some(start_expr) = start {
            self.visit_expr(start_expr, counter)
        } else {
            Ok(("0".to_string(), String::new()))
        }
    }

    fn visit_await(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        self.generate_await_expr(inner, counter)
    }

    fn visit_spawn(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        let (future_ptr, future_ir) = self.visit_expr(inner, counter)?;
        let mut ir = future_ir;
        ir.push_str(&format!("; Spawned task at {}\n", future_ptr));
        Ok((future_ptr, ir))
    }

    fn visit_lambda(
        &mut self,
        params: &[Param],
        body: &Spanned<Expr>,
        counter: &mut usize,
    ) -> GenResult {
        self.generate_lambda_expr(params, body, counter)
    }

    fn visit_try(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        self.generate_try_expr(inner, counter)
    }

    fn visit_unwrap(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        self.generate_unwrap_expr(inner, counter)
    }

    fn visit_comptime(&mut self, body: &Spanned<Expr>, _counter: &mut usize) -> GenResult {
        // Evaluate the comptime expression at compile time
        let mut evaluator = vais_types::ComptimeEvaluator::new();
        let value = evaluator.eval(body).map_err(|e| {
            CodegenError::TypeError(format!("Comptime evaluation failed: {}", e))
        })?;

        // Return the evaluated constant as LLVM IR
        match value {
            vais_types::ComptimeValue::Int(n) => Ok((n.to_string(), String::new())),
            vais_types::ComptimeValue::Float(f) => Ok((format!("{:e}", f), String::new())),
            vais_types::ComptimeValue::Bool(b) => Ok((if b { "1" } else { "0" }.to_string(), String::new())),
            vais_types::ComptimeValue::Unit => Ok(("void".to_string(), String::new())),
        }
    }

    fn visit_macro_invoke(&mut self, invoke: &vais_ast::MacroInvoke) -> GenResult {
        // Macro invocations should be expanded before codegen reaches this point.
        // If we get here, it means the macro was not expanded - this is a compiler bug.
        Err(crate::CodegenError::TypeError(format!(
            "Unexpanded macro invocation: {}! - macros must be expanded before code generation",
            invoke.name.node
        )))
    }

    fn visit_old(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        // old(expr) - Reference to pre-state value in ensures clause
        // Look up the snapshot storage or fallback to current evaluation
        let old_var_name = format!("__old_{}", *counter);
        *counter += 1;

        if let Some(snapshot_var) = self.old_snapshots.get(&old_var_name) {
            let ty = self.infer_expr_type(inner);
            let llvm_ty = self.type_to_llvm(&ty);
            let result = self.next_temp(counter);
            let ir = format!(
                "  {} = load {}, {}* %{}\n",
                result, llvm_ty, llvm_ty, snapshot_var
            );
            Ok((result, ir))
        } else {
            // Fallback: evaluate expression normally (when not in ensures context)
            self.visit_expr(inner, counter)
        }
    }

    fn visit_assert(
        &mut self,
        condition: &Spanned<Expr>,
        message: Option<&Spanned<Expr>>,
        counter: &mut usize,
    ) -> GenResult {
        self.generate_assert(condition, message, counter)
    }

    fn visit_assume(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        if self.release_mode {
            // In release mode, assume is a no-op
            Ok(("0".to_string(), String::new()))
        } else {
            self.generate_assume(inner, counter)
        }
    }

    fn visit_lazy(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        // Lazy evaluation: Create a thunk struct { i1 computed, T value, i8* thunk_fn }
        // For simplicity, we evaluate the expression immediately but wrap it in a Lazy struct.
        // A full implementation would create a closure and defer evaluation.

        let (inner_val, inner_ir) = self.visit_expr(inner, counter)?;
        let mut ir = inner_ir;

        // Infer the inner type to get the correct LLVM type
        let inner_type = self.infer_expr_type(inner);
        let inner_llvm_ty = self.type_to_llvm(&inner_type);
        let lazy_ty = format!("{{ i1, {}, i8* }}", inner_llvm_ty);

        // Allocate the lazy struct
        let lazy_ptr = self.next_temp(counter);
        ir.push_str(&format!("  {} = alloca {}\n", lazy_ptr, lazy_ty));

        // Store computed = true (eager evaluation for now)
        let computed_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {}, {}* {}, i32 0, i32 0\n",
            computed_ptr, lazy_ty, lazy_ty, lazy_ptr
        ));
        ir.push_str(&format!("  store i1 1, i1* {}\n", computed_ptr));

        // Store the computed value
        let value_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {}, {}* {}, i32 0, i32 1\n",
            value_ptr, lazy_ty, lazy_ty, lazy_ptr
        ));
        ir.push_str(&format!("  store {} {}, {}* {}\n", inner_llvm_ty, inner_val, inner_llvm_ty, value_ptr));

        // Store thunk pointer as null (not needed for eager evaluation)
        let thunk_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {}, {}* {}, i32 0, i32 2\n",
            thunk_ptr, lazy_ty, lazy_ty, lazy_ptr
        ));
        ir.push_str(&format!("  store i8* null, i8** {}\n", thunk_ptr));

        // Load and return the lazy struct
        let result = self.next_temp(counter);
        ir.push_str(&format!("  {} = load {}, {}* {}\n", result, lazy_ty, lazy_ty, lazy_ptr));

        Ok((result, ir))
    }

    fn visit_force(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        // Force evaluation: Extract value from Lazy struct
        // If computed flag is true, return cached value
        // Otherwise, call thunk and cache result (not implemented for eager mode)

        // Infer the lazy type to extract the inner type
        let lazy_type = self.infer_expr_type(inner);
        let inner_type = match &lazy_type {
            ResolvedType::Lazy(inner) => inner.as_ref().clone(),
            other => other.clone(), // If not lazy, just return the value
        };

        // If the inner expression is not a Lazy type, just return the value directly
        if !matches!(lazy_type, ResolvedType::Lazy(_)) {
            return self.visit_expr(inner, counter);
        }

        let inner_llvm_ty = self.type_to_llvm(&inner_type);

        // For lazy values, we need to handle the case where the expression is a variable
        // The variable holds a Lazy struct, and we need to extract the value field

        // Check if the inner expression is an identifier (variable reference)
        if let Expr::Ident(name) = &inner.node {
            if let Some(local) = self.locals.get(name.as_str()).cloned() {
                let lazy_ty = format!("{{ i1, {}, i8* }}", inner_llvm_ty);
                let mut ir = String::new();

                if local.is_ssa() {
                    // For SSA variables, the value is already the struct itself
                    // Use extractvalue to get the value field (index 1)
                    let result = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = extractvalue {} {}, 1\n",
                        result, lazy_ty, local.llvm_name
                    ));
                    return Ok((result, ir));
                } else {
                    // For alloca variables, get pointer to value field (index 1)
                    let value_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* %{}, i32 0, i32 1\n",
                        value_ptr, lazy_ty, lazy_ty, local.llvm_name
                    ));

                    let result = self.next_temp(counter);
                    ir.push_str(&format!("  {} = load {}, {}* {}\n", result, inner_llvm_ty, inner_llvm_ty, value_ptr));

                    return Ok((result, ir));
                }
            }
        }

        // For other expressions that produce lazy values, use extractvalue
        let (lazy_val, lazy_ir) = self.visit_expr(inner, counter)?;
        let mut ir = lazy_ir;

        // Use extractvalue to get the value field (index 1) from the struct
        let result = self.next_temp(counter);
        ir.push_str(&format!("  {} = extractvalue {{ i1, {}, i8* }} {}, 1\n", result, inner_llvm_ty, lazy_val));

        Ok((result, ir))
    }
}
