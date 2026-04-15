//! Module generation with generic instantiations.
//!
//! Handles compilation of modules that include generic function and struct
//! specializations, producing monomorphized LLVM IR for each concrete
//! type argument combination.

use super::*;

impl CodeGenerator {
    pub fn generate_module_with_instantiations(
        &mut self,
        module: &Module,
        instantiations: &[vais_types::GenericInstantiation],
    ) -> CodegenResult<String> {
        let mut ir = String::new();

        self.emit_module_header(&mut ir);
        // First pass: collect declarations (including generic templates)
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    // Track this function name (generic or not)
                    self.types.declared_functions.insert(f.name.node.clone());

                    if !f.generics.is_empty() {
                        // Store generic function for later specialization
                        self.generics
                            .function_templates
                            .insert(f.name.node.clone(), std::rc::Rc::new(f.clone()));
                    } else {
                        self.register_function(f)?;
                    }
                }
                Item::Struct(s) => {
                    if !s.generics.is_empty() {
                        // Store generic struct for later specialization
                        self.generics
                            .struct_defs
                            .insert(s.name.node.clone(), std::rc::Rc::new(s.clone()));
                    }
                    // Always register the struct (including generic ones) so that
                    // struct literals using the base name can find it. For generic
                    // structs, unresolved type params fall back to i64.
                    self.register_struct(s)?;
                    for method in &s.methods {
                        self.register_method(&s.name.node, &method.node)?;
                    }
                }
                Item::Enum(e) => self.register_enum(e)?,
                Item::Union(u) => self.register_union(u)?,
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        self.register_method(&type_name, &method.node)?;
                    }
                    // Register trait impl for vtable generation
                    if let Some(ref trait_name) = impl_block.trait_name {
                        let mut method_impls = HashMap::new();
                        for method in &impl_block.methods {
                            let fn_name = format!("{}_{}", type_name, method.node.name.node);
                            method_impls.insert(method.node.name.node.clone(), fn_name);
                        }
                        self.register_trait_impl(&type_name, &trait_name.node, method_impls);
                    }
                }
                Item::Trait(trait_def) => {
                    self.register_trait_from_ast(trait_def);
                }
                Item::ExternBlock(extern_block) => {
                    for func in &extern_block.functions {
                        self.register_extern_function(func, &extern_block.abi)?;
                    }
                }
                Item::Const(const_def) => {
                    self.register_const(const_def)?;
                }
                Item::Global(global_def) => {
                    self.register_global(global_def)?;
                }
                Item::TraitAlias(ta) => {
                    let bounds: Vec<String> = ta.bounds.iter().map(|b| b.node.clone()).collect();
                    self.types
                        .trait_aliases
                        .insert(ta.name.node.clone(), bounds);
                }
                Item::Use(_) | Item::TypeAlias(_) | Item::Macro(_) | Item::Error { .. } => {}
            }
        }

        // Pre-build method template lookup: (struct_name, method_name) -> Function AST.
        // Collects methods from impl blocks and struct inline methods that have their own
        // generic parameters (method-level generics, not struct-level generics).
        let mut method_templates: HashMap<(String, String), std::rc::Rc<Function>> = HashMap::new();
        // Also collect methods from impl blocks on generic structs where the method itself
        // has no generics but inherits type parameters from the struct (e.g., Vec<T>.push(T)).
        // These are stored separately because they need struct-level generic substitution.
        let mut generic_impl_methods: HashMap<(String, String), std::rc::Rc<Function>> =
            HashMap::new();
        for item in &module.items {
            match &item.node {
                Item::Impl(impl_block) => {
                    let (impl_type_name, is_generic_impl) = match &impl_block.target_type.node {
                        Type::Named {
                            name,
                            generics: type_params,
                        } => (
                            name.clone(),
                            !impl_block.generics.is_empty() || !type_params.is_empty(),
                        ),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        if !method.node.generics.is_empty() {
                            // Method-level generics
                            method_templates.insert(
                                (impl_type_name.clone(), method.node.name.node.clone()),
                                std::rc::Rc::new(method.node.clone()),
                            );
                        } else if is_generic_impl {
                            // Struct-level generics inherited by method (e.g., Vec<T>.push)
                            generic_impl_methods.insert(
                                (impl_type_name.clone(), method.node.name.node.clone()),
                                std::rc::Rc::new(method.node.clone()),
                            );
                        }
                    }
                }
                Item::Struct(s) => {
                    for method in &s.methods {
                        if !method.node.generics.is_empty() {
                            method_templates.insert(
                                (s.name.node.clone(), method.node.name.node.clone()),
                                std::rc::Rc::new(method.node.clone()),
                            );
                        } else if !s.generics.is_empty() {
                            // Struct-level generics inherited by inline method
                            generic_impl_methods.insert(
                                (s.name.node.clone(), method.node.name.node.clone()),
                                std::rc::Rc::new(method.node.clone()),
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        // Persist generic impl methods for on-demand specialization during codegen
        for ((sname, mname), func) in &generic_impl_methods {
            self.generics
                .generic_method_bodies
                .insert((sname.clone(), mname.clone()), func.clone());
        }

        // Build generic function instantiation mapping and register specialized function signatures.
        // Only process concrete instantiations (all type args are resolved, non-generic).
        // Non-concrete instantiations (e.g., make_container$T from inside a generic function body)
        // are skipped — they would produce unresolved generic IR like `@identity$T`.
        for inst in instantiations.iter() {
            if let vais_types::InstantiationKind::Function = inst.kind {
                // Skip instantiations with non-concrete type args
                if inst
                    .type_args
                    .iter()
                    .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)))
                {
                    continue;
                }
                if let Some(generic_fn) = self
                    .generics
                    .function_templates
                    .get(&inst.base_name)
                    .cloned()
                {
                    // Build instantiation mapping: base_name -> [(type_args, mangled_name)]
                    self.generics
                        .fn_instantiations
                        .entry(inst.base_name.clone())
                        .or_default()
                        .push((inst.type_args.clone(), inst.mangled_name.clone()));

                    // Register the specialized function signature so call codegen can find it
                    let mut substitutions: HashMap<String, ResolvedType> = generic_fn
                        .generics
                        .iter()
                        .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                        .zip(inst.type_args.iter())
                        .map(|(g, t)| (g.name.node.to_string(), t.clone()))
                        .collect();

                    // Add const generic args to substitution map (name -> I64 representation)
                    for (name, _value) in &inst.const_args {
                        substitutions
                            .entry(name.clone())
                            .or_insert(ResolvedType::I64);
                    }

                    let params: Vec<_> = generic_fn
                        .params
                        .iter()
                        .map(|p| {
                            let ty = self.ast_type_to_resolved(&p.ty.node);
                            let concrete_ty = vais_types::substitute_type(&ty, &substitutions);
                            (p.name.node.to_string(), concrete_ty, p.is_mut)
                        })
                        .collect();

                    let ret_type = generic_fn
                        .ret_type
                        .as_ref()
                        .map(|t| {
                            let ty = self.ast_type_to_resolved(&t.node);
                            vais_types::substitute_type(&ty, &substitutions)
                        })
                        .unwrap_or(ResolvedType::Unit);

                    let mangled = inst.mangled_name.clone();
                    self.types.functions.insert(
                        mangled.clone(),
                        FunctionInfo {
                            signature: vais_types::FunctionSig {
                                name: mangled,
                                params,
                                ret: ret_type,
                                is_async: generic_fn.is_async,
                                ..Default::default()
                            },
                            is_extern: false,
                            _extern_abi: None,
                        },
                    );
                }
            }
            // Handle Method instantiations — register specialized method signatures
            if let vais_types::InstantiationKind::Method { ref struct_name } = inst.kind {
                if inst
                    .type_args
                    .iter()
                    .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)))
                {
                    continue;
                }
                let key = (struct_name.clone(), inst.base_name.clone());
                // Try method-level generic templates first, then fall back to struct-level
                // generic methods from the struct definition, then impl block methods.
                let method_fn_opt = method_templates
                    .get(&key)
                    .cloned()
                    .or_else(|| {
                        // For struct-level generics (e.g., Vec<T>.push(T)),
                        // the method itself has no generics — they come from the struct.
                        // Find the method from the struct's inline methods.
                        self.generics.struct_defs.get(struct_name).and_then(|s| {
                            s.methods
                                .iter()
                                .find(|m| m.node.name.node == inst.base_name)
                                .map(|m| std::rc::Rc::new(m.node.clone()))
                        })
                    })
                    .or_else(|| {
                        // Fall back to impl block methods on generic structs (e.g., X Vec<T> { ... })
                        generic_impl_methods.get(&key).cloned()
                    });
                if let Some(method_fn) = method_fn_opt {
                    // For struct-level generics, the generic params come from the struct,
                    // not the method. Use the struct's generics for substitution.
                    let struct_generics = self
                        .generics
                        .struct_defs
                        .get(struct_name)
                        .map(|s| s.generics.clone())
                        .unwrap_or_default();
                    let method_base_name = format!("{}_{}", struct_name, inst.base_name);

                    // Build instantiation mapping for method
                    self.generics
                        .fn_instantiations
                        .entry(method_base_name.clone())
                        .or_default()
                        .push((inst.type_args.clone(), inst.mangled_name.clone()));

                    // Build substitution map: use method's own generics if present,
                    // otherwise use struct-level generics.
                    let generic_params: Vec<_> = if !method_fn.generics.is_empty() {
                        method_fn
                            .generics
                            .iter()
                            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                            .collect()
                    } else {
                        struct_generics
                            .iter()
                            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                            .collect()
                    };
                    let mut substitutions: HashMap<String, ResolvedType> = generic_params
                        .iter()
                        .zip(inst.type_args.iter())
                        .map(|(g, t)| (g.name.node.to_string(), t.clone()))
                        .collect();

                    // Add const generic args to substitution map
                    for (name, _value) in &inst.const_args {
                        substitutions
                            .entry(name.clone())
                            .or_insert(ResolvedType::I64);
                    }

                    // Add "Self" substitution for struct methods returning Self
                    let struct_concrete = ResolvedType::Named {
                        name: struct_name.clone(),
                        generics: inst.type_args.clone(),
                    };
                    substitutions.insert("Self".to_string(), struct_concrete);

                    let params: Vec<_> = method_fn
                        .params
                        .iter()
                        .map(|p| {
                            if p.name.node == "self" {
                                (
                                    "self".to_string(),
                                    ResolvedType::Ref(Box::new(ResolvedType::Named {
                                        name: struct_name.clone(),
                                        generics: vec![],
                                    })),
                                    false,
                                )
                            } else {
                                let ty = self.ast_type_to_resolved(&p.ty.node);
                                let concrete_ty = vais_types::substitute_type(&ty, &substitutions);
                                (p.name.node.to_string(), concrete_ty, p.is_mut)
                            }
                        })
                        .collect();

                    let ret_type = method_fn
                        .ret_type
                        .as_ref()
                        .map(|t| {
                            let ty = self.ast_type_to_resolved(&t.node);
                            vais_types::substitute_type(&ty, &substitutions)
                        })
                        .unwrap_or(ResolvedType::Unit);

                    let mangled = inst.mangled_name.clone();
                    self.types.functions.insert(
                        mangled.clone(),
                        FunctionInfo {
                            signature: vais_types::FunctionSig {
                                name: mangled,
                                params,
                                ret: ret_type,
                                is_async: method_fn.is_async,
                                ..Default::default()
                            },
                            is_extern: false,
                            _extern_abi: None,
                        },
                    );
                }
            }
        }

        // Generate specialized struct types from instantiations
        for inst in instantiations {
            if let vais_types::InstantiationKind::Struct = inst.kind {
                if let Some(generic_struct) =
                    self.generics.struct_defs.get(&inst.base_name).cloned()
                {
                    self.generate_specialized_struct_type(&generic_struct, inst, &mut ir)?;
                }
            }
        }

        // Synthesize concrete struct instantiations from function instantiations.
        // The type checker sometimes only records a generic struct instantiation (e.g.,
        // Container$T from inside make_container<T>) without recording the concrete one
        // (Container$i64). We detect this by scanning function instantiation return types
        // and parameter types for references to generic structs with concrete type args.
        {
            // Collect synthetic struct instantiations we need to generate
            let mut synthetic_insts: Vec<vais_types::GenericInstantiation> = Vec::new();
            for inst in instantiations {
                if let vais_types::InstantiationKind::Function = inst.kind {
                    // Only process function instantiations with all-concrete type args
                    if inst
                        .type_args
                        .iter()
                        .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)))
                    {
                        continue;
                    }
                    // Build substitution map for this function instantiation
                    if let Some(generic_fn) = self.generics.function_templates.get(&inst.base_name)
                    {
                        let subst: HashMap<String, ResolvedType> = generic_fn
                            .generics
                            .iter()
                            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                            .zip(inst.type_args.iter())
                            .map(|(g, t)| (g.name.node.to_string(), t.clone()))
                            .collect();

                        // Check return type for generic struct references
                        let types_to_check: Vec<vais_ast::Type> = generic_fn
                            .ret_type
                            .as_ref()
                            .map(|t| vec![t.node.clone()])
                            .unwrap_or_default()
                            .into_iter()
                            .chain(generic_fn.params.iter().map(|p| p.ty.node.clone()))
                            .collect();

                        for ast_ty in &types_to_check {
                            if let Type::Named {
                                name: sname,
                                generics: type_params,
                            } = ast_ty
                            {
                                if !type_params.is_empty()
                                    && self.generics.struct_defs.contains_key(sname)
                                {
                                    // Substitute type args to get concrete types
                                    let concrete_args: Vec<ResolvedType> = type_params
                                        .iter()
                                        .map(|tp| {
                                            let resolved = self.ast_type_to_resolved(&tp.node);
                                            vais_types::substitute_type(&resolved, &subst)
                                        })
                                        .collect();

                                    // Only create instantiation if all type args are concrete
                                    let all_concrete = concrete_args.iter().all(|t| {
                                        !matches!(
                                            t,
                                            ResolvedType::Generic(_) | ResolvedType::Var(_)
                                        )
                                    });
                                    if all_concrete {
                                        let mangled =
                                            vais_types::mangle_name(sname, &concrete_args);
                                        if !self.generics.generated_structs.contains_key(&mangled) {
                                            synthetic_insts.push(
                                                vais_types::GenericInstantiation {
                                                    kind: vais_types::InstantiationKind::Struct,
                                                    base_name: sname.clone(),
                                                    mangled_name: mangled,
                                                    type_args: concrete_args,
                                                    const_args: Vec::new(),
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Generate the synthetic struct instantiations
            for inst in synthetic_insts {
                if let Some(generic_struct) =
                    self.generics.struct_defs.get(&inst.base_name).cloned()
                {
                    self.generate_specialized_struct_type(&generic_struct, &inst, &mut ir)?;
                }
            }
        }
        // Generate non-generic struct types (skip already-emitted specialized generics)
        for (name, info) in &self.types.structs {
            if self.generics.generated_structs.contains_key(name) {
                continue;
            }
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }
        // Generate enum types
        for (name, info) in &self.types.enums {
            ir.push_str(&self.generate_enum_type(name, info));
            ir.push('\n');
        }

        // Ensure well-known generic enum types are defined even when their AST definition
        // is not in the module (e.g., Result from std/result.vais not loaded transitively).
        // The codegen body uses the base name (e.g., %Result) for alloca/GEP, and the
        // "everything is i64" pattern means all enum payloads fit in { i32, { i64 } }.
        // Also register in types.enums so type_to_llvm uses the base name for generic
        // instances (e.g., Result<i64, VaisError> → %Result instead of %Result$i64_VaisError),
        // and get_tuple_variant_info can find Ok/Err/Some/None as enum variants.
        {
            use crate::types::{EnumInfo, EnumVariantFields, EnumVariantInfo};
            if !self.types.enums.contains_key("Result")
                && !self.types.structs.contains_key("Result")
            {
                write_ir!(ir, "%Result = type {{ i32, {{ i64 }} }}");
                self.types.enums.insert(
                    "Result".to_string(),
                    EnumInfo {
                        name: "Result".to_string(),
                        variants: vec![
                            EnumVariantInfo {
                                name: "Ok".to_string(),
                                _tag: 0,
                                // Use Generic("T") so resolve_variant_field_types can substitute
                                // the concrete type from match_type generics (fixes struct erasure)
                                fields: EnumVariantFields::Tuple(vec![ResolvedType::Generic(
                                    "T".to_string(),
                                )]),
                            },
                            EnumVariantInfo {
                                name: "Err".to_string(),
                                _tag: 1,
                                fields: EnumVariantFields::Tuple(vec![ResolvedType::Generic(
                                    "E".to_string(),
                                )]),
                            },
                        ],
                    },
                );
            }
            if !self.types.enums.contains_key("Option")
                && !self.types.structs.contains_key("Option")
            {
                write_ir!(ir, "%Option = type {{ i32, {{ i64 }} }}");
                self.types.enums.insert(
                    "Option".to_string(),
                    EnumInfo {
                        name: "Option".to_string(),
                        variants: vec![
                            EnumVariantInfo {
                                name: "None".to_string(),
                                _tag: 0,
                                fields: EnumVariantFields::Unit,
                            },
                            EnumVariantInfo {
                                name: "Some".to_string(),
                                _tag: 1,
                                // Use Generic("T") so resolve_variant_field_types can substitute
                                // the concrete type from match_type generics (fixes struct erasure)
                                fields: EnumVariantFields::Tuple(vec![ResolvedType::Generic(
                                    "T".to_string(),
                                )]),
                            },
                        ],
                    },
                );
            }
        }

        // Generate union types
        for (name, info) in &self.types.unions {
            ir.push_str(&self.generate_union_type(name, info));
            ir.push('\n');
        }

        // Generate function declarations (extern functions)
        // Prioritize non-aliased functions (key == name) over aliased ones (key != name)
        let mut declared_fns = std::collections::HashSet::new();
        let mut sorted_fns: Vec<_> = self
            .types
            .functions
            .iter()
            .filter(|(_, info)| info.is_extern)
            .collect();
        sorted_fns.sort_by_key(|(key, info)| if **key == info.signature.name { 0 } else { 1 });
        for (_, info) in &sorted_fns {
            // ROADMAP #9: skip `declare` for runtime intrinsics whose body is emitted
            // by `generate_helper_functions()` in this same main module.
            if crate::function_gen::runtime::is_runtime_intrinsic(&info.signature.name) {
                declared_fns.insert(info.signature.name.clone());
                continue;
            }
            if !declared_fns.contains(&info.signature.name) {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(info.signature.name.clone());
            }
        }

        // Generate string constants (after processing functions to collect all strings)
        let mut body_ir = String::new();
        // Generate specialized functions from instantiations
        eprintln!(
            "[P1.7] generating {} specialized functions",
            instantiations.len()
        );
        for inst in instantiations {
            if let vais_types::InstantiationKind::Function = inst.kind {
                if let Some(generic_fn) = self
                    .generics
                    .function_templates
                    .get(&inst.base_name)
                    .cloned()
                {
                    let spec_result =
                        stacker::maybe_grow(32 * 1024 * 1024, 64 * 1024 * 1024, || {
                            self.generate_specialized_function(&generic_fn, inst)
                        });
                    body_ir.push_str(&spec_result?);
                    body_ir.push('\n');
                }
            }
            // Generate specialized methods from Method instantiations
            if let vais_types::InstantiationKind::Method { ref struct_name } = inst.kind {
                if inst
                    .type_args
                    .iter()
                    .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)))
                {
                    continue;
                }
                let key = (struct_name.clone(), inst.base_name.clone());
                // Try method-level templates first, then struct-level generic methods,
                // then impl block methods on generic structs
                let method_fn_opt = method_templates
                    .get(&key)
                    .cloned()
                    .or_else(|| {
                        self.generics.struct_defs.get(struct_name).and_then(|s| {
                            s.methods
                                .iter()
                                .find(|m| m.node.name.node == inst.base_name)
                                .map(|m| std::rc::Rc::new(m.node.clone()))
                        })
                    })
                    .or_else(|| {
                        // Fall back to impl block methods on generic structs (e.g., X Vec<T> { ... })
                        generic_impl_methods.get(&key).cloned()
                    });
                if let Some(method_fn) = method_fn_opt {
                    // Reuse generate_specialized_function by treating the method as a function
                    // with the mangled base name (StructName_methodName)
                    let method_key = format!("{}_{}", struct_name, inst.base_name);
                    let method_inst = vais_types::GenericInstantiation {
                        kind: vais_types::InstantiationKind::Function,
                        base_name: method_key.clone(),
                        mangled_name: inst.mangled_name.clone(),
                        type_args: inst.type_args.clone(),
                        const_args: inst.const_args.clone(),
                    };
                    // Temporarily register the method as a function template
                    self.generics
                        .function_templates
                        .insert(method_key.clone(), method_fn);
                    let template = self
                        .generics
                        .function_templates
                        .get(&method_key)
                        .ok_or_else(|| {
                            CodegenError::InternalError(format!(
                                "method template '{}' missing after insert",
                                method_key
                            ))
                        })?
                        .clone();
                    body_ir.push_str(&self.generate_specialized_function(&template, &method_inst)?);
                    body_ir.push('\n');
                    // Clean up the temporary template
                    self.generics.function_templates.remove(&method_key);
                }
            }
        }
        // Second pass: generate non-generic function bodies.
        // For generic functions that have NO concrete specialized version (e.g., `identity<T>`
        // only appears inside another generic function body), generate a "fallback" un-specialized
        // version. This maintains backward compatibility with generate_module() behavior — such
        // functions work correctly when called with i64 (the default numeric type).
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    if f.generics.is_empty() {
                        match stacker::maybe_grow(32 * 1024 * 1024, 64 * 1024 * 1024, || {
                            self.generate_function_with_span(f, item.span)
                        }) {
                            Ok(ir_fragment) => {
                                body_ir.push_str(&ir_fragment);
                                body_ir.push('\n');
                            }
                            Err(e)
                                if self.multi_error_mode && self.collected_errors.len() < 200 =>
                            {
                                let span = self.last_error_span.unwrap_or(item.span);
                                self.collected_errors.push(crate::SpannedCodegenError {
                                    error: e,
                                    span: Some(span),
                                });
                            }
                            Err(e) => return Err(e),
                        }
                    } else if !self.generics.fn_instantiations.contains_key(&f.name.node)
                        && !self.generics.generated_functions.contains_key(&f.name.node)
                        && is_function_called_in_module(&f.name.node, module)
                    {
                        // Generic function with no concrete instantiation but IS called
                        // from within another function (e.g., identity<T> called from
                        // double<T>). Generate a fallback version with i64.
                        let param_names: Vec<String> =
                            f.generics.iter().map(|g| g.name.node.clone()).collect();
                        self.emit_warning(crate::CodegenWarning::UninstantiatedGeneric {
                            function_name: f.name.node.clone(),
                            params: param_names,
                        });
                        if let Ok(()) = self.register_function(f) {
                            if let Ok(fn_ir) = self.generate_function_with_span(f, item.span) {
                                body_ir.push_str(&fn_ir);
                                body_ir.push('\n');
                            }
                        }
                    }
                }
                Item::Struct(s) => {
                    if s.generics.is_empty() {
                        for method in &s.methods {
                            match self.generate_method_with_span(
                                &s.name.node,
                                &method.node,
                                method.span,
                            ) {
                                Ok(ir_fragment) => {
                                    body_ir.push_str(&ir_fragment);
                                    body_ir.push('\n');
                                }
                                Err(e)
                                    if self.multi_error_mode
                                        && self.collected_errors.len() < 200 =>
                                {
                                    let span = self.last_error_span.unwrap_or(method.span);
                                    self.collected_errors.push(crate::SpannedCodegenError {
                                        error: e,
                                        span: Some(span),
                                    });
                                }
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        // Phase 191: skip base generic method if specialized version exists
                        let base_method_name = format!("{}_{}", type_name, method.node.name.node);
                        let has_specialization = self
                            .generics
                            .generated_functions
                            .keys()
                            .any(|k| k.starts_with(&format!("{}$", base_method_name)));
                        if has_specialization {
                            continue;
                        }
                        match self.generate_method_with_span(&type_name, &method.node, method.span)
                        {
                            Ok(ir_fragment) => {
                                body_ir.push_str(&ir_fragment);
                                body_ir.push('\n');
                            }
                            Err(e)
                                if self.multi_error_mode && self.collected_errors.len() < 200 =>
                            {
                                let span = self.last_error_span.unwrap_or(method.span);
                                self.collected_errors.push(crate::SpannedCodegenError {
                                    error: e,
                                    span: Some(span),
                                });
                            }
                            Err(e) => return Err(e),
                        }
                    }
                }
                Item::Enum(_)
                | Item::Union(_)
                | Item::Use(_)
                | Item::Trait(_)
                | Item::TypeAlias(_)
                | Item::TraitAlias(_)
                | Item::Macro(_)
                | Item::ExternBlock(_)
                | Item::Const(_)
                | Item::Global(_)
                | Item::Error { .. } => {}
            }
        }

        self.emit_string_constants(&mut ir, true);
        self.emit_global_vars(&mut ir);
        self.emit_body_lambdas_vtables(&mut ir, &body_ir);

        // Add WASM runtime if targeting WebAssembly
        if self.target.is_wasm() {
            ir.push_str(&self.generate_wasm_runtime());
        }

        // Add helper functions
        if !matches!(self.target, TargetTriple::Wasm32Unknown) {
            ir.push_str(&self.generate_helper_functions());
        }

        // Add string helper functions if needed
        if self.needs_string_helpers {
            ir.push_str(&self.generate_string_helper_functions());
            if !self.target.is_wasm() {
                ir.push_str(&self.generate_string_extern_declarations());
            }
        }

        // Vec<str> container-ownership helpers (RFC-002 §4.1, §4.4).
        if self.generics.generated_structs.contains_key("Vec$str") {
            ir.push_str(&self.generate_vec_str_container_helpers());
        }

        // Add contract runtime declarations if any contracts are present
        if !self.contracts.contract_constants.is_empty() {
            ir.push_str(&self.generate_contract_declarations());
            ir.push_str(&self.generate_contract_string_constants());
        }

        // Add debug intrinsics if debug info is enabled
        if self.debug_info.is_enabled() && !self.target.is_wasm() {
            ir.push_str("\n; Debug intrinsics\n");
            ir.push_str("declare void @llvm.dbg.declare(metadata, metadata, metadata)\n");
            ir.push_str("declare void @llvm.dbg.value(metadata, metadata, metadata)\n");
        }

        // Add debug metadata
        if !self.target.is_wasm() {
            ir.push_str(&self.debug_info.finalize());
        }

        // Add ABI version metadata
        // ABI version is stored in @__vais_abi_version global constant

        Ok(ir)
    }
}

/// Check whether a function with the given name is called anywhere in the module
/// (excluding its own definition). This is used to determine whether an uninstantiated
/// generic function needs a fallback version -- if no other function calls it, it can
/// be safely omitted.
pub(super) fn is_function_called_in_module(name: &str, module: &Module) -> bool {
    fn expr_calls(name: &str, expr: &Expr) -> bool {
        match expr {
            Expr::Call { func, args } => {
                if let Expr::Ident(n) = &func.node {
                    if n == name {
                        return true;
                    }
                }
                if expr_calls(name, &func.node) {
                    return true;
                }
                args.iter().any(|a| expr_calls(name, &a.node))
            }
            Expr::Block(stmts) => stmts.iter().any(|s| stmt_calls(name, &s.node)),
            Expr::If { cond, then, else_ } => {
                expr_calls(name, &cond.node)
                    || then.iter().any(|s| stmt_calls(name, &s.node))
                    || else_
                        .as_ref()
                        .map(|e| ifelse_calls(name, e))
                        .unwrap_or(false)
            }
            Expr::Binary { left, right, .. } => {
                expr_calls(name, &left.node) || expr_calls(name, &right.node)
            }
            Expr::Unary { expr: inner, .. } => expr_calls(name, &inner.node),
            Expr::MethodCall { receiver, args, .. } => {
                expr_calls(name, &receiver.node) || args.iter().any(|a| expr_calls(name, &a.node))
            }
            Expr::StaticMethodCall { args, .. } => args.iter().any(|a| expr_calls(name, &a.node)),
            Expr::Field { expr: inner, .. } => expr_calls(name, &inner.node),
            Expr::Index { expr: inner, index } => {
                expr_calls(name, &inner.node) || expr_calls(name, &index.node)
            }
            Expr::Ref(inner)
            | Expr::Deref(inner)
            | Expr::Try(inner)
            | Expr::Unwrap(inner)
            | Expr::Await(inner)
            | Expr::Yield(inner) => expr_calls(name, &inner.node),
            Expr::Assign { target, value } | Expr::AssignOp { target, value, .. } => {
                expr_calls(name, &target.node) || expr_calls(name, &value.node)
            }
            Expr::Range { start, end, .. } => {
                start
                    .as_ref()
                    .map(|e| expr_calls(name, &e.node))
                    .unwrap_or(false)
                    || end
                        .as_ref()
                        .map(|e| expr_calls(name, &e.node))
                        .unwrap_or(false)
            }
            Expr::Match {
                expr: scrutinee,
                arms,
            } => {
                expr_calls(name, &scrutinee.node)
                    || arms.iter().any(|arm| expr_calls(name, &arm.body.node))
            }
            Expr::Lambda { body, .. } => expr_calls(name, &body.node),
            Expr::Ternary { cond, then, else_ } => {
                expr_calls(name, &cond.node)
                    || expr_calls(name, &then.node)
                    || expr_calls(name, &else_.node)
            }
            Expr::Array(elems) | Expr::Tuple(elems) => {
                elems.iter().any(|e| expr_calls(name, &e.node))
            }
            Expr::StructLit { fields, .. } => fields.iter().any(|(_, e)| expr_calls(name, &e.node)),
            Expr::Cast { expr: inner, .. } | Expr::Comptime { body: inner } => {
                expr_calls(name, &inner.node)
            }
            Expr::Loop { iter, body, .. } => {
                iter.as_ref()
                    .map(|e| expr_calls(name, &e.node))
                    .unwrap_or(false)
                    || body.iter().any(|s| stmt_calls(name, &s.node))
            }
            Expr::While { condition, body } => {
                expr_calls(name, &condition.node) || body.iter().any(|s| stmt_calls(name, &s.node))
            }
            _ => false,
        }
    }

    fn ifelse_calls(name: &str, ie: &IfElse) -> bool {
        match ie {
            IfElse::Else(stmts) => stmts.iter().any(|s| stmt_calls(name, &s.node)),
            IfElse::ElseIf(cond, then_stmts, else_opt) => {
                expr_calls(name, &cond.node)
                    || then_stmts.iter().any(|s| stmt_calls(name, &s.node))
                    || else_opt
                        .as_ref()
                        .map(|e| ifelse_calls(name, e))
                        .unwrap_or(false)
            }
        }
    }

    fn stmt_calls(name: &str, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(e) => expr_calls(name, &e.node),
            Stmt::Let { value, .. } => expr_calls(name, &value.node),
            Stmt::LetDestructure { value, .. } => expr_calls(name, &value.node),
            Stmt::Return(Some(e)) | Stmt::Break(Some(e)) | Stmt::Defer(e) => {
                expr_calls(name, &e.node)
            }
            _ => false,
        }
    }

    fn body_calls(name: &str, body: &FunctionBody) -> bool {
        match body {
            FunctionBody::Expr(e) => expr_calls(name, &e.node),
            FunctionBody::Block(stmts) => stmts.iter().any(|s| stmt_calls(name, &s.node)),
        }
    }

    for item in &module.items {
        match &item.node {
            Item::Function(f) => {
                // Don't check the function's own body
                if f.name.node == name {
                    continue;
                }
                if body_calls(name, &f.body) {
                    return true;
                }
            }
            Item::Impl(impl_block) => {
                for method in &impl_block.methods {
                    if body_calls(name, &method.node.body) {
                        return true;
                    }
                }
            }
            Item::Struct(s) => {
                for method in &s.methods {
                    if body_calls(name, &method.node.body) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}
