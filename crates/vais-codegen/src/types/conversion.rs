//! Type-to-LLVM conversion methods for CodeGenerator

use super::*;
use crate::CodeGenerator;

impl CodeGenerator {
    /// Convert a ResolvedType to LLVM IR type string with caching.
    ///
    /// Uses a 2-tier approach:
    /// - Primitive types: immediate return (no cache interaction)
    /// - Complex types: hash-based cache with u64 key (avoids String allocation on cache hit)
    pub(crate) fn type_to_llvm(&self, ty: &ResolvedType) -> String {
        // Ultra-fast path: return static strings for primitive types
        // Avoids hash computation and cache lookup entirely
        match ty {
            ResolvedType::I8 => return String::from("i8"),
            ResolvedType::I16 => return String::from("i16"),
            ResolvedType::I32 => return String::from("i32"),
            ResolvedType::I64 => return String::from("i64"),
            ResolvedType::I128 => return String::from("i128"),
            ResolvedType::U8 => return String::from("i8"),
            ResolvedType::U16 => return String::from("i16"),
            ResolvedType::U32 => return String::from("i32"),
            ResolvedType::U64 => return String::from("i64"),
            ResolvedType::U128 => return String::from("i128"),
            ResolvedType::F32 => return String::from("float"),
            ResolvedType::F64 => return String::from("double"),
            ResolvedType::Bool => return String::from("i1"),
            ResolvedType::Str => return String::from("{ i8*, i64 }"),
            ResolvedType::Unit => return String::from("void"),
            _ => {}
        }

        // Semi-fast path: Named types without generics (common for struct types)
        // Use the name directly as cache key — avoids expensive Debug format
        if let ResolvedType::Named { name, generics } = ty {
            if generics.is_empty() {
                let cache_key = name.clone();
                if let Some(cached) = self.type_to_llvm_cache.borrow().get(&cache_key) {
                    return cached.clone();
                }
                let result = match self.type_to_llvm_impl(ty) {
                    Ok(r) => r,
                    Err(e) => {
                        self.emit_warning(crate::CodegenWarning::UnresolvedTypeFallback {
                            type_desc: e.to_string(),
                            backend: String::from("text"),
                        });
                        String::from("i64")
                    }
                };
                self.type_to_llvm_cache
                    .borrow_mut()
                    .insert(cache_key, result.clone());
                return result;
            }
        }

        // Complex types: use cache with Debug key
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
                self.emit_warning(crate::CodegenWarning::UnresolvedTypeFallback {
                    type_desc: e.to_string(),
                    backend: String::from("text"),
                });
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
        use std::sync::atomic::{AtomicUsize, Ordering};
        static TTL_COUNT: AtomicUsize = AtomicUsize::new(0);
        let _c = TTL_COUNT.fetch_add(1, Ordering::Relaxed);
        stacker::maybe_grow(32 * 1024 * 1024, 64 * 1024 * 1024, || {
            self.enter_type_recursion("type_to_llvm")?;
            let result = self.type_to_llvm_impl_inner(ty);
            self.exit_type_recursion();
            result
        })
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
            ResolvedType::Str => String::from("{ i8*, i64 }"),
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
            ResolvedType::RefMut(inner) => match inner.as_ref() {
                ResolvedType::DynTrait { .. }
                | ResolvedType::Slice(_)
                | ResolvedType::SliceMut(_) => self.type_to_llvm_impl(inner)?,
                _ => format!("{}*", self.type_to_llvm_impl(inner)?),
            },
            ResolvedType::Range(_inner) => {
                // Range is represented as a struct: { i64 start, i64 end, i1 inclusive }
                String::from("{ i64, i64, i1 }")
            }
            ResolvedType::Named { name, generics } => {
                // Single uppercase letter is likely a generic type parameter
                if name.len() == 1 && name.chars().next().is_some_and(|c| c.is_uppercase()) {
                    if let Some(concrete) = self.get_generic_substitution(name) {
                        self.type_to_llvm(&concrete)
                    } else {
                        String::from("i64")
                    }
                } else if !generics.is_empty() {
                    // Check if all generics are concrete (not Generic or Var types)
                    let all_concrete = generics
                        .iter()
                        .all(|g| !matches!(g, ResolvedType::Generic(_) | ResolvedType::Var(_)));

                    if all_concrete {
                        // Use mangled name for concrete instantiations.
                        // Check if a specialized struct type was generated (e.g., Vec$MyStruct).
                        let mangled = self.mangle_struct_name(name, generics);
                        if self.types.structs.contains_key(&mangled)
                            || self.generics.generated_structs.contains_key(&mangled)
                        {
                            format!("%{}", mangled)
                        } else if self.types.enums.contains_key(name)
                            || self.types.structs.contains_key(name)
                            || self.types.unions.contains_key(name)
                        {
                            // Base type exists but no specialization — use base name.
                            // This is correct for enums/unions where layout is type-independent.
                            format!("%{}", name)
                        } else {
                            // External or not-yet-generated specialization
                            format!("%{}", mangled)
                        }
                    } else {
                        // For generic types with unresolved parameters, use base struct name.
                        // Layout is i64-uniform when type args can't be resolved.
                        format!("%{}", name)
                    }
                } else if let Some(subst) = self.get_generic_substitution(name) {
                    // Named type that has a substitution (e.g., "Self" → Option)
                    self.type_to_llvm(&subst)
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
                    //
                    // With transitive instantiation (Phase 67), this path is now mostly reached
                    // only for un-specialized fallback versions of generic functions — i.e., when
                    // generate_module_with_instantiations generates a backward-compatible i64 version
                    // of a generic function that has no concrete instantiation.
                    //
                    // NOTE: returning Err here would break nested types like &T → i64 instead of i64*,
                    // because the error short-circuits the wrapper type conversion.
                    let context = self
                        .fn_ctx
                        .current_function
                        .as_deref()
                        .unwrap_or("<unknown>")
                        .to_string();
                    self.emit_warning(crate::CodegenWarning::GenericFallback {
                        param: param.clone(),
                        context,
                    });
                    String::from("i64")
                }
            }
            ResolvedType::ConstGeneric(param) => {
                // Check if we have a substitution for this const generic parameter
                if let Some(concrete) = self.get_generic_substitution(param) {
                    self.type_to_llvm_impl(&concrete)?
                } else {
                    // ConstGeneric parameter without substitution — use i64 fallback.
                    // Same rationale as Generic above: kept for backward-compatible fallback.
                    let context = self
                        .fn_ctx
                        .current_function
                        .as_deref()
                        .unwrap_or("<unknown>")
                        .to_string();
                    self.emit_warning(crate::CodegenWarning::GenericFallback {
                        param: param.clone(),
                        context,
                    });
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
                self.emit_warning_or_error(crate::CodegenWarning::UnresolvedTypeFallback {
                    type_desc: String::from("unresolved ImplTrait"),
                    backend: String::from("text"),
                })?;
                String::from("i64")
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
                self.emit_warning_or_error(crate::CodegenWarning::UnresolvedTypeFallback {
                    type_desc: String::from("bare lifetime"),
                    backend: String::from("text"),
                })?;
                String::from("i64")
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
                // Future is an opaque pointer to async state machine.
                // Represented as i64 in Text IR (pointer-as-integer convention).
                // NOTE: Inkwell backend uses i8* (actual pointer). The backends differ
                // because Text IR spawn codegen stores the future state in an i64-sized
                // malloc'd struct `{i64 state, i64 result}`, where state is an integer tag.
                String::from("i64")
            }
            ResolvedType::Never => {
                // Never type — functions that return ! use void
                String::from("void")
            }
            ResolvedType::Var(_) | ResolvedType::Unknown => {
                self.emit_warning_or_error(crate::CodegenWarning::UnresolvedTypeFallback {
                    type_desc: String::from("unresolved type variable"),
                    backend: String::from("text"),
                })?;
                String::from("i64")
            }
            ResolvedType::Associated {
                base,
                trait_name,
                assoc_name,
                ..
            } => {
                // Try to resolve the associated type by looking up trait impls
                if let Some(resolved) =
                    self.resolve_associated_type_in_codegen(base, trait_name.as_deref(), assoc_name)
                {
                    return self.type_to_llvm_impl(&resolved);
                }
                // Fallback: associated type that couldn't be resolved (generic base)
                let base_desc = format!("{:?}", base);
                self.emit_warning(crate::CodegenWarning::AssociatedTypeFallback {
                    assoc_name: assoc_name.clone(),
                    base_type: base_desc,
                });
                String::from("i64")
            }
            ResolvedType::HigherKinded { .. } => {
                self.emit_warning_or_error(crate::CodegenWarning::UnresolvedTypeFallback {
                    type_desc: String::from("unresolved HKT"),
                    backend: String::from("text"),
                })?;
                String::from("i64")
            }
        })
    }

    /// Try to resolve an associated type by looking up trait definitions and
    /// implementations registered in codegen state. Returns None if unresolvable.
    pub(crate) fn resolve_associated_type_in_codegen(
        &self,
        base: &ResolvedType,
        trait_name: Option<&str>,
        assoc_name: &str,
    ) -> Option<ResolvedType> {
        // Extract base type name
        let type_name = match base {
            ResolvedType::Named { name, .. } => name.as_str(),
            _ => return None, // Can't resolve for generic/unknown base
        };

        // Look for trait impl that maps this type + trait to associated types
        // Check all registered trait defs to find one with the associated type
        if let Some(tn) = trait_name {
            if let Some(trait_def) = self.types.trait_defs.get(tn) {
                // Check for a default associated type value
                if let Some(assoc_def) = trait_def.associated_types.get(assoc_name) {
                    if let Some(default) = &assoc_def.default {
                        return Some(default.clone());
                    }
                }
            }
        } else {
            // No trait name specified — search all trait impls for this type
            for (impl_type_key, tn) in self.types.trait_impl_methods.keys() {
                if impl_type_key != type_name {
                    continue;
                }
                if let Some(trait_def) = self.types.trait_defs.get(tn.as_str()) {
                    if let Some(assoc_def) = trait_def.associated_types.get(assoc_name) {
                        if let Some(default) = &assoc_def.default {
                            return Some(default.clone());
                        }
                    }
                }
            }
        }

        // Also try resolved function sigs — the TC may have resolved associated types
        // into concrete types stored in the resolved sigs
        None
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
    #[allow(dead_code)]
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
            self.emit_warning(crate::CodegenWarning::UnresolvedTypeFallback {
                type_desc: String::from("type recursion limit exceeded in ast_type_to_resolved"),
                backend: String::from("text"),
            });
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
            Type::Linear(inner) => {
                ResolvedType::Linear(Box::new(self.ast_type_to_resolved_impl(&inner.node)))
            }
            Type::Affine(inner) => {
                ResolvedType::Affine(Box::new(self.ast_type_to_resolved_impl(&inner.node)))
            }
            Type::Dependent {
                var_name,
                base,
                predicate,
            } => {
                let resolved_base = self.ast_type_to_resolved_impl(&base.node);
                // Store predicate as Debug string (consistent with type checker)
                let predicate_str = format!("{:?}", predicate.node);
                ResolvedType::Dependent {
                    var_name: var_name.clone(),
                    base: Box::new(resolved_base),
                    predicate: predicate_str,
                }
            }
            _ => ResolvedType::Unknown,
        }
    }
}
