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
}
