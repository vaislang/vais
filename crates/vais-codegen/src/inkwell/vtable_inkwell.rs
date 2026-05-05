//! Inkwell-side vtable infrastructure for `dyn Trait` dispatch.
//!
//! Mirrors `crates/vais-codegen/src/vtable.rs` (text-IR backend) but emits
//! via the inkwell builder API instead of string IR. Default backend for
//! `vaisc build` is inkwell — this is the user-impact path.
//!
//! # Status (2b-1, A4-12 step 2b sub-task plan, DEFERRED #18)
//!
//! Dead until call sites land in 2b-5 (`gen_aggregate.rs::generate_method_call`).
//! The F-23 GUARD currently rejects `dyn` receivers with `CodegenError::Unsupported`;
//! 2b-5 will replace that error with a real call into this module.
//!
//! # Memory layout (matches text-IR vtable.rs)
//!
//! Trait object = `{ i8*, i8* }` fat pointer (data, vtable).
//! VTable struct = `{ drop_fn_ptr, i64 size, i64 align, method_fn_ptrs... }`.

use inkwell::context::Context;
use inkwell::types::{BasicTypeEnum, StructType};
use vais_types::TraitDef;

/// Inkwell-side vtable generator. Holds zero state (stateless emitter);
/// vtable globals are deduplicated via the LLVM module symbol table by
/// global name (`@vtable_<ImplType>_<TraitName>`).
#[allow(dead_code)] // Activated in 2b-5 (DEFERRED #18 sub-task plan).
pub(super) struct InkwellVtableGenerator;

#[allow(dead_code)] // Activated in 2b-5.
impl InkwellVtableGenerator {
    /// Build the LLVM struct type for a trait's vtable.
    ///
    /// Layout (matches text-IR vtable.rs `generate_vtable_global`):
    ///   { drop_fn_ptr: i8*, size: i64, align: i64, method_fn_ptrs: i8*... }
    ///
    /// Each method slot is an opaque `i8*`; the actual fn-ptr type is
    /// recovered at call-site by bitcasting to the method's signature
    /// (mirrors text-IR `generate_dynamic_call`).
    pub(super) fn vtable_struct_type<'ctx>(
        ctx: &'ctx Context,
        trait_def: &TraitDef,
    ) -> StructType<'ctx> {
        let i8_ptr: BasicTypeEnum<'ctx> = ctx.i8_type().ptr_type(Default::default()).into();
        let i64_t: BasicTypeEnum<'ctx> = ctx.i64_type().into();

        // drop_fn_ptr + size + align
        let mut fields: Vec<BasicTypeEnum<'ctx>> = vec![i8_ptr, i64_t, i64_t];
        // One opaque fn-ptr slot per trait method, in declaration order.
        for _ in 0..trait_def.methods.len() {
            fields.push(i8_ptr);
        }

        ctx.struct_type(&fields, false)
    }

    /// LLVM type for a trait object: `{ i8*, i8* }` (data, vtable).
    /// Mirrors text-IR `vtable.rs::TRAIT_OBJECT_TYPE`.
    pub(super) fn trait_object_type<'ctx>(ctx: &'ctx Context) -> StructType<'ctx> {
        let i8_ptr: BasicTypeEnum<'ctx> = ctx.i8_type().ptr_type(Default::default()).into();
        ctx.struct_type(&[i8_ptr, i8_ptr], false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;
    use std::collections::HashMap;
    use vais_types::{ResolvedType, TraitDef, TraitMethodSig};

    fn empty_trait(name: &str, n_methods: usize) -> TraitDef {
        let mut methods = HashMap::new();
        for i in 0..n_methods {
            let mname = format!("m{}", i);
            methods.insert(
                mname.clone(),
                TraitMethodSig {
                    name: mname,
                    generics: vec![],
                    params: vec![],
                    ret: ResolvedType::Unit,
                    has_default: false,
                    is_async: false,
                    is_const: false,
                },
            );
        }
        TraitDef {
            name: name.to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods,
        }
    }

    #[test]
    fn vtable_struct_type_has_three_header_fields_plus_one_per_method() {
        let ctx = Context::create();
        let trait_def = empty_trait("T", 2);
        let st = InkwellVtableGenerator::vtable_struct_type(&ctx, &trait_def);
        // 3 header fields (drop, size, align) + 2 method slots = 5
        assert_eq!(st.count_fields(), 5);
    }

    #[test]
    fn vtable_struct_type_zero_methods() {
        let ctx = Context::create();
        let trait_def = empty_trait("Empty", 0);
        let st = InkwellVtableGenerator::vtable_struct_type(&ctx, &trait_def);
        assert_eq!(st.count_fields(), 3);
    }

    #[test]
    fn trait_object_type_is_two_i8_ptrs() {
        let ctx = Context::create();
        let to = InkwellVtableGenerator::trait_object_type(&ctx);
        assert_eq!(to.count_fields(), 2);
    }
}
