//! Expression Visitor implementation for CodeGenerator
//!
//! This module implements the ExprVisitor trait for CodeGenerator,
//! providing a clean separation of expression code generation logic.

use crate::visitor::{ExprVisitor, GenResult};
use crate::{CodeGenerator, CodegenError};
use vais_ast::{BinOp, Expr, MatchArm, Param, Span, Spanned, Stmt, Type, UnaryOp};
use vais_types::ResolvedType;

impl ExprVisitor for CodeGenerator {
    fn visit_expr(&mut self, expr: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        match &expr.node {
            Expr::Int(n) => self.visit_int(*n),
            Expr::Float(n) => self.visit_float(*n),
            Expr::Bool(b) => self.visit_bool(*b),
            Expr::String(s) => self.visit_string(s, counter),
            Expr::StringInterp(_) => {
                // Delegate to main generate_expr which handles desugaring
                self.generate_expr(expr, counter)
            }
            Expr::Unit => self.visit_unit(),
            Expr::Ident(name) => self.visit_ident(name, counter),
            Expr::SelfCall => self.visit_self_call(),
            Expr::Binary { op, left, right } => {
                self.visit_binary(op, left, right, counter, expr.span)
            }
            Expr::Unary { op, expr: inner } => self.visit_unary(op, inner, counter, expr.span),
            Expr::Ternary { cond, then, else_ } => self.visit_ternary(cond, then, else_, counter),
            Expr::Call { func, args } => self.visit_call(func, args, counter, expr.span),
            Expr::If { cond, then, else_ } => self.visit_if(cond, then, else_.as_ref(), counter),
            Expr::Loop {
                pattern: _,
                iter,
                body,
            } => self.visit_loop(iter.as_ref().map(|e| e.as_ref()), body, counter),
            Expr::While { condition, body } => self.visit_while(condition, body, counter),
            Expr::Block(stmts) => self.visit_block(stmts, counter),
            Expr::Assign { target, value } => self.visit_assign(target, value, counter),
            Expr::AssignOp { op, target, value } => {
                self.visit_assign_op(op, target, value, counter)
            }
            Expr::Array(elements) => self.visit_array(elements, counter),
            Expr::MapLit(_) => {
                // Delegate to main generate_expr for map literals
                self.generate_expr(expr, counter)
            }
            Expr::Tuple(elements) => self.visit_tuple(elements, counter),
            Expr::StructLit { name, fields } => self.visit_struct_lit(name, fields, counter),
            Expr::Index { expr: array, index } => {
                // Check if this is a slice operation (index is a Range expression)
                if let Expr::Range {
                    start,
                    end,
                    inclusive,
                } = &index.node
                {
                    return self.generate_slice(
                        array,
                        start.as_deref(),
                        end.as_deref(),
                        *inclusive,
                        counter,
                    );
                }
                self.visit_index(array, index, counter)
            }
            Expr::Field { expr: obj, field } => self.visit_field(obj, field, counter),
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => self.visit_method_call(receiver, method, args, counter),
            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => self.visit_static_method_call(type_name, method, args, counter),
            Expr::Spread(inner) => {
                // Spread is handled within array generation; standalone spread generates inner
                self.generate_expr(inner, counter)
            }
            Expr::Ref(inner) => self.visit_ref(inner, counter),
            Expr::Deref(inner) => self.visit_deref(inner, counter),
            Expr::Cast { expr, ty } => self.visit_cast(expr, ty, counter),
            Expr::Match {
                expr: match_expr,
                arms,
            } => self.visit_match(match_expr, arms, counter),
            Expr::Range {
                start,
                end,
                inclusive,
            } => self.visit_range(start.as_deref(), end.as_deref(), *inclusive, counter),
            Expr::Await(inner) => self.visit_await(inner, counter),
            Expr::Spawn(inner) => self.visit_spawn(inner, counter),
            Expr::Yield(inner) => self.visit_expr(inner, counter),
            Expr::Lambda { params, body, .. } => self.visit_lambda(params, body, counter),
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
        Ok((crate::types::format_llvm_float(n), String::new()))
    }

    fn visit_bool(&mut self, b: bool) -> GenResult {
        Ok((if b { "1" } else { "0" }.to_string(), String::new()))
    }

    fn visit_string(&mut self, s: &str, _counter: &mut usize) -> GenResult {
        let name = self.make_string_name();
        self.strings.counter += 1;
        self.strings.constants.push((name.clone(), s.to_string()));

        let len = s.len() + 1;
        Ok((
            format!(
                "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                len, len, name
            ),
            String::new(),
        ))
    }

    fn visit_unit(&mut self) -> GenResult {
        Ok(("void".to_string(), String::new()))
    }

    fn visit_ident(&mut self, name: &str, counter: &mut usize) -> GenResult {
        if let Some(local) = self.fn_ctx.locals.get(name).cloned() {
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
        if let Some(fn_name) = &self.fn_ctx.current_function {
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
            if let Some(local) = self.fn_ctx.locals.get(name.as_str()).cloned() {
                if local.is_alloca() {
                    // Alloca variables already have an address
                    return Ok((format!("%{}", local.llvm_name), String::new()));
                } else {
                    // SSA/Param values need to be spilled to stack to take their address
                    let mut ir = String::new();
                    let llvm_ty = self.type_to_llvm(&local.ty);
                    let (val, val_ir) = self.visit_expr(inner, counter)?;
                    ir.push_str(&val_ir);
                    let tmp_alloca = self.next_temp(counter);
                    ir.push_str(&format!("  {} = alloca {}\n", tmp_alloca, llvm_ty));
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        llvm_ty, val, llvm_ty, tmp_alloca
                    ));
                    return Ok((tmp_alloca, ir));
                }
            }
        }
        // For complex expressions, evaluate and return
        self.visit_expr(inner, counter)
    }

