//! Special expression types (try, unwrap, cast, assign, lambda, comptime, etc.)

use crate::comptime;
use crate::types::{self, ResolvedType, TypeError, TypeResult, VariantFieldTypes};
use crate::TypeChecker;
use vais_ast::*;

impl TypeChecker {
    pub(crate) fn check_special_expr(
        &mut self,
        expr: &Spanned<Expr>,
    ) -> Option<TypeResult<ResolvedType>> {
        match &expr.node {
            Expr::Try(inner) => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                // Try operator (?) works on both Result<T> and Option<T>
                // - Result<T>: returns T on Ok, propagates Err
                // - Option<T>: returns T on Some, propagates None
                match &inner_type {
                    ResolvedType::Result(ok_type, _err_type) => Some(Ok(*ok_type.clone())),
                    ResolvedType::Optional(some_type) => Some(Ok(*some_type.clone())),
                    // Also support user-defined enums named "Result" with Ok variant
                    ResolvedType::Named { name, .. } if name == "Result" => {
                        if let Some(enum_def) = self.enums.get("Result") {
                            if let Some(variant_fields) = enum_def.variants.get("Ok") {
                                match variant_fields {
                                    VariantFieldTypes::Tuple(types) if !types.is_empty() => {
                                        Some(Ok(types[0].clone()))
                                    }
                                    _ => Some(Ok(ResolvedType::I64)),
                                }
                            } else {
                                Some(Ok(ResolvedType::I64))
                            }
                        } else {
                            Some(Ok(ResolvedType::I64))
                        }
                    }
                    // Also support user-defined enums named "Option" with Some variant
                    ResolvedType::Named { name, .. } if name == "Option" => {
                        if let Some(enum_def) = self.enums.get("Option") {
                            if let Some(variant_fields) = enum_def.variants.get("Some") {
                                match variant_fields {
                                    VariantFieldTypes::Tuple(types) if !types.is_empty() => {
                                        Some(Ok(types[0].clone()))
                                    }
                                    _ => Some(Ok(ResolvedType::I64)),
                                }
                            } else {
                                Some(Ok(ResolvedType::I64))
                            }
                        } else {
                            Some(Ok(ResolvedType::I64))
                        }
                    }
                    _ => Some(Err(TypeError::Mismatch {
                        expected: "Result or Option type".to_string(),
                        found: inner_type.to_string(),
                        span: Some(inner.span),
                    })),
                }
            }

            Expr::Unwrap(inner) => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                match &inner_type {
                    ResolvedType::Optional(inner) => Some(Ok(*inner.clone())),
                    ResolvedType::Result(ok, _err) => Some(Ok(*ok.clone())),
                    // Support user-defined Result/Option enums
                    ResolvedType::Named { name, .. } if name == "Result" => {
                        if let Some(enum_def) = self.enums.get("Result") {
                            if let Some(VariantFieldTypes::Tuple(types)) =
                                enum_def.variants.get("Ok")
                            {
                                if !types.is_empty() {
                                    Some(Ok(types[0].clone()))
                                } else {
                                    Some(Ok(ResolvedType::I64))
                                }
                            } else {
                                Some(Ok(ResolvedType::I64))
                            }
                        } else {
                            Some(Ok(ResolvedType::I64))
                        }
                    }
                    ResolvedType::Named { name, .. } if name == "Option" => {
                        if let Some(enum_def) = self.enums.get("Option") {
                            if let Some(VariantFieldTypes::Tuple(types)) =
                                enum_def.variants.get("Some")
                            {
                                if !types.is_empty() {
                                    Some(Ok(types[0].clone()))
                                } else {
                                    Some(Ok(ResolvedType::I64))
                                }
                            } else {
                                Some(Ok(ResolvedType::I64))
                            }
                        } else {
                            Some(Ok(ResolvedType::I64))
                        }
                    }
                    _ => Some(Err(TypeError::Mismatch {
                        expected: "Optional or Result".to_string(),
                        found: inner_type.to_string(),
                        span: Some(inner.span),
                    })),
                }
            }

            Expr::Cast { expr: inner, ty } => {
                // Check the expression
                let _expr_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                // Resolve the target type
                let target_type = self.resolve_type(&ty.node);
                // For now, allow all casts - runtime will handle invalid ones
                Some(Ok(target_type))
            }

            Expr::Assign { target, value } => {
                // Allow assignment to all variables (mutable by default in Vais)
                let target_type = match self.check_expr(target) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let value_type = match self.check_expr(value) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                if let Err(e) = self.unify(&target_type, &value_type) {
                    return Some(Err(e));
                }
                Some(Ok(ResolvedType::Unit))
            }

