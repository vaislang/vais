//! Function and method body type checking.

use vais_ast::*;

use super::TypeChecker;
use crate::lifetime;
use crate::types::{ResolvedType, TypeError, TypeResult};

impl TypeChecker {
    /// Check a function body
    pub(crate) fn check_function(&mut self, f: &Function) -> TypeResult<()> {
        self.push_scope();

        // Set current generic parameters
        let (prev_generics, prev_bounds, prev_const_generics) = self.set_generics(&f.generics);

        // Add parameters to scope and validate object safety
        // Use types from the registered FunctionSig to share type variables
        // (important for Type::Infer parameters that become Var(id))
        let registered_param_types: Vec<_> = self
            .functions
            .get(&f.name.node)
            .map(|sig| sig.params.iter().map(|(_, ty, _)| ty.clone()).collect())
            .unwrap_or_default();
        for (i, param) in f.params.iter().enumerate() {
            let ty = if i < registered_param_types.len() {
                registered_param_types[i].clone()
            } else {
                self.resolve_type(&param.ty.node)
            };
            self.validate_dyn_trait_object_safety(&ty);
            self.define_var(&param.name.node, ty, param.is_mut);
        }

        // Set current function context
        let ret_type_inferred = f.ret_type.is_none();
        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.resolve_type(&t.node))
            .unwrap_or_else(|| self.fresh_type_var());
        self.validate_dyn_trait_object_safety(&ret_type);
        self.current_fn_ret = Some(ret_type.clone());
        self.current_fn_name = Some(f.name.node.clone());

        // Type check requires clauses (preconditions)
        // These can only reference function parameters
        for attr in &f.attributes {
            if attr.name == "requires" {
                if let Some(expr) = &attr.expr {
                    let expr_type = self.check_expr(expr)?;
                    if expr_type != ResolvedType::Bool {
                        return Err(TypeError::Mismatch {
                            expected: "bool".to_string(),
                            found: expr_type.to_string(),
                            span: Some(expr.span),
                        });
                    }
                }
            }
        }

        // Check body
        let body_type = match &f.body {
            FunctionBody::Expr(expr) => self.check_expr(expr)?,
            FunctionBody::Block(stmts) => self.check_block(stmts)?,
        };

        // Check return type (with auto-deref: &T unifies with T)
        let expected_ret = self.current_fn_ret.clone().expect(
            "Internal compiler error: current_fn_ret should be set during function checking",
        );
        let body_type_deref = if let ResolvedType::Ref(inner) = &body_type {
            if self.unify(&expected_ret, inner).is_ok() {
                *inner.clone()
            } else {
                body_type.clone()
            }
        } else {
            body_type.clone()
        };
        self.unify(&expected_ret, &body_type_deref)?;

        // Resolve inferred return type: if return type was omitted, apply substitutions
        // to resolve the type variable to the concrete type from the body.
        if ret_type_inferred {
            let resolved_ret = self.apply_substitutions(&ret_type);
            let final_ret = if matches!(resolved_ret, ResolvedType::Var(_)) {
                // Unresolved: default to i64 for numeric expressions
                ResolvedType::I64
            } else {
                resolved_ret
            };
            if let Some(sig) = self.functions.get_mut(&f.name.node) {
                sig.ret = final_ret;
            }
        }

        // Resolve inferred parameter types after body type checking.
        // Parameters declared without type annotations (Type::Infer) get Var(id) types
        // which may have been resolved through unification during body checking.
        // Update the FunctionSig with the resolved types, defaulting unresolved vars to i64.
        if f.params.iter().any(|p| matches!(p.ty.node, Type::Infer)) {
            let resolved_params: Vec<_> = {
                let sig = self.functions.get(&f.name.node);
                if let Some(sig) = sig {
                    sig.params
                        .iter()
                        .map(|(name, ty, is_mut)| {
                            let resolved = self.apply_substitutions(ty);
                            let final_ty = if matches!(resolved, ResolvedType::Var(_)) {
                                // Unresolved type variable: default to i64
                                ResolvedType::I64
                            } else {
                                resolved
                            };
                            (name.clone(), final_ty, *is_mut)
                        })
                        .collect()
                } else {
                    vec![]
                }
            };
            if let Some(sig) = self.functions.get_mut(&f.name.node) {
                sig.params = resolved_params;
            }
        }

        // Type check ensures clauses (postconditions)
        // Add 'return' variable to scope for ensures expressions
        self.define_var("return", ret_type.clone(), false);

        for attr in &f.attributes {
            if attr.name == "ensures" {
                if let Some(expr) = &attr.expr {
                    let expr_type = self.check_expr(expr)?;
                    if expr_type != ResolvedType::Bool {
                        return Err(TypeError::Mismatch {
                            expected: "bool".to_string(),
                            found: expr_type.to_string(),
                            span: Some(expr.span),
                        });
                    }
                }
            }
        }

        // Extract and store contracts in function signature
        let contracts = self.extract_contracts(f)?;
        if let Some(func_sig) = self.functions.get_mut(&f.name.node) {
            func_sig.contracts = contracts;
        }

        // Lifetime inference: check reference lifetimes in the function signature
        self.check_function_lifetimes(f)?;

        // Check for unused local variables (excluding function parameters) and emit warnings
        let param_names: Vec<String> = f.params.iter().map(|p| p.name.node.clone()).collect();
        self.check_unused_variables(&param_names);

        self.current_fn_ret = None;
        self.current_fn_name = None;
        self.restore_generics(prev_generics, prev_bounds, prev_const_generics);
        self.pop_scope();

