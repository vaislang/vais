//! Copy type determination

use super::OwnershipChecker;
use crate::types::ResolvedType;

impl OwnershipChecker {
    /// Determine if a type is Copy (can be implicitly copied rather than moved)
    pub fn is_copy_type(ty: &ResolvedType) -> bool {
        match ty {
            // Primitives are always Copy
            ResolvedType::I8
            | ResolvedType::I16
            | ResolvedType::I32
            | ResolvedType::I64
            | ResolvedType::I128
            | ResolvedType::U8
            | ResolvedType::U16
            | ResolvedType::U32
            | ResolvedType::U64
            | ResolvedType::U128
            | ResolvedType::F32
            | ResolvedType::F64
            | ResolvedType::Bool
            | ResolvedType::Unit
            | ResolvedType::Never => true,

            // References are Copy (the reference itself, not the referent)
            ResolvedType::Ref(_) | ResolvedType::RefLifetime { .. } => true,

            // Mutable references are NOT Copy (uniqueness requirement)
            ResolvedType::RefMut(_) | ResolvedType::RefMutLifetime { .. } => false,

            // Tuples are Copy if all elements are Copy
            ResolvedType::Tuple(elems) => elems.iter().all(Self::is_copy_type),

            // Const arrays are Copy if element type is Copy
            ResolvedType::ConstArray { element, .. } => Self::is_copy_type(element),

            // Strings are Copy (fat pointer { ptr, len } — a borrowed view, not owning data)
            ResolvedType::Str => true,

            // Dynamic arrays, maps, and other heap-allocated types are NOT Copy
            ResolvedType::Array(_) | ResolvedType::Map(_, _) => false,

            // Named structs/enums: permissive Copy by default (Phase 5.24+).
            //
            // Historically this returned `false`, which caused E022 "use after
            // move" to fire on every second use of any user struct parameter
            // — even when the struct contained only primitive fields (Point,
            // Role, User, etc). Stdlib files like hashmap/hashset accumulated
            // ~20 E022 errors purely from this over-strict default.
            //
            // The heap-allocated container types (Vec, HashMap, String) are
            // represented as `ResolvedType::Array`, `::Map`, and the Named
            // `"Vec"`/`"HashMap"` builtins — so they do NOT go through this
            // arm. Treating user Named types as Copy is the correct default
            // for Vais's value-oriented semantics.
            //
            // This intentionally diverges from Rust's more conservative
            // default. A user who needs strict move semantics can annotate
            // with `linear T` or `affine T` (Phase 4.19).
            ResolvedType::Named { name, .. } => {
                // Exception: Vec/HashMap/String/Str builtins ARE heap-allocated
                // and must move (not copy). These are Named with generics.
                !matches!(
                    name.as_str(),
                    "Vec" | "VecMut" | "HashMap" | "HashMapMut" | "String"
                )
            }

            // Generic types: conservative - not Copy
            ResolvedType::Generic(_) => false,

            // Function types are Copy
            ResolvedType::Fn { .. } => true,

            // Pointer types are Copy
            ResolvedType::Pointer(_) => true,

            // Optional/Result: Copy if inner is Copy
            ResolvedType::Optional(inner) => Self::is_copy_type(inner),
            ResolvedType::Result(ok, err) => Self::is_copy_type(ok) && Self::is_copy_type(err),

            // Linear/Affine types are explicitly NOT Copy
            ResolvedType::Linear(_) | ResolvedType::Affine(_) => false,

            // Unknown types: assume Copy to avoid false positives
            // (the type checker has already validated the code)
            ResolvedType::Unknown => true,

            // Everything else: conservative default
            _ => false,
        }
    }
}
