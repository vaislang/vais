//! Trait dispatch, vtable generation, and dynamic method calls

use super::*;

impl CodeGenerator {
    /// Register a trait definition for vtable generation
    pub fn register_trait(&mut self, trait_def: vais_types::TraitDef) {
        self.types
            .trait_defs
            .insert(trait_def.name.clone(), trait_def);
    }

    /// Register a trait from AST definition (converts AST Trait to TraitDef)
    #[inline(never)]
    pub(crate) fn register_trait_from_ast(&mut self, t: &vais_ast::Trait) {
        let mut methods = HashMap::new();
        for m in &t.methods {
            let params: Vec<(String, ResolvedType, bool)> = m
                .params
                .iter()
                .map(|p| {
                    let ty = if p.name.node == "self" {
                        // self parameter is a pointer to the implementing type
                        ResolvedType::I64 // placeholder, resolved at call site
                    } else {
                        self.ast_type_to_resolved(&p.ty.node)
                    };
                    (p.name.node.clone(), ty, p.is_mut)
                })
                .collect();

            let ret = m
                .ret_type
                .as_ref()
                .map(|t| self.ast_type_to_resolved(&t.node))
                .unwrap_or(ResolvedType::Unit);

            let meth_name = m.name.node.clone();
            methods.insert(
                meth_name.clone(),
                vais_types::TraitMethodSig {
                    name: meth_name,
                    generics: m.generics.iter().map(|g| g.name.node.clone()).collect(),
                    params,
                    ret,
                    has_default: m.default_body.is_some(),
                    is_async: m.is_async,
                    is_const: m.is_const,
                },
            );
        }

        // Register associated types from AST
        let mut associated_types = HashMap::new();
        for assoc in &t.associated_types {
            // Extract GAT generic parameters
            let generics = assoc.generics.iter().map(|g| g.name.node.clone()).collect();

            // Build generic bounds for GAT parameters
            let mut generic_bounds = HashMap::new();
            for gen_param in &assoc.generics {
                if !gen_param.bounds.is_empty() {
                    generic_bounds.insert(
                        gen_param.name.node.clone(),
                        gen_param.bounds.iter().map(|b| b.node.clone()).collect(),
                    );
                }
            }

            // Extract trait bounds on the associated type itself
            let bounds = assoc.bounds.iter().map(|b| b.node.clone()).collect();

            // Convert default type if provided
            let default = assoc
                .default
                .as_ref()
                .map(|d| self.ast_type_to_resolved(&d.node));

            let assoc_name = assoc.name.node.clone();
            associated_types.insert(
                assoc_name.clone(),
                vais_types::AssociatedTypeDef {
                    name: assoc_name,
                    generics,
                    generic_bounds,
                    bounds,
                    default,
                },
            );
        }

        let trait_def = vais_types::TraitDef {
            name: t.name.node.clone(),
            generics: t.generics.iter().map(|g| g.name.node.clone()).collect(),
            super_traits: t.super_traits.iter().map(|s| s.node.clone()).collect(),
            associated_types,
            methods,
        };
        self.register_trait(trait_def);
    }

    /// Register a trait implementation for vtable generation.
    /// If the trait is "Drop", also registers the drop function in drop_registry
    /// so that scope-exit codegen can automatically call it.
    pub fn register_trait_impl(
        &mut self,
        impl_type: &str,
        trait_name: &str,
        method_impls: HashMap<String, String>,
    ) {
        // If this is a Drop trait impl, register the drop function for auto-call at scope exit
        if trait_name == "Drop" {
            if let Some(drop_fn_name) = method_impls.get("drop") {
                self.types
                    .drop_registry
                    .insert(impl_type.to_string(), drop_fn_name.clone());
            } else {
                // Convention: Type_drop is the mangled name for Drop.drop()
                let drop_fn_name = format!("{}_drop", impl_type);
                self.types
                    .drop_registry
                    .insert(impl_type.to_string(), drop_fn_name);
            }
        }
        self.types.trait_impl_methods.insert(
            (impl_type.to_string(), trait_name.to_string()),
            method_impls,
        );
    }

    /// Get or generate a vtable for a specific type implementing a trait
    pub fn get_or_generate_vtable(
        &mut self,
        impl_type: &str,
        trait_name: &str,
    ) -> CodegenResult<vtable::VtableInfo> {
        let trait_def = self
            .types
            .trait_defs
            .get(trait_name)
            .ok_or_else(|| CodegenError::TypeError(format!("Unknown trait: {}", trait_name)))?;
        let trait_def = trait_def.clone();
        let method_impls = self
            .types
            .trait_impl_methods
            .get(&(impl_type.to_string(), trait_name.to_string()))
            .cloned()
            .unwrap_or_default();

        self.vtable_generator
            .generate_vtable(impl_type, &trait_def, &method_impls)
            .map_err(CodegenError::TypeError)
    }

