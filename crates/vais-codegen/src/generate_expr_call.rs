//! Call expression code generation for LLVM IR.

use vais_ast::*;
use vais_types::ResolvedType;

use crate::{
    format_did_you_mean, suggest_similar, CodeGenerator, CodegenError, CodegenResult,
};

impl CodeGenerator {
    /// Handle function call expressions (builtins, regular calls, indirect calls)
    pub(crate) fn generate_expr_call(
        &mut self,
        func: &Spanned<Expr>,
        args: &[Spanned<Expr>],
        span: vais_ast::Span,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Check if this is an enum variant constructor or builtin
        if let Expr::Ident(name) = &func.node {
            // Struct tuple literal: `Response(200, 1)` → desugar to StructLit
            let resolved = self.resolve_struct_name(name);
            if self.types.structs.contains_key(&resolved) && !self.types.functions.contains_key(name) {
                if let Some(struct_info) = self.types.structs.get(&resolved).cloned() {
                    let fields: Vec<_> = struct_info
                        .fields
                        .iter()
                        .zip(args.iter())
                        .map(|((fname, _), val)| {
                            (vais_ast::Spanned::new(fname.clone(), val.span), val.clone())
                        })
                        .collect();
                    let struct_lit = vais_ast::Spanned::new(
                        Expr::StructLit {
                            name: vais_ast::Spanned::new(name.clone(), func.span),
                            fields,
                        },
                        span,
                    );
                    return self.generate_expr(&struct_lit, counter);
                }
            }

            // Handle print/println builtins with format string support
            if name == "print" || name == "println" {
                return self.generate_print_call(name, args, counter, span);
            }

            // Handle format builtin: returns formatted string
            if name == "format" {
                return self.generate_format_call(args, counter, span);
            }

            // Handle str_to_ptr builtin: convert string pointer to i64
            if name == "str_to_ptr" {
                if args.len() != 1 {
                    return Err(CodegenError::TypeError(
                        "str_to_ptr expects 1 argument".to_string(),
                    ));
                }
                let (str_val, str_ir) = self.generate_expr(&args[0], counter)?;
                let mut ir = str_ir;
                let result = self.next_temp(counter);
                ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, str_val));
                return Ok((result, ir));
            }

