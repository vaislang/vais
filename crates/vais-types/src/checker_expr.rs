//! Expression and statement type checking.

use std::collections::HashMap;

use vais_ast::*;

use super::TypeChecker;
use crate::comptime;
use crate::types::{
    self, GenericInstantiation, Linearity, ResolvedType, TypeError, TypeResult, VariantFieldTypes,
};

impl TypeChecker {
    /// Check a block of statements
    pub(crate) fn check_block(&mut self, stmts: &[Spanned<Stmt>]) -> TypeResult<ResolvedType> {
        let mut last_type = ResolvedType::Unit;

        for stmt in stmts {
            last_type = self.check_stmt(stmt)?;
        }

        Ok(last_type)
    }

    /// Check a statement
    pub(crate) fn check_stmt(&mut self, stmt: &Spanned<Stmt>) -> TypeResult<ResolvedType> {
        match &stmt.node {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut,
                ownership,
            } => {
                let value_type = self.check_expr(value)?;
                let var_type = if let Some(ty) = ty {
                    let expected = self.resolve_type(&ty.node);
                    self.unify(&expected, &value_type)?;
                    expected
                } else {
                    value_type
                };

                // Convert AST Ownership to type system Linearity
                let linearity = match ownership {
                    Ownership::Linear => Linearity::Linear,
                    Ownership::Affine => Linearity::Affine,
                    Ownership::Move => Linearity::Affine, // Move semantics act like affine
                    Ownership::Regular => {
                        // Check if the type itself is linear/affine
                        match &var_type {
                            ResolvedType::Linear(_) => Linearity::Linear,
                            ResolvedType::Affine(_) => Linearity::Affine,
                            _ => Linearity::Unrestricted,
                        }
                    }
                };

                self.define_var_with_linearity(
                    &name.node,
                    var_type,
                    *is_mut,
                    linearity,
                    Some(name.span),
                );
                Ok(ResolvedType::Unit)
            }
            Stmt::LetDestructure {
                pattern,
                value,
                is_mut,
            } => {
                let value_type = self.check_expr(value)?;
                self.check_destructure_pattern(pattern, &value_type, *is_mut)?;
                Ok(ResolvedType::Unit)
            }
            Stmt::Expr(expr) => self.check_expr(expr),
            Stmt::Return(expr) => {
                let ret_type = if let Some(expr) = expr {
                    self.check_expr(expr)?
                } else {
                    ResolvedType::Unit
                };
                if let Some(expected) = self.current_fn_ret.clone() {
                    // Auto-deref: if returning &T but expected is T, allow implicit deref
                    let ret_type_deref = if let ResolvedType::Ref(inner) = &ret_type {
                        if self.unify(&expected, inner).is_ok() {
                            *inner.clone()
                        } else {
                            ret_type.clone()
                        }
                    } else {
                        ret_type.clone()
                    };
                    self.unify(&expected, &ret_type_deref)?;
                }
                // Return has "Never" type because execution doesn't continue past it
                Ok(ResolvedType::Never)
            }
            // Break and Continue have "Never" type because execution doesn't continue past them
            Stmt::Break(_) | Stmt::Continue => Ok(ResolvedType::Never),

            Stmt::Defer(expr) => {
                // Type check the deferred expression
                // Defer expressions typically should be function calls that return unit
                self.check_expr(expr)?;
                // Defer itself doesn't produce a value in the control flow
                Ok(ResolvedType::Unit)
            }
            Stmt::Error { .. } => {
                // Error nodes from recovery mode are treated as having Unknown type.
                // The parsing error has already been reported.
                Ok(ResolvedType::Unknown)
            }
        }
    }

    /// Check a destructuring pattern against a type and bind variables
    pub(crate) fn check_destructure_pattern(
        &mut self,
        pattern: &Spanned<Pattern>,
        ty: &ResolvedType,
        is_mut: bool,
    ) -> TypeResult<()> {
        match &pattern.node {
            Pattern::Ident(name) => {
                self.define_var_with_linearity(
                    name,
                    ty.clone(),
                    is_mut,
                    Linearity::Unrestricted,
                    Some(pattern.span),
                );
                Ok(())
            }
            Pattern::Tuple(patterns) => {
                if let ResolvedType::Tuple(types) = ty {
                    if patterns.len() != types.len() {
                        return Err(TypeError::Mismatch {
                            expected: format!("tuple of {} elements", patterns.len()),
                            found: format!("tuple of {} elements", types.len()),
                            span: Some(pattern.span),
                        });
                    }
                    for (pat, elem_ty) in patterns.iter().zip(types.iter()) {
                        self.check_destructure_pattern(pat, elem_ty, is_mut)?;
                    }
                    Ok(())
                } else {
                    Err(TypeError::Mismatch {
                        expected: "tuple".to_string(),
                        found: format!("{}", ty),
                        span: Some(pattern.span),
                    })
                }
            }
            Pattern::Wildcard => Ok(()),
            _ => Err(TypeError::Mismatch {
                expected: "identifier or tuple pattern".to_string(),
                found: "unsupported pattern in destructuring".to_string(),
                span: Some(pattern.span),
            }),
        }
    }

    /// Check an expression
    pub(crate) fn check_expr(&mut self, expr: &Spanned<Expr>) -> TypeResult<ResolvedType> {
        match &expr.node {
            Expr::Int(_) => Ok(ResolvedType::I64),
            Expr::Float(_) => Ok(ResolvedType::F64),
            Expr::Bool(_) => Ok(ResolvedType::Bool),
            Expr::String(_) => Ok(ResolvedType::Str),
            Expr::StringInterp(parts) => {
                // Type-check each interpolated expression
                for part in parts {
                    if let StringInterpPart::Expr(expr) = part {
                        self.check_expr(expr)?;
                    }
                }
                Ok(ResolvedType::Str)
            }
            Expr::Unit => Ok(ResolvedType::Unit),

            Expr::Ident(name) => {
                // Mark variable as used for linear type tracking
                self.mark_var_used(name);
                self.lookup_var_or_err(name)
            }

            Expr::SelfCall => {
                // @ can mean two things:
                // 1. In an impl method context, @ represents self (same as self variable)
                // 2. In a regular function, @(...) is a recursive call

                // First, check if we're in an impl method and have a 'self' variable
                // (this is for @.method() calls)
                if let Ok(var_info) = self.lookup_var_info("self") {
                    return Ok(var_info.ty);
                }

                // Otherwise, @ refers to current function (for recursion)
                if let Some(name) = &self.current_fn_name {
                    if let Some(sig) = self.functions.get(name) {
                        // For async functions, wrap the return type in Future
                        let ret_type = if sig.is_async {
                            ResolvedType::Future(Box::new(sig.ret.clone()))
                        } else {
                            sig.ret.clone()
                        };

                        return Ok(ResolvedType::Fn {
                            params: sig.params.iter().map(|(_, t, _)| t).cloned().collect(),
                            ret: Box::new(ret_type),
                            effects: None,
                        });
                    }
                }
                Err(TypeError::UndefinedFunction {
                    name: "@".to_string(),
                    span: Some(expr.span),
                    suggestion: None,
                })
            }

            Expr::Binary { op, left, right } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        // Allow string concatenation with +
                        if matches!(op, BinOp::Add) && matches!(left_type, ResolvedType::Str) {
                            self.unify(&left_type, &right_type)?;
                            return Ok(ResolvedType::Str);
                        }
                        if !left_type.is_numeric() {
                            return Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: left_type.to_string(),
                                span: Some(left.span),
                            });
                        }
                        self.unify(&left_type, &right_type)?;
                        Ok(left_type)
                    }
                    BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte => {
                        // Allow string comparison with <, >, <=, >=
                        if matches!(left_type, ResolvedType::Str) {
                            self.unify(&left_type, &right_type)?;
                            return Ok(ResolvedType::Bool);
                        }
                        if !left_type.is_numeric() {
                            return Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: left_type.to_string(),
                                span: Some(left.span),
                            });
                        }
                        self.unify(&left_type, &right_type)?;
                        Ok(ResolvedType::Bool)
                    }
                    BinOp::Eq | BinOp::Neq => {
                        self.unify(&left_type, &right_type)?;
                        Ok(ResolvedType::Bool)
                    }
                    BinOp::And | BinOp::Or => {
                        self.unify(&left_type, &ResolvedType::Bool)?;
                        self.unify(&right_type, &ResolvedType::Bool)?;
                        Ok(ResolvedType::Bool)
                    }
                    BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
                        // Allow bool operands for BitAnd (&) and BitOr (|) as logical and/or
                        if matches!(left_type, ResolvedType::Bool)
                            && matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
                        {
                            self.unify(&left_type, &right_type)?;
                            return Ok(ResolvedType::Bool);
                        }
                        if !left_type.is_integer() {
                            return Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: left_type.to_string(),
                                span: Some(left.span),
                            });
                        }
                        self.unify(&left_type, &right_type)?;
                        Ok(left_type)
                    }
                }
            }

            Expr::Unary { op, expr: inner } => {
                let inner_type = self.check_expr(inner)?;
                match op {
                    UnaryOp::Neg => {
                        if !inner_type.is_numeric() {
                            return Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: inner_type.to_string(),
                                span: Some(inner.span),
                            });
                        }
                        Ok(inner_type)
                    }
                    UnaryOp::Not => {
                        self.unify(&inner_type, &ResolvedType::Bool)?;
                        Ok(ResolvedType::Bool)
                    }
                    UnaryOp::BitNot => {
                        if !inner_type.is_integer() {
                            return Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: inner_type.to_string(),
                                span: Some(inner.span),
                            });
                        }
                        Ok(inner_type)
                    }
                }
            }

            Expr::Ternary { cond, then, else_ } => {
                let cond_type = self.check_expr(cond)?;
                self.unify(&cond_type, &ResolvedType::Bool)?;

                let then_type = self.check_expr(then)?;
                let else_type = self.check_expr(else_)?;
                self.unify(&then_type, &else_type)?;

                Ok(then_type)
            }

            Expr::If { cond, then, else_ } => {
                let cond_type = self.check_expr(cond)?;
                self.unify(&cond_type, &ResolvedType::Bool)?;

                self.push_scope();
                let then_type = self.check_block(then)?;
                self.pop_scope();

                if let Some(else_branch) = else_ {
                    let else_type = self.check_if_else(else_branch)?;
                    // If branch types don't unify, treat if-else as statement (Unit type)
                    if self.unify(&then_type, &else_type).is_ok() {
                        Ok(then_type)
                    } else {
                        Ok(ResolvedType::Unit)
                    }
                } else {
                    Ok(ResolvedType::Unit)
                }
            }

            Expr::Loop {
                pattern,
                iter,
                body,
            } => {
                self.push_scope();

                if let (Some(pattern), Some(iter)) = (pattern, iter) {
                    let iter_type = self.check_expr(iter)?;

                    // Try to infer the element type from the iterator
                    if let Some(elem_type) = self.get_iterator_item_type(&iter_type) {
                        // Bind the pattern variable with the inferred element type
                        if let Pattern::Ident(name) = &pattern.node {
                            self.define_var(name, elem_type, false);
                        }
                    } else {
                        // Couldn't infer iterator item type - this is a warning but not an error
                        // The loop will still work, just without type information for the pattern
                        if let Pattern::Ident(name) = &pattern.node {
                            self.warnings.push(format!(
                                "Cannot infer iterator item type for variable '{}' in loop",
                                name
                            ));
                        }
                    }
                }

                self.check_block(body)?;
                self.pop_scope();

                Ok(ResolvedType::Unit)
            }

            Expr::While { condition, body } => {
                // Check that condition is a boolean expression
                let cond_type = self.check_expr(condition)?;
                self.unify(&ResolvedType::Bool, &cond_type)?;

                self.push_scope();
                self.check_block(body)?;
                self.pop_scope();

                Ok(ResolvedType::Unit)
            }

            Expr::Match { expr, arms } => {
                let expr_type = self.check_expr(expr)?;
                let mut result_type: Option<ResolvedType> = None;

                for arm in arms {
                    self.push_scope();

                    // Register pattern bindings in scope
                    self.register_pattern_bindings(&arm.pattern, &expr_type)?;

                    // Check guard if present
                    if let Some(guard) = &arm.guard {
                        let guard_type = self.check_expr(guard)?;
                        self.unify(&ResolvedType::Bool, &guard_type)?;
                    }

                    let arm_type = self.check_expr(&arm.body)?;
                    self.pop_scope();

                    if let Some(ref prev) = result_type {
                        self.unify(prev, &arm_type)?;
                    } else {
                        result_type = Some(arm_type);
                    }
                }

                // Exhaustiveness check
                let exhaustiveness_result =
                    self.exhaustiveness_checker.check_match(&expr_type, arms);

                // Report unreachable arms as warnings
                for arm_idx in &exhaustiveness_result.unreachable_arms {
                    self.warnings
                        .push(format!("Unreachable pattern in match arm {}", arm_idx + 1));
                }

                // Non-exhaustive match is a warning (not error) for now
                // to maintain backwards compatibility
                if !exhaustiveness_result.is_exhaustive {
                    self.warnings.push(format!(
                        "Non-exhaustive match: missing patterns: {}",
                        exhaustiveness_result.missing_patterns.join(", ")
                    ));
                }

                Ok(result_type.unwrap_or(ResolvedType::Unit))
            }

            Expr::Call { func, args } => {
                // Check if this is a direct call to a known function
                if let Expr::Ident(func_name) = &func.node {
                    // Struct tuple literal: `Response(200, 1)` → `Response { status: 200, body: 1 }`
                    if !self.functions.contains_key(func_name) {
                        if let Some(struct_def) = self.structs.get(func_name).cloned() {
                            if args.len() != struct_def.field_order.len() {
                                return Err(TypeError::ArgCount {
                                    expected: struct_def.field_order.len(),
                                    got: args.len(),
                                    span: Some(expr.span),
                                });
                            }
                            // Desugar to StructLit and type-check
                            let fields: Vec<_> = struct_def
                                .field_order
                                .iter()
                                .zip(args.iter())
                                .map(|(fname, val)| {
                                    (vais_ast::Spanned::new(fname.clone(), val.span), val.clone())
                                })
                                .collect();
                            let struct_lit = vais_ast::Spanned::new(
                                Expr::StructLit {
                                    name: vais_ast::Spanned::new(func_name.clone(), func.span),
                                    fields,
                                },
                                expr.span,
                            );
                            return self.check_expr(&struct_lit);
                        }
                    }
                    if let Some(sig) = self.functions.get(func_name).cloned() {
                        if !sig.generics.is_empty() {
                            // Generic function call - infer type arguments
                            return self.check_generic_function_call(&sig, args);
                        }
                        // Check arg count with default parameters and vararg support
                        let min_args = sig.min_args();
                        let max_args = sig.params.len();
                        if args.len() < min_args || (!sig.is_vararg && args.len() > max_args) {
                            return Err(TypeError::ArgCount {
                                expected: max_args,
                                got: args.len(),
                                span: Some(expr.span),
                            });
                        }
                        // Type-check provided arguments against param types
                        for (i, arg) in args.iter().enumerate() {
                            let arg_type = self.check_expr(arg)?;
                            if i < sig.params.len() {
                                self.unify(&sig.params[i].1, &arg_type)?;
                            }
                        }
                        // For async functions, wrap the return type in Future
                        if sig.is_async {
                            return Ok(ResolvedType::Future(Box::new(sig.ret.clone())));
                        }
                        return Ok(sig.ret.clone());
                    }
                }

                let func_type = self.check_expr(func)?;

                match func_type {
                    ResolvedType::Fn { params, ret, .. }
                    | ResolvedType::FnPtr { params, ret, .. } => {
                        if params.len() != args.len() {
                            return Err(TypeError::ArgCount {
                                expected: params.len(),
                                got: args.len(),
                                span: Some(expr.span),
                            });
                        }

                        for (param_type, arg) in params.iter().zip(args) {
                            let arg_type = self.check_expr(arg)?;
                            self.unify(param_type, &arg_type)?;
                        }

                        Ok(*ret)
                    }
                    _ => Err(TypeError::NotCallable(func_type.to_string(), None)),
                }
            }

            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                let receiver_type = self.check_expr(receiver)?;

                // Extract the inner type if receiver is a reference
                let (inner_type, receiver_generics) = match &receiver_type {
                    ResolvedType::Named { name, generics } => (name.clone(), generics.clone()),
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                        if let ResolvedType::Named { name, generics } = inner.as_ref() {
                            (name.clone(), generics.clone())
                        } else {
                            (String::new(), vec![])
                        }
                    }
                    _ => (String::new(), vec![]),
                };

                // First, try to find the method on the struct or enum itself
                if !inner_type.is_empty() {
                    // Look up method in struct or enum
                    let found_method = self
                        .structs
                        .get(&inner_type)
                        .and_then(|s| s.methods.get(&method.node).cloned())
                        .or_else(|| {
                            self.enums
                                .get(&inner_type)
                                .and_then(|e| e.methods.get(&method.node).cloned())
                        });
                    let found_generics = self
                        .structs
                        .get(&inner_type)
                        .map(|s| s.generics.clone())
                        .or_else(|| self.enums.get(&inner_type).map(|e| e.generics.clone()))
                        .unwrap_or_default();
                    if let Some(method_sig) = found_method {
                        // Skip self parameter
                        let param_types: Vec<_> = method_sig
                            .params
                            .iter()
                            .skip(1)
                            .map(|(_, t, _)| t.clone())
                            .collect();

                        if param_types.len() != args.len() {
                            return Err(TypeError::ArgCount {
                                expected: param_types.len(),
                                got: args.len(),
                                span: Some(expr.span),
                            });
                        }

                        // Build substitution map from type's generic params to receiver's concrete types
                        let generic_substitutions: std::collections::HashMap<String, ResolvedType> =
                            found_generics
                                .iter()
                                .zip(receiver_generics.iter())
                                .map(|(param, arg)| (param.clone(), arg.clone()))
                                .collect();

                        // Check arguments with substituted parameter types
                        for (param_type, arg) in param_types.iter().zip(args) {
                            let arg_type = self.check_expr(arg)?;
                            let expected_type = if generic_substitutions.is_empty() {
                                param_type.clone()
                            } else {
                                self.substitute_generics(param_type, &generic_substitutions)
                            };
                            self.unify(&expected_type, &arg_type)?;
                        }

                        // Substitute generics in return type
                        let ret_type_raw = if generic_substitutions.is_empty() {
                            method_sig.ret.clone()
                        } else {
                            self.substitute_generics(&method_sig.ret, &generic_substitutions)
                        };

                        // For async methods, wrap the return type in Future
                        let ret_type = if method_sig.is_async {
                            ResolvedType::Future(Box::new(ret_type_raw))
                        } else {
                            ret_type_raw
                        };

                        return Ok(ret_type);
                    }
                }

                // If not found on struct/enum, try to find it in trait implementations
                if let Some(trait_method) = self.find_trait_method(&receiver_type, &method.node) {
                    // Skip self parameter (first parameter)
                    let param_types: Vec<_> = trait_method
                        .params
                        .iter()
                        .skip(1)
                        .map(|(_, t, _)| t.clone())
                        .collect();

                    if param_types.len() != args.len() {
                        return Err(TypeError::ArgCount {
                            expected: param_types.len(),
                            got: args.len(),
                            span: Some(expr.span),
                        });
                    }

                    for (param_type, arg) in param_types.iter().zip(args) {
                        let arg_type = self.check_expr(arg)?;
                        self.unify(param_type, &arg_type)?;
                    }

                    // For async trait methods, wrap the return type in Future
                    let ret_type = if trait_method.is_async {
                        ResolvedType::Future(Box::new(trait_method.ret.clone()))
                    } else {
                        trait_method.ret.clone()
                    };

                    return Ok(ret_type);
                }

                // Built-in string methods
                if matches!(receiver_type, ResolvedType::Str) {
                    match method.node.as_str() {
                        "len" => {
                            if !args.is_empty() {
                                return Err(TypeError::ArgCount {
                                    expected: 0,
                                    got: args.len(),
                                    span: Some(expr.span),
                                });
                            }
                            return Ok(ResolvedType::I64);
                        }
                        "charAt" => {
                            if args.len() != 1 {
                                return Err(TypeError::ArgCount {
                                    expected: 1,
                                    got: args.len(),
                                    span: Some(expr.span),
                                });
                            }
                            let arg_type = self.check_expr(&args[0])?;
                            self.unify(&ResolvedType::I64, &arg_type)?;
                            return Ok(ResolvedType::I64);
                        }
                        "contains" | "startsWith" | "endsWith" => {
                            if args.len() != 1 {
                                return Err(TypeError::ArgCount {
                                    expected: 1,
                                    got: args.len(),
                                    span: Some(expr.span),
                                });
                            }
                            let arg_type = self.check_expr(&args[0])?;
                            self.unify(&ResolvedType::Str, &arg_type)?;
                            return Ok(ResolvedType::Bool);
                        }
                        "indexOf" => {
                            if args.len() != 1 {
                                return Err(TypeError::ArgCount {
                                    expected: 1,
                                    got: args.len(),
                                    span: Some(expr.span),
                                });
                            }
                            let arg_type = self.check_expr(&args[0])?;
                            self.unify(&ResolvedType::Str, &arg_type)?;
                            return Ok(ResolvedType::I64);
                        }
                        "substring" => {
                            if args.len() != 2 {
                                return Err(TypeError::ArgCount {
                                    expected: 2,
                                    got: args.len(),
                                    span: Some(expr.span),
                                });
                            }
                            let start_type = self.check_expr(&args[0])?;
                            self.unify(&ResolvedType::I64, &start_type)?;
                            let end_type = self.check_expr(&args[1])?;
                            self.unify(&ResolvedType::I64, &end_type)?;
                            return Ok(ResolvedType::Str);
                        }
                        "isEmpty" => {
                            if !args.is_empty() {
                                return Err(TypeError::ArgCount {
                                    expected: 0,
                                    got: args.len(),
                                    span: Some(expr.span),
                                });
                            }
                            return Ok(ResolvedType::Bool);
                        }
                        _ => {} // Fall through to error
                    }
                }

                // Built-in slice methods
                if matches!(
                    &receiver_type,
                    ResolvedType::Slice(_) | ResolvedType::SliceMut(_)
                ) && method.node.as_str() == "len"
                {
                    if !args.is_empty() {
                        return Err(TypeError::ArgCount {
                            expected: 0,
                            got: args.len(),
                            span: Some(expr.span),
                        });
                    }
                    return Ok(ResolvedType::I64);
                }

                // Try to find similar method names for suggestion
                let suggestion = types::find_similar_name(
                    &method.node,
                    self.functions.keys().map(|s| s.as_str()),
                );
                Err(TypeError::UndefinedFunction {
                    name: method.node.clone(),
                    span: Some(method.span),
                    suggestion,
                })
            }

            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => {
                // Static method call: Type.method(args)
                if let Some(struct_def) = self.structs.get(&type_name.node).cloned() {
                    if let Some(method_sig) = struct_def.methods.get(&method.node).cloned() {
                        // For static methods, don't skip first param (no self)
                        // But if the first param is self, skip it for backwards compat
                        let param_types: Vec<_> = if method_sig
                            .params
                            .first()
                            .map(|(n, _, _)| n == "self")
                            .unwrap_or(false)
                        {
                            method_sig
                                .params
                                .iter()
                                .skip(1)
                                .map(|(_, t, _)| t.clone())
                                .collect()
                        } else {
                            method_sig
                                .params
                                .iter()
                                .map(|(_, t, _)| t.clone())
                                .collect()
                        };

                        if param_types.len() != args.len() {
                            return Err(TypeError::ArgCount {
                                expected: param_types.len(),
                                got: args.len(),
                                span: Some(expr.span),
                            });
                        }

                        // Handle generic struct type inference
                        if !struct_def.generics.is_empty() {
                            // Create fresh type variables for each struct generic parameter
                            let generic_substitutions: std::collections::HashMap<
                                String,
                                ResolvedType,
                            > = struct_def
                                .generics
                                .iter()
                                .map(|param| (param.clone(), self.fresh_type_var()))
                                .collect();

                            // Substitute generics in parameter types and check arguments
                            for (param_type, arg) in param_types.iter().zip(args) {
                                let arg_type = self.check_expr(arg)?;
                                let expected_type =
                                    self.substitute_generics(param_type, &generic_substitutions);
                                self.unify(&expected_type, &arg_type)?;
                            }

                            // Substitute generics in return type
                            let return_type =
                                self.substitute_generics(&method_sig.ret, &generic_substitutions);
                            let resolved_return = self.apply_substitutions(&return_type);

                            // Record the generic instantiation if all type arguments are concrete
                            let inferred_type_args: Vec<_> = struct_def
                                .generics
                                .iter()
                                .map(|param| {
                                    let ty = generic_substitutions.get(param).expect(
                                        "Generic parameter should exist in substitutions map",
                                    );
                                    self.apply_substitutions(ty)
                                })
                                .collect();

                            let all_concrete = inferred_type_args
                                .iter()
                                .all(|t| !matches!(t, ResolvedType::Var(_)));
                            if all_concrete {
                                let inst = GenericInstantiation::struct_type(
                                    &type_name.node,
                                    inferred_type_args,
                                );
                                self.add_instantiation(inst);
                            }

                            return Ok(resolved_return);
                        }

                        // Non-generic struct - original behavior
                        for (param_type, arg) in param_types.iter().zip(args) {
                            let arg_type = self.check_expr(arg)?;
                            self.unify(param_type, &arg_type)?;
                        }

                        return Ok(method_sig.ret.clone());
                    }
                }

                // Get struct methods for suggestion if available
                let suggestion = if let Some(struct_def) = self.structs.get(&type_name.node) {
                    types::find_similar_name(
                        &method.node,
                        struct_def.methods.keys().map(|s| s.as_str()),
                    )
                } else {
                    None
                };
                Err(TypeError::UndefinedFunction {
                    name: format!("{}::{}", type_name.node, method.node),
                    span: Some(method.span),
                    suggestion,
                })
            }

            Expr::Field { expr: inner, field } => {
                let inner_type = self.check_expr(inner)?;

                // Handle both direct Named types and references to Named types
                let type_name = match &inner_type {
                    ResolvedType::Named { name, .. } => Some(name.clone()),
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                        if let ResolvedType::Named { name, .. } = inner.as_ref() {
                            Some(name.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(name) = type_name.clone() {
                    // Check struct fields
                    if let Some(struct_def) = self.structs.get(&name) {
                        if let Some(field_type) = struct_def.fields.get(&field.node) {
                            return Ok(field_type.clone());
                        }
                    }
                    // Check union fields
                    if let Some(union_def) = self.unions.get(&name) {
                        if let Some(field_type) = union_def.fields.get(&field.node) {
                            return Ok(field_type.clone());
                        }
                    }
                }

                // Get field names for did-you-mean suggestion
                let suggestion = if let Some(ref name) = type_name {
                    if let Some(struct_def) = self.structs.get(name) {
                        types::find_similar_name(
                            &field.node,
                            struct_def.fields.keys().map(|s| s.as_str()),
                        )
                    } else if let Some(union_def) = self.unions.get(name) {
                        types::find_similar_name(
                            &field.node,
                            union_def.fields.keys().map(|s| s.as_str()),
                        )
                    } else {
                        None
                    }
                } else {
                    None
                };

                let display_type_name = type_name.unwrap_or_else(|| inner_type.to_string());
                Err(TypeError::NoSuchField {
                    field: field.node.clone(),
                    type_name: display_type_name,
                    suggestion,
                    span: Some(field.span),
                })
            }

            Expr::Index { expr: inner, index } => {
                let inner_type = self.check_expr(inner)?;
                let index_type = self.check_expr(index)?;

                // Check if this is a slice operation (index is a Range)
                let is_slice = matches!(index.node, Expr::Range { .. });

                match inner_type {
                    ResolvedType::Array(elem_type) => {
                        if is_slice {
                            // Slice returns a pointer to array elements
                            Ok(ResolvedType::Pointer(elem_type))
                        } else if !index_type.is_integer() {
                            Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            })
                        } else {
                            Ok(*elem_type)
                        }
                    }
                    ResolvedType::Map(key_type, value_type) => {
                        self.unify(&key_type, &index_type)?;
                        Ok(*value_type)
                    }
                    // Pointers can be indexed like arrays
                    ResolvedType::Pointer(elem_type) => {
                        if is_slice {
                            // Slice of pointer returns a pointer
                            Ok(ResolvedType::Pointer(elem_type))
                        } else if !index_type.is_integer() {
                            Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            })
                        } else {
                            Ok(*elem_type)
                        }
                    }
                    ResolvedType::Slice(elem_type) => {
                        if is_slice {
                            Ok(ResolvedType::Slice(elem_type))
                        } else if !index_type.is_integer() {
                            Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            })
                        } else {
                            Ok(*elem_type)
                        }
                    }
                    ResolvedType::SliceMut(elem_type) => {
                        if is_slice {
                            Ok(ResolvedType::SliceMut(elem_type))
                        } else if !index_type.is_integer() {
                            Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            })
                        } else {
                            Ok(*elem_type)
                        }
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "indexable type".to_string(),
                        found: inner_type.to_string(),
                        span: Some(expr.span),
                    }),
                }
            }

            Expr::Array(exprs) => {
                if exprs.is_empty() {
                    let var = self.fresh_type_var();
                    // Array literals decay to pointers in Vais
                    return Ok(ResolvedType::Pointer(Box::new(var)));
                }

                // Helper: get element type from an array element (handles Spread)
                let get_elem_type =
                    |checker: &mut Self, e: &Spanned<Expr>| -> TypeResult<ResolvedType> {
                        if let Expr::Spread(inner) = &e.node {
                            let inner_type = checker.check_expr(inner)?;
                            // Spread must be on a pointer/array type
                            match inner_type {
                                ResolvedType::Pointer(elem) => Ok(*elem),
                                ResolvedType::Array(elem) => Ok(*elem),
                                _ => Ok(inner_type),
                            }
                        } else {
                            checker.check_expr(e)
                        }
                    };

                let first_type = get_elem_type(self, &exprs[0])?;
                for expr in &exprs[1..] {
                    let t = get_elem_type(self, expr)?;
                    self.unify(&first_type, &t)?;
                }

                // Array literals produce pointers to first element
                Ok(ResolvedType::Pointer(Box::new(first_type)))
            }

            Expr::Tuple(exprs) => {
                let types: Result<Vec<_>, _> = exprs.iter().map(|e| self.check_expr(e)).collect();
                Ok(ResolvedType::Tuple(types?))
            }

            Expr::MapLit(pairs) => {
                if pairs.is_empty() {
                    let k = self.fresh_type_var();
                    let v = self.fresh_type_var();
                    return Ok(ResolvedType::Map(Box::new(k), Box::new(v)));
                }
                let first_key_type = self.check_expr(&pairs[0].0)?;
                let first_val_type = self.check_expr(&pairs[0].1)?;
                for (k, v) in &pairs[1..] {
                    let kt = self.check_expr(k)?;
                    let vt = self.check_expr(v)?;
                    self.unify(&first_key_type, &kt)?;
                    self.unify(&first_val_type, &vt)?;
                }
                Ok(ResolvedType::Map(
                    Box::new(first_key_type),
                    Box::new(first_val_type),
                ))
            }

            Expr::StructLit { name, fields } => {
                // First check for struct
                if let Some(struct_def) = self.structs.get(&name.node).cloned() {
                    // Create fresh type variables for generic parameters
                    let generic_substitutions: HashMap<String, ResolvedType> = struct_def
                        .generics
                        .iter()
                        .map(|param| (param.clone(), self.fresh_type_var()))
                        .collect();

                    // Check each field and unify with expected type
                    for (field_name, value) in fields {
                        let value_type = self.check_expr(value)?;
                        if let Some(expected_type) =
                            struct_def.fields.get(&field_name.node).cloned()
                        {
                            // Substitute generic parameters with type variables
                            let expected_type =
                                self.substitute_generics(&expected_type, &generic_substitutions);
                            self.unify(&expected_type, &value_type)?;
                        } else {
                            let suggestion = types::find_similar_name(
                                &field_name.node,
                                struct_def.fields.keys().map(|s| s.as_str()),
                            );
                            return Err(TypeError::UndefinedVar {
                                name: field_name.node.clone(),
                                span: Some(field_name.span),
                                suggestion,
                            });
                        }
                    }

                    // Apply substitutions to infer concrete generic types
                    let inferred_generics: Vec<_> = struct_def
                        .generics
                        .iter()
                        .map(|param| {
                            let ty = generic_substitutions.get(param)
                                .expect("Internal compiler error: generic parameter should exist in substitutions map");
                            self.apply_substitutions(ty)
                        })
                        .collect();

                    // Record generic struct instantiation if the struct has generic parameters
                    if !struct_def.generics.is_empty() {
                        // Only record if all type arguments are concrete (not type variables)
                        let all_concrete = inferred_generics
                            .iter()
                            .all(|t| !matches!(t, ResolvedType::Var(_)));
                        if all_concrete {
                            let inst = GenericInstantiation::struct_type(
                                &name.node,
                                inferred_generics.clone(),
                            );
                            self.add_instantiation(inst);
                        }
                    }

                    Ok(ResolvedType::Named {
                        name: name.node.clone(),
                        generics: inferred_generics,
                    })
                // Then check for union (uses same syntax: `UnionName { field: value }`)
                } else if let Some(union_def) = self.unions.get(&name.node).cloned() {
                    // Create fresh type variables for generic parameters
                    let generic_substitutions: HashMap<String, ResolvedType> = union_def
                        .generics
                        .iter()
                        .map(|param| (param.clone(), self.fresh_type_var()))
                        .collect();

                    // Union literal should have exactly one field
                    if fields.len() != 1 {
                        return Err(TypeError::Mismatch {
                            expected: "exactly one field for union initialization".to_string(),
                            found: format!("{} fields", fields.len()),
                            span: Some(expr.span),
                        });
                    }

                    // Check the field
                    let (field_name, value) = &fields[0];
                    let value_type = self.check_expr(value)?;
                    if let Some(expected_type) = union_def.fields.get(&field_name.node).cloned() {
                        let expected_type =
                            self.substitute_generics(&expected_type, &generic_substitutions);
                        self.unify(&expected_type, &value_type)?;
                    } else {
                        let suggestion = types::find_similar_name(
                            &field_name.node,
                            union_def.fields.keys().map(|s| s.as_str()),
                        );
                        return Err(TypeError::UndefinedVar {
                            name: field_name.node.clone(),
                            span: Some(field_name.span),
                            suggestion,
                        });
                    }

                    // Apply substitutions to infer concrete generic types
                    let inferred_generics: Vec<_> = union_def
                        .generics
                        .iter()
                        .map(|param| {
                            let ty = generic_substitutions.get(param)
                                .expect("Internal compiler error: generic parameter should exist in substitutions map");
                            self.apply_substitutions(ty)
                        })
                        .collect();

                    Ok(ResolvedType::Named {
                        name: name.node.clone(),
                        generics: inferred_generics,
                    })
                } else {
                    // Get all type names for suggestion
                    let mut type_candidates: Vec<&str> = Vec::new();
                    type_candidates.extend(self.structs.keys().map(|s| s.as_str()));
                    type_candidates.extend(self.enums.keys().map(|s| s.as_str()));
                    type_candidates.extend(self.unions.keys().map(|s| s.as_str()));
                    type_candidates.extend(self.type_aliases.keys().map(|s| s.as_str()));

                    let suggestion =
                        types::find_similar_name(&name.node, type_candidates.into_iter());
                    Err(TypeError::UndefinedType {
                        name: name.node.clone(),
                        span: Some(name.span),
                        suggestion,
                    })
                }
            }

            Expr::Range {
                start,
                end,
                inclusive: _,
            } => {
                // Infer the element type from start or end expressions
                let elem_type = if let Some(start_expr) = start {
                    let start_type = self.check_expr(start_expr)?;
                    // Ensure start is a numeric type (integer)
                    if !start_type.is_integer() {
                        return Err(TypeError::Mismatch {
                            expected: "integer type".to_string(),
                            found: start_type.to_string(),
                            span: Some(start_expr.span),
                        });
                    }

                    // If end is present, unify the types
                    if let Some(end_expr) = end {
                        let end_type = self.check_expr(end_expr)?;
                        if !end_type.is_integer() {
                            return Err(TypeError::Mismatch {
                                expected: "integer type".to_string(),
                                found: end_type.to_string(),
                                span: Some(end_expr.span),
                            });
                        }
                        self.unify(&start_type, &end_type)?;
                    }

                    start_type
                } else if let Some(end_expr) = end {
                    // Only end is present (e.g., ..10)
                    let end_type = self.check_expr(end_expr)?;
                    if !end_type.is_integer() {
                        return Err(TypeError::Mismatch {
                            expected: "integer type".to_string(),
                            found: end_type.to_string(),
                            span: Some(end_expr.span),
                        });
                    }
                    end_type
                } else {
                    // Neither start nor end (e.g., ..) - default to i64
                    ResolvedType::I64
                };

                Ok(ResolvedType::Range(Box::new(elem_type)))
            }

            Expr::Block(stmts) => {
                self.push_scope();
                let result = self.check_block(stmts);
                self.pop_scope();
                result
            }

            Expr::Await(inner) => {
                let inner_type = self.check_expr(inner)?;

                // Verify that the inner expression is a Future type
                if let ResolvedType::Future(output_type) = inner_type {
                    // Extract and return the inner type from Future<T>
                    Ok(*output_type)
                } else {
                    Err(TypeError::Mismatch {
                        expected: "Future<T>".to_string(),
                        found: inner_type.to_string(),
                        span: Some(inner.span),
                    })
                }
            }

            Expr::Try(inner) => {
                let inner_type = self.check_expr(inner)?;
                // Try operator (?) works on both Result<T> and Option<T>
                // - Result<T>: returns T on Ok, propagates Err
                // - Option<T>: returns T on Some, propagates None
                match &inner_type {
                    ResolvedType::Result(ok_type, _err_type) => Ok(*ok_type.clone()),
                    ResolvedType::Optional(some_type) => Ok(*some_type.clone()),
                    // Also support user-defined enums named "Result" with Ok variant
                    ResolvedType::Named { name, .. } if name == "Result" => {
                        if let Some(enum_def) = self.enums.get("Result") {
                            if let Some(variant_fields) = enum_def.variants.get("Ok") {
                                match variant_fields {
                                    VariantFieldTypes::Tuple(types) if !types.is_empty() => {
                                        Ok(types[0].clone())
                                    }
                                    _ => Ok(ResolvedType::I64),
                                }
                            } else {
                                Ok(ResolvedType::I64)
                            }
                        } else {
                            Ok(ResolvedType::I64)
                        }
                    }
                    // Also support user-defined enums named "Option" with Some variant
                    ResolvedType::Named { name, .. } if name == "Option" => {
                        if let Some(enum_def) = self.enums.get("Option") {
                            if let Some(variant_fields) = enum_def.variants.get("Some") {
                                match variant_fields {
                                    VariantFieldTypes::Tuple(types) if !types.is_empty() => {
                                        Ok(types[0].clone())
                                    }
                                    _ => Ok(ResolvedType::I64),
                                }
                            } else {
                                Ok(ResolvedType::I64)
                            }
                        } else {
                            Ok(ResolvedType::I64)
                        }
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "Result or Option type".to_string(),
                        found: inner_type.to_string(),
                        span: Some(inner.span),
                    }),
                }
            }

            Expr::Unwrap(inner) => {
                let inner_type = self.check_expr(inner)?;
                match &inner_type {
                    ResolvedType::Optional(inner) => Ok(*inner.clone()),
                    ResolvedType::Result(ok, _err) => Ok(*ok.clone()),
                    // Support user-defined Result/Option enums
                    ResolvedType::Named { name, .. } if name == "Result" => {
                        if let Some(enum_def) = self.enums.get("Result") {
                            if let Some(VariantFieldTypes::Tuple(types)) =
                                enum_def.variants.get("Ok")
                            {
                                if !types.is_empty() {
                                    Ok(types[0].clone())
                                } else {
                                    Ok(ResolvedType::I64)
                                }
                            } else {
                                Ok(ResolvedType::I64)
                            }
                        } else {
                            Ok(ResolvedType::I64)
                        }
                    }
                    ResolvedType::Named { name, .. } if name == "Option" => {
                        if let Some(enum_def) = self.enums.get("Option") {
                            if let Some(VariantFieldTypes::Tuple(types)) =
                                enum_def.variants.get("Some")
                            {
                                if !types.is_empty() {
                                    Ok(types[0].clone())
                                } else {
                                    Ok(ResolvedType::I64)
                                }
                            } else {
                                Ok(ResolvedType::I64)
                            }
                        } else {
                            Ok(ResolvedType::I64)
                        }
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "Optional or Result".to_string(),
                        found: inner_type.to_string(),
                        span: Some(inner.span),
                    }),
                }
            }

            Expr::Spread(inner) => {
                // Spread is valid inside array literals; standalone spread just checks inner
                self.check_expr(inner)
            }

            Expr::Ref(inner) => {
                let inner_type = self.check_expr(inner)?;
                Ok(ResolvedType::Ref(Box::new(inner_type)))
            }

            Expr::Deref(inner) => {
                let inner_type = self.check_expr(inner)?;
                match inner_type {
                    ResolvedType::Ref(t) | ResolvedType::RefMut(t) | ResolvedType::Pointer(t) => {
                        Ok(*t)
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "reference or pointer".to_string(),
                        found: inner_type.to_string(),
                        span: Some(inner.span),
                    }),
                }
            }

            Expr::Cast { expr, ty } => {
                // Check the expression
                let _expr_type = self.check_expr(expr)?;
                // Resolve the target type
                let target_type = self.resolve_type(&ty.node);
                // For now, allow all casts - runtime will handle invalid ones
                Ok(target_type)
            }

            Expr::Assign { target, value } => {
                // Allow assignment to all variables (mutable by default in Vais)
                let target_type = self.check_expr(target)?;
                let value_type = self.check_expr(value)?;
                self.unify(&target_type, &value_type)?;
                Ok(ResolvedType::Unit)
            }

            Expr::AssignOp {
                op: _,
                target,
                value,
            } => {
                let target_type = self.check_expr(target)?;
                let value_type = self.check_expr(value)?;
                self.unify(&target_type, &value_type)?;
                Ok(ResolvedType::Unit)
            }

            Expr::Lambda {
                params,
                body,
                captures: _,
                capture_mode,
            } => {
                // Find free variables (captures) before entering lambda scope
                let param_names: std::collections::HashSet<_> =
                    params.iter().map(|p| p.name.node.clone()).collect();
                let free_vars = self.find_free_vars_in_expr(body, &param_names);

                // Verify all captured variables exist in current scope
                for var in &free_vars {
                    if self.lookup_var(var).is_none() {
                        // Collect all available names for did-you-mean suggestion
                        let mut candidates: Vec<&str> = Vec::new();
                        for scope in &self.scopes {
                            candidates.extend(scope.keys().map(|s| s.as_str()));
                        }
                        candidates.extend(self.functions.keys().map(|s| s.as_str()));
                        let suggestion = types::find_similar_name(var, candidates.into_iter());

                        return Err(TypeError::UndefinedVar {
                            name: var.clone(),
                            span: Some(expr.span),
                            suggestion,
                        });
                    }
                }

                // Validate capture mode
                match capture_mode {
                    CaptureMode::ByMutRef => {
                        // ByMutRef: all captured variables must be declared as mutable
                        for var in &free_vars {
                            if let Some((_ty, is_mut)) = self.lookup_var_with_mut(var) {
                                if !is_mut {
                                    return Err(TypeError::Mismatch {
                                        expected: "mutable variable for &mut capture".to_string(),
                                        found: format!("immutable variable '{}'", var),
                                        span: Some(expr.span),
                                    });
                                }
                            }
                        }
                    }
                    CaptureMode::ByRef | CaptureMode::Move | CaptureMode::ByValue => {
                        // ByRef: captures by immutable reference (codegen handles pointer passing)
                        // Move/ByValue: validated during codegen/MIR.
                    }
                }

                self.push_scope();

                // Define captured variables in lambda scope
                for var in &free_vars {
                    if let Some((ty, is_mut)) = self.lookup_var_with_mut(var) {
                        // ByRef captures are immutable inside the lambda —
                        // the captured variable cannot be written through an immutable reference
                        let effective_mut = match capture_mode {
                            CaptureMode::ByRef => false,
                            _ => is_mut,
                        };
                        self.define_var(var, ty, effective_mut);
                    }
                }

                // Resolve parameter types (Type::Infer will create fresh type variables)
                let mut param_types: Vec<_> = params
                    .iter()
                    .map(|p| {
                        let ty = self.resolve_type(&p.ty.node);
                        self.define_var(&p.name.node, ty.clone(), p.is_mut);
                        ty
                    })
                    .collect();

                let ret_type = self.check_expr(body)?;
                self.pop_scope();

                // Apply substitutions to inferred parameter types
                param_types = param_types
                    .into_iter()
                    .map(|ty| self.apply_substitutions(&ty))
                    .collect();

                Ok(ResolvedType::Fn {
                    params: param_types,
                    ret: Box::new(ret_type),
                    effects: None,
                })
            }

            Expr::Spawn(inner) => {
                let inner_type = self.check_expr(inner)?;
                // Spawn semantics:
                // - `spawn async_fn()` where inner is Future<T>: returns Future<T> as-is
                //   (the async function already produces a Future; spawn schedules it)
                // - `spawn expr` where inner is non-Future T: wraps in Future<T>
                //   (creates an immediately-completed task; useful for lifting sync values
                //   into async context, e.g., `spawn 42` → Future<i64>)
                // Note: Current runtime uses synchronous fallback (no green threads/coroutines).
                // The Future<T> wrapper preserves type-level async semantics for future
                // runtime upgrades (e.g., work-stealing scheduler, coroutine state machines).
                match inner_type {
                    ResolvedType::Future(_) => Ok(inner_type),
                    other => Ok(ResolvedType::Future(Box::new(other))),
                }
            }

            Expr::Yield(inner) => {
                let inner_type = self.check_expr(inner)?;
                // Yield suspends the generator and returns the yielded value to the caller.
                // The yield expression evaluates to the type of the yielded value.
                Ok(inner_type)
            }

            Expr::Lazy(inner) => {
                let inner_type = self.check_expr(inner)?;
                // lazy expr creates a Lazy<T> thunk
                Ok(ResolvedType::Lazy(Box::new(inner_type)))
            }
            Expr::Force(inner) => {
                let inner_type = self.check_expr(inner)?;
                // force expr evaluates a Lazy<T> and returns T
                match inner_type {
                    ResolvedType::Lazy(t) => Ok(*t),
                    _ => {
                        // If not a Lazy type, force is a no-op (identity)
                        Ok(inner_type)
                    }
                }
            }

            Expr::Comptime { body } => {
                // Evaluate the comptime expression at compile time
                let mut evaluator = comptime::ComptimeEvaluator::new();
                let value = evaluator.eval(body)?;

                // Return the type based on the evaluated value
                match value {
                    comptime::ComptimeValue::Int(_) => Ok(ResolvedType::I64),
                    comptime::ComptimeValue::Float(_) => Ok(ResolvedType::F64),
                    comptime::ComptimeValue::Bool(_) => Ok(ResolvedType::Bool),
                    comptime::ComptimeValue::String(_) => Ok(ResolvedType::Str),
                    comptime::ComptimeValue::Array(ref arr) => {
                        // Infer array element type from first element
                        if arr.is_empty() {
                            Ok(ResolvedType::Array(Box::new(ResolvedType::I64)))
                        } else {
                            let elem_type = match &arr[0] {
                                comptime::ComptimeValue::Int(_) => ResolvedType::I64,
                                comptime::ComptimeValue::Float(_) => ResolvedType::F64,
                                comptime::ComptimeValue::Bool(_) => ResolvedType::Bool,
                                comptime::ComptimeValue::String(_) => ResolvedType::Str,
                                _ => ResolvedType::I64,
                            };
                            Ok(ResolvedType::Array(Box::new(elem_type)))
                        }
                    }
                    comptime::ComptimeValue::Unit => Ok(ResolvedType::Unit),
                }
            }

            Expr::MacroInvoke(invoke) => {
                // Macro invocations should be expanded before type checking.
                // If we reach here, the macro was not expanded - this is an error.
                Err(TypeError::UndefinedFunction {
                    name: format!("{}!", invoke.name.node),
                    span: Some(invoke.name.span),
                    suggestion: Some(
                        "Macro invocations must be expanded before type checking".to_string(),
                    ),
                })
            }

            Expr::Old(inner) => {
                // old(expr) is used in ensures clauses to refer to pre-state values
                // The type of old(expr) is the same as expr
                // Note: Semantic checking (that this is only in ensures) is done in codegen
                self.check_expr(inner)
            }

            Expr::Assert { condition, message } => {
                // assert(condition) or assert(condition, message)
                // Condition must be bool, message (if present) must be str
                let cond_type = self.check_expr(condition)?;
                self.unify(&cond_type, &ResolvedType::Bool)?;

                if let Some(msg) = message {
                    let msg_type = self.check_expr(msg)?;
                    self.unify(&msg_type, &ResolvedType::Str)?;
                }

                // assert returns unit (or diverges on failure)
                Ok(ResolvedType::Unit)
            }

            Expr::Assume(inner) => {
                // assume(expr) tells the verifier to assume expr is true
                // Condition must be bool
                let cond_type = self.check_expr(inner)?;
                self.unify(&cond_type, &ResolvedType::Bool)?;

                // assume returns unit
                Ok(ResolvedType::Unit)
            }

            Expr::Error { .. } => {
                // Error nodes from recovery mode are treated as having Unknown type.
                // The parsing error has already been reported.
                Ok(ResolvedType::Unknown)
            }
        }
    }

    /// Check if-else branch
    pub(crate) fn check_if_else(&mut self, branch: &IfElse) -> TypeResult<ResolvedType> {
        match branch {
            IfElse::ElseIf(cond, then, else_) => {
                let cond_type = self.check_expr(cond)?;
                self.unify(&cond_type, &ResolvedType::Bool)?;

                self.push_scope();
                let then_type = self.check_block(then)?;
                self.pop_scope();

                if let Some(else_branch) = else_ {
                    let else_type = self.check_if_else(else_branch)?;
                    // If branch types don't unify, treat as statement (Unit type)
                    if self.unify(&then_type, &else_type).is_ok() {
                        return Ok(then_type);
                    }
                    return Ok(ResolvedType::Unit);
                }

                Ok(then_type)
            }
            IfElse::Else(stmts) => {
                self.push_scope();
                let result = self.check_block(stmts);
                self.pop_scope();
                result
            }
        }
    }
}
