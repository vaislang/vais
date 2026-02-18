//! Literal expression generation.

use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

use super::super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(crate) fn generate_string_literal(
        &mut self,
        s: &str,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
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
            return Ok(ptr.into());
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
        Ok(ptr.into())
    }
}