            // Handle ptr_to_str builtin: convert i64 to string pointer (i8*)
            // If value is already a pointer type, pass through directly
            if name == "ptr_to_str" {
                if args.len() != 1 {
                    return Err(CodegenError::TypeError(
                        "ptr_to_str expects 1 argument".to_string(),
                    ));
                }
                let (ptr_val, ptr_ir) = self.generate_expr(&args[0], counter)?;
                let mut ir = ptr_ir;
                // Only do inttoptr if the arg is actually an integer (i64)
                // If it's already a pointer (str, malloc result, etc.), pass through
                let arg_type = self.infer_expr_type(&args[0]);
                if matches!(arg_type, vais_types::ResolvedType::I64) {
                    let result = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        result, ptr_val
                    ));
                    return Ok((result, ir));
                }
                // Already a pointer/str, no conversion needed
                return Ok((ptr_val, ir));
            }

            // sizeof(expr) — compile-time constant size query
            // Also supports sizeof(T) where T is a generic type parameter
            if name == "sizeof" && !args.is_empty() {
                let arg_type = if let Expr::Ident(ident) = &args[0].node {
                    if let Some(concrete) = self.get_generic_substitution(ident) {
                        concrete
                    } else {
                        self.infer_expr_type(&args[0])
                    }
                } else {
                    self.infer_expr_type(&args[0])
                };
                let size = self.compute_sizeof(&arg_type);
                return Ok((size.to_string(), String::new()));
            }

            // alignof(expr) — compile-time constant alignment query
            // Also supports alignof(T) where T is a generic type parameter
            if name == "alignof" && !args.is_empty() {
                let arg_type = if let Expr::Ident(ident) = &args[0].node {
                    if let Some(concrete) = self.get_generic_substitution(ident) {
                        concrete
                    } else {
                        self.infer_expr_type(&args[0])
                    }
                } else {
                    self.infer_expr_type(&args[0])
                };
                let align = self.compute_alignof(&arg_type);
                return Ok((align.to_string(), String::new()));
            }

            // type_size() — compile-time size of current generic type T
            // Returns sizeof(T) where T is resolved from generic_substitutions
            // Used in generic containers like Vec<T> to get element size
            if name == "type_size" && args.is_empty() {
                let resolved_t = self
                    .get_generic_substitution("T")
                    .unwrap_or(ResolvedType::I64);
                let size = self.compute_sizeof(&resolved_t);
                return Ok((size.to_string(), String::new()));
            }

            // load_typed(ptr) -> T — type-aware memory load
            // Dispatches to correct LLVM load based on generic type T
            if name == "load_typed" && !args.is_empty() {
                let (ptr_val, ptr_ir) = self.generate_expr(&args[0], counter)?;
                let mut ir = ptr_ir;
                // Resolve T from generic substitutions
                let resolved_t = self
                    .get_generic_substitution("T")
                    .unwrap_or(ResolvedType::I64);
                let size = self.compute_sizeof(&resolved_t);
                let result = self.next_temp(counter);
                match size {
                    1 => {
                        let tmp1 = self.next_temp(counter);
                        let tmp2 = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to i8*\n",
                            tmp1, ptr_val
                        ));
                        ir.push_str(&format!("  {} = load i8, i8* {}\n", tmp2, tmp1));
                        ir.push_str(&format!("  {} = zext i8 {} to i64\n", result, tmp2));
                    }
                    2 => {
                        let tmp1 = self.next_temp(counter);
                        let tmp2 = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to i16*\n",
                            tmp1, ptr_val
                        ));
                        ir.push_str(&format!("  {} = load i16, i16* {}\n", tmp2, tmp1));
                        ir.push_str(&format!("  {} = zext i16 {} to i64\n", result, tmp2));
                    }
                    4 if matches!(resolved_t, ResolvedType::F32) => {
                        let tmp1 = self.next_temp(counter);
                        let tmp2 = self.next_temp(counter);
                        let tmp3 = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to float*\n",
                            tmp1, ptr_val
                        ));
                        ir.push_str(&format!("  {} = load float, float* {}\n", tmp2, tmp1));
                        ir.push_str(&format!(
                            "  {} = fpext float {} to double\n",
                            tmp3, tmp2
                        ));
                        ir.push_str(&format!(
                            "  {} = bitcast double {} to i64\n",
                            result, tmp3
                        ));
                    }
                    4 => {
                        let tmp1 = self.next_temp(counter);
                        let tmp2 = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to i32*\n",
                            tmp1, ptr_val
                        ));
                        ir.push_str(&format!("  {} = load i32, i32* {}\n", tmp2, tmp1));
                        ir.push_str(&format!("  {} = zext i32 {} to i64\n", result, tmp2));
                    }
                    _ if matches!(resolved_t, ResolvedType::F64) => {
                        let tmp1 = self.next_temp(counter);
                        let tmp2 = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to double*\n",
                            tmp1, ptr_val
                        ));
                        ir.push_str(&format!(
                            "  {} = load double, double* {}\n",
                            tmp2, tmp1
                        ));
                        ir.push_str(&format!(
                            "  {} = bitcast double {} to i64\n",
                            result, tmp2
                        ));
                    }
                    _ => {
                        let tmp1 = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to i64*\n",
                            tmp1, ptr_val
                        ));
                        ir.push_str(&format!("  {} = load i64, i64* {}\n", result, tmp1));
                    }
                }
                return Ok((result, ir));
            }

            // store_typed(ptr, val) — type-aware memory store
            // Dispatches to correct LLVM store based on generic type T
            if name == "store_typed" && args.len() >= 2 {
                let (ptr_val, ptr_ir) = self.generate_expr(&args[0], counter)?;
                let (val_val, val_ir) = self.generate_expr(&args[1], counter)?;
                let mut ir = ptr_ir;
                ir.push_str(&val_ir);
                let resolved_t = self
                    .get_generic_substitution("T")
                    .unwrap_or(ResolvedType::I64);
                let size = self.compute_sizeof(&resolved_t);
                match size {
                    1 => {
                        let tmp1 = self.next_temp(counter);
                        let tmp2 = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to i8*\n",
                            tmp1, ptr_val
                        ));
                        ir.push_str(&format!("  {} = trunc i64 {} to i8\n", tmp2, val_val));
                        ir.push_str(&format!("  store i8 {}, i8* {}\n", tmp2, tmp1));
                    }
                    2 => {
                        let tmp1 = self.next_temp(counter);
                        let tmp2 = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to i16*\n",
                            tmp1, ptr_val
                        ));
                        ir.push_str(&format!(
                            "  {} = trunc i64 {} to i16\n",
                            tmp2, val_val
                        ));
                        ir.push_str(&format!("  store i16 {}, i16* {}\n", tmp2, tmp1));
                    }
                    4 if matches!(resolved_t, ResolvedType::F32) => {
                        let tmp1 = self.next_temp(counter);
                        let tmp2 = self.next_temp(counter);
                        let tmp3 = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to float*\n",
                            tmp1, ptr_val
                        ));
                        ir.push_str(&format!(
                            "  {} = bitcast i64 {} to double\n",
                            tmp2, val_val
                        ));
                        ir.push_str(&format!(
                            "  {} = fptrunc double {} to float\n",
                            tmp3, tmp2
                        ));
                        ir.push_str(&format!("  store float {}, float* {}\n", tmp3, tmp1));
                    }
                    4 => {
                        let tmp1 = self.next_temp(counter);
                        let tmp2 = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to i32*\n",
                            tmp1, ptr_val
                        ));
                        ir.push_str(&format!(
                            "  {} = trunc i64 {} to i32\n",
                            tmp2, val_val
                        ));
                        ir.push_str(&format!("  store i32 {}, i32* {}\n", tmp2, tmp1));
                    }
                    _ if matches!(resolved_t, ResolvedType::F64) => {
                        let tmp1 = self.next_temp(counter);
                        let tmp2 = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to double*\n",
                            tmp1, ptr_val
                        ));
                        ir.push_str(&format!(
                            "  {} = bitcast i64 {} to double\n",
                            tmp2, val_val
                        ));
                        ir.push_str(&format!(
                            "  store double {}, double* {}\n",
                            tmp2, tmp1
                        ));
                    }
                    _ => {
                        let tmp1 = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to i64*\n",
                            tmp1, ptr_val
                        ));
                        ir.push_str(&format!("  store i64 {}, i64* {}\n", val_val, tmp1));
                    }
                }
                return Ok(("0".to_string(), ir));
            }

            // swap(ptr, idx1, idx2) — delegate to __swap helper
            // Uses ptrtoint for ptr→i64 conversion (Vais pointers are i64 internally)
            if name == "swap" && args.len() >= 3 {
                let (ptr_val, ptr_ir) = self.generate_expr(&args[0], counter)?;
                let (idx1_val, idx1_ir) = self.generate_expr(&args[1], counter)?;
                let (idx2_val, idx2_ir) = self.generate_expr(&args[2], counter)?;
                let mut ir = ptr_ir;
                ir.push_str(&idx1_ir);
                ir.push_str(&idx2_ir);

                // Convert ptr to i64 for __swap(i64, i64, i64) signature
                let ptr_i64 = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = ptrtoint ptr {} to i64\n",
                    ptr_i64, ptr_val
                ));

                let dbg_info = self.debug_info.dbg_ref_from_span(span);
                ir.push_str(&format!(
                    "  call void @__swap(i64 {}, i64 {}, i64 {}){}\n",
                    ptr_i64, idx1_val, idx2_val, dbg_info
                ));

                return Ok(("void".to_string(), ir));
            }

            if let Some((enum_name, tag)) = self.get_tuple_variant_info(name) {
                // This is a tuple enum variant constructor
                let mut ir = String::new();

                // Generate argument values
                let mut arg_vals = Vec::new();
                for arg in args {
                    let (val, arg_ir) = self.generate_expr(arg, counter)?;
                    ir.push_str(&arg_ir);
                    arg_vals.push(val);
                }

                // Create enum value on stack: { i32 tag, i64 payload }
                let enum_ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = alloca %{}\n", enum_ptr, enum_name));

                // Store tag
                let tag_ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0\n",
                    tag_ptr, enum_name, enum_name, enum_ptr
                ));
                ir.push_str(&format!("  store i32 {}, i32* {}\n", tag, tag_ptr));

                // Store payload fields into the payload sub-struct
                for (i, arg_val) in arg_vals.iter().enumerate() {
                    let payload_field_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr %{}, %{}* {}, i32 0, i32 1, i32 {}\n",
                        payload_field_ptr, enum_name, enum_name, enum_ptr, i
                    ));
                    ir.push_str(&format!(
                        "  store i64 {}, i64* {}\n",
                        arg_val, payload_field_ptr
                    ));
                }

                // Return pointer to the enum
                return Ok((enum_ptr, ir));
            }

            // Check if this is a SIMD intrinsic
            if Self::is_simd_intrinsic(name) {
                return self.generate_simd_intrinsic(name, args, counter);
            }

            // Handle print_i64/print_f64 builtins: emit printf call
            // Skip if user defined their own function with the same name
            let has_user_print_i64 = self
                .types.functions
                .get("print_i64")
                .map(|f| !f.is_extern)
                .unwrap_or(false);
            if name == "print_i64" && args.len() == 1 && !has_user_print_i64 {
                return self.generate_print_i64_builtin(args, counter);
            }

            let has_user_print_f64 = self
                .types.functions
                .get("print_f64")
                .map(|f| !f.is_extern)
                .unwrap_or(false);
            if name == "print_f64" && args.len() == 1 && !has_user_print_f64 {
                return self.generate_print_f64_builtin(args, counter);
            }
        }

        // Check if this is a direct function call or indirect (lambda) call
        let (fn_name, is_indirect) = if let Expr::Ident(name) = &func.node {
            // Check if this is a generic function that needs monomorphization
            if let Some(instantiations_list) = self.generics.fn_instantiations.get(name) {
                // Infer argument types to select the right specialization
                let arg_types: Vec<ResolvedType> =
                    args.iter().map(|a| self.infer_expr_type(a)).collect();

                // Find the matching instantiation based on argument types
                let mangled =
                    self.resolve_generic_call(name, &arg_types, instantiations_list);
                (mangled, false)
            } else if self.types.functions.contains_key(name) {
                (name.clone(), false)
            } else if self.fn_ctx.locals.contains_key(name) {
                (name.clone(), true) // Lambda call
            } else if self.types.declared_functions.contains(name) {
                // Function declared in module (may be generic, will instantiate later)
                (name.clone(), false)
            } else {
                // Unknown function - provide suggestions
                let mut candidates: Vec<&str> = Vec::new();

                // Add declared function names (including generics)
                for func_name in &self.types.declared_functions {
                    candidates.push(func_name.as_str());
                }

                // Add instantiated function names
                for func_name in self.types.functions.keys() {
                    candidates.push(func_name.as_str());
                }

                // Add local variables (could be lambdas)
                for var_name in self.fn_ctx.locals.keys() {
                    candidates.push(var_name.as_str());
                }

                let suggestions = suggest_similar(name, &candidates, 3);
                let suggestion_text = format_did_you_mean(&suggestions);
                return Err(CodegenError::UndefinedFunction(format!(
                    "{}{}",
                    name, suggestion_text
                )));
            }
        } else if let Expr::SelfCall = &func.node {
            (self.fn_ctx.current_function.clone().unwrap_or_default(), false)
        } else {
            return Err(CodegenError::Unsupported(
                "complex indirect call".to_string(),
            ));
        };

        // Look up function info for parameter types (only for direct calls)
        let fn_info = if !is_indirect {
            self.types.functions.get(&fn_name).cloned()
        } else {
            None
        };

        let mut ir = String::new();
        let mut arg_vals = Vec::new();

        for (i, arg) in args.iter().enumerate() {
            let (mut val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);

            // Get parameter type from function info if available
            let param_ty = fn_info
                .as_ref()
                .and_then(|f| f.signature.params.get(i))
                .map(|(_, ty, _)| ty.clone());

            let arg_ty = if let Some(ref ty) = param_ty {
                self.type_to_llvm(ty)
            } else {
                // For vararg arguments, infer the type from the expression
                let inferred_ty = self.infer_expr_type(arg);
                self.type_to_llvm(&inferred_ty)
            };

            // For struct arguments, load the value if we have a pointer
            // (struct literals generate alloca+stores, returning pointers)
            if let Some(ResolvedType::Named { .. }) = &param_ty {
                // Check if val looks like a pointer (starts with %)
                if val.starts_with('%') {
                    let loaded = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        loaded, arg_ty, arg_ty, val
                    ));
                    val = loaded;
                }
            }

            // Trait object coercion: &ConcreteType -> &dyn Trait
            // When parameter expects &dyn Trait and argument is a concrete type reference,
            // create a fat pointer { data_ptr, vtable_ptr }
            if let Some(ref param_type) = param_ty {
                let dyn_trait = match param_type {
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                        if let ResolvedType::DynTrait { trait_name, .. } = inner.as_ref() {
                            Some(trait_name.clone())
                        } else {
                            None
                        }
                    }
                    ResolvedType::DynTrait { trait_name, .. } => Some(trait_name.clone()),
                    _ => None,
                };

                if let Some(trait_name) = dyn_trait {
                    // Get the concrete type of the argument
                    let arg_expr_type = self.infer_expr_type(arg);
                    let concrete_type_name = match &arg_expr_type {
                        ResolvedType::Named { name, .. } => Some(name.clone()),
                        ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                            if let ResolvedType::Named { name, .. } = inner.as_ref() {
                                Some(name.clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    if let Some(concrete_name) = concrete_type_name {
                        // Generate vtable for this concrete type + trait
                        let vtable_info =
                            self.get_or_generate_vtable(&concrete_name, &trait_name);

                        if let Some(vtable) = vtable_info {
                            // Load the actual struct pointer if we have a pointer-to-pointer
                            // (Ref expressions return the address of the storage, not the struct)
                            let struct_ptr = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = load %{}*, %{}** {}\n",
                                struct_ptr, concrete_name, concrete_name, val
                            ));
                            // Cast data pointer to i8*
                            let data_ptr = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = bitcast %{}* {} to i8*\n",
                                data_ptr, concrete_name, struct_ptr
                            ));

                            // Create fat pointer { i8*, i8* }
                            let trait_obj_1 = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = insertvalue {{ i8*, i8* }} undef, i8* {}, 0\n",
                                trait_obj_1, data_ptr
                            ));
                            let vtable_cast = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = bitcast {{ i8*, i64, i64, i64(i8*)* }}* {} to i8*\n",
                                vtable_cast, vtable.global_name
                            ));
                            let trait_obj_2 = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = insertvalue {{ i8*, i8* }} {}, i8* {}, 1\n",
                                trait_obj_2, trait_obj_1, vtable_cast
                            ));

                            val = trait_obj_2;
                        }
                    }
                }
            }

            // Insert integer conversion if needed (trunc for narrowing, sext for widening)
            if let Some(param_type) = &param_ty {
                let src_bits = self.get_integer_bits_from_val(&val);
                let dst_bits = self.get_integer_bits(param_type);

                if src_bits > 0 && dst_bits > 0 && src_bits != dst_bits {
                    let conv_tmp = self.next_temp(counter);
                    let src_ty = format!("i{}", src_bits);
                    let dst_ty = format!("i{}", dst_bits);

                    if src_bits > dst_bits {
                        // Truncate
                        ir.push_str(&format!(
                            "  {} = trunc {} {} to {}\n",
                            conv_tmp, src_ty, val, dst_ty
                        ));
                    } else {
                        // Sign extend
                        ir.push_str(&format!(
                            "  {} = sext {} {} to {}\n",
                            conv_tmp, src_ty, val, dst_ty
                        ));
                    }
                    val = conv_tmp;
                }
            }

            // Convert i64 to i8* when parameter expects str/i8* but arg is i64
            if let Some(ref param_type) = param_ty {
                if matches!(param_type, ResolvedType::Str) {
                    let actual_ty = self.infer_expr_type(arg);
                    if matches!(actual_ty, ResolvedType::I64) {
                        let ptr_tmp = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to i8*\n",
                            ptr_tmp, val
                        ));
                        val = ptr_tmp;
                    }
                }
            }

            arg_vals.push(format!("{} {}", arg_ty, val));
        }

        // Get return type and actual function name (may differ for builtins)
        let ret_ty = fn_info
            .as_ref()
            .map(|f| self.type_to_llvm(&f.signature.ret))
            .unwrap_or_else(|| "i64".to_string());

        let actual_fn_name = fn_info
            .as_ref()
            .map(|f| f.signature.name.clone())
            .unwrap_or_else(|| fn_name.clone());

        let is_vararg = fn_info
            .as_ref()
            .map(|f| f.signature.is_vararg)
            .unwrap_or(false);

        if is_indirect {
            // Check if this is a closure with captured variables
            let closure_info = self.lambdas.closures.get(&fn_name).cloned();

            // Prepend captured values to arguments if this is a closure
            let mut all_args = Vec::new();
            if let Some(ref info) = closure_info {
                for (_, capture_val) in &info.captures {
                    all_args.push(format!("i64 {}", capture_val));
                }
            }
            all_args.extend(arg_vals);

            // If we have closure info, we know the exact function name - call directly
            if let Some(ref info) = closure_info {
                let tmp = self.next_temp(counter);
                let dbg_info = self.debug_info.dbg_ref_from_span(span);
                ir.push_str(&format!(
                    "  {} = call i64 @{}({}){}\n",
                    tmp,
                    info.func_name,
                    all_args.join(", "),
                    dbg_info
                ));
                return Ok((tmp, ir));
            }

            // Get the local variable info
            let local_info = self.fn_ctx.locals.get(&fn_name).cloned();
            let is_ssa_or_param = local_info
                .as_ref()
                .map(|l| l.is_ssa() || l.is_param())
                .unwrap_or(false);

            let ptr_tmp = if is_ssa_or_param {
                // SSA or param: the value IS the function pointer (as i64), no load needed
                let local = match local_info.as_ref() {
                    Some(l) => l,
                    None => {
                        return Err(CodegenError::TypeError(format!(
                            "missing local info for '{}'",
                            fn_name
                        )))
                    }
                };
                let val = &local.llvm_name;
                if local.is_ssa() {
                    // SSA values already include the % prefix (e.g., "%5")
                    val.clone()
                } else {
                    // Param names don't include % prefix
                    format!("%{}", val)
                }
            } else {
                // Alloca: load the function pointer from the stack slot
                let llvm_var_name = local_info
                    .as_ref()
                    .map(|l| l.llvm_name.clone())
                    .unwrap_or_else(|| fn_name.clone());
                let tmp = self.next_temp(counter);
                ir.push_str(&format!("  {} = load i64, i64* %{}\n", tmp, llvm_var_name));
                tmp
            };

            // Build function type signature for indirect call (including captures)
            let arg_types: Vec<String> = all_args
                .iter()
                .map(|a| a.split_whitespace().next().unwrap_or("i64").to_string())
                .collect();
            let fn_type = format!("i64 ({})*", arg_types.join(", "));

            // Cast i64 to function pointer
            let fn_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = inttoptr i64 {} to {}\n",
                fn_ptr, ptr_tmp, fn_type
            ));

            // Make indirect call with all arguments
            let tmp = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            ir.push_str(&format!(
                "  {} = call i64 {}({}){}\n",
                tmp,
                fn_ptr,
                all_args.join(", "),
                dbg_info
            ));
            Ok((tmp, ir))
        } else if fn_name == "malloc" {
            // Special handling for malloc: call returns i8*, convert to i64
            let ptr_tmp = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            ir.push_str(&format!(
                "  {} = call i8* @malloc({}){}\n",
                ptr_tmp,
                arg_vals.join(", "),
                dbg_info
            ));
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, ptr_tmp));
            Ok((result, ir))
        } else if fn_name == "free" {
            // Special handling for free: convert i64 to i8*
            let ptr_tmp = self.next_temp(counter);
            // Extract the i64 value from arg_vals
            let arg_val = arg_vals
                .first()
                .map(|s| s.split_whitespace().last().unwrap_or("0"))
                .unwrap_or("0");
            ir.push_str(&format!(
                "  {} = inttoptr i64 {} to i8*\n",
                ptr_tmp, arg_val
            ));
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            ir.push_str(&format!("  call void @free(i8* {}){}\n", ptr_tmp, dbg_info));
            Ok(("void".to_string(), ir))
        } else if fn_name == "memcpy" || fn_name == "memcpy_str" {
            // Special handling for memcpy/memcpy_str: convert pointers as needed
            let dest_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
            let src_full = arg_vals.get(1).map(|s| s.as_str()).unwrap_or("i64 0");
            let n_val = arg_vals
                .get(2)
                .map(|s| s.split_whitespace().last().unwrap_or("0"))
                .unwrap_or("0");

            // Handle dest pointer
            let dest_ptr = if dest_full.starts_with("i8*") {
                // Use everything after "i8* " to preserve complex expressions
                dest_full
                    .strip_prefix("i8* ")
                    .unwrap_or(dest_full.split_whitespace().last().unwrap_or("null"))
                    .to_string()
            } else {
                let dest_val = dest_full.split_whitespace().last().unwrap_or("0");
                let ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = inttoptr i64 {} to i8*\n", ptr, dest_val));
                ptr
            };

            // Handle src pointer (can be i64 or i8* for memcpy_str)
            let src_ptr = if src_full.starts_with("i8*") {
                // Use everything after "i8* " to preserve complex expressions
                src_full
                    .strip_prefix("i8* ")
                    .unwrap_or(src_full.split_whitespace().last().unwrap_or("null"))
                    .to_string()
            } else {
                let src_val = src_full.split_whitespace().last().unwrap_or("0");
                let ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = inttoptr i64 {} to i8*\n", ptr, src_val));
                ptr
            };

            let result = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            ir.push_str(&format!(
                "  {} = call i8* @memcpy(i8* {}, i8* {}, i64 {}){}\n",
                result, dest_ptr, src_ptr, n_val, dbg_info
            ));
            // Convert result back to i64
            let result_i64 = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = ptrtoint i8* {} to i64\n",
                result_i64, result
            ));
            Ok((result_i64, ir))
        } else if fn_name == "strlen" {
            // Special handling for strlen: convert i64 to i8* if needed
            let arg_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
            let result = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);

            // Check if the argument is already i8* (str type) or i64 (pointer as integer)
            if arg_full.starts_with("i8*") {
                // Already a pointer, use directly
                // Use everything after "i8* " to preserve complex expressions like getelementptr
                let ptr_val = arg_full
                    .strip_prefix("i8* ")
                    .unwrap_or(arg_full.split_whitespace().last().unwrap_or("null"));
                ir.push_str(&format!(
                    "  {} = call i64 @strlen(i8* {}){}\n",
                    result, ptr_val, dbg_info
                ));
            } else {
                // Convert i64 to pointer
                let arg_val = arg_full.split_whitespace().last().unwrap_or("0");
                let ptr_tmp = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = inttoptr i64 {} to i8*\n",
                    ptr_tmp, arg_val
                ));
                ir.push_str(&format!(
                    "  {} = call i64 @strlen(i8* {}){}\n",
                    result, ptr_tmp, dbg_info
                ));
            }
            Ok((result, ir))
        } else if fn_name == "puts_ptr" {
            // Special handling for puts_ptr: convert i64 to i8*
            let arg_val = arg_vals
                .first()
                .map(|s| s.split_whitespace().last().unwrap_or("0"))
                .unwrap_or("0");
            let ptr_tmp = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = inttoptr i64 {} to i8*\n",
                ptr_tmp, arg_val
            ));
            let i32_result = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            ir.push_str(&format!(
                "  {} = call i32 @puts(i8* {}){}\n",
                i32_result, ptr_tmp, dbg_info
            ));
            // Convert i32 result to i64 for consistency
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = sext i32 {} to i64\n", result, i32_result));
            Ok((result, ir))
        } else if ret_ty == "void" {
            // Check for recursive call with decreases clause
            if self.is_recursive_call(&fn_name) {
                let check_ir = self.generate_recursive_decreases_check(args, counter)?;
                ir.push_str(&check_ir);
            }

            // Direct void function call
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            if is_vararg {
                let param_types: Vec<String> = fn_info
                    .as_ref()
                    .map(|f| {
                        f.signature
                            .params
                            .iter()
                            .map(|(_, ty, _)| self.type_to_llvm(ty))
                            .collect()
                    })
                    .unwrap_or_default();
                let sig = format!("void ({}, ...)", param_types.join(", "));
                ir.push_str(&format!(
                    "  call {} @{}({}){}\n",
                    sig,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            } else {
                ir.push_str(&format!(
                    "  call void @{}({}){}\n",
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            }
            Ok(("void".to_string(), ir))
        } else if ret_ty == "i32" {
            // Check for recursive call with decreases clause
            if self.is_recursive_call(&fn_name) {
                let check_ir = self.generate_recursive_decreases_check(args, counter)?;
                ir.push_str(&check_ir);
            }

            // i32 return function call - convert to i64 for consistency
            let i32_tmp = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            if is_vararg {
                // Variadic functions need explicit signature in LLVM IR call
                let param_types: Vec<String> = fn_info
                    .as_ref()
                    .map(|f| {
                        f.signature
                            .params
                            .iter()
                            .map(|(_, ty, _)| self.type_to_llvm(ty))
                            .collect()
                    })
                    .unwrap_or_default();
                let sig = format!("i32 ({}, ...)", param_types.join(", "));
                ir.push_str(&format!(
                    "  {} = call {} @{}({}){}\n",
                    i32_tmp,
                    sig,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            } else {
                ir.push_str(&format!(
                    "  {} = call i32 @{}({}){}\n",
                    i32_tmp,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            }
            let tmp = self.next_temp(counter);
            ir.push_str(&format!("  {} = sext i32 {} to i64\n", tmp, i32_tmp));
            Ok((tmp, ir))
        } else {
            // Check for recursive call with decreases clause
            if self.is_recursive_call(&fn_name) {
                let check_ir = self.generate_recursive_decreases_check(args, counter)?;
                ir.push_str(&check_ir);
            }

            // Direct function call with return value
            let tmp = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            if is_vararg {
                let param_types: Vec<String> = fn_info
                    .as_ref()
                    .map(|f| {
                        f.signature
                            .params
                            .iter()
                            .map(|(_, ty, _)| self.type_to_llvm(ty))
                            .collect()
                    })
                    .unwrap_or_default();
                let sig = format!("{} ({}, ...)", ret_ty, param_types.join(", "));
                ir.push_str(&format!(
                    "  {} = call {} @{}({}){}\n",
                    tmp,
                    sig,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            } else {
                ir.push_str(&format!(
                    "  {} = call {} @{}({}){}\n",
                    tmp,
                    ret_ty,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            }
            Ok((tmp, ir))
        }
    }
}
