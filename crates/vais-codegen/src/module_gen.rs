//! Module generation methods

use super::*;

impl CodeGenerator {
    pub fn generate_module_subset(
        &mut self,
        full_module: &Module,
        item_indices: &[usize],
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
            eprintln!(
                "Warning: {} item indices out of bounds (>= {}): {:?}",
                out_of_bounds.len(),
                items_len,
                out_of_bounds
            );
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
                    self.register_function(f)?;
                    if is_ours {
                        module_functions.insert(f.name.node.clone());
                    }
                }
                Item::Struct(s) => {
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

        // Generate struct types (all modules need these)
        for (name, info) in &self.types.structs {
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
        for &idx in &valid_indices {
            let item = &full_module.items[idx];
            match &item.node {
                Item::Function(f) => {
                    body_ir.push_str(&self.generate_function_with_span(f, item.span)?);
                    body_ir.push('\n');
                }
                Item::Struct(s) => {
                    for method in &s.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &s.name.node,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
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

    pub fn generate_module(&mut self, module: &Module) -> CodegenResult<String> {
        let mut ir = String::new();

        self.emit_module_header(&mut ir);

        // First pass: collect declarations
        for item in &module.items {
            match &item.node {
                Item::Function(f) => self.register_function(f)?,
                Item::Struct(s) => {
                    self.register_struct(s)?;
                    // Register struct methods
                    for method in &s.methods {
                        self.register_method(&s.name.node, &method.node)?;
                    }
                }
                Item::Enum(e) => self.register_enum(e)?,
                Item::Union(u) => self.register_union(u)?,
                Item::Impl(impl_block) => {
                    // Register impl methods
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
                Item::Use(_) => {
                    // Use statements are handled at the compiler level (AST merging)
                    // No code generation needed for imports
                }
                Item::Trait(trait_def) => {
                    // Register trait for vtable generation
                    self.register_trait_from_ast(trait_def);
                }
                Item::TypeAlias(_) | Item::TraitAlias(_) => {
                    // Type/trait aliases don't generate code
                }
                Item::Macro(_) => {
                    // Macro definitions are expanded at compile time
                    // No runtime code generation needed
                }
                Item::Error { .. } => {
                    // Error nodes indicate parsing failures
                    // Skip them during code generation - errors were reported during parsing
                }
                Item::ExternBlock(extern_block) => {
                    // Register extern functions
                    for func in &extern_block.functions {
                        self.register_extern_function(func, &extern_block.abi)?;
                    }
                }
                Item::Const(const_def) => {
                    // Register constant for code generation
                    self.register_const(const_def)?;
                }
                Item::Global(global_def) => {
                    // Register global variable
                    self.register_global(global_def)?;
                }
            }
        }

        // Generate struct types
        for (name, info) in &self.types.structs {
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }

        // Generate enum types
        for (name, info) in &self.types.enums {
            ir.push_str(&self.generate_enum_type(name, info));
            ir.push('\n');
        }

        // Generate union types
        for (name, info) in &self.types.unions {
            ir.push_str(&self.generate_union_type(name, info));
            ir.push('\n');
        }

        // Generate function declarations (deduplicate by actual function name)
        // Prioritize non-aliased functions (key == name) over aliased ones (key != name)
        // to ensure correct C type signatures in declare statements
        let mut declared_fns = std::collections::HashSet::new();
        let mut sorted_fns: Vec<_> = self
            .types
            .functions
            .iter()
            .filter(|(_, info)| info.is_extern)
            .collect();
        sorted_fns.sort_by_key(|(key, info)| if **key == info.signature.name { 0 } else { 1 });
        for (_, info) in &sorted_fns {
            if !declared_fns.contains(&info.signature.name) {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(info.signature.name.clone());
            }
        }

        // Generate string constants (after processing functions to collect all strings)
        let mut body_ir = String::new();

        // Second pass: generate function bodies
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    body_ir.push_str(&self.generate_function_with_span(f, item.span)?);
                    body_ir.push('\n');
                }
                Item::Struct(s) => {
                    // Generate methods for this struct
                    for method in &s.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &s.name.node,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                Item::Impl(impl_block) => {
                    // Generate methods from impl block
                    // Get the type name from target_type
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
                Item::Enum(_)
                | Item::Union(_)
                | Item::Use(_)
                | Item::Trait(_)
                | Item::TypeAlias(_)
                | Item::TraitAlias(_)
                | Item::Macro(_)
                | Item::ExternBlock(_) => {
                    // Already handled in first pass or no code generation needed
                }
                Item::Const(_) | Item::Global(_) => {
                    // Constants and globals are handled in first pass
                }
                Item::Error { .. } => {
                    // Error nodes are skipped - already logged in first pass
                }
            }
        }

        self.emit_string_constants(&mut ir, true);
        self.emit_body_lambdas_vtables(&mut ir, &body_ir);

        // Add WASM runtime functions if targeting WebAssembly
        if self.target.is_wasm() {
            ir.push_str(&self.generate_wasm_runtime());
        }

        // Add helper functions for memory operations (skip for wasm32-unknown-unknown,
        // which provides its own implementations)
        if !matches!(self.target, TargetTriple::Wasm32Unknown) {
            ir.push_str(&self.generate_helper_functions());
        } else {
            // For wasm32-unknown-unknown, only emit minimal helpers that don't conflict
            ir.push_str("\n; Minimal helpers for WASM\n");
        }

        // Add string helper functions if needed
        if self.needs_string_helpers {
            ir.push_str(&self.generate_string_helper_functions());
            if !self.target.is_wasm() {
                ir.push_str(&self.generate_string_extern_declarations());
            }
        }

        // Add contract runtime declarations if any contracts are present
        if !self.contracts.contract_constants.is_empty() {
            ir.push_str(&self.generate_contract_declarations());
            ir.push_str(&self.generate_contract_string_constants());
        }

        // Add debug intrinsic declaration if debug info is enabled
        if self.debug_info.is_enabled() && !self.target.is_wasm() {
            ir.push_str("\n; Debug intrinsics\n");
            ir.push_str("declare void @llvm.dbg.declare(metadata, metadata, metadata)\n");
            ir.push_str("declare void @llvm.dbg.value(metadata, metadata, metadata)\n");
        }

        // Add debug metadata at the end
        if !self.target.is_wasm() {
            ir.push_str(&self.debug_info.finalize());
        }

        // Add WASM import/export metadata attributes
        if self.target.is_wasm() && (!self.wasm_imports.is_empty() || !self.wasm_exports.is_empty())
        {
            ir.push_str("\n; WASM import/export metadata\n");
            ir.push_str(&self.generate_wasm_metadata());
        }

        // Add ABI version metadata
        // ABI version is stored in @__vais_abi_version global constant

        Ok(ir)
    }

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
                    } else {
                        self.register_struct(s)?;
                        for method in &s.methods {
                            self.register_method(&s.name.node, &method.node)?;
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

        // Build generic function instantiation mapping and register specialized function signatures.
        // Only process concrete instantiations (all type args are resolved, non-generic).
        // Non-concrete instantiations (e.g., make_container$T from inside a generic function body)
        // are skipped — they would produce unresolved generic IR like `@identity$T`.
        for inst in instantiations {
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
                    let substitutions: HashMap<String, ResolvedType> = generic_fn
                        .generics
                        .iter()
                        .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                        .zip(inst.type_args.iter())
                        .map(|(g, t)| (g.name.node.to_string(), t.clone()))
                        .collect();

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

                    self.types.functions.insert(
                        inst.mangled_name.clone(),
                        FunctionInfo {
                            signature: vais_types::FunctionSig {
                                name: inst.mangled_name.clone(),
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
                            if let Type::Named { name: sname, generics: type_params } = ast_ty {
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
                                        if !self
                                            .generics
                                            .generated_structs
                                            .contains_key(&mangled)
                                        {
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
            if !declared_fns.contains(&info.signature.name) {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(info.signature.name.clone());
            }
        }

        // Generate string constants (after processing functions to collect all strings)
        let mut body_ir = String::new();

        // Generate specialized functions from instantiations
        for inst in instantiations {
            if let vais_types::InstantiationKind::Function = inst.kind {
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
                        body_ir.push_str(&self.generate_function_with_span(f, item.span)?);
                        body_ir.push('\n');
                    } else if !self.generics.fn_instantiations.contains_key(&f.name.node)
                        && !self
                            .generics
                            .generated_functions
                            .contains_key(&f.name.node)
                    {
                        // Generic function with no concrete instantiation — generate as fallback
                        // (same as generate_module() would do: register and generate without specialization)
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
                            body_ir.push_str(&self.generate_method_with_span(
                                &s.name.node,
                                &method.node,
                                method.span,
                            )?);
                            body_ir.push('\n');
                        }
                    }
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
