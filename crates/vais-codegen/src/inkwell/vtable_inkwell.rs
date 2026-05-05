//! Inkwell-side vtable infrastructure for `dyn Trait` dispatch.
//!
//! Mirrors `crates/vais-codegen/src/vtable.rs` (text-IR backend) but emits
//! via the inkwell builder API instead of string IR. Default backend for
//! `vaisc build` is inkwell — this is the user-impact path.
//!
//! # Status (2b-2, A4-12 step 2b sub-task plan, DEFERRED #18)
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
use inkwell::module::Linkage;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{BasicValueEnum, GlobalValue};
use inkwell::AddressSpace;
use vais_types::TraitDef;

use super::generator::InkwellCodeGenerator;

/// Inkwell-side vtable type helpers. Stateless emitter — vtable globals are
/// deduplicated via `InkwellCodeGenerator::vtable_globals`.
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
        let i8_ptr: BasicTypeEnum<'ctx> =
            ctx.i8_type().ptr_type(AddressSpace::default()).into();
        let i64_t: BasicTypeEnum<'ctx> = ctx.i64_type().into();

        // drop_fn_ptr + size + align
        let mut fields: Vec<BasicTypeEnum<'ctx>> = vec![i8_ptr, i64_t, i64_t];
        // One opaque fn-ptr slot per trait method, in declaration order
        // (alphabetical sort by method name for determinism — text-IR side
        // uses HashMap iteration order which is non-deterministic; inkwell
        // side enforces stable order so dispatch indices are reproducible).
        for _ in 0..trait_def.methods.len() {
            fields.push(i8_ptr);
        }

        ctx.struct_type(&fields, false)
    }

    /// LLVM type for a trait object: `{ i8*, i8* }` (data, vtable).
    /// Mirrors text-IR `vtable.rs::TRAIT_OBJECT_TYPE`.
    pub(super) fn trait_object_type<'ctx>(ctx: &'ctx Context) -> StructType<'ctx> {
        let i8_ptr: BasicTypeEnum<'ctx> =
            ctx.i8_type().ptr_type(AddressSpace::default()).into();
        ctx.struct_type(&[i8_ptr, i8_ptr], false)
    }

    /// Stable method order: sort method names alphabetically. Used by both
    /// vtable global emission (slot order) and `generate_dynamic_call` (slot
    /// index). 2b-2 contract: any caller indexing into the vtable MUST go
    /// through this helper.
    pub(super) fn method_order(trait_def: &TraitDef) -> Vec<String> {
        let mut names: Vec<String> = trait_def.methods.keys().cloned().collect();
        names.sort();
        names
    }
}

#[allow(dead_code)] // Activated in 2b-5.
impl<'ctx> InkwellCodeGenerator<'ctx> {
    /// Register a trait definition for inkwell-side vtable generation.
    /// Mirrors text-IR `register_trait`. Inkwell-side does not currently
    /// auto-call this from the AST first pass — 2b-5 will wire it.
    pub(super) fn register_trait_inkwell(&mut self, trait_def: TraitDef) {
        self.trait_defs.insert(trait_def.name.clone(), trait_def);
    }

    /// Register a trait implementation: maps (impl_type, trait_name) to
    /// the concrete method names. Mirrors text-IR `register_trait_impl`.
    pub(super) fn register_trait_impl_inkwell(
        &mut self,
        impl_type: &str,
        trait_name: &str,
        method_impls: std::collections::HashMap<String, String>,
    ) {
        self.trait_impl_methods
            .insert((impl_type.to_string(), trait_name.to_string()), method_impls);
    }

    /// Get-or-generate vtable global for (impl_type, trait_name).
    ///
    /// Layout: `{ drop_ptr (null for now), size (8), align (8), method_ptrs... }`.
    /// All method pointers are bitcast to `i8*` for storage; call sites
    /// recover the concrete fn-ptr type via cast at dispatch (2b-4).
    ///
    /// Returns `None` if the trait is not registered (caller error) or if a
    /// required (non-default) method has no impl entry.
    pub(super) fn get_or_generate_vtable_inkwell(
        &mut self,
        impl_type: &str,
        trait_name: &str,
    ) -> Option<GlobalValue<'ctx>> {
        let key = (impl_type.to_string(), trait_name.to_string());
        if let Some(g) = self.vtable_globals.get(&key) {
            return Some(*g);
        }

        let trait_def = self.trait_defs.get(trait_name)?.clone();
        let method_impls = self
            .trait_impl_methods
            .get(&key)
            .cloned()
            .unwrap_or_default();

        let vtable_ty = InkwellVtableGenerator::vtable_struct_type(self.context, &trait_def);
        let i8_ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ty = self.context.i64_type();

        // Build initializer values in declaration order:
        //   [0] drop_fn_ptr (null for now — Drop trait integration is separate)
        //   [1] size (placeholder 8; type-size discovery is 2b-3 concern)
        //   [2] align (placeholder 8)
        //   [3..] method fn-ptrs, bitcast to i8*
        let drop_null: BasicValueEnum<'ctx> = i8_ptr_ty.const_null().into();
        let size_val: BasicValueEnum<'ctx> = i64_ty.const_int(8, false).into();
        let align_val: BasicValueEnum<'ctx> = i64_ty.const_int(8, false).into();

        let mut field_vals: Vec<BasicValueEnum<'ctx>> = vec![drop_null, size_val, align_val];

        for method_name in InkwellVtableGenerator::method_order(&trait_def) {
            let mangled = method_impls.get(&method_name).cloned().or_else(|| {
                // Fall back to Trait_method_default if has_default
                trait_def
                    .methods
                    .get(&method_name)
                    .filter(|sig| sig.has_default)
                    .map(|_| format!("{}_{}_default", trait_name, method_name))
            });

            let slot_val: BasicValueEnum<'ctx> = match mangled {
                Some(name) => {
                    let fn_val = self.module.get_function(&name)?;
                    let raw_ptr = fn_val.as_global_value().as_pointer_value();
                    raw_ptr.const_cast(i8_ptr_ty).into()
                }
                None => {
                    // Required method missing — caller is malformed. Bail.
                    return None;
                }
            };
            field_vals.push(slot_val);
        }

        let initializer = vtable_ty.const_named_struct(&field_vals);

        let global_name = format!("vtable_{}_{}", impl_type, trait_name);
        let global = self
            .module
            .add_global(vtable_ty, Some(AddressSpace::default()), &global_name);
        global.set_initializer(&initializer);
        global.set_linkage(Linkage::Internal);
        global.set_constant(true);

        self.vtable_globals.insert(key, global);
        Some(global)
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

    #[test]
    fn method_order_is_alphabetical_and_stable() {
        let mut methods = HashMap::new();
        for n in ["zebra", "alpha", "mango"] {
            methods.insert(
                n.to_string(),
                TraitMethodSig {
                    name: n.to_string(),
                    generics: vec![],
                    params: vec![],
                    ret: ResolvedType::Unit,
                    has_default: false,
                    is_async: false,
                    is_const: false,
                },
            );
        }
        let trait_def = TraitDef {
            name: "T".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods,
        };
        let order = InkwellVtableGenerator::method_order(&trait_def);
        assert_eq!(order, vec!["alpha", "mango", "zebra"]);
    }
}
