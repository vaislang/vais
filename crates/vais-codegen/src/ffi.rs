//! FFI (Foreign Function Interface) code generation support
//!
//! Generates LLVM IR for extern blocks, function pointers, and variadic functions.
//! Provides full C ABI compatibility with support for multiple calling conventions.

use crate::abi::CallingConvention;
use crate::{CodeGenerator, CodegenError, CodegenResult, FunctionInfo};
use vais_ast::*;
use vais_types::ResolvedType;

/// Threshold for passing structs by value vs by pointer in C ABI
/// Structs larger than 16 bytes are passed by pointer
const FFI_STRUCT_BYVAL_THRESHOLD: usize = 16;

impl CodeGenerator {
    /// Determine if a struct should be passed by value based on its size
    ///
    /// For C ABI compatibility:
    /// - Small structs (<=16 bytes): Pass in registers (by value)
    /// - Large structs (>16 bytes): Pass by pointer with `byval` attribute
    fn should_pass_struct_by_value(struct_size: usize) -> bool {
        struct_size <= FFI_STRUCT_BYVAL_THRESHOLD
    }

    /// Parse calling convention from ABI string
    ///
    /// Supported conventions:
    /// - "C" → cdecl (default)
    /// - "stdcall" → x86 stdcall
    /// - "fastcall" → x86 fastcall
    /// - "system" → platform-dependent (stdcall on Windows x86, cdecl elsewhere)
    fn parse_calling_convention(abi: &str) -> CallingConvention {
        CallingConvention::from_str(abi).unwrap_or(CallingConvention::C)
    }