        Ok(())
    }

    /// Run lifetime inference on a function's signature
    pub(crate) fn check_function_lifetimes(&mut self, f: &Function) -> TypeResult<()> {
        // Reset the lifetime inferencer for this function
        self.lifetime_inferencer.reset();

        // Build parameter list with resolved types
        let params: Vec<(String, ResolvedType, bool)> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.resolve_type(&p.ty.node);
                (p.name.node.clone(), ty, p.is_mut)
            })
            .collect();

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.resolve_type(&t.node))
            .unwrap_or(ResolvedType::Unit);

        // Check if the function has any reference types at all
        let has_ref_params = params.iter().any(|(_, ty, _)| {
            matches!(
                ty,
                ResolvedType::Ref(_)
                    | ResolvedType::RefMut(_)
                    | ResolvedType::RefLifetime { .. }
                    | ResolvedType::RefMutLifetime { .. }
            )
        });
        let has_ref_return = matches!(
            ret_type,
            ResolvedType::Ref(_)
                | ResolvedType::RefMut(_)
                | ResolvedType::RefLifetime { .. }
                | ResolvedType::RefMutLifetime { .. }
        );

        // Only run lifetime inference if there are references
        if !has_ref_params && !has_ref_return {
            return Ok(());
        }

        // Extract lifetime parameters and bounds from generics
        let lifetime_params = lifetime::LifetimeInferencer::extract_lifetime_params(&f.generics);
        let lifetime_bounds = lifetime::LifetimeInferencer::extract_lifetime_bounds(&f.generics);

        // Run lifetime inference
        let _resolution = self.lifetime_inferencer.infer_function_lifetimes(
            &f.name.node,
            &params,
            &ret_type,
            &lifetime_params,
            &lifetime_bounds,
        )?;

        Ok(())
    }

    /// Check an impl method body
    pub(crate) fn check_impl_method(
        &mut self,
        target_type: &Type,
        method: &Function,
        struct_generics: &[GenericParam],
    ) -> TypeResult<()> {
        self.push_scope();

        // Get the type name for self
        let self_type_name = match target_type {
            Type::Named { name, .. } => name.clone(),
            _ => return Ok(()), // Skip non-named types
        };

        // Combine struct generics with method generics
        let mut all_generics: Vec<GenericParam> = struct_generics.to_vec();
        all_generics.extend(method.generics.iter().cloned());

        // Set current generic parameters (including struct-level generics)
        let (prev_generics, prev_bounds, prev_const_generics) = self.set_generics(&all_generics);

        // Build the generics list for self type (struct-level generics as Generic types)
        let self_generics: Vec<ResolvedType> = struct_generics
            .iter()
            .map(|g| ResolvedType::Generic(g.name.node.clone()))
            .collect();

        // Add parameters to scope
        // Use registered method signature types when available (for Type::Infer support)
        let registered_method_params: Vec<_> = self
            .structs
            .get(&self_type_name)
            .and_then(|s| s.methods.get(&method.name.node))
            .map(|sig| sig.params.iter().map(|(_, ty, _)| ty.clone()).collect())
            .unwrap_or_default();
        for (i, param) in method.params.iter().enumerate() {
            // Handle &self parameter specially
            if param.name.node == "self" {
                // self is a reference to the target type with generics
                let self_ty = ResolvedType::Ref(Box::new(ResolvedType::Named {
                    name: self_type_name.clone(),
                    generics: self_generics.clone(),
                }));
                self.define_var("self", self_ty, param.is_mut);
            } else {
                let ty = if i < registered_method_params.len() {
                    registered_method_params[i].clone()
                } else {
                    self.resolve_type(&param.ty.node)
                };
                self.define_var(&param.name.node, ty, param.is_mut);
            }
        }

        // Set current function context
        self.current_fn_ret = Some(
            method
                .ret_type
                .as_ref()
                .map(|t| self.resolve_type(&t.node))
                .unwrap_or(ResolvedType::Unit),
        );
        self.current_fn_name = Some(format!("{}::{}", self_type_name, method.name.node));

        // Check body
        let body_type = match &method.body {
            FunctionBody::Expr(expr) => self.check_expr(expr)?,
            FunctionBody::Block(stmts) => self.check_block(stmts)?,
        };

        // Check return type (with auto-deref: &T unifies with T)
        let expected_ret = self.current_fn_ret.clone().expect(
            "Internal compiler error: current_fn_ret should be set during function checking",
        );
        let body_type_deref = if let ResolvedType::Ref(inner) = &body_type {
            if self.unify(&expected_ret, inner).is_ok() {
                *inner.clone()
            } else {
                body_type.clone()
            }
        } else {
            body_type.clone()
        };
        self.unify(&expected_ret, &body_type_deref)?;

        // Resolve inferred parameter types for impl methods (same as check_function)
        if method
            .params
            .iter()
            .any(|p| matches!(p.ty.node, Type::Infer))
        {
            let resolved_params: Vec<_> = {
                let sig = self
                    .structs
                    .get(&self_type_name)
                    .and_then(|s| s.methods.get(&method.name.node));
                if let Some(sig) = sig {
                    sig.params
                        .iter()
                        .map(|(name, ty, is_mut)| {
                            let resolved = self.apply_substitutions(ty);
                            let final_ty = if matches!(resolved, ResolvedType::Var(_)) {
                                ResolvedType::I64
                            } else {
                                resolved
                            };
                            (name.clone(), final_ty, *is_mut)
                        })
                        .collect()
                } else {
                    vec![]
                }
            };
            if !resolved_params.is_empty() {
                if let Some(struct_def) = self.structs.get_mut(&self_type_name) {
                    if let Some(sig) = struct_def.methods.get_mut(&method.name.node) {
                        sig.params = resolved_params;
                    }
                }
            }
        }

        self.current_fn_ret = None;
        self.current_fn_name = None;
        self.restore_generics(prev_generics, prev_bounds, prev_const_generics);
        self.pop_scope();

        Ok(())
    }
}
