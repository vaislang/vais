//! Lambda and lazy evaluation generation.

use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

use vais_ast::Expr;

use super::super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    /// Generate lazy expression: creates a thunk function and returns { i1, T, ptr } struct
    pub(super) fn generate_lazy(&mut self, inner: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        // For Inkwell, we generate the thunk function and build the lazy struct.
        // The thunk captures free variables from the current scope.

        let thunk_name = format!("__lazy_thunk_{}", self.lambda_counter);
        self.lambda_counter += 1;

        // Find captured variables
        let used_idents = Self::collect_idents(inner);
        let captured_vars: Vec<(
            String,
            BasicValueEnum<'ctx>,
            inkwell::types::BasicTypeEnum<'ctx>,
        )> = used_idents
            .iter()
            .filter_map(|name| {
                self.locals.get(name).map(|(ptr, var_type)| {
                    let val = self
                        .builder
                        .build_load(*var_type, *ptr, &format!("lazy_cap_{}", name))
                        .unwrap_or_else(|_| self.context.i64_type().const_int(0, false).into());
                    (name.clone(), val, *var_type)
                })
            })
            .collect();

        // Build thunk parameter types (captured vars only)
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = captured_vars
            .iter()
            .map(|(_, _, cap_type)| (*cap_type).into())
            .collect();

        // Infer the inner type (approximate: use i64 as fallback)
        let inner_type = self.context.i64_type();
        let fn_type = inner_type.fn_type(&param_types, false);
        let thunk_fn = self.module.add_function(&thunk_name, fn_type, None);

        // Save state
        let saved_function = self.current_function;
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_insert_block = self.builder.get_insert_block();

        // Set up thunk context
        self.current_function = Some(thunk_fn);
        let entry = self.context.append_basic_block(thunk_fn, "entry");
        self.builder.position_at_end(entry);

        // Register captured vars as params in thunk
        for (idx, (cap_name, _, cap_type)) in captured_vars.iter().enumerate() {
            let param_val = thunk_fn.get_nth_param(idx as u32).unwrap();
            let alloca = self
                .builder
                .build_alloca(*cap_type, &format!("__cap_{}", cap_name))
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals.insert(cap_name.clone(), (alloca, *cap_type));
        }

        // Generate thunk body
        let body_val = self.generate_expr(inner)?;
        self.builder
            .build_return(Some(&body_val))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Restore context
        self.current_function = saved_function;
        self.locals = saved_locals;
        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }
        self.functions.insert(thunk_name.clone(), thunk_fn);

        // Build lazy struct: { i1 false, T zeroinit, ptr thunk_fn }
        let bool_type = self.context.bool_type();
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let lazy_struct_type = self.context.struct_type(
            &[bool_type.into(), inner_type.into(), ptr_type.into()],
            false,
        );

        let lazy_alloca = self
            .builder
            .build_alloca(lazy_struct_type, "lazy_struct")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store computed = false
        let computed_ptr = self
            .builder
            .build_struct_gep(lazy_struct_type, lazy_alloca, 0, "computed_ptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(computed_ptr, bool_type.const_int(0, false))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store zero value
        let value_ptr = self
            .builder
            .build_struct_gep(lazy_struct_type, lazy_alloca, 1, "value_ptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(value_ptr, inner_type.const_int(0, false))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store thunk function pointer (cast to i8*)
        let thunk_ptr_val = thunk_fn.as_global_value().as_pointer_value();
        let thunk_as_i8_ptr = self
            .builder
            .build_bitcast(thunk_ptr_val, ptr_type, "thunk_i8ptr")
            .map_err(|e: inkwell::builder::BuilderError| CodegenError::LlvmError(e.to_string()))?;
        let thunk_slot = self
            .builder
            .build_struct_gep(lazy_struct_type, lazy_alloca, 2, "thunk_ptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(thunk_slot, thunk_as_i8_ptr)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store captured values for force to use later
        let cap_for_binding: Vec<(String, BasicValueEnum<'ctx>)> = captured_vars
            .iter()
            .map(|(name, val, _)| (name.clone(), *val))
            .collect();
        self._last_lazy_info = Some((thunk_name, cap_for_binding));

        // Load and return the lazy struct
        let result = self
            .builder
            .build_load(lazy_struct_type, lazy_alloca, "lazy_val")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(result)
    }

    /// Generate force expression: check computed flag, call thunk if needed, cache result
    pub(super) fn generate_force(&mut self, inner: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Look up lazy binding info if the inner is an identifier
        let lazy_info = if let Expr::Ident(name) = inner {
            self.lazy_bindings.get(name).cloned()
        } else {
            None
        };

        let val = self.generate_expr(inner)?;

        // Check if the value is a lazy struct (has struct type with 3 fields)
        if !val.is_struct_value() {
            return Ok(val);
        }

        let struct_val = val.into_struct_value();
        let struct_type = struct_val.get_type();

        // Verify this is a lazy struct (3 fields: i1, T, ptr)
        if struct_type.count_fields() != 3 {
            let result = self
                .builder
                .build_extract_value(struct_val, 1, "force_val")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            return Ok(result);
        }

        // If we have thunk info, generate proper conditional evaluation
        if let Some((thunk_name, captured_vals)) = lazy_info {
            let current_fn = self
                .current_function
                .ok_or_else(|| CodegenError::LlvmError("No current function".to_string()))?;

            let bool_type = self.context.bool_type();
            let inner_type = self.context.i64_type();

            // Spill lazy struct to alloca so we can update computed/value fields
            let lazy_alloca = self
                .builder
                .build_alloca(struct_type, "lazy_spill")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(lazy_alloca, struct_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Load computed flag (field 0)
            let computed_ptr = self
                .builder
                .build_struct_gep(struct_type, lazy_alloca, 0, "computed_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            let computed_flag = self
                .builder
                .build_load(bool_type, computed_ptr, "computed")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_int_value();

            // Create basic blocks for conditional evaluation
            let cached_bb = self.context.append_basic_block(current_fn, "lazy.cached");
            let compute_bb = self.context.append_basic_block(current_fn, "lazy.compute");
            let merge_bb = self.context.append_basic_block(current_fn, "lazy.merge");

            self.builder
                .build_conditional_branch(computed_flag, cached_bb, compute_bb)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Cached path: load value from struct field 1
            self.builder.position_at_end(cached_bb);
            let value_ptr = self
                .builder
                .build_struct_gep(struct_type, lazy_alloca, 1, "cached_val_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            let cached_val = self
                .builder
                .build_load(inner_type, value_ptr, "cached_val")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Compute path: call thunk, store result, set computed=true
            self.builder.position_at_end(compute_bb);

            // Build thunk call arguments (captured values)
            let thunk_fn = self.functions.get(&thunk_name).copied().ok_or_else(|| {
                CodegenError::LlvmError(format!("Thunk function '{}' not found", thunk_name))
            })?;

            let call_args: Vec<inkwell::values::BasicMetadataValueEnum> =
                captured_vals.iter().map(|(_, val)| (*val).into()).collect();

            let computed_val = self
                .builder
                .build_call(thunk_fn, &call_args, "thunk_result")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .try_as_basic_value()
                .left()
                .unwrap_or_else(|| inner_type.const_int(0, false).into());

            // Store computed value into lazy struct
            let store_val_ptr = self
                .builder
                .build_struct_gep(struct_type, lazy_alloca, 1, "store_val_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(store_val_ptr, computed_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Set computed = true
            self.builder
                .build_store(computed_ptr, bool_type.const_int(1, false))
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Write the updated lazy struct back to the original variable's alloca so
            // subsequent force calls see the cached value.
            if let Expr::Ident(var_name) = inner {
                if let Some((orig_alloca, _)) = self.locals.get(var_name).copied() {
                    // Load the full (now-updated) lazy struct and write it back
                    let updated_struct = self
                        .builder
                        .build_load(struct_type, lazy_alloca, "lazy_updated")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    self.builder
                        .build_store(orig_alloca, updated_struct)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }
            }

            self.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Merge: phi node selects cached or computed value
            self.builder.position_at_end(merge_bb);
            let phi = self
                .builder
                .build_phi(inner_type, "force_result")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            phi.add_incoming(&[(&cached_val, cached_bb), (&computed_val, compute_bb)]);

            return Ok(phi.as_basic_value());
        }

        // Fallback: no thunk info, just extract the value field (index 1)
        let result = self
            .builder
            .build_extract_value(struct_val, 1, "force_val")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(result)
    }
}
