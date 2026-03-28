//! Expression code generation for LLVM IR.
//!
//! This module implements the main expression dispatcher (`generate_expr`) which routes
//! each expression type to the appropriate code generation method.
//!
//! # Submodules
//!
//! - `special` — Spawn, Comptime, Range (extracted from inline match arms)
//! - `loops` — Loop, While (extracted for stack frame isolation)
//! - `ref_deref` — Ref, Deref (extracted for stack frame isolation)
//! - `map_lit` — MapLit (extracted for stack frame isolation)
//! - `string_lit` — String, StringInterp (extracted for stack frame isolation)
//! - `misc_expr` — Old, Assume, Yield, EnumAccess (extracted for stack frame isolation)
//!
//! Most expression types delegate to pre-existing helper modules:
//! - `expr_helpers` — Binary, Unary, Ident, Assign, AssignOp, Cast
//! - `expr_helpers_control` — Ternary, If
//! - `expr_helpers_data` — Array, Tuple, Index, Field
//! - `expr_helpers_misc` — Lambda, Try, Unwrap, Await
//! - `expr_helpers_call/method_call` — MethodCall, StaticMethodCall
//! - `generate_expr_call` — Call
//! - `generate_expr_struct` — StructLit
//! - `generate_expr_loop` — RangeForLoop
//! - `control_flow/match_gen` — Match
//! - `contracts/assert_assume` — Assert, Assume
//! - `string_ops` — String methods
//! - `helpers` — Slice

use vais_ast::*;

use crate::{CodeGenerator, CodegenError, CodegenResult};

mod loops;
mod map_lit;
mod misc_expr;
mod ref_deref;
mod special;
mod string_lit;

impl CodeGenerator {
    pub(crate) fn generate_expr(
        &mut self,
        expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Use stacker to grow the stack on demand, preventing stack overflow
        // for deeply nested expressions (e.g., complex struct specializations)
        stacker::maybe_grow(4 * 1024 * 1024, 16 * 1024 * 1024, || {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
            let count = CALL_COUNT.fetch_add(1, Ordering::Relaxed);
            if count > 100000 {
                std::process::abort();
            }
            self.generate_expr_inner(expr, counter)
        })
    }

