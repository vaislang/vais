//! Binary operation generation.

use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};

use vais_ast::BinOp;

use super::super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    /// Get or declare the abort() function for runtime errors
    pub(in crate::inkwell) fn get_or_declare_abort(&mut self) -> FunctionValue<'ctx> {
        // Check if abort is already declared
        if let Some(abort_fn) = self.module.get_function("abort") {
            return abort_fn;
        }

        // Declare abort: void abort(void)
        let void_type = self.context.void_type();
        let fn_type = void_type.fn_type(&[], false);
        self.module.add_function("abort", fn_type, None)
    }

    #[inline(never)]
    pub(crate) fn generate_int_binary(
        &mut self,
        op: BinOp,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Add division by zero guard for Div and Mod operations
        if matches!(op, BinOp::Div | BinOp::Mod) {
            let zero = rhs.get_type().const_zero();
            let is_zero = self
                .builder
                .build_int_compare(IntPredicate::EQ, rhs, zero, "div_zero_check")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            let current_fn = self.current_function.ok_or_else(|| {
                CodegenError::Unsupported("Division outside function context".to_string())
            })?;

            let div_zero_bb = self.context.append_basic_block(current_fn, "div_zero");
            let div_ok_bb = self.context.append_basic_block(current_fn, "div_ok");

            self.builder
                .build_conditional_branch(is_zero, div_zero_bb, div_ok_bb)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Division by zero block: call abort() and unreachable
            self.builder.position_at_end(div_zero_bb);
            let abort_fn = self.get_or_declare_abort();
            self.builder
                .build_call(abort_fn, &[], "")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_unreachable()
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Continue with division in the safe block
            self.builder.position_at_end(div_ok_bb);
        }

        let result = match op {
            BinOp::Add => self.builder.build_int_add(lhs, rhs, "add"),
            BinOp::Sub => self.builder.build_int_sub(lhs, rhs, "sub"),
            BinOp::Mul => self.builder.build_int_mul(lhs, rhs, "mul"),
            BinOp::Div => self.builder.build_int_signed_div(lhs, rhs, "div"),
            BinOp::Mod => self.builder.build_int_signed_rem(lhs, rhs, "rem"),
            BinOp::Eq => self
                .builder
                .build_int_compare(IntPredicate::EQ, lhs, rhs, "eq"),
            BinOp::Neq => self
                .builder
                .build_int_compare(IntPredicate::NE, lhs, rhs, "ne"),
            BinOp::Lt => self
                .builder
                .build_int_compare(IntPredicate::SLT, lhs, rhs, "lt"),
            BinOp::Lte => self
                .builder
                .build_int_compare(IntPredicate::SLE, lhs, rhs, "le"),
            BinOp::Gt => self
                .builder
                .build_int_compare(IntPredicate::SGT, lhs, rhs, "gt"),
            BinOp::Gte => self
                .builder
                .build_int_compare(IntPredicate::SGE, lhs, rhs, "ge"),
            BinOp::And => self.builder.build_and(lhs, rhs, "and"),
            BinOp::Or => self.builder.build_or(lhs, rhs, "or"),
            BinOp::BitAnd => self.builder.build_and(lhs, rhs, "bitand"),
            BinOp::BitOr => self.builder.build_or(lhs, rhs, "bitor"),
            BinOp::BitXor => self.builder.build_xor(lhs, rhs, "bitxor"),
            BinOp::Shl => self.builder.build_left_shift(lhs, rhs, "shl"),
            BinOp::Shr => self.builder.build_right_shift(lhs, rhs, true, "shr"),
        };
        result
            .map(|v| v.into())
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    #[inline(never)]
    pub(crate) fn generate_float_binary(
        &mut self,
        op: BinOp,
        lhs: inkwell::values::FloatValue<'ctx>,
        rhs: inkwell::values::FloatValue<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let result = match op {
            BinOp::Add => self
                .builder
                .build_float_add(lhs, rhs, "fadd")
                .map(|v| v.into()),
            BinOp::Sub => self
                .builder
                .build_float_sub(lhs, rhs, "fsub")
                .map(|v| v.into()),
            BinOp::Mul => self
                .builder
                .build_float_mul(lhs, rhs, "fmul")
                .map(|v| v.into()),
            BinOp::Div => self
                .builder
                .build_float_div(lhs, rhs, "fdiv")
                .map(|v| v.into()),
            BinOp::Mod => self
                .builder
                .build_float_rem(lhs, rhs, "frem")
                .map(|v| v.into()),
            BinOp::Eq => self
                .builder
                .build_float_compare(FloatPredicate::OEQ, lhs, rhs, "feq")
                .map(|v| v.into()),
            BinOp::Neq => self
                .builder
                .build_float_compare(FloatPredicate::ONE, lhs, rhs, "fne")
                .map(|v| v.into()),
            BinOp::Lt => self
                .builder
                .build_float_compare(FloatPredicate::OLT, lhs, rhs, "flt")
                .map(|v| v.into()),
            BinOp::Lte => self
                .builder
                .build_float_compare(FloatPredicate::OLE, lhs, rhs, "fle")
                .map(|v| v.into()),
            BinOp::Gt => self
                .builder
                .build_float_compare(FloatPredicate::OGT, lhs, rhs, "fgt")
                .map(|v| v.into()),
            BinOp::Gte => self
                .builder
                .build_float_compare(FloatPredicate::OGE, lhs, rhs, "fge")
                .map(|v| v.into()),
            _ => {
                return Err(CodegenError::Unsupported(format!(
                    "Float binary op: {:?}",
                    op
                )))
            }
        };
        result.map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    /// Handle binary operations on string fat pointers { ptr, i64 }.
    /// Supports: Add (concatenation), Eq, Neq (comparison via strcmp).
    #[inline(never)]
    pub(crate) fn generate_string_binary(
        &mut self,
        op: BinOp,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let lhs_ptr = self.extract_str_raw_ptr(lhs)?;
        let rhs_ptr = self.extract_str_raw_ptr(rhs)?;

        match op {
            BinOp::Add => {
                // String concatenation: call strlen on both, malloc, memcpy both, build fat ptr
                let strlen_fn = self
                    .module
                    .get_function("strlen")
                    .ok_or_else(|| CodegenError::UndefinedFunction("strlen".to_string()))?;
                let malloc_fn = self
                    .module
                    .get_function("malloc")
                    .ok_or_else(|| CodegenError::UndefinedFunction("malloc".to_string()))?;
                let memcpy_fn = self
                    .module
                    .get_function("memcpy")
                    .ok_or_else(|| CodegenError::UndefinedFunction("memcpy".to_string()))?;

                let i64_type = self.context.i64_type();

                // Get lengths
                let len1 = self
                    .builder
                    .build_call(strlen_fn, &[lhs_ptr.into()], "len1")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| i64_type.const_int(0, false).into())
                    .into_int_value();
                let len2 = self
                    .builder
                    .build_call(strlen_fn, &[rhs_ptr.into()], "len2")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| i64_type.const_int(0, false).into())
                    .into_int_value();

                // Total length (including null terminator)
                let total_len = self
                    .builder
                    .build_int_add(len1, len2, "total_len")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                let alloc_len = self
                    .builder
                    .build_int_add(total_len, i64_type.const_int(1, false), "alloc_len")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // Allocate buffer
                let buf = self
                    .builder
                    .build_call(malloc_fn, &[alloc_len.into()], "str_buf")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| {
                        self.context
                            .i8_type()
                            .ptr_type(AddressSpace::default())
                            .const_null()
                            .into()
                    })
                    .into_pointer_value();
                // Track allocation via entry-block alloca slot to avoid dominance issues in loops.
                // Create an alloca in the entry block, store the malloc'd pointer there,
                // and record the alloca (not the raw pointer) in alloc_tracker.
                let current_fn = self.builder.get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();
                let entry_block = current_fn.get_first_basic_block().unwrap();
                let current_block = self.builder.get_insert_block().unwrap();
                // Position at end of entry block (before terminator if exists)
                if let Some(terminator) = entry_block.get_terminator() {
                    self.builder.position_before(&terminator);
                } else {
                    self.builder.position_at_end(entry_block);
                }
                let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
                let alloc_slot = self.builder.build_alloca(ptr_type, &format!("__str_alloc_slot_{}", self.alloc_tracker.len()))
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                // Initialize slot to null so cleanup is safe even if loop never executes
                self.builder.build_store(alloc_slot, ptr_type.const_null())
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                // Restore builder position
                self.builder.position_at_end(current_block);
                // Store the malloc'd pointer into the slot
                self.builder.build_store(alloc_slot, buf)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.alloc_tracker.push(alloc_slot);

                // Copy first string
                let args1: Vec<BasicMetadataValueEnum> =
                    vec![buf.into(), lhs_ptr.into(), len1.into()];
                self.builder
                    .build_call(memcpy_fn, &args1, "cpy1")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // Offset for second string
                // SAFETY: GEP offset is len1, within the malloc'd (len1 + len2 + 1) byte buffer.
                let buf2 = unsafe {
                    self.builder
                        .build_in_bounds_gep(self.context.i8_type(), buf, &[len1], "buf2")
                }
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // Copy second string (including null terminator)
                let len2_plus1 = self
                    .builder
                    .build_int_add(len2, i64_type.const_int(1, false), "len2_plus1")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                let args2: Vec<BasicMetadataValueEnum> =
                    vec![buf2.into(), rhs_ptr.into(), len2_plus1.into()];
                self.builder
                    .build_call(memcpy_fn, &args2, "cpy2")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // Build fat pointer { ptr, i64 }
                let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
                let str_struct_type = self
                    .context
                    .struct_type(&[ptr_type.into(), i64_type.into()], false);
                let undef = str_struct_type.get_undef();
                let with_ptr = self
                    .builder
                    .build_insert_value(undef, buf, 0, "concat_ptr")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                let fat_ptr = self
                    .builder
                    .build_insert_value(with_ptr, total_len, 1, "concat_len")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                // Record ownership: this fat pointer SSA value owns `alloc_slot`.
                // At `return`, if this value is returned, its slot is excluded from
                // free so the caller receives a live buffer. See RFC-001 §4.6.
                use inkwell::values::AsValueRef;
                self.string_value_slot
                    .insert(fat_ptr.as_value_ref() as usize, alloc_slot);
                // Register slot with the innermost block scope so it gets freed
                // at block end (loop iteration end, etc.) if not transferred out.
                if let Some(frame) = self.scope_str_stack.last_mut() {
                    frame.push(alloc_slot);
                }
                // Intermediate free: if the LHS was itself a tracked concat
                // result, its buffer has been consumed (memcpy'd above) and is
                // no longer referenced. Free it and null the slot so end-of-
                // scope cleanup becomes a no-op for it. See RFC-001 §4.3.
                // This fires per `+` in a chain like `a+b+c+d` and inside loop
                // bodies, eliminating per-iteration concat leaks.
                if lhs.is_struct_value() {
                    let lhs_key = lhs.into_struct_value().as_value_ref() as usize;
                    if let Some(old_slot) = self.string_value_slot.remove(&lhs_key) {
                        self.emit_free_slot(old_slot)?;
                    }
                }
                Ok(fat_ptr.into())
            }
            BinOp::Eq | BinOp::Neq => {
                // String comparison via strcmp
                let strcmp_fn = self
                    .module
                    .get_function("strcmp")
                    .ok_or_else(|| CodegenError::UndefinedFunction("strcmp".to_string()))?;
                let cmp = self
                    .builder
                    .build_call(
                        strcmp_fn,
                        &[lhs_ptr.into(), rhs_ptr.into()],
                        "strcmp_result",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| self.context.i32_type().const_int(0, false).into())
                    .into_int_value();
                let zero = cmp.get_type().const_zero();
                let pred = if op == BinOp::Eq {
                    IntPredicate::EQ
                } else {
                    IntPredicate::NE
                };
                let result = self
                    .builder
                    .build_int_compare(pred, cmp, zero, "str_cmp")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                // Extend bool to i64 for Vais convention
                let extended = self
                    .builder
                    .build_int_z_extend(result, self.context.i64_type(), "str_cmp_ext")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(extended.into())
            }
            _ => Err(CodegenError::Unsupported(format!(
                "String binary op: {:?}",
                op
            ))),
        }
    }
}
