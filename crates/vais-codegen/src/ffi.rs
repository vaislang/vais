//! FFI (Foreign Function Interface) code generation support
//!
//! Generates LLVM IR for extern blocks, function pointers, and variadic functions.

use crate::{CodeGenerator, CodegenError, CodegenResult, FunctionInfo};
use vais_ast::*;
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Generate LLVM IR for extern block
    pub(crate) fn generate_extern_block(&mut self, block: &ExternBlock) -> CodegenResult<String> {
        let mut ir = String::new();

        ir.push_str(&format!("; extern \"{}\" block\n", block.abi));

        for func in &block.functions {
            ir.push_str(&self.generate_extern_function(func, &block.abi)?);
        }

        Ok(ir)
    }

    /// Generate extern function declaration
    fn generate_extern_function(
        &mut self,
        func: &ExternFunction,
        abi: &str,
    ) -> CodegenResult<String> {
        let func_name = &func.name.node;

        // Convert parameter types to LLVM types
        let mut param_types = Vec::new();
        for param in &func.params {
            let ty = self.ffi_ast_type_to_resolved(&param.ty)?;
            param_types.push(ty);
        }

        // Get return type
        let ret_type = if let Some(ret) = &func.ret_type {
            self.ffi_ast_type_to_resolved(ret)?
        } else {
            ResolvedType::Unit
        };

        // Register the function signature
        let func_sig = vais_types::FunctionSig {
            name: func_name.clone(),
            generics: vec![],
            generic_bounds: std::collections::HashMap::new(),
            params: param_types
                .iter()
                .enumerate()
                .map(|(i, ty)| (format!("arg{}", i), ty.clone(), false))
                .collect(),
            ret: ret_type.clone(),
            is_async: false,
            is_vararg: func.is_vararg,
            contracts: None,
            effect_annotation: vais_types::EffectAnnotation::Infer,
            inferred_effects: None,
        };

        self.functions.insert(
            func_name.clone(),
            FunctionInfo {
                signature: func_sig,
                is_extern: true,
                extern_abi: Some(abi.to_string()),
            },
        );

        // Generate extern function declaration
        let ret_llvm = self.type_to_llvm(&ret_type);
        let params_llvm: Vec<String> = param_types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("{} %{}", self.type_to_llvm(ty), i))
            .collect();

        let vararg_suffix = if func.is_vararg { ", ..." } else { "" };

        Ok(format!(
            "declare {} @{}({}{})\n\n",
            ret_llvm,
            func_name,
            params_llvm.join(", "),
            vararg_suffix
        ))
    }

    /// Convert AST type to ResolvedType for FFI contexts (with error handling)
    fn ffi_ast_type_to_resolved(&self, ty: &Spanned<Type>) -> CodegenResult<ResolvedType> {
        match &ty.node {
            Type::Named { name, generics } => {
                match name.as_str() {
                    "i8" => Ok(ResolvedType::I8),
                    "i16" => Ok(ResolvedType::I16),
                    "i32" => Ok(ResolvedType::I32),
                    "i64" => Ok(ResolvedType::I64),
                    "i128" => Ok(ResolvedType::I128),
                    "u8" => Ok(ResolvedType::U8),
                    "u16" => Ok(ResolvedType::U16),
                    "u32" => Ok(ResolvedType::U32),
                    "u64" => Ok(ResolvedType::U64),
                    "u128" => Ok(ResolvedType::U128),
                    "f32" => Ok(ResolvedType::F32),
                    "f64" => Ok(ResolvedType::F64),
                    "bool" => Ok(ResolvedType::Bool),
                    "str" => Ok(ResolvedType::Str),
                    _ => {
                        let generic_types: Result<Vec<_>, _> = generics
                            .iter()
                            .map(|g| self.ffi_ast_type_to_resolved(g))
                            .collect();
                        Ok(ResolvedType::Named {
                            name: name.clone(),
                            generics: generic_types?,
                        })
                    }
                }
            }
            Type::Pointer(inner) => {
                let inner_ty = self.ffi_ast_type_to_resolved(inner)?;
                Ok(ResolvedType::Pointer(Box::new(inner_ty)))
            }
            Type::FnPtr { params, ret, is_vararg } => {
                let param_types: Result<Vec<_>, _> =
                    params.iter().map(|p| self.ffi_ast_type_to_resolved(p)).collect();
                let ret_type = self.ffi_ast_type_to_resolved(ret)?;
                Ok(ResolvedType::FnPtr {
                    params: param_types?,
                    ret: Box::new(ret_type),
                    is_vararg: *is_vararg,
                    effects: None,
                })
            }
            Type::Unit => Ok(ResolvedType::Unit),
            Type::Tuple(elements) => {
                let elem_types: Result<Vec<_>, _> =
                    elements.iter().map(|e| self.ffi_ast_type_to_resolved(e)).collect();
                Ok(ResolvedType::Tuple(elem_types?))
            }
            Type::Array(elem) => {
                let elem_ty = self.ffi_ast_type_to_resolved(elem)?;
                Ok(ResolvedType::Array(Box::new(elem_ty)))
            }
            _ => Err(CodegenError::Unsupported(format!(
                "AST type conversion for FFI: {:?}",
                ty.node
            ))),
        }
    }

    /// Generate function pointer type in LLVM
    pub(crate) fn generate_fn_ptr_type(&self, params: &[ResolvedType], ret: &ResolvedType, is_vararg: bool) -> String {
        let param_types: Vec<String> = params.iter().map(|p| self.type_to_llvm(p)).collect();
        let ret_type = self.type_to_llvm(ret);

        let vararg_suffix = if is_vararg { ", ..." } else { "" };

        format!("{} ({}{})*", ret_type, param_types.join(", "), vararg_suffix)
    }

    /// Generate variadic function call
    /// For vararg calls, extra arguments are passed without type checking
    pub(crate) fn generate_vararg_call(
        &mut self,
        func_name: &str,
        args: &[String],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();

        // Look up function signature
        let func_info = self.functions.get(func_name).ok_or_else(|| {
            CodegenError::UndefinedFunction(func_name.to_string())
        })?;

        if !func_info.signature.is_vararg {
            return Err(CodegenError::TypeError(format!(
                "Function {} is not variadic",
                func_name
            )));
        }

        let ret_type = &func_info.signature.ret;
        let ret_llvm = self.type_to_llvm(ret_type);

        // Build call instruction with all arguments
        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call {} @{}({})\n",
            result,
            ret_llvm,
            func_name,
            args.join(", ")
        ));

        Ok((result, ir))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extern_block_generation() {
        let mut gen = CodeGenerator::new("test");

        let block = ExternBlock {
            abi: "C".to_string(),
            functions: vec![ExternFunction {
                name: Spanned::new("puts".to_string(), Span::new(0, 4)),
                params: vec![Param {
                    name: Spanned::new("s".to_string(), Span::new(0, 1)),
                    ty: Spanned::new(
                        Type::Pointer(Box::new(Spanned::new(
                            Type::Named {
                                name: "i8".to_string(),
                                generics: vec![],
                            },
                            Span::new(0, 2),
                        ))),
                        Span::new(0, 3),
                    ),
                    is_mut: false,
                    is_vararg: false,
                    ownership: Ownership::Regular,
                }],
                ret_type: Some(Spanned::new(
                    Type::Named {
                        name: "i32".to_string(),
                        generics: vec![],
                    },
                    Span::new(0, 3),
                )),
                is_vararg: false,
            }],
        };

        let ir = gen.generate_extern_block(&block).unwrap();
        assert!(ir.contains("declare i32 @puts"));
        assert!(ir.contains("i8*"));
    }

    #[test]
    fn test_vararg_extern_function() {
        let mut gen = CodeGenerator::new("test");

        let block = ExternBlock {
            abi: "C".to_string(),
            functions: vec![ExternFunction {
                name: Spanned::new("printf".to_string(), Span::new(0, 6)),
                params: vec![Param {
                    name: Spanned::new("fmt".to_string(), Span::new(0, 3)),
                    ty: Spanned::new(
                        Type::Pointer(Box::new(Spanned::new(
                            Type::Named {
                                name: "i8".to_string(),
                                generics: vec![],
                            },
                            Span::new(0, 2),
                        ))),
                        Span::new(0, 3),
                    ),
                    is_mut: false,
                    is_vararg: false,
                    ownership: Ownership::Regular,
                }],
                ret_type: Some(Spanned::new(
                    Type::Named {
                        name: "i32".to_string(),
                        generics: vec![],
                    },
                    Span::new(0, 3),
                )),
                is_vararg: true,
            }],
        };

        let ir = gen.generate_extern_block(&block).unwrap();
        assert!(ir.contains("declare i32 @printf"));
        assert!(ir.contains("..."));
    }

    #[test]
    fn test_function_pointer_type() {
        let gen = CodeGenerator::new("test");

        let params = vec![ResolvedType::I32, ResolvedType::I32];
        let ret = ResolvedType::I64;

        let fn_ptr_type = gen.generate_fn_ptr_type(&params, &ret, false);
        assert!(fn_ptr_type.contains("i64"));
        assert!(fn_ptr_type.contains("i32"));
        assert!(fn_ptr_type.contains("*"));
    }

    #[test]
    fn test_vararg_function_pointer() {
        let gen = CodeGenerator::new("test");

        let params = vec![ResolvedType::Pointer(Box::new(ResolvedType::I8))];
        let ret = ResolvedType::I32;

        let fn_ptr_type = gen.generate_fn_ptr_type(&params, &ret, true);
        assert!(fn_ptr_type.contains("..."));
        assert!(fn_ptr_type.contains("i32"));
        assert!(fn_ptr_type.contains("i8*"));
    }
}
