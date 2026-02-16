//! Expression code generation.
//!
//! Handles generation of all expression types: literals, variables,
//! binary/unary operations, function calls, etc.

// Submodules organized by expression category
mod literal;
mod var;
mod binary;
mod unary;
mod call;
mod lambda;
mod misc;

// Re-export the main generate_expr implementation
use inkwell::values::BasicValueEnum;
use vais_ast::{BinOp, Expr};

use super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn generate_expr(&mut self, expr: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        match expr {
            // Literals
            Expr::Int(n) => Ok(self.context.i64_type().const_int(*n as u64, true).into()),
            Expr::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            Expr::Bool(b) => Ok(self.context.bool_type().const_int(*b as u64, false).into()),
            Expr::String(s) => self.generate_string_literal(s),
            Expr::Unit => Ok(self.context.struct_type(&[], false).const_zero().into()),

            // Variable
            Expr::Ident(name) => self.generate_var(name),

            // Binary/Unary operations
            Expr::Binary { op, left, right } => self.generate_binary(*op, &left.node, &right.node),
            Expr::Unary { op, expr: operand } => self.generate_unary(*op, &operand.node),

            // Function call
            Expr::Call { func, args } => self.generate_call(&func.node, args),

            // Block
            Expr::Block(stmts) => self.generate_block(stmts),

            // Control flow
            Expr::If { cond, then, else_ } => {
                self.generate_if_expr(&cond.node, then, else_.as_ref())
            }
            Expr::Loop {
                pattern,
                iter,
                body,
            } => self.generate_loop(pattern.as_ref(), iter.as_deref(), body),
            Expr::While { condition, body } => self.generate_while_loop(condition, body),
            Expr::Match {
                expr: match_expr,
                arms,
            } => self.generate_match(match_expr, arms),

            // Struct
            Expr::StructLit { name, fields } => self.generate_struct_literal(&name.node, fields),
            Expr::Field { expr: obj, field } => self.generate_field_access(&obj.node, &field.node),

            // Array/Tuple/Index
            Expr::Array(elements) => self.generate_array(elements),
            Expr::MapLit(_pairs) => {
                // Map literals not yet supported in inkwell backend
                Ok(self.context.i64_type().const_int(0, false).into())
            }
            Expr::Tuple(elements) => self.generate_tuple(elements),
            Expr::Index { expr: arr, index } => {
                // Check if this is a slice operation (index is a Range expression)
                if let Expr::Range {
                    start,
                    end,
                    inclusive,
                } = &index.node
                {
                    return self.generate_slice(
                        &arr.node,
                        start.as_deref(),
                        end.as_deref(),
                        *inclusive,
                    );
                }
                self.generate_index(&arr.node, &index.node)
            }

            // Method call
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => self.generate_method_call(&receiver.node, &method.node, args),

            // Lambda/Closure
            Expr::Lambda {
                params,
                body,
                captures,
                capture_mode,
            } => self.generate_lambda(params, &body.node, captures, *capture_mode),

            // Try/Unwrap
            Expr::Try(inner) => self.generate_try(&inner.node),
            Expr::Unwrap(inner) => self.generate_unwrap(&inner.node),

            // Assignment
            Expr::Assign { target, value } => self.generate_assign(&target.node, &value.node),
            Expr::AssignOp { op, target, value } => {
                self.generate_assign_op(*op, &target.node, &value.node)
            }

            // Reference/Dereference
            Expr::Ref(inner) => {
                // Get address of inner expression (lvalue)
                match &inner.node {
                    Expr::Ident(name) => {
                        if let Some((ptr, _)) = self.locals.get(name) {
                            Ok((*ptr).into())
                        } else {
                            let val = self.generate_expr(&inner.node)?;
                            Ok(val)
                        }
                    }
                    _ => {
                        // For non-lvalue expressions, create a temporary alloca
                        let val = self.generate_expr(&inner.node)?;
                        let alloca = self
                            .builder
                            .build_alloca(val.get_type(), "ref_tmp")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        self.builder
                            .build_store(alloca, val)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(alloca.into())
                    }
                }
            }
            Expr::Deref(inner) => self.generate_deref(&inner.node),

            // Type cast
            Expr::Cast {
                expr: cast_expr,
                ty: cast_ty,
            } => self.generate_cast(&cast_expr.node, &cast_ty.node),

            // Range
            Expr::Range {
                start,
                end,
                inclusive,
            } => self.generate_range(start.as_deref(), end.as_deref(), *inclusive),

            // Ternary
            Expr::Ternary { cond, then, else_ } => {
                self.generate_ternary(&cond.node, &then.node, &else_.node)
            }

            // Assert: evaluate condition, abort if false
            Expr::Assert { condition, message } => {
                self.generate_assert(&condition.node, message.as_deref())
            }

            // Comptime: evaluate at compile time (for now, just evaluate normally)
            Expr::Comptime { body } => self.generate_expr(&body.node),

            // Lazy: create deferred evaluation thunk
            Expr::Lazy(inner) => self.generate_lazy(&inner.node),

            // Await: evaluate the inner expression (async functions compile as synchronous
            // in Inkwell backend, so await is effectively identity — the function has
            // already completed and returned its result directly)
            Expr::Await(inner) => self.generate_expr(&inner.node),

            // Static method call: Type::method(args)
            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => {
                // Look up function as TypeName_method or just method
                let fn_name = format!("{}_{}", type_name.node, method.node);
                if self.functions.contains_key(&fn_name)
                    || self.module.get_function(&fn_name).is_some()
                {
                    let callee = Expr::Ident(fn_name);
                    self.generate_call(&callee, args)
                } else {
                    // Try just the method name
                    let callee = Expr::Ident(method.node.clone());
                    self.generate_call(&callee, args)
                }
            }

            // String interpolation
            Expr::StringInterp(parts) => self.generate_string_interp(parts),

            // Assume: compiler assumption for verification (no-op at runtime)
            Expr::Assume(inner) => {
                // Evaluate the inner expression but discard result
                let _ = self.generate_expr(&inner.node)?;
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }

            // Spread: just evaluate the inner expression
            Expr::Spread(inner) => self.generate_expr(&inner.node),

            // Force: evaluate a lazy value (check computed flag, call thunk if needed)
            Expr::Force(inner) => self.generate_force(&inner.node),

            // Spawn: evaluate inner expression to create a concurrent task.
            // In Inkwell backend, async functions compile as synchronous, so spawn
            // evaluates the inner expression immediately (eager evaluation).
            Expr::Spawn(inner) => self.generate_expr(&inner.node),

            // Yield: evaluate the inner expression and return its value.
            // Yields the value to the generator's caller. In the current synchronous
            // model, this evaluates and returns the value directly.
            Expr::Yield(inner) => self.generate_expr(&inner.node),

            // SelfCall (@): recursive call to current function
            Expr::SelfCall => {
                // Return a reference to the current function (for indirect calls)
                if let Some(func) = self.current_function {
                    Ok(func.as_global_value().as_pointer_value().into())
                } else {
                    Err(CodegenError::Unsupported(
                        "SelfCall outside of function".to_string(),
                    ))
                }
            }

            _ => Err(CodegenError::Unsupported(format!(
                "Expression kind not yet implemented: {:?}",
                expr
            ))),
        }
    }

    pub(super) fn generate_binary(
        &mut self,
        op: BinOp,
        lhs: &Expr,
        rhs: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let lhs_val = self.generate_expr(lhs)?;
        let rhs_val = self.generate_expr(rhs)?;

        // Determine if we're dealing with integers or floats
        let is_float = lhs_val.is_float_value();

        if is_float {
            self.generate_float_binary(op, lhs_val.into_float_value(), rhs_val.into_float_value())
        } else {
            self.generate_int_binary(op, lhs_val.into_int_value(), rhs_val.into_int_value())
        }
    }
}