    /// Generate all vtable globals for the module.
    ///
    /// 2a-A (DEFERRED #17): use `generate_vtable_global_typed` so each
    /// vtable's slot fn-ptr type strings precisely match the impl
    /// methods' return / arg types. Pre-2a-A this used the simplified
    /// `generate_vtable_global` (everything was `i64`), which silently
    /// produced IR-shape mismatches at the indirect-call dispatch site
    /// for impls with `Result<T,E>` / struct returns.
    pub fn generate_vtable_globals(&self) -> String {
        let mut ir = String::new();

        for vtable_info in self.vtable_generator.get_vtables() {
            if let Some(trait_def) = self.types.trait_defs.get(&vtable_info.trait_name) {
                let type_size = 8;
                let type_align = 8;

                // Build per-method (arg_tys, ret_ty) in deterministic
                // alphabetical order — must match info.methods order
                // (now also sorted; DEFERRED #19 2a-C-1 audit fix).
                let methods_typed: Vec<(Vec<String>, String)> = super::vtable::sorted_method_names(trait_def)
                    .into_iter()
                    .map(|name| {
                        let sig = &trait_def.methods[&name];
                        let arg_tys: Vec<String> = sig
                            .params
                            .iter()
                            .skip(1) // self
                            .map(|(_n, ty, _mut)| self.type_to_llvm(ty))
                            .collect();
                        let ret_ty = if sig.is_async {
                            String::from("i64")
                        } else if matches!(sig.ret, ResolvedType::Unit) {
                            String::from("void")
                        } else {
                            self.type_to_llvm(&sig.ret)
                        };
                        (arg_tys, ret_ty)
                    })
                    .collect();

                ir.push_str(&self.vtable_generator.generate_vtable_global_typed(
                    vtable_info,
                    trait_def,
                    type_size,
                    type_align,
                    &methods_typed,
                ));
                ir.push('\n');
            }
        }

        ir
    }

    /// Generate code to create a trait object from a concrete value
    pub fn generate_trait_object_creation(
        &mut self,
        concrete_value: &str,
        concrete_type: &str,
        impl_type: &str,
        trait_name: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let vtable_info = self.get_or_generate_vtable(impl_type, trait_name)?;

        // Remember the counter value — create_trait_object will use it for the malloc register
        let alloc_counter = *counter;
        let result = self.vtable_generator.create_trait_object(
            concrete_value,
            concrete_type,
            &vtable_info,
            counter,
        );
        // Track the trait data allocation for automatic cleanup at scope exit
        let (val, mut ir) = result;
        ir.push_str(&self.track_alloc(format!("%trait_data_{}", alloc_counter)));
        Ok((val, ir))
    }

    /// Generate code for a dynamic method call on a trait object.
    ///
    /// 2a-A (DEFERRED #17): use `type_to_llvm(&method_sig.ret)` to lower
    /// the trait method's declared return type to a precise LLVM IR
    /// string instead of the legacy `vtable_ret_type` 4-bucket
    /// approximation (Unit→void, Str→fat, async→i64, all-else→i64).
    /// The legacy bucket misclassified `Result<T,E>` / `Option<T>` /
    /// struct returns as `i64`, producing IR-shape mismatches at the
    /// caller's `extractvalue` site (vaisdb baseline regression cause).
    pub fn generate_dyn_method_call(
        &self,
        trait_object: &str,
        trait_name: &str,
        method_name: &str,
        args: &[String],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let trait_def =
            self.types.trait_defs.get(trait_name).ok_or_else(|| {
                CodegenError::Unsupported(format!("Unknown trait: {}", trait_name))
            })?;

        // Method dispatch index — must match the slot order used by
        // generate_vtable / generate_vtable_global. All sites use
        // `sorted_method_names` (alphabetical) for determinism per
        // DEFERRED #19 step 2a-C-1 audit fix.
        let sorted = super::vtable::sorted_method_names(trait_def);
        let method_index = sorted
            .iter()
            .position(|n| n == method_name)
            .ok_or_else(|| {
                CodegenError::Unsupported(format!(
                    "Method {} not found in trait {}",
                    method_name, trait_name
                ))
            })?;

        // Get method signature
        let method_sig = trait_def.methods.get(method_name).ok_or_else(|| {
            CodegenError::Unsupported(format!(
                "Method {} not found in trait {}",
                method_name, trait_name
            ))
        })?;

        // 2a-A: precise ResolvedType-aware LLVM lowering for return.
        let ret_type_string: String = if method_sig.is_async {
            String::from("i64")
        } else if matches!(method_sig.ret, ResolvedType::Unit) {
            String::from("void")
        } else {
            self.type_to_llvm(&method_sig.ret)
        };

        // 2a-A: precise per-arg LLVM type list. method_sig.params[0] is
        // self, args[i] corresponds to method_sig.params[i+1].
        let arg_types_typed: Vec<(String, String)> = args
            .iter()
            .enumerate()
            .map(|(i, val)| {
                let pty = method_sig
                    .params
                    .get(i + 1)
                    .map(|(_n, ty, _mut)| ty.clone())
                    .unwrap_or(ResolvedType::I64);
                let llvm_ty = self.type_to_llvm(&pty);
                (llvm_ty, val.clone())
            })
            .collect();

        // 2a-A: per-trait-method (arg_tys, ret_ty) in deterministic
        // alphabetical order — must match the shape used by
        // `generate_vtable_globals` so emission/dispatch sides agree
        // (DEFERRED #19 2a-C-1 audit fix).
        let methods_typed: Vec<(Vec<String>, String)> = super::vtable::sorted_method_names(trait_def)
            .into_iter()
            .map(|name| {
                let sig = &trait_def.methods[&name];
                let arg_tys: Vec<String> = sig
                    .params
                    .iter()
                    .skip(1)
                    .map(|(_n, ty, _mut)| self.type_to_llvm(ty))
                    .collect();
                let ret_ty = if sig.is_async {
                    String::from("i64")
                } else if matches!(sig.ret, ResolvedType::Unit) {
                    String::from("void")
                } else {
                    self.type_to_llvm(&sig.ret)
                };
                (arg_tys, ret_ty)
            })
            .collect();

        Ok(self.vtable_generator.generate_dynamic_call_typed(
            trait_object,
            method_index,
            &arg_types_typed,
            &ret_type_string,
            trait_def,
            &methods_typed,
            counter,
        ))
    }
}
