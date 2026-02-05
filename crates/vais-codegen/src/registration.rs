//! Type and function registration for Vais code generator
//!
//! This module contains functions for registering functions, methods,
//! structs, enums, and unions during the first pass of code generation.

use crate::types::{
    EnumInfo, EnumVariantFields, EnumVariantInfo, FunctionInfo, StructInfo, UnionInfo,
};
use crate::{CodeGenerator, CodegenResult};
use vais_ast::{ExternFunction, Function, Struct, VariantFields};
use vais_types::{FunctionSig, ResolvedType};

impl CodeGenerator {
    pub(crate) fn register_function(&mut self, f: &Function) -> CodegenResult<()> {
        // Use resolved function signatures from type checker when available
        // (needed for functions with inferred parameter types - Type::Infer)
        let has_inferred = f
            .params
            .iter()
            .any(|p| matches!(p.ty.node, vais_ast::Type::Infer));
        let params: Vec<_> = if has_inferred {
            if let Some(resolved_sig) = self.resolved_function_sigs.get(&f.name.node) {
                resolved_sig.params.clone()
            } else {
                f.params
                    .iter()
                    .map(|p| {
                        let ty = self.ast_type_to_resolved(&p.ty.node);
                        let ty = if matches!(ty, ResolvedType::Unknown)
                            && matches!(p.ty.node, vais_ast::Type::Infer)
                        {
                            ResolvedType::I64 // default to i64 for unresolved inferred types
                        } else {
                            ty
                        };
                        (p.name.node.to_string(), ty, p.is_mut)
                    })
                    .collect()
            }
        } else {
            f.params
                .iter()
                .map(|p| {
                    let ty = self.ast_type_to_resolved(&p.ty.node);
                    (p.name.node.to_string(), ty, p.is_mut)
                })
                .collect()
        };

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        self.functions.insert(
            f.name.node.to_string(),
            FunctionInfo {
                signature: FunctionSig {
                    name: f.name.node.to_string(),
                    generics: f.generics.iter().map(|g| g.name.node.clone()).collect(),
                    generic_bounds: f
                        .generics
                        .iter()
                        .map(|g| {
                            (
                                g.name.node.clone(),
                                g.bounds.iter().map(|b| b.node.clone()).collect(),
                            )
                        })
                        .collect(),
                    params,
                    ret: ret_type,
                    is_async: f.is_async,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: vais_types::EffectAnnotation::Infer,
                    inferred_effects: None,
                },
                is_extern: false,
                extern_abi: None,
            },
        );

        Ok(())
    }

    /// Register a method as a function with Type_methodName naming convention
    pub(crate) fn register_method(&mut self, type_name: &str, f: &Function) -> CodegenResult<()> {
        let method_name = format!("{}_{}", type_name, f.name.node);

        // Check if this is a static method (no &self or self parameter)
        let has_self = f
            .params
            .first()
            .map(|p| p.name.node == "self")
            .unwrap_or(false);

        // Build parameter list
        let mut params = Vec::new();

        if has_self {
            // Instance method: add self parameter (pointer to struct type)
            params.push((
                "self".to_string(),
                ResolvedType::Named {
                    name: type_name.to_string(),
                    generics: vec![],
                },
                false,
            ));
        }

        // Add remaining parameters (skip self if it exists)
        for p in &f.params {
            if p.name.node != "self" {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                params.push((p.name.node.to_string(), ty, p.is_mut));
            }
        }

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        self.functions.insert(
            method_name.to_string(),
            FunctionInfo {
                signature: FunctionSig {
                    name: method_name.clone(),
                    generics: f.generics.iter().map(|g| g.name.node.clone()).collect(),
                    generic_bounds: f
                        .generics
                        .iter()
                        .map(|g| {
                            (
                                g.name.node.clone(),
                                g.bounds.iter().map(|b| b.node.clone()).collect(),
                            )
                        })
                        .collect(),
                    params,
                    ret: ret_type,
                    is_async: f.is_async,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: vais_types::EffectAnnotation::Infer,
                    inferred_effects: None,
                },
                is_extern: false,
                extern_abi: None,
            },
        );

        Ok(())
    }

