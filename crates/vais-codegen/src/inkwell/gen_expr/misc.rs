//! Miscellaneous expression generation (cast, deref, etc.).

use inkwell::values::BasicValueEnum;

use vais_ast::{Expr, Type};

use super::super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn generate_deref(&mut self, inner: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        let ptr = self.generate_expr(inner)?;
        let ptr_val = if ptr.is_pointer_value() {
            ptr.into_pointer_value()
        } else {
            // IntValue (i64) → PointerValue via inttoptr
            let int_val = ptr.into_int_value();
            self.builder
                .build_int_to_ptr(
                    int_val,
                    self.context
                        .i64_type()
                        .ptr_type(inkwell::AddressSpace::default()),
                    "deref_ptr",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };
        self.builder
            .build_load(self.context.i64_type(), ptr_val, "deref")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    pub(super) fn generate_cast(
        &mut self,
        cast_expr: &Expr,
        cast_ty: &Type,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let val = self.generate_expr(cast_expr)?;
        let target_resolved = self.ast_type_to_resolved(cast_ty);
        let target_type = self.type_mapper.map_type(&target_resolved);

        // Perform actual type conversions
        if val.is_int_value() && target_type.is_float_type() {
            // i64 -> f64
            let result = self
                .builder
                .build_signed_int_to_float(
                    val.into_int_value(),
                    target_type.into_float_type(),
                    "cast_itof",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(result.into())
        } else if val.is_float_value() && target_type.is_int_type() {
            // f64 -> i64
            let result = self
                .builder
                .build_float_to_signed_int(
                    val.into_float_value(),
                    target_type.into_int_type(),
                    "cast_ftoi",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(result.into())
        } else if val.is_int_value() && target_type.is_int_type() {
            let src_width = val.into_int_value().get_type().get_bit_width();
            let dst_width = target_type.into_int_type().get_bit_width();
            if src_width < dst_width {
                let result = self
                    .builder
                    .build_int_s_extend(
                        val.into_int_value(),
                        target_type.into_int_type(),
                        "cast_sext",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            } else if src_width > dst_width {
                let result = self
                    .builder
                    .build_int_truncate(
                        val.into_int_value(),
                        target_type.into_int_type(),
                        "cast_trunc",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            } else {
                Ok(val)
            }
        } else if val.is_int_value() && target_type.is_pointer_type() {
            // i64 -> ptr
            let result = self
                .builder
                .build_int_to_ptr(
                    val.into_int_value(),
                    target_type.into_pointer_type(),
                    "cast_itoptr",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(result.into())
        } else if val.is_pointer_value() && target_type.is_int_type() {
            // ptr -> i64
            let result = self
                .builder
                .build_ptr_to_int(
                    val.into_pointer_value(),
                    target_type.into_int_type(),
                    "cast_ptrtoi",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(result.into())
        } else {
            // Same type or unsupported cast - return as-is
            Ok(val)
        }
    }
}
