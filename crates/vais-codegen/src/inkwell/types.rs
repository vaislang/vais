//! Type mapping between Vais types and LLVM types.

use inkwell::context::Context;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, StructType};
use inkwell::AddressSpace;
use std::collections::HashMap;
use vais_types::ResolvedType;

/// Maps Vais types to LLVM types using inkwell.
pub struct TypeMapper<'ctx> {
    context: &'ctx Context,
    struct_types: HashMap<String, StructType<'ctx>>,
}

impl<'ctx> TypeMapper<'ctx> {
    /// Creates a new type mapper with the given LLVM context.
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            struct_types: HashMap::new(),
        }
    }

    /// Registers a named struct type.
    pub fn register_struct(&mut self, name: &str, struct_type: StructType<'ctx>) {
        self.struct_types.insert(name.to_string(), struct_type);
    }

    /// Gets a registered struct type by name.
    pub fn get_struct(&self, name: &str) -> Option<StructType<'ctx>> {
        self.struct_types.get(name).copied()
    }

    /// Maps a Vais resolved type to an LLVM basic type.
    pub fn map_type(&self, ty: &ResolvedType) -> BasicTypeEnum<'ctx> {
        match ty {
            ResolvedType::I8 => self.context.i8_type().into(),
            ResolvedType::I16 => self.context.i16_type().into(),
            ResolvedType::I32 => self.context.i32_type().into(),
            ResolvedType::I64 => self.context.i64_type().into(),
            ResolvedType::U8 => self.context.i8_type().into(),
            ResolvedType::U16 => self.context.i16_type().into(),
            ResolvedType::U32 => self.context.i32_type().into(),
            ResolvedType::U64 => self.context.i64_type().into(),
            ResolvedType::F32 => self.context.f32_type().into(),
            ResolvedType::F64 => self.context.f64_type().into(),
            ResolvedType::Bool => self.context.bool_type().into(),
            ResolvedType::Char => self.context.i32_type().into(), // Unicode code point
            ResolvedType::Str => self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into(),
            ResolvedType::Unit => {
                // Unit type represented as empty struct
                self.context.struct_type(&[], false).into()
            }
            ResolvedType::Array(elem_ty, size) => {
                let elem_llvm = self.map_type(elem_ty);
                elem_llvm.array_type(*size as u32).into()
            }
            ResolvedType::Ptr(inner) => {
                let _inner_llvm = self.map_type(inner);
                // In LLVM 17+, pointers are opaque
                self.context
                    .ptr_type(AddressSpace::default())
                    .into()
            }
            ResolvedType::Struct(name, _) => {
                if let Some(st) = self.struct_types.get(name) {
                    (*st).into()
                } else {
                    // Return opaque struct placeholder
                    self.context.opaque_struct_type(name).into()
                }
            }
            ResolvedType::Enum(name, _) => {
                // Enums are represented as tagged unions
                // For now, use i64 for tag + pointer for data
                if let Some(st) = self.struct_types.get(name) {
                    (*st).into()
                } else {
                    self.context.opaque_struct_type(name).into()
                }
            }
            ResolvedType::Function(params, ret) => {
                // Function type as pointer
                let param_types: Vec<BasicMetadataTypeEnum> = params
                    .iter()
                    .map(|p| self.map_type(p).into())
                    .collect();
                let ret_type = self.map_type(ret);
                let fn_type = ret_type.fn_type(&param_types, false);
                fn_type.ptr_type(AddressSpace::default()).into()
            }
            ResolvedType::Generic(name) => {
                // Generic types should be substituted before codegen
                panic!("Unsubstituted generic type: {}", name);
            }
            ResolvedType::Infer => {
                // Should be resolved before codegen
                panic!("Unresolved infer type in codegen");
            }
            ResolvedType::Never => {
                // Never type - use void pointer as placeholder
                self.context
                    .ptr_type(AddressSpace::default())
                    .into()
            }
            ResolvedType::Tuple(elems) => {
                let elem_types: Vec<BasicTypeEnum> =
                    elems.iter().map(|e| self.map_type(e)).collect();
                self.context.struct_type(&elem_types, false).into()
            }
            ResolvedType::Slice(elem_ty) => {
                // Slice is a fat pointer: { *T, len }
                let _elem_llvm = self.map_type(elem_ty);
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let len_type = self.context.i64_type();
                self.context
                    .struct_type(&[ptr_type.into(), len_type.into()], false)
                    .into()
            }
            ResolvedType::Option(inner) => {
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
                let err_llvm = self.map_type(err);
                let tag_type = self.context.i8_type();
                // Use pointer to hold either value
                let value_type = self.context.ptr_type(AddressSpace::default());
                self.context
                    .struct_type(&[tag_type.into(), value_type.into()], false)
                    .into()
            }
        }
    }

    /// Gets the size of a type in bytes (approximate).
    pub fn size_of(&self, ty: &ResolvedType) -> u64 {
        match ty {
            ResolvedType::I8 | ResolvedType::U8 | ResolvedType::Bool => 1,
            ResolvedType::I16 | ResolvedType::U16 => 2,
            ResolvedType::I32 | ResolvedType::U32 | ResolvedType::Char | ResolvedType::F32 => 4,
            ResolvedType::I64 | ResolvedType::U64 | ResolvedType::F64 => 8,
            ResolvedType::Str | ResolvedType::Ptr(_) => 8, // 64-bit pointer
            ResolvedType::Unit => 0,
            ResolvedType::Array(elem, size) => self.size_of(elem) * (*size as u64),
            ResolvedType::Tuple(elems) => elems.iter().map(|e| self.size_of(e)).sum(),
            ResolvedType::Slice(_) => 16, // ptr + len
            ResolvedType::Option(inner) => 1 + self.size_of(inner),
            ResolvedType::Result(ok, err) => 1 + self.size_of(ok).max(self.size_of(err)),
            _ => 8, // Default for structs, enums, functions
        }
    }
}
