//! Type system and generics support.
//!
//! Handles type conversion, generic substitution, specialization,
//! and try/unwrap operators.

use std::collections::HashMap;

use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, StructType};
use inkwell::values::{BasicValueEnum, FunctionValue};

use vais_ast::{Expr, Type};
use vais_types::ResolvedType;

use super::generator::InkwellCodeGenerator;
use crate::CodegenResult;

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn ast_type_to_resolved(&self, ty: &Type) -> ResolvedType {
        match ty {
            Type::Named { name, generics } => match name.as_str() {
                "i8" => ResolvedType::I8,
                "i16" => ResolvedType::I16,
                "i32" => ResolvedType::I32,
                "i64" => ResolvedType::I64,
                "i128" => ResolvedType::I128,
                "u8" => ResolvedType::U8,
                "u16" => ResolvedType::U16,
                "u32" => ResolvedType::U32,
                "u64" => ResolvedType::U64,
                "u128" => ResolvedType::U128,
                "f32" => ResolvedType::F32,
                "f64" => ResolvedType::F64,
                "bool" => ResolvedType::Bool,
                "str" => ResolvedType::Str,
                _ => {
                    // Single uppercase letter is likely a generic type parameter
                    if name.len() == 1 && name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        if generics.is_empty() {
                            ResolvedType::Generic(name.clone())
                        } else {
                            // HKT application: F<A> — keep as Named so substitute_type
                            // can replace the constructor name
                            let generic_types: Vec<ResolvedType> = generics
                                .iter()
                                .map(|g| self.ast_type_to_resolved(&g.node))
                                .collect();
                            ResolvedType::Named {
                                name: name.clone(),
                                generics: generic_types,
                            }
                        }
                    } else {
                        let generic_types: Vec<ResolvedType> = generics
                            .iter()
                            .map(|g| self.ast_type_to_resolved(&g.node))
                            .collect();
                        ResolvedType::Named {
                            name: name.clone(),
                            generics: generic_types,
                        }
                    }
                }
            },
            Type::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Array(inner) => {
                ResolvedType::Array(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Tuple(elems) => {
                let elem_types: Vec<ResolvedType> = elems
                    .iter()
                    .map(|e| self.ast_type_to_resolved(&e.node))
                    .collect();
                ResolvedType::Tuple(elem_types)
            }
            Type::FnPtr {
                params,
                ret,
                is_vararg,
            } => {
                let param_types: Vec<ResolvedType> = params
                    .iter()
                    .map(|p| self.ast_type_to_resolved(&p.node))
                    .collect();
                let ret_type = self.ast_type_to_resolved(&ret.node);
                ResolvedType::FnPtr {
                    params: param_types,
                    ret: Box::new(ret_type),
                    is_vararg: *is_vararg,
                    effects: None,
                }
            }
            Type::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Result(inner) => ResolvedType::Result(
                Box::new(self.ast_type_to_resolved(&inner.node)),
                Box::new(ResolvedType::I64),
            ),
            Type::Ref(inner) => ResolvedType::Ref(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Map(key, value) => ResolvedType::Map(
                Box::new(self.ast_type_to_resolved(&key.node)),
                Box::new(self.ast_type_to_resolved(&value.node)),
            ),
            Type::Unit => ResolvedType::Unit,
            _ => ResolvedType::I64, // Fallback for Infer, ConstArray, etc.
        }
    }

    // ========== Generic Type Handling ==========

    /// Get current generic substitution for a type parameter
    #[inline]
    pub fn get_generic_substitution(&self, param: &str) -> Option<ResolvedType> {
        self.generic_substitutions.get(param).cloned()
    }

    /// Set generic substitutions for the current context
    pub fn set_generic_substitutions(&mut self, subst: HashMap<String, ResolvedType>) {
        self.generic_substitutions = subst;
        self.type_mapper
            .set_generic_substitutions(&self.generic_substitutions);
    }

    /// Clear generic substitutions
    pub fn clear_generic_substitutions(&mut self) {
        self.generic_substitutions.clear();
        self.type_mapper.clear_generic_substitutions();
    }

    /// Substitute generic type parameters with concrete types
    pub fn substitute_type(&self, ty: &ResolvedType) -> ResolvedType {
        vais_types::substitute_type(ty, &self.generic_substitutions)
    }

    /// Generate mangled name for a generic struct
    #[inline]
    pub fn mangle_struct_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
    }

    /// Generate mangled name for a generic function
    #[inline]
    pub fn mangle_function_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
    }

    /// Define a specialized (monomorphized) struct type
    pub fn define_specialized_struct(
        &mut self,
        base_name: &str,
        type_args: &[ResolvedType],
        fields: &[(String, ResolvedType)],
        generic_param_names: &[String],
    ) -> CodegenResult<StructType<'ctx>> {
        let mangled_name = self.mangle_struct_name(base_name, type_args);

        // Check if already generated
        if let Some(st) = self.generated_structs.get(&mangled_name) {
            return Ok(*st);
        }

        // Build substitution map from actual generic param names to type args
        let mut substitutions = HashMap::new();
        for (name, type_arg) in generic_param_names.iter().zip(type_args.iter()) {
            substitutions.insert(name.clone(), type_arg.clone());
        }

        // Substitute types in fields
        let field_types: Vec<BasicTypeEnum> = fields
            .iter()
            .map(|(_, ty)| {
                let substituted = vais_types::substitute_type(ty, &substitutions);
                self.type_mapper.map_type(&substituted)
            })
            .collect();

        // Store field names
        let field_names: Vec<String> = fields.iter().map(|(name, _)| name.clone()).collect();
        self.struct_fields.insert(mangled_name.clone(), field_names);

        // Create struct type
        let struct_type = self.context.struct_type(&field_types, false);
        self.type_mapper.register_struct(&mangled_name, struct_type);
        self.generated_structs.insert(mangled_name, struct_type);

        Ok(struct_type)
    }

    /// Declare a specialized (monomorphized) function
    pub fn declare_specialized_function(
        &mut self,
        base_name: &str,
        type_args: &[ResolvedType],
        param_types: &[ResolvedType],
        return_type: &ResolvedType,
        generic_param_names: &[String],
    ) -> CodegenResult<FunctionValue<'ctx>> {
        let mangled_name = self.mangle_function_name(base_name, type_args);

        // Check if already declared
        if let Some(fn_val) = self.functions.get(&mangled_name) {
            return Ok(*fn_val);
        }

        // Build substitution map from actual generic param names to type args
        let mut substitutions = HashMap::new();
        for (name, type_arg) in generic_param_names.iter().zip(type_args.iter()) {
            substitutions.insert(name.clone(), type_arg.clone());
        }

        // Substitute types in parameters
        let llvm_param_types: Vec<BasicMetadataTypeEnum> = param_types
            .iter()
            .map(|ty| {
                let substituted = vais_types::substitute_type(ty, &substitutions);
                self.type_mapper.map_type(&substituted).into()
            })
            .collect();

        // Substitute return type
        let substituted_ret = vais_types::substitute_type(return_type, &substitutions);
        let fn_type = if matches!(substituted_ret, ResolvedType::Unit) {
            self.context.void_type().fn_type(&llvm_param_types, false)
        } else {
            let ret_type = self.type_mapper.map_type(&substituted_ret);
            ret_type.fn_type(&llvm_param_types, false)
        };

        let fn_value = self.module.add_function(&mangled_name, fn_type, None);
        self.functions.insert(mangled_name, fn_value);

        Ok(fn_value)
    }

    // ========== Try/Unwrap ==========

    pub(super) fn generate_try(&mut self, inner: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Try (?) operator - propagate error if Result/Option is error/None
        // For now, just evaluate the inner expression
        self.generate_expr(inner)
    }

    pub(super) fn generate_unwrap(&mut self, inner: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Unwrap (!) operator - panic if Result/Option is error/None
        // For now, just evaluate the inner expression
        self.generate_expr(inner)
    }

    // ========== Assignment ==========
}
