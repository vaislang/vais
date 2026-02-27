//! Binary operation generation.

use inkwell::values::{BasicValueEnum, FunctionValue, IntValue};
use inkwell::{FloatPredicate, IntPredicate};

use vais_ast::BinOp;

use super::super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    /// Get or declare the abort() function for runtime errors
    fn get_or_declare_abort(&mut self) -> FunctionValue<'ctx> {
        // Check if abort is already declared
        if let Some(abort_fn) = self.module.get_function("abort") {
            return abort_fn;
        }

        // Declare abort: void abort(void)
        let void_type = self.context.void_type();
        let fn_type = void_type.fn_type(&[], false);
        self.module.add_function("abort", fn_type, None)
    }

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
