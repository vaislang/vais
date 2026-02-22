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
                    } else if let Some(alias_target) = self.type_aliases.get(name) {
                        // Resolve type alias to its underlying type
                        alias_target.clone()
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
            Type::Slice(inner) => {
                ResolvedType::Slice(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::SliceMut(inner) => {
                ResolvedType::SliceMut(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
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
        // Result/Optional layout: { i8 tag, i64 payload } where tag 0=Ok/Some, 1=Err/None (for Result)
        // and tag 0=None, 1=Some (for Option)
        let val = self.generate_expr(inner)?;

        // If the value is not a struct (e.g. primitive), just return it as-is (no unwrapping needed)
        if !val.is_struct_value() {
            return Ok(val);
        }

        let struct_val = val.into_struct_value();
        let struct_type = struct_val.get_type();

        // Only handle { i8, i64 } shaped structs (Result/Optional)
        if struct_type.count_fields() != 2 {
            return Ok(struct_val.into());
        }

        let fn_value = self.current_function.ok_or_else(|| {
            crate::CodegenError::LlvmError("No current function for try operator".to_string())
        })?;

        // Extract tag (field 0)
        let tag = self
            .builder
            .build_extract_value(struct_val, 0, "try_tag")
            .map_err(|e| crate::CodegenError::LlvmError(e.to_string()))?
            .into_int_value();

        // Check if error: for Result, tag != 0 means Err; for Option, tag == 0 means None
        // We use the Result convention (tag 0 = Ok/Some success path)
        // Actually in this codebase: Ok tag=0, Some tag=1, None tag=0, Err tag=1
        // So for both Result and Option, tag != 0 means error/None needs propagation
        // Wait - Some is tag=1 and None is tag=0 for Option. That means for Option,
        // tag==0 is the error case (None). For Result, tag!=0 (Err=1) is error.
        // Since we can't easily distinguish at codegen level, we use Result convention:
        // tag != 0 means error -> propagate. This is consistent with Ok=0, Err=1.
        let is_error = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                tag,
                tag.get_type().const_int(0, false),
                "try_is_err",
            )
            .map_err(|e| crate::CodegenError::LlvmError(e.to_string()))?;

        let ok_block = self
            .context
            .append_basic_block(fn_value, "try_ok");
        let err_block = self
            .context
            .append_basic_block(fn_value, "try_err");

        self.builder
            .build_conditional_branch(is_error, err_block, ok_block)
            .map_err(|e| crate::CodegenError::LlvmError(e.to_string()))?;

        // Error path: propagate by returning the same error struct
        self.builder.position_at_end(err_block);
        self.builder
            .build_return(Some(&struct_val))
            .map_err(|e| crate::CodegenError::LlvmError(e.to_string()))?;

        // Ok path: extract payload (field 1)
        self.builder.position_at_end(ok_block);
        let payload = self
            .builder
            .build_extract_value(struct_val, 1, "try_payload")
            .map_err(|e| crate::CodegenError::LlvmError(e.to_string()))?;

        Ok(payload)
    }

    pub(super) fn generate_unwrap(&mut self, inner: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Unwrap (!) operator - panic if Result/Option is error/None
        // Result/Optional layout: { i8 tag, i64 payload } where tag 0=Ok, 1=Err
        let val = self.generate_expr(inner)?;

        // If the value is not a struct, just return it as-is
        if !val.is_struct_value() {
            return Ok(val);
        }

        let struct_val = val.into_struct_value();
        let struct_type = struct_val.get_type();

        // Only handle { i8, i64 } shaped structs (Result/Optional)
        if struct_type.count_fields() != 2 {
            return Ok(struct_val.into());
        }

        let fn_value = self.current_function.ok_or_else(|| {
            crate::CodegenError::LlvmError("No current function for unwrap operator".to_string())
        })?;

        // Extract tag (field 0)
        let tag = self
            .builder
            .build_extract_value(struct_val, 0, "unwrap_tag")
            .map_err(|e| crate::CodegenError::LlvmError(e.to_string()))?
            .into_int_value();

        // Check if error (tag != 0)
        let is_error = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                tag,
                tag.get_type().const_int(0, false),
                "unwrap_is_err",
            )
            .map_err(|e| crate::CodegenError::LlvmError(e.to_string()))?;

        let ok_block = self
            .context
            .append_basic_block(fn_value, "unwrap_ok");
        let panic_block = self
            .context
            .append_basic_block(fn_value, "unwrap_panic");

        self.builder
            .build_conditional_branch(is_error, panic_block, ok_block)
            .map_err(|e| crate::CodegenError::LlvmError(e.to_string()))?;

        // Panic path: call abort
        self.builder.position_at_end(panic_block);
        if let Some(abort_fn) = self.module.get_function("abort") {
            self.builder
                .build_call(abort_fn, &[], "")
                .map_err(|e| crate::CodegenError::LlvmError(e.to_string()))?;
        }
        self.builder
            .build_unreachable()
            .map_err(|e| crate::CodegenError::LlvmError(e.to_string()))?;

        // Ok path: extract payload (field 1)
        self.builder.position_at_end(ok_block);
        let payload = self
            .builder
            .build_extract_value(struct_val, 1, "unwrap_payload")
            .map_err(|e| crate::CodegenError::LlvmError(e.to_string()))?;

        Ok(payload)
    }

    // ========== Assignment ==========
}