    #[inline(never)]
    fn generate_expr_inner(
        &mut self,
        expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Track the current expression span for error diagnostics.
        // When an error propagates up, the driver can read `last_error_span`
        // to construct a SpannedCodegenError with source location.
        self.last_error_span = Some(expr.span);

        // Infer the resolved type BEFORE code generation (read-only).
        // Used to populate temp_var_types for named temporaries produced below.
        let inferred_type = self.infer_expr_type(expr);

        let result = match &expr.node {
            Expr::Int(n) => Ok((n.to_string(), String::new())),
            Expr::Float(n) => Ok((crate::types::format_llvm_float(*n), String::new())),
            Expr::Bool(b) => Ok((if *b { "1" } else { "0" }.to_string(), String::new())),
            Expr::String(s) => self.generate_string_literal_expr(s, counter),
            Expr::StringInterp(parts) => self.generate_string_interp_expr(parts, expr.span, counter),
            Expr::Unit => Ok(("void".to_string(), String::new())),

            Expr::Ident(name) => self.generate_ident_expr(name, counter),

            Expr::SelfCall => {
                if let Some(fn_name) = &self.fn_ctx.current_function {
                    Ok((format!("@{}", fn_name), String::new()))
                } else {
                    Err(CodegenError::UndefinedFunction("@".to_string()))
                }
            }

            Expr::Binary { op, left, right } => {
                self.generate_binary_expr(op, left, right, counter, expr.span)
            }

            Expr::Unary { op, expr: inner } => {
                self.generate_unary_expr(op, inner, counter, expr.span)
            }

            Expr::Ternary { cond, then, else_ } => {
                self.generate_ternary_expr(cond, then, else_, counter)
            }

            Expr::Call { func, args } => self.generate_expr_call(func, args, expr.span, counter),

            // If/Else expression with basic blocks
            Expr::If { cond, then, else_ } => {
                self.generate_if_expr(cond, then, else_.as_ref(), counter)
            }

            // Loop expression
            Expr::Loop {
                pattern,
                iter,
                body,
            } => self.generate_loop_with_pattern(pattern.as_ref(), iter.as_ref(), body, counter),

            // While loop expression
            Expr::While { condition, body } => {
                self.generate_while_loop_expr(condition, body, counter)
            }

            // Block expression
            Expr::Block(stmts) => {
                let (val, ir, _terminated) = self.generate_block_stmts(stmts, counter)?;
                Ok((val, ir))
            }

            // Assignment expression
            Expr::Assign { target, value } => self.generate_assign_expr(target, value, counter),

            // Compound assignment (+=, -=, etc.)
            Expr::AssignOp { op, target, value } => {
                self.generate_assign_op_expr(op, target, value, counter)
            }

            // Array literal: [a, b, c]
            Expr::Array(elements) => self.generate_array_expr(elements, counter),

            // Map literal: {k: v, ...}
            Expr::MapLit(pairs) => self.generate_map_lit_expr(pairs, counter),

            // Tuple literal: (a, b, c)
            Expr::Tuple(elements) => self.generate_tuple_expr(elements, counter),

            // Struct literal: Point{x:1, y:2} or enum variant: Shape.Circle{radius:5.0}
            Expr::StructLit { name, fields, enum_name } => {
                if let Some(ref ename) = enum_name {
                    // Enum struct variant — generate as enum variant constructor
                    let variant = &name.node;
                    self.generate_enum_variant_struct(ename, variant, fields, counter)
                } else {
                    self.generate_expr_struct_lit(name, fields, counter)
                }
            }

            // Index: arr[idx] or slice: arr[start..end]
            Expr::Index {
                expr: array_expr,
                index,
            } => self.generate_index_expr(array_expr, index, counter),

            // Field access: obj.field
            Expr::Field {
                expr: obj_expr,
                field,
            } => self.generate_field_expr(obj_expr, field, counter),

            // Method call: obj.method(args)
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => self.generate_method_call_expr(receiver, method, args, counter),

            // Static method call: Type.method(args)
            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => self.generate_static_method_call_expr(type_name, method, args, counter),

            // Spread: ..expr (handled within array generation; standalone generates inner)
            Expr::Spread(inner) => self.generate_expr(inner, counter),

            // Reference: &expr
            Expr::Ref(inner) => self.generate_ref_expr(inner, counter),

            // Dereference: *expr
            Expr::Deref(inner) => self.generate_deref_expr(inner, counter),

            // Type cast: expr as Type
            Expr::Cast { expr, ty } => self.generate_cast_expr(expr, ty, counter),

            // Match expression: M expr { pattern => body, ... }
            Expr::Match {
                expr: match_expr,
                arms,
            } => self.generate_match(match_expr, arms, counter),

            // Range expression: produce { i64 start, i64 end, i1 inclusive } struct
            Expr::Range {
                start,
                end,
                inclusive,
            } => self.generate_range_expr(start, end, *inclusive, counter),

            // Await expression: poll the future until Ready
            Expr::Await(inner) => self.generate_await_expr(inner, counter),

            // Spawn expression: create a concurrent task
            Expr::Spawn(inner) => self.generate_spawn_expr(inner, counter),

            // Yield expression: yield a value from a generator.
            Expr::Yield(inner) => self.generate_yield_expr(inner, counter),

            // Comptime expression: evaluate at compile time and emit constant
            Expr::Comptime { body } => self.generate_comptime_expr(body, counter),

            // Macro invocation (should be expanded before codegen)
            Expr::MacroInvoke(invoke) => Err(CodegenError::TypeError(format!(
                "Unexpanded macro invocation: {}! - macros must be expanded before code generation",
                invoke.name.node
            ))),

            // Old expression for contract ensures clauses
            Expr::Old(inner) => self.generate_old_expr(inner, counter),

            // Assert expression
            Expr::Assert { condition, message } => {
                self.generate_assert(condition, message.as_deref(), counter)
            }

            // Assume expression (verification hint, no runtime effect in release)
            Expr::Assume(inner) => self.generate_assume_expr(inner, counter),

            // Lambda expression with captures
            Expr::Lambda {
                params,
                body,
                capture_mode,
                ..
            } => self.generate_lambda_expr(params, body, capture_mode, counter),

            // Try expression: expr? - propagate Err early, continue with Ok value
            Expr::Try(inner) => self.generate_try_expr(inner, counter),

            // Unwrap expression: expr! - panic on Err/None, continue with value
            Expr::Unwrap(inner) => self.generate_unwrap_expr(inner, counter),

            // Error nodes should not reach codegen
            Expr::Error { message, .. } => Err(CodegenError::Unsupported(format!(
                "Parse error in expression: {}",
                message
            ))),

            // Lazy and Force expressions - delegate to visitor
            Expr::Lazy(inner) => {
                use crate::visitor::ExprVisitor;
                self.visit_lazy(inner, counter)
            }
            Expr::Force(inner) => {
                use crate::visitor::ExprVisitor;
                self.visit_force(inner, counter)
            }

            // Enum variant access: EnumName::Variant or EnumName::Variant(data)
            // Unit variant: treat as identifier lookup (same as Expr::Ident(variant))
            // Tuple variant with data: treat as call Variant(data)
            Expr::EnumAccess {
                enum_name: _,
                variant,
                data: None,
            } => self.generate_ident_expr(variant, counter),
            Expr::EnumAccess {
                enum_name: _,
                variant,
                data: Some(data_expr),
            } => self.generate_enum_access_data_expr(variant, data_expr, counter),
        };

        // Register the resolved type for named temporaries.
        // This enables downstream passes (store, binary, icmp, call, phi) to emit
        // correct LLVM IR types instead of defaulting to i64.
        // Covers %tN temporaries, %name.N locals, and other %-prefixed registers.
        if let Ok((ref val, _)) = result {
            if val.starts_with('%') {
                self.fn_ctx.register_temp_type(val, inferred_type);
            }
        }

        result
    }
}
