//! Single-source index access type derivation (Phase Ω P1.3, iter 97~100).
//!
//! Pre-Phase-Ω there were 4 separate `match arr_ty` blocks in:
//!   - `expr_helpers_data.rs::generate_index_expr` (Path 1, data read) — MIGRATED iter 97
//!   - `expr_helpers_assign.rs::generate_assign_expr` (Path 2, simple) — MIGRATED iter 98
//!   - `expr_helpers_assign.rs::generate_assign_op_expr` (Path 3, compound) — DEFERRED iter 99+
//!     - Reason: minimal current implementation falls back to `llvm_type_of`,
//!       not full match. iter 74 stash@{0} `phaseO_compound_assign_fix`
//!       represents the missing logic; cascade risk 7-8/10 (vaisdb -3 regression
//!       in iter 74 attempt).
//!   - `inkwell/gen_aggregate.rs::generate_index` (Path 4, inkwell) — NOT MIGRATED
//!     - Reason: inkwell uses `inkwell::types::BasicTypeEnum<'ctx>` — a
//!       lifetime-parameterized typed LLVM type — fundamentally incompatible
//!       with this helper's `String` elem_llvm. Inkwell also uses
//!       `var_resolved_types` HashMap directly rather than codegen-local
//!       `infer_expr_type`. Migrating Path 4 would either require a separate
//!       inkwell-specific helper (duplicating logic) or a lossy String round-trip
//!       (parsing fragility). Document as known scope: this helper covers Text IR
//!       backend only. Inkwell Path 4 is treated as a separate implementation
//!       to be revisited in P1.4 (Type-Tagged IR Builder) where the broader
//!       refactor naturally addresses both backends.
//!
//! Each independently mapped `ResolvedType → (elem_llvm_ty, is_fat_ptr,
//! elem_resolved)`. Drift between them caused vaisdb test_btree class of
//! errors (Class 2 in ADR 0002): one path emits `getelementptr i32` while
//! another emits `getelementptr i64` for the same Vec<i32>.
//!
//! This module is the **single source** for the Text IR backend's 3 paths
//! (Path 1, 2, eventually 3). Path 4 (inkwell) is intentionally separate.
//!
//! ADR 0001 §1 invariant (R1):
//!   "Every indexing emit site derives elem_llvm/is_fat_ptr/elem_resolved
//!    from `resolve_index_access`. No site re-implements the match."
//!
//! ADR 0002 Class 2 R3 audit:
//!   `grep -rn 'getelementptr' crates/vais-codegen/src/` should after full
//!   P1.3 migration only contain emit sites whose elem_llvm came from
//!   `resolve_index_access` output.

use crate::error::CodegenError;
use crate::CodeGenerator;
use vais_types::ResolvedType;

/// How an indexed access reaches the element memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AccessKind {
    /// Pointer/Array/ConstArray — direct GEP on `<elem>* base`.
    Direct,
    /// Slice/SliceMut — fat pointer `{ i8*, i64 }`, extract data ptr first.
    FatPtr,
    /// Named{"Vec", [T]} — load `data` field (i64 → i8*), GEP by elem type.
    VecData,
    /// Str — byte-level access via `i8*`.
    StrByte,
}

/// Result of resolving an indexed access.
#[derive(Debug, Clone)]
pub(crate) struct IndexAccess {
    /// LLVM element type string ("i8", "i32", "i64", "%StructName" etc.).
    pub elem_llvm: String,
    /// Element access pattern.
    pub access_kind: AccessKind,
    /// Full ResolvedType for the element when known (Vec<T>'s T). Used by
    /// downstream struct detection / fat-ptr re-derivation.
    pub elem_resolved: Option<ResolvedType>,
}

