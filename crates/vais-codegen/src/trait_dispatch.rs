//! Trait dispatch, vtable generation, and dynamic method calls

use super::*;

impl CodeGenerator {
    /// Register a trait definition for vtable generation
    pub fn register_trait(&mut self, trait_def: vais_types::TraitDef) {
        self.types.trait_defs.insert(trait_def.name.clone(), trait_def);
    }

    /// Register a trait from AST definition (converts AST Trait to TraitDef)
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

            methods.insert(
                m.name.node.clone(),
                vais_types::TraitMethodSig {
                    name: m.name.node.clone(),
                    params,
                    ret,
                    has_default: m.default_body.is_some(),
                    is_async: m.is_async,
                    is_const: m.is_const,
                },
            );
        }

        let trait_def = vais_types::TraitDef {
            name: t.name.node.clone(),
            generics: t.generics.iter().map(|g| g.name.node.clone()).collect(),
            super_traits: t.super_traits.iter().map(|s| s.node.clone()).collect(),
            associated_types: HashMap::new(), // Simplified for now
            methods,
        };
        self.register_trait(trait_def);
    }

    /// Register a trait implementation for vtable generation
    pub fn register_trait_impl(
        &mut self,
        impl_type: &str,
        trait_name: &str,
        method_impls: HashMap<String, String>,
    ) {
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
    ) -> Option<vtable::VtableInfo> {
        let trait_def = self.types.trait_defs.get(trait_name)?.clone();
        let method_impls = self
            .types.trait_impl_methods
            .get(&(impl_type.to_string(), trait_name.to_string()))
            .cloned()
            .unwrap_or_default();

        Some(
            self.vtable_generator
                .generate_vtable(impl_type, &trait_def, &method_impls),
        )
    }

    /// Generate all vtable globals for the module
    pub fn generate_vtable_globals(&self) -> String {
        let mut ir = String::new();

        for vtable_info in self.vtable_generator.get_vtables() {
            if let Some(trait_def) = self.types.trait_defs.get(&vtable_info.trait_name) {
                let type_size = 8; // Default size, could be refined
                let type_align = 8; // Default alignment

                ir.push_str(&self.vtable_generator.generate_vtable_global(
                    vtable_info,
                    trait_def,
                    type_size,
                    type_align,
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
        let vtable_info = self
            .get_or_generate_vtable(impl_type, trait_name)
            .ok_or_else(|| {
                CodegenError::Unsupported(format!(
                    "No vtable for {} implementing {}",
                    impl_type, trait_name
                ))
            })?;

        Ok(self.vtable_generator.create_trait_object(
            concrete_value,
            concrete_type,
            &vtable_info,
            counter,
        ))
    }

    /// Generate code for a dynamic method call on a trait object
    pub fn generate_dyn_method_call(
        &self,
        trait_object: &str,
        trait_name: &str,
        method_name: &str,
        args: &[String],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let trait_def = self
            .types.trait_defs
            .get(trait_name)
            .ok_or_else(|| CodegenError::Unsupported(format!("Unknown trait: {}", trait_name)))?;

        // Find method index in trait
        let method_names: Vec<&String> = trait_def.methods.keys().collect();
        let method_index = method_names
            .iter()
            .position(|&n| n == method_name)
            .ok_or_else(|| {
                CodegenError::Unsupported(format!(
                    "Method {} not found in trait {}",
                    method_name, trait_name
                ))
            })?;

        // Get return type
        let method_sig = trait_def.methods.get(method_name).ok_or_else(|| {
            CodegenError::Unsupported(format!(
                "Method {} not found in trait {}",
                method_name, trait_name
            ))
        })?;

        let ret_type = if matches!(method_sig.ret, ResolvedType::Unit) {
            "void"
        } else {
            "i64" // Simplified
        };

        Ok(self.vtable_generator.generate_dynamic_call(
            trait_object,
            method_index,
            args,
            ret_type,
            trait_def,
            counter,
        ))
    }
}
