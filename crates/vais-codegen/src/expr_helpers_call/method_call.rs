use super::*;
use vais_ast::{Expr, Span, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Generate memcpy call
    pub(super) fn generate_memcpy_call(
        &mut self,
        arg_vals: &[String],
        counter: &mut usize,
        span: Span,
        ir: &mut String,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
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
        write_ir!(
            ir,
            "  {} = call i8* @memcpy(i8* {}, i8* {}, i64 {}){}",
            result,
            dest_ptr,
            src_ptr,
            n_val,
            dbg_info
        );
        let result_i64 = self.next_temp(counter);
        write_ir!(ir, "  {} = ptrtoint i8* {} to i64", result_i64, result);
        Ok((result_i64, std::mem::take(ir)))
    }

    /// Generate strlen call
    pub(super) fn generate_strlen_call(
        &mut self,
        arg_vals: &[String],
        counter: &mut usize,
        span: Span,
        ir: &mut String,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let arg_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
        let result = self.next_temp(counter);

        // Handle different argument types: str fat pointer { i8*, i64 }, raw i8*, or i64
        if arg_full.starts_with("{ i8*, i64 }") {
            // String fat pointer — extract the raw i8* pointer
            let arg_val = arg_full
                .strip_prefix("{ i8*, i64 } ")
                .unwrap_or(arg_full.split_whitespace().last().unwrap_or("undef"));
            let ptr_tmp = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                ptr_tmp,
                arg_val
            );
            write_ir!(
                ir,
                "  {} = call i64 @strlen(i8* {}){}",
                result,
                ptr_tmp,
                dbg_info
            );
        } else if arg_full.starts_with("i8*") {
            // Already a raw pointer, use directly
            let ptr_val = arg_full.split_whitespace().last().unwrap_or("null");
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
        Ok((result, std::mem::take(ir)))
    }

    /// Generate puts_ptr call
    pub(super) fn generate_puts_ptr_call(
        &mut self,
        arg_vals: &[String],
        counter: &mut usize,
        span: Span,
        ir: &mut String,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let arg_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");

        // Handle different argument types: str fat pointer { i8*, i64 }, raw i8*, or i64
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
        Ok((result, std::mem::take(ir)))
    }

    /// Generate if expression
    #[inline(never)]
    pub(crate) fn generate_method_call_expr(
        &mut self,
        receiver: &Spanned<Expr>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (recv_val, recv_ir, recv_type) = if matches!(&receiver.node, Expr::SelfCall) {
            if let Some(local) = self.fn_ctx.locals.get("self") {
                let recv_type = local.ty.clone();
                ("%self".to_string(), String::new(), recv_type)
            } else {
                return Err(CodegenError::Unsupported(
                    "@.method() used outside of a method with self".to_string(),
                ));
            }
        } else {
            let (recv_val, recv_ir) = self.generate_expr(receiver, counter)?;
            let recv_type = self.infer_expr_type(receiver);
            (recv_val, recv_ir, recv_type)
        };
        let mut ir = recv_ir;

        let method_name = &method.node;

        // String method calls: str.len(), str.charAt(), str.contains(), etc.
        if matches!(recv_type, ResolvedType::Str) {
            return self.generate_string_method_call(&recv_val, &ir, method_name, args, counter);
        }

        // clone() on any type — return the receiver value unchanged
        if method_name == "clone" && args.is_empty() {
            return Ok((recv_val, ir));
        }

        // Slice .len() — extract length from fat pointer { i8*, i64 } field 1
        if method_name == "len" {
            let is_slice_type = match &recv_type {
                ResolvedType::Slice(_) | ResolvedType::SliceMut(_) => true,
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => matches!(
                    inner.as_ref(),
                    ResolvedType::Slice(_) | ResolvedType::SliceMut(_)
                ),
                _ => false,
            };
            if is_slice_type {
                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {{ i8*, i64 }} {}, 1",
                    result,
                    recv_val
                );
                return Ok((result, ir));
            }
        }

        // Use resolve_struct_name to match definition naming (e.g., Pair → Pair$i64)
        // For generic structs with type args, try mangled name first (e.g., Vec_push$GraphNode)
        // Unwrap Ref/RefMut to get the inner Named type
        let inner_recv_type = match &recv_type {
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref(),
            other => other,
        };
        let full_method_name = if let ResolvedType::Named { name, generics } = inner_recv_type {
            let resolved = self.resolve_struct_name(name);
            let base = format!("{}_{}", resolved, method_name);

            if !generics.is_empty() {
                // Set generic substitutions so that compute_sizeof/type_size resolve T correctly
                // when processing method arguments for generic containers like Vec<T>
                if let Some(struct_def) = self.generics.struct_defs.get(name).cloned() {
                    for (param, concrete) in struct_def.generics.iter().zip(generics.iter()) {
                        if !matches!(param.kind, vais_ast::GenericParamKind::Lifetime { .. }) {
                            self.generics
                                .substitutions
                                .entry(param.name.node.clone())
                                .or_insert_with(|| concrete.clone());
                        }
                    }
                }
                // Try mangled specialized name (Vec_push$GraphNode format)
                let mangled = vais_types::mangle_name(&base, generics);
                if self.types.functions.contains_key(&mangled) {
                    mangled
                } else {
                    base
                }
            } else {
                // Generics are empty — this happens in SINGLE_MODULE mode where
                // the receiver type is Named{Vec, []} instead of Named{Vec, [Str]}.
                // Try to infer the generic type arguments from method arguments
                // and route to a specialized function if one exists.

                // Strategy 1: Use fn_instantiations if available (registered during
                // module_gen/instantiations.rs processing of GenericInstantiation records).
                if let Some(inst_list) = self.generics.fn_instantiations.get(&base).cloned() {
                    let arg_types: Vec<ResolvedType> =
                        args.iter().map(|a| self.infer_expr_type(a)).collect();
                    let resolved = self.resolve_generic_call(&base, &arg_types, &inst_list);
                    if self.types.functions.contains_key(&resolved) {
                        resolved
                    } else {
                        base
                    }
                }
                // Strategy 2: No fn_instantiations entry, but the struct is a known
                // generic container. Infer T from the first argument's type and try
                // to construct the mangled name directly.
                else if (self.generics.struct_defs.contains_key(name)
                    || self
                        .generics
                        .generic_method_bodies
                        .keys()
                        .any(|(s, _)| s == name))
                    && !args.is_empty()
                {
                    let arg_types: Vec<ResolvedType> =
                        args.iter().map(|a| self.infer_expr_type(a)).collect();
                    // Filter out non-informative types (I64 is the default fallback)
                    let informative_args: Vec<&ResolvedType> = arg_types
                        .iter()
                        .filter(|t| {
                            !matches!(
                                t,
                                ResolvedType::I64 | ResolvedType::Generic(_) | ResolvedType::Var(_)
                            )
                        })
                        .collect();

                    if !informative_args.is_empty() {
                        // Build candidate type args from informative argument types.
                        // For single-generic containers (Vec<T>), use the first
                        // informative arg. For multi-generic (HashMap<K,V>), use
                        // up to the number of struct generic params.
                        let struct_def = self.generics.struct_defs.get(name).cloned();
                        let n_generic_params = struct_def
                            .as_ref()
                            .map(|s| {
                                s.generics
                                    .iter()
                                    .filter(|g| {
                                        !matches!(
                                            g.kind,
                                            vais_ast::GenericParamKind::Lifetime { .. }
                                        )
                                    })
                                    .count()
                            })
                            .unwrap_or(1);

                        let inferred_type_args: Vec<ResolvedType> = informative_args
                            .iter()
                            .take(n_generic_params)
                            .map(|t| (*t).clone())
                            .collect();

                        if !inferred_type_args.is_empty() {
                            // Set generic substitutions so downstream code resolves T correctly
                            if let Some(ref sd) = struct_def {
                                for (param, concrete) in
                                    sd.generics.iter().zip(inferred_type_args.iter())
                                {
                                    if !matches!(
                                        param.kind,
                                        vais_ast::GenericParamKind::Lifetime { .. }
                                    ) {
                                        self.generics
                                            .substitutions
                                            .entry(param.name.node.clone())
                                            .or_insert_with(|| concrete.clone());
                                    }
                                }
                            }

                            let mangled = vais_types::mangle_name(&base, &inferred_type_args);
                            if self.types.functions.contains_key(&mangled) {
                                mangled
                            } else {
                                // Try on-demand specialization: generate the specialized
                                // function if we have the method template and type args
                                let generated = self.try_generate_vec_specialization(
                                    name,
                                    method_name,
                                    &inferred_type_args,
                                    counter,
                                );
                                if let Some(gen_name) = generated {
                                    gen_name
                                } else {
                                    base
                                }
                            }
                        } else {
                            base
                        }
                    } else {
                        base
                    }
                } else {
                    base
                }
            }
        } else {
            // Receiver type is not Named (e.g., I64 from fallback inference).
            // Try to find the correct struct-qualified method name by searching
            // self.types.functions for any `StructName_method` that matches.
            let method_suffix = format!("_{}", method_name);
            let mut candidates: Vec<String> = Vec::new();
            for fn_name in self.types.functions.keys() {
                if fn_name.ends_with(&method_suffix) {
                    candidates.push(fn_name.clone());
                }
            }
            // Also check resolved_function_sigs from type checker
            for fn_name in self.types.resolved_function_sigs.keys() {
                if fn_name.ends_with(&method_suffix) && !candidates.contains(fn_name) {
                    candidates.push(fn_name.clone());
                }
            }
            // If exactly one struct provides this method, use it unambiguously
            if candidates.len() == 1 {
                candidates.into_iter().next().unwrap()
            } else {
                method_name.clone()
            }
        };

        let recv_llvm_ty = if matches!(&recv_type, ResolvedType::Named { .. }) {
            format!("{}*", self.type_to_llvm(&recv_type))
        } else {
            self.type_to_llvm(&recv_type)
        };

        // Look up function info for parameter types
        let fn_info = self.types.functions.get(&full_method_name).cloned();

        let mut arg_vals = vec![format!("{} {}", recv_llvm_ty, recv_val)];

        for (i, arg) in args.iter().enumerate() {
            let (mut val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);
            let inferred_ty = self.infer_expr_type(arg);

            // Method params: index 0 is self, so args[i] corresponds to params[i+1]
            let param_ty = fn_info
                .as_ref()
                .and_then(|f| f.signature.params.get(i + 1))
                .map(|(_, ty, _)| ty.clone())
                // Fallback: check resolved_function_sigs from type checker
                // This handles cross-module methods not registered in self.types.functions
                .or_else(|| {
                    self.types
                        .resolved_function_sigs
                        .get(&full_method_name)
                        .and_then(|sig| sig.params.get(i + 1))
                        .map(|(_, ty, _)| ty.clone())
                });

            // Use parameter type from signature if available, unless generic
            let arg_llvm_ty = if let Some(ref pt) = param_ty {
                if matches!(pt, ResolvedType::Generic(_)) {
                    // Generic params are erased to i64 in LLVM IR
                    // Use i64 as the LLVM type, and coerce if needed
                    "i64".to_string()
                } else {
                    self.type_to_llvm(pt)
                }
            } else {
                self.type_to_llvm(&inferred_ty)
            };

            // Integer width coercion: coerce to match arg_llvm_ty
            {
                let src_bits = self.get_integer_bits(&inferred_ty);
                // Parse dst_bits from arg_llvm_ty (e.g., "i64" -> 64)
                let dst_bits = if arg_llvm_ty.starts_with('i') {
                    arg_llvm_ty[1..].parse::<u32>().unwrap_or(0)
                } else if let Some(ref pt) = param_ty {
                    self.get_integer_bits(pt)
                } else {
                    0
                };
                if src_bits > 0 && dst_bits > 0 && src_bits != dst_bits {
                    let conv_tmp = self.next_temp(counter);
                    let src_ty = format!("i{}", src_bits);
                    let dst_ty = format!("i{}", dst_bits);
                    if src_bits > dst_bits {
                        write_ir!(
                            ir,
                            "  {} = trunc {} {} to {}",
                            conv_tmp,
                            src_ty,
                            val,
                            dst_ty
                        );
                    } else {
                        write_ir!(ir, "  {} = zext {} {} to {}", conv_tmp, src_ty, val, dst_ty);
                    }
                    val = conv_tmp;
                }
            }

            // Float/double coercion: fpext float→double or fptrunc double→float
            {
                let src_is_float = matches!(inferred_ty, ResolvedType::F32);
                let src_is_double = matches!(inferred_ty, ResolvedType::F64);
                let dst_is_double = arg_llvm_ty == "double";
                let dst_is_float = arg_llvm_ty == "float";
                if src_is_float && dst_is_double {
                    let conv_tmp = self.next_temp(counter);
                    write_ir!(ir, "  {} = fpext float {} to double", conv_tmp, val);
                    val = conv_tmp;
                } else if src_is_double && dst_is_float {
                    let conv_tmp = self.next_temp(counter);
                    write_ir!(ir, "  {} = fptrunc double {} to float", conv_tmp, val);
                    val = conv_tmp;
                }
            }

            // For struct types passed to generic (T→i64) params: store struct, ptrtoint to i64
            // EXCEPT: if a specialized function exists for this arg type, skip the conversion
            // and pass the struct value directly.
            let arg_inferred = self.infer_expr_type(arg);
            let skip_erasure = if matches!(
                &arg_inferred,
                ResolvedType::Named { .. } | ResolvedType::Str
            ) {
                // Check if a specialized version exists for this method + arg type
                let spec_name = vais_types::mangle_name(
                    &format!(
                        "{}_{}",
                        self.resolve_struct_name(
                            if let ResolvedType::Named { name, .. } = &recv_type {
                                name
                            } else {
                                "Unknown"
                            }
                        ),
                        method_name
                    ),
                    &[arg_inferred.clone()],
                );
                self.types.functions.contains_key(&spec_name)
                    || self.generics.generic_method_bodies.contains_key(&(
                        if let ResolvedType::Named { name, .. } = &recv_type {
                            name.clone()
                        } else {
                            String::new()
                        },
                        method_name.to_string(),
                    ))
            } else {
                false
            };
            if matches!(&arg_inferred, ResolvedType::Named { .. }) && !skip_erasure {
                let struct_llvm = self.type_to_llvm(&arg_inferred);
                if arg_llvm_ty == "i64" && struct_llvm.starts_with('%') {
                    // Generic erasure: struct → i64 via alloca+store+ptrtoint
                    if self.is_expr_value(arg) {
                        let alloca_tmp = self.next_temp(counter);
                        self.emit_entry_alloca(&alloca_tmp, &struct_llvm);
                        write_ir!(
                            ir,
                            "  store {} {}, {}* {}",
                            struct_llvm,
                            val,
                            struct_llvm,
                            alloca_tmp
                        );
                        let ptr_tmp = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = ptrtoint {}* {} to i64",
                            ptr_tmp,
                            struct_llvm,
                            alloca_tmp
                        );
                        val = ptr_tmp;
                    } else {
                        // val is already a pointer — just ptrtoint
                        let ptr_tmp = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = ptrtoint {}* {} to i64",
                            ptr_tmp,
                            struct_llvm,
                            val
                        );
                        val = ptr_tmp;
                    }
                } else if !self.is_expr_value(arg) {
                    // Non-generic struct param: load from pointer
                    let loaded = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}",
                        loaded,
                        arg_llvm_ty,
                        arg_llvm_ty,
                        val
                    );
                    val = loaded;
                }
            }

            arg_vals.push(format!("{} {}", arg_llvm_ty, val));
        }

        // Infer the actual return type of the method from function info.
        //
        // When the method is not found in `self.types.functions`, this falls back to
        // "i64". This can happen when multiple trait implementations provide the same
        // method name for a type and the codegen-level resolution hasn't registered
        // the correct mangled name. The type checker is responsible for detecting
        // ambiguous trait method dispatch (E039); here we emit a best-effort fallback.
        let (ret_type, ret_resolved) = {
            let fn_info = self.types.functions.get(&full_method_name);
            if let Some(info) = fn_info {
                (
                    self.type_to_llvm(&info.signature.ret),
                    info.signature.ret.clone(),
                )
            } else if let Some(sig) = self.types.resolved_function_sigs.get(&full_method_name) {
                // Fallback: use type checker's resolved signature
                (self.type_to_llvm(&sig.ret), sig.ret.clone())
            } else {
                // Fallback: check if any trait impl provides this method for the receiver type
                if let ResolvedType::Named { name, .. } = &recv_type {
                    let mut candidate_count = 0usize;
                    for ((impl_type, _trait_name), methods) in &self.types.trait_impl_methods {
                        if impl_type == name && methods.contains_key(method_name) {
                            candidate_count += 1;
                        }
                    }
                    if candidate_count > 1 {
                        self.emit_warning(crate::CodegenWarning::UnresolvedTypeFallback {
                            type_desc: format!(
                                "ambiguous trait method dispatch for {}.{}() — {} trait impls",
                                name, method_name, candidate_count
                            ),
                            backend: String::from("text"),
                        });
                    } else {
                        self.emit_warning(crate::CodegenWarning::UnresolvedTypeFallback {
                            type_desc: format!(
                                "method {}.{}() return type not found in function registry",
                                name, method_name
                            ),
                            backend: String::from("text"),
                        });
                    }
                } else {
                    self.emit_warning(crate::CodegenWarning::UnresolvedTypeFallback {
                        type_desc: format!(
                            "method {}() return type unknown (receiver: {:?})",
                            method_name, recv_type
                        ),
                        backend: String::from("text"),
                    });
                }
                ("i64".to_string(), ResolvedType::I64)
            }
        };

        // Vec elem_size patch: before calling a specialized Vec method,
        // set self.elem_size to sizeof(T) and adjust capacity.
        if (full_method_name.starts_with("Vec_push")
            || full_method_name.starts_with("Vec_insert")
            || full_method_name.starts_with("Vec_set"))
            && full_method_name.contains('$')
            && !arg_vals.is_empty()
        {
            if let Some(dollar_pos) = full_method_name.find('$') {
                let type_suffix = &full_method_name[dollar_pos + 1..];
                let elem_size: i64 = match type_suffix {
                    "u8" | "i8" => 1,
                    "u16" | "i16" => 2,
                    "u32" | "i32" | "f32" => 4,
                    "u64" | "i64" | "f64" => 8,
                    "str" => 16,
                    _ => self
                        .types
                        .structs
                        .get(type_suffix)
                        .map(|s| s.fields.iter().map(|(_, ty)| self.compute_sizeof(ty)).sum())
                        .unwrap_or(0),
                };
                if elem_size > 0 && elem_size != 8 {
                    let self_ptr = arg_vals[0]
                        .split_whitespace()
                        .last()
                        .unwrap_or("")
                        .to_string();
                    if !self_ptr.is_empty() {
                        let es_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr %Vec, %Vec* {}, i32 0, i32 3",
                            es_ptr,
                            self_ptr
                        );
                        write_ir!(ir, "  store i64 {}, i64* {}", elem_size, es_ptr);
                        // Adjust capacity to match new elem_size
                        let cap_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr %Vec, %Vec* {}, i32 0, i32 2",
                            cap_ptr,
                            self_ptr
                        );
                        let old_cap = self.next_temp(counter);
                        write_ir!(ir, "  {} = load i64, i64* {}", old_cap, cap_ptr);
                        let bytes = self.next_temp(counter);
                        write_ir!(ir, "  {} = mul i64 {}, 8", bytes, old_cap);
                        let new_cap = self.next_temp(counter);
                        write_ir!(ir, "  {} = sdiv i64 {}, {}", new_cap, bytes, elem_size);
                        write_ir!(ir, "  store i64 {}, i64* {}", new_cap, cap_ptr);
                    }
                }
            }
        }

        if ret_type == "void" {
            write_ir!(
                ir,
                "  call void @{}({})",
                full_method_name,
                arg_vals.join(", ")
            );
            Ok(("void".to_string(), ir))
        } else {
            let tmp = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = call {} @{}({})",
                tmp,
                ret_type,
                full_method_name,
                arg_vals.join(", ")
            );
            // Register the call result's resolved type for downstream type tracking
            self.fn_ctx.register_temp_type(&tmp, ret_resolved);
            Ok((tmp, ir))
        }
    }

    /// Generate static method call expression
    #[inline(never)]
    pub(crate) fn generate_static_method_call_expr(
        &mut self,
        type_name: &Spanned<String>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();

        let base_method_name = format!("{}_{}", type_name.node, method.node);

        // Try to route to a specialized version of the static method.
        // For generic containers like Vec/HashMap, the base name (e.g., "Vec_new")
        // may have specialized versions (e.g., "Vec_new$str") registered via
        // fn_instantiations.
        let full_method_name = if self.types.functions.contains_key(&base_method_name) {
            // Already found directly — use as-is
            base_method_name.clone()
        } else if let Some(inst_list) = self
            .generics
            .fn_instantiations
            .get(&base_method_name)
            .cloned()
        {
            let arg_types: Vec<ResolvedType> =
                args.iter().map(|a| self.infer_expr_type(a)).collect();
            let resolved = self.resolve_generic_call(&base_method_name, &arg_types, &inst_list);
            if self.types.functions.contains_key(&resolved) {
                resolved
            } else {
                base_method_name.clone()
            }
        } else if self.generics.struct_defs.contains_key(&type_name.node) && !args.is_empty() {
            // Infer type args from arguments for generic struct static methods
            let arg_types: Vec<ResolvedType> =
                args.iter().map(|a| self.infer_expr_type(a)).collect();
            let informative_args: Vec<&ResolvedType> = arg_types
                .iter()
                .filter(|t| {
                    !matches!(
                        t,
                        ResolvedType::I64 | ResolvedType::Generic(_) | ResolvedType::Var(_)
                    )
                })
                .collect();
            if !informative_args.is_empty() {
                let struct_def = self.generics.struct_defs.get(&type_name.node).cloned();
                let n_generic_params = struct_def
                    .as_ref()
                    .map(|s| {
                        s.generics
                            .iter()
                            .filter(|g| {
                                !matches!(g.kind, vais_ast::GenericParamKind::Lifetime { .. })
                            })
                            .count()
                    })
                    .unwrap_or(1);
                let inferred_type_args: Vec<ResolvedType> = informative_args
                    .iter()
                    .take(n_generic_params)
                    .map(|t| (*t).clone())
                    .collect();
                let mangled = vais_types::mangle_name(&base_method_name, &inferred_type_args);
                if self.types.functions.contains_key(&mangled) {
                    mangled
                } else {
                    base_method_name.clone()
                }
            } else {
                base_method_name.clone()
            }
        } else {
            base_method_name.clone()
        };

        // Look up function info for parameter types
        let fn_info = self.types.functions.get(&full_method_name).cloned();

        let mut arg_vals = Vec::with_capacity(args.len());
        for (i, arg) in args.iter().enumerate() {
            let (mut val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);
            let inferred_ty = self.infer_expr_type(arg);

            // Use parameter type from function signature if available
            let param_ty = fn_info
                .as_ref()
                .and_then(|f| f.signature.params.get(i))
                .map(|(_, ty, _)| ty.clone())
                // Fallback: check resolved_function_sigs from type checker
                // This handles cross-module static methods not registered in self.types.functions
                .or_else(|| {
                    self.types
                        .resolved_function_sigs
                        .get(&full_method_name)
                        .and_then(|sig| sig.params.get(i))
                        .map(|(_, ty, _)| ty.clone())
                });

            // Determine LLVM type: prefer parameter type over inferred type,
            // unless param is generic (in which case use inferred)
            let arg_llvm_ty = if let Some(ref pt) = param_ty {
                if matches!(pt, ResolvedType::Generic(_)) {
                    self.type_to_llvm(&inferred_ty)
                } else {
                    self.type_to_llvm(pt)
                }
            } else {
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
                        write_ir!(
                            ir,
                            "  {} = trunc {} {} to {}",
                            conv_tmp,
                            src_ty,
                            val,
                            dst_ty
                        );
                    } else {
                        write_ir!(ir, "  {} = sext {} {} to {}", conv_tmp, src_ty, val, dst_ty);
                    }
                    val = conv_tmp;
                }
            }

            // For struct types, load the value from pointer if the expression produces a pointer.
            // Use param type OR inferred type to determine if this is a struct.
            let type_to_check = match &param_ty {
                Some(ty) => ty.clone(),
                None => inferred_ty,
            };
            if matches!(type_to_check, ResolvedType::Named { .. }) && !self.is_expr_value(arg) {
                let loaded = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = load {}, {}* {}",
                    loaded,
                    arg_llvm_ty,
                    arg_llvm_ty,
                    val
                );
                val = loaded;
            }

            arg_vals.push(format!("{} {}", arg_llvm_ty, val));
        }

        let ret_type = fn_info
            .as_ref()
            .map(|info| self.type_to_llvm(&info.signature.ret))
            .or_else(|| {
                // Fallback: check resolved_function_sigs from type checker
                self.types
                    .resolved_function_sigs
                    .get(&full_method_name)
                    .map(|sig| self.type_to_llvm(&sig.ret))
            })
            .unwrap_or_else(|| "i64".to_string());

        if ret_type == "void" {
            write_ir!(
                ir,
                "  call void @{}({})",
                full_method_name,
                arg_vals.join(", ")
            );
            Ok(("void".to_string(), ir))
        } else {
            let tmp = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = call {} @{}({})",
                tmp,
                ret_type,
                full_method_name,
                arg_vals.join(", ")
            );
            Ok((tmp, ir))
        }
    }

    /// Generate str_to_ptr builtin call
    pub(super) fn generate_str_to_ptr_builtin(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        if args.len() != 1 {
            return Err(CodegenError::TypeError(
                "str_to_ptr expects 1 argument".to_string(),
            ));
        }
        let (str_val, str_ir) = self.generate_expr(&args[0], counter)?;
        let mut ir = str_ir;
        // String is now a fat pointer { i8*, i64 } — extract the raw i8* pointer
        let raw_ptr = self.extract_str_ptr(&str_val, counter, &mut ir);
        let result = self.next_temp(counter);
        write_ir!(ir, "  {} = ptrtoint i8* {} to i64", result, raw_ptr);
        Ok((result, ir))
    }

    /// Generate ptr_to_str builtin call — returns { i8*, i64 } fat pointer
    pub(super) fn generate_ptr_to_str_builtin(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        if args.len() != 1 {
            return Err(CodegenError::TypeError(
                "ptr_to_str expects 1 argument".to_string(),
            ));
        }
        let (ptr_val, ptr_ir) = self.generate_expr(&args[0], counter)?;
        let mut ir = ptr_ir;
        let arg_type = self.infer_expr_type(&args[0]);

        // Convert i64 to raw i8* pointer if needed
        let raw_ptr = if matches!(arg_type, vais_types::ResolvedType::I64) {
            let tmp = self.next_temp(counter);
            write_ir!(ir, "  {} = inttoptr i64 {} to i8*", tmp, ptr_val);
            tmp
        } else {
            ptr_val
        };

        // Call strlen to get the length, then build fat pointer { i8*, i64 }
        let len = self.next_temp(counter);
        write_ir!(ir, "  {} = call i64 @strlen(i8* {})", len, raw_ptr);
        let result = self.build_str_fat_ptr(&raw_ptr, &len, counter, &mut ir);
        Ok((result, ir))
    }

    /// Try to generate a specialized Vec/generic method on demand.
    /// Returns the mangled function name if generation succeeded, None otherwise.
    fn try_generate_vec_specialization(
        &mut self,
        struct_name: &str,
        method_name: &str,
        type_args: &[ResolvedType],
        _counter: &mut usize,
    ) -> Option<String> {
        use vais_types::GenericInstantiation;
        use vais_types::InstantiationKind;

        // Only handle known generic containers
        if struct_name != "Vec" && struct_name != "HashMap" && struct_name != "Option" {
            return None;
        }

        let base_name = format!("{}_{}", struct_name, method_name);
        let mangled = vais_types::mangle_name(&base_name, type_args);

        // Already exists? (concurrent generation check)
        if self.types.functions.contains_key(&mangled) {
            return Some(mangled);
        }

        // Build synthetic instantiation
        let inst = GenericInstantiation {
            base_name: base_name.clone(),
            mangled_name: mangled.clone(),
            type_args: type_args.to_vec(),
            const_args: vec![],
            kind: InstantiationKind::Method {
                struct_name: struct_name.to_string(),
            },
        };

        // Find the method template from struct definitions or impl blocks
        let method_fn = self
            .generics
            .struct_defs
            .get(struct_name)
            .and_then(|s| {
                s.methods
                    .iter()
                    .find(|m| m.node.name.node == method_name)
                    .map(|m| std::rc::Rc::new(m.node.clone()))
            })
            .or_else(|| {
                // Try generic_method_bodies (from impl blocks on generic structs)
                self.generics
                    .generic_method_bodies
                    .get(&(struct_name.to_string(), method_name.to_string()))
                    .cloned()
            });

        if let Some(method_fn) = method_fn {
            // Skip if already generated (prevent infinite recursion)
            if self.generics.generated_functions.contains_key(&mangled) {
                return Some(mangled);
            }
            // Generate the specialized function
            match self.generate_specialized_function(&method_fn, &inst) {
                Ok(ir_code) => {
                    // Append to pending specialized IR
                    self.fn_ctx.pending_specialized_ir.push(ir_code);
                    Some(mangled)
                }
                Err(_) => None,
            }
        } else {
            None
        }
    }
}
