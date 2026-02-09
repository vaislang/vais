//! Module-level type checking: check_module, registration, and generics.

use std::collections::HashMap;

use vais_ast::*;

use super::TypeChecker;
use crate::object_safety;
use crate::ownership;
use crate::traits::TraitImpl;
use crate::traits::{AssociatedTypeDef, TraitDef, TraitMethodSig};
use crate::types::{
    self, EffectAnnotation, EnumDef, FunctionSig, ResolvedType, StructDef, TypeError, TypeResult,
    UnionDef, VariantFieldTypes,
};

impl TypeChecker {
    /// Type checks a complete module.
    ///
    /// Performs two-pass type checking:
    /// 1. First pass: Collect all type definitions (functions, structs, enums, traits)
    /// 2. Second pass: Type check all function bodies and implementations
    ///
    /// # Arguments
    ///
    /// * `module` - The parsed AST module to type check
    ///
    /// # Returns
    ///
    /// Ok(()) if type checking succeeds, or a TypeError on failure.
    pub fn check_module(&mut self, module: &Module) -> TypeResult<()> {
        // First pass: collect all type definitions
        for item in &module.items {
            match &item.node {
                Item::Function(f) => self.register_function(f)?,
                Item::Struct(s) => self.register_struct(s)?,
                Item::Enum(e) => self.register_enum(e)?,
                Item::Union(u) => self.register_union(u)?,
                Item::TypeAlias(t) => self.register_type_alias(t)?,
                Item::Use(_use_stmt) => {
                    // Use statements are handled at the compiler level (AST merging)
                    // by the time we reach type checking, all imports are already resolved
                }
                Item::Trait(t) => self.register_trait(t)?,
                Item::Impl(impl_block) => {
                    // Register impl methods to the target type
                    self.register_impl(impl_block)?;
                }
                Item::Macro(_) => {
                    // Macro definitions are handled at the expansion phase
                    // before type checking
                }
                Item::Error { .. } => {
                    // Error nodes from recovery mode are skipped during type checking.
                    // They represent parsing failures that have already been reported.
                }
                Item::ExternBlock(ext) => {
                    // Register extern functions
                    for func in &ext.functions {
                        self.register_extern_function(func)?;
                    }
                }
                Item::Const(const_def) => {
                    // Register constant with its type
                    let const_type = self.resolve_type(&const_def.ty.node);
                    self.constants
                        .insert(const_def.name.node.clone(), const_type);
                }
                Item::Global(_global_def) => {
                    // Global variable definitions
                    // Type checking happens during code generation
                }
            }
        }

        // Second pass: check function bodies
        for item in &module.items {
            match &item.node {
                Item::Function(f) => self.check_function(f)?,
                Item::Impl(impl_block) => {
                    // Check impl method bodies
                    // Get struct generics if the target is a struct
                    let struct_generics = match &impl_block.target_type.node {
                        Type::Named { name, .. } => {
                            // Look up the struct definition to get its generics
                            self.structs
                                .get(name)
                                .map(|s| {
                                    s.generics
                                        .iter()
                                        .map(|g| {
                                            GenericParam::new_type(
                                                Spanned::new(g.clone(), Span::default()),
                                                vec![],
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default()
                        }
                        _ => vec![],
                    };
                    // Also include impl-level generics
                    let mut all_generics = struct_generics;
                    all_generics.extend_from_slice(&impl_block.generics);

                    for method in &impl_block.methods {
                        self.check_impl_method(
                            &impl_block.target_type.node,
                            &method.node,
                            &all_generics,
                        )?;
                    }
                }
                _ => {}
            }
        }

        // Third pass: ownership and borrow checking (skip imported items)
        if let Some(strict) = self.ownership_check_mode {
            let mut ownership_checker = ownership::OwnershipChecker::new_collecting();
            // Only check ownership for items from the current file, not imported modules
            let local_module =
                if self.imported_item_count > 0 && self.imported_item_count < module.items.len() {
                    Module {
                        items: module.items[self.imported_item_count..].to_vec(),
                        modules_map: None,
                    }
                } else {
                    module.clone()
                };
            // Run ownership check in collecting mode (never fails, collects all errors)
            let _ = ownership_checker.check_module(&local_module);
            let ownership_errors = ownership_checker.take_errors();

            if !ownership_errors.is_empty() {
                if strict {
                    // Strict mode: return first error
                    return Err(ownership_errors.into_iter().next().unwrap());
                } else {
                    // Warn mode: add to warnings
                    for err in &ownership_errors {
                        self.warnings.push(format!("[ownership] {}", err));
                    }
                }
            }
        }

        Ok(())
    }

    /// Set current generics with their bounds for type resolution
    #[allow(clippy::type_complexity)]
    pub(crate) fn set_generics(
        &mut self,
        generics: &[GenericParam],
    ) -> (
        Vec<String>,
        HashMap<String, Vec<String>>,
        HashMap<String, ResolvedType>,
    ) {
        let prev_generics = std::mem::replace(
            &mut self.current_generics,
            generics.iter().map(|g| &g.name.node).cloned().collect(),
        );
        let prev_bounds = std::mem::replace(
            &mut self.current_generic_bounds,
            generics
                .iter()
                .map(|g| {
                    (
                        g.name.node.clone(),
                        g.bounds.iter().map(|b| &b.node).cloned().collect(),
                    )
                })
                .collect(),
        );
        // Track const generic parameters with their types
        // Collect first to avoid borrow conflict with self.resolve_type
        let new_const_generics: HashMap<String, ResolvedType> = generics
            .iter()
            .filter_map(|g| {
                if let GenericParamKind::Const { ty } = &g.kind {
                    Some((g.name.node.clone(), self.resolve_type(&ty.node)))
                } else {
                    None
                }
            })
            .collect();
        let prev_const_generics =
            std::mem::replace(&mut self.current_const_generics, new_const_generics);
        (prev_generics, prev_bounds, prev_const_generics)
    }

    /// Restore previous generics
    pub(crate) fn restore_generics(
        &mut self,
        prev_generics: Vec<String>,
        prev_bounds: HashMap<String, Vec<String>>,
        prev_const_generics: HashMap<String, ResolvedType>,
    ) {
        self.current_generics = prev_generics;
        self.current_generic_bounds = prev_bounds;
        self.current_const_generics = prev_const_generics;
    }

    /// Extract contract specification from function attributes
    ///
    /// Parses requires/ensures/invariant attributes and builds a ContractSpec.
    /// Contract expressions must evaluate to bool.
    pub(crate) fn extract_contracts(
        &mut self,
        f: &Function,
    ) -> TypeResult<Option<types::ContractSpec>> {
        use types::{ContractClause, ContractSpec};

        let mut spec = ContractSpec::default();

        for attr in &f.attributes {
            match attr.name.as_str() {
                "requires" | "ensures" => {
                    if let Some(expr) = &attr.expr {
                        // Type check the contract expression - it must be bool
                        let expr_type = self.check_expr(expr)?;
                        if expr_type != ResolvedType::Bool {
                            return Err(TypeError::Mismatch {
                                expected: "bool".to_string(),
                                found: expr_type.to_string(),
                                span: Some(expr.span),
                            });
                        }

                        let clause = ContractClause {
                            expr_str: attr.args.first().cloned().unwrap_or_default(),
                            span: expr.span,
                        };

                        if attr.name == "requires" {
                            spec.requires.push(clause);
                        } else {
                            spec.ensures.push(clause);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(if spec.is_empty() { None } else { Some(spec) })
    }

    /// Register a function signature
    pub(crate) fn register_function(&mut self, f: &Function) -> TypeResult<()> {
        let name = f.name.node.clone();

        // Set current generics for type resolution
        let (prev_generics, prev_bounds, prev_const_generics) = self.set_generics(&f.generics);

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
            .unwrap_or_else(|| self.fresh_type_var());

        // Restore previous generics
        self.restore_generics(prev_generics, prev_bounds, prev_const_generics);

        let generic_bounds: HashMap<String, Vec<String>> = f
            .generics
            .iter()
            .map(|g| {
                (
                    g.name.node.clone(),
                    g.bounds.iter().map(|b| &b.node).cloned().collect(),
                )
            })
            .collect();

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
                is_vararg: false,
                required_params,
                contracts: None, // Contracts will be extracted in check_function
                effect_annotation: EffectAnnotation::Infer,
                inferred_effects: None,
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
        let (prev_generics, prev_bounds, prev_const_generics) = self.set_generics(&s.generics);

        let mut fields = HashMap::new();
        for field in &s.fields {
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

            let method_bounds: HashMap<String, Vec<String>> = method
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
                },
            );
        }

        // Restore previous generics
        self.restore_generics(prev_generics, prev_bounds, prev_const_generics);

        self.structs.insert(
            name.clone(),
            StructDef {
                name,
                generics: s.generics.iter().map(|g| &g.name.node).cloned().collect(),
                fields,
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
        let (prev_generics, prev_bounds, prev_const_generics) = self.set_generics(&e.generics);

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
        self.restore_generics(prev_generics, prev_bounds, prev_const_generics);

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
        let (prev_generics, prev_bounds, prev_const_generics) = self.set_generics(&u.generics);

        let mut fields = HashMap::new();
        for field in &u.fields {
            fields.insert(field.name.node.clone(), self.resolve_type(&field.ty.node));
        }

        // Restore previous generics
        self.restore_generics(prev_generics, prev_bounds, prev_const_generics);

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
        let (prev_generics, prev_bounds, prev_const_generics) = self.set_generics(&all_generics);

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
                    span: None,
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
                            span: None,
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
                            span: None,
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
        self.restore_generics(prev_generics, prev_bounds, prev_const_generics);

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
        let (prev_generics, prev_bounds, prev_const_generics) = self.set_generics(&t.generics);

        // Add "Self" as an implicit generic parameter for trait methods
        self.current_generics.push("Self".to_string());

        // Parse associated types (with GAT support)
        let mut associated_types = HashMap::new();
        for assoc in &t.associated_types {
            let bounds: Vec<String> = assoc.bounds.iter().map(|b| &b.node).cloned().collect();
            let default = assoc.default.as_ref().map(|ty| self.resolve_type(&ty.node));

            // Extract GAT generic parameters and their bounds
            let gat_generics: Vec<String> =
                assoc.generics.iter().map(|g| &g.name.node).cloned().collect();
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

        self.restore_generics(prev_generics, prev_bounds, prev_const_generics);

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

    /// Validate object safety for dyn Trait types
    pub(crate) fn validate_dyn_trait_object_safety(&mut self, ty: &ResolvedType) {
        match ty {
            ResolvedType::DynTrait {
                trait_name,
                generics,
            } => {
                if let Some(trait_def) = self.traits.get(trait_name) {
                    if let Err(violations) = object_safety::check_object_safety(trait_def) {
                        let mut error_msg = format!(
                            "trait `{}` cannot be used as a trait object (not object-safe)",
                            trait_name
                        );
                        for violation in &violations {
                            error_msg.push_str(&format!("\n  - {}", violation.description()));
                        }
                        self.warnings.push(error_msg);
                    }
                }
                // Check generics recursively
                for gen in generics {
                    self.validate_dyn_trait_object_safety(gen);
                }
            }
            // Recursively check compound types
            ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Pointer(inner)
            | ResolvedType::Array(inner)
            | ResolvedType::Optional(inner)
            | ResolvedType::Future(inner)
            | ResolvedType::Range(inner) => {
                self.validate_dyn_trait_object_safety(inner);
            }
            ResolvedType::Result(ok, err) => {
                self.validate_dyn_trait_object_safety(ok);
                self.validate_dyn_trait_object_safety(err);
            }
            ResolvedType::ConstArray { element, .. } => {
                self.validate_dyn_trait_object_safety(element);
            }
            ResolvedType::Map(k, v) => {
                self.validate_dyn_trait_object_safety(k);
                self.validate_dyn_trait_object_safety(v);
            }
            ResolvedType::Tuple(types) => {
                for t in types {
                    self.validate_dyn_trait_object_safety(t);
                }
            }
            ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
                for p in params {
                    self.validate_dyn_trait_object_safety(p);
                }
                self.validate_dyn_trait_object_safety(ret);
            }
            ResolvedType::Named { generics, .. } => {
                for g in generics {
                    self.validate_dyn_trait_object_safety(g);
                }
            }
            _ => {}
        }
    }
}
