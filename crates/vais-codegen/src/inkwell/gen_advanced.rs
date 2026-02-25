//! Advanced expression code generation.
//!
//! Handles assignments, compound assignments, ranges, ternary operators,
//! struct literals, and field access.

use inkwell::types::BasicType;
use inkwell::values::{BasicValue, BasicValueEnum};
use inkwell::{AddressSpace, IntPredicate};

use vais_ast::{BinOp, Expr, Spanned};

use super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn generate_assign(
        &mut self,
        target: &Expr,
        value: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let val = self.generate_expr(value)?;

        match target {
            Expr::Ident(name) => {
                if let Some((ptr, _var_type)) = self.locals.get(name).cloned() {
                    self.builder
                        .build_store(ptr, val)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    Ok(val)
                } else {
                    Err(CodegenError::UndefinedVar(name.clone()))
                }
            }
            Expr::Field { expr: obj, field } => {
                // Field assignment
                let struct_name = self.infer_struct_name(&obj.node)?;
                let field_idx = self.get_field_index(&struct_name, &field.node)?;

                // Get struct pointer
                if let Expr::Ident(var_name) = &obj.node {
                    if let Some((ptr, _)) = self.locals.get(var_name).cloned() {
                        let struct_type = self
                            .generated_structs
                            .get(&struct_name)
                            .ok_or_else(|| CodegenError::UndefinedVar(struct_name.clone()))?;

                        let field_ptr = self
                            .builder
                            .build_struct_gep(*struct_type, ptr, field_idx, "field_ptr")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                        self.builder
                            .build_store(field_ptr, val)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        return Ok(val);
                    }
                }
                Err(CodegenError::Unsupported(
                    "Complex field assignment".to_string(),
                ))
            }
            Expr::Index { expr: arr, index } => {
                // Array/slice index assignment
                let arr_val = self.generate_expr(&arr.node)?;
                let idx_val = self.generate_expr(&index.node)?;
                let idx_int = idx_val.into_int_value();

                // Check if this is a slice fat pointer { ptr, i64 } — extract data pointer
                let arr_ptr = if arr_val.is_struct_value() {
                    let struct_val = arr_val.into_struct_value();
                    let struct_type = struct_val.get_type();
                    if struct_type.count_fields() == 2 {
                        // Slice fat pointer: extract field 0 (data pointer)
                        self.builder
                            .build_extract_value(struct_val, 0, "slice_data_ptr")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                            .into_pointer_value()
                    } else {
                        arr_val.into_pointer_value()
                    }
                } else if arr_val.is_pointer_value() {
                    arr_val.into_pointer_value()
                } else {
                    // Fallback: treat as pointer (e.g. i64 interpreted as ptr)
                    self.builder
                        .build_int_to_ptr(
                            arr_val.into_int_value(),
                            val.get_type().ptr_type(AddressSpace::default()),
                            "idx_assign_ptr",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                };

                // Use the stored value's type for GEP element type.
                // This is correct when val's type matches the array's element type,
                // which is the common case since you store one element at a time.
                let elem_type = val.get_type();
                let elem_ptr = unsafe {
                    self.builder
                        .build_gep(elem_type, arr_ptr, &[idx_int], "elem_ptr")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                };

                self.builder
                    .build_store(elem_ptr, val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(val)
            }
            Expr::Deref(inner) => {
                // Dereference assignment: *ptr = val
                let ptr_val = self.generate_expr(&inner.node)?;
                let ptr = if ptr_val.is_pointer_value() {
                    ptr_val.into_pointer_value()
                } else {
                    // Convert i64 to pointer
                    self.builder
                        .build_int_to_ptr(
                            ptr_val.into_int_value(),
                            val.get_type().ptr_type(AddressSpace::default()),
                            "deref_assign_ptr",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                };
                self.builder
                    .build_store(ptr, val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(val)
            }
            _ => Err(CodegenError::Unsupported("Assignment target".to_string())),
        }
    }

    pub(super) fn generate_assign_op(
        &mut self,
        op: BinOp,
        target: &Expr,
        value: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Load current value
        let current = self.generate_expr(target)?;
        let rhs = self.generate_expr(value)?;

        // Perform operation
        let result = if current.is_int_value() {
            self.generate_int_binary(op, current.into_int_value(), rhs.into_int_value())?
        } else {
            self.generate_float_binary(op, current.into_float_value(), rhs.into_float_value())?
        };

        // Store back
        if let Expr::Ident(name) = target {
            if let Some((ptr, _)) = self.locals.get(name).cloned() {
                self.builder
                    .build_store(ptr, result)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            }
        }

        Ok(result)
    }

    // ========== Range ==========

    pub(super) fn generate_range(
        &mut self,
        start: Option<&Spanned<Expr>>,
        end: Option<&Spanned<Expr>>,
        inclusive: bool,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Range is represented as a struct { start: i64, end: i64, inclusive: i1 }
        let range_type = self.context.struct_type(
            &[
                self.context.i64_type().into(),
                self.context.i64_type().into(),
                self.context.bool_type().into(),
            ],
            false,
        );

        let start_val = if let Some(s) = start {
            self.generate_expr(&s.node)?
        } else {
            self.context.i64_type().const_int(0, false).into()
        };

        let end_val = if let Some(e) = end {
            self.generate_expr(&e.node)?
        } else {
            self.context
                .i64_type()
                .const_int(i64::MAX as u64, false)
                .into()
        };

        let incl_val = self
            .context
            .bool_type()
            .const_int(u64::from(inclusive), false);

        let mut range_val = range_type.get_undef();
        range_val = self
            .builder
            .build_insert_value(range_val, start_val, 0, "range_start")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();
        range_val = self
            .builder
            .build_insert_value(range_val, end_val, 1, "range_end")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();
        range_val = self
            .builder
            .build_insert_value(range_val, incl_val, 2, "range_inclusive")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();

        Ok(range_val.into())
    }

    // ========== Ternary ==========

    pub(super) fn generate_ternary(
        &mut self,
        cond: &Expr,
        then_expr: &Expr,
        else_expr: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self.current_function.ok_or_else(|| {
            CodegenError::LlvmError("No current function for ternary".to_string())
        })?;

        let cond_val = self.generate_expr(cond)?;
        let cond_bool = if cond_val.is_int_value() {
            let int_val = cond_val.into_int_value();
            if int_val.get_type().get_bit_width() > 1 {
                self.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        int_val,
                        int_val.get_type().const_int(0, false),
                        "ternary_cond",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                int_val
            }
        } else {
            self.context.bool_type().const_int(1, false)
        };

        let then_block = self.context.append_basic_block(fn_value, "ternary_then");
        let else_block = self.context.append_basic_block(fn_value, "ternary_else");
        let merge_block = self.context.append_basic_block(fn_value, "ternary_merge");

        self.builder
            .build_conditional_branch(cond_bool, then_block, else_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        self.builder.position_at_end(then_block);
        let then_val = self.generate_expr(then_expr)?;
        let then_end = self.builder.get_insert_block().unwrap();
        let then_terminated = then_end.get_terminator().is_some();
        if !then_terminated {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        self.builder.position_at_end(else_block);
        let else_val = self.generate_expr(else_expr)?;
        let else_end = self.builder.get_insert_block().unwrap();
        let else_terminated = else_end.get_terminator().is_some();
        if !else_terminated {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        self.builder.position_at_end(merge_block);

        if then_terminated && else_terminated {
            self.builder
                .build_unreachable()
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            return Ok(self.context.struct_type(&[], false).const_zero().into());
        }

        let mut incoming: Vec<(
            &dyn BasicValue<'ctx>,
            inkwell::basic_block::BasicBlock<'ctx>,
        )> = Vec::new();
        if !then_terminated {
            incoming.push((&then_val, then_end));
        }
        if !else_terminated {
            incoming.push((&else_val, else_end));
        }

        if incoming.len() == 1 {
            Ok(incoming[0].0.as_basic_value_enum())
        } else {
            let phi = self
                .builder
                .build_phi(then_val.get_type(), "ternary_result")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            for (val, block) in &incoming {
                phi.add_incoming(&[(*val, *block)]);
            }
            Ok(phi.as_basic_value())
        }
    }

    pub(super) fn generate_struct_literal(
        &mut self,
        name: &str,
        fields: &[(Spanned<String>, Spanned<Expr>)],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Generate field values mapped by name
        let mut field_name_values: Vec<(String, BasicValueEnum)> = Vec::new();
        for (field_name, expr) in fields {
            let val = self.generate_expr(&expr.node)?;
            field_name_values.push((field_name.node.clone(), val));
        }

        let struct_type = *self
            .generated_structs
            .get(name)
            .ok_or_else(|| CodegenError::UndefinedVar(format!("Struct not found: {}", name)))?;

        // Check if this is a union (single LLVM field, multiple logical fields)
        let num_llvm_fields = struct_type.count_fields();
        let num_logical_fields = self.struct_fields.get(name).map(|f| f.len()).unwrap_or(0);
        let is_union = num_llvm_fields == 1 && num_logical_fields > 1;

        let mut struct_val = struct_type.get_undef();
        if is_union {
            // Union: all fields share index 0, need to bitcast if types differ
            if let Some((_, val)) = field_name_values.first() {
                let field_llvm_type = struct_type.get_field_type_at_index(0).unwrap();
                let coerced_val = if val.get_type() != field_llvm_type {
                    // Bitcast through memory for type punning (e.g., f64 -> i64)
                    if val.is_float_value() && field_llvm_type.is_int_type() {
                        let bits = self
                            .builder
                            .build_bitcast(*val, field_llvm_type, "union_bitcast")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        bits
                    } else if val.is_int_value() && field_llvm_type.is_float_type() {
                        let bits = self
                            .builder
                            .build_bitcast(*val, field_llvm_type, "union_bitcast")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        bits
                    } else {
                        *val
                    }
                } else {
                    *val
                };
                struct_val = self
                    .builder
                    .build_insert_value(struct_val, coerced_val, 0, "union_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
            }
        } else if let Some(struct_field_names) = self.struct_fields.get(name) {
            // Reorder fields to match struct definition order
            for (i, def_field_name) in struct_field_names.iter().enumerate() {
                let val = field_name_values
                    .iter()
                    .find(|(n, _)| n == def_field_name)
                    .map(|(_, v)| *v)
                    .unwrap_or_else(|| {
                        // Use the actual field type's zero value from the struct definition
                        if let Some(field_type) = struct_type.get_field_type_at_index(i as u32) {
                            self.get_default_value(field_type)
                        } else {
                            self.context.i64_type().const_int(0, false).into()
                        }
                    });
                struct_val = self
                    .builder
                    .build_insert_value(struct_val, val, i as u32, &format!("field_{}", i))
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
            }
        } else {
            // Fallback: insert in source order
            for (i, (_, val)) in field_name_values.iter().enumerate() {
                struct_val = self
                    .builder
                    .build_insert_value(struct_val, *val, i as u32, &format!("field_{}", i))
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
            }
        }
        Ok(struct_val.into())
    }

    pub(super) fn generate_field_access(
        &mut self,
        obj: &Expr,
        field: &str,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let obj_val = self.generate_expr(obj)?;

        // Get struct type name from the expression
        let struct_name = self.infer_struct_name(obj)?;

        // Check if this is a union (single-field struct where all "fields" map to index 0)
        if let Some(struct_type) = self.generated_structs.get(&struct_name) {
            let num_fields = struct_type.count_fields();
            if let Some(field_names) = self.struct_fields.get(&struct_name) {
                if num_fields == 1 && field_names.len() > 1 {
                    // This is a union - all fields share index 0
                    let struct_val = obj_val.into_struct_value();
                    let raw_val = self
                        .builder
                        .build_extract_value(struct_val, 0, field)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    // If the field expects f64 but storage is i64, bitcast
                    // Determine expected field type from field name position
                    let field_idx = field_names.iter().position(|f| f == field).unwrap_or(0);
                    let _ = field_idx; // For now, return raw; the caller handles type interpretation
                    return Ok(raw_val);
                }
            }
        }

        // Lookup field index by name
        let field_idx = self.get_field_index(&struct_name, field)?;

        let struct_val = obj_val.into_struct_value();
        self.builder
            .build_extract_value(struct_val, field_idx, field)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }
}
