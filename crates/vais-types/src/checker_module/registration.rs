//! Registration of functions, structs, enums, unions, and type/trait aliases.

use super::*;

impl TypeChecker {
    /// Register a function signature
    pub(crate) fn register_function(&mut self, f: &Function) -> TypeResult<()> {
        let name = f.name.node.clone();

        // Set current generics for type resolution
        let saved = self.set_generics(&f.generics);

        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.resolve_type(&p.ty.node);
                (p.name.node.clone(), ty, p.is_mut)
            })
            .collect();

        let ret = f
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

        // Restore previous generics
        self.restore_generics(saved);

        // Build generic bounds: merge inline bounds with where clause bounds
        let mut generic_bounds: HashMap<String, Vec<String>> = f
            .generics
            .iter()
            .map(|g| {
                (
                    g.name.node.clone(),
                    g.bounds.iter().map(|b| &b.node).cloned().collect(),
                )
            })
            .collect();

        // Merge where clause bounds (dedup to avoid duplicates from inline + where)
        for predicate in &f.where_clause {
            let bounds = generic_bounds.entry(predicate.ty.node.clone()).or_default();
            for b in &predicate.bounds {
                if !bounds.contains(&b.node) {
                    bounds.push(b.node.clone());
                }
            }
        }

        // Count required parameters (those without default values)
        let has_defaults = f.params.iter().any(|p| p.default_value.is_some());
        let required_params = if has_defaults {
            Some(
                f.params
                    .iter()
                    .filter(|p| p.default_value.is_none())
                    .count(),
            )
        } else {
            None // All required (backward compatible)
        };

        self.functions.insert(
            name.clone(),
            FunctionSig {
                name,
                generics: f.generics.iter().map(|g| &g.name.node).cloned().collect(),
                generic_bounds,
                params,
                ret,
                is_async: f.is_async,
                required_params,
                is_vararg: false,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
                hkt_params: extract_hkt_params(&f.generics),
            },
        );

        Ok(())
    }

    /// Register an extern function
    pub(crate) fn register_extern_function(
        &mut self,
        func: &vais_ast::ExternFunction,
    ) -> TypeResult<()> {
        let name = func.name.node.clone();
        if self.functions.contains_key(&name) {
            // Allow re-declaring extern functions that are already registered
            // (e.g., printf declared in example when already in builtins)
            return Ok(());
        }

        let params: Vec<_> = func
            .params
            .iter()
            .map(|p| {
                let ty = self.resolve_type(&p.ty.node);
                (p.name.node.clone(), ty, p.is_mut)
            })
            .collect();

        let ret = func
            .ret_type
            .as_ref()
            .map(|t| self.resolve_type(&t.node))
            .unwrap_or(ResolvedType::Unit);

        // Validate common extern function signatures and warn on mismatches
        self.validate_extern_signature(&name, &ret);

        self.functions.insert(
            name.clone(),
            FunctionSig {
                name,
                generics: vec![],
                generic_bounds: HashMap::new(),
                params,
                ret,
                is_async: false,
                is_vararg: func.is_vararg,
                required_params: None,
                contracts: None,
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
                hkt_params: HashMap::new(),
            },
        );

        Ok(())
    }

    /// Validate extern function signatures for common patterns.
    /// Emits warnings for known extern functions with unexpected return types.
    pub(crate) fn validate_extern_signature(&mut self, name: &str, ret: &ResolvedType) {
        // Common allocation/pointer-returning functions should return i64 (pointer)
        let pointer_returning_fns = ["malloc", "calloc", "realloc", "mmap", "fopen", "dlopen"];
        if pointer_returning_fns.contains(&name)
            && !matches!(
                ret,
                ResolvedType::I64 | ResolvedType::Pointer(_) | ResolvedType::Unknown
            )
        {
            self.warnings.push(format!(
                "extern function `{}` should return `i64` (pointer), found `{}`",
                name, ret
            ));
        }
    }

    /// Register a struct
    pub(crate) fn register_struct(&mut self, s: &Struct) -> TypeResult<()> {
        let name = s.name.node.clone();
        if self.structs.contains_key(&name) {
            // Allow re-declaration (e.g., from imports or std modules)
            return Ok(());
        }

        // Set current generics for type resolution
        let saved = self.set_generics(&s.generics);

        // Merge where clause bounds into current generic bounds
        self.merge_where_clause(&s.where_clause);

        let mut fields = HashMap::new();
        let mut field_order = Vec::new();
        for field in &s.fields {
            field_order.push(field.name.node.clone());
            fields.insert(field.name.node.clone(), self.resolve_type(&field.ty.node));
        }

        let mut methods = HashMap::new();
        for method in &s.methods {
            let params: Vec<_> = method
                .node
                .params
                .iter()
                .map(|p| {
                    let ty = self.resolve_type(&p.ty.node);
                    (p.name.node.clone(), ty, p.is_mut)
                })
                .collect();

            let ret = method
                .node
                .ret_type
                .as_ref()
                .map(|t| self.resolve_type(&t.node))
                .unwrap_or(ResolvedType::Unit);

            // Build method generic bounds: merge inline bounds with where clause bounds
            let mut method_bounds: HashMap<String, Vec<String>> = method
                .node
                .generics
                .iter()
                .map(|g| {
                    (
                        g.name.node.clone(),
                        g.bounds.iter().map(|b| &b.node).cloned().collect(),
                    )
                })
                .collect();

            // Merge method where clause bounds
            for predicate in &method.node.where_clause {
                method_bounds
                    .entry(predicate.ty.node.clone())
                    .or_default()
                    .extend(predicate.bounds.iter().map(|b| b.node.clone()));
            }

            methods.insert(
                method.node.name.node.clone(),
                FunctionSig {
                    name: method.node.name.node.clone(),
                    generics: method
                        .node
                        .generics
                        .iter()
                        .map(|g| g.name.node.clone())
                        .collect(),
                    generic_bounds: method_bounds,
                    params,
                    ret,
                    is_async: method.node.is_async,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                    hkt_params: extract_hkt_params(&method.node.generics),
                },
            );
        }

        // Restore previous generics
        self.restore_generics(saved);

        self.structs.insert(
            name.clone(),
            StructDef {
                name,
                generics: s.generics.iter().map(|g| &g.name.node).cloned().collect(),
                fields,
                field_order,
                methods,
                repr_c: s
                    .attributes
                    .iter()
                    .any(|a| a.name == "repr" && a.args.iter().any(|arg| arg == "C")),
            },
        );

        Ok(())
    }

    /// Register an enum
    pub(crate) fn register_enum(&mut self, e: &Enum) -> TypeResult<()> {
        let name = e.name.node.clone();
        // Allow re-registration of built-in enums (Result, Option) from std lib
        let is_builtin_override = name == "Result" || name == "Option";
        if self.enums.contains_key(&name) && !is_builtin_override {
            return Err(TypeError::Duplicate(name, None));
        }

        // Set current generics for type resolution
        let saved = self.set_generics(&e.generics);

        let mut variants = HashMap::new();
        for variant in &e.variants {
            let field_types = match &variant.fields {
                VariantFields::Unit => VariantFieldTypes::Unit,
                VariantFields::Tuple(ts) => {
                    let types: Vec<ResolvedType> =
                        ts.iter().map(|t| self.resolve_type(&t.node)).collect();
                    VariantFieldTypes::Tuple(types)
                }
                VariantFields::Struct(fields) => {
                    let mut field_map = HashMap::new();
                    for field in fields {
                        let field_name = field.name.node.clone();
                        let field_type = self.resolve_type(&field.ty.node);
                        field_map.insert(field_name, field_type);
                    }
                    VariantFieldTypes::Struct(field_map)
                }
            };
            variants.insert(variant.name.node.clone(), field_types);
        }

        // Restore previous generics
        self.restore_generics(saved);

        // Register enum variants for exhaustiveness checking
        let variant_names: Vec<String> = e.variants.iter().map(|v| &v.name.node).cloned().collect();
        self.exhaustiveness_checker
            .register_enum(&name, variant_names);

        self.enums.insert(
            name.clone(),
            EnumDef {
                name,
                generics: e.generics.iter().map(|g| &g.name.node).cloned().collect(),
                variants,
                methods: HashMap::new(),
            },
        );

        Ok(())
    }

    /// Register a union (untagged, C-style)
    pub(crate) fn register_union(&mut self, u: &Union) -> TypeResult<()> {
        let name = u.name.node.clone();
        if self.unions.contains_key(&name) {
            return Err(TypeError::Duplicate(name, None));
        }

        // Set current generics for type resolution
        let saved = self.set_generics(&u.generics);

        let mut fields = HashMap::new();
        for field in &u.fields {
            fields.insert(field.name.node.clone(), self.resolve_type(&field.ty.node));
        }

        // Restore previous generics
        self.restore_generics(saved);

        self.unions.insert(
            name.clone(),
            UnionDef {
                name,
                generics: u.generics.iter().map(|g| &g.name.node).cloned().collect(),
                fields,
            },
        );

        Ok(())
    }

    /// Register a type alias
    pub(crate) fn register_type_alias(&mut self, t: &TypeAlias) -> TypeResult<()> {
        let name = t.name.node.clone();
        if self.type_aliases.contains_key(&name) {
            return Err(TypeError::Duplicate(name, None));
        }

        let resolved = self.resolve_type(&t.ty.node);
        self.type_aliases.insert(name, resolved);

        Ok(())
    }

    pub(crate) fn register_trait_alias(&mut self, ta: &vais_ast::TraitAlias) -> TypeResult<()> {
        let name = ta.name.node.clone();
        if self.trait_aliases.contains_key(&name) {
            return Err(TypeError::Duplicate(name, None));
        }
        let bounds: Vec<String> = ta.bounds.iter().map(|b| b.node.clone()).collect();
        // Check for cyclic trait alias references
        for bound in &bounds {
            if self.trait_alias_reaches(&name, bound) {
                return Err(TypeError::Duplicate(
                    format!("cyclic trait alias: {} references {}", name, bound),
                    None,
                ));
            }
        }
        self.trait_aliases.insert(name, bounds);
        Ok(())
    }

    /// Check if expanding `target` eventually references `name` (cycle detection)
    fn trait_alias_reaches(&self, name: &str, target: &str) -> bool {
        if target == name {
            return true;
        }
        let mut visited = HashSet::new();
        let mut stack = vec![target.to_string()];
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            if let Some(bounds) = self.trait_aliases.get(&current) {
                for b in bounds {
                    if b == name {
                        return true;
                    }
                    stack.push(b.clone());
                }
            }
        }
        false
    }
}
