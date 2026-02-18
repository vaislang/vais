//! Registration of impl blocks and trait definitions.

use super::*;

impl TypeChecker {
    /// Register impl block methods to the target type
    pub(crate) fn register_impl(&mut self, impl_block: &Impl) -> TypeResult<()> {
        // Get the type name
        let type_name = match &impl_block.target_type.node {
            Type::Named { name, .. } => name.clone(),
            _ => return Ok(()), // Skip non-named types for now
        };

        // Check if struct or enum exists
        let is_enum = self.enums.contains_key(&type_name);
        if !self.structs.contains_key(&type_name) && !is_enum {
            return Ok(()); // Type not registered yet, skip
        }

        // Get type generics and set them as current for type resolution
        let struct_generics: Vec<GenericParam> = if is_enum {
            self.enums
                .get(&type_name)
                .map(|e| {
                    e.generics
                        .iter()
                        .map(|g| {
                            GenericParam::new_type(Spanned::new(g.clone(), Span::default()), vec![])
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            self.structs
                .get(&type_name)
                .map(|s| {
                    s.generics
                        .iter()
                        .map(|g| {
                            GenericParam::new_type(Spanned::new(g.clone(), Span::default()), vec![])
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        // Combine struct generics with impl-level generics
        let mut all_generics = struct_generics;
        all_generics.extend_from_slice(&impl_block.generics);

        // Set current generics for proper type resolution
        let saved = self.set_generics(&all_generics);

        // If implementing a trait, validate the impl
        if let Some(trait_name) = &impl_block.trait_name {
            let trait_name_str = trait_name.node.clone();

            // Check trait exists
            if !self.traits.contains_key(&trait_name_str) {
                let suggestion = types::find_similar_name(
                    &trait_name_str,
                    self.traits.keys().map(|s| s.as_str()),
                );
                return Err(TypeError::UndefinedType {
                    name: format!("trait {}", trait_name_str),
                    span: Some(trait_name.span),
                    suggestion,
                });
            }

            // Parse associated type implementations
            let mut assoc_type_impls = std::collections::HashMap::new();
            for assoc in &impl_block.associated_types {
                let resolved_ty = self.resolve_type(&assoc.ty.node);
                assoc_type_impls.insert(assoc.name.node.clone(), resolved_ty);
            }

            // Record that this type implements this trait
            self.trait_impls.push(TraitImpl {
                trait_name: trait_name_str.clone(),
                type_name: type_name.clone(),
                associated_types: assoc_type_impls.clone(),
            });

            // Validate that all required associated types are implemented
            if let Some(trait_def) = self.traits.get(&trait_name_str).cloned() {
                for (assoc_name, assoc_def) in &trait_def.associated_types {
                    if assoc_def.default.is_none() && !assoc_type_impls.contains_key(assoc_name) {
                        return Err(TypeError::Mismatch {
                            expected: format!(
                                "associated type '{}' from trait '{}'",
                                assoc_name, trait_name_str
                            ),
                            found: "missing".to_string(),
                            span: None, // No span available for associated type validation
                        });
                    }
                }
            }

            // Validate that all required trait methods are implemented
            if let Some(trait_def) = self.traits.get(&trait_name_str).cloned() {
                let impl_method_names: std::collections::HashSet<_> = impl_block
                    .methods
                    .iter()
                    .map(|m| m.node.name.node.clone())
                    .collect();

                for (method_name, trait_method) in &trait_def.methods {
                    if !trait_method.has_default && !impl_method_names.contains(method_name) {
                        return Err(TypeError::Mismatch {
                            expected: format!(
                                "implementation of method '{}' from trait '{}'",
                                method_name, trait_name_str
                            ),
                            found: "missing".to_string(),
                            span: None, // No span available for trait method validation
                        });
                    }
                }
            }
        }

        // Collect method signatures first (to avoid borrow issues)
        let mut method_sigs = Vec::new();
        for method in &impl_block.methods {
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

            let impl_method_bounds: HashMap<String, Vec<String>> = method
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

            method_sigs.push((
                method.node.name.node.clone(),
                FunctionSig {
                    name: method.node.name.node.clone(),
                    generics: method
                        .node
                        .generics
                        .iter()
                        .map(|g| g.name.node.clone())
                        .collect(),
                    generic_bounds: impl_method_bounds,
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
            ));
        }

        // Now insert methods into the struct or enum
        if let Some(struct_def) = self.structs.get_mut(&type_name) {
            for (name, sig) in method_sigs {
                struct_def.methods.insert(name, sig);
            }
        } else if let Some(enum_def) = self.enums.get_mut(&type_name) {
            for (name, sig) in method_sigs {
                enum_def.methods.insert(name, sig);
            }
        }

        // Restore previous generics
        self.restore_generics(saved);

        Ok(())
    }

    /// Register a trait definition
    pub(crate) fn register_trait(&mut self, t: &vais_ast::Trait) -> TypeResult<()> {
        let name = t.name.node.clone();
        if self.traits.contains_key(&name) {
            return Err(TypeError::Duplicate(name, None));
        }

        // Validate super traits exist
        for super_trait in &t.super_traits {
            if !self.traits.contains_key(&super_trait.node) {
                // Allow forward references - will be validated later
                self.warnings.push(format!(
                    "Super trait '{}' referenced before definition",
                    super_trait.node
                ));
            }
        }

        // Set current generics for type resolution
        let saved = self.set_generics(&t.generics);

        // Merge where clause bounds into current generic bounds
        self.merge_where_clause(&t.where_clause);

        // Add "Self" as an implicit generic parameter for trait methods
        self.current_generics.push("Self".to_string());

        // Parse associated types (with GAT support)
        let mut associated_types = HashMap::new();
        for assoc in &t.associated_types {
            let bounds: Vec<String> = assoc.bounds.iter().map(|b| &b.node).cloned().collect();
            let default = assoc.default.as_ref().map(|ty| self.resolve_type(&ty.node));

            // Extract GAT generic parameters and their bounds
            let gat_generics: Vec<String> = assoc
                .generics
                .iter()
                .map(|g| &g.name.node)
                .cloned()
                .collect();
            let gat_bounds: HashMap<String, Vec<String>> = assoc
                .generics
                .iter()
                .map(|g| {
                    (
                        g.name.node.clone(),
                        g.bounds.iter().map(|b| &b.node).cloned().collect(),
                    )
                })
                .collect();

            associated_types.insert(
                assoc.name.node.clone(),
                AssociatedTypeDef {
                    name: assoc.name.node.clone(),
                    generics: gat_generics,
                    generic_bounds: gat_bounds,
                    bounds,
                    default,
                },
            );
        }

        let mut methods = HashMap::new();
        for method in &t.methods {
            let params: Vec<_> = method
                .params
                .iter()
                .map(|p| {
                    let ty = self.resolve_type(&p.ty.node);
                    (p.name.node.clone(), ty, p.is_mut)
                })
                .collect();

            let ret = method
                .ret_type
                .as_ref()
                .map(|rt| self.resolve_type(&rt.node))
                .unwrap_or(ResolvedType::Unit);

            methods.insert(
                method.name.node.clone(),
                TraitMethodSig {
                    name: method.name.node.clone(),
                    params,
                    ret,
                    has_default: method.default_body.is_some(),
                    is_async: method.is_async,
                    is_const: method.is_const,
                },
            );
        }

        // Remove "Self" from generics before restoring
        self.current_generics.pop();

        self.restore_generics(saved);

        self.traits.insert(
            name.clone(),
            TraitDef {
                name,
                generics: t.generics.iter().map(|g| &g.name.node).cloned().collect(),
                super_traits: t.super_traits.iter().map(|s| &s.node).cloned().collect(),
                associated_types,
                methods,
            },
        );

        Ok(())
    }
}
