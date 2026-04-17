//! Function/method call expression checking

use crate::types::{self, GenericInstantiation, ResolvedType, TypeError, TypeResult};
use crate::TypeChecker;
use std::collections::HashMap;
use vais_ast::*;

impl TypeChecker {
    /// Check SelfCall expression (@)
    #[inline]
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
            // Enum variant constructor: Ok(value), Err(err), Some(value)
            if !self.functions.contains_key(func_name) && !self.structs.contains_key(func_name) {
                // Search all enums for a variant with this name
                let found_enum = self
                    .enums
                    .iter()
                    .find(|(_, def)| def.variants.contains_key(func_name))
                    .map(|(name, def)| (name.clone(), def.clone()));
                if let Some((enum_name, enum_def)) = found_enum {
                    if let Some(variant_fields) = enum_def.variants.get(func_name) {
                        match variant_fields {
                            crate::types::VariantFieldTypes::Tuple(field_types) => {
                                if args.len() == field_types.len() {
                                    for (arg, expected_ty) in args.iter().zip(field_types.iter()) {
                                        let arg_ty = self.check_expr(arg)?;
                                        self.unify(expected_ty, &arg_ty)?;
                                    }
                                }
                            }
                            crate::types::VariantFieldTypes::Unit => {}
                            _ => {}
                        }
                    }
                    return Ok(ResolvedType::Named {
                        name: enum_name,
                        generics: enum_def
                            .generics
                            .iter()
                            .map(|_| self.fresh_type_var())
                            .collect(),
                    });
                }
            }

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
                            enum_name: None,
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
                    // Track struct-typed variables passed by value as moved
                    // Skip if parameter type is a reference (&T or &mut T)
                    if i < sig.params.len() {
                        let param_ty = self.apply_substitutions(&sig.params[i].1);
                        let is_ref =
                            matches!(&param_ty, ResolvedType::Ref(_) | ResolvedType::RefMut(_));
                        if !is_ref {
                            if let ResolvedType::Named {
                                name: ref type_name,
                                ..
                            } = param_ty
                            {
                                if self.structs.contains_key(type_name.as_str()) {
                                    if let Expr::Ident(var_name) = &arg.node {
                                        self.moved_vars.insert(var_name.clone());
                                    }
                                }
                            }
                        }
                    }
                    if i < sig.params.len() {
                        // Str ↔ I64 coercion for function calls: Vais represents str as i64
                        // (pointer to string data) at the IR level. The std library declares
                        // str parameters as i64 (e.g., strlen, starts_with, assert_eq).
                        // Allow str variables/expressions to be passed to i64 parameters
                        // (and i64 to str parameters) during function calls, maintaining
                        // IR compatibility. String LITERALS passed to i64 params are still
                        // errors (user likely intended a different type).
                        let param_resolved = self.apply_substitutions(&sig.params[i].1);
                        let arg_resolved = self.apply_substitutions(&arg_type);
                        let is_str_literal =
                            matches!(&arg.node, Expr::String(_) | Expr::StringInterp(_));
                        let is_str_i64_coercion = !is_str_literal
                            && matches!(
                                (&param_resolved, &arg_resolved),
                                (ResolvedType::Str, ResolvedType::I64)
                                    | (ResolvedType::I64, ResolvedType::Str)
                            );
                        // Implicit error propagation (Phase 4b.1 / #7).
                        //
                        // Only try when opt-in is on. The helper is a no-op when
                        // `self.implicit_try_mode == false` so the legacy unify path
                        // below is unchanged for existing code.
                        //
                        // Skip when the argument is already an explicit Try — the
                        // user wrote `?` manually and `Expr::Try` has already
                        // been resolved by `check_expr`, producing the unwrapped
                        // inner type, so there is nothing to do here.
                        let already_try = matches!(&arg.node, Expr::Try(_));
                        let propagated = if !already_try {
                            self.try_implicit_error_propagation(
                                &sig.params[i].1,
                                &arg_type,
                                arg.span,
                                is_str_i64_coercion,
                            )?
                        } else {
                            None
                        };
                        if propagated.is_none() && !is_str_i64_coercion {
                            self.unify(&sig.params[i].1, &arg_type)?;
                        }
                        // Check dependent type refinement for literal arguments
                        if let ResolvedType::Dependent {
                            var_name,
                            predicate,
                            base,
                            ..
                        } = &sig.params[i].1
                        {
                            if base.is_float() {
                                // f64/f32 dependent type: check float literals
                                if let Some(lit_val) =
                                    Self::extract_float_literal_from_expr(&arg.node)
                                {
                                    self.check_refinement_from_string_f64(
                                        var_name,
                                        predicate,
                                        lit_val,
                                        Some(arg.span),
                                    )?;
                                }
                            } else if let Some(lit_val) =
                                Self::extract_integer_literal_from_expr(&arg.node)
                            {
                                self.check_refinement_from_string(
                                    var_name,
                                    predicate,
                                    lit_val,
                                    Some(arg.span),
                                )?;
                            }
                        }
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
            ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
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

        // Extract the inner type if receiver is a reference or pointer (auto-deref)
        let (inner_type, receiver_generics) = match &receiver_type {
            ResolvedType::Named { name, generics } => (name.clone(), generics.clone()),
            ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Pointer(inner) => {
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
                let mut generic_substitutions: HashMap<String, ResolvedType> = found_generics
                    .iter()
                    .zip(receiver_generics.iter())
                    .map(|(param, arg)| (param.clone(), arg.clone()))
                    .collect();

                // Also create fresh type variables for method-level generics
                // that are not already covered by struct-level generics
                for method_generic in &method_sig.generics {
                    if !generic_substitutions.contains_key(method_generic) {
                        generic_substitutions.insert(method_generic.clone(), self.fresh_type_var());
                    }
                }

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

                // Resolve any lingering type variables (e.g. receiver Vec<?N>
                // whose ?N was later unified with a concrete type by arg checks).
                // Without this, Option<T> returns can stay as Option<?N> and
                // match-arm pattern bindings fail to propagate T into sub-patterns.
                let ret_type_raw = self.apply_substitutions(&ret_type_raw);

                // For async methods, wrap the return type in Future
                let ret_type = if method_sig.is_async {
                    ResolvedType::Future(Box::new(ret_type_raw))
                } else {
                    ret_type_raw
                };

                // Record generic instantiation for monomorphization
                if !receiver_generics.is_empty() {
                    let inferred_type_args: Vec<_> = receiver_generics
                        .iter()
                        .map(|t| self.apply_substitutions(t))
                        .collect();
                    let all_concrete = inferred_type_args
                        .iter()
                        .all(|t| !matches!(t, ResolvedType::Var(_) | ResolvedType::Generic(_)));
                    if all_concrete {
                        // Record struct instantiation (e.g., Vec<i64>)
                        let struct_inst = crate::types::GenericInstantiation::struct_type(
                            &inner_type,
                            inferred_type_args.clone(),
                        );
                        self.add_instantiation(struct_inst);
                        // Record method instantiation (e.g., Vec_push<i64>)
                        let method_inst = crate::types::GenericInstantiation::method(
                            &inner_type,
                            &method.node,
                            inferred_type_args,
                        );
                        self.add_instantiation(method_inst);
                    }
                }

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
                "push_str" => {
                    if args.len() != 1 {
                        return Err(TypeError::ArgCount {
                            expected: 1,
                            got: args.len(),
                            span: Some(expr_span),
                        });
                    }
                    let arg_type = self.check_expr(&args[0])?;
                    self.unify(&ResolvedType::Str, &arg_type)?;
                    return Ok(ResolvedType::Str);
                }
                "as_bytes" => {
                    if !args.is_empty() {
                        return Err(TypeError::ArgCount {
                            expected: 0,
                            got: args.len(),
                            span: Some(expr_span),
                        });
                    }
                    // Returns raw byte pointer (i8* as i64)
                    return Ok(ResolvedType::I64);
                }
                "clone" | "to_string" | "as_str" => {
                    if !args.is_empty() {
                        return Err(TypeError::ArgCount {
                            expected: 0,
                            got: args.len(),
                            span: Some(expr_span),
                        });
                    }
                    return Ok(ResolvedType::Str);
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

        // H5-H6 builtin Vec/HashMap/ByteBuffer/Mutex methods removed
        // C1 fix enables proper impl lookup from std modules

        // Phase 24 Task 5: .iter() / .enumerate() builtin for iterable receivers
        // Matches any type whose item type is derivable via get_iterator_item_type
        // (Vec<T>, Array<T>, Slice<T>, Range<T>, ConstArray, etc.).
        //
        // .iter() is a no-op at the type level — it returns the receiver unchanged,
        // so the for-each loop checker keeps extracting the element type the same way.
        // .enumerate() returns a virtual EnumerateIter<T> whose item type is (i64, T);
        // see lookup.rs get_iterator_item_type_inner for the binding. Codegen
        // (Task 6) recognizes these method calls in for-each loops and desugars
        // them to index-based iteration.
        if args.is_empty() && (method.node == "iter" || method.node == "enumerate") {
            if let Some(elem_ty) = self.get_iterator_item_type(&receiver_type) {
                // Skip EnumerateIter here — if the receiver is already EnumerateIter<T>,
                // elem_ty is already (i64, T) and we must NOT wrap it again.
                let is_enumerate_iter = matches!(
                    &receiver_type,
                    ResolvedType::Named { name, .. } if name == "EnumerateIter"
                );
                if method.node == "iter" {
                    // .iter() is idempotent at the type level.
                    return Ok(receiver_type.clone());
                }
                // .enumerate()
                if is_enumerate_iter {
                    // Already an EnumerateIter — chaining is a no-op.
                    return Ok(receiver_type.clone());
                }
                return Ok(ResolvedType::Named {
                    name: "EnumerateIter".to_string(),
                    generics: vec![elem_ty],
                });
            }
        }

        // Minimal builtin fallbacks (kept because VaisDB structs lack explicit impl blocks for these)
        // clone: identity-copy semantics for any struct
        if method.node == "clone" && args.is_empty() {
            return Ok(receiver_type.clone());
        }
        // serialize: writes to ByteBuffer, returns unit
        if method.node == "serialize" {
            for arg in args {
                let _ = self.check_expr(arg)?;
            }
            return Ok(ResolvedType::Unit);
        }
        // deserialize: reads from ByteBuffer, returns Result<Self, VaisError>
        if method.node == "deserialize" {
            for arg in args {
                let _ = self.check_expr(arg)?;
            }
            return Ok(ResolvedType::Result(
                Box::new(receiver_type.clone()),
                Box::new(ResolvedType::Named {
                    name: "VaisError".to_string(),
                    generics: vec![],
                }),
            ));
        }

        // Try to find similar method names for suggestion
        let suggestion =
            types::find_similar_name(&method.node, self.functions.keys().map(|s| s.as_str()));
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
                            let ty = generic_substitutions.get(param).unwrap_or_else(|| {
                                // ICE: generic parameter missing from substitutions
                                eprintln!(
                                    "ICE: generic parameter '{}' missing from substitutions map",
                                    param
                                );
                                &ResolvedType::Unknown
                            });
                            self.apply_substitutions(ty)
                        })
                        .collect();

                    let all_concrete = inferred_type_args
                        .iter()
                        .all(|t| !matches!(t, ResolvedType::Var(_) | ResolvedType::Generic(_)));
                    if all_concrete {
                        let inst = GenericInstantiation::struct_type(
                            &type_name.node,
                            inferred_type_args.clone(),
                        );
                        self.add_instantiation(inst);

                        // Also record method instantiation for cross-module specialization.
                        // This ensures that Vec_push$Point, Vec_with_capacity$Point, etc.
                        // are generated when Vec<Point> methods are called from another module.
                        let method_inst = GenericInstantiation::method(
                            &type_name.node,
                            &method.node,
                            inferred_type_args,
                        );
                        self.add_instantiation(method_inst);
                    } else {
                        // Phase 193 R-1b: type args still contain free vars (e.g., a
                        // static ctor like `Vec.with_capacity(n)` whose element type
                        // has not yet been constrained by this expression alone).
                        // Defer instantiation until after function body unification
                        // resolves the vars against the declared return type.
                        self.pending_method_instantiations.push((
                            type_name.node.clone(),
                            method.node.clone(),
                            inferred_type_args,
                        ));
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

        // Built-in Vec static methods
        if type_name.node == "Vec" {
            match method.node.as_str() {
                "new" => {
                    let elem_type = self.fresh_type_var();
                    return Ok(ResolvedType::Named {
                        name: "Vec".to_string(),
                        generics: vec![elem_type],
                    });
                }
                "with_capacity" => {
                    if args.len() == 1 {
                        let arg_type = self.check_expr(&args[0])?;
                        let _ = self.unify(&ResolvedType::I64, &arg_type);
                    }
                    let elem_type = self.fresh_type_var();
                    return Ok(ResolvedType::Named {
                        name: "Vec".to_string(),
                        generics: vec![elem_type],
                    });
                }
                _ => {}
            }
        }

        // Built-in HashMap static methods
        if type_name.node == "HashMap" {
            match method.node.as_str() {
                "new" | "with_capacity" => {
                    for arg in args {
                        let _ = self.check_expr(arg)?;
                    }
                    return Ok(ResolvedType::Named {
                        name: "HashMap".to_string(),
                        generics: vec![self.fresh_type_var(), self.fresh_type_var()],
                    });
                }
                _ => {}
            }
        }

        // Built-in Mutex static methods
        if type_name.node == "Mutex" && method.node == "new" {
            for arg in args {
                let _ = self.check_expr(arg)?;
            }
            return Ok(ResolvedType::Named {
                name: "Mutex".to_string(),
                generics: vec![self.fresh_type_var()],
            });
        }

        // Fallback: for unknown types, if method returns Self type
        if method.node == "new"
            || method.node.starts_with("create")
            || method.node.starts_with("from")
            || method.node == "clone"
            || method.node == "default"
        {
            for arg in args {
                let _ = self.check_expr(arg)?;
            }
            return Ok(ResolvedType::Named {
                name: type_name.node.clone(),
                generics: vec![],
            });
        }
        // Methods that return Result<Self, VaisError>
        if method.node == "deserialize"
            || method.node == "load"
            || method.node == "read"
            || method.node == "parse"
            || method.node == "read_from_page"
        {
            for arg in args {
                let _ = self.check_expr(arg)?;
            }
            let self_type = ResolvedType::Named {
                name: type_name.node.clone(),
                generics: vec![],
            };
            return Ok(ResolvedType::Result(
                Box::new(self_type),
                Box::new(ResolvedType::Named {
                    name: "VaisError".to_string(),
                    generics: vec![],
                }),
            ));
        }
        // For other static methods on unknown types, check args and return i64
        for arg in args {
            let _ = self.check_expr(arg)?;
        }
        return Ok(ResolvedType::I64);

        // Get struct methods for suggestion if available (unreachable now but kept for reference)
        #[allow(unreachable_code)]
        let suggestion = if let Some(struct_def) = self.structs.get(&type_name.node) {
            types::find_similar_name(&method.node, struct_def.methods.keys().map(|s| s.as_str()))
        } else {
            None
        };
        Err(TypeError::UndefinedFunction {
            name: format!("{}::{}", type_name.node, method.node),
            span: Some(method.span),
            suggestion,
        })
    }

    /// Implicit error propagation (Phase 4b.1 / #7).
    ///
    /// Applied at call-site argument type checking when
    /// `self.implicit_try_mode == true`. Detects the pattern
    /// `param_ty = T` while `arg_ty = Result<T, E>` (or `Option<T>`) and,
    /// when valid, unwraps `arg_ty` to its inner success type — mirroring
    /// what an explicit `?` on the argument would do.
    ///
    /// Returns:
    /// - `Ok(Some(inner))` — unwrap succeeded, caller should use `inner` in
    ///   place of `arg_ty` and the site has been recorded for codegen.
    /// - `Ok(None)` — no implicit-try transformation is applicable; the
    ///   caller should continue with the existing unification path.
    /// - `Err(TypeError)` — the transformation was applicable but the
    ///   enclosing function does not return a compatible `Result`/`Option`
    ///   so propagation would be unsound. The caller must surface the error
    ///   rather than fall through to regular unification.
    ///
    /// Only the direct `Expr::Call` path uses this today. Method calls and
    /// indirect calls fall through unchanged, which preserves existing
    /// behavior for code that does not opt in.
    pub(crate) fn try_implicit_error_propagation(
        &mut self,
        param_ty: &ResolvedType,
        arg_ty: &ResolvedType,
        arg_span: Span,
        is_str_i64_coercion: bool,
    ) -> TypeResult<Option<ResolvedType>> {
        if !self.implicit_try_mode {
            return Ok(None);
        }
        // Never trigger for the legacy str↔i64 coercion — that path has its
        // own unify skip and is orthogonal to error propagation.
        if is_str_i64_coercion {
            return Ok(None);
        }
        let param_resolved = self.apply_substitutions(param_ty);
        let arg_resolved = self.apply_substitutions(arg_ty);
        // If the parameter itself is Result/Option, the user is passing the
        // whole container deliberately — do not unwrap.
        if matches!(
            &param_resolved,
            ResolvedType::Result(_, _) | ResolvedType::Optional(_)
        ) {
            return Ok(None);
        }
        // Named Result<T,E> / Option<T> on the parameter side is also a
        // deliberate pass-through.
        if let ResolvedType::Named { name, .. } = &param_resolved {
            if name == "Result" || name == "Option" {
                return Ok(None);
            }
        }

        // Extract (inner_ok, err_opt, is_option) from the argument type.
        let unwrap = |ty: &ResolvedType| -> Option<(ResolvedType, Option<ResolvedType>, bool)> {
            match ty {
                ResolvedType::Result(ok, err) => {
                    Some(((**ok).clone(), Some((**err).clone()), false))
                }
                ResolvedType::Optional(ok) => Some(((**ok).clone(), None, true)),
                ResolvedType::Named { name, generics } if name == "Result" => {
                    let ok = generics.first().cloned().unwrap_or(ResolvedType::I64);
                    let err = generics.get(1).cloned();
                    Some((ok, err, false))
                }
                ResolvedType::Named { name, generics } if name == "Option" => {
                    let ok = generics.first().cloned().unwrap_or(ResolvedType::I64);
                    Some((ok, None, true))
                }
                _ => None,
            }
        };
        let Some((inner_ok, arg_err, is_option)) = unwrap(&arg_resolved) else {
            return Ok(None);
        };

        // Cheap compatibility probe: does the inner success type unify with
        // the parameter? Perform this on a checkpoint so a failure here does
        // not leak stray substitutions into later checking.
        let snapshot = self.substitutions.clone();
        let unifies = self.unify(param_ty, &inner_ok).is_ok();
        if !unifies {
            self.substitutions = snapshot;
            return Ok(None);
        }
        // Commit the unification — caller will use `inner_ok` as the arg
        // type but we must not re-unify, so we record the success here and
        // leave `self.substitutions` as-is.

        // Enclosing-function compatibility: propagation requires the caller
        // to return Result/Option that matches the argument's container.
        let Some(current_ret) = self.current_fn_ret.clone() else {
            self.substitutions = snapshot;
            return Err(TypeError::Mismatch {
                expected: "enclosing function returning Result or Option \
                           (required by --implicit-try)"
                    .to_string(),
                found: "no function context".to_string(),
                span: Some(arg_span),
            });
        };
        let current_resolved = self.apply_substitutions(&current_ret);
        let ret_unwrap = unwrap(&current_resolved);
        let (ret_err_opt, ret_is_option) = match ret_unwrap {
            Some((_, err, is_opt)) => (err, is_opt),
            None => {
                self.substitutions = snapshot;
                return Err(TypeError::Mismatch {
                    expected: if is_option {
                        "Option<_> return type (required by --implicit-try \
                         when propagating Option)"
                            .to_string()
                    } else {
                        "Result<_, _> return type (required by --implicit-try \
                         when propagating Result)"
                            .to_string()
                    },
                    found: current_resolved.to_string(),
                    span: Some(arg_span),
                });
            }
        };
        if is_option != ret_is_option {
            // Result → Option or vice versa: not directly propagatable.
            self.substitutions = snapshot;
            return Err(TypeError::Mismatch {
                expected: if is_option {
                    "Option<_> return type".to_string()
                } else {
                    "Result<_, _> return type".to_string()
                },
                found: current_resolved.to_string(),
                span: Some(arg_span),
            });
        }
        // For Result, err types must unify so the propagation is type-safe.
        if let (Some(arg_err_ty), Some(ret_err_ty)) = (arg_err.as_ref(), ret_err_opt.as_ref()) {
            if self.unify(ret_err_ty, arg_err_ty).is_err() {
                self.substitutions = snapshot;
                return Err(TypeError::Mismatch {
                    expected: format!(
                        "Result<_, {}> (to match argument's error type)",
                        arg_err_ty
                    ),
                    found: current_resolved.to_string(),
                    span: Some(arg_span),
                });
            }
        }

        // All compatibility checks passed — record the site and return the
        // unwrapped inner type. The caller uses this as the effective
        // argument type; codegen will consult `is_implicit_try_site` to wrap
        // the original argument in Try semantics.
        self.implicit_try_sites
            .insert((arg_span.start, arg_span.end));
        Ok(Some(inner_ok))
    }
}
