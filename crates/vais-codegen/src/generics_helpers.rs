//! Generic type resolution and instantiation helpers

use super::*;
use std::collections::{HashMap, HashSet};

impl CodeGenerator {
    /// Get current generic substitution for a type parameter
    #[inline]
    pub(crate) fn get_generic_substitution(&self, param: &str) -> Option<ResolvedType> {
        self.generics.substitutions.get(param).cloned()
    }

    /// Set generic substitutions for the current context
    pub(crate) fn _set_generic_substitutions(&mut self, subst: HashMap<String, ResolvedType>) {
        self.generics.substitutions = subst;
    }

    /// Clear generic substitutions
    pub(crate) fn _clear_generic_substitutions(&mut self) {
        self.generics.substitutions.clear();
    }

    /// Resolve a struct name, checking aliases for generic specializations.
    /// Returns the mangled name if the base name has a registered alias (e.g., "Box" -> "Box$i64").
    #[inline]
    pub(crate) fn resolve_struct_name(&self, name: &str) -> String {
        // 1. Direct hit in the struct registry — use as-is.
        if self.types.structs.contains_key(name) {
            return name.to_string();
        }
        // 2. Alias registered for this base name (e.g., "Box" -> "Box$i64").
        if let Some(mangled) = self.generics.struct_aliases.get(name) {
            return mangled.clone();
        }
        // 3. Already a specialized/mangled struct name (e.g., "Point$i64") in generated_structs.
        if self.generics.generated_structs.contains_key(name) {
            return name.to_string();
        }
        // 4. If the name contains '$', it is already in mangled form — accept as-is even if
        //    the struct hasn't been emitted yet (it will be resolved later by the LLVM emitter).
        if name.contains('$') {
            return name.to_string();
        }
        // 5. If the name is a plain base name, try to find any generated struct whose
        //    mangled name starts with "<name>$" (picks the first/only specialization).
        //    This handles cross-module access where only one specialization exists.
        let prefix = format!("{name}$");
        if let Some(mangled_key) = self
            .generics
            .generated_structs
            .keys()
            .find(|k| k.starts_with(&prefix))
        {
            return mangled_key.clone();
        }
        // Final fallback: return name unchanged.
        // Note: this may fail later during LLVM type lookup — the caller is responsible
        // for handling unknown struct names gracefully.
        name.to_string()
    }

