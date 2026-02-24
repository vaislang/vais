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

#[derive(Debug, Clone)]
pub(crate) struct FunctionInfo {
    pub signature: vais_types::FunctionSig,
    pub is_extern: bool,
    pub _extern_abi: Option<String>, // ABI for extern functions (e.g., "C")
}

#[derive(Debug, Clone)]
pub(crate) struct StructInfo {
    pub _name: String,
    pub fields: Vec<(String, ResolvedType)>,
    pub _repr_c: bool, // true if #[repr(C)] attribute is present
    /// Invariant expressions for formal verification
    /// These are checked after struct construction/modification
    pub _invariants: Vec<vais_ast::Spanned<vais_ast::Expr>>,
}

#[derive(Debug, Clone)]
pub(crate) struct EnumInfo {
    pub name: String,
    pub variants: Vec<EnumVariantInfo>,
}

#[derive(Debug, Clone)]
pub(crate) struct UnionInfo {
    pub _name: String,
    pub fields: Vec<(String, ResolvedType)>,
}

#[derive(Debug, Clone)]
pub(crate) struct EnumVariantInfo {
    pub name: String,
    pub _tag: u32,
    pub fields: EnumVariantFields,
}

#[derive(Debug, Clone)]
pub(crate) enum EnumVariantFields {
    Unit,
    Tuple(Vec<ResolvedType>),
    Struct(Vec<(String, ResolvedType)>),
}

/// Constant definition info
#[derive(Debug, Clone)]
pub(crate) struct ConstInfo {
    pub _name: String,
    pub _ty: ResolvedType,
    pub value: vais_ast::Spanned<vais_ast::Expr>,
}

/// Global variable definition info
#[derive(Debug, Clone)]
pub(crate) struct GlobalInfo {
    pub _name: String,
    pub _ty: ResolvedType,
    pub _value: vais_ast::Spanned<vais_ast::Expr>,
    pub _is_mutable: bool,
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
    pub fn is_alloca(&self) -> bool {
        matches!(self.kind, LocalVarKind::Alloca)
    }
}

/// Information about a closure (lambda with captures)
#[derive(Debug, Clone)]
pub(crate) struct ClosureInfo {
    /// The generated LLVM function name for this lambda
    pub func_name: String,
    /// Captured variable names and their loaded values (var_name, llvm_value)
    pub captures: Vec<(String, String)>,
    /// Whether captures are passed by reference (pointer) vs by value
    pub is_ref_capture: bool,
}

/// Information about a lazy thunk (deferred evaluation)
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields reserved for lazy evaluation codegen
pub(crate) struct LazyThunkInfo {
    /// The generated LLVM thunk function name
    pub thunk_name: String,
    /// Captured variable names, LLVM types, and loaded values (var_name, llvm_type, llvm_value)
    pub captures: Vec<(String, String, String)>,
    /// LLVM type of the inner (computed) value
    pub inner_llvm_ty: String,
}

/// Information about an await point in an async function.
/// Used to track suspension points in the async state machine.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields reserved for async state machine codegen
pub(crate) struct AsyncAwaitPoint {
    /// State index after this await
    pub state_index: usize,
    /// Variable to store the awaited result
    pub result_var: String,
    /// LLVM type of the result
    pub result_type: String,
}

