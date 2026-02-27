//! Aggregate type code generation.
//!
//! Handles arrays, tuples, indexing, slicing, method calls,
//! and lambda expressions.

use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, PointerValue};
use inkwell::{AddressSpace, IntPredicate};

use vais_ast::{self as ast, Expr, IfElse, Spanned, Stmt};

use super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn generate_array(
        &mut self,
        elements: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        if elements.is_empty() {
            // Empty array - return null pointer
            return Ok(self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .const_null()
                .into());
        }

        // Generate all elements
        let mut values: Vec<BasicValueEnum> = Vec::new();
        for elem in elements {
            values.push(self.generate_expr(&elem.node)?);
        }

        // Determine element type from first element
        let elem_type = values[0].get_type();
        let array_type = elem_type.array_type(elements.len() as u32);

        // Allocate array on stack
        let array_ptr = self
            .builder
            .build_alloca(array_type, "array")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store each element
        for (i, val) in values.iter().enumerate() {
            let idx = self.context.i64_type().const_int(i as u64, false);
            let elem_ptr = unsafe {
                self.builder
                    .build_gep(
                        array_type,
                        array_ptr,
                        &[self.context.i64_type().const_int(0, false), idx],
                        &format!("array_elem_{}", i),
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            };
            self.builder
                .build_store(elem_ptr, *val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        Ok(array_ptr.into())
    }

    pub(super) fn generate_tuple(
        &mut self,
        elements: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        if elements.is_empty() {
            return Ok(self.context.struct_type(&[], false).const_zero().into());
        }

        // Generate all elements
        let mut values: Vec<BasicValueEnum> = Vec::new();
        for elem in elements {
            values.push(self.generate_expr(&elem.node)?);
        }

        // Create anonymous struct type for tuple
        let field_types: Vec<BasicTypeEnum> = values.iter().map(|v| v.get_type()).collect();
        let tuple_type = self.context.struct_type(&field_types, false);

        // Build tuple value
        let mut tuple_val = tuple_type.get_undef();
        for (i, val) in values.iter().enumerate() {
            tuple_val = self
                .builder
                .build_insert_value(tuple_val, *val, i as u32, "tuple")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_struct_value();
        }

        Ok(tuple_val.into())
    }

    /// Infer the element LLVM type for a slice or array expression.
    /// Looks up the variable's resolved type from `var_resolved_types` and extracts
    /// the inner element type for Slice/SliceMut/Array. Falls back to i64 if unknown.
    fn infer_element_llvm_type(&self, arr_expr: &Expr) -> inkwell::types::BasicTypeEnum<'ctx> {
        if let Expr::Ident(name) = arr_expr {
            if let Some(
                vais_types::ResolvedType::Slice(inner)
                | vais_types::ResolvedType::SliceMut(inner)
                | vais_types::ResolvedType::Array(inner),
            ) = self.var_resolved_types.get(name)
            {
                return self.type_mapper.map_type(inner);
            }
        }
        // Fallback to i64 for untracked expressions
        self.context.i64_type().into()
    }

    pub(super) fn generate_index(
        &mut self,
        arr: &Expr,
        index: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Infer element type before generating (uses AST-level info)
        let inferred_elem_type = self.infer_element_llvm_type(arr);

        let arr_val = self.generate_expr(arr)?;
        let idx_val = self.generate_expr(index)?;

        // Check if this is a slice (fat pointer struct with 2 fields: ptr + len)
        if arr_val.is_struct_value() {
            let struct_val = arr_val.into_struct_value();
            let struct_type = struct_val.get_type();

            // Slice is { i8*, i64 } - check if it has exactly 2 fields
            if struct_type.count_fields() == 2 {
                // This is likely a slice - extract data pointer (field 0)
                let data_ptr = self
                    .builder
                    .build_extract_value(struct_val, 0, "data_ptr")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // Use the inferred element type from the variable's resolved type.
                // The GEP instruction uses elem_type for stride calculation.
                let elem_type = inferred_elem_type;
                let data_ptr_val = data_ptr.into_pointer_value();
                let idx_int = idx_val.into_int_value();

                // GEP to get element pointer (stride = sizeof(elem_type))
                let elem_ptr = unsafe {
                    self.builder
                        .build_gep(elem_type, data_ptr_val, &[idx_int], "elem_ptr")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                };

                // Load element
                return self
                    .builder
                    .build_load(elem_type, elem_ptr, "elem")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()));
            }
        }

        // Regular array/pointer indexing — use inferred element type
        let arr_ptr = arr_val.into_pointer_value();
        let idx_int = idx_val.into_int_value();

        // Get element pointer
        let elem_ptr = unsafe {
            self.builder
                .build_gep(inferred_elem_type, arr_ptr, &[idx_int], "elem_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };

        // Load element
        self.builder
            .build_load(inferred_elem_type, elem_ptr, "elem")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    pub(super) fn generate_slice(
        &mut self,
        arr: &Expr,
        start: Option<&Spanned<Expr>>,
        end: Option<&Spanned<Expr>>,
        inclusive: bool,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self
            .current_function
            .ok_or_else(|| CodegenError::LlvmError("No current function for slice".to_string()))?;

        let arr_val = self.generate_expr(arr)?;

        // Determine if the source is a Slice/SliceMut (fat pointer struct with 2 fields)
        let is_slice_source = if let BasicValueEnum::StructValue(sv) = arr_val {
            let struct_type = sv.get_type();
            struct_type.count_fields() == 2
        } else {
            false
        };

        let arr_ptr = if is_slice_source {
            // Extract data pointer from fat pointer (field 0)
            let data_ptr = self
                .builder
                .build_extract_value(arr_val.into_struct_value(), 0, "slice_data")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_pointer_value();
            // Bitcast i8* to i64*
            self.builder
                .build_pointer_cast(
                    data_ptr,
                    self.context.i64_type().ptr_type(AddressSpace::default()),
                    "slice_ptr_typed",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        } else {
            arr_val.into_pointer_value()
        };

        // Get start index (default 0)
        let start_val = if let Some(start_expr) = start {
            self.generate_expr(&start_expr.node)?.into_int_value()
        } else {
            self.context.i64_type().const_int(0, false)
        };

        // Get end index
        let end_val = if let Some(end_expr) = end {
            let val = self.generate_expr(&end_expr.node)?.into_int_value();
            if inclusive {
                self.builder
                    .build_int_add(val, self.context.i64_type().const_int(1, false), "incl_end")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                val
            }
        } else {
            // Open-end slice: arr[start..]
            if is_slice_source {
                // Extract length from fat pointer (field 1)
                self.builder
                    .build_extract_value(arr_val.into_struct_value(), 1, "slice_len")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_int_value()
            } else {
                // Array/Pointer source doesn't have length information
                return Err(CodegenError::Unsupported(
                    "Open-end slicing requires a slice source; array length is unknown".to_string(),
                ));
            }
        };

        // Calculate slice length: end - start
        let length = self
            .builder
            .build_int_sub(end_val, start_val, "slice_len")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Allocate new array: malloc(length * 8)
        let byte_size = self
            .builder
            .build_int_mul(
                length,
                self.context.i64_type().const_int(8, false),
                "byte_size",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let malloc_fn = self.module.get_function("malloc").unwrap_or_else(|| {
            let fn_type = self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .fn_type(&[self.context.i64_type().into()], false);
            self.module.add_function("malloc", fn_type, None)
        });

        let raw_ptr = self
            .builder
            .build_call(malloc_fn, &[byte_size.into()], "slice_raw")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .try_as_basic_value()
            .left()
            .unwrap();
        let slice_ptr = self
            .builder
            .build_pointer_cast(
                raw_ptr.into_pointer_value(),
                self.context.i64_type().ptr_type(AddressSpace::default()),
                "slice_ptr",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Copy elements using a loop
        let loop_var = self
            .builder
            .build_alloca(self.context.i64_type(), "slice_i")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(loop_var, self.context.i64_type().const_int(0, false))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let loop_cond = self.context.append_basic_block(fn_value, "slice_cond");
        let loop_body = self.context.append_basic_block(fn_value, "slice_body");
        let loop_end = self.context.append_basic_block(fn_value, "slice_end");

        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Loop condition: i < length
        self.builder.position_at_end(loop_cond);
        let i = self
            .builder
            .build_load(self.context.i64_type(), loop_var, "i")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();
        let cmp = self
            .builder
            .build_int_compare(IntPredicate::SLT, i, length, "slice_cmp")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_conditional_branch(cmp, loop_body, loop_end)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Loop body: slice_ptr[i] = arr_ptr[start + i]
        self.builder.position_at_end(loop_body);
        let src_idx = self
            .builder
            .build_int_add(start_val, i, "src_idx")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let src_ptr = unsafe {
            self.builder
                .build_gep(self.context.i64_type(), arr_ptr, &[src_idx], "src_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };
        let elem = self
            .builder
            .build_load(self.context.i64_type(), src_ptr, "elem")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let dst_ptr = unsafe {
            self.builder
                .build_gep(self.context.i64_type(), slice_ptr, &[i], "dst_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };
        self.builder
            .build_store(dst_ptr, elem)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // i++
        let next_i = self
            .builder
            .build_int_add(i, self.context.i64_type().const_int(1, false), "next_i")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(loop_var, next_i)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // After loop
        self.builder.position_at_end(loop_end);
        Ok(slice_ptr.into())
    }

    // ========== Method Call ==========

    pub(super) fn generate_method_call(
        &mut self,
        receiver: &Expr,
        method: &str,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Transform method call to function call with receiver as first arg
        // e.g., obj.method(a, b) -> TypeName_method(obj, a, b)

        // Special case: Slice.len() / SliceMut.len() — fat pointer { i8*, i64 }
        // Field 1 is the length.  Restrict to known slice types to avoid triggering on
        // other 2-field structs such as Result or Optional.
        if method == "len" {
            // Check whether the receiver's resolved type is a Slice/SliceMut before
            // generating the expression value (avoids side-effect duplication).
            let is_slice_receiver = match receiver {
                Expr::Ident(name) => {
                    // Check if the variable's type name indicates a slice
                    let type_name = self.var_struct_types.get(name).map(|s| s.as_str());
                    matches!(type_name, Some("Slice") | Some("SliceMut"))
                        || self.locals.get(name).is_some_and(|(_, ty)| {
                            // Also accept if the LLVM type is { ptr, i64 } and is NOT a
                            // known named struct (i.e. an anonymous fat-pointer struct)
                            if let inkwell::types::BasicTypeEnum::StructType(st) = ty {
                                let nf = st.count_fields();
                                if nf == 2 {
                                    // Must not be a named struct registered in generated_structs
                                    !self.generated_structs.values().any(|known| known == st)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        })
                }
                _ => false,
            };

            if is_slice_receiver {
                let recv_val = self.generate_expr(receiver)?;
                if recv_val.is_struct_value() {
                    let struct_val = recv_val.into_struct_value();
                    if struct_val.get_type().count_fields() == 2 {
                        // Extract field 1 (the length i64)
                        let len_val = self
                            .builder
                            .build_extract_value(struct_val, 1, "slice_len")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        return Ok(len_val);
                    }
                }
            }
        }

        // Try to resolve the struct type name from the receiver
        let mut struct_name = self.infer_struct_name(receiver).ok();

        // For SelfCall (@), infer struct type from current function name (TypeName_method pattern)
        if struct_name.is_none() && matches!(receiver, Expr::SelfCall) {
            if let Some(func) = self.current_function {
                let fn_name = func.get_name().to_str().unwrap_or("").to_string();
                if let Some(idx) = fn_name.find('_') {
                    struct_name = Some(fn_name[..idx].to_string());
                }
            }
        }

        // Get receiver pointer for pass-by-reference self parameter
        let receiver_ptr: Option<PointerValue<'ctx>> = match receiver {
            Expr::Ident(name) => self.locals.get(name).map(|(ptr, _)| *ptr),
            Expr::SelfCall => {
                // @ in method context: self is already a pointer
                self.locals.get("self").map(|(ptr, _)| *ptr)
            }
            _ => None,
        };

        // Also generate the receiver value as fallback (for non-method calls or unknown receivers)
        let receiver_val = self.generate_expr(receiver)?;

        // Try qualified name: TypeName_method
        let qualified_name = struct_name.as_ref().map(|sn| format!("{}_{}", sn, method));

        let fn_value = qualified_name
            .as_ref()
            .and_then(|qn| {
                self.functions
                    .get(qn)
                    .copied()
                    .or_else(|| self.module.get_function(qn))
            })
            // Fallback: try bare method name
            .or_else(|| {
                self.functions
                    .get(method)
                    .copied()
                    .or_else(|| self.module.get_function(method))
            });

        // If not found, try broader search: look for any TypeName_method pattern.
        // When the receiver struct name is known, use it directly (deterministic).
        // Only fall back to iterating all structs when the name is unknown.
        let fn_value = if let Some(f) = fn_value {
            f
        } else if let Some(ref sn) = struct_name {
            // Struct name is known — avoid non-deterministic HashMap iteration.
            // The qualified name was already tried above; nothing more to do.
            let tried = format!("{}_{}", sn, method);
            return Err(CodegenError::UndefinedFunction(format!(
                "{} (method call on {:?})",
                tried, receiver
            )));
        } else {
            // Struct name unknown — scan all registered structs for a matching method.
            // Collect into a sorted Vec first to make the search order deterministic.
            let mut candidates: Vec<String> = self.generated_structs.keys().cloned().collect();
            candidates.sort();
            let mut found = None;
            for sn in &candidates {
                let candidate = format!("{}_{}", sn, method);
                if let Some(f) = self
                    .functions
                    .get(&candidate)
                    .copied()
                    .or_else(|| self.module.get_function(&candidate))
                {
                    found = Some(f);
                    break;
                }
            }
            if let Some(f) = found {
                f
            } else {
                let tried = qualified_name.as_deref().unwrap_or(method);
                return Err(CodegenError::UndefinedFunction(format!(
                    "{} (method call on {:?})",
                    tried, receiver
                )));
            }
        };

        // Generate arguments (receiver first, pass as pointer for methods)
        let mut arg_values: Vec<BasicMetadataValueEnum> = if let Some(ptr) = receiver_ptr {
            // Pass receiver as pointer (self by reference)
            vec![ptr.into()]
        } else {
            // Fallback: for struct literal receivers or complex expressions,
            // create a temporary alloca and pass its pointer
            if struct_name.is_some() {
                let alloca = self
                    .builder
                    .build_alloca(receiver_val.get_type(), "tmp_self")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder
                    .build_store(alloca, receiver_val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                vec![alloca.into()]
            } else {
                vec![receiver_val.into()]
            }
        };
        for arg in args {
            arg_values.push(self.generate_expr(&arg.node)?.into());
        }

        // Build call
        let call = self
            .builder
            .build_call(fn_value, &arg_values, "method_call")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(call
            .try_as_basic_value()
            .left()
            .unwrap_or_else(|| self.context.struct_type(&[], false).const_zero().into()))
    }

    // ========== Lambda/Closure ==========

    pub(super) fn generate_lambda(
        &mut self,
        params: &[ast::Param],
        body: &Expr,
        captures: &[String],
        capture_mode: ast::CaptureMode,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Generate unique lambda function name
        let lambda_name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        // Find captured variables from current scope
        // If captures list is empty (type checker didn't fill it), auto-detect from body
        let effective_captures: Vec<String> = if captures.is_empty() {
            let param_names: std::collections::HashSet<String> =
                params.iter().map(|p| p.name.node.clone()).collect();
            let used_idents = Self::collect_idents(body);
            used_idents
                .into_iter()
                .filter(|name| !param_names.contains(name) && self.locals.contains_key(name))
                .collect()
        } else {
            captures.to_vec()
        };

        let is_ref_capture = matches!(
            capture_mode,
            ast::CaptureMode::ByRef | ast::CaptureMode::ByMutRef
        );

        let mut captured_vars: Vec<(String, BasicValueEnum<'ctx>, BasicTypeEnum<'ctx>)> =
            Vec::new();
        for cap_name in &effective_captures {
            if let Some((ptr, var_type)) = self.locals.get(cap_name) {
                if is_ref_capture {
                    // ByRef/ByMutRef: pass the alloca pointer directly
                    // The pointer value itself is the captured value
                    captured_vars.push((
                        cap_name.clone(),
                        (*ptr).into(),
                        (*var_type)
                            .ptr_type(inkwell::AddressSpace::default())
                            .into(),
                    ));
                } else {
                    // By-value or explicit move: load and pass the value
                    let val = self
                        .builder
                        .build_load(*var_type, *ptr, &format!("cap_{}", cap_name))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    captured_vars.push((cap_name.clone(), val, *var_type));
                }
            }
        }

        // Build parameter types: captured vars first, then lambda params
        let mut param_types: Vec<BasicMetadataTypeEnum> = Vec::new();

        // First add captured variables as parameters
        for (_, _, cap_type) in &captured_vars {
            param_types.push((*cap_type).into());
        }

        // Then add original lambda parameters
        for p in params {
            let resolved = self.ast_type_to_resolved(&p.ty.node);
            param_types.push(self.type_mapper.map_type(&resolved).into());
        }

        // Create function type (always returns i64 for now)
        let fn_type = self.context.i64_type().fn_type(&param_types, false);
        let lambda_fn = self.module.add_function(&lambda_name, fn_type, None);
        self.lambda_functions.push(lambda_fn);

        // Save current state (move instead of clone to avoid HashMap allocation).
        // SAFETY: if generate_expr below returns Err, the entire codegen aborts,
        // so empty self.locals after take is acceptable (never accessed post-error).
        let saved_function = self.current_function;
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_insert_block = self.builder.get_insert_block();

        // Set up lambda context
        self.current_function = Some(lambda_fn);

        // Create entry block for lambda
        let entry = self.context.append_basic_block(lambda_fn, "entry");
        self.builder.position_at_end(entry);

        // Register captured variables as parameters in lambda scope
        let mut param_idx = 0u32;
        for (cap_name, _, cap_type) in &captured_vars {
            let param_val = lambda_fn.get_nth_param(param_idx).unwrap();
            if is_ref_capture {
                // ByRef/ByMutRef: parameter is already a pointer to the outer alloca.
                // Use it directly — build_load will read from the outer variable.
                let ptr_val = param_val.into_pointer_value();
                // Get the original value type from saved locals
                let val_type = saved_locals
                    .get(cap_name)
                    .map(|(_, t)| *t)
                    .unwrap_or_else(|| self.context.i64_type().into());
                self.locals.insert(cap_name.clone(), (ptr_val, val_type));
            } else {
                let alloca = self
                    .builder
                    .build_alloca(*cap_type, &format!("__cap_{}", cap_name))
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder
                    .build_store(alloca, param_val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.locals.insert(cap_name.clone(), (alloca, *cap_type));
            }
            param_idx += 1;
        }

        // Register original parameters
        for p in params {
            let param_val = lambda_fn.get_nth_param(param_idx).unwrap();
            let ty = self.ast_type_to_resolved(&p.ty.node);
            let param_type = self.type_mapper.map_type(&ty);
            let alloca = self
                .builder
                .build_alloca(param_type, &p.name.node)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals
                .insert(p.name.node.clone(), (alloca, param_type));
            param_idx += 1;
        }

        // Generate lambda body
        let body_val = self.generate_expr(body)?;

        // Add return
        self.builder
            .build_return(Some(&body_val))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Restore context
        self.current_function = saved_function;
        self.locals = saved_locals;
        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }

        // Register lambda as a callable function
        self.functions.insert(lambda_name.clone(), lambda_fn);

        // Store captured values for later use at call sites
        let captured_for_binding: Vec<(String, BasicValueEnum<'ctx>)> = captured_vars
            .iter()
            .map(|(name, val, _)| (name.clone(), *val))
            .collect();

        // Store the last lambda info so Stmt::Let can track it
        self._last_lambda_info = Some((lambda_name.clone(), captured_for_binding));

        // Return function pointer as i64
        let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
        let fn_int = self
            .builder
            .build_ptr_to_int(fn_ptr, self.context.i64_type(), "lambda_ptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(fn_int.into())
    }

    /// Collect all Ident names used in an expression (for auto-capture detection)
    pub(super) fn collect_idents(expr: &Expr) -> Vec<String> {
        let mut idents = Vec::new();
        Self::collect_idents_inner(expr, &mut idents);
        idents.sort();
        idents.dedup();
        idents
    }

    pub(super) fn collect_idents_inner(expr: &Expr, idents: &mut Vec<String>) {
        match expr {
            Expr::Ident(name) => idents.push(name.clone()),
            Expr::Binary { left, right, .. } => {
                Self::collect_idents_inner(&left.node, idents);
                Self::collect_idents_inner(&right.node, idents);
            }
            Expr::Unary { expr, .. } => Self::collect_idents_inner(&expr.node, idents),
            Expr::Call { func, args } => {
                Self::collect_idents_inner(&func.node, idents);
                for arg in args {
                    Self::collect_idents_inner(&arg.node, idents);
                }
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    if let Stmt::Expr(e) = &stmt.node {
                        Self::collect_idents_inner(&e.node, idents);
                    }
                }
            }
            Expr::If { cond, then, else_ } => {
                Self::collect_idents_inner(&cond.node, idents);
                for stmt in then {
                    if let Stmt::Expr(e) = &stmt.node {
                        Self::collect_idents_inner(&e.node, idents);
                    }
                }
                if let Some(else_branch) = else_ {
                    match else_branch {
                        IfElse::ElseIf(cond_expr, then_stmts, _else_opt) => {
                            Self::collect_idents_inner(&cond_expr.node, idents);
                            for stmt in then_stmts {
                                if let Stmt::Expr(e) = &stmt.node {
                                    Self::collect_idents_inner(&e.node, idents);
                                }
                            }
                        }
                        IfElse::Else(stmts) => {
                            for stmt in stmts {
                                if let Stmt::Expr(e) = &stmt.node {
                                    Self::collect_idents_inner(&e.node, idents);
                                }
                            }
                        }
                    }
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                Self::collect_idents_inner(&receiver.node, idents);
                for arg in args {
                    Self::collect_idents_inner(&arg.node, idents);
                }
            }
            Expr::Field { expr, .. } => Self::collect_idents_inner(&expr.node, idents),
            Expr::Index { expr, index } => {
                Self::collect_idents_inner(&expr.node, idents);
                Self::collect_idents_inner(&index.node, idents);
            }
            Expr::Tuple(elems) | Expr::Array(elems) => {
                for e in elems {
                    Self::collect_idents_inner(&e.node, idents);
                }
            }
            _ => {}
        }
    }
}
