//! Literal expression generation.

use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

use super::super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    /// Build a string fat pointer `{ ptr, i64 }` from a raw i8* pointer and length.
    fn build_str_fat_ptr(
        &self,
        raw_ptr: inkwell::values::PointerValue<'ctx>,
        len: u64,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let len_type = self.context.i64_type();
        let str_struct_type =
            self.context
                .struct_type(&[ptr_type.into(), len_type.into()], false);

        // Build { ptr, i64 } struct: insertvalue undef, ptr, 0; insertvalue ..., i64 len, 1
        let undef = str_struct_type.get_undef();
        let with_ptr = self
            .builder
            .build_insert_value(undef, raw_ptr, 0, "str_fat_ptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();
        let len_val = len_type.const_int(len, false);
        let fat_ptr = self
            .builder
            .build_insert_value(with_ptr, len_val, 1, "str_fat_ptr_len")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();
        Ok(fat_ptr.into())
    }

    pub(crate) fn generate_string_literal(
        &mut self,
        s: &str,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let str_len = s.len() as u64;

        // Check if we already have this string
        if let Some(global) = self.string_constants.get(s) {
            let ptr = self
                .builder
                .build_pointer_cast(
                    global.as_pointer_value(),
                    self.context.i8_type().ptr_type(AddressSpace::default()),
                    "str_ptr",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            return self.build_str_fat_ptr(ptr, str_len);
        }

        // Create new global string constant
        let name = format!(".str.{}", self.string_counter);
        self.string_counter += 1;

        let string_value = self.context.const_string(s.as_bytes(), true);
        let global = self.module.add_global(
            string_value.get_type(),
            Some(AddressSpace::default()),
            &name,
        );
        global.set_initializer(&string_value);
        global.set_constant(true);

        self.string_constants.insert(s.to_string(), global);

        let ptr = self
            .builder
            .build_pointer_cast(
                global.as_pointer_value(),
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "str_ptr",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.build_str_fat_ptr(ptr, str_len)
    }
}
