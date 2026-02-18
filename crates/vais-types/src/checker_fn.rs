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
        let saved = self.set_generics(&f.generics);

        // Merge where clause bounds into current generic bounds
        self.merge_where_clause(&f.where_clause);

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

        // Detect recursive functions (using @) with inferred return type — this is undecidable
        if ret_type_inferred && Self::body_contains_self_call(&f.body) {
            return Err(TypeError::InferFailed {
                kind: "return type".to_string(),
                name: f.name.node.clone(),
                context: f.name.node.clone(),
                span: Some(f.name.span),
                suggestion: Some(
                    "Recursive functions require explicit return type annotation".to_string(),
                ),
            });
        }

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.resolve_type(&t.node))
            .unwrap_or_else(|| {
                // main() without return type defaults to i64 (program exit code)
                if f.name.node == "main" {
                    ResolvedType::I64
                } else {
                    self.fresh_type_var()
                }
            });
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
        // main() with implicit i64 return: allow Unit body (auto-return 0)
        if f.name.node == "main"
            && ret_type_inferred
            && expected_ret == ResolvedType::I64
            && body_type_deref == ResolvedType::Unit
        {
            // Skip unification — codegen will insert `ret i64 0`
        } else {
            self.unify(&expected_ret, &body_type_deref)?;
        }

        // Verify ImplTrait/DynTrait bounds: if return type is impl Trait or dyn Trait,
        // check that the concrete body type implements the required trait bounds.
        self.verify_trait_type_bounds(&expected_ret, &body_type_deref);

        // Resolve inferred return type: if return type was omitted, apply substitutions
        // to resolve the type variable to the concrete type from the body.
        if ret_type_inferred {
            let resolved_ret = self.apply_substitutions(&ret_type);
            let final_ret = if matches!(resolved_ret, ResolvedType::Var(_)) {
                // Unresolved type variable: compile error instead of silent i64 default
                return Err(TypeError::InferFailed {
                    kind: "return type".to_string(),
                    name: f.name.node.clone(),
                    context: f.name.node.clone(),
                    span: Some(f.name.span),
                    suggestion: Some("Add explicit return type annotation".to_string()),
                });
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
        // Update the FunctionSig with resolved types; error on unresolved vars.
        if f.params.iter().any(|p| matches!(p.ty.node, Type::Infer)) {
            // First check for unresolved type variables (before mutably borrowing)
            if let Some(sig) = self.functions.get(&f.name.node) {
                for (name, ty, _) in &sig.params {
                    let resolved = self.apply_substitutions(ty);
                    if matches!(resolved, ResolvedType::Var(_)) {
                        let param_span = f
                            .params
                            .iter()
                            .find(|p| p.name.node == *name)
                            .map(|p| p.name.span);
                        return Err(TypeError::InferFailed {
                            kind: "parameter".to_string(),
                            name: name.clone(),
                            context: f.name.node.clone(),
                            span: param_span,
                            suggestion: Some(format!("Add explicit type: `{}: <type>`", name)),
                        });
                    }
                }
            }
            let resolved_params: Vec<_> = {
                let sig = self.functions.get(&f.name.node);
                if let Some(sig) = sig {
                    sig.params
                        .iter()
                        .map(|(name, ty, is_mut)| {
                            let resolved = self.apply_substitutions(ty);
                            (name.clone(), resolved, *is_mut)
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

        // Validate that no unresolved type variables survive into codegen for non-generic functions.
        // Generic functions may legitimately contain Generic/ConstGeneric/ImplTrait/etc. in their
        // signatures; those are checked at instantiation time instead.
        if f.generics.is_empty() {
            // Check parameter types
            if let Some(sig) = self.functions.get(&f.name.node) {
                let params_snapshot: Vec<(String, ResolvedType)> = sig
                    .params
                    .iter()
                    .map(|(name, ty, _)| (name.clone(), ty.clone()))
                    .collect();
                for (param_name, ty) in &params_snapshot {
                    // Skip 'self' parameter — its type is resolved at impl block level
                    if param_name == "self" {
                        continue;
                    }
                    let resolved = self.apply_substitutions(ty);
                    if let Some(unresolved_desc) = Self::contains_unresolved_type(&resolved) {
                        let param_span = f
                            .params
                            .iter()
                            .find(|p| p.name.node == *param_name)
                            .map(|p| p.name.span);
                        return Err(TypeError::InferFailed {
                            kind: "parameter".to_string(),
                            name: param_name.clone(),
                            context: format!(
                                "{} (contains {})",
                                f.name.node, unresolved_desc
                            ),
                            span: param_span,
                            suggestion: Some(format!(
                                "Add explicit type annotation for parameter `{}`",
                                param_name
                            )),
                        });
                    }
                }

                // Check return type
                let ret_snapshot = sig.ret.clone();
                let resolved_ret = self.apply_substitutions(&ret_snapshot);
                if let Some(unresolved_desc) = Self::contains_unresolved_type(&resolved_ret) {
                    return Err(TypeError::InferFailed {
                        kind: "return type".to_string(),
                        name: f.name.node.clone(),
                        context: format!(
                            "{} (contains {})",
                            f.name.node, unresolved_desc
                        ),
                        span: Some(f.name.span),
                        suggestion: Some("Add explicit return type annotation".to_string()),
                    });
                }
            }
        }

        self.current_fn_ret = None;
        self.current_fn_name = None;
        self.restore_generics(saved);
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
        all_generics.extend_from_slice(&method.generics);

        // Set current generic parameters (including struct-level generics)
        let saved = self.set_generics(&all_generics);

        // Merge where clause bounds into current generic bounds
        self.merge_where_clause(&method.where_clause);

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
            // First check for unresolved type variables
            if let Some(sig) = self
                .structs
                .get(&self_type_name)
                .and_then(|s| s.methods.get(&method.name.node))
            {
                for (name, ty, _) in &sig.params {
                    // Skip 'self' parameter — its type is resolved at impl block level
                    if name == "self" {
                        continue;
                    }
                    let resolved = self.apply_substitutions(ty);
                    if matches!(resolved, ResolvedType::Var(_)) {
                        let param_span = method
                            .params
                            .iter()
                            .find(|p| p.name.node == *name)
                            .map(|p| p.name.span);
                        return Err(TypeError::InferFailed {
                            kind: "parameter".to_string(),
                            name: name.clone(),
                            context: format!("{}::{}", self_type_name, method.name.node),
                            span: param_span,
                            suggestion: Some(format!("Add explicit type: `{}: <type>`", name)),
                        });
                    }
                }
            }
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
                            (name.clone(), resolved, *is_mut)
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
        self.restore_generics(saved);
        self.pop_scope();

        Ok(())
    }

    /// Check if a function body contains any `@` (self-call/recursion)
    fn body_contains_self_call(body: &FunctionBody) -> bool {
        match body {
            FunctionBody::Expr(expr) => Self::expr_contains_self_call(&expr.node),
            FunctionBody::Block(stmts) => {
                stmts.iter().any(|s| Self::stmt_contains_self_call(&s.node))
            }
        }
    }

    fn expr_contains_self_call(expr: &Expr) -> bool {
        match expr {
            Expr::SelfCall => true,
            Expr::Call { func, args } => {
                Self::expr_contains_self_call(&func.node)
                    || args.iter().any(|a| Self::expr_contains_self_call(&a.node))
            }
            Expr::Binary { left, right, .. } => {
                Self::expr_contains_self_call(&left.node)
                    || Self::expr_contains_self_call(&right.node)
            }
            Expr::Unary { expr, .. } => Self::expr_contains_self_call(&expr.node),
            Expr::If {
                cond, then, else_, ..
            } => {
                Self::expr_contains_self_call(&cond.node)
                    || then.iter().any(|s| Self::stmt_contains_self_call(&s.node))
                    || else_.as_ref().is_some_and(Self::if_else_contains_self_call)
            }
            Expr::Block(stmts) => stmts.iter().any(|s| Self::stmt_contains_self_call(&s.node)),
            Expr::Assign { value, .. } => Self::expr_contains_self_call(&value.node),
            _ => false,
        }
    }

    fn if_else_contains_self_call(else_branch: &IfElse) -> bool {
        match else_branch {
            IfElse::Else(stmts) => stmts.iter().any(|s| Self::stmt_contains_self_call(&s.node)),
            IfElse::ElseIf(cond, stmts, next) => {
                Self::expr_contains_self_call(&cond.node)
                    || stmts.iter().any(|s| Self::stmt_contains_self_call(&s.node))
                    || next
                        .as_ref()
                        .is_some_and(|n| Self::if_else_contains_self_call(n))
            }
        }
    }

    fn stmt_contains_self_call(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr) => Self::expr_contains_self_call(&expr.node),
            Stmt::Let { value, .. } => Self::expr_contains_self_call(&value.node),
            Stmt::Return(Some(e)) => Self::expr_contains_self_call(&e.node),
            _ => false,
        }
    }

    /// Check if a resolved type tree contains any unresolved types
    /// that should have been resolved before codegen.
    /// Returns the name of the first unresolved type found, if any.
    fn contains_unresolved_type(ty: &ResolvedType) -> Option<String> {
        match ty {
            ResolvedType::Var(id) => Some(format!("type variable #{}", id)),
            ResolvedType::Unknown => Some("unknown type".to_string()),
            // Generic/ConstGeneric/ImplTrait/Associated/HigherKinded are OK in generic function
            // DEFINITIONS — they only become errors when they survive monomorphization,
            // so we don't check them here; they're checked at instantiation time.

            // Recurse into compound types
            ResolvedType::Pointer(inner)
            | ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Optional(inner)
            | ResolvedType::Slice(inner)
            | ResolvedType::SliceMut(inner)
            | ResolvedType::Future(inner)
            | ResolvedType::Linear(inner)
            | ResolvedType::Affine(inner)
            | ResolvedType::Lazy(inner)
            | ResolvedType::Range(inner) => Self::contains_unresolved_type(inner),
            ResolvedType::Array(inner) => Self::contains_unresolved_type(inner),
            ResolvedType::ConstArray { element, .. } => Self::contains_unresolved_type(element),
            ResolvedType::Result(ok, err) | ResolvedType::Map(ok, err) => {
                Self::contains_unresolved_type(ok)
                    .or_else(|| Self::contains_unresolved_type(err))
            }
            ResolvedType::Tuple(elems) => {
                elems.iter().find_map(Self::contains_unresolved_type)
            }
            ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
                params
                    .iter()
                    .find_map(Self::contains_unresolved_type)
                    .or_else(|| Self::contains_unresolved_type(ret))
            }
            ResolvedType::RefLifetime { inner, .. } | ResolvedType::RefMutLifetime { inner, .. } => {
                Self::contains_unresolved_type(inner)
            }
            ResolvedType::Dependent { base, .. } => Self::contains_unresolved_type(base),
            ResolvedType::Vector { element, .. } => Self::contains_unresolved_type(element),
            ResolvedType::Named { generics, .. } => {
                generics.iter().find_map(Self::contains_unresolved_type)
            }
            ResolvedType::DynTrait { generics, .. } => {
                generics.iter().find_map(Self::contains_unresolved_type)
            }
            ResolvedType::Associated { base, generics, .. } => {
                Self::contains_unresolved_type(base)
                    .or_else(|| generics.iter().find_map(Self::contains_unresolved_type))
            }
            // All other types (primitives, Never, Generic, ConstGeneric, ImplTrait,
            // HigherKinded, Lifetime) are acceptable outside of monomorphization contexts.
            _ => None,
        }
    }
}
