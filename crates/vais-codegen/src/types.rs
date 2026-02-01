//! Type definitions and type conversion utilities for Vais code generator

use vais_ast::Type;
use vais_types::ResolvedType;

/// Format an f64 value as a valid LLVM IR floating-point constant.
/// LLVM requires format like "1.000000e+00", not Rust's "1.000000e0".
pub(crate) fn format_llvm_float(n: f64) -> String {
    // Use Rust's scientific notation then fix the exponent format
    let s = format!("{:.6e}", n);
    // Rust produces "1.000000e0", LLVM needs "1.000000e+00"
    // Find the 'e' and fix the exponent
    if let Some(e_pos) = s.rfind('e') {
        let (mantissa, exp_part) = s.split_at(e_pos);
        let exp_str = &exp_part[1..]; // skip 'e'
        if let Ok(exp_val) = exp_str.parse::<i32>() {
            if exp_val >= 0 {
                format!("{}e+{:02}", mantissa, exp_val)
            } else {
                format!("{}e-{:02}", mantissa, exp_val.unsigned_abs())
            }
        } else {
            s
        }
    } else {
        s
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LoopLabels {
    pub continue_label: String,
    pub break_label: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct FunctionInfo {
    pub signature: vais_types::FunctionSig,
    pub is_extern: bool,
    pub extern_abi: Option<String>, // ABI for extern functions (e.g., "C")
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct StructInfo {
    #[allow(dead_code)]
    pub name: String,
    pub fields: Vec<(String, ResolvedType)>,
    pub repr_c: bool, // true if #[repr(C)] attribute is present
    /// Invariant expressions for formal verification
    /// These are checked after struct construction/modification
    pub invariants: Vec<vais_ast::Spanned<vais_ast::Expr>>,
}

#[derive(Debug, Clone)]
pub(crate) struct EnumInfo {
    #[allow(dead_code)]
    pub name: String,
    pub variants: Vec<EnumVariantInfo>,
}

#[derive(Debug, Clone)]
pub(crate) struct UnionInfo {
    #[allow(dead_code)]
    pub name: String,
    pub fields: Vec<(String, ResolvedType)>,
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

/// Constant definition info
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct ConstInfo {
    pub name: String,
    pub ty: ResolvedType,
    pub value: vais_ast::Spanned<vais_ast::Expr>,
}

/// Global variable definition info
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct GlobalInfo {
    pub name: String,
    pub ty: ResolvedType,
    pub value: vais_ast::Spanned<vais_ast::Expr>,
    pub is_mutable: bool,
}

/// Represents the storage kind of a local variable
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalVarKind {
    /// Function parameter (SSA value, no alloca)
    Param,
    /// Immutable simple binding (SSA style, no alloca, direct register)
    Ssa,
    /// Stack-allocated variable (uses alloca)
    Alloca,
}

#[derive(Debug, Clone)]
pub(crate) struct LocalVar {
    pub ty: ResolvedType,
    /// The storage kind of this variable
    pub kind: LocalVarKind,
    /// The actual LLVM IR name for this variable (may differ from source name in loops)
    pub llvm_name: String,
}

impl LocalVar {
    /// Create a new parameter variable (SSA value, no alloca)
    pub fn param(ty: ResolvedType, llvm_name: impl Into<String>) -> Self {
        Self {
            ty,
            kind: LocalVarKind::Param,
            llvm_name: llvm_name.into(),
        }
    }

    /// Create a new SSA variable (immutable simple binding, no alloca)
    pub fn ssa(ty: ResolvedType, llvm_name: impl Into<String>) -> Self {
        Self {
            ty,
            kind: LocalVarKind::Ssa,
            llvm_name: llvm_name.into(),
        }
    }

    /// Create a new alloca variable (stack-allocated)
    pub fn alloca(ty: ResolvedType, llvm_name: impl Into<String>) -> Self {
        Self {
            ty,
            kind: LocalVarKind::Alloca,
            llvm_name: llvm_name.into(),
        }
    }

    /// Returns true if this is a function parameter
    #[inline]
    pub fn is_param(&self) -> bool {
        matches!(self.kind, LocalVarKind::Param)
    }

    /// Returns true if this variable uses SSA style (no alloca)
    #[inline]
    pub fn is_ssa(&self) -> bool {
        matches!(self.kind, LocalVarKind::Ssa)
    }

    /// Returns true if this variable uses alloca
    #[inline]
    #[allow(dead_code)]
    pub fn is_alloca(&self) -> bool {
        matches!(self.kind, LocalVarKind::Alloca)
    }
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
    /// Convert a ResolvedType to LLVM IR type string with caching
    pub(crate) fn type_to_llvm(&self, ty: &ResolvedType) -> String {
        // Create a key for caching - use the debug representation
        let cache_key = format!("{:?}", ty);

        // Check cache first
        if let Some(cached) = self.type_to_llvm_cache.borrow().get(&cache_key) {
            return cached.clone();
        }

        // Convert type to LLVM representation with recursion tracking
        let result = match self.type_to_llvm_impl(ty) {
            Ok(r) => r,
            Err(e) => {
                // On recursion error, return a fallback type
                #[cfg(debug_assertions)]
                eprintln!("Warning: {}", e);
                let _ = e;
                "i64".to_string()
            }
        };

        // Cache the result using interior mutability
        self.type_to_llvm_cache.borrow_mut().insert(cache_key, result.clone());
        result
    }

    /// Internal implementation of type_to_llvm without caching
    fn type_to_llvm_impl(&self, ty: &ResolvedType) -> crate::CodegenResult<String> {
        // Track recursion depth
        self.enter_type_recursion("type_to_llvm")?;

        let result = self.type_to_llvm_impl_inner(ty);

        // Always exit recursion, even on error
        self.exit_type_recursion();
        result
    }

    /// Inner implementation of type_to_llvm (actual conversion logic)
    fn type_to_llvm_impl_inner(&self, ty: &ResolvedType) -> crate::CodegenResult<String> {
        Ok(match ty {
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
            ResolvedType::Array(inner) => format!("{}*", self.type_to_llvm_impl(inner)?),
            ResolvedType::ConstArray { element, size } => {
                // Const-sized array: [N x T]
                let elem_ty = self.type_to_llvm_impl(element)?;
                match size.try_evaluate() {
                    Some(n) => format!("[{} x {}]", n, elem_ty),
                    None => {
                        // If size cannot be evaluated, fall back to pointer (dynamic array)
                        format!("{}*", elem_ty)
                    }
                }
            }
            ResolvedType::Pointer(inner) => format!("{}*", self.type_to_llvm_impl(inner)?),
            ResolvedType::Ref(inner) => {
                // &dyn Trait is a fat pointer itself (not a pointer to fat pointer)
                if matches!(inner.as_ref(), ResolvedType::DynTrait { .. }) {
                    self.type_to_llvm_impl(inner)?
                } else {
                    format!("{}*", self.type_to_llvm_impl(inner)?)
                }
            }
            ResolvedType::RefMut(inner) => {
                if matches!(inner.as_ref(), ResolvedType::DynTrait { .. }) {
                    self.type_to_llvm_impl(inner)?
                } else {
                    format!("{}*", self.type_to_llvm_impl(inner)?)
                }
            }
            ResolvedType::Range(_inner) => {
                // Range is represented as a struct with start and end fields
                // For now, we'll use a simple struct: { i64 start, i64 end, i1 inclusive }
                "%Range".to_string()
            }
            ResolvedType::Named { name, generics } => {
                // Single uppercase letter is likely a generic type parameter
                if name.len() == 1 && name.chars().next().is_some_and(|c| c.is_uppercase()) {
                    "i64".to_string()
                } else if !generics.is_empty() {
                    // In Vais, all values are i64-sized, so struct/enum/union layout is the same
                    // regardless of type arguments. Use base name for enums, structs, and unions.
                    if self.enums.contains_key(name) || self.structs.contains_key(name) || self.unions.contains_key(name) {
                        format!("%{}", name)
                    } else {
                        // Generic struct with type arguments (not in our structs map - external?)
                        // Check if all generics are concrete (not Generic or Var types)
                        let all_concrete = generics.iter().all(|g| {
                            !matches!(g, ResolvedType::Generic(_) | ResolvedType::Var(_))
                        });

                        if all_concrete {
                            // Use mangled name for concrete instantiations
                            let mangled = self.mangle_struct_name(name, generics);
                            format!("%{}", mangled)
                        } else {
                            // For generic types with unresolved parameters, use base struct name
                            // In Vais, all values are i64-sized, so struct layout is the same
                            format!("%{}", name)
                        }
                    }
                } else {
                    // Non-generic struct/enum/union - return type without pointer
                    format!("%{}", name)
                }
            }
            ResolvedType::Generic(param) => {
                // Check if we have a substitution for this generic parameter
                if let Some(concrete) = self.get_generic_substitution(param) {
                    self.type_to_llvm_impl(&concrete)?
                } else {
                    // Fallback to i64 for unresolved generics
                    "i64".to_string()
                }
            }
            ResolvedType::ConstGeneric(param) => {
                // Const generics should be resolved at monomorphization time
                // If we reach here, it's an error, but fall back to i64
                #[cfg(debug_assertions)]
                eprintln!("Warning: Unresolved const generic parameter: {}", param);
                let _ = param;
                "i64".to_string()
            }
            ResolvedType::Vector { element, lanes } => {
                // SIMD vector type: <lanes x element_type>
                let elem_ty = self.type_to_llvm_impl(element)?;
                format!("<{} x {}>", lanes, elem_ty)
            }
            ResolvedType::DynTrait { .. } => {
                // Dynamic trait object is a fat pointer: { data_ptr, vtable_ptr }
                // data_ptr: i8* pointing to the actual object data
                // vtable_ptr: i8* pointing to the vtable for this trait
                crate::vtable::TRAIT_OBJECT_TYPE.to_string()
            }
            ResolvedType::FnPtr { params, ret, is_vararg, .. } => {
                // Function pointer type
                let param_types: Result<Vec<String>, _> = params.iter().map(|p| self.type_to_llvm_impl(p)).collect();
                let param_types = param_types?;
                let ret_type = self.type_to_llvm_impl(ret)?;
                let vararg_suffix = if *is_vararg { ", ..." } else { "" };
                format!("{}({}{})*", ret_type, param_types.join(", "), vararg_suffix)
            }
            ResolvedType::Linear(inner) | ResolvedType::Affine(inner) => {
                // Linear and Affine types are transparent wrappers
                // They only affect type checking, not runtime representation
                self.type_to_llvm_impl(inner)?
            }
            ResolvedType::Dependent { base, .. } => {
                // Dependent types (refinement types) are transparent at runtime
                // The predicate is checked at compile time and potentially at runtime
                // via assertions, but the underlying representation is the base type
                self.type_to_llvm_impl(base)?
            }
            ResolvedType::RefLifetime { inner, .. } => {
                // Lifetime is erased at runtime, just generate pointer to inner type
                format!("{}*", self.type_to_llvm_impl(inner)?)
            }
            ResolvedType::RefMutLifetime { inner, .. } => {
                // Lifetime is erased at runtime, just generate pointer to inner type
                format!("{}*", self.type_to_llvm_impl(inner)?)
            }
            ResolvedType::Lifetime(_) => {
                // Lifetimes don't have a runtime representation
                // This shouldn't normally be reached in codegen
                "i64".to_string()
            }
            ResolvedType::Lazy(inner) => {
                // Lazy<T> is represented as a struct with:
                // - computed: i1 (has been evaluated)
                // - value: T (cached value)
                // - thunk: closure pointer (function to compute value)
                // For simplicity, we use a pointer to struct
                format!("{{ i1, {}, i8* }}", self.type_to_llvm_impl(inner)?)
            }
            _ => "i64".to_string(), // Default fallback
        })
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
        // Track recursion depth
        if self.enter_type_recursion("ast_type_to_resolved").is_err() {
            // On recursion limit, return Unknown type as fallback
            #[cfg(debug_assertions)]
            eprintln!("Warning: Type recursion limit exceeded in ast_type_to_resolved");
            return ResolvedType::Unknown;
        }

        let result = self.ast_type_to_resolved_impl(ty);

        // Always exit recursion
        self.exit_type_recursion();
        result
    }

    /// Internal implementation of ast_type_to_resolved
    fn ast_type_to_resolved_impl(&self, ty: &Type) -> ResolvedType {
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
                // SIMD Vector types
                "Vec2f32" => ResolvedType::Vector { element: Box::new(ResolvedType::F32), lanes: 2 },
                "Vec4f32" => ResolvedType::Vector { element: Box::new(ResolvedType::F32), lanes: 4 },
                "Vec8f32" => ResolvedType::Vector { element: Box::new(ResolvedType::F32), lanes: 8 },
                "Vec2f64" => ResolvedType::Vector { element: Box::new(ResolvedType::F64), lanes: 2 },
                "Vec4f64" => ResolvedType::Vector { element: Box::new(ResolvedType::F64), lanes: 4 },
                "Vec4i32" => ResolvedType::Vector { element: Box::new(ResolvedType::I32), lanes: 4 },
                "Vec8i32" => ResolvedType::Vector { element: Box::new(ResolvedType::I32), lanes: 8 },
                "Vec2i64" => ResolvedType::Vector { element: Box::new(ResolvedType::I64), lanes: 2 },
                "Vec4i64" => ResolvedType::Vector { element: Box::new(ResolvedType::I64), lanes: 4 },
                _ => {
                    // Single uppercase letter is likely a generic type parameter
                    if name.len() == 1 && name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        ResolvedType::Generic(name.clone())
                    } else {
                        ResolvedType::Named {
                            name: name.clone(),
                            generics: generics
                                .iter()
                                .map(|g| self.ast_type_to_resolved_impl(&g.node))
                                .collect(),
                        }
                    }
                }
            },
            Type::Array(inner) => {
                ResolvedType::Array(Box::new(self.ast_type_to_resolved_impl(&inner.node)))
            }
            Type::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.ast_type_to_resolved_impl(&inner.node)))
            }
            Type::Ref(inner) => ResolvedType::Ref(Box::new(self.ast_type_to_resolved_impl(&inner.node))),
            Type::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.ast_type_to_resolved_impl(&inner.node)))
            }
            Type::RefLifetime { lifetime, inner } => {
                // Lifetime info is preserved but runtime representation is same as regular ref
                ResolvedType::RefLifetime {
                    lifetime: lifetime.clone(),
                    inner: Box::new(self.ast_type_to_resolved_impl(&inner.node)),
                }
            }
            Type::RefMutLifetime { lifetime, inner } => {
                ResolvedType::RefMutLifetime {
                    lifetime: lifetime.clone(),
                    inner: Box::new(self.ast_type_to_resolved_impl(&inner.node)),
                }
            }
            Type::Unit => ResolvedType::Unit,
            Type::DynTrait { trait_name, generics } => {
                ResolvedType::DynTrait {
                    trait_name: trait_name.clone(),
                    generics: generics
                        .iter()
                        .map(|g| self.ast_type_to_resolved_impl(&g.node))
                        .collect(),
                }
            }
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

            // Estimate size based on actual field types
            let size: usize = variant_types.iter().map(|t| self.estimate_type_size(t)).sum();
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

    /// Generate LLVM union type definition (untagged, C-style)
    /// All fields share the same memory location (offset 0).
    /// The type is sized to the largest field.
    pub(crate) fn generate_union_type(&self, name: &str, info: &UnionInfo) -> String {
        // Find the largest field type (by estimated size)
        let largest_type = info
            .fields
            .iter()
            .map(|(_, ty)| self.type_to_llvm(ty))
            .max_by_key(|s| self.estimate_type_size(s))
            .unwrap_or_else(|| "i64".to_string());

        format!("%{} = type {{ {} }}", name, largest_type)
    }

    /// Estimate the size of an LLVM type (for union layout)
    fn estimate_type_size(&self, llvm_type: &str) -> usize {
        match llvm_type {
            "i1" => 1,
            "i8" => 1,
            "i16" => 2,
            "i32" | "float" => 4,
            "i64" | "double" | "i8*" => 8,
            "i128" => 16,
            s if s.ends_with('*') => 8, // pointers are 8 bytes on 64-bit
            s if s.starts_with('<') => {
                // SIMD vector type: <N x T>
                // Parse and calculate
                if let Some(rest) = s.strip_prefix('<') {
                    if let Some(idx) = rest.find(" x ") {
                        if let Ok(lanes) = rest[..idx].trim().parse::<usize>() {
                            let elem_type = &rest[idx + 3..rest.len() - 1];
                            return lanes * self.estimate_type_size(elem_type);
                        }
                    }
                }
                8 // fallback
            }
            s if s.starts_with('%') => 8, // Named types default to 8
            _ => 8, // Default fallback
        }
    }
}
