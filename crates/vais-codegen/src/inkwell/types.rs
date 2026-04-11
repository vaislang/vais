//! Type mapping between Vais types and LLVM types.

use inkwell::context::Context;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, StructType};
use inkwell::AddressSpace;
use std::collections::HashMap;
use vais_types::ResolvedType;

/// Maps Vais types to LLVM types using inkwell.
pub(crate) struct TypeMapper<'ctx> {
    context: &'ctx Context,
    struct_types: HashMap<String, StructType<'ctx>>,
    /// Generic substitutions mirrored from InkwellCodeGenerator.
    /// Updated via `set_generic_substitutions` / `clear_generic_substitutions`.
    pub(crate) generic_substitutions: HashMap<String, ResolvedType>,
    /// Structured warnings collected during type mapping.
    /// Uses RefCell for interior mutability (map_type takes &self).
    pub(crate) warnings: std::cell::RefCell<Vec<crate::CodegenWarning>>,
    /// When true, ICE-level fallbacks become errors instead of warnings.
    /// Set from `InkwellCodeGenerator::set_strict_type_mode()`.
    pub(crate) strict_type_mode: bool,
    /// Deferred error from `map_type` when strict mode is enabled.
    /// Since `map_type` returns `BasicTypeEnum` (not `Result`), errors from
    /// ICE-level fallbacks are stored here for the caller to check via
    /// `take_pending_error()` after calling `map_type`.
    pub(crate) pending_error: std::cell::RefCell<Option<crate::CodegenError>>,
}

