//! Variable and identifier handling.

use inkwell::values::BasicValueEnum;

use super::super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn generate_var(&mut self, name: &str) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Handle None as a built-in value: { i8 tag=0, i64 data=0 }
        if name == "None" {
            let enum_type = self.context.struct_type(
                &[
                    self.context.i8_type().into(),
                    self.context.i64_type().into(),
                ],
                false,
            );
            let mut val = enum_type.get_undef();
            val = self
                .builder
                .build_insert_value(
                    val,
                    self.context.i8_type().const_int(0, false),
                    0,
                    "none_tag",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_struct_value();
            val = self
                .builder
                .build_insert_value(
                    val,
                    self.context.i64_type().const_int(0, false),
                    1,
                    "none_data",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_struct_value();
            return Ok(val.into());
        }

        let result = self.locals.get(name);

        if let Some((ptr, var_type)) = result {
            let value = self
                .builder
                .build_load(*var_type, *ptr, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(value)
        } else if let Some(val) = self.constants.get(name) {
            // Check constants
            Ok(*val)
        } else {
            // Check if this is an enum variant name (e.g., Red, Green, Blue)
            for ((_, v_name), tag) in &self.enum_variants {
                if v_name == name {
                    let enum_type = self.context.struct_type(
                        &[
                            self.context.i8_type().into(),
                            self.context.i64_type().into(),
                        ],
                        false,
                    );
                    let mut val = enum_type.get_undef();
                    val = self
                        .builder
                        .build_insert_value(
                            val,
                            self.context.i8_type().const_int(*tag as u64, false),
                            0,
                            "variant_tag",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                        .into_struct_value();
                    val = self
                        .builder
                        .build_insert_value(
                            val,
                            self.context.i64_type().const_int(0, false),
                            1,
                            "variant_data",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                        .into_struct_value();
                    return Ok(val.into());
                }
            }

            // Check if this is a function name (function reference as variable)
            if let Some(func) = self
                .functions
                .get(name)
                .copied()
                .or_else(|| self.module.get_function(name))
            {
                // Return function pointer as i64
                let fn_ptr = func.as_global_value().as_pointer_value();
                let fn_int = self
                    .builder
                    .build_ptr_to_int(fn_ptr, self.context.i64_type(), "fn_ptr_as_int")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                return Ok(fn_int.into());
            }

            // Collect all available symbols for suggestions
            let mut candidates: Vec<&str> = Vec::new();

            // Add local variables
            for var_name in self.locals.keys() {
                candidates.push(var_name.as_str());
            }

            // Add function names
            for func_name in self.functions.keys() {
                candidates.push(func_name.as_str());
            }

            // Get suggestions
            let suggestions = crate::suggest_similar(name, &candidates, 3);
            let suggestion_text = crate::format_did_you_mean(&suggestions);

            Err(CodegenError::UndefinedVar(format!(
                "{}{}",
                name, suggestion_text
            )))
        }
    }
}