    fn visit_deref(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        let (ptr_val, ptr_ir) = self.visit_expr(inner, counter)?;
        let mut ir = ptr_ir;

        // Infer the pointed-to type to generate correct load instruction
        let deref_ty = self.infer_expr_type(inner);
        let llvm_ty = match deref_ty {
            vais_types::ResolvedType::Pointer(ref inner_ty) => self.type_to_llvm(inner_ty),
            vais_types::ResolvedType::Ref(ref inner_ty) => self.type_to_llvm(inner_ty),
            vais_types::ResolvedType::RefMut(ref inner_ty) => self.type_to_llvm(inner_ty),
            _ => "i64".to_string(),
        };

        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load {}, {}* {}\n",
            result, llvm_ty, llvm_ty, ptr_val
        ));

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

    fn visit_comptime(&mut self, body: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        // Evaluate the comptime expression at compile time
        let mut evaluator = vais_types::ComptimeEvaluator::new();
        let value = evaluator
            .eval(body)
            .map_err(|e| CodegenError::TypeError(format!("Comptime evaluation failed: {}", e)))?;

        // Return the evaluated constant as LLVM IR
        match value {
            vais_types::ComptimeValue::Int(n) => Ok((n.to_string(), String::new())),
            vais_types::ComptimeValue::Float(f) => {
                Ok((crate::types::format_llvm_float(f), String::new()))
            }
            vais_types::ComptimeValue::Bool(b) => {
                Ok((if b { "1" } else { "0" }.to_string(), String::new()))
            }
            vais_types::ComptimeValue::String(s) => {
                // Create a global string constant
                let name = self.make_string_name();
                self.strings.counter += 1;
                self.strings.constants.push((name.clone(), s.clone()));
                let len = s.len() + 1;
                Ok((
                    format!(
                        "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                        len, len, name
                    ),
                    String::new(),
                ))
            }
            vais_types::ComptimeValue::Array(arr) => {
                // Generate array literal from comptime array
                let mut elements = Vec::new();
                let mut ir = String::new();

                for elem in arr {
                    match elem {
                        vais_types::ComptimeValue::Int(n) => elements.push(n.to_string()),
                        vais_types::ComptimeValue::Float(f) => {
                            elements.push(crate::types::format_llvm_float(f))
                        }
                        vais_types::ComptimeValue::Bool(b) => {
                            elements.push(if b { "1" } else { "0" }.to_string())
                        }
                        _ => {
                            return Err(CodegenError::TypeError(
                                "Comptime arrays can only contain simple values (int, float, bool)"
                                    .to_string(),
                            ));
                        }
                    }
                }

                // Create array on the stack
                let array_name = format!("%comptime_array_{}", counter);
                *counter += 1;
                let len = elements.len();

                // For now, assume i64 elements (we'd need better type inference for mixed types)
                ir.push_str(&format!("  {} = alloca [{} x i64]\n", array_name, len));

                for (i, elem_val) in elements.iter().enumerate() {
                    let elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr [{} x i64], [{} x i64]* {}, i64 0, i64 {}\n",
                        elem_ptr, len, len, array_name, i
                    ));
                    ir.push_str(&format!("  store i64 {}, i64* {}\n", elem_val, elem_ptr));
                }

                Ok((array_name, ir))
            }
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

        if let Some(snapshot_var) = self.contracts.old_snapshots.get(&old_var_name) {
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
        // Lazy evaluation: Create a thunk function that captures free variables
        // and returns { i1 computed=false, T zeroinit, i8* thunk_fn_ptr }
        // The thunk is called on first `force`, which caches the result.

        let inner_type = self.infer_expr_type(inner);
        let inner_llvm_ty = self.type_to_llvm(&inner_type);
        let lazy_ty = format!("{{ i1, {}, i8* }}", inner_llvm_ty);

        // Generate a thunk function that computes the lazy value
        let thunk_name = format!("__lazy_thunk_{}", self.fn_ctx.label_counter);
        self.fn_ctx.label_counter += 1;

        // Find free variables in the lazy expression (for capture)
        // param_names is empty because lazy has no parameters of its own —
        // all identifiers from the outer scope (including function params) are free variables
        // that need to be captured by the thunk function.
        let param_names = std::collections::HashSet::new();
        let mut free_vars = Vec::new();
        self.collect_free_vars_in_expr(&inner.node, &param_names, &mut free_vars);
        let mut seen = std::collections::HashSet::new();
        free_vars.retain(|v| seen.insert(v.clone()));

        // Collect captured variable info
        let mut captured_vars: Vec<(String, crate::ResolvedType, String)> = Vec::new();
        let mut capture_ir = String::new();

        for cap_name in &free_vars {
            if let Some(local) = self.fn_ctx.locals.get(cap_name) {
                let ty = local.ty.clone();
                let llvm_ty = self.type_to_llvm(&ty);
                if local.is_param() {
                    captured_vars.push((cap_name.clone(), ty, format!("%{}", local.llvm_name)));
                } else if local.is_ssa() {
                    captured_vars.push((cap_name.clone(), ty, local.llvm_name.clone()));
                } else {
                    let tmp = self.next_temp(counter);
                    capture_ir.push_str(&format!(
                        "  {} = load {}, {}* %{}\n",
                        tmp, llvm_ty, llvm_ty, local.llvm_name
                    ));
                    captured_vars.push((cap_name.clone(), ty, tmp));
                }
            }
        }

        // Build thunk parameter list (captured vars only — thunk takes no explicit args)
        let mut thunk_param_strs = Vec::new();
        let mut thunk_param_types = Vec::new();
        for (cap_name, cap_ty, _) in &captured_vars {
            let llvm_ty = self.type_to_llvm(cap_ty);
            thunk_param_strs.push(format!("{} %__cap_{}", llvm_ty, cap_name));
            thunk_param_types.push(llvm_ty);
        }

        // Save and set up thunk context
        let saved_function = self.fn_ctx.current_function.take();
        let saved_locals = std::mem::take(&mut self.fn_ctx.locals);

        self.fn_ctx.current_function = Some(thunk_name.clone());

        // Register captured variables as locals in thunk
        for (cap_name, cap_ty, _) in &captured_vars {
            self.fn_ctx.locals.insert(
                cap_name.clone(),
                crate::types::LocalVar::param(cap_ty.clone(), format!("__cap_{}", cap_name)),
            );
        }

        // Generate thunk body
        let mut thunk_counter = 0;
        let (body_val, body_ir) = self.generate_expr(inner, &mut thunk_counter)?;

        // Build thunk function IR (returns the inner type)
        let ret_ty = &inner_llvm_ty;
        let mut thunk_ir = format!(
            "define {} @{}({}) {{\nentry:\n",
            ret_ty,
            thunk_name,
            thunk_param_strs.join(", ")
        );
        thunk_ir.push_str(&body_ir);
        thunk_ir.push_str(&format!("  ret {} {}\n}}\n", ret_ty, body_val));

        self.lambdas.generated_ir.push(thunk_ir);

        // Restore context
        self.fn_ctx.current_function = saved_function;
        self.fn_ctx.locals = saved_locals;

        // Build the lazy struct: { i1 false, T zeroinitializer, i8* thunk_ptr }
        let mut ir = capture_ir;

        // Allocate the lazy struct
        let lazy_ptr = self.next_temp(counter);
        ir.push_str(&format!("  {} = alloca {}\n", lazy_ptr, lazy_ty));

        // Store computed = false
        let computed_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {}, {}* {}, i32 0, i32 0\n",
            computed_ptr, lazy_ty, lazy_ty, lazy_ptr
        ));
        ir.push_str(&format!("  store i1 0, i1* {}\n", computed_ptr));

        // Store zero-initialized value (will be filled on first force)
        let value_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {}, {}* {}, i32 0, i32 1\n",
            value_ptr, lazy_ty, lazy_ty, lazy_ptr
        ));
        ir.push_str(&format!(
            "  store {} 0, {}* {}\n",
            inner_llvm_ty, inner_llvm_ty, value_ptr
        ));

        // Store thunk function pointer (cast to i8*)
        let thunk_fn_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = bitcast {} ({})* @{} to i8*\n",
            thunk_fn_ptr,
            ret_ty,
            thunk_param_types.join(", "),
            thunk_name
        ));
        let thunk_slot = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {}, {}* {}, i32 0, i32 2\n",
            thunk_slot, lazy_ty, lazy_ty, lazy_ptr
        ));
        ir.push_str(&format!(
            "  store i8* {}, i8** {}\n",
            thunk_fn_ptr, thunk_slot
        ));

        // Load and return the lazy struct
        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load {}, {}* {}\n",
            result, lazy_ty, lazy_ty, lazy_ptr
        ));

        // Store lazy thunk info for force to use at call sites
        self.lambdas.last_lazy_info = Some(crate::types::LazyThunkInfo {
            thunk_name: thunk_name.clone(),
            captures: captured_vars
                .iter()
                .map(|(name, ty, val)| (name.clone(), self.type_to_llvm(ty), val.clone()))
                .collect(),
            inner_llvm_ty: inner_llvm_ty.clone(),
        });

        Ok((result, ir))
    }

    fn visit_force(&mut self, inner: &Spanned<Expr>, counter: &mut usize) -> GenResult {
        // Force evaluation: Check computed flag, call thunk if needed, cache result

        let lazy_type = self.infer_expr_type(inner);
        let inner_type = match &lazy_type {
            ResolvedType::Lazy(inner) => inner.as_ref().clone(),
            other => other.clone(),
        };

        // If not a Lazy type, just return the value directly
        if !matches!(lazy_type, ResolvedType::Lazy(_)) {
            return self.visit_expr(inner, counter);
        }

        let inner_llvm_ty = self.type_to_llvm(&inner_type);
        let lazy_ty = format!("{{ i1, {}, i8* }}", inner_llvm_ty);

        // Look up lazy thunk info if the inner is an identifier
        let lazy_info = if let Expr::Ident(name) = &inner.node {
            self.lambdas.lazy_bindings.get(name.as_str()).cloned()
        } else {
            None
        };

        // If we have thunk info, generate proper conditional evaluation
        if let (Expr::Ident(name), Some(info)) = (&inner.node, &lazy_info) {
            if let Some(local) = self.fn_ctx.locals.get(name.as_str()).cloned() {
                let mut ir = String::new();

                // The lazy value must be in an alloca so we can update computed/value fields
                // If SSA, we need to spill it first
                let lazy_alloca = if local.is_ssa() {
                    let alloca = self.next_temp(counter);
                    ir.push_str(&format!("  {} = alloca {}\n", alloca, lazy_ty));
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        lazy_ty, local.llvm_name, lazy_ty, alloca
                    ));
                    alloca
                } else {
                    format!("%{}", local.llvm_name)
                };

                // Load computed flag
                let computed_ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr {}, {}* {}, i32 0, i32 0\n",
                    computed_ptr, lazy_ty, lazy_ty, lazy_alloca
                ));
                let computed = self.next_temp(counter);
                ir.push_str(&format!("  {} = load i1, i1* {}\n", computed, computed_ptr));

                // Branch on computed flag
                let label_id = self.fn_ctx.label_counter;
                self.fn_ctx.label_counter += 3;
                let cached_label = format!("lazy.cached.{}", label_id);
                let compute_label = format!("lazy.compute.{}", label_id + 1);
                let merge_label = format!("lazy.merge.{}", label_id + 2);

                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    computed, cached_label, compute_label
                ));

                // Cached path: load value from struct
                ir.push_str(&format!("{}:\n", cached_label));
                let cached_val_ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr {}, {}* {}, i32 0, i32 1\n",
                    cached_val_ptr, lazy_ty, lazy_ty, lazy_alloca
                ));
                let cached_val = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    cached_val, inner_llvm_ty, inner_llvm_ty, cached_val_ptr
                ));
                ir.push_str(&format!("  br label %{}\n", merge_label));

                // Compute path: call thunk, store result, set computed=true
                ir.push_str(&format!("{}:\n", compute_label));

                // Build thunk call args (captured values)
                let mut thunk_args = Vec::new();
                for (cap_name, cap_llvm_ty_stored, cap_val) in &info.captures {
                    // Re-load captured value from current scope (it may have changed)
                    if let Some(cap_local) = self.fn_ctx.locals.get(cap_name.as_str()) {
                        let cap_llvm_ty = self.type_to_llvm(&cap_local.ty);
                        if cap_local.is_param() {
                            thunk_args.push(format!("{} %{}", cap_llvm_ty, cap_local.llvm_name));
                        } else if cap_local.is_ssa() {
                            thunk_args.push(format!("{} {}", cap_llvm_ty, cap_local.llvm_name));
                        } else {
                            let tmp = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = load {}, {}* %{}\n",
                                tmp, cap_llvm_ty, cap_llvm_ty, cap_local.llvm_name
                            ));
                            thunk_args.push(format!("{} {}", cap_llvm_ty, tmp));
                        }
                    } else {
                        // Fallback: use the captured value and stored type from thunk creation time
                        thunk_args.push(format!("{} {}", cap_llvm_ty_stored, cap_val));
                    }
                }

                let computed_val = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call {} @{}({})\n",
                    computed_val,
                    inner_llvm_ty,
                    info.thunk_name,
                    thunk_args.join(", ")
                ));

                // Store computed value into lazy struct
                let store_val_ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr {}, {}* {}, i32 0, i32 1\n",
                    store_val_ptr, lazy_ty, lazy_ty, lazy_alloca
                ));
                ir.push_str(&format!(
                    "  store {} {}, {}* {}\n",
                    inner_llvm_ty, computed_val, inner_llvm_ty, store_val_ptr
                ));

                // Set computed = true
                ir.push_str(&format!("  store i1 1, i1* {}\n", computed_ptr));

                ir.push_str(&format!("  br label %{}\n", merge_label));

                // Merge: phi node
                ir.push_str(&format!("{}:\n", merge_label));
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = phi {} [{}, %{}], [{}, %{}]\n",
                    result, inner_llvm_ty, cached_val, cached_label, computed_val, compute_label
                ));

                return Ok((result, ir));
            }
        }

        // Fallback: for non-identifier or no thunk info, use simple extractvalue
        let (lazy_val, lazy_ir) = self.visit_expr(inner, counter)?;
        let mut ir = lazy_ir;

        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = extractvalue {} {}, 1\n",
            result, lazy_ty, lazy_val
        ));

        Ok((result, ir))
    }
}
