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
        let full_method_name = if let ResolvedType::Named { name, generics } = &recv_type {
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
                base
            }
        } else {
            method_name.clone()
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
                    self.type_to_llvm(&inferred_ty)
                } else {
                    self.type_to_llvm(pt)
                }
            } else {
                self.type_to_llvm(&inferred_ty)
            };

            // Integer width coercion
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

            // For struct types, load the value from pointer if the expression produces a pointer.
            // Struct literals and local struct variables return pointers (alloca),
            // but function params expect values.
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
                (self.type_to_llvm(&info.signature.ret), info.signature.ret.clone())
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
    pub(crate) fn generate_static_method_call_expr(
        &mut self,
        type_name: &Spanned<String>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();

        let full_method_name = format!("{}_{}", type_name.node, method.node);

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
                        write_ir!(ir, "  {} = trunc {} {} to {}", conv_tmp, src_ty, val, dst_ty);
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
}