/// Information about the current async function being compiled.
/// Tracks the state machine structure for async function code generation.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields reserved for async function codegen
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
        // All types use cache - primitive types cached on first call
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
                String::from("i64")
            }
        };

        // Cache the result using interior mutability
        self.type_to_llvm_cache
            .borrow_mut()
            .insert(cache_key, result.clone());
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
            ResolvedType::I8 => String::from("i8"),
            ResolvedType::I16 => String::from("i16"),
            ResolvedType::I32 => String::from("i32"),
            ResolvedType::I64 => String::from("i64"),
            ResolvedType::I128 => String::from("i128"),
            ResolvedType::U8 => String::from("i8"),
            ResolvedType::U16 => String::from("i16"),
            ResolvedType::U32 => String::from("i32"),
            ResolvedType::U64 => String::from("i64"),
            ResolvedType::U128 => String::from("i128"),
            ResolvedType::F32 => String::from("float"),
            ResolvedType::F64 => String::from("double"),
            ResolvedType::Bool => String::from("i1"),
            ResolvedType::Str => String::from("i8*"),
            ResolvedType::Unit => String::from("void"),
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
                // &[T] (Slice) and &mut [T] (SliceMut) are also fat pointers { i8*, i64 }
                // — a slice reference IS a fat pointer, not a pointer to one
                match inner.as_ref() {
                    ResolvedType::DynTrait { .. }
                    | ResolvedType::Slice(_)
                    | ResolvedType::SliceMut(_) => self.type_to_llvm_impl(inner)?,
                    _ => format!("{}*", self.type_to_llvm_impl(inner)?),
                }
            }
            ResolvedType::RefMut(inner) => {
                match inner.as_ref() {
                    ResolvedType::DynTrait { .. }
                    | ResolvedType::Slice(_)
                    | ResolvedType::SliceMut(_) => self.type_to_llvm_impl(inner)?,
                    _ => format!("{}*", self.type_to_llvm_impl(inner)?),
                }
            }
            ResolvedType::Range(_inner) => {
                // Range is represented as a struct: { i64 start, i64 end, i1 inclusive }
                String::from("{ i64, i64, i1 }")
            }
            ResolvedType::Named { name, generics } => {
                // Single uppercase letter is likely a generic type parameter
                if name.len() == 1 && name.chars().next().is_some_and(|c| c.is_uppercase()) {
                    String::from("i64")
                } else if !generics.is_empty() {
                    // In Vais, all values are i64-sized, so struct/enum/union layout is the same
                    // regardless of type arguments. Use base name for enums, structs, and unions.
                    if self.types.enums.contains_key(name)
                        || self.types.structs.contains_key(name)
                        || self.types.unions.contains_key(name)
                    {
                        format!("%{}", name)
                    } else {
                        // Generic struct with type arguments (not in our structs map - external?)
                        // Check if all generics are concrete (not Generic or Var types)
                        let all_concrete = generics
                            .iter()
                            .all(|g| !matches!(g, ResolvedType::Generic(_) | ResolvedType::Var(_)));

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
                    // Generic parameter without substitution — use i64 fallback.
                    // NOTE: returning Err here would break nested types like &T → i64 instead of i64*,
                    // because the error short-circuits the wrapper type conversion.
                    // This fallback is safe when generate_module (not generate_module_with_instantiations)
                    // is used, where generic functions are codegen'd with i64 as the default type.
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "Warning: unresolved generic parameter '{}' reached codegen, using i64 fallback",
                        param
                    );
                    String::from("i64")
                }
            }
            ResolvedType::ConstGeneric(param) => {
                // Check if we have a substitution for this const generic parameter
                if let Some(concrete) = self.get_generic_substitution(param) {
                    self.type_to_llvm_impl(&concrete)?
                } else {
                    // ConstGeneric parameter without substitution — use i64 fallback.
                    // Same rationale as Generic above.
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "Warning: unresolved const generic '{}' reached codegen, using i64 fallback",
                        param
                    );
                    String::from("i64")
                }
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
            ResolvedType::ImplTrait { .. } => {
                return Err(crate::CodegenError::InternalError(
                    "ImplTrait should be monomorphized before codegen".to_string()
                ));
            }
            ResolvedType::FnPtr {
                params,
                ret,
                is_vararg,
                ..
            } => {
                // Function pointer type
                let param_types: Result<Vec<String>, _> =
                    params.iter().map(|p| self.type_to_llvm_impl(p)).collect();
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
                return Err(crate::CodegenError::InternalError(
                    "bare lifetime has no runtime representation".to_string()
                ));
            }
            ResolvedType::Map(key, _val) => {
                // Map is represented as a pointer to key array (parallel arrays)
                format!("{}*", self.type_to_llvm_impl(key)?)
            }
            ResolvedType::Lazy(inner) => {
                // Lazy<T> is represented as a struct with:
                // - computed: i1 (has been evaluated)
                // - value: T (cached value)
                // - thunk: closure pointer (function to compute value)
                format!("{{ i1, {}, i8* }}", self.type_to_llvm_impl(inner)?)
            }
            ResolvedType::Tuple(elems) => {
                let elem_types: Vec<String> = elems
                    .iter()
                    .map(|e| self.type_to_llvm_impl(e))
                    .collect::<Result<_, _>>()?;
                format!("{{ {} }}", elem_types.join(", "))
            }
            ResolvedType::Slice(_) | ResolvedType::SliceMut(_) => {
                // Slice is a fat pointer: { i8* data, i64 length }
                String::from("{ i8*, i64 }")
            }
            ResolvedType::Fn { params, ret, .. } => {
                // Function type as pointer (same as FnPtr at runtime)
                let param_types: Result<Vec<String>, _> =
                    params.iter().map(|p| self.type_to_llvm_impl(p)).collect();
                let param_types = param_types?;
                let ret_type = self.type_to_llvm_impl(ret)?;
                format!("{}({})*", ret_type, param_types.join(", "))
            }
            ResolvedType::Optional(inner) => {
                // Option<T> is { i8 tag, T value }
                let inner_ty = self.type_to_llvm_impl(inner)?;
                format!("{{ i8, {} }}", inner_ty)
            }
            ResolvedType::Result(ok, _err) => {
                // Result<T, E> is { i8 tag, T value } (use ok type for payload)
                let ok_ty = self.type_to_llvm_impl(ok)?;
                format!("{{ i8, {} }}", ok_ty)
            }
            ResolvedType::Future(_) => {
                // Future is an opaque pointer to async state machine
                // Represented as i64 in text IR (pointer-as-integer convention)
                String::from("i64")
            }
            ResolvedType::Never => {
                // Never type — functions that return ! use void
                String::from("void")
            }
            ResolvedType::Var(_) | ResolvedType::Unknown => {
                return Err(crate::CodegenError::InternalError(
                    "unresolved type variable reached codegen".to_string()
                ));
            }
            ResolvedType::Associated { .. } => {
                return Err(crate::CodegenError::InternalError(
                    "unresolved associated type in codegen".to_string()
                ));
            }
            ResolvedType::HigherKinded { .. } => {
                return Err(crate::CodegenError::InternalError(
                    "unresolved higher-kinded type in codegen".to_string()
                ));
            }
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
                "Vec2f32" => ResolvedType::Vector {
                    element: Box::new(ResolvedType::F32),
                    lanes: 2,
                },
                "Vec4f32" => ResolvedType::Vector {
                    element: Box::new(ResolvedType::F32),
                    lanes: 4,
                },
                "Vec8f32" => ResolvedType::Vector {
                    element: Box::new(ResolvedType::F32),
                    lanes: 8,
                },
                "Vec2f64" => ResolvedType::Vector {
                    element: Box::new(ResolvedType::F64),
                    lanes: 2,
                },
                "Vec4f64" => ResolvedType::Vector {
                    element: Box::new(ResolvedType::F64),
                    lanes: 4,
                },
                "Vec4i32" => ResolvedType::Vector {
                    element: Box::new(ResolvedType::I32),
                    lanes: 4,
                },
                "Vec8i32" => ResolvedType::Vector {
                    element: Box::new(ResolvedType::I32),
                    lanes: 8,
                },
                "Vec2i64" => ResolvedType::Vector {
                    element: Box::new(ResolvedType::I64),
                    lanes: 2,
                },
                "Vec4i64" => ResolvedType::Vector {
                    element: Box::new(ResolvedType::I64),
                    lanes: 4,
                },
                _ => {
                    // Single uppercase letter is likely a generic type parameter
                    if name.len() == 1 && name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        if generics.is_empty() {
                            ResolvedType::Generic(name.clone())
                        } else {
                            // HKT application: F<A> — keep as Named so substitute_type
                            // can replace the constructor name
                            ResolvedType::Named {
                                name: name.clone(),
                                generics: generics
                                    .iter()
                                    .map(|g| self.ast_type_to_resolved_impl(&g.node))
                                    .collect(),
                            }
                        }
                    } else if let Some(alias_target) = self.types.type_aliases.get(name) {
                        // Resolve type alias to its underlying type
                        alias_target.clone()
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
            Type::Ref(inner) => {
                ResolvedType::Ref(Box::new(self.ast_type_to_resolved_impl(&inner.node)))
            }
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
            Type::RefMutLifetime { lifetime, inner } => ResolvedType::RefMutLifetime {
                lifetime: lifetime.clone(),
                inner: Box::new(self.ast_type_to_resolved_impl(&inner.node)),
            },
            Type::Tuple(elems) => ResolvedType::Tuple(
                elems
                    .iter()
                    .map(|e| self.ast_type_to_resolved_impl(&e.node))
                    .collect(),
            ),
            Type::Unit => ResolvedType::Unit,
            Type::DynTrait {
                trait_name,
                generics,
            } => ResolvedType::DynTrait {
                trait_name: trait_name.clone(),
                generics: generics
                    .iter()
                    .map(|g| self.ast_type_to_resolved_impl(&g.node))
                    .collect(),
            },
            Type::ImplTrait { bounds } => ResolvedType::ImplTrait {
                bounds: bounds.iter().map(|b| b.node.clone()).collect(),
            },
            Type::Slice(inner) => {
                ResolvedType::Slice(Box::new(self.ast_type_to_resolved_impl(&inner.node)))
            }
            Type::SliceMut(inner) => {
                ResolvedType::SliceMut(Box::new(self.ast_type_to_resolved_impl(&inner.node)))
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
            let size: usize = variant_types
                .iter()
                .map(|t| self.estimate_type_size(t))
                .sum();
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
            .unwrap_or_else(|| String::from("i64"));

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
            _ => 8,                       // Default fallback
        }
    }

    /// Compute sizeof for a ResolvedType (in bytes)
    /// Returns the size in Vais's runtime representation
    pub(crate) fn compute_sizeof(&self, ty: &ResolvedType) -> i64 {
        match ty {
            ResolvedType::I8 | ResolvedType::U8 | ResolvedType::Bool => 1,
            ResolvedType::I16 | ResolvedType::U16 => 2,
            ResolvedType::I32 | ResolvedType::U32 | ResolvedType::F32 => 4,
            ResolvedType::I64 | ResolvedType::U64 | ResolvedType::F64 => 8,
            ResolvedType::I128 | ResolvedType::U128 => 16,
            ResolvedType::Str => 8, // pointer
            ResolvedType::Unit => 0,
            ResolvedType::Pointer(_) | ResolvedType::Ref(_) | ResolvedType::RefMut(_) => 8,
            ResolvedType::Array(_) => 8,     // pointer to heap
            ResolvedType::Optional(_) => 8,  // tag + value in i64
            ResolvedType::Result(_, _) => 8, // tag + value in i64
            ResolvedType::Tuple(elems) => elems.iter().map(|e| self.compute_sizeof(e)).sum(),
            ResolvedType::Named { name, .. } => {
                if let Some(struct_info) = self.types.structs.get(name) {
                    struct_info
                        .fields
                        .iter()
                        .map(|(_name, ty)| self.compute_sizeof(ty))
                        .sum()
                } else {
                    8 // enum (tag + payload) or unknown named type
                }
            }
            _ => 8, // default for complex types
        }
    }

    /// Compute alignof for a ResolvedType (in bytes)
    /// Returns the alignment requirement of the type
    pub(crate) fn compute_alignof(&self, ty: &ResolvedType) -> i64 {
        match ty {
            ResolvedType::I8 | ResolvedType::U8 | ResolvedType::Bool => 1,
            ResolvedType::I16 | ResolvedType::U16 => 2,
            ResolvedType::I32 | ResolvedType::U32 | ResolvedType::F32 => 4,
            ResolvedType::I64 | ResolvedType::U64 | ResolvedType::F64 => 8,
            ResolvedType::I128 | ResolvedType::U128 => 16,
            ResolvedType::Str
            | ResolvedType::Pointer(_)
            | ResolvedType::Ref(_)
            | ResolvedType::RefMut(_) => 8,
            ResolvedType::Unit => 1,
            ResolvedType::Tuple(elems) => elems
                .iter()
                .map(|e| self.compute_alignof(e))
                .max()
                .unwrap_or(8),
            ResolvedType::Named { name, .. } => {
                if let Some(struct_info) = self.types.structs.get(name) {
                    struct_info
                        .fields
                        .iter()
                        .map(|(_name, ty)| self.compute_alignof(ty))
                        .max()
                        .unwrap_or(8)
                } else {
                    8 // enum or unknown named type
                }
            }
            _ => 8, // default for complex types
        }
    }
}

