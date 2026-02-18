//! Function call generation.

use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use inkwell::AddressSpace;

use vais_ast::{Expr, Spanned};

use super::super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn generate_call(
        &mut self,
        callee: &Expr,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Struct tuple literal: `Response(200, 1)` → desugar to StructLit
        if let Expr::Ident(name) = callee {
            if self.generated_structs.contains_key(name.as_str())
                && self.module.get_function(name).is_none()
            {
                if let Some(field_names) = self.struct_fields.get(name.as_str()).cloned() {
                    let fields: Vec<_> = field_names
                        .iter()
                        .zip(args.iter())
                        .map(|(fname, val)| {
                            (vais_ast::Spanned::new(fname.clone(), val.span), val.clone())
                        })
                        .collect();
                    return self.generate_struct_literal(name, &fields);
                }
            }
        }

        // Get function name
        let fn_name = match callee {
            Expr::Ident(name) => name.clone(),
            Expr::SelfCall => {
                // @ recursive call: check if we're in TCO mode
                if let Some(tco) = &self.tco_state {
                    // TCO: update parameters and branch back to loop header
                    let param_allocas = tco.param_allocas.clone();
                    let loop_header = tco.loop_header;

                    // Evaluate all arguments first (before updating any params)
                    let arg_values: Vec<BasicValueEnum<'ctx>> = args
                        .iter()
                        .map(|arg| self.generate_expr(&arg.node))
                        .collect::<CodegenResult<Vec<_>>>()?;

                    // Update parameter allocas with new values
                    for (i, (_, alloca, param_type)) in param_allocas.iter().enumerate() {
                        if i < arg_values.len() {
                            let mut new_val = arg_values[i];
                            // Cast if needed
                            if new_val.get_type() != *param_type
                                && new_val.is_int_value()
                                && param_type.is_int_type()
                            {
                                new_val = self
                                    .builder
                                    .build_int_cast(
                                        new_val.into_int_value(),
                                        param_type.into_int_type(),
                                        "tco_cast",
                                    )
                                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                                    .into();
                            }
                            self.builder
                                .build_store(*alloca, new_val)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        }
                    }

                    // Branch back to loop header
                    self.builder
                        .build_unconditional_branch(loop_header)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Return a dummy value (this code path is dead after the branch)
                    return Ok(self.context.i64_type().const_int(0, false).into());
                }

                // Non-TCO: regular recursive call
                if let Some(func) = self.current_function {
                    let fn_name = func.get_name().to_str().unwrap_or("").to_string();
                    fn_name
                } else {
                    return Err(CodegenError::Unsupported(
                        "SelfCall outside function".to_string(),
                    ));
                }
            }
            _ => {
                // Indirect call: evaluate the callee expression to get a function pointer.
                // Vais closures/lambdas are represented as i64 (pointer-sized integer), so we
                // convert i64 → i8* pointer then perform an indirect call via build_indirect_call.
                let callee_val = self.generate_expr(callee)?;

                // Evaluate arguments before building the function type.
                let arg_values: Vec<BasicMetadataValueEnum> = args
                    .iter()
                    .map(|arg| self.generate_expr(&arg.node).map(|v| v.into()))
                    .collect::<CodegenResult<Vec<_>>>()?;

                // Build a function type: (i64, i64, ...) -> i64, one i64 per argument.
                let i64_type = self.context.i64_type();
                let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> =
                    (0..arg_values.len())
                        .map(|_| i64_type.into())
                        .collect();
                let fn_type = i64_type.fn_type(&param_types, false);

                // Obtain a PointerValue for the callee.
                let fn_ptr = if callee_val.is_pointer_value() {
                    callee_val.into_pointer_value()
                } else {
                    // Convert i64 (function pointer stored as integer) to i8*.
                    let i8_ptr_type = self
                        .context
                        .i8_type()
                        .ptr_type(AddressSpace::default());
                    self.builder
                        .build_int_to_ptr(
                            callee_val.into_int_value(),
                            i8_ptr_type,
                            "indirect_fn_ptr",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                };

                // Cast the arguments: ensure each is an i64 (or its existing type).
                let coerced_args: Vec<BasicMetadataValueEnum> = arg_values
                    .into_iter()
                    .map(|v| {
                        if let BasicMetadataValueEnum::IntValue(iv) = v {
                            if iv.get_type() != i64_type {
                                self.builder
                                    .build_int_cast(iv, i64_type, "indirect_arg_cast")
                                    .map(BasicMetadataValueEnum::IntValue)
                                    .map_err(|e| CodegenError::LlvmError(e.to_string()))
                            } else {
                                Ok(BasicMetadataValueEnum::IntValue(iv))
                            }
                        } else {
                            Ok(v)
                        }
                    })
                    .collect::<CodegenResult<Vec<_>>>()?;

                let call = self
                    .builder
                    .build_indirect_call(fn_type, fn_ptr, &coerced_args, "indirect_call")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                return Ok(call
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| i64_type.const_int(0, false).into()));
            }
        };

        // Handle built-in pseudo-functions that need special codegen
        // Handle puts with string interpolation: printf the interp, then puts("") for newline
        if fn_name == "puts" && args.len() == 1 && matches!(&args[0].node, Expr::StringInterp(_)) {
            let _interp_val = self.generate_expr(&args[0].node)?;
            // String interp already printed via printf; now add newline
            let printf_fn = self
                .module
                .get_function("printf")
                .ok_or_else(|| CodegenError::UndefinedFunction("printf".to_string()))?;
            let newline = self.generate_string_literal("\n")?;
            self.builder
                .build_call(printf_fn, &[newline.into()], "puts_nl")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            return Ok(self.context.struct_type(&[], false).const_zero().into());
        }
        match fn_name.as_str() {
            "println" => return self.generate_println_call(args),
            "print" => return self.generate_print_call(args),
            "format" => return self.generate_format_call(args),
            "store_i64" => return self.generate_store_i64(args),
            "load_i64" => return self.generate_load_i64(args),
            "swap" => return self.generate_swap(args),
            "store_byte" => return self.generate_store_byte(args),
            "load_byte" => return self.generate_load_byte(args),
            "store_f64" => return self.generate_store_f64(args),
            "load_f64" => return self.generate_load_f64(args),
            // Option constructors: Some(val) -> { i8 tag=1, i64 data=val }
            "Some" => {
                let enum_type = self.context.struct_type(
                    &[
                        self.context.i8_type().into(),
                        self.context.i64_type().into(),
                    ],
                    false,
                );
                let data_val = if args.is_empty() {
                    self.context.i64_type().const_int(0, false)
                } else {
                    let v = self.generate_expr(&args[0].node)?;
                    if v.is_int_value() {
                        v.into_int_value()
                    } else {
                        self.context.i64_type().const_int(0, false)
                    }
                };
                let mut val = enum_type.get_undef();
                val = self
                    .builder
                    .build_insert_value(
                        val,
                        self.context.i8_type().const_int(1, false),
                        0,
                        "some_tag",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                val = self
                    .builder
                    .build_insert_value(val, data_val, 1, "some_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                return Ok(val.into());
            }
            // Result constructors: Ok(val) -> { i8 tag=0, i64 data=val }
            "Ok" => {
                let enum_type = self.context.struct_type(
                    &[
                        self.context.i8_type().into(),
                        self.context.i64_type().into(),
                    ],
                    false,
                );
                let data_val = if args.is_empty() {
                    self.context.i64_type().const_int(0, false)
                } else {
                    let v = self.generate_expr(&args[0].node)?;
                    if v.is_int_value() {
                        v.into_int_value()
                    } else {
                        self.context.i64_type().const_int(0, false)
                    }
                };
                let mut val = enum_type.get_undef();
                val = self
                    .builder
                    .build_insert_value(
                        val,
                        self.context.i8_type().const_int(0, false),
                        0,
                        "ok_tag",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                val = self
                    .builder
                    .build_insert_value(val, data_val, 1, "ok_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                return Ok(val.into());
            }
            // Err(val) -> { i8 tag=1, i64 data=val }
            "Err" => {
                let enum_type = self.context.struct_type(
                    &[
                        self.context.i8_type().into(),
                        self.context.i64_type().into(),
                    ],
                    false,
                );
                let data_val = if args.is_empty() {
                    self.context.i64_type().const_int(0, false)
                } else {
                    let v = self.generate_expr(&args[0].node)?;
                    if v.is_int_value() {
                        v.into_int_value()
                    } else {
                        self.context.i64_type().const_int(0, false)
                    }
                };
                let mut val = enum_type.get_undef();
                val = self
                    .builder
                    .build_insert_value(
                        val,
                        self.context.i8_type().const_int(1, false),
                        0,
                        "err_tag",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                val = self
                    .builder
                    .build_insert_value(val, data_val, 1, "err_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                return Ok(val.into());
            }
            "puts_ptr" => {
                // puts_ptr(i64) -> i32: convert i64 to ptr then call puts
                if args.is_empty() {
                    return Err(CodegenError::InternalError(format!(
                        "builtin 'puts_ptr' requires 1 argument(s), got {}",
                        args.len()
                    )));
                }
                let arg = self.generate_expr(&args[0].node)?;
                let ptr = self
                    .builder
                    .build_int_to_ptr(
                        arg.into_int_value(),
                        self.context.i8_type().ptr_type(AddressSpace::default()),
                        "puts_ptr_arg",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                let puts_fn = self
                    .module
                    .get_function("puts")
                    .ok_or_else(|| CodegenError::UndefinedFunction("puts".to_string()))?;
                let call = self
                    .builder
                    .build_call(puts_fn, &[ptr.into()], "puts_call")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                return Ok(call
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into()));
            }
            "str_to_ptr" => {
                // str_to_ptr(ptr) -> i64: convert ptr to i64
                if args.is_empty() {
                    return Err(CodegenError::InternalError(format!(
                        "builtin 'str_to_ptr' requires 1 argument(s), got {}",
                        args.len()
                    )));
                }
                let arg = self.generate_expr(&args[0].node)?;
                if arg.is_pointer_value() {
                    let result = self
                        .builder
                        .build_ptr_to_int(
                            arg.into_pointer_value(),
                            self.context.i64_type(),
                            "str_to_ptr_result",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    return Ok(result.into());
                } else {
                    // Already an integer
                    return Ok(arg);
                }
            }
            "ptr_to_str" => {
                // ptr_to_str(i64) -> ptr: convert i64 to i8*
                if args.is_empty() {
                    return Err(CodegenError::InternalError(format!(
                        "builtin 'ptr_to_str' requires 1 argument(s), got {}",
                        args.len()
                    )));
                }
                let arg = self.generate_expr(&args[0].node)?;
                if arg.is_int_value() {
                    let i8_ptr_type = self
                        .context
                        .i8_type()
                        .ptr_type(inkwell::AddressSpace::default());
                    let result = self
                        .builder
                        .build_int_to_ptr(arg.into_int_value(), i8_ptr_type, "ptr_to_str_result")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    return Ok(result.into());
                } else {
                    // Already a pointer
                    return Ok(arg);
                }
            }
            "strlen_ptr" => {
                // strlen_ptr(i64) -> i64: convert i64 to ptr then call strlen
                if args.is_empty() {
                    return Err(CodegenError::InternalError(format!(
                        "builtin 'strlen_ptr' requires 1 argument(s), got {}",
                        args.len()
                    )));
                }
                let arg = self.generate_expr(&args[0].node)?;
                let ptr = self
                    .builder
                    .build_int_to_ptr(
                        arg.into_int_value(),
                        self.context.i8_type().ptr_type(AddressSpace::default()),
                        "strlen_ptr_arg",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                let strlen_fn = self
                    .module
                    .get_function("strlen")
                    .ok_or_else(|| CodegenError::UndefinedFunction("strlen".to_string()))?;
                let call = self
                    .builder
                    .build_call(strlen_fn, &[ptr.into()], "strlen_call")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                return Ok(call
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into()));
            }
            "__strlen" => {
                // __strlen is an alias for strlen
                let strlen_fn = self
                    .module
                    .get_function("strlen")
                    .ok_or_else(|| CodegenError::UndefinedFunction("strlen".to_string()))?;
                let arg_values: Vec<BasicMetadataValueEnum> = args
                    .iter()
                    .map(|arg| self.generate_expr(&arg.node).map(|v| v.into()))
                    .collect::<CodegenResult<Vec<_>>>()?;
                let call = self
                    .builder
                    .build_call(strlen_fn, &arg_values, "strlen_call")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                return Ok(call
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into()));
            }
            _ => {}
        }

        // Check if this is a lambda binding (closure call)
        if let Some((lambda_fn_name, captured_vals)) = self.lambda_bindings.get(&fn_name).cloned() {
            if let Some(lambda_fn) = self
                .functions
                .get(&lambda_fn_name)
                .copied()
                .or_else(|| self.module.get_function(&lambda_fn_name))
            {
                // Build args: captured values first, then actual args
                let mut arg_values: Vec<BasicMetadataValueEnum> =
                    captured_vals.iter().map(|(_, val)| (*val).into()).collect();
                for arg in args {
                    arg_values.push(self.generate_expr(&arg.node)?.into());
                }
                let call = self
                    .builder
                    .build_call(lambda_fn, &arg_values, "lambda_call")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                return Ok(call
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| self.context.struct_type(&[], false).const_zero().into()));
            }
        }

        // Get function value
        let fn_value = self
            .functions
            .get(&fn_name)
            .copied()
            .or_else(|| self.module.get_function(&fn_name));

        let fn_value = if let Some(func) = fn_value {
            func
        } else {
            // Check if this is an enum variant constructor (tuple variant)
            let is_enum_variant = self
                .enum_variants
                .iter()
                .find(|((_, v_name), _)| v_name == &fn_name)
                .map(|((_, _), tag)| *tag);

            if let Some(tag) = is_enum_variant {
                // Build enum value: { i8 tag, i64 data }
                let enum_type = self.context.struct_type(
                    &[
                        self.context.i8_type().into(),
                        self.context.i64_type().into(),
                    ],
                    false,
                );
                let data_val = if args.is_empty() {
                    self.context.i64_type().const_int(0, false)
                } else {
                    let v = self.generate_expr(&args[0].node)?;
                    if v.is_int_value() {
                        v.into_int_value()
                    } else {
                        self.context.i64_type().const_int(0, false)
                    }
                };
                let mut val = enum_type.get_undef();
                val = self
                    .builder
                    .build_insert_value(
                        val,
                        self.context.i8_type().const_int(tag as u64, false),
                        0,
                        "variant_tag",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                val = self
                    .builder
                    .build_insert_value(val, data_val, 1, "variant_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                return Ok(val.into());
            }

            // Collect all available function names for suggestions
            let mut candidate_strings: Vec<String> = Vec::new();

            // Add registered functions
            for func_name in self.functions.keys() {
                candidate_strings.push(func_name.clone());
            }

            // Add module functions
            let mut current_func = self.module.get_first_function();
            while let Some(func) = current_func {
                if let Ok(name) = func.get_name().to_str() {
                    candidate_strings.push(name.to_string());
                }
                current_func = func.get_next_function();
            }

            // Get suggestions
            let candidates: Vec<&str> = candidate_strings.iter().map(|s| s.as_str()).collect();
            let suggestions = crate::suggest_similar(&fn_name, &candidates, 3);
            let suggestion_text = crate::format_did_you_mean(&suggestions);

            return Err(CodegenError::UndefinedFunction(format!(
                "{}{}",
                fn_name, suggestion_text
            )));
        };

        // Generate arguments
        let arg_values: Vec<BasicMetadataValueEnum> = args
            .iter()
            .map(|arg| self.generate_expr(&arg.node).map(|v| v.into()))
            .collect::<CodegenResult<Vec<_>>>()?;

        // Build call
        let call = self
            .builder
            .build_call(fn_value, &arg_values, "call")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Return call result or unit
        Ok(call
            .try_as_basic_value()
            .left()
            .unwrap_or_else(|| self.context.struct_type(&[], false).const_zero().into()))
    }
}
