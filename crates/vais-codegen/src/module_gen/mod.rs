//! Module generation methods.
//!
//! This module provides three main entry points for generating LLVM IR from a Vais module:
//!
//! - `generate_module` — Full single-module compilation (most common path)
//! - `generate_module_subset` — Subset compilation for parallel builds
//! - `generate_module_with_instantiations` — Compilation with generic specialization
//!
//! # Submodules
//!
//! - `subset` — `generate_module_subset` implementation
//! - `instantiations` — `generate_module_with_instantiations` + call analysis helpers

use super::*;

mod instantiations;
mod subset;

impl CodeGenerator {
    pub fn generate_module(&mut self, module: &Module) -> CodegenResult<String> {
        // Pre-allocate IR output: ~100 bytes per item is a reasonable estimate
        let estimated_size = module.items.len() * 100 + 4096;
        let mut ir = String::with_capacity(estimated_size);

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
            // ROADMAP #9: skip `declare` for runtime intrinsics whose body is emitted
            // by `generate_helper_functions()` in this same module (single-module path
            // always acts as the main module).
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
        // Pre-allocate body IR: ~200 bytes per function is reasonable
        let fn_count = module
            .items
            .iter()
            .filter(|i| matches!(i.node, Item::Function(_)))
            .count();
        let mut body_ir = String::with_capacity(fn_count * 200 + 2048);

        // Second pass: generate function bodies
        // In multi_error_mode, errors are collected and stub functions emitted
        // so that multiple codegen errors can be reported at once.
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    // Skip generic functions — they need monomorphization to resolve
                    // trait method calls (e.g., x.method1() on generic T).
                    // Without concrete type args, we'd emit undefined symbols.
                    // Use generate_module_with_instantiations() for generic codegen.
                    if !f.generics.is_empty() {
                        // Fallback: if the generic function IS called from within this module
                        // but TC produced no instantiations (e.g., T cannot be inferred from
                        // args because it only appears in a struct field that TC can't track),
                        // emit a fallback version treating all type params as i64.
                        if instantiations::is_function_called_in_module(&f.name.node, module) {
                            let param_names: Vec<String> =
                                f.generics.iter().map(|g| g.name.node.clone()).collect();
                            self.emit_warning(CodegenWarning::UninstantiatedGeneric {
                                function_name: f.name.node.clone(),
                                params: param_names,
                            });
                            if let Ok(()) = self.register_function(f) {
                                match self.generate_function_with_span(f, item.span) {
                                    Ok(ir_fragment) => {
                                        body_ir.push_str(&ir_fragment);
                                        body_ir.push('\n');
                                    }
                                    Err(e)
                                        if self.multi_error_mode
                                            && self.collected_errors.len() < 200 =>
                                    {
                                        let span = self.last_error_span.unwrap_or(item.span);
                                        self.collected_errors.push(SpannedCodegenError {
                                            error: e,
                                            span: Some(span),
                                        });
                                    }
                                    Err(_) => {} // Ignore errors in fallback generic codegen
                                }
                            }
                        }
                        continue;
                    }
                    match self.generate_function_with_span(f, item.span) {
                        Ok(ir_fragment) => {
                            body_ir.push_str(&ir_fragment);
                            body_ir.push('\n');
                        }
                        Err(e) if self.multi_error_mode && self.collected_errors.len() < 200 => {
                            let span = self.last_error_span.unwrap_or(item.span);
                            self.collected_errors.push(SpannedCodegenError {
                                error: e,
                                span: Some(span),
                            });
                        }
                        Err(e) => return Err(e),
                    }
                }
                Item::Struct(s) => {
                    // Generate methods for this struct
                    for method in &s.methods {
                        // Phase 191: skip base method if specialized version exists
                        let base_method_name = format!("{}_{}", s.name.node, method.node.name.node);
                        let has_specialization = self
                            .generics
                            .generated_functions
                            .keys()
                            .any(|k| k.starts_with(&format!("{}$", base_method_name)));
                        if has_specialization {
                            continue;
                        }
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
                                if self.multi_error_mode && self.collected_errors.len() < 200 =>
                            {
                                let span = self.last_error_span.unwrap_or(method.span);
                                self.collected_errors.push(SpannedCodegenError {
                                    error: e,
                                    span: Some(span),
                                });
                            }
                            Err(e) => return Err(e),
                        }
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
                        // Phase 191: skip base generic method if a specialized version
                        // was already generated. The specialized version (e.g., Vec_new$f32)
                        // has correct typed params; the base version (Vec_new) has i64-uniform
                        // params that produce type-mismatched IR with specialized struct layouts.
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
                                self.collected_errors.push(SpannedCodegenError {
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
        self.emit_global_vars(&mut ir);
        self.emit_body_lambdas_vtables(&mut ir, &body_ir);

        // Emit on-demand specialized functions (e.g., Vec$str_push generated during
        // method call processing when the TC didn't provide instantiation records)
        if !self.fn_ctx.pending_specialized_ir.is_empty() {
            ir.push_str("\n; On-demand specialized functions\n");
            for spec_ir in self.fn_ctx.pending_specialized_ir.drain(..) {
                ir.push_str(&spec_ir);
                ir.push('\n');
            }
        }

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

        // Vec<str> container-ownership helpers (RFC-002 §4.1, §4.4).
        // Emit only when a Vec<str> specialization exists in this module.
        if self.generics.generated_structs.contains_key("Vec$str") {
            ir.push_str(&self.generate_vec_str_container_helpers());
        }

        // Struct shallow-free helpers (RFC-002 §4.2, Phase 191 #2b-C).
        for struct_name in &self.needs_struct_shallow.clone() {
            if let Some(info) = self.types.structs.get(struct_name) {
                let field_count = info.fields.len();
                let heap_fields = info.heap_fields.clone();
                ir.push_str(&self.generate_struct_shallow_free_helper(
                    struct_name,
                    field_count,
                    &heap_fields,
                ));
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
}