#[cfg(test)]
mod sizeof_alignof_tests {
    use super::*;
    use crate::CodeGenerator;
    use vais_types::ResolvedType;

    #[test]
    fn test_tuple_sizeof_sums_elements() {
        let gen = CodeGenerator::new("test");
        // (i8, i8) should be 2 bytes, not 16
        let tuple_type = ResolvedType::Tuple(vec![ResolvedType::I8, ResolvedType::I8]);
        assert_eq!(gen.compute_sizeof(&tuple_type), 2);
    }

    #[test]
    fn test_tuple_alignof_takes_max() {
        let gen = CodeGenerator::new("test");
        // (i8, i32) should have alignment of 4 (from i32)
        let tuple_type = ResolvedType::Tuple(vec![ResolvedType::I8, ResolvedType::I32]);
        assert_eq!(gen.compute_alignof(&tuple_type), 4);
    }

    #[test]
    fn test_struct_sizeof_sums_fields() {
        let mut gen = CodeGenerator::new("test");
        // Struct with two i8 fields
        gen.types.structs.insert(
            "Point2D".to_string(),
            StructInfo {
                _name: "Point2D".to_string(),
                fields: vec![
                    ("x".to_string(), ResolvedType::I8),
                    ("y".to_string(), ResolvedType::I8),
                ],
                _repr_c: false,
                _invariants: vec![],
            },
        );
        let struct_type = ResolvedType::Named {
            name: "Point2D".to_string(),
            generics: vec![],
        };
        assert_eq!(gen.compute_sizeof(&struct_type), 2);
    }

