//! Binary operation generation.

use inkwell::values::{BasicValueEnum, IntValue};
use inkwell::{FloatPredicate, IntPredicate};

use vais_ast::BinOp;

use super::super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(crate) fn generate_int_binary(
        &mut self,
        op: BinOp,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
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
}
