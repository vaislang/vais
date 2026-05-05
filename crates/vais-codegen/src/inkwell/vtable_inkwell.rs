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
use inkwell::types::{BasicType, BasicTypeEnum, StructType};
use inkwell::values::{BasicValueEnum, GlobalValue, StructValue};
use inkwell::AddressSpace;
use vais_types::{ResolvedType, TraitDef, TraitMethodSig, AssociatedTypeDef};
use vais_ast::Trait as AstTrait;

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
    /// Mirrors text-IR `register_trait`. Wired in 2b-5a.
    pub(super) fn register_trait_inkwell(&mut self, trait_def: TraitDef) {
        self.trait_defs.insert(trait_def.name.clone(), trait_def);
    }

    /// Build a `TraitDef` from an AST `Trait` node and register it.
    /// Mirrors text-IR `register_trait_from_ast` (trait_dispatch.rs:15).
    /// Wired in 2b-5a from `generate_module` Item::Trait handling.
    pub(super) fn register_trait_from_ast_inkwell(&mut self, t: &AstTrait) {
        let mut methods = std::collections::HashMap::new();
        for m in &t.methods {
            let params: Vec<(String, ResolvedType, bool)> = m
                .params
                .iter()
                .map(|p| {
                    let ty = if p.name.node == "self" {
                        // self parameter is a pointer to the implementing type;
                        // resolved at call site. Match text-IR placeholder.
                        ResolvedType::I64
                    } else {
                        self.ast_type_to_resolved(&p.ty.node)
                    };
                    (p.name.node.clone(), ty, p.is_mut)
                })
                .collect();

            let ret = m
                .ret_type
                .as_ref()
                .map(|t| self.ast_type_to_resolved(&t.node))
                .unwrap_or(ResolvedType::Unit);

            let meth_name = m.name.node.clone();
            methods.insert(
                meth_name.clone(),
                TraitMethodSig {
                    name: meth_name,
                    generics: m.generics.iter().map(|g| g.name.node.clone()).collect(),
                    params,
                    ret,
                    has_default: m.default_body.is_some(),
                    is_async: m.is_async,
                    is_const: m.is_const,
                },
            );
        }

        // Associated types (mirror text-IR shape).
        let mut associated_types = std::collections::HashMap::new();
        for assoc in &t.associated_types {
            let generics = assoc.generics.iter().map(|g| g.name.node.clone()).collect();

            let mut generic_bounds = std::collections::HashMap::new();
            for gen_param in &assoc.generics {
                if !gen_param.bounds.is_empty() {
                    generic_bounds.insert(
                        gen_param.name.node.clone(),
                        gen_param.bounds.iter().map(|b| b.node.clone()).collect(),
                    );
                }
            }

            let bounds = assoc.bounds.iter().map(|b| b.node.clone()).collect();
            let default = assoc
                .default
                .as_ref()
                .map(|d| self.ast_type_to_resolved(&d.node));

            let assoc_name = assoc.name.node.clone();
            associated_types.insert(
                assoc_name.clone(),
                AssociatedTypeDef {
                    name: assoc_name,
                    generics,
                    generic_bounds,
                    bounds,
                    default,
                },
            );
        }

        let trait_def = TraitDef {
            name: t.name.node.clone(),
            generics: t.generics.iter().map(|g| g.name.node.clone()).collect(),
            super_traits: t.super_traits.iter().map(|s| s.node.clone()).collect(),
            associated_types,
            methods,
        };
        self.register_trait_inkwell(trait_def);
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

    /// Create a trait object value: `{ data_ptr, vtable_ptr }` fat pointer
    /// (DEFERRED #18 sub-task 2b-3).
    ///
    /// Strategy (matches text-IR `create_trait_object` shape):
    ///   1. malloc(8) for the data slot (placeholder size; refined later).
    ///   2. store concrete_value into the alloc'd slot (via bitcast).
    ///   3. cast vtable global pointer to i8*.
    ///   4. insertvalue into `{ i8*, i8* }` undef.
    ///
    /// Returns the trait object as a StructValue. None if vtable cannot be
    /// generated (caller should have already validated the trait/impl pair).
    ///
    /// Pre-conditions: builder must be positioned at a valid insertion point
    /// (i.e. inside a function body). 2b-5 ensures this.
    pub(super) fn create_trait_object_inkwell(
        &mut self,
        concrete_value: BasicValueEnum<'ctx>,
        impl_type: &str,
        trait_name: &str,
    ) -> Option<StructValue<'ctx>> {
        let vtable_global = self.get_or_generate_vtable_inkwell(impl_type, trait_name)?;
        let i8_ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ty = self.context.i64_type();

        // 1. Allocate data slot via malloc(8).
        let malloc_fn = self.module.get_function("malloc")?;
        let size_arg = i64_ty.const_int(8, false);
        let data_ptr_call = self
            .builder
            .build_call(malloc_fn, &[size_arg.into()], "trait_data")
            .ok()?;
        let data_ptr = data_ptr_call
            .try_as_basic_value()
            .left()?
            .into_pointer_value();

        // 2. Bitcast data_ptr to <concrete_value type>* and store.
        let concrete_ty = concrete_value.get_type();
        let typed_ptr_ty = concrete_ty.ptr_type(AddressSpace::default());
        let typed_ptr = self
            .builder
            .build_bitcast(data_ptr, typed_ptr_ty, "trait_cast")
            .ok()?
            .into_pointer_value();
        self.builder.build_store(typed_ptr, concrete_value).ok()?;

        // 3. Cast vtable global pointer to i8*.
        let vtable_raw_ptr = vtable_global.as_pointer_value();
        let vtable_i8_ptr = self
            .builder
            .build_bitcast(vtable_raw_ptr, i8_ptr_ty, "vtable_cast")
            .ok()?
            .into_pointer_value();

        // 4. Build `{ i8*, i8* }` via insertvalue.
        let to_ty = InkwellVtableGenerator::trait_object_type(self.context);
        let undef = to_ty.get_undef();
        let with_data = self
            .builder
            .build_insert_value(undef, data_ptr, 0, "trait_obj.data")
            .ok()?;
        let full = self
            .builder
            .build_insert_value(with_data, vtable_i8_ptr, 1, "trait_obj.vtable")
            .ok()?;

        Some(full.into_struct_value())
    }

    /// Indirect dispatch through a trait object's vtable
    /// (DEFERRED #18 sub-task 2b-4).
    ///
    /// Strategy (matches text-IR `generate_dynamic_call` shape):
    ///   1. extractvalue 0 → data_ptr (i8*)
    ///   2. extractvalue 1 → vtable_ptr_i8 (i8*)
    ///   3. bitcast vtable_ptr_i8 to <VTableTy>*
    ///   4. GEP slot index = method_index + 3 (skip drop/size/align)
    ///   5. load i8* from slot, then bitcast to fn-ptr
    ///   6. call fn(data_ptr, args...)
    ///
    /// Simplified ABI (matches text-IR side):
    /// - self → i8*
    /// - every additional arg → i64
    /// - return type → i64 / void / { i8*, i64 } per `vtable_ret_type` rules
    ///
    /// Returns the call result as `Option<BasicValueEnum>` (None for void).
    /// Outer `Option` wraps build-error / lookup-error.
    pub(super) fn generate_dynamic_call_inkwell(
        &mut self,
        trait_object: StructValue<'ctx>,
        trait_name: &str,
        method_name: &str,
        args: &[BasicValueEnum<'ctx>],
    ) -> Option<Option<BasicValueEnum<'ctx>>> {
        let trait_def = self.trait_defs.get(trait_name)?.clone();
        let order = InkwellVtableGenerator::method_order(&trait_def);
        let method_index = order.iter().position(|n| n == method_name)?;

        let i8_ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ty = self.context.i64_type();
        let vtable_ty = InkwellVtableGenerator::vtable_struct_type(self.context, &trait_def);

        // 1. extract data_ptr (i8*).
        let data_ptr = self
            .builder
            .build_extract_value(trait_object, 0, "dyn.data")
            .ok()?
            .into_pointer_value();

        // 2. extract vtable_ptr (i8*).
        let vtable_i8 = self
            .builder
            .build_extract_value(trait_object, 1, "dyn.vtable")
            .ok()?
            .into_pointer_value();

        // 3. cast vtable_ptr to <VTableTy>*.
        let vtable_ptr_ty = vtable_ty.ptr_type(AddressSpace::default());
        let vtable_typed = self
            .builder
            .build_bitcast(vtable_i8, vtable_ptr_ty, "dyn.vtable_typed")
            .ok()?
            .into_pointer_value();

        // 4. GEP into slot (method_index + 3).
        let slot_index = method_index as u32 + 3;
        let fn_ptr_ptr = self
            .builder
            .build_struct_gep(vtable_ty, vtable_typed, slot_index, "dyn.fn_ptr_ptr")
            .ok()?;

        // 5. Load i8* from slot, then bitcast to concrete fn-ptr type.
        let raw_fn_ptr = self
            .builder
            .build_load(i8_ptr_ty, fn_ptr_ptr, "dyn.fn_ptr_raw")
            .ok()?
            .into_pointer_value();

        // Build fn type: ret(i8*, i64*) where additional args are widened to i64.
        let method_sig = trait_def.methods.get(method_name)?;
        let ret_ty_str = super::super::vtable::vtable_ret_type(&method_sig.ret, method_sig.is_async);

        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> =
            vec![i8_ptr_ty.into()];
        for _ in args {
            param_types.push(i64_ty.into());
        }

        let fn_type = match ret_ty_str {
            "void" => self.context.void_type().fn_type(&param_types, false),
            "i64" => i64_ty.fn_type(&param_types, false),
            "{ i8*, i64 }" => {
                // String fat-pointer return — build matching struct type.
                let fat = self
                    .context
                    .struct_type(&[i8_ptr_ty.into(), i64_ty.into()], false);
                fat.fn_type(&param_types, false)
            }
            _ => i64_ty.fn_type(&param_types, false),
        };
        let fn_ptr_typed = self
            .builder
            .build_bitcast(
                raw_fn_ptr,
                fn_type.ptr_type(AddressSpace::default()),
                "dyn.fn_ptr",
            )
            .ok()?
            .into_pointer_value();

        // 6. Call (data_ptr, args...).
        let mut call_args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> =
            vec![data_ptr.into()];
        for arg in args {
            // Widen any non-i64 args to i64 (simplification matching text-IR).
            let widened = match *arg {
                BasicValueEnum::IntValue(iv) if iv.get_type() != i64_ty => self
                    .builder
                    .build_int_z_extend_or_bit_cast(iv, i64_ty, "dyn.arg.zext")
                    .ok()?
                    .into(),
                BasicValueEnum::IntValue(iv) => iv.into(),
                other => other.into(),
            };
            call_args.push(widened);
        }

        let call_site =
            self.builder
                .build_indirect_call(fn_type, fn_ptr_typed, &call_args, "dyn.call")
                .ok()?;

        Some(call_site.try_as_basic_value().left())
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
