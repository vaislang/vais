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
}
