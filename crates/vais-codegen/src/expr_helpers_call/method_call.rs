use super::*;
use std::collections::HashMap;
use vais_ast::{Expr, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
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
            let mut recv_type = self.infer_expr_type(receiver);

            // If receiver type has unresolved or missing generics (e.g., Vec<T> or Vec),
            // try to recover concrete generics from the receiver expression context.
            if let ResolvedType::Named { name, generics } = &recv_type {
                let has_concrete_generics = !generics.is_empty()
                    && generics
                        .iter()
                        .all(|g| !matches!(g, ResolvedType::Generic(_) | ResolvedType::Var(_)));
                if !has_concrete_generics
                    && (name == "Vec" || name == "HashMap" || name == "Option")
                {
                    // Strategy A: Field access — look up struct field type definition
                    if let Expr::Field {
                        expr: obj_expr,
                        field,
                    } = &receiver.node
                    {
                        let obj_type = self.infer_expr_type(obj_expr);
                        let inner = match &obj_type {
                            ResolvedType::Ref(i) | ResolvedType::RefMut(i) => i.as_ref(),
                            ResolvedType::Pointer(i) => i.as_ref(),
                            other => other,
                        };
                        if let ResolvedType::Named { name: sname, .. } = inner {
                            let resolved = self.resolve_struct_name(sname);
                            for candidate in &[sname.as_str(), resolved.as_str()] {
                                if let Some(si) = self.types.structs.get(*candidate) {
                                    for (fname, ftype) in &si.fields {
                                        if fname == &field.node {
                                            if let ResolvedType::Named { generics: fg, .. } = ftype
                                            {
                                                if !fg.is_empty() {
                                                    recv_type = ftype.clone();
                                                }
                                            }
                                            break;
                                        }
                                    }
                                }
                                if !matches!(&recv_type, ResolvedType::Named { generics, .. } if generics.is_empty())
                                {
                                    break;
                                }
                            }
                        }
                    }
                    // Strategy B: TC expr_types — try TC-resolved type
                    if matches!(&recv_type, ResolvedType::Named { generics, .. } if generics.is_empty())
                    {
                        if let Some(tc_ty) = self.tc_expr_type(receiver) {
                            if let ResolvedType::Named { generics: tg, .. } = tc_ty {
                                if !tg.is_empty() {
                                    recv_type = tc_ty.clone();
                                }
                            }
                        }
                    }
                }
            }

            (recv_val, recv_ir, recv_type)
        };
        let mut ir = recv_ir;

        let method_name = &method.node;

        // String method calls: str.len(), &str.len(), str.clone(), etc.
        let recv_is_str = matches!(recv_type, ResolvedType::Str)
            || matches!(&recv_type, ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
                if matches!(inner.as_ref(), ResolvedType::Str));
        if recv_is_str {
            return self.generate_string_method_call(&recv_val, &ir, method_name, args, counter);
        }

        if self.is_string_builder_type(&recv_type) && method_name == "as_str" && args.is_empty() {
            let source_is_value = self.is_expr_value(receiver);
            let value =
                self.coerce_string_builder_to_str(&recv_val, source_is_value, counter, &mut ir);
            return Ok((value, ir));
        }

        // clone() on any type — return the receiver value unchanged.
        // For Named types (structs), generate_ident_expr returns a pointer
        // (%Type*) for SSA/alloca locals. We must load the struct value so
        // the caller gets a by-value %Type, not a %Type* pointer.
        if method_name == "clone" && args.is_empty() {
            let inner_recv = match &recv_type {
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref(),
                other => other,
            };
            if matches!(inner_recv, ResolvedType::Named { .. }) {
                let is_ptr_receiver = !self.is_expr_value(receiver)
                    || if let Expr::Ident(name) = &receiver.node {
                        self.fn_ctx.locals.get(name.as_str()).is_some_and(|local| {
                            matches!(local.ty, ResolvedType::Named { .. })
                                && (local.is_ssa() || local.is_alloca())
                        })
                    } else {
                        // Field access on structs also returns pointers (GEP results)
                        matches!(&receiver.node, Expr::Field { .. })
                    };
                if is_ptr_receiver {
                    let llvm_ty = self.type_to_llvm(inner_recv);
                    let loaded = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}",
                        loaded,
                        llvm_ty,
                        llvm_ty,
                        recv_val
                    );
                    return Ok((loaded, ir));
                }
            }
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

        let slice_elem_ty = match &recv_type {
            ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => Some(elem.as_ref().clone()),
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => match inner.as_ref() {
                ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => {
                    Some(elem.as_ref().clone())
                }
                _ => None,
            },
            _ => None,
        };
        if method_name == "get" && args.len() == 1 {
            if let Some(elem_ty) = slice_elem_ty {
                let (idx_val, idx_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&idx_ir);
                let idx_resolved = self.infer_expr_type(&args[0]);
                let idx_llvm = self.type_to_llvm(&idx_resolved);
                let idx_i64 = if idx_llvm != "i64" && idx_llvm.starts_with('i') {
                    let tmp = self.next_temp(counter);
                    write_ir!(ir, "  {} = sext {} {} to i64", tmp, idx_llvm, idx_val);
                    tmp
                } else {
                    idx_val
                };

                self.needs_bounds_check = true;
                let len = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {{ i8*, i64 }} {}, 1",
                    len,
                    recv_val
                );
                let in_bounds = self.next_temp(counter);
                write_ir!(ir, "  {} = icmp ult i64 {}, {}", in_bounds, idx_i64, len);
                let safe_label = self.next_label("slice_get_safe");
                let oob_label = self.next_label("slice_get_oob");
                write_ir!(
                    ir,
                    "  br i1 {}, label %{}, label %{}",
                    in_bounds,
                    safe_label,
                    oob_label
                );
                write_ir!(ir, "{}:", oob_label);
                ir.push_str("  call void @abort()\n");
                ir.push_str("  unreachable\n");
                write_ir!(ir, "{}:", safe_label);
                self.fn_ctx.current_block.clone_from(&safe_label);

                let raw_data = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                    raw_data,
                    recv_val
                );
                let elem_llvm = self.type_to_llvm(&elem_ty);
                let data_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = bitcast i8* {} to {}*",
                    data_ptr,
                    raw_data,
                    elem_llvm
                );
                let elem_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr {}, {}* {}, i64 {}",
                    elem_ptr,
                    elem_llvm,
                    elem_llvm,
                    data_ptr,
                    idx_i64
                );
                let loaded = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = load {}, {}* {}",
                    loaded,
                    elem_llvm,
                    elem_llvm,
                    elem_ptr
                );
                self.fn_ctx.register_temp_type(&loaded, elem_ty);
                return Ok((loaded, ir));
            }
        }

        // Use resolve_struct_name to match definition naming (e.g., Pair → Pair$i64)
        // For generic structs with type args, try mangled name first (e.g., Vec_push$GraphNode)
        // Unwrap Ref/RefMut/Pointer to get the inner Named type.
        let inner_recv_type_owned = match &recv_type {
            ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Pointer(inner) => inner.as_ref().clone(),
            other => other.clone(),
        };
        let guard_forward_inner = match &inner_recv_type_owned {
            ResolvedType::Named { name, generics }
                if matches!(
                    name.as_str(),
                    "MutexGuard" | "RwLockReadGuard" | "RwLockWriteGuard"
                ) && !generics.is_empty() =>
            {
                let guard_method_arity = match method_name.as_str() {
                    "new" => Some(1),
                    "get" | "unlock" => Some(0),
                    "set" => Some(1),
                    _ => None,
                };
                if guard_method_arity == Some(args.len()) {
                    None
                } else {
                    Some(generics[0].clone())
                }
            }
            _ => None,
        };
        let mut effective_recv_type_owned = guard_forward_inner
            .clone()
            .unwrap_or_else(|| inner_recv_type_owned.clone());

        if let ResolvedType::Named { name, generics } = &effective_recv_type_owned {
            if generics.is_empty() {
                if let Some(ResolvedType::Named {
                    name: self_name,
                    generics: self_generics,
                }) = self.generics.substitutions.get("Self").cloned()
                {
                    if self_name == *name
                        && !self_generics.is_empty()
                        && self_generics
                            .iter()
                            .all(|g| !matches!(g, ResolvedType::Generic(_) | ResolvedType::Var(_)))
                    {
                        effective_recv_type_owned = ResolvedType::Named {
                            name: name.clone(),
                            generics: self_generics,
                        };
                    }
                }
            }
        }

        if let ResolvedType::Named { name, generics } = &effective_recv_type_owned {
            if generics.is_empty() {
                if let Some(struct_def) = self.generics.struct_defs.get(name).cloned() {
                    let mut recovered = Vec::new();
                    for param in struct_def
                        .generics
                        .iter()
                        .filter(|g| !matches!(g.kind, vais_ast::GenericParamKind::Lifetime { .. }))
                    {
                        if let Some(concrete) =
                            self.generics.substitutions.get(&param.name.node).cloned()
                        {
                            if matches!(concrete, ResolvedType::Generic(_) | ResolvedType::Var(_)) {
                                recovered.clear();
                                break;
                            }
                            recovered.push(concrete);
                        } else {
                            recovered.clear();
                            break;
                        }
                    }
                    if !recovered.is_empty() {
                        effective_recv_type_owned = ResolvedType::Named {
                            name: name.clone(),
                            generics: recovered,
                        };
                    }
                }
            }
        }

        let effective_recv_type = &effective_recv_type_owned;

        let full_method_name = if let ResolvedType::Named { name, generics } = effective_recv_type {
            let resolved = self.resolve_struct_name(name);
            let base = format!("{}_{}", resolved, method_name);

            if !generics.is_empty() {
                // Check if generics are all concrete (not Generic("T") or Var)
                let all_concrete = generics
                    .iter()
                    .all(|g| !matches!(g, ResolvedType::Generic(_) | ResolvedType::Var(_)));

                if all_concrete {
                    // Concrete generics: set substitutions and try mangled name.
                    // Use insert() (not or_insert_with) so that concrete receiver-type
                    // substitutions always override any stale i64-fallback substitution
                    // that may have been set in an enclosing generic function context.
                    if let Some(struct_def) = self.generics.struct_defs.get(name).cloned() {
                        for (param, concrete) in struct_def.generics.iter().zip(generics.iter()) {
                            if !matches!(param.kind, vais_ast::GenericParamKind::Lifetime { .. }) {
                                self.generics
                                    .substitutions
                                    .insert(param.name.node.clone(), concrete.clone());
                            }
                        }
                    }
                    let mangled = vais_types::mangle_name(&base, generics);
                    if self.types.functions.contains_key(&mangled)
                        || self.generics.generated_functions.contains_key(&mangled)
                    {
                        mangled
                    } else {
                        // Not yet registered. Try on-demand specialization — the
                        // receiver type has concrete generics but the method body
                        // that referenced this call (e.g., Vec_push$T body calling
                        // @.grow()) was specialized without also scheduling the
                        // inner generic method. See ROADMAP Phase 191 #10.
                        self.try_generate_vec_specialization(
                            &resolved,
                            method_name,
                            generics,
                            counter,
                        )
                        .unwrap_or(base)
                    }
                } else {
                    // Unresolved generics (e.g., Generic("T")): use fn_instantiations
                    // to find the correct specialization based on argument types
                    if let Some(inst_list) = self.generics.fn_instantiations.get(&base).cloned() {
                        let arg_types: Vec<ResolvedType> =
                            args.iter().map(|a| self.infer_expr_type(a)).collect();
                        let resolved = self.resolve_generic_call(&base, &arg_types, &inst_list);
                        if self.types.functions.contains_key(&resolved) {
                            // Set substitutions for the resolved type
                            if let Some(dollar_pos) = resolved.find('$') {
                                let type_suffix = &resolved[dollar_pos + 1..];
                                let resolved_type =
                                    self.resolve_type_suffix_to_resolved(type_suffix);
                                if let Some(struct_def) =
                                    self.generics.struct_defs.get(name).cloned()
                                {
                                    for param in struct_def.generics.iter() {
                                        if !matches!(
                                            param.kind,
                                            vais_ast::GenericParamKind::Lifetime { .. }
                                        ) {
                                            self.generics.substitutions.insert(
                                                param.name.node.clone(),
                                                resolved_type.clone(),
                                            );
                                        }
                                    }
                                }
                            }
                            resolved
                        } else {
                            base
                        }
                    } else {
                        // No instantiations registered — fall back to argument inference
                        // (same as Strategy 2 in the empty generics path)
                        self.resolve_method_generic_name(name, &base, args, counter)
                            .unwrap_or(base)
                    }
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
                    self.resolve_method_generic_name_with_specialization(
                        name,
                        method_name,
                        &base,
                        args,
                        counter,
                    )
                    .unwrap_or(base)
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

        // Look up function info for parameter types
        let fn_info = self.types.functions.get(&full_method_name).cloned();

        // Determine receiver LLVM type.
        // For specialized methods, use the function's actual self parameter type
        // to match the calling convention (e.g., %Vec$u8* instead of %Vec*).
        let recv_llvm_ty = if let Some(ref fi) = fn_info {
            if let Some((_, self_ty, _)) = fi.signature.params.first() {
                let self_llvm = self.type_to_llvm(self_ty);
                // If the function expects by-pointer (e.g., %Vec$u8*), use that
                if self_llvm.ends_with('*') {
                    self_llvm
                } else {
                    format!("{}*", self_llvm)
                }
            } else if matches!(effective_recv_type, ResolvedType::Named { .. }) {
                format!("{}*", self.type_to_llvm(effective_recv_type))
            } else {
                self.type_to_llvm(effective_recv_type)
            }
        } else if matches!(effective_recv_type, ResolvedType::Named { .. }) {
            format!("{}*", self.type_to_llvm(effective_recv_type))
        } else {
            self.type_to_llvm(effective_recv_type)
        };

        // If receiver is a struct value (from function call) but method expects pointer,
        // store the value to an alloca and pass the pointer instead.
        // Skip for SelfCall — `%self` is already a pointer in method context.
        let receiver_is_pointer_value = matches!(
            recv_type,
            ResolvedType::Ref(_)
                | ResolvedType::RefMut(_)
                | ResolvedType::RefLifetime { .. }
                | ResolvedType::RefMutLifetime { .. }
                | ResolvedType::Pointer(_)
        );
        let mut recv_val = if guard_forward_inner.is_none()
            && recv_llvm_ty.ends_with('*')
            && !matches!(&receiver.node, Expr::SelfCall)
            && !receiver_is_pointer_value
            && self.is_expr_value(receiver)
            && matches!(effective_recv_type, ResolvedType::Named { .. })
        {
            let struct_llvm = self.type_to_llvm(effective_recv_type);
            let alloca_tmp = self.next_temp(counter);
            self.emit_entry_alloca(&alloca_tmp, &struct_llvm);
            write_ir!(
                ir,
                "  store {} {}, {}* {}",
                struct_llvm,
                recv_val,
                struct_llvm,
                alloca_tmp
            );
            alloca_tmp
        } else {
            recv_val
        };

        if let Some(inner_ty) = &guard_forward_inner {
            let guard_llvm = self.type_to_llvm(&inner_recv_type_owned);
            let mutex_word_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                mutex_word_ptr,
                guard_llvm,
                guard_llvm,
                recv_val
            );
            let mutex_word = self.next_temp(counter);
            write_ir!(ir, "  {} = load i64, i64* {}", mutex_word, mutex_word_ptr);

            let mutex_ty = ResolvedType::Named {
                name: "Mutex".to_string(),
                generics: vec![inner_ty.clone()],
            };
            let mutex_llvm = self.type_to_llvm(&mutex_ty);
            let mutex_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = inttoptr i64 {} to {}*",
                mutex_ptr,
                mutex_word,
                mutex_llvm
            );
            let inner_ptr = self.next_temp(counter);
            let inner_llvm = self.type_to_llvm(inner_ty);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                inner_ptr,
                mutex_llvm,
                mutex_llvm,
                mutex_ptr
            );
            recv_val = if recv_llvm_ty == format!("{}*", inner_llvm) {
                inner_ptr
            } else {
                let casted = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = bitcast {}* {} to {}",
                    casted,
                    inner_llvm,
                    inner_ptr,
                    recv_llvm_ty
                );
                casted
            };
        }
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
                if let ResolvedType::Generic(ref name) = pt {
                    // Check if a concrete substitution exists for this generic parameter
                    if let Some(concrete_ty) = self.get_generic_substitution(name) {
                        self.type_to_llvm(&concrete_ty)
                    } else {
                        // Generic params are erased to i64 in LLVM IR
                        // Use i64 as the LLVM type, and coerce if needed
                        "i64".to_string()
                    }
                } else {
                    self.type_to_llvm(pt)
                }
            } else {
                self.type_to_llvm(&inferred_ty)
            };

            if let Some(ref pt) = param_ty {
                if Self::is_vec_to_slice_coercion(pt, &inferred_ty) {
                    let expr_is_value = self.is_expr_value(arg);
                    if let Some(slice_val) = self.coerce_vec_to_slice_fat_ptr(
                        &val,
                        &inferred_ty,
                        expr_is_value,
                        counter,
                        &mut ir,
                    ) {
                        val = slice_val;
                    }
                }
                let param_is_str_fat = matches!(pt, ResolvedType::Str)
                    || matches!(pt, ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
                        if matches!(inner.as_ref(), ResolvedType::Str));
                if param_is_str_fat {
                    if let Some(str_val) =
                        self.coerce_ref_str_to_str_fat_ptr(&val, &inferred_ty, counter, &mut ir)
                    {
                        val = str_val;
                    }
                }
            }

            // Integer width coercion: coerce to match arg_llvm_ty
            {
                let actual_arg_llvm_ty = self.llvm_type_of(&val);
                let actual_arg_bits = Self::int_type_width(&actual_arg_llvm_ty);
                let src_bits = if actual_arg_bits > 0 {
                    actual_arg_bits
                } else {
                    self.get_integer_bits(&inferred_ty)
                };
                // Parse dst_bits from arg_llvm_ty (e.g., "i64" -> 64)
                let dst_bits = if let Some(rest) = arg_llvm_ty.strip_prefix('i') {
                    rest.parse::<u32>().unwrap_or(0)
                } else if let Some(ref pt) = param_ty {
                    self.get_integer_bits(pt)
                } else {
                    0
                };
                if src_bits > 0 && dst_bits > 0 && src_bits != dst_bits {
                    let conv_tmp = self.next_temp(counter);
                    let src_ty = if actual_arg_bits > 0 {
                        actual_arg_llvm_ty
                    } else {
                        format!("i{}", src_bits)
                    };
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
                    self.fn_ctx.register_temp_type(
                        &conv_tmp,
                        param_ty.clone().unwrap_or_else(|| inferred_ty.clone()),
                    );
                    val = conv_tmp;
                }
            }

            if let Some(ref pt) = param_ty {
                if matches!(pt, ResolvedType::Fn { .. } | ResolvedType::FnPtr { .. }) {
                    let val_ty = self.llvm_type_of(&val);
                    if crate::helpers::is_llvm_integer_scalar(&val_ty) && arg_llvm_ty.ends_with('*')
                    {
                        let fn_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = inttoptr {} {} to {}",
                            fn_ptr,
                            val_ty,
                            val,
                            arg_llvm_ty
                        );
                        self.fn_ctx.register_temp_type(&fn_ptr, pt.clone());
                        val = fn_ptr;
                    }
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

            // For struct types: handle generic erasure (struct → i64) and
            // pointer-to-value loading for non-generic struct params.
            let arg_inferred = self.infer_expr_type(arg);
            let mut erased_named_to_i64 = false;
            if matches!(&arg_inferred, ResolvedType::Named { .. }) {
                let struct_llvm = self.type_to_llvm(&arg_inferred);
                if arg_llvm_ty == "i64" && struct_llvm.starts_with('%') {
                    let expr_is_value = self.is_expr_value(arg);
                    let val_llvm_ty = self.llvm_type_of(&val);
                    if expr_is_value && val_llvm_ty == "i64" {
                        erased_named_to_i64 = true;
                    } else if !expr_is_value || val_llvm_ty == format!("{}*", struct_llvm) {
                        let ptr_tmp = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = ptrtoint {}* {} to i64",
                            ptr_tmp,
                            struct_llvm,
                            val
                        );
                        val = ptr_tmp;
                        erased_named_to_i64 = true;
                    } else {
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
                        erased_named_to_i64 = true;
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

            // Coerce struct pointer → i64 when arg_ty is i64 but value is a Named type
            if arg_llvm_ty == "i64" && !erased_named_to_i64 {
                let inferred = self.infer_expr_type(arg);
                if matches!(inferred, ResolvedType::Named { .. }) {
                    let struct_llvm = self.type_to_llvm(&inferred);
                    if self.is_expr_value(arg) && self.llvm_type_of(&val) != "i64" {
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
                        let tmp = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = ptrtoint {}* {} to i64",
                            tmp,
                            struct_llvm,
                            alloca_tmp
                        );
                        val = tmp;
                    } else if self.llvm_type_of(&val) != "i64" {
                        let tmp = self.next_temp(counter);
                        write_ir!(ir, "  {} = ptrtoint {}* {} to i64", tmp, struct_llvm, val);
                        val = tmp;
                    }
                }
            }

            // Coerce i64 → struct when specialized param expects struct but value is i64
            // This happens in generic function bodies (e.g., Vec_map) that call
            // specialized methods (e.g., Vec_push$BTreeLeafEntry)
            if arg_llvm_ty.starts_with('%') && !arg_llvm_ty.ends_with('*') {
                let inferred = self.infer_expr_type(arg);
                if !matches!(inferred, ResolvedType::Named { .. }) {
                    // Value is usually i64 generic erasure, but unit enum variants
                    // produce a typed alloca pointer even when local inference falls
                    // back to i64. Preserve that pointer path instead of emitting
                    // invalid `inttoptr i64 %ptr`.
                    let val_llvm_ty = self.llvm_type_of(&val);
                    let ptr_tmp = if val_llvm_ty == format!("{}*", arg_llvm_ty)
                        || val_llvm_ty == "ptr"
                        || val_llvm_ty.ends_with('*')
                    {
                        val.clone()
                    } else {
                        let ptr_tmp = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = inttoptr i64 {} to {}*",
                            ptr_tmp,
                            val,
                            arg_llvm_ty
                        );
                        ptr_tmp
                    };
                    let loaded = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}",
                        loaded,
                        arg_llvm_ty,
                        arg_llvm_ty,
                        ptr_tmp
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
        // Applied to ALL Vec methods that access elements (push/insert/set/get/pop/grow/etc.)
        //
        // Gate: only the stdlib std::Vec<T> layout has the 4th field `elem_size`.
        // User-defined `Vec<T>` with different layout (e.g. only `elem`+`len`) must
        // not trigger this patch — it would emit OOB GEPs into a 2-field struct.
        let stdlib_vec_layout = self
            .types
            .structs
            .get("Vec")
            .map(|si| {
                si.fields.len() >= 4
                    && si
                        .fields
                        .get(3)
                        .map(|(n, _)| n == "elem_size")
                        .unwrap_or(false)
            })
            .unwrap_or(false);
        if stdlib_vec_layout
            && full_method_name.starts_with("Vec_")
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
                        // Read old elem_size; only adjust capacity if still default (8)
                        let old_es = self.next_temp(counter);
                        write_ir!(ir, "  {} = load i64, i64* {}", old_es, es_ptr);
                        write_ir!(ir, "  store i64 {}, i64* {}", elem_size, es_ptr);
                        let needs_adjust = self.next_temp(counter);
                        write_ir!(ir, "  {} = icmp eq i64 {}, 8", needs_adjust, old_es);
                        let lbl_adjust = format!("vec_es_adjust.{}", counter);
                        let lbl_done = format!("vec_es_done.{}", counter);
                        *counter += 1;
                        write_ir!(
                            ir,
                            "  br i1 {}, label %{}, label %{}",
                            needs_adjust,
                            lbl_adjust,
                            lbl_done
                        );
                        write_ir!(ir, "{}:", lbl_adjust);
                        // Adjust capacity: old was in 8-byte slots, convert to elem_size slots
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
                        write_ir!(ir, "  br label %{}", lbl_done);
                        write_ir!(ir, "{}:", lbl_done);
                        self.fn_ctx.current_block.clone_from(&lbl_done);
                    }
                }
            }
        }

        // Vec<str>.drop shallow-free prelude (RFC-002 §4.4; Phase 191 #2a').
        // Before the user-level Vec_drop$str runs (which frees `self.data`), free
        // any heap-owned element string buffers tracked in the owned bitmap so
        // `drop()` doesn't leak them. drop() itself only frees the data block.
        if full_method_name == "Vec_drop$str" && !arg_vals.is_empty() {
            let self_ptr = arg_vals[0]
                .split_whitespace()
                .last()
                .unwrap_or("")
                .to_string();
            if !self_ptr.is_empty() {
                write_ir!(
                    ir,
                    "  call void @__vais_vec_str_shallow_free(%Vec* {})",
                    self_ptr
                );
                self.needs_vec_str_helpers = true;
            }
        }

        // Vec<str>.push ownership wrapping (RFC-002 §4.1/§4.4; Phase 191 #2a').
        // When a heap-owned str (tracked via string_value_slot) is pushed into a
        // Vec<str>, transfer ownership to the container's `owned` bitmap. Literal
        // / borrowed strs are left as no-ops — they don't need freeing on drop.
        if full_method_name == "Vec_push$str" && arg_vals.len() >= 2 {
            let self_ptr = arg_vals[0]
                .split_whitespace()
                .last()
                .unwrap_or("")
                .to_string();
            // arg_vals[1] is "{ i8*, i64 } %tN" — extract %tN.
            let rvalue_token = arg_vals[1]
                .split_whitespace()
                .last()
                .unwrap_or("")
                .to_string();
            if !self_ptr.is_empty() && !rvalue_token.is_empty() {
                let slot_opt = self.fn_ctx.string_value_slot.get(&rvalue_token).cloned();
                if let Some(slot) = slot_opt {
                    // 1) Ensure bitmap capacity covers current self.len.
                    let len_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr %Vec, %Vec* {}, i32 0, i32 1",
                        len_ptr,
                        self_ptr
                    );
                    let cur_len = self.next_temp(counter);
                    write_ir!(ir, "  {} = load i64, i64* {}", cur_len, len_ptr);
                    let idx = cur_len.clone();
                    let need_plus_one = self.next_temp(counter);
                    write_ir!(ir, "  {} = add i64 {}, 1", need_plus_one, idx);
                    write_ir!(
                        ir,
                        "  call void @__vais_vec_str_owned_ensure(%Vec* {}, i64 {})",
                        self_ptr,
                        need_plus_one
                    );
                    // 2) Mark owned bit.
                    write_ir!(
                        ir,
                        "  call void @__vais_vec_str_owned_set(%Vec* {}, i64 {})",
                        self_ptr,
                        idx
                    );
                    // 3) Transfer: null out slot pointer + remove string_value_slot
                    //    so scope-exit cleanup skips it. alloc_tracker entry
                    //    stays (Phase 191 #5 pattern) to avoid slot-id reuse.
                    write_ir!(ir, "  store i8* null, i8** {}", slot);
                    self.fn_ctx.string_value_slot.remove(&rvalue_token);
                    // Also remove from the current scope_str_stack frame so
                    // the enclosing block's string-scope cleanup doesn't
                    // free a now-null slot it doesn't own anymore.
                    if let Some(frame) = self.fn_ctx.scope_str_stack.last_mut() {
                        frame.retain(|s| s != &slot);
                    }
                    self.needs_vec_str_helpers = true;
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

    /// Resolve a generic method name by inferring type arguments from call arguments.
    ///
    /// Collects argument types, filters out non-informative types (I64/Generic/Var),
    /// looks up the struct's generic parameter count, infers type args, sets substitutions,
    /// and attempts to mangle the base name. Returns `Some(mangled)` if a specialized
    /// function exists, or `None` if no specialization was found.
    fn resolve_method_generic_name(
        &mut self,
        struct_name: &str,
        base: &str,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> Option<String> {
        let _ = counter; // unused here, kept for API symmetry with the specialization variant
        let arg_types: Vec<ResolvedType> = args.iter().map(|a| self.infer_expr_type(a)).collect();
        let informative_args: Vec<&ResolvedType> = arg_types
            .iter()
            .filter(|t| {
                !matches!(
                    t,
                    ResolvedType::I64 | ResolvedType::Generic(_) | ResolvedType::Var(_)
                )
            })
            .collect();
        if informative_args.is_empty() {
            return None;
        }
        let struct_def = self.generics.struct_defs.get(struct_name).cloned();
        let n_generic_params = struct_def
            .as_ref()
            .map(|s| {
                s.generics
                    .iter()
                    .filter(|g| !matches!(g.kind, vais_ast::GenericParamKind::Lifetime { .. }))
                    .count()
            })
            .unwrap_or(1);
        let inferred_type_args: Vec<ResolvedType> = informative_args
            .iter()
            .take(n_generic_params)
            .map(|t| (*t).clone())
            .collect();
        if inferred_type_args.is_empty() {
            return None;
        }
        // Set generic substitutions so downstream code resolves T correctly
        if let Some(ref sd) = struct_def {
            for (param, concrete) in sd.generics.iter().zip(inferred_type_args.iter()) {
                if !matches!(param.kind, vais_ast::GenericParamKind::Lifetime { .. }) {
                    self.generics
                        .substitutions
                        .entry(param.name.node.clone())
                        .or_insert_with(|| concrete.clone());
                }
            }
        }
        let mangled = vais_types::mangle_name(base, &inferred_type_args);
        if self.types.functions.contains_key(&mangled) {
            Some(mangled)
        } else {
            None
        }
    }

    /// Like [`resolve_method_generic_name`] but also attempts on-demand specialization
    /// via `try_generate_vec_specialization` when the mangled name is not yet registered.
    fn resolve_method_generic_name_with_specialization(
        &mut self,
        struct_name: &str,
        method_name: &str,
        base: &str,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> Option<String> {
        let arg_types: Vec<ResolvedType> = args.iter().map(|a| self.infer_expr_type(a)).collect();
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
        if informative_args.is_empty() {
            return None;
        }
        // Build candidate type args from informative argument types.
        // For single-generic containers (Vec<T>), use the first
        // informative arg. For multi-generic (HashMap<K,V>), use
        // up to the number of struct generic params.
        let struct_def = self.generics.struct_defs.get(struct_name).cloned();
        let n_generic_params = struct_def
            .as_ref()
            .map(|s| {
                s.generics
                    .iter()
                    .filter(|g| !matches!(g.kind, vais_ast::GenericParamKind::Lifetime { .. }))
                    .count()
            })
            .unwrap_or(1);
        let inferred_type_args: Vec<ResolvedType> = informative_args
            .iter()
            .take(n_generic_params)
            .map(|t| (*t).clone())
            .collect();
        if inferred_type_args.is_empty() {
            return None;
        }
        // Set generic substitutions so downstream code resolves T correctly
        if let Some(ref sd) = struct_def {
            for (param, concrete) in sd.generics.iter().zip(inferred_type_args.iter()) {
                if !matches!(param.kind, vais_ast::GenericParamKind::Lifetime { .. }) {
                    self.generics
                        .substitutions
                        .entry(param.name.node.clone())
                        .or_insert_with(|| concrete.clone());
                }
            }
        }
        let mangled = vais_types::mangle_name(base, &inferred_type_args);
        if self.types.functions.contains_key(&mangled) {
            Some(mangled)
        } else {
            // Try on-demand specialization: generate the specialized
            // function if we have the method template and type args
            self.try_generate_vec_specialization(
                struct_name,
                method_name,
                &inferred_type_args,
                counter,
            )
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
        // Check if this is actually an enum variant constructor (EnumType.Variant(...))
        // e.g., Shape.Rect(10, 20) or Option.Some(42). Must be handled before the
        // static-method dispatch because there is no `EnumType_Variant` function to call.
        if let Some(enum_info) = self.types.enums.get(&type_name.node).cloned() {
            for (tag, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == method.node {
                    return self.generate_enum_variant_constructor(
                        &type_name.node,
                        tag as i32,
                        args,
                        counter,
                    );
                }
            }
        }

        let mut ir = String::new();

        let base_method_name = format!("{}_{}", type_name.node, method.node);

        // Try to route to a specialized version of the static method.
        // For generic containers like Vec/HashMap, the base name (e.g., "Vec_new")
        // may have specialized versions (e.g., "Vec_new$str") registered via
        // fn_instantiations.
        // Prefer specialized versions over the base generic method so that the
        // call signature matches the type inference (which also resolves to the
        // specialized return type).
        let expected_specialization = self
            .static_type_args_from_current_substitutions(&type_name.node)
            .and_then(|type_args| {
                let mangled = vais_types::mangle_name(&base_method_name, &type_args);
                if self.types.functions.contains_key(&mangled)
                    || self.generics.generated_functions.contains_key(&mangled)
                {
                    Some(mangled)
                } else {
                    self.try_generate_vec_specialization(
                        &type_name.node,
                        &method.node,
                        &type_args,
                        counter,
                    )
                }
            });

        let full_method_name = if let Some(specialized) = expected_specialization {
            specialized
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
        } else if self.types.functions.contains_key(&base_method_name) {
            // Already found directly — use as-is
            base_method_name.clone()
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
                } else if let Some(spec) = self.try_generate_vec_specialization(
                    &type_name.node,
                    &method.node,
                    &inferred_type_args,
                    counter,
                ) {
                    spec
                } else {
                    base_method_name.clone()
                }
            } else if let Some(type_args) =
                self.infer_static_ctor_type_args_from_peers(&type_name.node, &method.node)
            {
                // Phase 193 R-1: static ctor (Vec.new / Vec.with_capacity) has
                // non-informative args (e.g., just i64 capacity). Recover T by
                // scanning already-registered method instantiations for the same
                // struct and reuse the first concrete type_args seen.
                if let Some(spec) = self.try_generate_vec_specialization(
                    &type_name.node,
                    &method.node,
                    &type_args,
                    counter,
                ) {
                    spec
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

            let saved_generic_substitutions = param_ty.as_ref().and_then(|expected| {
                self.push_expected_static_ctor_substitutions(expected, &arg.node)
            });
            let generated_arg = self.generate_expr(arg, counter);
            if let Some(saved) = saved_generic_substitutions {
                self.generics.substitutions = saved;
            }
            let (mut val, arg_ir) = generated_arg?;
            ir.push_str(&arg_ir);
            let inferred_ty = self.infer_expr_type(arg);

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

            if let Some(ref pt) = param_ty {
                if Self::is_vec_to_slice_coercion(pt, &inferred_ty) {
                    let expr_is_value = self.is_expr_value(arg);
                    if let Some(slice_val) = self.coerce_vec_to_slice_fat_ptr(
                        &val,
                        &inferred_ty,
                        expr_is_value,
                        counter,
                        &mut ir,
                    ) {
                        val = slice_val;
                    }
                }
                let param_is_str_fat = matches!(pt, ResolvedType::Str)
                    || matches!(pt, ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
                        if matches!(inner.as_ref(), ResolvedType::Str));
                if param_is_str_fat {
                    if let Some(str_val) =
                        self.coerce_ref_str_to_str_fat_ptr(&val, &inferred_ty, counter, &mut ir)
                    {
                        val = str_val;
                    }
                }
            }

            // Integer width coercion: prefer the actual generated SSA type when
            // known. Inference can conservatively say i64 for byte indexing
            // expressions even though codegen already emitted an i8 load.
            if let Some(ref pt) = param_ty {
                let actual_arg_llvm_ty = self.llvm_type_of(&val);
                let actual_arg_bits = Self::int_type_width(&actual_arg_llvm_ty);
                let src_bits = if actual_arg_bits > 0 {
                    actual_arg_bits
                } else {
                    self.get_integer_bits(&inferred_ty)
                };
                let dst_bits = if let Some(rest) = arg_llvm_ty.strip_prefix('i') {
                    rest.parse::<u32>().unwrap_or(0)
                } else {
                    self.get_integer_bits(pt)
                };
                if src_bits > 0 && dst_bits > 0 && src_bits != dst_bits {
                    let conv_tmp = self.next_temp(counter);
                    let src_ty = if actual_arg_bits > 0 {
                        actual_arg_llvm_ty
                    } else {
                        format!("i{}", src_bits)
                    };
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
                    self.fn_ctx.register_temp_type(&conv_tmp, pt.clone());
                    val = conv_tmp;
                }
            }

            if let Some(ref pt) = param_ty {
                if matches!(pt, ResolvedType::Fn { .. } | ResolvedType::FnPtr { .. }) {
                    let val_ty = self.llvm_type_of(&val);
                    if crate::helpers::is_llvm_integer_scalar(&val_ty) && arg_llvm_ty.ends_with('*')
                    {
                        let fn_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = inttoptr {} {} to {}",
                            fn_ptr,
                            val_ty,
                            val,
                            arg_llvm_ty
                        );
                        self.fn_ctx.register_temp_type(&fn_ptr, pt.clone());
                        val = fn_ptr;
                    }
                }
            }

            // For struct types, load the value from pointer if the expression produces a pointer.
            // Use param type OR inferred type to determine if this is a struct.
            let type_to_check = match &param_ty {
                Some(ty) => ty.clone(),
                None => inferred_ty.clone(),
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

        let ret_resolved = fn_info
            .as_ref()
            .map(|info| info.signature.ret.clone())
            .or_else(|| {
                // Fallback: check resolved_function_sigs from type checker
                self.types
                    .resolved_function_sigs
                    .get(&full_method_name)
                    .map(|sig| sig.ret.clone())
            });
        let ret_type = ret_resolved
            .as_ref()
            .map(|ret| self.type_to_llvm(ret))
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
            if let Some(ret) = ret_resolved {
                self.fn_ctx.register_temp_type(&tmp, ret);
            }
            Ok((tmp, ir))
        }
    }

    /// Phase 193 R-1: For static constructors like `Vec.new()` / `Vec.with_capacity(8)`
    /// whose args carry no informative type (e.g., only `i64 capacity`), recover the
    /// element type `T` by scanning method instantiations already registered for the
    /// same generic struct. Returns the first set of concrete type_args found.
    ///
    /// Rationale: TC's built-in Vec/HashMap path returns a fresh type var without
    /// registering an instantiation for the ctor itself; other method calls on the
    /// resulting value (e.g., `v.push(42)`) do register e.g. `Vec_push$i64`, so we
    /// can piggyback on that to monomorphize the ctor.
    fn infer_static_ctor_type_args_from_peers(
        &self,
        struct_name: &str,
        skip_method: &str,
    ) -> Option<Vec<ResolvedType>> {
        // fn_instantiations keys: method base names can be stored either as
        // raw "push" or as mangled-prefix "Vec_push" depending on the TC path.
        // Match both. Also scan method_instantiations in generic instantiations.
        let prefix = format!("{}_", struct_name);
        let skip_base = format!("{}_{}", struct_name, skip_method);
        for (base, insts) in self.generics.fn_instantiations.iter() {
            // Accept either "push" (raw) or "Vec_push" (prefixed) as matching our struct.
            let matches_struct = base.starts_with(&prefix) || {
                // Raw method name — check if the mangled suffix points to our struct.
                insts.iter().any(|(_, mangled)| {
                    mangled.starts_with(&prefix)
                        && mangled != &skip_base
                        && !mangled.contains("$Var")
                })
            };
            if !matches_struct || base == &skip_base || base == skip_method {
                continue;
            }
            for (type_args, mangled) in insts {
                // If base is raw method name, make sure the mangled is for our struct.
                if !base.starts_with(&prefix) && !mangled.starts_with(&prefix) {
                    continue;
                }
                let concrete = !type_args.is_empty()
                    && type_args
                        .iter()
                        .all(|t| !matches!(t, ResolvedType::Var(_) | ResolvedType::Generic(_)));
                if concrete {
                    return Some(type_args.clone());
                }
            }
        }
        None
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

        if type_args
            .iter()
            .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)))
        {
            return None;
        }

        // Only handle generic structs that we have a template for.
        // Previously restricted to {Vec, HashMap, Option}; generalized so that
        // user-defined generic impls also get on-demand specialization when
        // reached from another specialization's body (Phase 191 #10 fix).
        let has_template = self.generics.struct_defs.contains_key(struct_name)
            || self
                .generics
                .generic_method_bodies
                .keys()
                .any(|(s, _)| s == struct_name);
        if !has_template {
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

            // Register the specialized method signature in types.functions BEFORE
            // generating the body, so that: (a) callers looking up return/arg types
            // during codegen find correct concrete types, and (b) recursive
            // references within the body itself resolve (e.g., Vec_push$T calling
            // @.grow() — Vec_grow$T must be looked-up-able mid-specialization).
            let struct_generics = self
                .generics
                .struct_defs
                .get(struct_name)
                .map(|s| s.generics.clone())
                .unwrap_or_default();
            let method_generic_params: Vec<_> = if !method_fn.generics.is_empty() {
                method_fn
                    .generics
                    .iter()
                    .filter(|g| !matches!(g.kind, vais_ast::GenericParamKind::Lifetime { .. }))
                    .collect()
            } else {
                struct_generics
                    .iter()
                    .filter(|g| !matches!(g.kind, vais_ast::GenericParamKind::Lifetime { .. }))
                    .collect()
            };
            let mut substitutions: std::collections::HashMap<String, ResolvedType> =
                method_generic_params
                    .iter()
                    .zip(type_args.iter())
                    .map(|(g, t)| (g.name.node.to_string(), t.clone()))
                    .collect();
            substitutions.insert(
                "Self".to_string(),
                ResolvedType::Named {
                    name: struct_name.to_string(),
                    generics: type_args.to_vec(),
                },
            );
            let params: Vec<_> = method_fn
                .params
                .iter()
                .map(|p| {
                    let name = p.name.node.to_string();
                    let ty = if name == "self" {
                        substitutions
                            .get("Self")
                            .cloned()
                            .unwrap_or(ResolvedType::Unknown)
                    } else {
                        let raw = self.ast_type_to_resolved(&p.ty.node);
                        vais_types::substitute_type(&raw, &substitutions)
                    };
                    (name, ty, p.is_mut)
                })
                .collect();
            let ret_type = method_fn
                .ret_type
                .as_ref()
                .map(|t| {
                    let ty = self.ast_type_to_resolved(&t.node);
                    vais_types::substitute_type(&ty, &substitutions)
                })
                .unwrap_or(ResolvedType::Unit);
            self.types.functions.insert(
                mangled.clone(),
                crate::types::FunctionInfo {
                    signature: vais_types::FunctionSig {
                        name: mangled.clone(),
                        params,
                        ret: ret_type,
                        is_async: method_fn.is_async,
                        ..Default::default()
                    },
                    is_extern: false,
                    _extern_abi: None,
                },
            );

            // Snapshot fn_ctx fields that `initialize_function_state` will clobber.
            // Required because this runs mid-body of another function, so locals
            // like `value`, `self` (belonging to the outer spec) must survive.
            // Why: without this, generating Vec_grow$T from inside Vec_push$T's
            // body erases Vec_push's `value` local → later IR emits fail.
            // How to apply: only around re-entrant specialization; not needed at
            // top-level module_gen because fn_ctx is already empty there.
            let saved_current_function = self.fn_ctx.current_function.take();
            let saved_current_return_type = self.fn_ctx.current_return_type.take();
            let saved_locals = std::mem::take(&mut self.fn_ctx.locals);
            let saved_label_counter = self.fn_ctx.label_counter;
            let saved_loop_stack = std::mem::take(&mut self.fn_ctx.loop_stack);
            let saved_current_block =
                std::mem::replace(&mut self.fn_ctx.current_block, String::from("entry"));
            let saved_future_poll_fns = std::mem::take(&mut self.fn_ctx.future_poll_fns);
            let saved_async_poll_context = self.fn_ctx.async_poll_context.take();
            let saved_alloc_tracker = std::mem::take(&mut self.fn_ctx.alloc_tracker);
            let saved_string_value_slot = std::mem::take(&mut self.fn_ctx.string_value_slot);
            let saved_pending_return_skip_slot =
                std::mem::take(&mut self.fn_ctx.pending_return_skip_slot);
            let saved_var_string_slot = std::mem::take(&mut self.fn_ctx.var_string_slot);
            let saved_var_string_slots_multi =
                std::mem::take(&mut self.fn_ctx.var_string_slots_multi);
            let saved_phi_extra_slots = std::mem::take(&mut self.fn_ctx.phi_extra_slots);
            let saved_temp_var_types = std::mem::take(&mut self.fn_ctx.temp_var_types);
            let saved_scope_stack = std::mem::take(&mut self.fn_ctx.scope_stack);
            let saved_scope_str_stack = std::mem::take(&mut self.fn_ctx.scope_str_stack);
            let saved_scope_drop_label_counter = self.fn_ctx.scope_drop_label_counter;
            let saved_entry_allocas = std::mem::take(&mut self.fn_ctx.entry_allocas);

            // Generate the specialized function
            let result = self.generate_specialized_function(&method_fn, &inst);

            // Restore fn_ctx snapshot (pending_specialized_ir intentionally kept
            // accumulating — it's flushed at module emission).
            self.fn_ctx.current_function = saved_current_function;
            self.fn_ctx.current_return_type = saved_current_return_type;
            self.fn_ctx.locals = saved_locals;
            self.fn_ctx.label_counter = saved_label_counter;
            self.fn_ctx.loop_stack = saved_loop_stack;
            self.fn_ctx.current_block = saved_current_block;
            self.fn_ctx.future_poll_fns = saved_future_poll_fns;
            self.fn_ctx.async_poll_context = saved_async_poll_context;
            self.fn_ctx.alloc_tracker = saved_alloc_tracker;
            self.fn_ctx.string_value_slot = saved_string_value_slot;
            self.fn_ctx.pending_return_skip_slot = saved_pending_return_skip_slot;
            self.fn_ctx.var_string_slot = saved_var_string_slot;
            self.fn_ctx.var_string_slots_multi = saved_var_string_slots_multi;
            self.fn_ctx.phi_extra_slots = saved_phi_extra_slots;
            self.fn_ctx.temp_var_types = saved_temp_var_types;
            self.fn_ctx.scope_stack = saved_scope_stack;
            self.fn_ctx.scope_str_stack = saved_scope_str_stack;
            self.fn_ctx.scope_drop_label_counter = saved_scope_drop_label_counter;
            self.fn_ctx.entry_allocas = saved_entry_allocas;

            match result {
                Ok(ir_code) if !ir_code.trim().is_empty() => {
                    self.fn_ctx.pending_specialized_ir.push(ir_code);
                    Some(mangled)
                }
                Ok(_) | Err(_) => {
                    self.types.functions.remove(&mangled);
                    self.generics.generated_functions.remove(&mangled);
                    None
                }
            }
        } else {
            None
        }
    }

    pub(crate) fn push_expected_static_ctor_substitutions(
        &mut self,
        expected_type: &ResolvedType,
        value: &Expr,
    ) -> Option<HashMap<String, ResolvedType>> {
        let Expr::StaticMethodCall { type_name, .. } = value else {
            return None;
        };

        let expected_inner = match expected_type {
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref(),
            other => other,
        };
        let ResolvedType::Named { name, generics } = expected_inner else {
            return None;
        };
        if name != &type_name.node || generics.is_empty() {
            return None;
        }

        let struct_def = self.generics.struct_defs.get(&type_name.node)?;
        let generic_params: Vec<String> = struct_def
            .generics
            .iter()
            .filter(|param| !matches!(param.kind, vais_ast::GenericParamKind::Lifetime { .. }))
            .map(|param| param.name.node.clone())
            .collect();
        if generic_params.is_empty() {
            return None;
        }

        let mut concrete_pairs = Vec::new();
        for (param, concrete) in generic_params.iter().zip(generics.iter()) {
            if matches!(concrete, ResolvedType::Generic(_) | ResolvedType::Var(_)) {
                return None;
            }
            concrete_pairs.push((param.clone(), concrete.clone()));
        }
        if concrete_pairs.is_empty() {
            return None;
        }

        let saved = self.generics.substitutions.clone();
        for (param, concrete) in concrete_pairs {
            self.generics.substitutions.insert(param, concrete);
        }
        Some(saved)
    }

    fn static_type_args_from_current_substitutions(
        &self,
        struct_name: &str,
    ) -> Option<Vec<ResolvedType>> {
        let struct_def = self.generics.struct_defs.get(struct_name)?;
        let mut type_args = Vec::new();
        for param in &struct_def.generics {
            if matches!(param.kind, vais_ast::GenericParamKind::Lifetime { .. }) {
                continue;
            }
            let concrete = self.generics.substitutions.get(&param.name.node)?;
            if matches!(concrete, ResolvedType::Generic(_) | ResolvedType::Var(_)) {
                return None;
            }
            type_args.push(concrete.clone());
        }
        if type_args.is_empty() {
            None
        } else {
            Some(type_args)
        }
    }
    /// Resolve a type suffix (from mangled name like "u8", "str", "GraphNode") to ResolvedType
    fn resolve_type_suffix_to_resolved(&self, suffix: &str) -> ResolvedType {
        match suffix {
            "u8" => ResolvedType::U8,
            "i8" => ResolvedType::I8,
            "u16" => ResolvedType::U16,
            "i16" => ResolvedType::I16,
            "u32" => ResolvedType::U32,
            "i32" => ResolvedType::I32,
            "f32" => ResolvedType::F32,
            "u64" => ResolvedType::U64,
            "i64" => ResolvedType::I64,
            "f64" => ResolvedType::F64,
            "str" => ResolvedType::Str,
            "bool" => ResolvedType::Bool,
            other => ResolvedType::Named {
                name: other.to_string(),
                generics: vec![],
            },
        }
    }
}