impl CodeGenerator {
    /// Resolve indexed access on a value of `arr_ty` into the element
    /// derivation triple. Returns `Err(CodegenError::TypeError)` when
    /// `arr_ty` is concretely non-indexable (i64, f64, bool, …).
    ///
    /// Phase Ω P1.3 (iter 97 LANDED): single-source derivation.
    ///
    /// Behavior is **identical** to the prior inline `match arr_ty` blocks
    /// in `expr_helpers_data.rs::generate_index_expr` (Path 1). Paths 2/3/4
    /// migrate in subsequent iters; until then this helper preserves the
    /// exact existing semantics including the i64 fallback for
    /// Named/Unknown/Generic types.
    pub(crate) fn resolve_index_access(
        &self,
        arr_ty: &ResolvedType,
    ) -> Result<IndexAccess, CodegenError> {
        let triple: (String, AccessKind, Option<ResolvedType>) = match arr_ty {
            ResolvedType::Pointer(elem) => {
                (self.type_to_llvm(elem), AccessKind::Direct, None)
            }
            ResolvedType::Array(elem) => {
                (self.type_to_llvm(elem), AccessKind::Direct, None)
            }
            ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => {
                (self.type_to_llvm(elem), AccessKind::FatPtr, None)
            }
            // Vec<T>[idx] → element type T, access via data pointer.
            ResolvedType::Named { name, generics }
                if name == "Vec" && !generics.is_empty() =>
            {
                (
                    self.type_to_llvm(&generics[0]),
                    AccessKind::VecData,
                    Some(generics[0].clone()),
                )
            }
            // &Vec<T>[idx] / &Slice / &Array — peel one ref level.
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                match inner.as_ref() {
                    ResolvedType::Named { name, generics }
                        if name == "Vec" && !generics.is_empty() =>
                    {
                        (
                            self.type_to_llvm(&generics[0]),
                            AccessKind::VecData,
                            Some(generics[0].clone()),
                        )
                    }
                    ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => {
                        (self.type_to_llvm(elem), AccessKind::FatPtr, None)
                    }
                    ResolvedType::Array(elem) => {
                        (self.type_to_llvm(elem), AccessKind::Direct, None)
                    }
                    _ => {
                        // Preserve prior fallback: treat as i64 pointer.
                        // ADR 0002 Class 4 (var-to-llvm) territory; Pillar 1.4
                        // Type-Tagged IR Builder will tighten this.
                        ("i64".to_string(), AccessKind::Direct, None)
                    }
                }
            }
            // Str indexing → byte access.
            ResolvedType::Str => ("i8".to_string(), AccessKind::StrByte, None),
            // Named (non-Vec) / Unknown / Generic — i64 fallback.
            // ADR 0002 Class 4 (var-to-llvm): tightening is Pillar 1.4 work.
            ResolvedType::Named { .. }
            | ResolvedType::Unknown
            | ResolvedType::Generic(_) => {
                ("i64".to_string(), AccessKind::Direct, None)
            }
            other => {
                return Err(CodegenError::TypeError(format!(
                    "Cannot index into type '{}' — indexing requires an array, slice, pointer, Vec, or string type",
                    other
                )));
            }
        };
        Ok(IndexAccess {
            elem_llvm: triple.0,
            access_kind: triple.1,
            elem_resolved: triple.2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_gen() -> CodeGenerator {
        CodeGenerator::new("test")
    }

    #[test]
    fn resolve_pointer_i32() {
        let g = make_gen();
        let arr = ResolvedType::Pointer(Box::new(ResolvedType::I32));
        let acc = g.resolve_index_access(&arr).unwrap();
        assert_eq!(acc.elem_llvm, "i32");
        assert_eq!(acc.access_kind, AccessKind::Direct);
        assert!(acc.elem_resolved.is_none());
    }

    #[test]
    fn resolve_slice_u8_is_fat_ptr() {
        let g = make_gen();
        let arr = ResolvedType::Slice(Box::new(ResolvedType::U8));
        let acc = g.resolve_index_access(&arr).unwrap();
        assert_eq!(acc.elem_llvm, "i8");
        assert_eq!(acc.access_kind, AccessKind::FatPtr);
    }

    #[test]
    fn resolve_vec_i32_is_vec_data() {
        let g = make_gen();
        let arr = ResolvedType::Named {
            name: "Vec".to_string(),
            generics: vec![ResolvedType::I32],
        };
        let acc = g.resolve_index_access(&arr).unwrap();
        assert_eq!(acc.elem_llvm, "i32");
        assert_eq!(acc.access_kind, AccessKind::VecData);
        assert_eq!(acc.elem_resolved, Some(ResolvedType::I32));
    }

    #[test]
    fn resolve_ref_vec_u8_peels_ref() {
        let g = make_gen();
        let arr = ResolvedType::Ref(Box::new(ResolvedType::Named {
            name: "Vec".to_string(),
            generics: vec![ResolvedType::U8],
        }));
        let acc = g.resolve_index_access(&arr).unwrap();
        assert_eq!(acc.elem_llvm, "i8");
        assert_eq!(acc.access_kind, AccessKind::VecData);
    }

    #[test]
    fn resolve_str_is_byte_access() {
        let g = make_gen();
        let acc = g.resolve_index_access(&ResolvedType::Str).unwrap();
        assert_eq!(acc.elem_llvm, "i8");
        assert_eq!(acc.access_kind, AccessKind::StrByte);
    }

    #[test]
    fn resolve_unknown_falls_back_to_i64() {
        // ADR 0002 Class 4: this fallback is tracked as known-imprecise
        // until Pillar 1.4 (Type-Tagged IR Builder) tightens it.
        let g = make_gen();
        let acc = g.resolve_index_access(&ResolvedType::Unknown).unwrap();
        assert_eq!(acc.elem_llvm, "i64");
    }

    #[test]
    fn resolve_concrete_non_indexable_errors() {
        let g = make_gen();
        let err = g.resolve_index_access(&ResolvedType::I64).unwrap_err();
        match err {
            CodegenError::TypeError(msg) => {
                assert!(msg.contains("Cannot index into type"));
            }
            _ => panic!("expected TypeError, got {:?}", err),
        }
    }
}
