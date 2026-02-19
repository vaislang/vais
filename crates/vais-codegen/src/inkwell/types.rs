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
}

impl<'ctx> TypeMapper<'ctx> {
    /// Creates a new type mapper with the given LLVM context.
    pub(crate) fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            struct_types: HashMap::new(),
            generic_substitutions: HashMap::new(),
        }
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

    /// Gets a registered struct type by name.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn get_struct(&self, name: &str) -> Option<StructType<'ctx>> {
        self.struct_types.get(name).copied()
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
            ResolvedType::Str => self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into(),
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
            ResolvedType::Pointer(inner)
            | ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner) => {
                // In LLVM 17+, pointers are opaque
                let _inner_llvm = self.map_type(inner);
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()
            }
            ResolvedType::Slice(_) | ResolvedType::SliceMut(_) => {
                // Slice is a fat pointer: { ptr: i8*, len: i64 }
                let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
                let len_type = self.context.i64_type();
                self.context
                    .struct_type(&[ptr_type.into(), len_type.into()], false)
                    .into()
            }
            ResolvedType::Named { name, .. } => {
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
                    // Monomorphization should resolve all generics — warn and fallback.
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "Warning: unresolved generic '{}' in Inkwell codegen, using i64 fallback",
                        name
                    );
                    self.context.i64_type().into()
                }
            }
            ResolvedType::Var(_) | ResolvedType::Unknown => {
                unreachable!("ICE: unresolved type variable reached Inkwell codegen")
            }
            ResolvedType::Never => {
                // Never type - use void pointer as placeholder
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()
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
            ResolvedType::Lazy(inner) => {
                // Lazy<T> = { i1 computed, T value, ptr thunk_fn }
                let inner_ty = self.map_type(inner);
                self.context
                    .struct_type(
                        &[
                            self.context.bool_type().into(), // computed flag
                            inner_ty,                        // cached value
                            self.context
                                .i8_type()
                                .ptr_type(inkwell::AddressSpace::default())
                                .into(), // thunk function pointer
                        ],
                        false,
                    )
                    .into()
            }
            ResolvedType::ConstGeneric(name) => {
                // Check if we have a substitution for this const generic parameter.
                if let Some(concrete) = self.generic_substitutions.get(name.as_str()).cloned() {
                    self.map_type(&concrete)
                } else {
                    // Monomorphization should resolve all const generics — warn and fallback.
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "Warning: unresolved const generic '{}' in Inkwell codegen, using i64 fallback",
                        name
                    );
                    self.context.i64_type().into()
                }
            }
            ResolvedType::Lifetime(_) => {
                unreachable!("ICE: bare lifetime has no runtime representation in Inkwell codegen")
            }
            ResolvedType::Associated { .. } => {
                unreachable!("ICE: unresolved associated type in Inkwell codegen")
            }
            ResolvedType::Dependent { base, .. } => {
                // Dependent types are transparent at runtime — use base type
                self.map_type(base)
            }
            ResolvedType::ImplTrait { .. } => {
                unreachable!("ICE: unresolved ImplTrait in Inkwell codegen")
            }
            ResolvedType::HigherKinded { .. } => {
                unreachable!("ICE: unresolved HKT in Inkwell codegen")
            }
        }
    }

    /// Gets the size of a type in bytes (approximate).
    #[allow(dead_code)]
    pub(crate) fn size_of(&self, ty: &ResolvedType) -> u64 {
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
            ResolvedType::Unit => 0,
            ResolvedType::ConstArray { element, size } => {
                let sz = match size {
                    vais_types::ResolvedConst::Value(n) => *n as u64,
                    _ => 1,
                };
                self.size_of(element) * sz
            }
            ResolvedType::Array(_) => 8, // pointer
            ResolvedType::Tuple(elems) => elems.iter().map(|e| self.size_of(e)).sum(),
            ResolvedType::Optional(inner) => 1 + self.size_of(inner),
            ResolvedType::Result(ok, err) => 1 + std::cmp::max(self.size_of(ok), self.size_of(err)),
            _ => 8, // Default for structs, enums, functions
        }
    }

    /// Gets the alignment of a type in bytes.
    #[allow(dead_code)]
    pub(crate) fn align_of(&self, ty: &ResolvedType) -> u64 {
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
            ResolvedType::Tuple(elems) => elems.iter().map(|e| self.align_of(e)).max().unwrap_or(8),
            ResolvedType::Optional(inner) => self.align_of(inner),
            ResolvedType::Result(ok, err) => std::cmp::max(self.align_of(ok), self.align_of(err)),
            _ => 8, // Default for structs, enums, functions
        }
    }
}
