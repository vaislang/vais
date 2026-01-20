//! Type definitions and type conversion utilities for Vais code generator

use vais_ast::Type;
use vais_types::ResolvedType;

#[derive(Debug, Clone)]
pub(crate) struct LoopLabels {
    pub continue_label: String,
    pub break_label: String,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionInfo {
    pub name: String,
    pub params: Vec<(String, ResolvedType)>,
    pub ret_type: ResolvedType,
    pub is_extern: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct StructInfo {
    #[allow(dead_code)]
    pub name: String,
    pub fields: Vec<(String, ResolvedType)>,
}

#[derive(Debug, Clone)]
pub(crate) struct EnumInfo {
    #[allow(dead_code)]
    pub name: String,
    pub variants: Vec<EnumVariantInfo>,
}

#[derive(Debug, Clone)]
pub(crate) struct EnumVariantInfo {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub tag: u32,
    pub fields: EnumVariantFields,
}

#[derive(Debug, Clone)]
pub(crate) enum EnumVariantFields {
    Unit,
    Tuple(Vec<ResolvedType>),
    Struct(Vec<(String, ResolvedType)>),
}

#[derive(Debug, Clone)]
pub(crate) struct LocalVar {
    pub ty: ResolvedType,
    /// True if this is a function parameter (SSA value), false if alloca'd
    pub is_param: bool,
    /// The actual LLVM IR name for this variable (may differ from source name in loops)
    pub llvm_name: String,
}

/// Information about a closure (lambda with captures)
#[derive(Debug, Clone)]
pub(crate) struct ClosureInfo {
    /// The generated LLVM function name for this lambda
    #[allow(dead_code)]
    pub func_name: String,
    /// Captured variable names and their loaded values (var_name, llvm_value)
    pub captures: Vec<(String, String)>,
}

/// Information about an await point in an async function
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct AsyncAwaitPoint {
    /// State index after this await
    pub state_index: usize,
    /// Variable to store the awaited result
    pub result_var: String,
    /// LLVM type of the result
    pub result_type: String,
}

/// Information about the current async function being compiled
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct AsyncFunctionInfo {
    /// Original function name
    pub name: String,
    /// State struct name for this async function
    pub state_struct: String,
    /// Captured variables that need to be stored in state
    pub captured_vars: Vec<(String, ResolvedType)>,
    /// Return type of the future
    pub ret_type: ResolvedType,
}

use crate::CodeGenerator;

impl CodeGenerator {
    /// Convert a ResolvedType to LLVM IR type string
    pub(crate) fn type_to_llvm(&self, ty: &ResolvedType) -> String {
        match ty {
            ResolvedType::I8 => "i8".to_string(),
            ResolvedType::I16 => "i16".to_string(),
            ResolvedType::I32 => "i32".to_string(),
            ResolvedType::I64 => "i64".to_string(),
            ResolvedType::I128 => "i128".to_string(),
            ResolvedType::U8 => "i8".to_string(),
            ResolvedType::U16 => "i16".to_string(),
            ResolvedType::U32 => "i32".to_string(),
            ResolvedType::U64 => "i64".to_string(),
            ResolvedType::U128 => "i128".to_string(),
            ResolvedType::F32 => "float".to_string(),
            ResolvedType::F64 => "double".to_string(),
            ResolvedType::Bool => "i1".to_string(),
            ResolvedType::Str => "i8*".to_string(),
            ResolvedType::Unit => "void".to_string(),
            ResolvedType::Array(inner) => format!("{}*", self.type_to_llvm(inner)),
            ResolvedType::Pointer(inner) => format!("{}*", self.type_to_llvm(inner)),
            ResolvedType::Ref(inner) => format!("{}*", self.type_to_llvm(inner)),
            ResolvedType::RefMut(inner) => format!("{}*", self.type_to_llvm(inner)),
            ResolvedType::Range(_inner) => {
                // Range is represented as a struct with start and end fields
                // For now, we'll use a simple struct: { i64 start, i64 end, i1 inclusive }
                "%Range".to_string()
            }
            ResolvedType::Named { name, .. } => {
                // Single uppercase letter is likely a generic type parameter
                if name.len() == 1 && name.chars().next().map_or(false, |c| c.is_uppercase()) {
                    "i64".to_string()
                } else {
                    // Return struct type without pointer - caller adds * when needed
                    format!("%{}", name)
                }
            }
            ResolvedType::Generic(_) => "i64".to_string(), // Generic erased to i64 at runtime
            _ => "i64".to_string(), // Default fallback
        }
    }

    /// Get bit width for integer types
    pub(crate) fn get_integer_bits(&self, ty: &ResolvedType) -> u32 {
        match ty {
            ResolvedType::I8 | ResolvedType::U8 => 8,
            ResolvedType::I16 | ResolvedType::U16 => 16,
            ResolvedType::I32 | ResolvedType::U32 => 32,
            ResolvedType::I64 | ResolvedType::U64 => 64,
            ResolvedType::I128 | ResolvedType::U128 => 128,
            _ => 0, // Not an integer type
        }
    }

    /// Try to determine bit width from a value (heuristic based on SSA variable naming)
    pub(crate) fn get_integer_bits_from_val(&self, val: &str) -> u32 {
        // If it's a temp variable, we assume i64 (default Vais integer)
        // If it's a literal number, we assume i64
        if val.starts_with('%') || val.parse::<i64>().is_ok() {
            64
        } else {
            0
        }
    }

    /// Convert AST Type to ResolvedType
    pub(crate) fn ast_type_to_resolved(&self, ty: &Type) -> ResolvedType {
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
                    if name.len() == 1 && name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        ResolvedType::Generic(name.clone())
                    } else {
                        ResolvedType::Named {
                            name: name.clone(),
                            generics: generics
                                .iter()
                                .map(|g| self.ast_type_to_resolved(&g.node))
                                .collect(),
                        }
                    }
                }
            },
            Type::Array(inner) => {
                ResolvedType::Array(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Ref(inner) => ResolvedType::Ref(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Unit => ResolvedType::Unit,
            _ => ResolvedType::Unknown,
        }
    }

    /// Generate LLVM struct type definition
    pub(crate) fn generate_struct_type(&self, name: &str, info: &StructInfo) -> String {
        let fields: Vec<_> = info
            .fields
            .iter()
            .map(|(_, ty)| self.type_to_llvm(ty))
            .collect();

        format!("%{} = type {{ {} }}", name, fields.join(", "))
    }

    /// Generate LLVM enum type definition
    pub(crate) fn generate_enum_type(&self, name: &str, info: &EnumInfo) -> String {
        // Enum is represented as { i32 tag, union payload }
        // For simplicity, we use the largest variant size for the payload
        let mut max_payload_size = 0usize;
        let mut payload_types: Vec<String> = Vec::new();

        for variant in &info.variants {
            let variant_types = match &variant.fields {
                EnumVariantFields::Unit => vec![],
                EnumVariantFields::Tuple(types) => {
                    types.iter().map(|t| self.type_to_llvm(t)).collect()
                }
                EnumVariantFields::Struct(fields) => {
                    fields.iter().map(|(_, t)| self.type_to_llvm(t)).collect()
                }
            };

            // Estimate size (rough: each i64 = 8 bytes)
            let size = variant_types.len() * 8;
            if size > max_payload_size {
                max_payload_size = size;
                payload_types = variant_types;
            }
        }

        if payload_types.is_empty() {
            // Simple enum with no payload - just use i32 for tag
            format!("%{} = type {{ i32 }}", name)
        } else {
            // Enum with payload - tag + payload struct
            format!(
                "%{} = type {{ i32, {{ {} }} }}",
                name,
                payload_types.join(", ")
            )
        }
    }
}
