//! Module subset generation for parallel builds.
//!
//! Generates LLVM IR for a subset of items from a full module,
//! with cross-module extern declarations for items in other subsets.

use super::*;

impl CodeGenerator {
    pub fn generate_module_subset(
        &mut self,
        full_module: &Module,
        item_indices: &[usize],
        instantiations: &[vais_types::GenericInstantiation],
        is_main_module: bool,
    ) -> CodegenResult<String> {
        // Validate item_indices are within bounds
        let items_len = full_module.items.len();
        let mut out_of_bounds = Vec::new();
        for &idx in item_indices {
            if idx >= items_len {
                out_of_bounds.push(idx);
            }
        }
        if !out_of_bounds.is_empty() {
            return Err(crate::CodegenError::InternalError(format!(
                "{} item indices out of bounds (>= {}): {:?}",
                out_of_bounds.len(),
                items_len,
                out_of_bounds
            )));
        }

        // Filter to valid indices only
        let valid_indices: Vec<usize> = item_indices
            .iter()
            .copied()
            .filter(|&i| i < items_len)
            .collect();

        let mut ir = String::new();
        let index_set: std::collections::HashSet<usize> = valid_indices.iter().copied().collect();

        self.emit_module_header(&mut ir);

        // Snapshot builtin function keys (registered in constructor, before AST items)
        // These should NOT appear as cross-module extern declarations.
        let builtin_fn_keys: std::collections::HashSet<String> =
            self.types.functions.keys().cloned().collect();

        // First pass: register ALL type definitions (structs, enums, unions) from full module
        // and register functions — tracking which are "ours" vs external
        let mut module_functions: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for (idx, item) in full_module.items.iter().enumerate() {
            let is_ours = index_set.contains(&idx);
            match &item.node {
                Item::Function(f) => {
                    self.types.declared_functions.insert(f.name.node.clone());
                    if !f.generics.is_empty() {
                        // Store generic function template for later specialization
                        self.generics
                            .function_templates
                            .insert(f.name.node.clone(), std::rc::Rc::new(f.clone()));
                    } else {
                        self.register_function(f)?;
                    }
                    if is_ours {
                        module_functions.insert(f.name.node.clone());
                    }
                }
                Item::Struct(s) => {
                    if !s.generics.is_empty() {
                        self.generics
                            .struct_defs
                            .insert(s.name.node.clone(), std::rc::Rc::new(s.clone()));
                    }
                    self.register_struct(s)?;
                    for method in &s.methods {
                        self.register_method(&s.name.node, &method.node)?;
                        if is_ours {
                            module_functions
                                .insert(format!("{}_{}", s.name.node, method.node.name.node));
                        }
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
                        if is_ours {
                            module_functions
                                .insert(format!("{}_{}", type_name, method.node.name.node));
                        }
                    }
                    // Add impl methods to struct_defs for generic monomorphization
                    if let Some(struct_def) = self.generics.struct_defs.get_mut(&type_name) {
                        let struct_mut = std::rc::Rc::make_mut(struct_def);
                        for method in &impl_block.methods {
                            if !struct_mut.methods.iter().any(|m| m.node.name.node == method.node.name.node) {
                                struct_mut.methods.push(method.clone());
                            }
                        }
                    }
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

        // Pre-build method template lookup: (struct_name, method_name) -> Function AST
        let mut method_templates: HashMap<(String, String), std::rc::Rc<Function>> = HashMap::new();
        // Also collect methods from impl blocks on generic structs (struct-level generics)
        let mut generic_impl_methods: HashMap<(String, String), std::rc::Rc<Function>> = HashMap::new();
        for item in &full_module.items {
            match &item.node {
                Item::Impl(impl_block) => {
                    if let Type::Named { name, generics: type_params } = &impl_block.target_type.node {
                        let is_generic_impl = !impl_block.generics.is_empty()
                            || !type_params.is_empty();
                        for method in &impl_block.methods {
                            if !method.node.generics.is_empty() {
                                method_templates.insert(
                                    (name.clone(), method.node.name.node.clone()),
                                    std::rc::Rc::new(method.node.clone()),
                                );
                            } else if is_generic_impl {
                                generic_impl_methods.insert(
                                    (name.clone(), method.node.name.node.clone()),
                                    std::rc::Rc::new(method.node.clone()),
                                );
                            }
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

        // Register specialized function signatures from instantiations
        // (mirrors generate_module_with_instantiations lines 128-324)
        for inst in instantiations {
            if let vais_types::InstantiationKind::Function = inst.kind {
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
                    self.generics
                        .fn_instantiations
                        .entry(inst.base_name.clone())
                        .or_default()
                        .push((inst.type_args.clone(), inst.mangled_name.clone()));

                    let mut substitutions: HashMap<String, ResolvedType> = generic_fn
                        .generics
                        .iter()
                        .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                        .zip(inst.type_args.iter())
                        .map(|(g, t)| (g.name.node.to_string(), t.clone()))
                        .collect();

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
            // Handle Method instantiations
            if let vais_types::InstantiationKind::Method { ref struct_name } = inst.kind {
                if inst
                    .type_args
                    .iter()
                    .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)))
                {
                    continue;
                }
                let key = (struct_name.clone(), inst.base_name.clone());
                let method_fn_opt = method_templates.get(&key).cloned().or_else(|| {
                    self.generics
                        .struct_defs
                        .get(struct_name)
                        .and_then(|s| {
                            s.methods
                                .iter()
                                .find(|m| m.node.name.node == inst.base_name)
                                .map(|m| std::rc::Rc::new(m.node.clone()))
                        })
                }).or_else(|| {
                    generic_impl_methods.get(&key).cloned()
                });
                if let Some(method_fn) = method_fn_opt {
                    let struct_generics = self
                        .generics
                        .struct_defs
                        .get(struct_name)
                        .map(|s| s.generics.clone())
                        .unwrap_or_default();
                    let method_base_name = format!("{}_{}", struct_name, inst.base_name);

                    self.generics
                        .fn_instantiations
                        .entry(method_base_name.clone())
                        .or_default()
                        .push((inst.type_args.clone(), inst.mangled_name.clone()));

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

                    for (name, _value) in &inst.const_args {
                        substitutions
                            .entry(name.clone())
                            .or_insert(ResolvedType::I64);
                    }

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

        // Generate specialized struct types from explicit Struct instantiations
        for inst in instantiations {
            if let vais_types::InstantiationKind::Struct = inst.kind {
                if let Some(generic_struct) =
                    self.generics.struct_defs.get(&inst.base_name).cloned()
                {
                    self.generate_specialized_struct_type(&generic_struct, inst, &mut ir)?;
                }
            }
        }

        // Synthesize concrete struct instantiations from function instantiations
        {
            let mut synthetic_insts: Vec<vais_types::GenericInstantiation> = Vec::new();
            for inst in instantiations {
                if let vais_types::InstantiationKind::Function = inst.kind {
                    if inst
                        .type_args
                        .iter()
                        .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)))
                    {
                        continue;
                    }
                    if let Some(generic_fn) =
                        self.generics.function_templates.get(&inst.base_name)
                    {
                        let subst: HashMap<String, ResolvedType> = generic_fn
                            .generics
                            .iter()
                            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                            .zip(inst.type_args.iter())
                            .map(|(g, t)| (g.name.node.to_string(), t.clone()))
                            .collect();

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
                                    let concrete_args: Vec<ResolvedType> = type_params
                                        .iter()
                                        .map(|tp| {
                                            let resolved = self.ast_type_to_resolved(&tp.node);
                                            vais_types::substitute_type(&resolved, &subst)
                                        })
                                        .collect();

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
                                            synthetic_insts.push(vais_types::GenericInstantiation {
                                                kind: vais_types::InstantiationKind::Struct,
                                                base_name: sname.clone(),
                                                mangled_name: mangled,
                                                type_args: concrete_args,
                                                const_args: Vec::new(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for inst in synthetic_insts {
                if let Some(generic_struct) =
                    self.generics.struct_defs.get(&inst.base_name).cloned()
                {
                    self.generate_specialized_struct_type(&generic_struct, &inst, &mut ir)?;
                }
            }
        }

        // Generate non-specialized struct types (skip already-emitted specialized generics)
        for (name, info) in &self.types.structs {
            if self.generics.generated_structs.contains_key(name) {
                continue;
            }
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }
        for (name, info) in &self.types.enums {
            ir.push_str(&self.generate_enum_type(name, info));
            ir.push('\n');
        }
        for (name, info) in &self.types.unions {
            ir.push_str(&self.generate_union_type(name, info));
            ir.push('\n');
        }

        // Generate extern declarations for ALL extern functions (is_extern = true)
        // Builtin helpers (is_extern = false) are handled separately below.
        let mut declared_fns = std::collections::HashSet::new();
        let mut sorted_fns: Vec<_> = self
            .types
            .functions
            .iter()
            .filter(|(_, info)| info.is_extern)
            .collect();
        sorted_fns.sort_by_key(|(key, info)| if **key == info.signature.name { 0 } else { 1 });
        for (key, info) in &sorted_fns {
            if !declared_fns.contains(&info.signature.name)
                && !module_functions.contains(&info.signature.name)
                && !module_functions.contains(*key)
            {
                if !is_main_module && info.signature.name == "fopen_ptr" {
                    // Non-main modules should declare fopen_ptr (not define it).
                    // The wrapper definition lives in the main module only.
                    let params: Vec<_> = info
                        .signature
                        .params
                        .iter()
                        .map(|(_, ty, _)| self.type_to_llvm(ty))
                        .collect();
                    let ret = self.type_to_llvm(&info.signature.ret);
                    ir.push_str(&format!(
                        "declare {} @fopen_ptr({})\n",
                        ret,
                        params.join(", ")
                    ));
                } else {
                    ir.push_str(&self.generate_extern_decl(info));
                    ir.push('\n');
                }
                declared_fns.insert(info.signature.name.clone());
            }
        }

        // Generate extern declarations for cross-module Vais functions
        // (functions registered from AST but not in this module's item set)
        // Skip builtins — they are handled by generate_helper_functions() or the non-main extern block.
        for (name, info) in &self.types.functions {
            if !info.is_extern
                && !module_functions.contains(name)
                && !declared_fns.contains(name)
                && !builtin_fn_keys.contains(name)
            {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(name.clone());
            }
        }

        // Generate function bodies only for this module's items
        let mut body_ir = String::new();

        // Generate specialized function/method bodies from instantiations —
        // only for functions/methods defined in this module subset
        for inst in instantiations {
            if let vais_types::InstantiationKind::Function = inst.kind {
                // Only generate body if this module owns the generic function
                if !module_functions.contains(&inst.base_name) {
                    continue;
                }
                if let Some(generic_fn) = self
                    .generics
                    .function_templates
                    .get(&inst.base_name)
                    .cloned()
                {
                    body_ir.push_str(&self.generate_specialized_function(&generic_fn, inst)?);
                    body_ir.push('\n');
                }
            }
            if let vais_types::InstantiationKind::Method { ref struct_name } = inst.kind {
                if inst
                    .type_args
                    .iter()
                    .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)))
                {
                    continue;
                }
                let method_key = format!("{}_{}", struct_name, inst.base_name);
                // Generate body if this module owns the base method OR the struct's impl
                if !module_functions.contains(&method_key) {
                    continue;
                }
                let key = (struct_name.clone(), inst.base_name.clone());
                let method_fn_opt = method_templates.get(&key).cloned().or_else(|| {
                    self.generics
                        .struct_defs
                        .get(struct_name)
                        .and_then(|s| {
                            s.methods
                                .iter()
                                .find(|m| m.node.name.node == inst.base_name)
                                .map(|m| std::rc::Rc::new(m.node.clone()))
                        })
                }).or_else(|| {
                    generic_impl_methods.get(&key).cloned()
                });
                if let Some(method_fn) = method_fn_opt {
                    let method_inst = vais_types::GenericInstantiation {
                        kind: vais_types::InstantiationKind::Function,
                        base_name: method_key.clone(),
                        mangled_name: inst.mangled_name.clone(),
                        type_args: inst.type_args.clone(),
                        const_args: inst.const_args.clone(),
                    };
                    // Set up struct-level generic substitutions before specialization
                    // This ensures T → concrete type is available for self.field access
                    let struct_generics_params = self.generics.struct_defs.get(struct_name)
                        .map(|s| s.generics.clone()).unwrap_or_default();
                    for (param, concrete) in struct_generics_params.iter()
                        .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                        .zip(inst.type_args.iter())
                    {
                        self.generics.substitutions.insert(param.name.node.clone(), concrete.clone());
                    }
                    // Also set Self → struct type
                    self.generics.substitutions.insert("Self".to_string(), ResolvedType::Named {
                        name: struct_name.clone(),
                        generics: inst.type_args.clone(),
                    });

                    // Temporarily register the method as a function template
                    // with the struct's generics so generate_specialized_function can find them
                    let mut method_fn_with_generics = (*method_fn).clone();
                    if method_fn_with_generics.generics.is_empty() {
                        method_fn_with_generics.generics = struct_generics_params
                            .iter()
                            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                            .cloned()
                            .collect();
                    }
                    let method_fn_rc = std::rc::Rc::new(method_fn_with_generics);
                    self.generics
                        .function_templates
                        .insert(method_key.clone(), method_fn_rc);
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
                    let specialized_ir = self.generate_specialized_function(&template, &method_inst)?;
                    body_ir.push_str(&specialized_ir);
                    body_ir.push('\n');
                    self.generics.function_templates.remove(&method_key);
                }
            }
        }

        // Generate non-generic function/method bodies for this module's items
        for &idx in &valid_indices {
            let item = &full_module.items[idx];
            match &item.node {
                Item::Function(f) => {
                    if f.generics.is_empty() {
                        body_ir.push_str(&self.generate_function_with_span(f, item.span)?);
                        body_ir.push('\n');
                    }
                    // generic functions are handled by the instantiation loop above
                }
                Item::Struct(s) => {
                    if s.generics.is_empty() {
                        for method in &s.methods {
                            body_ir.push_str(&self.generate_method_with_span(
                                &s.name.node,
                                &method.node,
                                method.span,
                            )?);
                            body_ir.push('\n');
                        }
                    }
                    // generic struct methods handled by instantiation loop above
                }
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &type_name,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                _ => {} // Other items handled in registration pass
            }
        }

        self.emit_string_constants(&mut ir, is_main_module);
        self.emit_global_vars(&mut ir);
        self.emit_body_lambdas_vtables(&mut ir, &body_ir);

        // Add WASM runtime for main module
        if is_main_module && self.target.is_wasm() {
            ir.push_str(&self.generate_wasm_runtime());
        }

        if is_main_module {
            // Main module defines all helper functions
            if !matches!(self.target, TargetTriple::Wasm32Unknown) {
                ir.push_str(&self.generate_helper_functions());
            }
        } else {
            // Non-main modules declare builtin helpers as extern
            // (these are defined by generate_helper_functions() in the main module)
            ir.push_str("\n; Extern declarations for runtime helpers\n");
            let mut sorted_helpers: Vec<_> = builtin_fn_keys.iter().collect();
            sorted_helpers.sort();
            for key in sorted_helpers {
                if let Some(info) = self.types.functions.get(key) {
                    if !info.is_extern
                        && !declared_fns.contains(&info.signature.name)
                        && !module_functions.contains(key)
                        && !module_functions.contains(&info.signature.name)
                    {
                        ir.push_str(&self.generate_extern_decl(info));
                        ir.push('\n');
                        declared_fns.insert(info.signature.name.clone());
                    }
                }
            }
        }

        if self.needs_string_helpers {
            if is_main_module {
                ir.push_str(&self.generate_string_helper_functions());
            }
            if !self.target.is_wasm() {
                ir.push_str(&self.generate_string_extern_declarations());
            }
        }

        if !self.contracts.contract_constants.is_empty() {
            ir.push_str(&self.generate_contract_declarations());
            ir.push_str(&self.generate_contract_string_constants());
        }

        if self.debug_info.is_enabled() && !self.target.is_wasm() {
            ir.push_str("\n; Debug intrinsics\n");
            ir.push_str("declare void @llvm.dbg.declare(metadata, metadata, metadata)\n");
            ir.push_str("declare void @llvm.dbg.value(metadata, metadata, metadata)\n");
        }

        if !self.target.is_wasm() {
            ir.push_str(&self.debug_info.finalize());
        }

        // Add WASM import/export metadata attributes
        if self.target.is_wasm() && (!self.wasm_imports.is_empty() || !self.wasm_exports.is_empty())
        {
            ir.push_str("\n; WASM import/export metadata\n");
            ir.push_str(&self.generate_wasm_metadata());
        }

        Ok(ir)
    }
}