    #[test]
    fn test_struct_alignof_takes_max_field() {
        let mut gen = CodeGenerator::new("test");
        // Struct with i8 and i32 fields
        gen.types.structs.insert(
            "MixedStruct".to_string(),
            StructInfo {
                _name: "MixedStruct".to_string(),
                fields: vec![
                    ("a".to_string(), ResolvedType::I8),
                    ("b".to_string(), ResolvedType::I32),
                    ("c".to_string(), ResolvedType::I16),
                ],
                _repr_c: false,
                _invariants: vec![],
            },
        );
        let struct_type = ResolvedType::Named {
            name: "MixedStruct".to_string(),
            generics: vec![],
        };
        // Size: 1 + 4 + 2 = 7
        assert_eq!(gen.compute_sizeof(&struct_type), 7);
        // Alignment: max(1, 4, 2) = 4
        assert_eq!(gen.compute_alignof(&struct_type), 4);
    }

    #[test]
    fn test_primitive_types() {
        let gen = CodeGenerator::new("test");
        assert_eq!(gen.compute_sizeof(&ResolvedType::I8), 1);
        assert_eq!(gen.compute_alignof(&ResolvedType::I8), 1);
        assert_eq!(gen.compute_sizeof(&ResolvedType::I32), 4);
        assert_eq!(gen.compute_alignof(&ResolvedType::I32), 4);
        assert_eq!(gen.compute_sizeof(&ResolvedType::I64), 8);
        assert_eq!(gen.compute_alignof(&ResolvedType::I64), 8);
        assert_eq!(gen.compute_sizeof(&ResolvedType::I128), 16);
        assert_eq!(gen.compute_alignof(&ResolvedType::I128), 16);
    }
}
