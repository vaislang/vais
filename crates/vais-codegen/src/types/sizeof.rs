//! sizeof and alignof computations for Vais types

use super::*;
use crate::CodeGenerator;

impl CodeGenerator {
    /// Estimate the size of an LLVM type (for union layout)
    pub(crate) fn estimate_type_size(&self, llvm_type: &str) -> usize {
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
            s if s.starts_with('{') => {
                // Struct type: { T1, T2, ... }
                // Sum the sizes of the fields
                let inner = s
                    .strip_prefix("{ ")
                    .and_then(|s| s.strip_suffix(" }"))
                    .unwrap_or(s);
                inner
                    .split(", ")
                    .map(|field| self.estimate_type_size(field.trim()))
                    .sum()
            }
            s if s.starts_with('%') => {
                // Try to look up the named struct in our type registry
                let type_name = &s[1..]; // strip '%' prefix
                if let Some(struct_info) = self.types.structs.get(type_name) {
                    struct_info
                        .fields
                        .iter()
                        .map(|(_name, ty)| self.compute_sizeof(ty) as usize)
                        .sum()
                } else {
                    8 // Unknown named type — default to i64 size
                }
            }
            _ => 8, // Default fallback
        }
    }

    /// Check if a type recursively contains a specific generic parameter
    fn type_contains_generic(ty: &ResolvedType, param: &str) -> bool {
        match ty {
            ResolvedType::Generic(p) => p == param,
            ResolvedType::Named { generics, .. } => generics
                .iter()
                .any(|g| Self::type_contains_generic(g, param)),
            ResolvedType::Optional(inner)
            | ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Slice(inner)
            | ResolvedType::SliceMut(inner) => Self::type_contains_generic(inner, param),
            ResolvedType::Result(ok, err) => {
                Self::type_contains_generic(ok, param) || Self::type_contains_generic(err, param)
            }
            ResolvedType::Tuple(elems) => {
                elems.iter().any(|e| Self::type_contains_generic(e, param))
            }
            _ => false,
        }
    }

    /// Compute sizeof for a ResolvedType (in bytes)
    /// Returns the size in Vais's runtime representation
    pub(crate) fn compute_sizeof(&self, ty: &ResolvedType) -> i64 {
        // For Named types, check and track visited to prevent infinite recursion
        // from circular struct references (e.g., A contains B, B contains A)
        if let ResolvedType::Named { name, generics } = ty {
            let visit_key = if generics.is_empty() {
                name.clone()
            } else {
                self.mangle_struct_name(name, generics)
            };
            if self.sizeof_visited.borrow().contains(&visit_key) {
                return 8; // Break circular reference with pointer-size default
            }
            self.sizeof_visited.borrow_mut().insert(visit_key.clone());
            let result = stacker::maybe_grow(32 * 1024 * 1024, 64 * 1024 * 1024, || {
                self.compute_sizeof_inner(ty)
            });
            self.sizeof_visited.borrow_mut().remove(&visit_key);
            return result;
        }
        stacker::maybe_grow(32 * 1024 * 1024, 64 * 1024 * 1024, || {
            self.compute_sizeof_inner(ty)
        })
    }

    fn compute_sizeof_inner(&self, ty: &ResolvedType) -> i64 {
        match ty {
            ResolvedType::I8 | ResolvedType::U8 | ResolvedType::Bool => 1,
            ResolvedType::I16 | ResolvedType::U16 => 2,
            ResolvedType::I32 | ResolvedType::U32 | ResolvedType::F32 => 4,
            ResolvedType::I64 | ResolvedType::U64 | ResolvedType::F64 => 8,
            ResolvedType::I128 | ResolvedType::U128 => 16,
            ResolvedType::Str => 16, // fat pointer { i8*, i64 }
            ResolvedType::Unit => 0,
            ResolvedType::Pointer(_) => 8,
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => match inner.as_ref() {
                ResolvedType::Str
                | ResolvedType::Slice(_)
                | ResolvedType::SliceMut(_)
                | ResolvedType::DynTrait { .. } => self.compute_sizeof(inner),
                _ => 8,
            },
            ResolvedType::Slice(_) | ResolvedType::SliceMut(_) => 16, // fat pointer { i8*, i64 }
            ResolvedType::DynTrait { .. } => 16, // fat pointer { data, vtable }
            ResolvedType::Array(_) => 8,         // pointer to heap
            ResolvedType::Optional(inner) => {
                // Option<T> is { i8 tag, T value } — actual size depends on T
                let inner_size = self.compute_sizeof(inner);
                1 + inner_size // i8 tag + payload
            }
            ResolvedType::Result(ok, err) => {
                // Result<T, E> is { i8 tag, max(T, E) value }
                let ok_size = self.compute_sizeof(ok);
                let err_size = self.compute_sizeof(err);
                1 + std::cmp::max(ok_size, err_size) // i8 tag + largest payload
            }
            ResolvedType::Tuple(elems) => self.compute_aggregate_size(elems),
            ResolvedType::Named { name, generics } => {
                // Visited check is done in compute_sizeof() wrapper

                // First try the base name (non-generic or already resolved struct)
                if let Some(struct_info) = self.types.structs.get(name) {
                    return self.compute_struct_size(struct_info);
                }
                // For generic Named types, try the mangled (specialized) name
                if !generics.is_empty() {
                    let all_concrete = generics
                        .iter()
                        .all(|g| !matches!(g, ResolvedType::Generic(_) | ResolvedType::Var(_)));
                    if all_concrete {
                        let mangled = self.mangle_struct_name(name, generics);
                        if let Some(struct_info) = self.types.structs.get(&mangled) {
                            return self.compute_struct_size(struct_info);
                        }
                    }
                }
                // Try enums by their canonical name: { i32 tag, { i64 × N } }
                if let Some(enum_info) = self.types.enums.get(name) {
                    return Self::compute_enum_sizeof_from_info(enum_info);
                }
                // Try enum variant names: e.g. "FloatVal" is a variant of "SqlValue".
                // A value of variant type must occupy the full parent enum size (tagged union).
                {
                    let parent_enum = self
                        .types
                        .enums
                        .values()
                        .find(|ei| ei.variants.iter().any(|v| &v.name == name));
                    if let Some(enum_info) = parent_enum {
                        return Self::compute_enum_sizeof_from_info(enum_info);
                    }
                }
                // Try with generic-mangled names in generated_structs
                if !generics.is_empty() {
                    let mangled = self.mangle_struct_name(name, generics);
                    if self.generics.generated_structs.contains_key(&mangled) {
                        // Generated but not in types.structs — compute from fields
                        // with proper substitution of generic parameters
                        if let Some(struct_def) = self.generics.struct_defs.get(name) {
                            // Build substitution map: generic param -> concrete type arg
                            let subst: std::collections::HashMap<String, ResolvedType> = struct_def
                                .generics
                                .iter()
                                .filter(|g| {
                                    !matches!(g.kind, vais_ast::GenericParamKind::Lifetime { .. })
                                })
                                .zip(generics.iter())
                                .map(|(g, t)| (g.name.node.clone(), t.clone()))
                                .collect();
                            let field_types: Vec<_> = struct_def
                                .fields
                                .iter()
                                .map(|f| {
                                    let field_ty = self.ast_type_to_resolved(&f.ty.node);
                                    vais_types::substitute_type(&field_ty, &subst)
                                })
                                .collect();
                            return self.compute_aggregate_size(&field_types);
                        }
                    }
                }
                // Try type aliases: Named type might be an alias for a known type
                if let Some(alias_ty) = self.types.type_aliases.get(name) {
                    let alias_ty = alias_ty.clone();
                    // Guard against alias pointing back to same name to avoid infinite recursion
                    if !matches!(&alias_ty, ResolvedType::Named { name: alias_name, .. } if alias_name == name)
                    {
                        return self.compute_sizeof(&alias_ty);
                    }
                }
                // Try struct_aliases: e.g., "Box" -> "Box$i64" (generic alias registered by name)
                if let Some(mangled) = self.generics.struct_aliases.get(name) {
                    let mangled = mangled.clone();
                    if let Some(struct_info) = self.types.structs.get(&mangled) {
                        return self.compute_struct_size(struct_info);
                    }
                }
                // Try struct_defs directly (AST-level struct definitions not yet in types.structs)
                // This handles structs that were registered in the AST but not yet resolved into
                // types.structs — compute size from AST field types directly.
                if let Some(struct_def) = self.generics.struct_defs.get(name) {
                    if struct_def.generics.is_empty() {
                        // Non-generic struct — compute directly from AST fields
                        let field_types: Vec<_> = struct_def
                            .fields
                            .iter()
                            .map(|f| self.ast_type_to_resolved(&f.ty.node))
                            .collect();
                        return self.compute_aggregate_size(&field_types);
                    }
                }
                // Std Vec<T> is represented by the runtime `%Vec` layout:
                // data pointer, len, cap, elem_size. When codegen runs without
                // std struct metadata (unit tests and some single-module paths),
                // still treat a Vec value as the full struct, not a pointer.
                if name == "Vec" {
                    return 32;
                }
                // Box<T>, Rc<T>, Arc<T> are always pointer-sized (8 bytes on 64-bit)
                if matches!(name.as_str(), "Box" | "Rc" | "Arc") {
                    return 8;
                }
                // Unknown named type: warn and fall back to pointer-sized i64
                eprintln!(
                    "[vais-codegen] compute_sizeof: unknown Named type '{}' (generics={:?}), \
                     falling back to 8 bytes. This may cause incorrect elem_size in Vec/Array ops.",
                    name, generics
                );
                8 // unknown named type fallback
            }
            ResolvedType::Generic(param) => {
                // Check if we have a substitution for this generic parameter
                if let Some(concrete) = self.get_generic_substitution(param) {
                    // Guard against infinite recursion: if the substitution still
                    // contains the same generic parameter, use default size
                    if Self::type_contains_generic(&concrete, param) {
                        8
                    } else {
                        self.compute_sizeof(&concrete)
                    }
                } else {
                    8 // default i64 size for unresolved generics
                }
            }
            _ => 8, // default for complex types
        }
    }

    fn align_size(offset: i64, align: i64) -> i64 {
        if align <= 1 {
            offset
        } else {
            ((offset + align - 1) / align) * align
        }
    }

    fn compute_aggregate_size(&self, fields: &[ResolvedType]) -> i64 {
        let mut offset = 0;
        let mut max_align = 1;
        for field_ty in fields {
            let align = self.compute_alignof(field_ty).max(1);
            max_align = max_align.max(align);
            offset = Self::align_size(offset, align);
            offset += self.compute_sizeof(field_ty);
        }
        Self::align_size(offset, max_align)
    }

    fn compute_struct_size(&self, struct_info: &crate::types::StructInfo) -> i64 {
        let mut fields: Vec<ResolvedType> = struct_info
            .fields
            .iter()
            .map(|(_name, ty)| ty.clone())
            .collect();
        if struct_info.has_owned_mask {
            fields.push(ResolvedType::I64);
        }
        self.compute_aggregate_size(&fields)
    }

    /// Compute the byte size of an enum from its EnumInfo.
    ///
    /// Enum layout (generated by `generate_enum_type`):
    ///   - No payload variants only: `{ i32 }` → 4 bytes
    ///   - At least one payload variant:
    ///     `{ i32, { i64, i64, … } }` where slot count = max variant field count.
    ///     Raw size = 4 + max_fields × 8.  Padded up to the next multiple of 8
    ///     (LLVM target-abi alignment for i64-containing structs on x86-64).
    fn enum_max_payload_field_count(enum_info: &crate::types::EnumInfo) -> usize {
        enum_info
            .variants
            .iter()
            .map(|v| match &v.fields {
                crate::types::EnumVariantFields::Unit => 0,
                crate::types::EnumVariantFields::Tuple(ts) => ts.len(),
                crate::types::EnumVariantFields::Struct(fs) => fs.len(),
            })
            .max()
            .unwrap_or(0)
    }

    fn compute_enum_sizeof_from_info(enum_info: &crate::types::EnumInfo) -> i64 {
        let max_field_count = Self::enum_max_payload_field_count(enum_info);

        if max_field_count == 0 {
            4 // { i32 } — simple tag-only enum
        } else {
            // { i32, { i64 × N } } — raw = 4 + N*8, padded to next multiple of 8
            let raw = 4 + (max_field_count as i64) * 8;
            // Align to 8-byte boundary
            (raw + 7) & !7
        }
    }

    fn compute_enum_alignof_from_info(enum_info: &crate::types::EnumInfo) -> i64 {
        if Self::enum_max_payload_field_count(enum_info) == 0 {
            4
        } else {
            8
        }
    }

    /// Compute alignof for a ResolvedType (in bytes)
    /// Returns the alignment requirement of the type
    pub(crate) fn compute_alignof(&self, ty: &ResolvedType) -> i64 {
        if let ResolvedType::Named { name, generics } = ty {
            let visit_key = if generics.is_empty() {
                name.clone()
            } else {
                self.mangle_struct_name(name, generics)
            };
            if self.sizeof_visited.borrow().contains(&visit_key) {
                return 8;
            }
            self.sizeof_visited.borrow_mut().insert(visit_key.clone());
            let result = stacker::maybe_grow(32 * 1024 * 1024, 64 * 1024 * 1024, || {
                self.compute_alignof_inner(ty)
            });
            self.sizeof_visited.borrow_mut().remove(&visit_key);
            return result;
        }
        stacker::maybe_grow(32 * 1024 * 1024, 64 * 1024 * 1024, || {
            self.compute_alignof_inner(ty)
        })
    }

    fn compute_alignof_inner(&self, ty: &ResolvedType) -> i64 {
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
            ResolvedType::Optional(inner) => {
                // Option<T> alignment is max(1, align(T))
                std::cmp::max(1, self.compute_alignof(inner))
            }
            ResolvedType::Result(ok, err) => {
                // Result<T, E> alignment is max(1, align(T), align(E))
                std::cmp::max(
                    1,
                    std::cmp::max(self.compute_alignof(ok), self.compute_alignof(err)),
                )
            }
            ResolvedType::Tuple(elems) => elems
                .iter()
                .map(|e| self.compute_alignof(e))
                .max()
                .unwrap_or(8),
            ResolvedType::Named { name, generics } => {
                // First try the base name
                if let Some(struct_info) = self.types.structs.get(name) {
                    return struct_info
                        .fields
                        .iter()
                        .map(|(_name, ty)| self.compute_alignof(ty))
                        .max()
                        .unwrap_or(8);
                }
                // For generic Named types, try the mangled name
                if !generics.is_empty() {
                    let all_concrete = generics
                        .iter()
                        .all(|g| !matches!(g, ResolvedType::Generic(_) | ResolvedType::Var(_)));
                    if all_concrete {
                        let mangled = self.mangle_struct_name(name, generics);
                        if let Some(struct_info) = self.types.structs.get(&mangled) {
                            return struct_info
                                .fields
                                .iter()
                                .map(|(_name, ty)| self.compute_alignof(ty))
                                .max()
                                .unwrap_or(8);
                        }
                    }
                }
                if let Some(enum_info) = self.types.enums.get(name) {
                    return Self::compute_enum_alignof_from_info(enum_info);
                }
                let parent_enum = self
                    .types
                    .enums
                    .values()
                    .find(|ei| ei.variants.iter().any(|v| &v.name == name));
                if let Some(enum_info) = parent_enum {
                    return Self::compute_enum_alignof_from_info(enum_info);
                }
                8 // unknown named type
            }
            ResolvedType::Generic(param) => {
                // Check if we have a substitution for this generic parameter
                if let Some(concrete) = self.get_generic_substitution(param) {
                    self.compute_alignof(&concrete)
                } else {
                    8 // default i64 alignment for unresolved generics
                }
            }
            _ => 8, // default for complex types
        }
    }
}
