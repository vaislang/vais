//! Aggregate type code generation.
//!
//! Handles arrays, tuples, indexing, slicing, method calls,
//! and lambda expressions.

use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, PointerValue};
use inkwell::{AddressSpace, IntPredicate};

use vais_ast::{self as ast, Expr, IfElse, Spanned, Stmt};

use super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    /// Generate a map literal as parallel key/value arrays on the stack.
    ///
    /// `{k1: v1, k2: v2}` becomes:
    /// - `keys:   [K x N]` allocated on the stack with each key stored
    /// - `values: [V x N]` allocated on the stack with each value stored
    /// - Returns a pointer to the keys array (consistent with Text IR backend)
    pub(super) fn generate_map_literal(
        &mut self,
        pairs: &[(Spanned<Expr>, Spanned<Expr>)],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        if pairs.is_empty() {
            // Empty map — return null pointer
            return Ok(self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .const_null()
                .into());
        }

        let len = pairs.len() as u32;

        // Generate all key-value pairs first to determine types
        let mut key_vals: Vec<BasicValueEnum<'ctx>> = Vec::new();
        let mut val_vals: Vec<BasicValueEnum<'ctx>> = Vec::new();
        for (k, v) in pairs {
            key_vals.push(self.generate_expr(&k.node)?);
            val_vals.push(self.generate_expr(&v.node)?);
        }

        // Determine key/value types from first pair
        let key_type = key_vals[0].get_type();
        let val_type = val_vals[0].get_type();

        let key_arr_type = key_type.array_type(len);
        let val_arr_type = val_type.array_type(len);

        // Allocate key and value arrays on stack
        let keys_ptr = self
            .builder
            .build_alloca(key_arr_type, "map_keys")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let vals_ptr = self
            .builder
            .build_alloca(val_arr_type, "map_vals")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store each key-value pair
        let zero = self.context.i64_type().const_int(0, false);
        for (i, (kv, vv)) in key_vals.iter().zip(val_vals.iter()).enumerate() {
            let idx = self.context.i64_type().const_int(i as u64, false);

            // Store key
            // SAFETY: GEP index i is bounded by key_vals.len(), matching the alloca'd array size.
            let k_elem_ptr = unsafe {
                self.builder
                    .build_gep(
                        key_arr_type,
                        keys_ptr,
                        &[zero, idx],
                        &format!("map_k_{}", i),
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            };
            self.builder
                .build_store(k_elem_ptr, *kv)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Store value
            // SAFETY: GEP index i is bounded by val_vals.len(), matching the alloca'd array size.
            let v_elem_ptr = unsafe {
                self.builder
                    .build_gep(
                        val_arr_type,
                        vals_ptr,
                        &[zero, idx],
                        &format!("map_v_{}", i),
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            };
            self.builder
                .build_store(v_elem_ptr, *vv)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Return pointer to keys array (consistent with Text IR backend representation)
        Ok(keys_ptr.into())
    }

    pub(super) fn generate_array(
        &mut self,
        elements: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        if elements.is_empty() {
            // Empty array - return null pointer
            return Ok(self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .const_null()
                .into());
        }

        // Generate all elements
        let mut values: Vec<BasicValueEnum> = Vec::new();
        for elem in elements {
            values.push(self.generate_expr(&elem.node)?);
        }

        // Determine element type from first element
        let elem_type = values[0].get_type();
        let array_type = elem_type.array_type(elements.len() as u32);

        // Allocate array on stack
        let array_ptr = self
            .builder
            .build_alloca(array_type, "array")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store each element
        for (i, val) in values.iter().enumerate() {
            let idx = self.context.i64_type().const_int(i as u64, false);
            // SAFETY: GEP index i is bounded by elements.len(), matching the alloca'd array size.
            let elem_ptr = unsafe {
                self.builder
                    .build_gep(
                        array_type,
                        array_ptr,
                        &[self.context.i64_type().const_int(0, false), idx],
                        &format!("array_elem_{}", i),
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            };
            self.builder
                .build_store(elem_ptr, *val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        Ok(array_ptr.into())
    }

    pub(super) fn generate_tuple(
        &mut self,
        elements: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        if elements.is_empty() {
            return Ok(self.unit_value());
        }

        // Generate all elements
        let mut values: Vec<BasicValueEnum> = Vec::new();
        for elem in elements {
            values.push(self.generate_expr(&elem.node)?);
        }

        // Create anonymous struct type for tuple
        let field_types: Vec<BasicTypeEnum> = values.iter().map(|v| v.get_type()).collect();
        let tuple_type = self.context.struct_type(&field_types, false);

        // Build tuple value
        let mut tuple_val = tuple_type.get_undef();
        for (i, val) in values.iter().enumerate() {
            tuple_val = self
                .builder
                .build_insert_value(tuple_val, *val, i as u32, "tuple")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_struct_value();
        }

        Ok(tuple_val.into())
    }

    /// Infer the element LLVM type for a slice, array, or Vec expression.
    /// Looks up the variable's resolved type from `var_resolved_types` and extracts
    /// the inner element type for Slice/SliceMut/Array/Vec<T>. Falls back to i64 if unknown.
    fn infer_element_llvm_type(&self, arr_expr: &Expr) -> inkwell::types::BasicTypeEnum<'ctx> {
        // Phase 3.14: handle struct-field Vec<T> via type-name string parse.
        if let Expr::Field {
            expr: object,
            field,
        } = arr_expr
        {
            if let Expr::Ident(parent_name) = &object.node {
                let parent_struct = match self.var_resolved_types.get(parent_name) {
                    Some(vais_types::ResolvedType::Named { name, .. }) => Some(name.clone()),
                    Some(vais_types::ResolvedType::Ref(inner))
                    | Some(vais_types::ResolvedType::RefMut(inner)) => {
                        if let vais_types::ResolvedType::Named { name, .. } = inner.as_ref() {
                            Some(name.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                if let Some(ps) = parent_struct {
                    if let Some(fields) = self.struct_field_type_names.get(&ps) {
                        for (fname, ftype) in fields {
                            if fname == &field.node {
                                // Parse "Vec<Column>" / "Vec<u64>" style
                                if let Some(start) = ftype.find('<') {
                                    if ftype.ends_with('>') && ftype.starts_with("Vec<") {
                                        let inner_name = &ftype[start + 1..ftype.len() - 1];
                                        let inner_ty = match inner_name {
                                            "i8" => vais_types::ResolvedType::I8,
                                            "i16" => vais_types::ResolvedType::I16,
                                            "i32" => vais_types::ResolvedType::I32,
                                            "i64" => vais_types::ResolvedType::I64,
                                            "u8" => vais_types::ResolvedType::U8,
                                            "u16" => vais_types::ResolvedType::U16,
                                            "u32" => vais_types::ResolvedType::U32,
                                            "u64" => vais_types::ResolvedType::U64,
                                            "f32" => vais_types::ResolvedType::F32,
                                            "f64" => vais_types::ResolvedType::F64,
                                            "bool" => vais_types::ResolvedType::Bool,
                                            "str" => vais_types::ResolvedType::Str,
                                            other => vais_types::ResolvedType::Named {
                                                name: other.to_string(),
                                                generics: vec![],
                                            },
                                        };
                                        return self.type_mapper.map_type(&inner_ty);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Expr::Ident(name) = arr_expr {
            match self.var_resolved_types.get(name) {
                Some(
                    vais_types::ResolvedType::Slice(inner)
                    | vais_types::ResolvedType::SliceMut(inner)
                    | vais_types::ResolvedType::Array(inner),
                ) => {
                    return self.type_mapper.map_type(inner);
                }
                // Vec<T>[idx] -> element type T
                Some(vais_types::ResolvedType::Named {
                    name: type_name,
                    generics,
                }) if type_name == "Vec" && !generics.is_empty() => {
                    return self.type_mapper.map_type(&generics[0]);
                }
                _ => {}
            }
        }
        // Fallback to i64 for untracked expressions
        self.context.i64_type().into()
    }

    /// Check if an expression resolves to a Vec type (Named { name: "Vec", ... }).
    /// Phase 3.14: also handles Expr::Field for struct-embedded Vec<T> by
    /// looking up the parent struct's field type via struct_field_type_names
    /// and parsing the type string.
    pub(super) fn is_vec_expr(&self, arr_expr: &Expr) -> bool {
        match arr_expr {
            Expr::Ident(name) => {
                if let Some(vais_types::ResolvedType::Named {
                    name: type_name, ..
                }) = self.var_resolved_types.get(name)
                {
                    return type_name == "Vec";
                }
                false
            }
            Expr::Field {
                expr: object,
                field,
            } => {
                // Resolve parent struct name from the object's type
                let parent_struct = if let Expr::Ident(parent_name) = &object.node {
                    match self.var_resolved_types.get(parent_name) {
                        Some(vais_types::ResolvedType::Named { name, .. }) => name.clone(),
                        Some(vais_types::ResolvedType::Ref(inner))
                        | Some(vais_types::ResolvedType::RefMut(inner)) => {
                            if let vais_types::ResolvedType::Named { name, .. } = inner.as_ref() {
                                name.clone()
                            } else {
                                return false;
                            }
                        }
                        _ => return false,
                    }
                } else {
                    return false;
                };
                let fields = match self.struct_field_type_names.get(&parent_struct) {
                    Some(f) => f,
                    None => return false,
                };
                for (fname, ftype) in fields {
                    if fname == &field.node {
                        return ftype.starts_with("Vec<") || ftype == "Vec";
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub(super) fn generate_index(
        &mut self,
        arr: &Expr,
        index: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Infer element type before generating (uses AST-level info)
        let inferred_elem_type = self.infer_element_llvm_type(arr);
        let is_vec = self.is_vec_expr(arr);

        let arr_val = self.generate_expr(arr)?;
        let idx_val = self.generate_expr(index)?;

        // Check if this is a struct value (slice or Vec)
        if arr_val.is_struct_value() {
            let struct_val = arr_val.into_struct_value();
            let struct_type = struct_val.get_type();

            // Vec<T> struct: { i64 data, i64 len, i64 cap, i64 elem_size }
            // The data field holds a raw pointer as i64; elem_size provides stride.
            if is_vec && struct_type.count_fields() >= 4 {
                let idx_int = idx_val.into_int_value();

                // Extract data pointer (field 0) as i64, then convert to pointer
                let data_i64 = self
                    .builder
                    .build_extract_value(struct_val, 0, "vec_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_int_value();

                let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
                let data_ptr = self
                    .builder
                    .build_int_to_ptr(data_i64, i8_ptr_type, "vec_data_ptr")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // Extract elem_size (field 3) for correct stride calculation
                let elem_size = self
                    .builder
                    .build_extract_value(struct_val, 3, "vec_elem_size")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_int_value();

                // Calculate byte offset: idx * elem_size
                let byte_offset = self
                    .builder
                    .build_int_mul(idx_int, elem_size, "byte_offset")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // GEP on i8* with byte offset to get element pointer
                // SAFETY: Runtime index from user code — bounds checking is the caller's responsibility.
                // data_ptr comes from a valid Vec data field.
                let elem_ptr_i8 = unsafe {
                    self.builder
                        .build_gep(
                            self.context.i8_type(),
                            data_ptr,
                            &[byte_offset],
                            "vec_elem_ptr",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                };

                // Load element using the inferred element type
                return self
                    .builder
                    .build_load(inferred_elem_type, elem_ptr_i8, "vec_elem")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()));
            }

            // Slice is { i8*, i64 } - check if it has exactly 2 fields
            if struct_type.count_fields() == 2 {
                // Extract length (field 1) for bounds check
                let len_val = self
                    .builder
                    .build_extract_value(struct_val, 1, "slice_len")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_int_value();

                let idx_int = idx_val.into_int_value();

                // Bounds check: idx < len (unsigned)
                let in_bounds = self
                    .builder
                    .build_int_compare(IntPredicate::ULT, idx_int, len_val, "bounds_check")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                let current_fn = self.current_function.ok_or_else(|| {
                    CodegenError::LlvmError("No current function for bounds check".to_string())
                })?;

                let safe_bb = self.context.append_basic_block(current_fn, "bounds_safe");
                let oob_bb = self.context.append_basic_block(current_fn, "bounds_oob");

                self.builder
                    .build_conditional_branch(in_bounds, safe_bb, oob_bb)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // OOB path: abort
                self.builder.position_at_end(oob_bb);
                let abort_fn = self.get_or_declare_abort();
                self.builder
                    .build_call(abort_fn, &[], "")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder
                    .build_unreachable()
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // Safe path: continue with element access
                self.builder.position_at_end(safe_bb);

                // Extract data pointer (field 0)
                let data_ptr = self
                    .builder
                    .build_extract_value(struct_val, 0, "data_ptr")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // Use the inferred element type from the variable's resolved type.
                // The GEP instruction uses elem_type for stride calculation.
                let elem_type = inferred_elem_type;
                let data_ptr_val = data_ptr.into_pointer_value();

                // GEP to get element pointer (stride = sizeof(elem_type))
                // SAFETY: Runtime index from user code — bounds checking is the caller's responsibility.
                // data_ptr comes from a valid Vec/Slice data field.
                let elem_ptr = unsafe {
                    self.builder
                        .build_gep(elem_type, data_ptr_val, &[idx_int], "elem_ptr")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                };

                // Load element
                return self
                    .builder
                    .build_load(elem_type, elem_ptr, "elem")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()));
            }
        }

        // Phase 3.14 safety: clean error for struct values that didn't match
        // any prior arm (Vec 4-field or Slice 2-field). Without this the
        // into_pointer_value() call below panics in inkwell.
        if arr_val.is_struct_value() {
            return Err(CodegenError::TypeError(format!(
                "Cannot index into this struct value directly at codegen. \
                 If this is `obj.vec_field[i]` with a struct-embedded Vec, \
                 the type lookup may have failed. Workaround: copy to a \
                 local first — `v := obj.vec_field; x := v[i]`."
            )));
        }

        // Phase 0 bug C6 fix: a fixed-size array value (e.g. extracted from
        // a struct field via `extractvalue %T %p, 0`) arrives as an
        // ArrayValue, not a PointerValue. Spill to an alloca so we can GEP
        // through it. This handles `S P { c: [i64;3] } let p := P{...}; p.c[i]`.
        if arr_val.is_array_value() {
            let array_val = arr_val.into_array_value();
            let array_ty = array_val.get_type();
            let tmp = self
                .builder
                .build_alloca(array_ty, "array_tmp")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(tmp, array_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            let idx_int = idx_val.into_int_value();
            let zero = self.context.i64_type().const_zero();
            // SAFETY: `tmp` is an alloca of `array_ty`, and the GEP uses the
            // canonical `[0, idx]` path into that aggregate. Runtime bounds are
            // enforced by the source language/checker before codegen reaches here.
            let elem_ptr = unsafe {
                self.builder
                    .build_gep(array_ty, tmp, &[zero, idx_int], "array_elem_ptr")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            };
            return self
                .builder
                .build_load(inferred_elem_type, elem_ptr, "array_elem")
                .map_err(|e| CodegenError::LlvmError(e.to_string()));
        }

        // Regular array/pointer indexing — use inferred element type
        let arr_ptr = arr_val.into_pointer_value();
        let idx_int = idx_val.into_int_value();

        // Get element pointer
        // SAFETY: Runtime index from user code — bounds checking is the caller's responsibility.
        // arr_ptr is a valid array/pointer from a prior allocation or parameter.
        let elem_ptr = unsafe {
            self.builder
                .build_gep(inferred_elem_type, arr_ptr, &[idx_int], "elem_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };

        // Load element
        self.builder
            .build_load(inferred_elem_type, elem_ptr, "elem")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    pub(super) fn generate_slice(
        &mut self,
        arr: &Expr,
        start: Option<&Spanned<Expr>>,
        end: Option<&Spanned<Expr>>,
        inclusive: bool,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self
            .current_function
            .ok_or_else(|| CodegenError::LlvmError("No current function for slice".to_string()))?;

        let arr_val = self.generate_expr(arr)?;

        // Determine if the source is a Slice/SliceMut (fat pointer struct with 2 fields)
        let is_slice_source = if let BasicValueEnum::StructValue(sv) = arr_val {
            let struct_type = sv.get_type();
            struct_type.count_fields() == 2
        } else {
            false
        };

        let arr_ptr = if is_slice_source {
            // Extract data pointer from fat pointer (field 0)
            let data_ptr = self
                .builder
                .build_extract_value(arr_val.into_struct_value(), 0, "slice_data")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_pointer_value();
            // Bitcast i8* to i64*
            self.builder
                .build_pointer_cast(
                    data_ptr,
                    self.context.i64_type().ptr_type(AddressSpace::default()),
                    "slice_ptr_typed",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        } else {
            arr_val.into_pointer_value()
        };

        // Get start index (default 0)
        let start_val = if let Some(start_expr) = start {
            self.generate_expr(&start_expr.node)?.into_int_value()
        } else {
            self.context.i64_type().const_int(0, false)
        };

        // Get end index
        let end_val = if let Some(end_expr) = end {
            let val = self.generate_expr(&end_expr.node)?.into_int_value();
            if inclusive {
                self.builder
                    .build_int_add(val, self.context.i64_type().const_int(1, false), "incl_end")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                val
            }
        } else {
            // Open-end slice: arr[start..]
            if is_slice_source {
                // Extract length from fat pointer (field 1)
                self.builder
                    .build_extract_value(arr_val.into_struct_value(), 1, "slice_len")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_int_value()
            } else {
                // Array/Pointer source doesn't have length information
                return Err(CodegenError::Unsupported(
                    "Open-end slicing requires a slice source; array length is unknown".to_string(),
                ));
            }
        };

        // Calculate slice length: end - start
        let length = self
            .builder
            .build_int_sub(end_val, start_val, "slice_len")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Allocate new array: malloc(length * 8)
        let byte_size = self
            .builder
            .build_int_mul(
                length,
                self.context.i64_type().const_int(8, false),
                "byte_size",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let malloc_fn = self.module.get_function("malloc").unwrap_or_else(|| {
            let fn_type = self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .fn_type(&[self.context.i64_type().into()], false);
            self.module.add_function("malloc", fn_type, None)
        });

        let raw_ptr = self
            .builder
            .build_call(malloc_fn, &[byte_size.into()], "slice_raw")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| {
                CodegenError::LlvmError("ICE: malloc call returned void instead of pointer".into())
            })?;
        // Track allocation via entry-block alloca slot to avoid dominance issues in loops.
        {
            let current_fn = self
                .builder
                .get_insert_block()
                .unwrap()
                .get_parent()
                .unwrap();
            let entry_block = current_fn.get_first_basic_block().unwrap();
            let current_block = self.builder.get_insert_block().unwrap();
            if let Some(terminator) = entry_block.get_terminator() {
                self.builder.position_before(&terminator);
            } else {
                self.builder.position_at_end(entry_block);
            }
            let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
            let alloc_slot = self
                .builder
                .build_alloca(
                    ptr_type,
                    &format!("__slice_alloc_slot_{}", self.alloc_tracker.len()),
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(alloc_slot, ptr_type.const_null())
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder.position_at_end(current_block);
            self.builder
                .build_store(alloc_slot, raw_ptr.into_pointer_value())
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.alloc_tracker.push(alloc_slot);
        }
        let slice_ptr = self
            .builder
            .build_pointer_cast(
                raw_ptr.into_pointer_value(),
                self.context.i64_type().ptr_type(AddressSpace::default()),
                "slice_ptr",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Copy elements using a loop
        let loop_var = self
            .builder
            .build_alloca(self.context.i64_type(), "slice_i")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(loop_var, self.context.i64_type().const_int(0, false))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let loop_cond = self.context.append_basic_block(fn_value, "slice_cond");
        let loop_body = self.context.append_basic_block(fn_value, "slice_body");
        let loop_end = self.context.append_basic_block(fn_value, "slice_end");

        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Loop condition: i < length
        self.builder.position_at_end(loop_cond);
        let i = self
            .builder
            .build_load(self.context.i64_type(), loop_var, "i")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();
        let cmp = self
            .builder
            .build_int_compare(IntPredicate::SLT, i, length, "slice_cmp")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_conditional_branch(cmp, loop_body, loop_end)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Loop body: slice_ptr[i] = arr_ptr[start + i]
        self.builder.position_at_end(loop_body);
        let src_idx = self
            .builder
            .build_int_add(start_val, i, "src_idx")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        // SAFETY: src_idx = start + i, bounded by slice length validation in the loop condition.
        // arr_ptr is a valid array pointer from a prior allocation.
        let src_ptr = unsafe {
            self.builder
                .build_gep(self.context.i64_type(), arr_ptr, &[src_idx], "src_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };
        let elem = self
            .builder
            .build_load(self.context.i64_type(), src_ptr, "elem")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        // SAFETY: dst index i is bounded by slice_len (loop condition), matching the malloc'd buffer.
        let dst_ptr = unsafe {
            self.builder
                .build_gep(self.context.i64_type(), slice_ptr, &[i], "dst_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };
        self.builder
            .build_store(dst_ptr, elem)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // i++
        let next_i = self
            .builder
            .build_int_add(i, self.context.i64_type().const_int(1, false), "next_i")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(loop_var, next_i)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // After loop
        self.builder.position_at_end(loop_end);
        Ok(slice_ptr.into())
    }

    // ========== Method Call ==========

    pub(super) fn generate_method_call(
        &mut self,
        receiver: &Expr,
        method: &str,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Transform method call to function call with receiver as first arg
        // e.g., obj.method(a, b) -> TypeName_method(obj, a, b)

        // Special case: Slice.len() / SliceMut.len() — fat pointer { i8*, i64 }
        // Field 1 is the length.  Restrict to known slice types to avoid triggering on
        // other 2-field structs such as Result or Optional.
        if method == "len" {
            // Check whether the receiver's resolved type is a Slice/SliceMut before
            // generating the expression value (avoids side-effect duplication).
            let is_slice_receiver = match receiver {
                Expr::Ident(name) => {
                    // Check if the variable's type name indicates a slice
                    let type_name = self.var_struct_types.get(name).map(|s| s.as_str());
                    matches!(type_name, Some("Slice") | Some("SliceMut"))
                        || self.locals.get(name).is_some_and(|(_, ty)| {
                            // Also accept if the LLVM type is { ptr, i64 } and is NOT a
                            // known named struct (i.e. an anonymous fat-pointer struct)
                            if let inkwell::types::BasicTypeEnum::StructType(st) = ty {
                                let nf = st.count_fields();
                                if nf == 2 {
                                    // Must not be a named struct registered in generated_structs
                                    !self.generated_structs.values().any(|known| known == st)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        })
                }
                _ => false,
            };

            if is_slice_receiver {
                let recv_val = self.generate_expr(receiver)?;
                if recv_val.is_struct_value() {
                    let struct_val = recv_val.into_struct_value();
                    if struct_val.get_type().count_fields() == 2 {
                        // Extract field 1 (the length i64)
                        let len_val = self
                            .builder
                            .build_extract_value(struct_val, 1, "slice_len")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        return Ok(len_val);
                    }
                }
            }
        }

        // Try to resolve the struct type name from the receiver
        let mut struct_name = self.infer_struct_name(receiver).ok();

        // For SelfCall (@), infer struct type from current function name (TypeName_method pattern)
        if struct_name.is_none() && matches!(receiver, Expr::SelfCall) {
            if let Some(func) = self.current_function {
                let fn_name = func.get_name().to_str().unwrap_or("").to_string();
                if let Some(idx) = fn_name.find('_') {
                    struct_name = Some(fn_name[..idx].to_string());
                }
            }
        }

        // Get receiver pointer for pass-by-reference self parameter
        let receiver_ptr: Option<PointerValue<'ctx>> = match receiver {
            Expr::Ident(name) => self.locals.get(name).map(|(ptr, _)| *ptr),
            Expr::SelfCall => {
                // @ in method context: self is already a pointer
                self.locals.get("self").map(|(ptr, _)| *ptr)
            }
            _ => None,
        };

        // Also generate the receiver value as fallback (for non-method calls or unknown receivers)
        let receiver_val = self.generate_expr(receiver)?;

        // B.2: str method dispatch (inkwell backend). Receiver types covered:
        //   - `ResolvedType::Str` via `var_resolved_types` (for Ident receivers)
        //   - LLVM { i8*, i64 } fat pointer at the value level (all other cases —
        //     string literals, function returns, etc.)
        // Methods dispatched inline (no TypeName_method lookup):
        //   - char_at(i) → i8 at ptr+i, zext to i64
        //   - parse_f64() / parse_f32() → strtod/strtof + Result<f{64,32}, str>
        //   - parse_i64() / parse_i32() → strtoll + Result<i{64,32}, str>
        //   - parse_u64() / parse_u32() → strtoull + Result<u{64,32}, str>
        // The text-IR backend has (now unreachable) inline copies in string_ops.rs;
        // this is the live path for `vaisc build` which defaults to inkwell.
        let receiver_is_str = match receiver {
            Expr::Ident(name) => matches!(
                self.var_resolved_types.get(name),
                Some(vais_types::ResolvedType::Str)
            ),
            _ => false,
        } || Self::is_str_fat_pointer(&receiver_val);
        if receiver_is_str {
            if let Some(result) = self.try_generate_str_method(&receiver_val, method, args)? {
                return Ok(result);
            }
        }

        // Try qualified name: TypeName_method
        let qualified_name = struct_name.as_ref().map(|sn| format!("{}_{}", sn, method));

        let fn_value = qualified_name
            .as_ref()
            .and_then(|qn| {
                self.functions
                    .get(qn)
                    .copied()
                    .or_else(|| self.module.get_function(qn))
            })
            // Fallback: try bare method name
            .or_else(|| {
                self.functions
                    .get(method)
                    .copied()
                    .or_else(|| self.module.get_function(method))
            });

        // If not found, try broader search: look for any TypeName_method pattern.
        // When the receiver struct name is known, use it directly (deterministic).
        // Only fall back to iterating all structs when the name is unknown.
        let fn_value = if let Some(f) = fn_value {
            f
        } else if let Some(ref sn) = struct_name {
            // Struct name is known — avoid non-deterministic HashMap iteration.
            // The qualified name was already tried above; nothing more to do.
            let tried = format!("{}_{}", sn, method);
            return Err(CodegenError::UndefinedFunction(format!(
                "{} (method call on {:?})",
                tried, receiver
            )));
        } else {
            // Struct name unknown — scan all registered structs for a matching method.
            // Collect into a sorted Vec first to make the search order deterministic.
            //
            // F-23 BUG (STEP7_FINDINGS, 2026-05-04): when the receiver is a
            // `&dyn Trait` / `&mut dyn Trait` parameter, this loop silently
            // binds to the alphabetically-first impl whose method name
            // matches, producing wrong runtime values across cross-impl
            // dispatch. The intended behavior is vtable-indirected dispatch
            // via vtable.rs::generate_dynamic_call; that integration is the
            // A4-12 reclass round work. A guard for this site was prototyped
            // but did not trigger because dyn-parameter receivers don't
            // populate `var_resolved_types` consistently; the proper fix
            // requires plumbing receiver-type info from the type checker.
            // See STEP7_FINDINGS F-23 for empirical evidence.
            let mut candidates: Vec<String> = self.generated_structs.keys().cloned().collect();
            candidates.sort();
            let mut found = None;
            for sn in &candidates {
                let candidate = format!("{}_{}", sn, method);
                if let Some(f) = self
                    .functions
                    .get(&candidate)
                    .copied()
                    .or_else(|| self.module.get_function(&candidate))
                {
                    found = Some(f);
                    break;
                }
            }
            if let Some(f) = found {
                f
            } else {
                let tried = qualified_name.as_deref().unwrap_or(method);
                return Err(CodegenError::UndefinedFunction(format!(
                    "{} (method call on {:?})",
                    tried, receiver
                )));
            }
        };

        // Generate arguments (receiver first, pass as pointer for methods)
        let mut arg_values: Vec<BasicMetadataValueEnum> = if let Some(ptr) = receiver_ptr {
            // Pass receiver as pointer (self by reference)
            vec![ptr.into()]
        } else {
            // Fallback: for struct literal receivers or complex expressions,
            // create a temporary alloca and pass its pointer
            if struct_name.is_some() {
                let alloca = self
                    .builder
                    .build_alloca(receiver_val.get_type(), "tmp_self")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder
                    .build_store(alloca, receiver_val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                vec![alloca.into()]
            } else {
                vec![receiver_val.into()]
            }
        };
        for arg in args {
            arg_values.push(self.generate_expr(&arg.node)?.into());
        }

        // Build call
        let call = self
            .builder
            .build_call(fn_value, &arg_values, "method_call")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(call
            .try_as_basic_value()
            .left()
            .unwrap_or_else(|| self.unit_value()))
    }

    // ========== Lambda/Closure ==========

    pub(super) fn generate_lambda(
        &mut self,
        params: &[ast::Param],
        body: &Expr,
        captures: &[String],
        capture_mode: ast::CaptureMode,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Generate unique lambda function name
        let lambda_name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        // Find captured variables from current scope
        // If captures list is empty (type checker didn't fill it), auto-detect from body
        let effective_captures: Vec<String> = if captures.is_empty() {
            let param_names: std::collections::HashSet<String> =
                params.iter().map(|p| p.name.node.clone()).collect();
            let used_idents = Self::collect_idents(body);
            used_idents
                .into_iter()
                .filter(|name| !param_names.contains(name) && self.locals.contains_key(name))
                .collect()
        } else {
            captures.to_vec()
        };

        let is_ref_capture = matches!(
            capture_mode,
            ast::CaptureMode::ByRef | ast::CaptureMode::ByMutRef
        );

        let mut captured_vars: Vec<(String, BasicValueEnum<'ctx>, BasicTypeEnum<'ctx>)> =
            Vec::new();
        for cap_name in &effective_captures {
            if let Some((ptr, var_type)) = self.locals.get(cap_name) {
                if is_ref_capture {
                    // ByRef/ByMutRef: pass the alloca pointer directly
                    // The pointer value itself is the captured value
                    captured_vars.push((
                        cap_name.clone(),
                        (*ptr).into(),
                        (*var_type)
                            .ptr_type(inkwell::AddressSpace::default())
                            .into(),
                    ));
                } else {
                    // By-value or explicit move: load and pass the value
                    let val = self
                        .builder
                        .build_load(*var_type, *ptr, &format!("cap_{}", cap_name))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    captured_vars.push((cap_name.clone(), val, *var_type));
                }
            }
        }

        // Build parameter types: captured vars first, then lambda params
        let mut param_types: Vec<BasicMetadataTypeEnum> = Vec::new();

        // First add captured variables as parameters
        for (_, _, cap_type) in &captured_vars {
            param_types.push((*cap_type).into());
        }

        // Then add original lambda parameters
        for p in params {
            let resolved = self.ast_type_to_resolved(&p.ty.node);
            param_types.push(self.type_mapper.map_type(&resolved).into());
        }

        // Create function type (always returns i64 for now)
        let fn_type = self.context.i64_type().fn_type(&param_types, false);
        let lambda_fn = self.module.add_function(&lambda_name, fn_type, None);
        self.lambda_functions.push(lambda_fn);

        // Save current state (move instead of clone to avoid HashMap allocation).
        // SAFETY: if generate_expr below returns Err, the entire codegen aborts,
        // so empty self.locals after take is acceptable (never accessed post-error).
        let saved_function = self.current_function;
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_insert_block = self.builder.get_insert_block();

        // Set up lambda context
        self.current_function = Some(lambda_fn);

        // Create entry block for lambda
        let entry = self.context.append_basic_block(lambda_fn, "entry");
        self.builder.position_at_end(entry);

        // Register captured variables as parameters in lambda scope
        let mut param_idx = 0u32;
        for (cap_name, _, cap_type) in &captured_vars {
            let param_val = lambda_fn.get_nth_param(param_idx).ok_or_else(|| {
                CodegenError::InternalError(format!(
                    "ICE: captured variable '{}' parameter index {} out of bounds for lambda",
                    cap_name, param_idx
                ))
            })?;
            if is_ref_capture {
                // ByRef/ByMutRef: parameter is already a pointer to the outer alloca.
                // Use it directly — build_load will read from the outer variable.
                let ptr_val = param_val.into_pointer_value();
                // Get the original value type from saved locals
                let val_type = saved_locals
                    .get(cap_name)
                    .map(|(_, t)| *t)
                    .unwrap_or_else(|| self.context.i64_type().into());
                self.locals.insert(cap_name.clone(), (ptr_val, val_type));
            } else {
                let alloca = self
                    .builder
                    .build_alloca(*cap_type, &format!("__cap_{}", cap_name))
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder
                    .build_store(alloca, param_val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.locals.insert(cap_name.clone(), (alloca, *cap_type));
            }
            param_idx += 1;
        }

        // Register original parameters
        for p in params {
            let param_val = lambda_fn.get_nth_param(param_idx).ok_or_else(|| {
                CodegenError::InternalError(format!(
                    "ICE: lambda parameter '{}' index {} out of bounds",
                    p.name.node, param_idx
                ))
            })?;
            let ty = self.ast_type_to_resolved(&p.ty.node);
            let param_type = self.type_mapper.map_type(&ty);
            let alloca = self
                .builder
                .build_alloca(param_type, &p.name.node)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals
                .insert(p.name.node.clone(), (alloca, param_type));
            param_idx += 1;
        }

        // Generate lambda body
        let body_val = self.generate_expr(body)?;

        // Lambda signature is fixed to `i64` return (see fn_type above) so all
        // callsites can use `call i64 %lambda(...)`. When the body is unit
        // (e.g. `|| puts("...")` whose last expression is a side-effect call),
        // body_val is a non-i64 aggregate — coerce to `i64 0` so the return
        // instruction matches the function signature.
        let i64_ty = self.context.i64_type();
        let ret_val: BasicValueEnum<'ctx> = if body_val.get_type() == i64_ty.into() {
            body_val
        } else {
            i64_ty.const_zero().into()
        };

        // Add return
        self.builder
            .build_return(Some(&ret_val))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Restore context
        self.current_function = saved_function;
        self.locals = saved_locals;
        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }

        // Register lambda as a callable function
        self.functions.insert(lambda_name.clone(), lambda_fn);

        // Store captured values for later use at call sites
        let captured_for_binding: Vec<(String, BasicValueEnum<'ctx>)> = captured_vars
            .iter()
            .map(|(name, val, _)| (name.clone(), *val))
            .collect();

        // Store the last lambda info so Stmt::Let can track it
        self._last_lambda_info = Some((lambda_name.clone(), captured_for_binding));

        // Return function pointer as i64
        let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
        let fn_int = self
            .builder
            .build_ptr_to_int(fn_ptr, self.context.i64_type(), "lambda_ptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(fn_int.into())
    }

    /// Collect all Ident names used in an expression (for auto-capture detection)
    pub(super) fn collect_idents(expr: &Expr) -> Vec<String> {
        let mut idents = Vec::new();
        Self::collect_idents_inner(expr, &mut idents);
        idents.sort();
        idents.dedup();
        idents
    }

    pub(super) fn collect_idents_inner(expr: &Expr, idents: &mut Vec<String>) {
        match expr {
            Expr::Ident(name) => idents.push(name.clone()),
            Expr::Binary { left, right, .. } => {
                Self::collect_idents_inner(&left.node, idents);
                Self::collect_idents_inner(&right.node, idents);
            }
            Expr::Unary { expr, .. } => Self::collect_idents_inner(&expr.node, idents),
            Expr::Call { func, args } => {
                Self::collect_idents_inner(&func.node, idents);
                for arg in args {
                    Self::collect_idents_inner(&arg.node, idents);
                }
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    if let Stmt::Expr(e) = &stmt.node {
                        Self::collect_idents_inner(&e.node, idents);
                    }
                }
            }
            Expr::If { cond, then, else_ } => {
                Self::collect_idents_inner(&cond.node, idents);
                for stmt in then {
                    if let Stmt::Expr(e) = &stmt.node {
                        Self::collect_idents_inner(&e.node, idents);
                    }
                }
                if let Some(else_branch) = else_ {
                    match else_branch {
                        IfElse::ElseIf(cond_expr, then_stmts, _else_opt) => {
                            Self::collect_idents_inner(&cond_expr.node, idents);
                            for stmt in then_stmts {
                                if let Stmt::Expr(e) = &stmt.node {
                                    Self::collect_idents_inner(&e.node, idents);
                                }
                            }
                        }
                        IfElse::Else(stmts) => {
                            for stmt in stmts {
                                if let Stmt::Expr(e) = &stmt.node {
                                    Self::collect_idents_inner(&e.node, idents);
                                }
                            }
                        }
                    }
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                Self::collect_idents_inner(&receiver.node, idents);
                for arg in args {
                    Self::collect_idents_inner(&arg.node, idents);
                }
            }
            Expr::Field { expr, .. } => Self::collect_idents_inner(&expr.node, idents),
            Expr::Index { expr, index } => {
                Self::collect_idents_inner(&expr.node, idents);
                Self::collect_idents_inner(&index.node, idents);
            }
            Expr::Tuple(elems) | Expr::Array(elems) => {
                for e in elems {
                    Self::collect_idents_inner(&e.node, idents);
                }
            }
            Expr::StringInterp(parts) => {
                // Interpolated `{name}` references must be surfaced as captures.
                for part in parts {
                    if let vais_ast::StringInterpPart::Expr(e) = part {
                        Self::collect_idents_inner(&e.node, idents);
                    }
                }
            }
            _ => {}
        }
    }

    // ========== B.2: str method dispatch (inkwell backend) ==========

    /// Detect a value that looks like the str fat-pointer ABI `{ i8*, i64 }`.
    /// Used as a fallback when var_resolved_types doesn't classify the receiver
    /// as Str (e.g., string literals, function returns, complex expressions).
    fn is_str_fat_pointer(v: &BasicValueEnum<'_>) -> bool {
        if !v.is_struct_value() {
            return false;
        }
        let st = v.into_struct_value().get_type();
        if st.count_fields() != 2 {
            return false;
        }
        let f0 = st.get_field_type_at_index(0);
        let f1 = st.get_field_type_at_index(1);
        matches!(f0, Some(t) if t.is_pointer_type())
            && matches!(f1, Some(t) if t.is_int_type() && t.into_int_type().get_bit_width() == 64)
    }

    /// Try to lower `<str>.method(args)` for a known builtin method. Returns
    /// `Ok(Some(value))` on a hit, `Ok(None)` if `method` is not a recognized
    /// str builtin (caller falls back to TypeName_method lookup).
    fn try_generate_str_method(
        &mut self,
        recv: &BasicValueEnum<'ctx>,
        method: &str,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<Option<BasicValueEnum<'ctx>>> {
        match method {
            "char_at" | "charAt" => Ok(Some(self.str_method_char_at(recv, args)?)),
            "parse_i64" | "parse_int" => {
                Ok(Some(self.str_method_parse_int(
                    recv, /*signed=*/ true, /*bits=*/ 64,
                )?))
            }
            "parse_i32" => Ok(Some(self.str_method_parse_int(recv, true, 32)?)),
            "parse_u64" => Ok(Some(self.str_method_parse_int(recv, false, 64)?)),
            "parse_u32" => Ok(Some(self.str_method_parse_int(recv, false, 32)?)),
            "parse_f64" => Ok(Some(self.str_method_parse_float(recv, /*bits=*/ 64)?)),
            "parse_f32" => Ok(Some(self.str_method_parse_float(recv, 32)?)),
            _ => Ok(None),
        }
    }

    /// `s.char_at(i) -> u8` (typed as u8 by TC; we return as i8 — promotion to
    /// i64 in `as` casts is handled by the cast layer).
    fn str_method_char_at(
        &mut self,
        recv: &BasicValueEnum<'ctx>,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        if args.is_empty() {
            return Err(CodegenError::Unsupported(
                "char_at requires 1 argument".to_string(),
            ));
        }
        let raw_ptr = self.extract_str_raw_ptr(*recv)?;
        let idx_val = self.generate_expr(&args[0].node)?;
        let idx_i64 = self.coerce_to_i64(idx_val)?;
        let i8_ty = self.context.i8_type();
        // SAFETY: `raw_ptr` is extracted from the canonical Vais string
        // `{ptr,len}` representation. `char_at` callers are responsible for
        // bounds validation before indexing into the byte buffer.
        let elem_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(i8_ty, raw_ptr, &[idx_i64], "char_at_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };
        let byte = self
            .builder
            .build_load(i8_ty, elem_ptr, "char_at_byte")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(byte)
    }

    /// `s.parse_iN()` / `s.parse_uN()` → `Result<iN/uN, str>` via strtoll/strtoull.
    /// Success criterion: end pointer landed past the start AND points at NUL
    /// (full string consumed). Otherwise return `Err("parse error")`.
    fn str_method_parse_int(
        &mut self,
        recv: &BasicValueEnum<'ctx>,
        signed: bool,
        bits: u32,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        use inkwell::AddressSpace;
        let i8_ty = self.context.i8_type();
        let i32_ty = self.context.i32_type();
        let i64_ty = self.context.i64_type();
        let i8_ptr_ty = i8_ty.ptr_type(AddressSpace::default());
        let i8_ptr_ptr_ty = i8_ptr_ty.ptr_type(AddressSpace::default());

        // Declare extern long long strtoll(const char*, char**, int)
        // or unsigned long long strtoull(...).
        let extern_name = if signed { "strtoll" } else { "strtoull" };
        let strto_fn = self.module.get_function(extern_name).unwrap_or_else(|| {
            let fn_ty = i64_ty.fn_type(
                &[i8_ptr_ty.into(), i8_ptr_ptr_ty.into(), i32_ty.into()],
                false,
            );
            self.module.add_function(extern_name, fn_ty, None)
        });

        let raw_ptr = self.extract_str_raw_ptr(*recv)?;
        let endptr_alloca = self
            .builder
            .build_alloca(i8_ptr_ty, "parse_endptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let base = i32_ty.const_int(10, false);
        let parsed_full = self
            .builder
            .build_call(
                strto_fn,
                &[raw_ptr.into(), endptr_alloca.into(), base.into()],
                "parse_int_call",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| CodegenError::LlvmError("strtoll returned no value".into()))?
            .into_int_value();

        // Truncate / extend to requested bit width if not 64.
        let parsed = if bits == 64 {
            parsed_full
        } else {
            let target_ty = self.context.custom_width_int_type(bits);
            self.builder
                .build_int_truncate(parsed_full, target_ty, "parse_int_trunc")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };

        // Success check: endptr > raw_ptr AND *endptr == 0 (NUL terminator —
        // caller must trust the str is NUL-terminated, which all Vais string
        // literals are).
        let endptr_loaded = self
            .builder
            .build_load(i8_ptr_ty, endptr_alloca, "parse_endptr_loaded")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_pointer_value();
        let end_int = self
            .builder
            .build_ptr_to_int(endptr_loaded, i64_ty, "parse_end_int")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let raw_int = self
            .builder
            .build_ptr_to_int(raw_ptr, i64_ty, "parse_raw_int")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let advanced = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::UGT,
                end_int,
                raw_int,
                "parse_advanced",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let last_byte = self
            .builder
            .build_load(i8_ty, endptr_loaded, "parse_last_byte")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();
        let consumed_all = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::EQ,
                last_byte,
                i8_ty.const_int(0, false),
                "parse_consumed_all",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let success = self
            .builder
            .build_and(advanced, consumed_all, "parse_success")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Pack payload as i64 for the Result { i8 tag, i64 payload } ABI (B.1).
        let payload_ok = if bits == 64 {
            parsed
        } else {
            // Zero/sign-extend back to i64 for the payload slot.
            if signed {
                self.builder
                    .build_int_s_extend(parsed, i64_ty, "parse_ok_sext")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                self.builder
                    .build_int_z_extend(parsed, i64_ty, "parse_ok_zext")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            }
        };
        let err_payload_i64 = self.build_static_err_payload_i64("parse error")?;
        let chosen_payload = self
            .builder
            .build_select(success, payload_ok, err_payload_i64, "parse_payload")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();
        // Tag: success → 0 (Ok), failure → 1 (Err).
        let tag = self
            .builder
            .build_select(
                success,
                i8_ty.const_int(0, false),
                i8_ty.const_int(1, false),
                "parse_tag",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();
        self.assemble_result_struct(tag, chosen_payload)
    }

    /// `s.parse_fN()` → `Result<fN, str>` via strtod/strtof.
    /// Success criterion mirrors parse_int: full string consumed.
    fn str_method_parse_float(
        &mut self,
        recv: &BasicValueEnum<'ctx>,
        bits: u32,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        use inkwell::AddressSpace;
        let i8_ty = self.context.i8_type();
        let i64_ty = self.context.i64_type();
        let f64_ty = self.context.f64_type();
        let f32_ty = self.context.f32_type();
        let i8_ptr_ty = i8_ty.ptr_type(AddressSpace::default());
        let i8_ptr_ptr_ty = i8_ptr_ty.ptr_type(AddressSpace::default());

        let raw_ptr = self.extract_str_raw_ptr(*recv)?;
        let endptr_alloca = self
            .builder
            .build_alloca(i8_ptr_ty, "parse_endptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Always go through strtod (returns f64), then convert. f32 path uses
        // fptrunc — bounds-clamp via float comparison would over-engineer the
        // simple case; tolerate inf/-inf for now.
        let strtod_fn = self.module.get_function("strtod").unwrap_or_else(|| {
            let fn_ty = f64_ty.fn_type(&[i8_ptr_ty.into(), i8_ptr_ptr_ty.into()], false);
            self.module.add_function("strtod", fn_ty, None)
        });
        let parsed_f64 = self
            .builder
            .build_call(
                strtod_fn,
                &[raw_ptr.into(), endptr_alloca.into()],
                "parse_f_call",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| CodegenError::LlvmError("strtod returned no value".into()))?
            .into_float_value();

        // Success: endptr advanced + landed on NUL.
        let endptr_loaded = self
            .builder
            .build_load(i8_ptr_ty, endptr_alloca, "parse_endptr_loaded")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_pointer_value();
        let end_int = self
            .builder
            .build_ptr_to_int(endptr_loaded, i64_ty, "parse_end_int")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let raw_int = self
            .builder
            .build_ptr_to_int(raw_ptr, i64_ty, "parse_raw_int")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let advanced = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::UGT,
                end_int,
                raw_int,
                "parse_advanced",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let last_byte = self
            .builder
            .build_load(i8_ty, endptr_loaded, "parse_last_byte")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();
        let consumed_all = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::EQ,
                last_byte,
                i8_ty.const_int(0, false),
                "parse_consumed_all",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let success = self
            .builder
            .build_and(advanced, consumed_all, "parse_success")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Pack payload as i64. f64 → bitcast i64. f32 → fptrunc → bitcast i32 → zext i64.
        let payload_ok = if bits == 64 {
            self.builder
                .build_bitcast(parsed_f64, i64_ty, "parse_ok_f64_bits")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_int_value()
        } else {
            let parsed_f32 = self
                .builder
                .build_float_trunc(parsed_f64, f32_ty, "parse_f32_trunc")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            let bits_i32 = self
                .builder
                .build_bitcast(parsed_f32, self.context.i32_type(), "parse_ok_f32_bits")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_int_value();
            self.builder
                .build_int_z_extend(bits_i32, i64_ty, "parse_ok_f32_to_i64")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };
        let err_payload_i64 = self.build_static_err_payload_i64("parse error")?;
        let chosen_payload = self
            .builder
            .build_select(success, payload_ok, err_payload_i64, "parse_payload")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();
        let tag = self
            .builder
            .build_select(
                success,
                i8_ty.const_int(0, false),
                i8_ty.const_int(1, false),
                "parse_tag",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();
        self.assemble_result_struct(tag, chosen_payload)
    }

    /// Pack a static `str` constant (always "parse error" today) into the
    /// Result-payload i64 slot. The Err branch uses this when parsing fails.
    /// Layout: heap-alloc'd `{ i8*, i64 }` fat pointer, ptrtoint to i64.
    /// Heap allocation matches B.1's >8B struct path so the match-arm decoder
    /// recovers the str via int_to_ptr+load.
    fn build_static_err_payload_i64(
        &mut self,
        msg: &str,
    ) -> CodegenResult<inkwell::values::IntValue<'ctx>> {
        use inkwell::AddressSpace;
        let i8_ty = self.context.i8_type();
        let i64_ty = self.context.i64_type();
        let i8_ptr_ty = i8_ty.ptr_type(AddressSpace::default());

        // Build a global string constant for the message, then assemble a fat
        // pointer struct on the heap.
        let str_global = self
            .builder
            .build_global_string_ptr(msg, "parse_err_msg")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .as_pointer_value();
        let len = i64_ty.const_int(msg.len() as u64, false);
        let fat_ty = self
            .context
            .struct_type(&[i8_ptr_ty.into(), i64_ty.into()], false);
        let mut fat_undef = fat_ty.get_undef();
        fat_undef = self
            .builder
            .build_insert_value(fat_undef, str_global, 0, "parse_err_fat_p")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();
        fat_undef = self
            .builder
            .build_insert_value(fat_undef, len, 1, "parse_err_fat_l")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();

        // Heap-allocate via malloc(16) and store, then ptrtoint to i64. Matches
        // B.1's >8B struct payload path; gen_match.rs's payload decoder will
        // int_to_ptr + load symmetrically.
        let malloc_fn = self.module.get_function("malloc").unwrap_or_else(|| {
            let malloc_ty = i8_ptr_ty.fn_type(&[i64_ty.into()], false);
            self.module.add_function("malloc", malloc_ty, None)
        });
        let size = i64_ty.const_int(16, false);
        let heap_ptr = self
            .builder
            .build_call(malloc_fn, &[size.into()], "parse_err_heap")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let typed_ptr = self
            .builder
            .build_bitcast(
                heap_ptr,
                fat_ty.ptr_type(AddressSpace::default()),
                "parse_err_heap_typed",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_pointer_value();
        self.builder
            .build_store(typed_ptr, fat_undef)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_ptr_to_int(heap_ptr, i64_ty, "parse_err_payload")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    /// Assemble the Result `{ i8 tag, i64 payload }` value used by B.1's ABI.
    fn assemble_result_struct(
        &mut self,
        tag: inkwell::values::IntValue<'ctx>,
        payload: inkwell::values::IntValue<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let i8_ty = self.context.i8_type();
        let i64_ty = self.context.i64_type();
        let result_ty = self
            .context
            .struct_type(&[i8_ty.into(), i64_ty.into()], false);
        let mut val = result_ty.get_undef();
        val = self
            .builder
            .build_insert_value(val, tag, 0, "result_tag")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();
        val = self
            .builder
            .build_insert_value(val, payload, 1, "result_payload")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();
        Ok(val.into())
    }
}
