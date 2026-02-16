//! Unary operation generation.

use inkwell::values::BasicValueEnum;

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
            UnaryOp::Not => self
                .builder
                .build_not(val.into_int_value(), "not")
                .map(|v| v.into())
                .map_err(|e| CodegenError::LlvmError(e.to_string())),
            UnaryOp::BitNot => self
                .builder
                .build_not(val.into_int_value(), "bitnot")
                .map(|v| v.into())
                .map_err(|e| CodegenError::LlvmError(e.to_string())),
        }
    }
}
