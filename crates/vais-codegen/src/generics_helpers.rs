//! Generic type resolution and instantiation helpers

use super::*;

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
        if self.types.structs.contains_key(name) {
            return name.to_string();
        }
        if let Some(mangled) = self.generics.struct_aliases.get(name) {
            return mangled.clone();
        }
        name.to_string()
    }

    /// Generate mangled name for a generic struct
    #[inline]
    pub(crate) fn mangle_struct_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
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
        // If only one instantiation exists, use it directly
        if instantiations_list.len() == 1 {
            return instantiations_list[0].1.clone();
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
            _ => {}
        }
    }
}