    pub(crate) fn register_struct(&mut self, s: &Struct) -> CodegenResult<()> {
        let fields: Vec<_> = s
            .fields
            .iter()
            .map(|f| {
                let ty = self.ast_type_to_resolved(&f.ty.node);
                (f.name.node.to_string(), ty)
            })
            .collect();

        // Extract invariant expressions from attributes
        let invariants: Vec<_> = s
            .attributes
            .iter()
            .filter(|a| a.name == "invariant")
            .filter_map(|a| a.expr.as_ref().map(|e| (**e).clone()))
            .collect();

        self.structs.insert(
            s.name.node.to_string(),
            StructInfo {
                name: s.name.node.to_string(),
                fields,
                repr_c: s
                    .attributes
                    .iter()
                    .any(|a| a.name == "repr" && a.args.iter().any(|arg| arg == "C")),
                invariants,
            },
        );

        Ok(())
    }

    pub(crate) fn register_enum(&mut self, e: &vais_ast::Enum) -> CodegenResult<()> {
        let mut variants = Vec::new();

        for (tag, variant) in e.variants.iter().enumerate() {
            let fields = match &variant.fields {
                VariantFields::Unit => EnumVariantFields::Unit,
                VariantFields::Tuple(types) => {
                    let resolved: Vec<_> = types
                        .iter()
                        .map(|t| self.ast_type_to_resolved(&t.node))
                        .collect();
                    EnumVariantFields::Tuple(resolved)
                }
                VariantFields::Struct(named_fields) => {
                    let resolved: Vec<_> = named_fields
                        .iter()
                        .map(|f| {
                            let ty = self.ast_type_to_resolved(&f.ty.node);
                            (f.name.node.to_string(), ty)
                        })
                        .collect();
                    EnumVariantFields::Struct(resolved)
                }
            };

            variants.push(EnumVariantInfo {
                name: variant.name.node.to_string(),
                tag: tag as u32,
                fields,
            });
        }

        self.enums.insert(
            e.name.node.to_string(),
            EnumInfo {
                name: e.name.node.to_string(),
                variants,
            },
        );

        Ok(())
    }

    pub(crate) fn register_union(&mut self, u: &vais_ast::Union) -> CodegenResult<()> {
        let fields: Vec<_> = u
            .fields
            .iter()
            .map(|f| {
                let ty = self.ast_type_to_resolved(&f.ty.node);
                (f.name.node.to_string(), ty)
            })
            .collect();

        self.unions.insert(
            u.name.node.to_string(),
            UnionInfo {
                name: u.name.node.to_string(),
                fields,
            },
        );

        Ok(())
    }

    pub(crate) fn register_extern_function(
        &mut self,
        func: &ExternFunction,
        abi: &str,
    ) -> CodegenResult<()> {
        let func_name = func.name.node.to_string();

        // Check if this is already registered as a builtin helper function
        // Builtin helpers have is_extern=false and should not be overridden
        if let Some(existing) = self.functions.get(&func_name) {
            if !existing.is_extern {
                // This is a builtin helper function - don't override it
                // Just skip the registration silently
                return Ok(());
            }
        }

        let params: Vec<_> = func
            .params
            .iter()
            .map(|p| {
                let resolved = self.ast_type_to_resolved(&p.ty.node);
                (p.name.node.to_string(), resolved, p.is_mut)
            })
            .collect();

        let ret_type = func
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        self.functions.insert(
            func_name,
            FunctionInfo {
                signature: FunctionSig {
                    name: func.name.node.to_string(),
                    generics: vec![],
                    generic_bounds: std::collections::HashMap::new(),
                    params,
                    ret: ret_type,
                    is_async: false,
                    is_vararg: func.is_vararg,
                    required_params: None,
                    contracts: None,
                    effect_annotation: vais_types::EffectAnnotation::Infer,
                    inferred_effects: None,
                },
                is_extern: true,
                extern_abi: Some(abi.to_string()),
            },
        );

        Ok(())
    }

    /// Register a constant definition
    pub(crate) fn register_const(&mut self, const_def: &vais_ast::ConstDef) -> CodegenResult<()> {
        // Store constant in the constants map for later lookup
        self.constants.insert(
            const_def.name.node.clone(),
            crate::types::ConstInfo {
                name: const_def.name.node.clone(),
                ty: self.ast_type_to_resolved(&const_def.ty.node),
                value: const_def.value.clone(),
            },
        );
        Ok(())
    }

    /// Register a global variable definition
    pub(crate) fn register_global(
        &mut self,
        global_def: &vais_ast::GlobalDef,
    ) -> CodegenResult<()> {
        // Store global in the globals map for later code generation
        self.globals.insert(
            global_def.name.node.clone(),
            crate::types::GlobalInfo {
                name: global_def.name.node.clone(),
                ty: self.ast_type_to_resolved(&global_def.ty.node),
                value: global_def.value.clone(),
                is_mutable: global_def.is_mutable,
            },
        );
        Ok(())
    }
}
