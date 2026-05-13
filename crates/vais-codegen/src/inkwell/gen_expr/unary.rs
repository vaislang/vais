//! Unary operation generation.

use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;

use vais_ast::{Expr, UnaryOp};

use super::super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn generate_unary(
        &mut self,
        op: UnaryOp,
        operand: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let val = self.generate_expr(operand)?;

        match op {
            UnaryOp::Neg => {
                if val.is_float_value() {
                    self.builder
                        .build_float_neg(val.into_float_value(), "fneg")
                        .map(|v| v.into())
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))
                } else {
                    self.builder
                        .build_int_neg(val.into_int_value(), "neg")
                        .map(|v| v.into())
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))
                }
            }
            UnaryOp::Not => {
                // Logical NOT: compare value != 0 to get i1, then NOT the i1, then zero-extend
                // back to the original integer width so the result type matches the operand.
                let int_val = val.into_int_value();
                let zero = int_val.get_type().const_int(0, false);
                let is_nonzero = self
                    .builder
                    .build_int_compare(IntPredicate::NE, int_val, zero, "not_cmp")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                // Flip the i1 to get the logical NOT result (true → false, false → true)
                let flipped = self
                    .builder
                    .build_not(is_nonzero, "not_flip")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                // Zero-extend back to the original bit width
                let result = self
                    .builder
                    .build_int_z_extend(flipped, int_val.get_type(), "not_zext")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            UnaryOp::BitNot => self
                .builder
                .build_not(val.into_int_value(), "bitnot")
                .map(|v| v.into())
                .map_err(|e| CodegenError::LlvmError(e.to_string())),
        }
    }
}
