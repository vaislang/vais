//! Function/method call expression checking

use std::collections::HashMap;
use vais_ast::*;
use crate::TypeChecker;
use crate::types::{self, GenericInstantiation, ResolvedType, TypeError, TypeResult};

impl TypeChecker {
    /// Check SelfCall expression (@)
    pub(crate) fn check_self_call(&mut self, expr_span: Span) -> TypeResult<ResolvedType> {
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
            span: Some(expr_span),
            suggestion: None,
        })
    }

    /// Check call expressions
    pub(crate) fn check_call_expr(
        &mut self,
        func: &Spanned<Expr>,
        args: &[Spanned<Expr>],
        expr_span: Span,
    ) -> TypeResult<ResolvedType> {
        // Check if this is a direct call to a known function
        if let Expr::Ident(func_name) = &func.node {
            // Struct tuple literal: `Response(200, 1)` → `Response { status: 200, body: 1 }`
            if !self.functions.contains_key(func_name) {
                if let Some(struct_def) = self.structs.get(func_name).cloned() {
                    if args.len() != struct_def.field_order.len() {
                        return Err(TypeError::ArgCount {
                            expected: struct_def.field_order.len(),
                            got: args.len(),
                            span: Some(expr_span),
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
                        expr_span,
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
                        span: Some(expr_span),
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
                        span: Some(expr_span),
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

    /// Check method call expressions
    pub(crate) fn check_method_call(
        &mut self,
        receiver: &Spanned<Expr>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        expr_span: Span,
    ) -> TypeResult<ResolvedType> {
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
                        span: Some(expr_span),
                    });
                }

                // Build substitution map from type's generic params to receiver's concrete types
                let generic_substitutions: HashMap<String, ResolvedType> =
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
                    span: Some(expr_span),
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
                            span: Some(expr_span),
                        });
                    }
                    return Ok(ResolvedType::I64);
                }
                "charAt" => {
                    if args.len() != 1 {
                        return Err(TypeError::ArgCount {
                            expected: 1,
                            got: args.len(),
                            span: Some(expr_span),
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
                            span: Some(expr_span),
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
                            span: Some(expr_span),
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
                            span: Some(expr_span),
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
                            span: Some(expr_span),
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
                    span: Some(expr_span),
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

    /// Check static method call expressions
    pub(crate) fn check_static_method_call(
        &mut self,
        type_name: &Spanned<String>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        expr_span: Span,
    ) -> TypeResult<ResolvedType> {
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
                        span: Some(expr_span),
                    });
                }

                // Handle generic struct type inference
                if !struct_def.generics.is_empty() {
                    // Create fresh type variables for each struct generic parameter
                    let generic_substitutions: HashMap<String, ResolvedType> = struct_def
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
}
