//! Variable and identifier handling.

use inkwell::values::BasicValueEnum;

use super::super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn generate_var(&mut self, name: &str) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Handle None as a built-in value using the canonical erased Option ABI.
        if name == "None" {
            let enum_type = self.erased_enum_type("Option");
            let tag = self.get_enum_variant_tag("None");
            return self.build_erased_enum_value(enum_type, tag, &[], "none");
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
        } else if let Some((global, ty)) = self.globals.get(name).copied() {
            // User-declared global (`G name: T = v`) — load from the module-level pointer.
            let value = self
                .builder
                .build_load(ty, global.as_pointer_value(), name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(value)
        } else {
            // Check if this is an enum variant name (e.g., Red, Green, Blue)
            let variant = self
                .enum_variants
                .iter()
                .find(|((_, v_name), _)| v_name == name)
                .map(|((enum_name, _), tag)| (enum_name.clone(), *tag));
            if let Some((enum_name, tag)) = variant {
                let enum_type = self
                    .generated_structs
                    .get(&enum_name)
                    .copied()
                    .unwrap_or_else(|| self.erased_enum_type(&enum_name));
                return self.build_erased_enum_value(enum_type, tag, &[], "variant");
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
