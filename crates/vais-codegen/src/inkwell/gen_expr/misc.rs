//! Miscellaneous expression generation (cast, deref, coerce, etc.).

use inkwell::values::{BasicValueEnum, IntValue};

use vais_ast::{Expr, Type};

use super::super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    /// Coerce any BasicValueEnum to an i64 IntValue.
    /// - IntValue: widen/truncate to i64
    /// - FloatValue: bitcast f64 to i64
    /// - PointerValue: ptrtoint to i64
    /// - StructValue: extract first int field, or return 0
    pub(crate) fn coerce_to_i64(
        &self,
        v: BasicValueEnum<'ctx>,
    ) -> CodegenResult<IntValue<'ctx>> {
        let i64_type = self.context.i64_type();
        if v.is_int_value() {
            let iv = v.into_int_value();
            if iv.get_type().get_bit_width() == 64 {
                Ok(iv)
            } else {
                self.builder
                    .build_int_cast(iv, i64_type, "coerce_int")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))
            }
        } else if v.is_float_value() {
            // Bitcast f64 to i64 to preserve the bits
            let fv = v.into_float_value();
            let bitcast = self
                .builder
                .build_bitcast(fv, i64_type, "coerce_f2i")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(bitcast.into_int_value())
        } else if v.is_pointer_value() {
            self.builder
                .build_ptr_to_int(v.into_pointer_value(), i64_type, "coerce_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))
        } else {
            // StructValue or other: return 0 as fallback
            Ok(i64_type.const_int(0, false))
        }
    }

    pub(super) fn generate_deref(&mut self, inner: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Infer pointee type from the expression's resolved type before generating.
        // For Pointer(T), Ref(T), RefMut(T) the pointee type is T.
        let pointee_llvm_type = if let Expr::Ident(name) = inner {
            if let Some(
                vais_types::ResolvedType::Pointer(inner_ty)
                | vais_types::ResolvedType::Ref(inner_ty)
                | vais_types::ResolvedType::RefMut(inner_ty),
            ) = self.var_resolved_types.get(name)
            {
                self.type_mapper.map_type(inner_ty)
            } else {
                self.context.i64_type().into()
            }
        } else {
            self.context.i64_type().into()
        };

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
        // Load using the inferred pointee type (context-based, not hardcoded i64)
        self.builder
            .build_load(pointee_llvm_type, ptr_val, "deref")
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