    /// Generate mangled name for a generic struct
    #[inline]
    pub(crate) fn mangle_struct_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
    }

    pub(crate) fn generic_struct_layout_uses_type_args(&self, name: &str) -> bool {
        let Some(struct_def) = self.generics.struct_defs.get(name) else {
            return false;
        };
        let params: HashSet<String> = struct_def
            .generics
            .iter()
            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
            .map(|g| g.name.node.to_string())
            .collect();
        if params.is_empty() {
            return false;
        }

        if let Some(struct_info) = self.types.structs.get(name) {
            return struct_info
                .fields
                .iter()
                .any(|(_, ty)| Self::type_mentions_generic_params(ty, &params));
        }

        struct_def.fields.iter().any(|field| {
            let ty = self.ast_type_to_resolved(&field.ty.node);
            Self::type_mentions_generic_params(&ty, &params)
        })
    }

    pub(crate) fn infer_struct_literal_generic_args(
        &self,
        struct_name: &str,
        struct_fields: &[(String, ResolvedType)],
        literal_fields: &[(Spanned<String>, Spanned<Expr>)],
    ) -> Option<Vec<ResolvedType>> {
        let struct_def = self.generics.struct_defs.get(struct_name)?;
        let params: Vec<String> = struct_def
            .generics
            .iter()
            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
            .map(|g| g.name.node.to_string())
            .collect();
        if params.is_empty() {
            return None;
        }
        let param_set: HashSet<String> = params.iter().cloned().collect();
        let mut bindings = HashMap::new();
        let declared_fields: Vec<(String, ResolvedType)> = struct_def
            .fields
            .iter()
            .map(|field| {
                (
                    field.name.node.to_string(),
                    self.ast_type_to_resolved(&field.ty.node),
                )
            })
            .collect();

        for (field_name, field_expr) in literal_fields {
            let Some((_, declared_ty)) = declared_fields
                .iter()
                .chain(struct_fields.iter())
                .find(|(name, _)| name == &field_name.node)
            else {
                continue;
            };
            let actual_ty = self
                .refine_base_named_with_unique_specialization(&self.infer_expr_type(field_expr));
            Self::bind_generic_params_from_type_pattern(
                declared_ty,
                &actual_ty,
                &param_set,
                &mut bindings,
            );
        }

        params
            .into_iter()
            .map(|param| bindings.remove(&param))
            .collect::<Option<Vec<_>>>()
            .filter(|args| args.iter().all(Self::is_concrete_generic_arg))
    }

    fn refine_base_named_with_unique_specialization(&self, ty: &ResolvedType) -> ResolvedType {
        let ResolvedType::Named { name, generics } = ty else {
            return ty.clone();
        };
        if !generics.is_empty() {
            return ty.clone();
        }
        let Some(struct_def) = self.generics.struct_defs.get(name) else {
            return ty.clone();
        };
        let params: Vec<String> = struct_def
            .generics
            .iter()
            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
            .map(|g| g.name.node.to_string())
            .collect();
        if params.is_empty() {
            return ty.clone();
        }

        let prefix = format!("{name}$");
        let mut candidates = self
            .types
            .structs
            .iter()
            .filter(|(candidate_name, _)| candidate_name.starts_with(&prefix));
        let Some((_, specialized)) = candidates.next() else {
            return ty.clone();
        };
        if candidates.next().is_some() {
            return ty.clone();
        }

        let Some(base) = self.types.structs.get(name) else {
            return ty.clone();
        };
        if base.fields.len() != specialized.fields.len() {
            return ty.clone();
        }

        let param_set: HashSet<String> = params.iter().cloned().collect();
        let mut bindings = HashMap::new();
        for ((_, pattern), (_, actual)) in base.fields.iter().zip(specialized.fields.iter()) {
            Self::bind_generic_params_from_type_pattern(pattern, actual, &param_set, &mut bindings);
        }

        params
            .into_iter()
            .map(|param| bindings.remove(&param))
            .collect::<Option<Vec<_>>>()
            .filter(|args| args.iter().all(Self::is_concrete_generic_arg))
            .map(|generics| ResolvedType::Named {
                name: name.clone(),
                generics,
            })
            .unwrap_or_else(|| ty.clone())
    }

    fn is_concrete_generic_arg(ty: &ResolvedType) -> bool {
        !matches!(
            ty,
            ResolvedType::Generic(_) | ResolvedType::Var(_) | ResolvedType::Unknown
        )
    }

    fn type_mentions_generic_params(ty: &ResolvedType, params: &HashSet<String>) -> bool {
        match ty {
            ResolvedType::Generic(name) => params.contains(name),
            ResolvedType::Named { generics, .. } | ResolvedType::DynTrait { generics, .. } => {
                generics
                    .iter()
                    .any(|g| Self::type_mentions_generic_params(g, params))
            }
            ResolvedType::Optional(inner)
            | ResolvedType::Pointer(inner)
            | ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Slice(inner)
            | ResolvedType::SliceMut(inner)
            | ResolvedType::Array(inner)
            | ResolvedType::Future(inner)
            | ResolvedType::Linear(inner)
            | ResolvedType::Affine(inner)
            | ResolvedType::Range(inner)
            | ResolvedType::Dependent { base: inner, .. }
            | ResolvedType::RefLifetime { inner, .. }
            | ResolvedType::RefMutLifetime { inner, .. } => {
                Self::type_mentions_generic_params(inner, params)
            }
            ResolvedType::Result(ok, err) | ResolvedType::Map(ok, err) => {
                Self::type_mentions_generic_params(ok, params)
                    || Self::type_mentions_generic_params(err, params)
            }
            ResolvedType::Tuple(items) => items
                .iter()
                .any(|item| Self::type_mentions_generic_params(item, params)),
            ResolvedType::Fn {
                params: ps, ret, ..
            }
            | ResolvedType::FnPtr {
                params: ps, ret, ..
            } => {
                ps.iter()
                    .any(|p| Self::type_mentions_generic_params(p, params))
                    || Self::type_mentions_generic_params(ret, params)
            }
            ResolvedType::Vector { element, .. } | ResolvedType::ConstArray { element, .. } => {
                Self::type_mentions_generic_params(element, params)
            }
            ResolvedType::Associated { base, generics, .. } => {
                Self::type_mentions_generic_params(base, params)
                    || generics
                        .iter()
                        .any(|g| Self::type_mentions_generic_params(g, params))
            }
            _ => false,
        }
    }

    fn bind_generic_params_from_type_pattern(
        pattern: &ResolvedType,
        actual: &ResolvedType,
        params: &HashSet<String>,
        bindings: &mut HashMap<String, ResolvedType>,
    ) {
        match (pattern, actual) {
            (ResolvedType::Generic(name), actual) if params.contains(name) => {
                if Self::is_concrete_generic_arg(actual) {
                    bindings
                        .entry(name.clone())
                        .or_insert_with(|| actual.clone());
                }
            }
            (
                ResolvedType::Named {
                    name: p_name,
                    generics: p_generics,
                },
                ResolvedType::Named {
                    name: a_name,
                    generics: a_generics,
                },
            ) if p_name == a_name && p_generics.len() == a_generics.len() => {
                for (p, a) in p_generics.iter().zip(a_generics.iter()) {
                    Self::bind_generic_params_from_type_pattern(p, a, params, bindings);
                }
            }
            (ResolvedType::Ref(p), ResolvedType::Ref(a))
            | (ResolvedType::RefMut(p), ResolvedType::RefMut(a))
            | (ResolvedType::Pointer(p), ResolvedType::Pointer(a))
            | (ResolvedType::Optional(p), ResolvedType::Optional(a))
            | (ResolvedType::Slice(p), ResolvedType::Slice(a))
            | (ResolvedType::SliceMut(p), ResolvedType::SliceMut(a))
            | (ResolvedType::Array(p), ResolvedType::Array(a))
            | (ResolvedType::Range(p), ResolvedType::Range(a)) => {
                Self::bind_generic_params_from_type_pattern(p, a, params, bindings);
            }
            (ResolvedType::Result(p_ok, p_err), ResolvedType::Result(a_ok, a_err))
            | (ResolvedType::Map(p_ok, p_err), ResolvedType::Map(a_ok, a_err)) => {
                Self::bind_generic_params_from_type_pattern(p_ok, a_ok, params, bindings);
                Self::bind_generic_params_from_type_pattern(p_err, a_err, params, bindings);
            }
            (ResolvedType::Tuple(ps), ResolvedType::Tuple(as_)) if ps.len() == as_.len() => {
                for (p, a) in ps.iter().zip(as_.iter()) {
                    Self::bind_generic_params_from_type_pattern(p, a, params, bindings);
                }
            }
            _ => {}
        }
    }

    /// Generate mangled name for a generic function
    pub(crate) fn _mangle_function_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
    }

    /// Resolve a generic function call to the appropriate mangled specialized name.
    /// Given a base function name and the inferred argument types, finds the
    /// matching instantiation from the pre-computed instantiation list.
    pub(crate) fn resolve_generic_call(
        &self,
        base_name: &str,
        arg_types: &[ResolvedType],
        instantiations_list: &[(Vec<ResolvedType>, String)],
    ) -> String {
        self.resolve_generic_call_with_hint(base_name, arg_types, instantiations_list, None)
    }

    /// Phase 16 A2.5: resolve_generic_call with an optional expected return
    /// type hint. Functions whose generic parameter only appears in the return
    /// type (e.g. `Vec.with_capacity(cap: i64) -> Vec<T>`) cannot be inferred
    /// from argument types alone, which would previously fall back to the
    /// `I64` default and pick the wrong monomorphization. When the caller
    /// knows the desired return type (for example a struct field store target
    /// or a `let` binding with an annotated type), it should flow that
    /// information in through `expected_ret` so we can unify against the
    /// declared return pattern and recover the missing type argument.
    pub(crate) fn resolve_generic_call_with_hint(
        &self,
        base_name: &str,
        arg_types: &[ResolvedType],
        instantiations_list: &[(Vec<ResolvedType>, String)],
        expected_ret: Option<&ResolvedType>,
    ) -> String {
        // Phase 16 A2.5: if we have a direct return-type hint and the
        // template's generics map one-to-one onto the return type's generic
        // positions (true for simple container constructors like
        // `Vec.with_capacity(cap: i64) -> Vec<T>`), pick the instantiation
        // whose type args match the return-type generics directly. This
        // bypasses the I64 fallback in the argument-only inference path.
        if let Some(ret_ty) = expected_ret {
            if let ResolvedType::Named {
                generics: ret_generics,
                ..
            } = ret_ty
            {
                if !ret_generics.is_empty() {
                    for (inst_types, mangled) in instantiations_list {
                        if inst_types == ret_generics {
                            return mangled.clone();
                        }
                    }
                }
            }
        }

        // If only one instantiation exists, check if it has generic type args.
        // If so, apply current substitutions to derive the concrete mangled name.
        if instantiations_list.len() == 1 {
            let (type_args, name) = &instantiations_list[0];
            let has_generic_args = type_args
                .iter()
                .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)));
            if has_generic_args && !self.generics.substitutions.is_empty() {
                // Substitute current generics to get concrete type args
                let concrete_args: Vec<ResolvedType> = type_args
                    .iter()
                    .map(|t| vais_types::substitute_type(t, &self.generics.substitutions))
                    .collect();
                // Only use concrete name if all args are now concrete
                let all_concrete = concrete_args
                    .iter()
                    .all(|t| !matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)));
                if all_concrete {
                    let mangled = vais_types::mangle_name(base_name, &concrete_args);
                    return mangled;
                }
            }
            return name.clone();
        }

        // Look up the generic function template to map argument types to type parameters
        if let Some(template) = self.generics.function_templates.get(base_name) {
            let type_params: Vec<&String> = template
                .generics
                .iter()
                .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                .map(|g| &g.name.node)
                .collect();

            // Infer type arguments from argument types by matching against parameter types
            let mut inferred: HashMap<String, ResolvedType> = HashMap::new();
            for (i, param) in template.params.iter().enumerate() {
                if i < arg_types.len() {
                    self.infer_type_args(
                        &self.ast_type_to_resolved(&param.ty.node),
                        &arg_types[i],
                        &type_params,
                        &mut inferred,
                    );
                }
            }

            // Phase 16 A2.5: if the caller supplied an expected return type,
            // unify it against the template's declared return pattern to
            // recover generic params that don't appear in any parameter
            // (e.g. `Vec.with_capacity(cap: i64) -> Vec<T>`).
            if let (Some(ret_ty), Some(ret_ast)) = (expected_ret, template.ret_type.as_ref()) {
                let ret_pattern = self.ast_type_to_resolved(&ret_ast.node);
                self.infer_type_args(&ret_pattern, ret_ty, &type_params, &mut inferred);
            }

            // Build type_args vector in order of generic params
            let type_args: Vec<ResolvedType> = type_params
                .iter()
                .map(|name| inferred.get(*name).cloned().unwrap_or(ResolvedType::I64))
                .collect();

            // Find exact match in instantiations
            for (inst_types, mangled) in instantiations_list {
                if inst_types == &type_args {
                    return mangled.clone();
                }
            }
        }

        // Try substituting generic type args in instantiations (for transitive calls)
        if !self.generics.substitutions.is_empty() {
            for (inst_types, _) in instantiations_list {
                let has_generic = inst_types
                    .iter()
                    .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)));
                if has_generic {
                    let concrete_args: Vec<ResolvedType> = inst_types
                        .iter()
                        .map(|t| vais_types::substitute_type(t, &self.generics.substitutions))
                        .collect();
                    let all_concrete = concrete_args
                        .iter()
                        .all(|t| !matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)));
                    if all_concrete {
                        let mangled = vais_types::mangle_name(base_name, &concrete_args);
                        if self.types.functions.contains_key(&mangled) {
                            return mangled;
                        }
                    }
                }
            }
        }

        // Fallback: try to mangle based on argument types directly
        let mangled = vais_types::mangle_name(base_name, arg_types);
        if self.types.functions.contains_key(&mangled) {
            return mangled;
        }

        // Last resort: use first instantiation
        instantiations_list
            .first()
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| base_name.to_string())
    }

    /// Infer type arguments by matching a parameter type pattern against a concrete argument type.
    pub(crate) fn infer_type_args(
        &self,
        param_type: &ResolvedType,
        arg_type: &ResolvedType,
        type_params: &[&String],
        inferred: &mut HashMap<String, ResolvedType>,
    ) {
        match param_type {
            ResolvedType::Generic(name) => {
                // Direct generic type parameter (e.g., T)
                if type_params.contains(&name) {
                    inferred
                        .entry(name.clone())
                        .or_insert_with(|| arg_type.clone());
                }
            }
            ResolvedType::Named { name, generics } => {
                // Check if this is a type parameter name
                if type_params.contains(&name) {
                    inferred
                        .entry(name.clone())
                        .or_insert_with(|| arg_type.clone());
                } else if let ResolvedType::Named {
                    generics: arg_generics,
                    ..
                } = arg_type
                {
                    // Recurse into generic arguments
                    for (g, ag) in generics.iter().zip(arg_generics.iter()) {
                        self.infer_type_args(g, ag, type_params, inferred);
                    }
                }
            }
            ResolvedType::Array(inner) => {
                if let ResolvedType::Array(arg_inner) = arg_type {
                    self.infer_type_args(inner, arg_inner, type_params, inferred);
                }
            }
            ResolvedType::Pointer(inner) => {
                if let ResolvedType::Pointer(arg_inner) = arg_type {
                    self.infer_type_args(inner, arg_inner, type_params, inferred);
                }
            }
            ResolvedType::Optional(inner) => {
                if let ResolvedType::Optional(arg_inner) = arg_type {
                    self.infer_type_args(inner, arg_inner, type_params, inferred);
                }
            }
            // HKT removed in ROADMAP #18.
            _ => {}
        }
    }
}
