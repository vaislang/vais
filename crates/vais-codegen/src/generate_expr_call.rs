//! Call expression code generation for LLVM IR.

use vais_ast::{Expr, Spanned};
use vais_types::ResolvedType;

use crate::{format_did_you_mean, suggest_similar, CodeGenerator, CodegenError, CodegenResult};

impl CodeGenerator {
    /// Handle function call expressions (builtins, regular calls, indirect calls)
    #[inline(never)]
    pub(crate) fn generate_expr_call(
        &mut self,
        func: &Spanned<Expr>,
        args: &[Spanned<Expr>],
        span: vais_ast::Span,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Check if this is an enum variant constructor or builtin
        if let Expr::Ident(name) = &func.node {
            // Result/Option variant constructors: look up actual tag from enum registry,
            // falling back to hardcoded values if not registered.
            // NOTE: Tag must match the enum definition order (e.g., E Option { None=0, Some=1 })
            match name.as_str() {
                "Ok" => {
                    let tag = self.get_enum_variant_tag("Ok");
                    return self.generate_enum_variant_constructor("Result", tag, args, counter);
                }
                "Err" => {
                    let tag = self.get_enum_variant_tag("Err");
                    return self.generate_enum_variant_constructor("Result", tag, args, counter);
                }
                "Some" => {
                    let tag = self.get_enum_variant_tag("Some");
                    return self.generate_enum_variant_constructor("Option", tag, args, counter);
                }
                _ => {}
            }

            // Struct tuple literal: `Response(200, 1)` → desugar to StructLit
            let resolved = self.resolve_struct_name(name);
            if self.types.structs.contains_key(&resolved)
                && !self.types.functions.contains_key(name)
            {
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
                            enum_name: None,
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

            // Handle str_to_ptr builtin: extract i8* pointer from string fat pointer, convert to i64
            if name == "str_to_ptr" {
                if args.len() != 1 {
                    return Err(CodegenError::TypeError(
                        "str_to_ptr expects 1 argument".to_string(),
                    ));
                }
                let (str_val, str_ir) = self.generate_expr(&args[0], counter)?;
                let mut ir = str_ir;
                // Extract raw i8* from fat pointer { i8*, i64 }
                let raw_ptr = self.extract_str_ptr(&str_val, counter, &mut ir);
                let result = self.next_temp(counter);
                write_ir!(ir, "  {} = ptrtoint i8* {} to i64", result, raw_ptr);
                return Ok((result, ir));
            }

            // Handle ptr_to_str builtin: convert i64 to string fat pointer { i8*, i64 }
            if name == "ptr_to_str" {
                if args.len() != 1 {
                    return Err(CodegenError::TypeError(
                        "ptr_to_str expects 1 argument".to_string(),
                    ));
                }
                let (ptr_val, ptr_ir) = self.generate_expr(&args[0], counter)?;
                let mut ir = ptr_ir;
                let arg_type = self.infer_expr_type(&args[0]);
                let raw_ptr = if matches!(arg_type, vais_types::ResolvedType::I64) {
                    let tmp = self.next_temp(counter);
                    write_ir!(ir, "  {} = inttoptr i64 {} to i8*", tmp, ptr_val);
                    tmp
                } else if matches!(arg_type, vais_types::ResolvedType::Str) {
                    // Already a str fat pointer — extract the raw pointer
                    self.extract_str_ptr(&ptr_val, counter, &mut ir)
                } else {
                    // Assume it's already an i8* pointer
                    ptr_val
                };
                // Compute length with strlen and build fat pointer
                let len = self.next_temp(counter);
                write_ir!(ir, "  {} = call i64 @strlen(i8* {})", len, raw_ptr);
                let result = self.build_str_fat_ptr(&raw_ptr, &len, counter, &mut ir);
                return Ok((result, ir));
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
                return self.generate_load_typed(args, counter);
            }

            // store_typed(ptr, val) — type-aware memory store
            // Dispatches to correct LLVM store based on generic type T
            if name == "store_typed" && args.len() >= 2 {
                return self.generate_store_typed(args, counter);
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
                write_ir!(ir, "  {} = ptrtoint ptr {} to i64", ptr_i64, ptr_val);

                let dbg_info = self.debug_info.dbg_ref_from_span(span);
                write_ir!(
                    ir,
                    "  call void @__swap(i64 {}, i64 {}, i64 {}){}",
                    ptr_i64,
                    idx1_val,
                    idx2_val,
                    dbg_info
                );

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
                write_ir!(ir, "  {} = alloca %{}", enum_ptr, enum_name);

                // Store tag
                let tag_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0",
                    tag_ptr,
                    enum_name,
                    enum_name,
                    enum_ptr
                );
                write_ir!(ir, "  store i32 {}, i32* {}", tag, tag_ptr);

                // Store payload fields into the payload sub-struct
                for (i, arg_val) in arg_vals.iter().enumerate() {
                    let payload_field_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr %{}, %{}* {}, i32 0, i32 1, i32 {}",
                        payload_field_ptr,
                        enum_name,
                        enum_name,
                        enum_ptr,
                        i
                    );
                    write_ir!(ir, "  store i64 {}, i64* {}", arg_val, payload_field_ptr);
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
                .types
                .functions
                .get("print_i64")
                .map(|f| !f.is_extern)
                .unwrap_or(false);
            if name == "print_i64" && args.len() == 1 && !has_user_print_i64 {
                return self.generate_print_i64_builtin(args, counter);
            }

            let has_user_print_f64 = self
                .types
                .functions
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
                let mangled = self.resolve_generic_call(name, &arg_types, instantiations_list);
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
            // Inside an async poll function (e.g., countdown__poll), @(args) should
            // call the create function (countdown), not the poll function itself.
            let cur = self.fn_ctx.current_function.clone().unwrap_or_default();
            let base = if cur.ends_with("__poll") {
                cur.trim_end_matches("__poll").to_string()
            } else {
                cur
            };
            (base, false)
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

        // Check if this is an extern C function (needs Str→i8* extraction at call boundary)
        let is_extern_c = fn_info.as_ref().map(|f| f.is_extern).unwrap_or(false);

        for (i, arg) in args.iter().enumerate() {
            let (mut val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);

            // Get parameter type from function info if available
            let param_ty = fn_info
                .as_ref()
                .and_then(|f| f.signature.params.get(i))
                .map(|(_, ty, _)| ty.clone());

            // For extern C functions with Str parameters, extract the raw i8* pointer
            // from the fat pointer { i8*, i64 } and use i8* as the argument type
            if is_extern_c {
                if let Some(ResolvedType::Str) = &param_ty {
                    let raw_ptr = self.extract_str_ptr(&val, counter, &mut ir);
                    val = raw_ptr;
                    arg_vals.push(format!("i8* {}", val));
                    continue;
                }
            }

            let inferred_ty = self.infer_expr_type(arg);
            let arg_ty = if let Some(ref ty) = param_ty {
                if matches!(ty, ResolvedType::Generic(_)) {
                    self.type_to_llvm(&inferred_ty)
                } else {
                    self.type_to_llvm(ty)
                }
            } else {
                // For vararg arguments, infer the type from the expression
                self.type_to_llvm(&inferred_ty)
            };

            // Integer width coercion: if param expects i32 but expr produces i64, trunc
            if let Some(ref pt) = param_ty {
                let src_bits = self.get_integer_bits(&inferred_ty);
                let dst_bits = self.get_integer_bits(pt);
                if src_bits > 0 && dst_bits > 0 && src_bits != dst_bits {
                    let conv_tmp = self.next_temp(counter);
                    let src_ty = format!("i{}", src_bits);
                    let dst_ty = format!("i{}", dst_bits);
                    if src_bits > dst_bits {
                        write_ir!(ir, "  {} = trunc {} {} to {}", conv_tmp, src_ty, val, dst_ty);
                    } else {
                        write_ir!(ir, "  {} = sext {} {} to {}", conv_tmp, src_ty, val, dst_ty);
                    }
                    val = conv_tmp;
                }
            }

            // For struct arguments, load the value if we have a pointer
            // (struct literals generate alloca+stores, returning pointers)
            // Use both param_ty and inferred_ty for struct detection
            let type_to_check = match &param_ty {
                Some(ty) => ty.clone(),
                None => inferred_ty,
            };
            if matches!(type_to_check, ResolvedType::Named { .. }) && !self.is_expr_value(arg) {
                let loaded = self.next_temp(counter);
                write_ir!(ir, "  {} = load {}, {}* {}", loaded, arg_ty, arg_ty, val);
                val = loaded;
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
                        let vtable_result =
                            self.get_or_generate_vtable(&concrete_name, &trait_name);

                        match vtable_result {
                            Ok(vtable) => {
                                // Load the actual struct pointer if we have a pointer-to-pointer
                                // (Ref expressions return the address of the storage, not the struct)
                                let struct_ptr = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = load %{}*, %{}** {}",
                                    struct_ptr,
                                    concrete_name,
                                    concrete_name,
                                    val
                                );
                                // Cast data pointer to i8*
                                let data_ptr = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = bitcast %{}* {} to i8*",
                                    data_ptr,
                                    concrete_name,
                                    struct_ptr
                                );

                                // Create fat pointer { i8*, i8* }
                                let trait_obj_1 = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = insertvalue {{ i8*, i8* }} undef, i8* {}, 0",
                                    trait_obj_1,
                                    data_ptr
                                );
                                let vtable_cast = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = bitcast {{ i8*, i64, i64, i64(i8*)* }}* {} to i8*",
                                    vtable_cast,
                                    vtable.global_name
                                );
                                let trait_obj_2 = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = insertvalue {{ i8*, i8* }} {}, i8* {}, 1",
                                    trait_obj_2,
                                    trait_obj_1,
                                    vtable_cast
                                );

                                val = trait_obj_2;
                            }
                            Err(e) => {
                                // Propagate the error - missing required trait method
                                return Err(e);
                            }
                        }
                    }
                }
            }

            // NOTE: Integer width coercion is already handled above (lines 496-511)
            // using the type system (infer_expr_type + get_integer_bits).
            // A duplicate conversion was removed here in Phase 144 because
            // get_integer_bits_from_val() incorrectly assumed all %vars are i64,
            // causing double trunc (e.g., trunc i64 %t0 to i32 when %t0 is already i32).

            // Convert i64 to str fat pointer { i8*, i64 } when parameter expects str but arg is i64
            if let Some(ref param_type) = param_ty {
                if matches!(param_type, ResolvedType::Str) {
                    let actual_ty = self.infer_expr_type(arg);
                    if matches!(actual_ty, ResolvedType::I64) {
                        let raw_ptr = self.next_temp(counter);
                        write_ir!(ir, "  {} = inttoptr i64 {} to i8*", raw_ptr, val);
                        let len = self.next_temp(counter);
                        write_ir!(ir, "  {} = call i64 @strlen(i8* {})", len, raw_ptr);
                        val = self.build_str_fat_ptr(&raw_ptr, &len, counter, &mut ir);
                    }
                }
            }

            arg_vals.push(format!("{} {}", arg_ty, val));
        }

        // Fill in default parameter values for omitted trailing arguments
        let param_count = fn_info
            .as_ref()
            .map(|f| f.signature.params.len())
            .unwrap_or(args.len());
        if args.len() < param_count {
            // Clone the default param expressions to avoid borrow conflict with &mut self
            let defaults: Option<Vec<Option<Box<vais_ast::Spanned<vais_ast::Expr>>>>> =
                self.types.default_params.get(&fn_name).cloned();
            if let Some(defaults) = defaults {
                for i in args.len()..param_count {
                    if let Some(Some(default_expr)) = defaults.get(i) {
                        let param_ty = fn_info
                            .as_ref()
                            .and_then(|f| f.signature.params.get(i))
                            .map(|(_, ty, _)| ty.clone());
                        let arg_ty = if let Some(ref pt) = param_ty {
                            self.type_to_llvm(pt)
                        } else {
                            "i64".to_string()
                        };
                        let (val, default_ir) = self.generate_expr(default_expr, counter)?;
                        ir.push_str(&default_ir);
                        arg_vals.push(format!("{} {}", arg_ty, val));
                    }
                }
            }
        }

        // Get return type and actual function name (may differ for builtins)
        // Async functions always return i64 (state pointer) from their create function,
        // regardless of declared return type. The value is extracted via poll function.
        // Extern C functions use C ABI types (Str→i8* instead of { i8*, i64 }).
        let ret_ty = fn_info
            .as_ref()
            .map(|f| {
                if f.signature.is_async {
                    "i64".to_string()
                } else if is_extern_c {
                    self.type_to_llvm_extern(&f.signature.ret)
                } else {
                    self.type_to_llvm(&f.signature.ret)
                }
            })
            .or_else(|| {
                // Fallback: check resolved_function_sigs from type checker.
                // This handles methods from imported modules (e.g., TestSuite_new, ByteBuffer_new)
                // that weren't registered in self.types.functions during codegen init.
                self.types
                    .resolved_function_sigs
                    .get(&fn_name)
                    .map(|sig| self.type_to_llvm(&sig.ret))
            })
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
                if info.is_ref_capture {
                    // Reference capture: pass pointers to captured variables
                    for (_, capture_val) in &info.captures {
                        all_args.push(format!("i64* {}", capture_val));
                    }
                } else {
                    for (_, capture_val) in &info.captures {
                        all_args.push(format!("i64 {}", capture_val));
                    }
                }
            }
            all_args.extend(arg_vals);

            // If we have closure info, we know the exact function name - call directly
            if let Some(ref info) = closure_info {
                let tmp = self.next_temp(counter);
                let dbg_info = self.debug_info.dbg_ref_from_span(span);
                write_ir!(
                    ir,
                    "  {} = call i64 @{}({}){}",
                    tmp,
                    info.func_name,
                    all_args.join(", "),
                    dbg_info
                );
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
                write_ir!(ir, "  {} = load i64, i64* %{}", tmp, llvm_var_name);
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
            write_ir!(ir, "  {} = inttoptr i64 {} to {}", fn_ptr, ptr_tmp, fn_type);

            // Make indirect call with all arguments
            let tmp = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            write_ir!(
                ir,
                "  {} = call i64 {}({}){}",
                tmp,
                fn_ptr,
                all_args.join(", "),
                dbg_info
            );
            Ok((tmp, ir))
        } else if fn_name == "malloc" {
            // Special handling for malloc: call returns i8*, convert to i64
            let ptr_tmp = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            write_ir!(
                ir,
                "  {} = call i8* @malloc({}){}",
                ptr_tmp,
                arg_vals.join(", "),
                dbg_info
            );
            let result = self.next_temp(counter);
            write_ir!(ir, "  {} = ptrtoint i8* {} to i64", result, ptr_tmp);
            Ok((result, ir))
        } else if fn_name == "free" {
            // Special handling for free: handle str fat pointer, i8*, or i64
            let arg_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
            let ptr_tmp = if arg_full.starts_with("{ i8*, i64 }") {
                let val = arg_full
                    .strip_prefix("{ i8*, i64 } ")
                    .unwrap_or(arg_full.split_whitespace().last().unwrap_or("undef"));
                let ptr = self.next_temp(counter);
                write_ir!(ir, "  {} = extractvalue {{ i8*, i64 }} {}, 0", ptr, val);
                ptr
            } else if arg_full.starts_with("i8*") {
                arg_full
                    .split_whitespace()
                    .last()
                    .unwrap_or("null")
                    .to_string()
            } else {
                let arg_val = arg_full.split_whitespace().last().unwrap_or("0");
                let ptr = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i8*", ptr, arg_val);
                ptr
            };
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            write_ir!(ir, "  call void @free(i8* {}){}", ptr_tmp, dbg_info);
            Ok(("void".to_string(), ir))
        } else if fn_name == "memcpy" || fn_name == "memcpy_str" {
            // Special handling for memcpy/memcpy_str: convert pointers as needed
            let dest_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
            let src_full = arg_vals.get(1).map(|s| s.as_str()).unwrap_or("i64 0");
            let n_val = arg_vals
                .get(2)
                .map(|s| s.split_whitespace().last().unwrap_or("0"))
                .unwrap_or("0");

            // Handle dest pointer — can be { i8*, i64 } (str fat ptr), i8*, or i64
            let dest_ptr = if dest_full.starts_with("{ i8*, i64 }") {
                let val = dest_full
                    .strip_prefix("{ i8*, i64 } ")
                    .unwrap_or(dest_full.split_whitespace().last().unwrap_or("undef"));
                let ptr = self.next_temp(counter);
                write_ir!(ir, "  {} = extractvalue {{ i8*, i64 }} {}, 0", ptr, val);
                ptr
            } else if dest_full.starts_with("i8*") {
                dest_full
                    .strip_prefix("i8* ")
                    .unwrap_or(dest_full.split_whitespace().last().unwrap_or("null"))
                    .to_string()
            } else {
                let dest_val = dest_full.split_whitespace().last().unwrap_or("0");
                let ptr = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i8*", ptr, dest_val);
                ptr
            };

            // Handle src pointer — can be { i8*, i64 } (str fat ptr), i8*, or i64
            let src_ptr = if src_full.starts_with("{ i8*, i64 }") {
                let val = src_full
                    .strip_prefix("{ i8*, i64 } ")
                    .unwrap_or(src_full.split_whitespace().last().unwrap_or("undef"));
                let ptr = self.next_temp(counter);
                write_ir!(ir, "  {} = extractvalue {{ i8*, i64 }} {}, 0", ptr, val);
                ptr
            } else if src_full.starts_with("i8*") {
                src_full
                    .strip_prefix("i8* ")
                    .unwrap_or(src_full.split_whitespace().last().unwrap_or("null"))
                    .to_string()
            } else {
                let src_val = src_full.split_whitespace().last().unwrap_or("0");
                let ptr = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i8*", ptr, src_val);
                ptr
            };

            let result = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            write_ir!(
                ir,
                "  {} = call i8* @memcpy(i8* {}, i8* {}, i64 {}){}",
                result,
                dest_ptr,
                src_ptr,
                n_val,
                dbg_info
            );
            // Convert result back to i64
            let result_i64 = self.next_temp(counter);
            write_ir!(ir, "  {} = ptrtoint i8* {} to i64", result_i64, result);
            Ok((result_i64, ir))
        } else if fn_name == "strlen" {
            // Special handling for strlen: extract i8* from various argument types
            let arg_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
            let result = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);

            // Handle different argument types: str fat pointer { i8*, i64 }, raw i8*, or i64
            if arg_full.starts_with("{ i8*, i64 }") {
                // String fat pointer — extract the raw i8* pointer
                let val = arg_full
                    .strip_prefix("{ i8*, i64 } ")
                    .unwrap_or(arg_full.split_whitespace().last().unwrap_or("undef"));
                let ptr_tmp = self.next_temp(counter);
                write_ir!(ir, "  {} = extractvalue {{ i8*, i64 }} {}, 0", ptr_tmp, val);
                write_ir!(
                    ir,
                    "  {} = call i64 @strlen(i8* {}){}",
                    result,
                    ptr_tmp,
                    dbg_info
                );
            } else if arg_full.starts_with("i8*") {
                // Already a pointer, use directly
                let ptr_val = arg_full
                    .strip_prefix("i8* ")
                    .unwrap_or(arg_full.split_whitespace().last().unwrap_or("null"));
                write_ir!(
                    ir,
                    "  {} = call i64 @strlen(i8* {}){}",
                    result,
                    ptr_val,
                    dbg_info
                );
            } else {
                // Convert i64 to pointer
                let arg_val = arg_full.split_whitespace().last().unwrap_or("0");
                let ptr_tmp = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i8*", ptr_tmp, arg_val);
                write_ir!(
                    ir,
                    "  {} = call i64 @strlen(i8* {}){}",
                    result,
                    ptr_tmp,
                    dbg_info
                );
            }
            Ok((result, ir))
        } else if fn_name == "puts_ptr" {
            // Special handling for puts_ptr: handle str fat pointer, i8*, or i64
            let arg_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
            let ptr_tmp = if arg_full.starts_with("{ i8*, i64 }") {
                let val = arg_full
                    .strip_prefix("{ i8*, i64 } ")
                    .unwrap_or(arg_full.split_whitespace().last().unwrap_or("undef"));
                let ptr = self.next_temp(counter);
                write_ir!(ir, "  {} = extractvalue {{ i8*, i64 }} {}, 0", ptr, val);
                ptr
            } else if arg_full.starts_with("i8*") {
                arg_full
                    .split_whitespace()
                    .last()
                    .unwrap_or("null")
                    .to_string()
            } else {
                let arg_val = arg_full.split_whitespace().last().unwrap_or("0");
                let ptr = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i8*", ptr, arg_val);
                ptr
            };
            let i32_result = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            write_ir!(
                ir,
                "  {} = call i32 @puts(i8* {}){}",
                i32_result,
                ptr_tmp,
                dbg_info
            );
            // Convert i32 result to i64 for consistency
            let result = self.next_temp(counter);
            write_ir!(ir, "  {} = sext i32 {} to i64", result, i32_result);
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
                            .map(|(_, ty, _)| {
                                if is_extern_c {
                                    self.type_to_llvm_extern(ty)
                                } else {
                                    self.type_to_llvm(ty)
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let sig = format!("void ({}, ...)", param_types.join(", "));
                write_ir!(
                    ir,
                    "  call {} @{}({}){}",
                    sig,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                );
            } else {
                write_ir!(
                    ir,
                    "  call void @{}({}){}",
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                );
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
                            .map(|(_, ty, _)| {
                                if is_extern_c {
                                    self.type_to_llvm_extern(ty)
                                } else {
                                    self.type_to_llvm(ty)
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let sig = format!("i32 ({}, ...)", param_types.join(", "));
                write_ir!(
                    ir,
                    "  {} = call {} @{}({}){}",
                    i32_tmp,
                    sig,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                );
            } else {
                write_ir!(
                    ir,
                    "  {} = call i32 @{}({}){}",
                    i32_tmp,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                );
            }
            let tmp = self.next_temp(counter);
            write_ir!(ir, "  {} = sext i32 {} to i64", tmp, i32_tmp);
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
                            .map(|(_, ty, _)| {
                                if is_extern_c {
                                    self.type_to_llvm_extern(ty)
                                } else {
                                    self.type_to_llvm(ty)
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let sig = format!("{} ({}, ...)", ret_ty, param_types.join(", "));
                write_ir!(
                    ir,
                    "  {} = call {} @{}({}){}",
                    tmp,
                    sig,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                );
            } else {
                write_ir!(
                    ir,
                    "  {} = call {} @{}({}){}",
                    tmp,
                    ret_ty,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                );
            }

            // For extern C functions returning str, the C function returns i8*
            // but Vais expects { i8*, i64 } fat pointer. Wrap the result.
            if is_extern_c {
                let actual_ret = fn_info
                    .as_ref()
                    .map(|f| f.signature.ret.clone())
                    .unwrap_or(ResolvedType::I64);
                if matches!(actual_ret, ResolvedType::Str) {
                    let len = self.next_temp(counter);
                    write_ir!(ir, "  {} = call i64 @strlen(i8* {})", len, tmp);
                    let fat_ptr = self.build_str_fat_ptr(&tmp, &len, counter, &mut ir);
                    return Ok((fat_ptr, ir));
                }
            }

            Ok((tmp, ir))
        }
    }

    /// Type-aware memory load for generic type T.
    /// Extracted from generate_expr_call to reduce stack frame size.
    #[inline(never)]
    fn generate_load_typed(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
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
                write_ir!(ir, "  {} = inttoptr i64 {} to i8*", tmp1, ptr_val);
                write_ir!(ir, "  {} = load i8, i8* {}", tmp2, tmp1);
                write_ir!(ir, "  {} = zext i8 {} to i64", result, tmp2);
            }
            2 => {
                let tmp1 = self.next_temp(counter);
                let tmp2 = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i16*", tmp1, ptr_val);
                write_ir!(ir, "  {} = load i16, i16* {}", tmp2, tmp1);
                write_ir!(ir, "  {} = zext i16 {} to i64", result, tmp2);
            }
            4 if matches!(resolved_t, ResolvedType::F32) => {
                let tmp1 = self.next_temp(counter);
                let tmp2 = self.next_temp(counter);
                let tmp3 = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to float*", tmp1, ptr_val);
                write_ir!(ir, "  {} = load float, float* {}", tmp2, tmp1);
                write_ir!(ir, "  {} = fpext float {} to double", tmp3, tmp2);
                write_ir!(ir, "  {} = bitcast double {} to i64", result, tmp3);
            }
            4 => {
                let tmp1 = self.next_temp(counter);
                let tmp2 = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i32*", tmp1, ptr_val);
                write_ir!(ir, "  {} = load i32, i32* {}", tmp2, tmp1);
                write_ir!(ir, "  {} = zext i32 {} to i64", result, tmp2);
            }
            _ if matches!(resolved_t, ResolvedType::F64) => {
                let tmp1 = self.next_temp(counter);
                let tmp2 = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to double*", tmp1, ptr_val);
                write_ir!(ir, "  {} = load double, double* {}", tmp2, tmp1);
                write_ir!(ir, "  {} = bitcast double {} to i64", result, tmp2);
            }
            n if n > 8 && matches!(resolved_t, ResolvedType::Named { .. } | ResolvedType::Str) => {
                // Large struct: copy via memcpy from the array slot to a stack
                // alloca. Return the alloca pointer — struct values in Text IR
                // are pointer-based, and field access uses GEP on typed pointers.
                self.needs_llvm_memcpy = true;
                let llvm_ty = self.type_to_llvm(&resolved_t);
                // Use result as the alloca directly
                write_ir!(ir, "  {} = alloca {}", result, llvm_ty);
                let dst_ptr = self.next_temp(counter);
                write_ir!(ir, "  {} = bitcast {}* {} to i8*", dst_ptr, llvm_ty, result);
                let src_ptr = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i8*", src_ptr, ptr_val);
                write_ir!(
                    ir,
                    "  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {}, i8* {}, i64 {}, i1 false)",
                    dst_ptr,
                    src_ptr,
                    n
                );
            }
            _ => {
                let tmp1 = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i64*", tmp1, ptr_val);
                write_ir!(ir, "  {} = load i64, i64* {}", result, tmp1);
            }
        }
        Ok((result, ir))
    }

    /// Type-aware memory store for generic type T.
    /// Extracted from generate_expr_call to reduce stack frame size.
    #[inline(never)]
    fn generate_store_typed(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
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
                write_ir!(ir, "  {} = inttoptr i64 {} to i8*", tmp1, ptr_val);
                write_ir!(ir, "  {} = trunc i64 {} to i8", tmp2, val_val);
                write_ir!(ir, "  store i8 {}, i8* {}", tmp2, tmp1);
            }
            2 => {
                let tmp1 = self.next_temp(counter);
                let tmp2 = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i16*", tmp1, ptr_val);
                write_ir!(ir, "  {} = trunc i64 {} to i16", tmp2, val_val);
                write_ir!(ir, "  store i16 {}, i16* {}", tmp2, tmp1);
            }
            4 if matches!(resolved_t, ResolvedType::F32) => {
                let tmp1 = self.next_temp(counter);
                let tmp2 = self.next_temp(counter);
                let tmp3 = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to float*", tmp1, ptr_val);
                write_ir!(ir, "  {} = bitcast i64 {} to double", tmp2, val_val);
                write_ir!(ir, "  {} = fptrunc double {} to float", tmp3, tmp2);
                write_ir!(ir, "  store float {}, float* {}", tmp3, tmp1);
            }
            4 => {
                let tmp1 = self.next_temp(counter);
                let tmp2 = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i32*", tmp1, ptr_val);
                write_ir!(ir, "  {} = trunc i64 {} to i32", tmp2, val_val);
                write_ir!(ir, "  store i32 {}, i32* {}", tmp2, tmp1);
            }
            _ if matches!(resolved_t, ResolvedType::F64) => {
                let tmp1 = self.next_temp(counter);
                let tmp2 = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to double*", tmp1, ptr_val);
                write_ir!(ir, "  {} = bitcast i64 {} to double", tmp2, val_val);
                write_ir!(ir, "  store double {}, double* {}", tmp2, tmp1);
            }
            n if n > 8 && matches!(resolved_t, ResolvedType::Named { .. } | ResolvedType::Str) => {
                // Large type: store via memcpy
                self.needs_llvm_memcpy = true;
                let dst_ptr = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i8*", dst_ptr, ptr_val);
                let llvm_ty = self.type_to_llvm(&resolved_t);
                // If val is a value (not pointer), alloca+store to get a pointer
                let src_ptr = if self.is_expr_value(&args[1]) || matches!(resolved_t, ResolvedType::Str) {
                    let tmp_alloca = format!("%__store_typed_tmp.{}", counter);
                    *counter += 1;
                    self.emit_entry_alloca(&tmp_alloca, &llvm_ty);
                    write_ir!(ir, "  store {} {}, {}* {}", llvm_ty, val_val, llvm_ty, tmp_alloca);
                    let cast = self.next_temp(counter);
                    write_ir!(ir, "  {} = bitcast {}* {} to i8*", cast, llvm_ty, tmp_alloca);
                    cast
                } else {
                    let cast = self.next_temp(counter);
                    write_ir!(ir, "  {} = bitcast {}* {} to i8*", cast, llvm_ty, val_val);
                    cast
                };
                write_ir!(
                    ir,
                    "  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {}, i8* {}, i64 {}, i1 false)",
                    dst_ptr,
                    src_ptr,
                    n
                );
            }
            _ => {
                let tmp1 = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i64*", tmp1, ptr_val);
                write_ir!(ir, "  store i64 {}, i64* {}", val_val, tmp1);
            }
        }
        Ok(("0".to_string(), ir))
    }
}