            Expr::AssignOp {
                op: _,
                target,
                value,
            } => {
                let target_type = match self.check_expr(target) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let value_type = match self.check_expr(value) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                if let Err(e) = self.unify(&target_type, &value_type) {
                    return Some(Err(e));
                }
                Some(Ok(ResolvedType::Unit))
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

                        return Some(Err(TypeError::UndefinedVar {
                            name: var.clone(),
                            span: Some(expr.span),
                            suggestion,
                        }));
                    }
                }

                // Validate capture mode
                match capture_mode {
                    CaptureMode::ByMutRef => {
                        // ByMutRef: all captured variables must be declared as mutable
                        for var in &free_vars {
                            if let Some((_ty, is_mut)) = self.lookup_var_with_mut(var) {
                                if !is_mut {
                                    return Some(Err(TypeError::Mismatch {
                                        expected: "mutable variable for &mut capture".to_string(),
                                        found: format!("immutable variable '{}'", var),
                                        span: Some(expr.span),
                                    }));
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

                let ret_type = match self.check_expr(body) {
                    Ok(t) => t,
                    Err(e) => {
                        self.pop_scope();
                        return Some(Err(e));
                    }
                };
                self.pop_scope();

                // Apply substitutions to inferred parameter types
                param_types = param_types
                    .into_iter()
                    .map(|ty| self.apply_substitutions(&ty))
                    .collect();

                Some(Ok(ResolvedType::Fn {
                    params: param_types,
                    ret: Box::new(ret_type),
                    effects: None,
                }))
            }

            Expr::Comptime { body } => {
                // Evaluate the comptime expression at compile time
                let mut evaluator = comptime::ComptimeEvaluator::new();
                let value = match evaluator.eval(body) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };

                // Return the type based on the evaluated value
                match value {
                    comptime::ComptimeValue::Int(_) => Some(Ok(ResolvedType::I64)),
                    comptime::ComptimeValue::Float(_) => Some(Ok(ResolvedType::F64)),
                    comptime::ComptimeValue::Bool(_) => Some(Ok(ResolvedType::Bool)),
                    comptime::ComptimeValue::String(_) => Some(Ok(ResolvedType::Str)),
                    comptime::ComptimeValue::Array(ref arr) => {
                        // Infer array element type from first element
                        if arr.is_empty() {
                            Some(Ok(ResolvedType::Array(Box::new(ResolvedType::I64))))
                        } else {
                            let elem_type = match &arr[0] {
                                comptime::ComptimeValue::Int(_) => ResolvedType::I64,
                                comptime::ComptimeValue::Float(_) => ResolvedType::F64,
                                comptime::ComptimeValue::Bool(_) => ResolvedType::Bool,
                                comptime::ComptimeValue::String(_) => ResolvedType::Str,
                                _ => ResolvedType::I64,
                            };
                            Some(Ok(ResolvedType::Array(Box::new(elem_type))))
                        }
                    }
                    comptime::ComptimeValue::Unit => Some(Ok(ResolvedType::Unit)),
                }
            }

            Expr::MacroInvoke(invoke) => {
                // Macro invocations should be expanded before type checking.
                // If we reach here, the macro was not expanded - this is an error.
                Some(Err(TypeError::UndefinedFunction {
                    name: format!("{}!", invoke.name.node),
                    span: Some(invoke.name.span),
                    suggestion: Some(
                        "Macro invocations must be expanded before type checking".to_string(),
                    ),
                }))
            }

            Expr::Old(inner) => {
                // old(expr) is used in ensures clauses to refer to pre-state values
                // The type of old(expr) is the same as expr
                // Note: Semantic checking (that this is only in ensures) is done in codegen
                match self.check_expr(inner) {
                    Ok(t) => Some(Ok(t)),
                    Err(e) => Some(Err(e)),
                }
            }

            Expr::Assert { condition, message } => {
                // assert(condition) or assert(condition, message)
                // Condition must be bool, message (if present) must be str
                let cond_type = match self.check_expr(condition) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                if let Err(e) = self.unify(&cond_type, &ResolvedType::Bool) {
                    return Some(Err(e));
                }

                if let Some(msg) = message {
                    let msg_type = match self.check_expr(msg) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    if let Err(e) = self.unify(&msg_type, &ResolvedType::Str) {
                        return Some(Err(e));
                    }
                }

                // assert returns unit (or diverges on failure)
                Some(Ok(ResolvedType::Unit))
            }

            Expr::Assume(inner) => {
                // assume(expr) tells the verifier to assume expr is true
                // Condition must be bool
                let cond_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                if let Err(e) = self.unify(&cond_type, &ResolvedType::Bool) {
                    return Some(Err(e));
                }

                // assume returns unit
                Some(Ok(ResolvedType::Unit))
            }

            Expr::Error { .. } => {
                // Error nodes from recovery mode are treated as having Unknown type.
                // The parsing error has already been reported.
                Some(Ok(ResolvedType::Unknown))
            }

            _ => None,
        }
    }
}