    /// Validate that a type is FFI-safe
    ///
    /// Checks:
    /// - No generics in FFI types
    /// - No trait objects in FFI
    /// - Warns on platform-dependent types
    fn validate_ffi_type(&self, ty: &ResolvedType, param_name: &str) -> CodegenResult<()> {
        match ty {
            // Primitive types are always FFI-safe
            ResolvedType::I8 | ResolvedType::I16 | ResolvedType::I32 | ResolvedType::I64 |
            ResolvedType::I128 | ResolvedType::U8 | ResolvedType::U16 | ResolvedType::U32 |
            ResolvedType::U64 | ResolvedType::U128 | ResolvedType::F32 | ResolvedType::F64 |
            ResolvedType::Bool => Ok(()),

            // Pointers and references are FFI-safe
            ResolvedType::Pointer(inner) | ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                self.validate_ffi_type(inner, param_name)
            }

            // Function pointers are FFI-safe
            ResolvedType::FnPtr { params, ret, .. } => {
                for (i, param) in params.iter().enumerate() {
                    self.validate_ffi_type(param, &format!("{}[{}]", param_name, i))?;
                }
                self.validate_ffi_type(ret, &format!("{}:ret", param_name))
            }

            // Named types (structs, etc.) need validation
            ResolvedType::Named { name, generics } => {
                // FFI types must not have generic parameters
                if !generics.is_empty() {
                    return Err(CodegenError::TypeError(format!(
                        "FFI parameter '{}' has generic type '{}' - generics are not allowed in FFI",
                        param_name, name
                    )));
                }
                // Check if it's a known struct
                if let Some(_struct_info) = self.structs.get(name) {
                    // Struct is valid for FFI
                    Ok(())
                } else {
                    // Unknown type - might be an issue
                    eprintln!("Warning: FFI parameter '{}' uses unknown type '{}'", param_name, name);
                    Ok(())
                }
            }

            // Generic types are not allowed in FFI
            ResolvedType::Generic(name) => {
                Err(CodegenError::TypeError(format!(
                    "FFI parameter '{}' is generic type '{}' - generics are not allowed in FFI",
                    param_name, name
                )))
            }

            // Trait objects are not allowed in FFI
            ResolvedType::DynTrait { .. } => {
                Err(CodegenError::TypeError(format!(
                    "FFI parameter '{}' is a trait object - trait objects are not allowed in FFI",
                    param_name
                )))
            }

            // Unit type is FFI-safe (void in C)
            ResolvedType::Unit => Ok(()),

            // Tuples should be avoided in FFI but we'll allow them
            ResolvedType::Tuple(elements) => {
                eprintln!("Warning: FFI parameter '{}' is a tuple - consider using a struct instead", param_name);
                for (i, elem) in elements.iter().enumerate() {
                    self.validate_ffi_type(elem, &format!("{}[{}]", param_name, i))?;
                }
                Ok(())
            }

            // Arrays are FFI-safe but size matters
            ResolvedType::Array(elem) => {
                self.validate_ffi_type(elem, &format!("{}[]", param_name))
            }

            // String types are not directly FFI-safe
            ResolvedType::Str => {
                eprintln!("Warning: FFI parameter '{}' is 'str' - use *const i8 or *mut i8 instead", param_name);
                Ok(())
            }

            // Other types
            _ => {
                eprintln!("Warning: FFI parameter '{}' has type {:?} which may not be FFI-safe", param_name, ty);
                Ok(())
            }
        }
    }

    /// Validate all types in an extern function signature
    pub(crate) fn validate_ffi_function(
        &self,
        func_name: &str,
        params: &[(String, ResolvedType)],
        ret_type: &ResolvedType,
    ) -> CodegenResult<()> {
        // Validate all parameters
        for (param_name, param_type) in params {
            self.validate_ffi_type(param_type, param_name)?;
        }

        // Validate return type
        if !matches!(ret_type, ResolvedType::Unit) {
            self.validate_ffi_type(ret_type, &format!("{}:return", func_name))?;
        }

        Ok(())
    }

    /// Generate LLVM IR for extern block
    #[allow(dead_code)]
    pub(crate) fn generate_extern_block(&mut self, block: &ExternBlock) -> CodegenResult<String> {
        let mut ir = String::new();

        ir.push_str(&format!("; extern \"{}\" block\n", block.abi));

        for func in &block.functions {
            ir.push_str(&self.generate_extern_function(func, &block.abi)?);
        }

        Ok(ir)
    }

    /// Generate extern function declaration with full C ABI support
    #[allow(dead_code)]
    fn generate_extern_function(
        &mut self,
        func: &ExternFunction,
        abi: &str,
    ) -> CodegenResult<String> {
        let func_name = &func.name.node;

        // Parse calling convention from ABI string
        let calling_conv = Self::parse_calling_convention(abi);

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

        // Validate FFI types
        let params_with_names: Vec<_> = param_types
            .iter()
            .enumerate()
            .map(|(i, ty)| (format!("arg{}", i), ty.clone()))
            .collect();
        self.validate_ffi_function(func_name, &params_with_names, &ret_type)?;

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
            required_params: None,
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

        // Handle return type for C ABI
        // Large structs use sret (struct return) parameter
        let ret_llvm = self.type_to_llvm(&ret_type);
        let (actual_ret_type, sret_param) = if let ResolvedType::Named { name, .. } = &ret_type {
            if let Some(_struct_info) = self.structs.get(name) {
                let struct_size = self.type_size(&ret_type);
                if !Self::should_pass_struct_by_value(struct_size) {
                    // Large struct: use sret parameter, return void
                    (
                        "void".to_string(),
                        Some(format!("{} sret({}) %sret_result", ret_llvm, ret_llvm))
                    )
                } else {
                    (ret_llvm, None)
                }
            } else {
                (ret_llvm, None)
            }
        } else {
            (ret_llvm, None)
        };

        // Generate parameter list with struct handling
        let mut params_llvm: Vec<String> = Vec::new();

        // Add sret parameter first if needed
        if let Some(sret) = sret_param {
            params_llvm.push(sret);
        }

        // Add regular parameters with byval attribute for large structs
        for (i, ty) in param_types.iter().enumerate() {
            let llvm_ty = self.type_to_llvm(ty);
            let param_str = if let ResolvedType::Named { name, .. } = ty {
                if let Some(_struct_info) = self.structs.get(name) {
                    let struct_size = self.type_size(ty);
                    if !Self::should_pass_struct_by_value(struct_size) {
                        // Large struct: pass by pointer with byval
                        format!("{} byval({}) %{}", llvm_ty, llvm_ty, i)
                    } else {
                        // Small struct: pass by value
                        format!("{} %{}", llvm_ty, i)
                    }
                } else {
                    format!("{} %{}", llvm_ty, i)
                }
            } else {
                format!("{} %{}", llvm_ty, i)
            };
            params_llvm.push(param_str);
        }

        let vararg_suffix = if func.is_vararg { ", ..." } else { "" };

        // Generate declaration with calling convention
        let cc_str = calling_conv.to_llvm_str();

        Ok(format!(
            "declare {} {} @{}({}{})\n\n",
            cc_str,
            actual_ret_type,
            func_name,
            params_llvm.join(", "),
            vararg_suffix
        ))
    }

    /// Convert AST type to ResolvedType for FFI contexts (with error handling)
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub(crate) fn generate_fn_ptr_type(&self, params: &[ResolvedType], ret: &ResolvedType, is_vararg: bool) -> String {
        let param_types: Vec<String> = params.iter().map(|p| self.type_to_llvm(p)).collect();
        let ret_type = self.type_to_llvm(ret);

        let vararg_suffix = if is_vararg { ", ..." } else { "" };

        format!("{} ({}{})*", ret_type, param_types.join(", "), vararg_suffix)
    }

    /// Generate variadic function call
    /// For vararg calls, extra arguments are passed without type checking
    #[allow(dead_code)]
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
                    default_value: None,
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
        assert!(ir.contains("declare"));
        assert!(ir.contains("@puts"));
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
                    default_value: None,
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
        assert!(ir.contains("declare"));
        assert!(ir.contains("@printf"));
        assert!(ir.contains("..."));
    }

    #[test]
    fn test_calling_convention_parsing() {
        // Test C calling convention (default)
        let cc = CodeGenerator::parse_calling_convention("C");
        assert_eq!(cc, CallingConvention::C);
        assert_eq!(cc.to_llvm_str(), "ccc");

        // Test stdcall
        let cc = CodeGenerator::parse_calling_convention("stdcall");
        assert_eq!(cc, CallingConvention::StdCall);
        assert_eq!(cc.to_llvm_str(), "x86_stdcallcc");

        // Test fastcall
        let cc = CodeGenerator::parse_calling_convention("fastcall");
        assert_eq!(cc, CallingConvention::FastCall);
        assert_eq!(cc.to_llvm_str(), "x86_fastcallcc");

        // Test system (platform-dependent)
        let cc = CodeGenerator::parse_calling_convention("system");
        assert_eq!(cc, CallingConvention::System);
        #[cfg(all(target_os = "windows", target_arch = "x86"))]
        assert_eq!(cc.to_llvm_str(), "x86_stdcallcc");
        #[cfg(not(all(target_os = "windows", target_arch = "x86")))]
        assert_eq!(cc.to_llvm_str(), "ccc");

        // Test unknown ABI defaults to C
        let cc = CodeGenerator::parse_calling_convention("unknown");
        assert_eq!(cc, CallingConvention::C);
    }

    #[test]
    fn test_struct_parameter_passing_decision() {
        // Small structs (<=16 bytes) should be passed by value
        assert!(CodeGenerator::should_pass_struct_by_value(8));
        assert!(CodeGenerator::should_pass_struct_by_value(16));

        // Large structs (>16 bytes) should be passed by pointer
        assert!(!CodeGenerator::should_pass_struct_by_value(17));
        assert!(!CodeGenerator::should_pass_struct_by_value(32));
        assert!(!CodeGenerator::should_pass_struct_by_value(64));
    }

    #[test]
    fn test_ffi_type_validation_primitives() {
        let gen = CodeGenerator::new("test");

        // Primitive types should be valid
        assert!(gen.validate_ffi_type(&ResolvedType::I32, "test").is_ok());
        assert!(gen.validate_ffi_type(&ResolvedType::F64, "test").is_ok());
        assert!(gen.validate_ffi_type(&ResolvedType::Bool, "test").is_ok());
    }

    #[test]
    fn test_ffi_type_validation_pointers() {
        let gen = CodeGenerator::new("test");

        // Pointers should be valid
        let ptr_ty = ResolvedType::Pointer(Box::new(ResolvedType::I8));
        assert!(gen.validate_ffi_type(&ptr_ty, "test").is_ok());

        // References should be valid
        let ref_ty = ResolvedType::Ref(Box::new(ResolvedType::I32));
        assert!(gen.validate_ffi_type(&ref_ty, "test").is_ok());
    }

    #[test]
    fn test_ffi_type_validation_generics_rejected() {
        let gen = CodeGenerator::new("test");

        // Generic types should be rejected
        let generic_ty = ResolvedType::Generic("T".to_string());
        assert!(gen.validate_ffi_type(&generic_ty, "test").is_err());

        // Named types with generics should be rejected
        let generic_named = ResolvedType::Named {
            name: "Vec".to_string(),
            generics: vec![ResolvedType::I32],
        };
        assert!(gen.validate_ffi_type(&generic_named, "test").is_err());
    }

    #[test]
    fn test_ffi_type_validation_trait_objects_rejected() {
        let gen = CodeGenerator::new("test");

        // Trait objects should be rejected
        let trait_obj = ResolvedType::DynTrait {
            trait_name: "Display".to_string(),
            generics: vec![],
        };
        assert!(gen.validate_ffi_type(&trait_obj, "test").is_err());
    }

    #[test]
    fn test_ffi_type_validation_function_pointers() {
        let gen = CodeGenerator::new("test");

        // Function pointers should be valid
        let fn_ptr = ResolvedType::FnPtr {
            params: vec![ResolvedType::I32, ResolvedType::I32],
            ret: Box::new(ResolvedType::I64),
            is_vararg: false,
            effects: None,
        };
        assert!(gen.validate_ffi_type(&fn_ptr, "test").is_ok());
    }

    #[test]
    fn test_extern_function_with_calling_convention() {
        let mut gen = CodeGenerator::new("test");

        // Test stdcall calling convention
        let block = ExternBlock {
            abi: "stdcall".to_string(),
            functions: vec![ExternFunction {
                name: Spanned::new("WindowsAPI".to_string(), Span::new(0, 10)),
                params: vec![Param {
                    name: Spanned::new("x".to_string(), Span::new(0, 1)),
                    ty: Spanned::new(
                        Type::Named {
                            name: "i32".to_string(),
                            generics: vec![],
                        },
                        Span::new(0, 3),
                    ),
                    is_mut: false,
                    is_vararg: false,
                    ownership: Ownership::Regular,
                    default_value: None,
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
        assert!(ir.contains("x86_stdcallcc"));
        assert!(ir.contains("@WindowsAPI"));
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
