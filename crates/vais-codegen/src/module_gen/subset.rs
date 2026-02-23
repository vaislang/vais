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
}
