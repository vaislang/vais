//! Type and function registration for Vais code generator
//!
//! This module contains functions for registering functions, methods,
//! structs, enums, and unions during the first pass of code generation.

use crate::types::{EnumInfo, EnumVariantFields, EnumVariantInfo, FunctionInfo, StructInfo, UnionInfo};
use crate::{CodeGenerator, CodegenResult};
use vais_ast::{Function, Struct, VariantFields, ExternFunction};
use vais_types::{ResolvedType, FunctionSig};

impl CodeGenerator {
    pub(crate) fn register_function(&mut self, f: &Function) -> CodegenResult<()> {
        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                (p.name.node.to_string(), ty, p.is_mut)
            })
            .collect();

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
                    generic_bounds: f.generics.iter()
                        .map(|g| (g.name.node.clone(), g.bounds.iter().map(|b| b.node.clone()).collect()))
                        .collect(),
                    params,
                    ret: ret_type,
                    is_async: f.is_async,
                    is_vararg: false,
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
                    generic_bounds: f.generics.iter()
                        .map(|g| (g.name.node.clone(), g.bounds.iter().map(|b| b.node.clone()).collect()))
                        .collect(),
                    params,
                    ret: ret_type,
                    is_async: f.is_async,
                    is_vararg: false,
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

        self.structs.insert(
            s.name.node.to_string(),
            StructInfo {
                name: s.name.node.to_string(),
                fields,
                repr_c: s.attributes.iter().any(|a| a.name == "repr" && a.args.iter().any(|arg| arg == "C")),
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

    pub(crate) fn register_extern_function(&mut self, func: &ExternFunction, abi: &str) -> CodegenResult<()> {
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
            func.name.node.to_string(),
            FunctionInfo {
                signature: FunctionSig {
                    name: func.name.node.to_string(),
                    generics: vec![],
                    generic_bounds: std::collections::HashMap::new(),
                    params,
                    ret: ret_type,
                    is_async: false,
                    is_vararg: func.is_vararg,
                },
                is_extern: true,
                extern_abi: Some(abi.to_string()),
            },
        );

        Ok(())
    }
}