impl<'ctx> TypeMapper<'ctx> {
    /// Creates a new type mapper with the given LLVM context.
    pub(crate) fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            struct_types: HashMap::new(),
            generic_substitutions: HashMap::new(),
            warnings: std::cell::RefCell::new(Vec::new()),
            strict_type_mode: true,
            pending_error: std::cell::RefCell::new(None),
        }
    }

    /// Record a structured codegen warning.
    fn emit_warning(&self, warning: crate::CodegenWarning) {
        self.warnings.borrow_mut().push(warning);
    }

    /// Emit a warning, or store a deferred error in strict type mode for ICE-level fallbacks.
    ///
    /// Since `map_type` returns `BasicTypeEnum` (not `Result`), errors from strict mode
    /// are stored in `pending_error` for the caller to check via `take_pending_error()`.
    /// The i64 fallback value is still returned so `map_type` can complete, but the
    /// caller should check for and propagate the error before using the result.
    fn emit_warning_or_error(&self, warning: crate::CodegenWarning) {
        if self.strict_type_mode {
            if let crate::CodegenWarning::UnresolvedTypeFallback {
                ref type_desc,
                ref backend,
            } = warning
            {
                // Store the first error (don't overwrite if one is already pending)
                let mut pending = self.pending_error.borrow_mut();
                if pending.is_none() {
                    *pending = Some(crate::CodegenError::InternalError(format!(
                        "[strict] {} in {} codegen — i64 fallback disabled",
                        type_desc, backend
                    )));
                }
                return;
            }
        }
        self.emit_warning(warning);
    }

    /// Take the pending error (if any) from a previous `map_type` call.
    /// Returns `None` if no error occurred.
    pub(crate) fn take_pending_error(&self) -> Option<crate::CodegenError> {
        self.pending_error.borrow_mut().take()
    }

    /// Drain all collected warnings (transfers ownership to caller).
    pub(crate) fn take_warnings(&self) -> Vec<crate::CodegenWarning> {
        std::mem::take(&mut *self.warnings.borrow_mut())
    }

    /// Synchronise the substitution table with the generator's current map.
    /// Skips clone when the source map is empty (common case after clear).
    pub(crate) fn set_generic_substitutions(&mut self, subst: &HashMap<String, ResolvedType>) {
        if subst.is_empty() {
            self.generic_substitutions.clear();
        } else {
            self.generic_substitutions = subst.clone();
        }
    }

    /// Clear the substitution table (call when leaving a generic context).
    pub(crate) fn clear_generic_substitutions(&mut self) {
        self.generic_substitutions.clear();
    }

    /// Registers a named struct type.
    pub(crate) fn register_struct(&mut self, name: &str, struct_type: StructType<'ctx>) {
        self.struct_types.insert(name.to_string(), struct_type);
    }

    /// Maps a Vais resolved type to an LLVM basic type.
    pub(crate) fn map_type(&self, ty: &ResolvedType) -> BasicTypeEnum<'ctx> {
        match ty {
            ResolvedType::I8 => self.context.i8_type().into(),
            ResolvedType::I16 => self.context.i16_type().into(),
            ResolvedType::I32 => self.context.i32_type().into(),
            ResolvedType::I64 => self.context.i64_type().into(),
            ResolvedType::I128 => self.context.i128_type().into(),
            ResolvedType::U8 => self.context.i8_type().into(),
            ResolvedType::U16 => self.context.i16_type().into(),
            ResolvedType::U32 => self.context.i32_type().into(),
            ResolvedType::U64 => self.context.i64_type().into(),
            ResolvedType::U128 => self.context.i128_type().into(),
            ResolvedType::F32 => self.context.f32_type().into(),
            ResolvedType::F64 => self.context.f64_type().into(),
            ResolvedType::Bool => self.context.bool_type().into(),
            ResolvedType::Str => {
                // String is a fat pointer: { i8* ptr, i64 len }
                let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
                let len_type = self.context.i64_type();
                self.context
                    .struct_type(&[ptr_type.into(), len_type.into()], false)
                    .into()
            }
            ResolvedType::Unit => {
                // Unit type represented as empty struct
                self.context.struct_type(&[], false).into()
            }
            ResolvedType::Array(elem_ty) => {
                // Dynamic array is a pointer
                let _elem_llvm = self.map_type(elem_ty);
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()
            }
            ResolvedType::ConstArray { element, size } => {
                let elem_llvm = self.map_type(element);
                let sz = match size {
                    vais_types::ResolvedConst::Value(n) => *n as u32,
                    _ => 1,
                };
                elem_llvm.array_type(sz).into()
            }
            ResolvedType::Pointer(inner) => {
                // In LLVM 17+, pointers are opaque
                let _inner_llvm = self.map_type(inner);
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()
            }
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                // &[T] and &mut [T] are fat pointers (same as Slice/SliceMut)
                // — a slice reference IS a fat pointer, not a pointer to one
                match inner.as_ref() {
                    ResolvedType::Slice(_) | ResolvedType::SliceMut(_) => {
                        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
                        let len_type = self.context.i64_type();
                        self.context
                            .struct_type(&[ptr_type.into(), len_type.into()], false)
                            .into()
                    }
                    _ => {
                        // In LLVM 17+, pointers are opaque
                        let _inner_llvm = self.map_type(inner);
                        self.context
                            .i8_type()
                            .ptr_type(AddressSpace::default())
                            .into()
                    }
                }
            }
            ResolvedType::Slice(_) | ResolvedType::SliceMut(_) => {
                // Slice is a fat pointer: { ptr: i8*, len: i64 }
                let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
                let len_type = self.context.i64_type();
                self.context
                    .struct_type(&[ptr_type.into(), len_type.into()], false)
                    .into()
            }
            ResolvedType::Named { name, generics } => {
                // If generics are present and all concrete, try mangled name first (e.g., "Vec$f32")
                if !generics.is_empty() {
                    let all_concrete = generics
                        .iter()
                        .all(|g| !matches!(g, ResolvedType::Generic(_) | ResolvedType::Var(_)));
                    if all_concrete {
                        let mangled = vais_types::mangle_name(name, generics);
                        if let Some(st) = self.struct_types.get(mangled.as_str()) {
                            return (*st).into();
                        }
                    }
                }
                if let Some(st) = self.struct_types.get(name.as_str()) {
                    (*st).into()
                } else {
                    // Return opaque struct placeholder
                    self.context.opaque_struct_type(name).into()
                }
            }
            ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
                // Function type as pointer
                let _param_types: Vec<BasicMetadataTypeEnum> =
                    params.iter().map(|p| self.map_type(p).into()).collect();
                let _ret_type = self.map_type(ret);
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()
            }
            ResolvedType::Generic(name) => {
                // Check if we have a substitution for this generic parameter.
                if let Some(concrete) = self.generic_substitutions.get(name.as_str()).cloned() {
                    self.map_type(&concrete)
                } else {
                    // Generic parameter without substitution — use i64 fallback.
                    // With transitive instantiation (Phase 67), this path is now mostly
                    // reached only for un-specialized fallback versions of generic functions.
                    self.emit_warning(crate::CodegenWarning::GenericFallback {
                        param: name.clone(),
                        context: String::from("<inkwell>"),
                    });
                    self.context.i64_type().into()
                }
            }
            ResolvedType::Var(_) | ResolvedType::Unknown => {
                self.emit_warning_or_error(crate::CodegenWarning::UnresolvedTypeFallback {
                    type_desc: String::from("unresolved type variable"),
                    backend: String::from("inkwell"),
                });
                self.context.i64_type().into()
            }
            ResolvedType::Never => {
                // Never type — use empty struct as placeholder (consistent with void in Text IR).
                // Never values should never exist at runtime; this is purely a type-level placeholder.
                self.context.struct_type(&[], false).into()
            }
            ResolvedType::Tuple(elems) => {
                let elem_types: Vec<BasicTypeEnum> =
                    elems.iter().map(|e| self.map_type(e)).collect();
                self.context.struct_type(&elem_types, false).into()
            }
            ResolvedType::Optional(inner) => {
                // Option<T> is { tag: i8, value: T }
                let inner_llvm = self.map_type(inner);
                let tag_type = self.context.i8_type();
                self.context
                    .struct_type(&[tag_type.into(), inner_llvm], false)
                    .into()
            }
            ResolvedType::Result(ok, err) => {
                // Result<T, E> is { tag: i8, value: max(T, E) }
                let ok_llvm = self.map_type(ok);
                let _err_llvm = self.map_type(err);
                let tag_type = self.context.i8_type();
                // Use ok type as payload (largest variant strategy handled by enum layout)
                self.context
                    .struct_type(&[tag_type.into(), ok_llvm], false)
                    .into()
            }
            ResolvedType::Map(key, value) => {
                // Map is a pointer to runtime structure
                let _key_llvm = self.map_type(key);
                let _val_llvm = self.map_type(value);
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()
            }
            ResolvedType::Range(_) => {
                // Range is { start: i64, end: i64, inclusive: i1 }
                let i64_type = self.context.i64_type();
                let bool_type = self.context.bool_type();
                self.context
                    .struct_type(&[i64_type.into(), i64_type.into(), bool_type.into()], false)
                    .into()
            }
            ResolvedType::Future(inner) => {
                let _inner_llvm = self.map_type(inner);
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()
            }
            ResolvedType::DynTrait { .. } => {
                // Fat pointer: { data_ptr, vtable_ptr }
                let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
                self.context
                    .struct_type(&[ptr_type.into(), ptr_type.into()], false)
                    .into()
            }
            ResolvedType::Vector { element, lanes } => {
                let elem_llvm = self.map_type(element);
                match elem_llvm {
                    BasicTypeEnum::IntType(t) => t.vec_type(*lanes).into(),
                    BasicTypeEnum::FloatType(t) => t.vec_type(*lanes).into(),
                    _ => self.context.i64_type().vec_type(*lanes).into(),
                }
            }
            ResolvedType::RefLifetime { inner, .. }
            | ResolvedType::RefMutLifetime { inner, .. } => {
                let _inner_llvm = self.map_type(inner);
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()
            }
            ResolvedType::Linear(inner) | ResolvedType::Affine(inner) => {
                // Transparent wrappers at runtime
                self.map_type(inner)
            }
            ResolvedType::ConstGeneric(name) => {
                // Check if we have a substitution for this const generic parameter.
                if let Some(concrete) = self.generic_substitutions.get(name.as_str()).cloned() {
                    self.map_type(&concrete)
                } else {
                    // ConstGeneric parameter without substitution — use i64 fallback.
                    // Same rationale as Generic above: kept for backward-compatible fallback.
                    self.emit_warning(crate::CodegenWarning::GenericFallback {
                        param: name.clone(),
                        context: String::from("<inkwell>"),
                    });
                    self.context.i64_type().into()
                }
            }
            ResolvedType::Lifetime(_) => {
                self.emit_warning_or_error(crate::CodegenWarning::UnresolvedTypeFallback {
                    type_desc: String::from("bare lifetime"),
                    backend: String::from("inkwell"),
                });
                self.context.i64_type().into()
            }
            ResolvedType::Associated {
                ref base,
                ref assoc_name,
                ..
            } => {
                self.emit_warning_or_error(crate::CodegenWarning::UnresolvedTypeFallback {
                    type_desc: format!("unresolved associated type `{}` on {:?}", assoc_name, base),
                    backend: String::from("inkwell"),
                });
                self.context.i64_type().into()
            }
            ResolvedType::Dependent { base, .. } => {
                // Dependent types are transparent at runtime — use base type
                self.map_type(base)
            }
            ResolvedType::ImplTrait { .. } => {
                self.emit_warning_or_error(crate::CodegenWarning::UnresolvedTypeFallback {
                    type_desc: String::from("unresolved ImplTrait"),
                    backend: String::from("inkwell"),
                });
                self.context.i64_type().into()
            }
            ResolvedType::HigherKinded { .. } => {
                self.emit_warning_or_error(crate::CodegenWarning::UnresolvedTypeFallback {
                    type_desc: String::from("unresolved HKT"),
                    backend: String::from("inkwell"),
                });
                self.context.i64_type().into()
            }
        }
    }
}
