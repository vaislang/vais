//! Struct, enum, and union LLVM type definition generation

use super::*;
use crate::CodeGenerator;

impl CodeGenerator {
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
        // Enum is represented as { i32 tag, { i64, i64, ... } }
        // The payload is a union of all variants, using i64 slots.
        //
        // All payload fields are stored as i64 by generate_enum_variant_constructor:
        // - Small types (<=8 bytes): value is bitcast into the i64 slot
        // - Large types (>8 bytes, e.g., str { i8*, i64 }): heap-allocated, pointer stored as i64
        // - Pointer types: stored directly as i64
        //
        // This uniform i64 representation ensures type consistency between the
        // enum type definition and the actual storage/extraction IR.
        let mut max_field_count = 0usize;

        for variant in &info.variants {
            let field_count = match &variant.fields {
                EnumVariantFields::Unit => 0,
                EnumVariantFields::Tuple(types) => types.len(),
                EnumVariantFields::Struct(fields) => fields.len(),
            };
            if field_count > max_field_count {
                max_field_count = field_count;
            }
        }

        if max_field_count == 0 {
            // Simple enum with no payload - just use i32 for tag
            format!("%{} = type {{ i32 }}", name)
        } else {
            // Enum with payload - tag + payload struct of i64 slots
            let payload_types: Vec<&str> = vec!["i64"; max_field_count];
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
}
