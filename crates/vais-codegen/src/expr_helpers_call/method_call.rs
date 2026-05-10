use super::*;
use vais_ast::{Expr, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
    #[inline(never)]
    pub(crate) fn generate_method_call_expr(
        &mut self,
        receiver: &Spanned<Expr>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        call_span: Option<vais_ast::Span>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (recv_val, recv_ir, mut recv_type) = if matches!(&receiver.node, Expr::SelfCall) {
            if let Some(local) = self.fn_ctx.locals.get("self") {
                let recv_type = local.ty.clone();
                ("%self".to_string(), String::new(), recv_type)
            } else {
                return Err(CodegenError::Unsupported(
                    "@.method() used outside of a method with self".to_string(),
                ));
            }
        } else if let Expr::Index { expr, index } = &receiver.node {
            let recv_type = self.infer_expr_type(receiver);
            let receiver_is_named = match &recv_type {
                ResolvedType::Named { .. } => true,
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                    matches!(inner.as_ref(), ResolvedType::Named { .. })
                }
                _ => false,
            };
            if receiver_is_named {
                self.generate_index_lvalue_ptr_expr(expr, index, counter)?
            } else {
                let (recv_val, recv_ir) = self.generate_expr(receiver, counter)?;
                (recv_val, recv_ir, recv_type)
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

        // String method calls: str.len(), &str.len(), str.charAt(), etc.
        let is_str_receiver = matches!(recv_type, ResolvedType::Str)
            || matches!(
                &recv_type,
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
                    if matches!(inner.as_ref(), ResolvedType::Str)
            );
        if is_str_receiver {
            return self.generate_string_method_call(&recv_val, &ir, method_name, args, counter);
        }

        if args.is_empty()
            && matches!(
                method_name.as_str(),
                "to_le_bytes" | "to_be_bytes" | "to_ne_bytes"
            )
            && matches!(
                recv_type,
                ResolvedType::I8
                    | ResolvedType::U8
                    | ResolvedType::I16
                    | ResolvedType::U16
                    | ResolvedType::I32
                    | ResolvedType::U32
                    | ResolvedType::I64
                    | ResolvedType::U64
                    | ResolvedType::F32
                    | ResolvedType::F64
            )
        {
            let mut ir = ir;
            let (bits_ty, byte_count, bits_val) = match recv_type {
                ResolvedType::F32 => {
                    let bits = self.next_temp(counter);
                    write_ir!(ir, "  {} = bitcast float {} to i32", bits, recv_val);
                    self.fn_ctx.record_emitted_type(&bits, "i32");
                    ("i32", 4usize, bits)
                }
                ResolvedType::F64 => {
                    let bits = self.next_temp(counter);
                    write_ir!(ir, "  {} = bitcast double {} to i64", bits, recv_val);
                    self.fn_ctx.record_emitted_type(&bits, "i64");
                    ("i64", 8usize, bits)
                }
                ResolvedType::I8 | ResolvedType::U8 => {
                    let actual = self.llvm_type_of(&recv_val);
                    let byte = if actual == "i8" {
                        recv_val.clone()
                    } else {
                        let truncated = self.next_temp(counter);
                        write_ir!(ir, "  {} = trunc {} {} to i8", truncated, actual, recv_val);
                        self.fn_ctx.record_emitted_type(&truncated, "i8");
                        truncated
                    };
                    ("i8", 1usize, byte)
                }
                ResolvedType::I16 | ResolvedType::U16 => {
                    let actual = self.llvm_type_of(&recv_val);
                    let bits = if actual == "i16" {
                        recv_val.clone()
                    } else {
                        let truncated = self.next_temp(counter);
                        write_ir!(ir, "  {} = trunc {} {} to i16", truncated, actual, recv_val);
                        self.fn_ctx.record_emitted_type(&truncated, "i16");
                        truncated
                    };
                    ("i16", 2usize, bits)
                }
                ResolvedType::I32 | ResolvedType::U32 => {
                    let actual = self.llvm_type_of(&recv_val);
                    let bits = if actual == "i32" {
                        recv_val.clone()
                    } else {
                        let truncated = self.next_temp(counter);
                        write_ir!(ir, "  {} = trunc {} {} to i32", truncated, actual, recv_val);
                        self.fn_ctx.record_emitted_type(&truncated, "i32");
                        truncated
                    };
                    ("i32", 4usize, bits)
                }
                _ => ("i64", 8usize, recv_val.clone()),
            };

            let vec_ty = ResolvedType::Named {
                name: "Vec".to_string(),
                generics: vec![ResolvedType::U8],
            };
            let vec_llvm = self.type_to_llvm(&vec_ty);
            let ctor = self
                .try_generate_vec_specialization(
                    "Vec",
                    "with_capacity",
                    &[ResolvedType::U8],
                    counter,
                )
                .unwrap_or_else(|| "Vec_with_capacity$u8".to_string());
            let push = self
                .try_generate_vec_specialization("Vec", "push", &[ResolvedType::U8], counter)
                .unwrap_or_else(|| "Vec_push$u8".to_string());
            let vec_val = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = call {} @{}(i64 {})",
                vec_val,
                vec_llvm,
                ctor,
                byte_count
            );
            self.fn_ctx.register_temp_type(&vec_val, vec_ty.clone());
            let vec_ptr = self.next_temp(counter);
            self.emit_entry_alloca(&vec_ptr, &vec_llvm);
            write_ir!(
                ir,
                "  store {} {}, {}* {}",
                vec_llvm,
                vec_val,
                vec_llvm,
                vec_ptr
            );

            for out_idx in 0..byte_count {
                let shift_idx = if method_name == "to_be_bytes" {
                    byte_count - 1 - out_idx
                } else {
                    out_idx
                };
                let byte_val = if byte_count == 1 {
                    bits_val.clone()
                } else {
                    let shift = (shift_idx * 8) as u32;
                    let shifted = if shift == 0 {
                        bits_val.clone()
                    } else {
                        let tmp = self.next_temp(counter);
                        write_ir!(ir, "  {} = lshr {} {}, {}", tmp, bits_ty, bits_val, shift);
                        self.fn_ctx.record_emitted_type(&tmp, bits_ty);
                        tmp
                    };
                    let masked = self.next_temp(counter);
                    write_ir!(ir, "  {} = and {} {}, 255", masked, bits_ty, shifted);
                    self.fn_ctx.record_emitted_type(&masked, bits_ty);
                    let byte = self.next_temp(counter);
                    write_ir!(ir, "  {} = trunc {} {} to i8", byte, bits_ty, masked);
                    self.fn_ctx.record_emitted_type(&byte, "i8");
                    byte
                };
                let _push_ret = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call i64 @{}(%Vec* {}, i8 {})",
                    _push_ret,
                    push,
                    vec_ptr,
                    byte_val
                );
            }

            let result = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = load {}, {}* {}",
                result,
                vec_llvm,
                vec_llvm,
                vec_ptr
            );
            self.fn_ctx.register_temp_type(&result, vec_ty);
            return Ok((result, ir));
        }

        // Option<T>.ok_or(E): lower directly to the builtin Result ABI.
        // The std Option surface exposes this as a method even when no
        // monomorphized Option_ok_or function exists in the current module.
        if matches!(method_name.as_str(), "ok_or" | "ok_or_else") && args.len() == 1 {
            let option_inner = match &recv_type {
                ResolvedType::Named { name, generics } if name == "Option" => {
                    generics.first().cloned()
                }
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => match inner.as_ref() {
                    ResolvedType::Named { name, generics } if name == "Option" => {
                        generics.first().cloned()
                    }
                    _ => None,
                },
                _ => None,
            };

            if let Some(ok_ty) = option_inner {
                let (err_val, err_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&err_ir);
                let err_ty = self.infer_expr_type(&args[0]);
                let result_ty = ResolvedType::Named {
                    name: "Result".to_string(),
                    generics: vec![ok_ty, err_ty.clone()],
                };
                let result_llvm = self.type_to_llvm(&result_ty);
                let option_llvm = "%Option".to_string();
                let recv_actual = self
                    .llvm_type_of_checked(&recv_val)
                    .unwrap_or_else(|| self.llvm_type_of(&recv_val));
                let option_ptr = if recv_actual == "%Option*" || recv_actual == "ptr" {
                    recv_val.clone()
                } else if recv_actual.ends_with('*') {
                    let casted = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = bitcast {} {} to %Option*",
                        casted,
                        recv_actual,
                        recv_val
                    );
                    self.fn_ctx.record_emitted_type(&casted, "%Option*");
                    casted
                } else {
                    let tmp = self.next_temp(counter);
                    self.emit_entry_alloca(&tmp, &option_llvm);
                    write_ir!(
                        ir,
                        "  store {} {}, {}* {}",
                        option_llvm,
                        recv_val,
                        option_llvm,
                        tmp
                    );
                    tmp
                };

                let result_ptr = self.next_temp(counter);
                self.emit_entry_alloca(&result_ptr, &result_llvm);
                let opt_tag_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr %Option, %Option* {}, i32 0, i32 0",
                    opt_tag_ptr,
                    option_ptr
                );
                let opt_tag = self.next_temp(counter);
                write_ir!(ir, "  {} = load i32, i32* {}", opt_tag, opt_tag_ptr);
                self.fn_ctx.record_emitted_type(&opt_tag, "i32");
                let is_some = self.next_temp(counter);
                write_ir!(ir, "  {} = icmp eq i32 {}, 1", is_some, opt_tag);
                self.fn_ctx.record_emitted_type(&is_some, "i1");
                let some_label = self.next_label("option.ok_or.some");
                let err_label = self.next_label("option.ok_or.err");
                let merge_label = self.next_label("option.ok_or.merge");
                write_ir!(
                    ir,
                    "  br i1 {}, label %{}, label %{}",
                    is_some,
                    some_label,
                    err_label
                );

                write_ir!(ir, "{}:", some_label);
                let result_tag_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                    result_tag_ptr,
                    result_llvm,
                    result_llvm,
                    result_ptr
                );
                write_ir!(ir, "  store i32 0, i32* {}", result_tag_ptr);
                let opt_payload_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr %Option, %Option* {}, i32 0, i32 1, i32 0",
                    opt_payload_ptr,
                    option_ptr
                );
                let opt_payload = self.next_temp(counter);
                write_ir!(ir, "  {} = load i64, i64* {}", opt_payload, opt_payload_ptr);
                self.fn_ctx.record_emitted_type(&opt_payload, "i64");
                let result_payload_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr {}, {}* {}, i32 0, i32 1, i32 0",
                    result_payload_ptr,
                    result_llvm,
                    result_llvm,
                    result_ptr
                );
                write_ir!(
                    ir,
                    "  store i64 {}, i64* {}",
                    opt_payload,
                    result_payload_ptr
                );
                write_ir!(ir, "  br label %{}", merge_label);

                write_ir!(ir, "{}:", err_label);
                let result_tag_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                    result_tag_ptr,
                    result_llvm,
                    result_llvm,
                    result_ptr
                );
                write_ir!(ir, "  store i32 1, i32* {}", result_tag_ptr);
                let result_payload_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr {}, {}* {}, i32 0, i32 1, i32 0",
                    result_payload_ptr,
                    result_llvm,
                    result_llvm,
                    result_ptr
                );

                let err_llvm = self.type_to_llvm(&err_ty);
                let err_payload = if err_val == "void" {
                    "0".to_string()
                } else {
                    let actual = self
                        .llvm_type_of_checked(&err_val)
                        .unwrap_or_else(|| self.llvm_type_of(&err_val));
                    if err_llvm == "i64" {
                        err_val.clone()
                    } else if matches!(err_llvm.as_str(), "i1" | "i8" | "i16" | "i32") {
                        let widened = self.next_temp(counter);
                        write_ir!(ir, "  {} = zext {} {} to i64", widened, err_llvm, err_val);
                        self.fn_ctx.record_emitted_type(&widened, "i64");
                        widened
                    } else if err_llvm == "float" || err_llvm == "double" {
                        let cast_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = bitcast i64* {} to {}*",
                            cast_ptr,
                            result_payload_ptr,
                            err_llvm
                        );
                        write_ir!(
                            ir,
                            "  store {} {}, {}* {}",
                            err_llvm,
                            err_val,
                            err_llvm,
                            cast_ptr
                        );
                        String::new()
                    } else if err_llvm.ends_with('*') {
                        let casted = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = ptrtoint {} {} to i64",
                            casted,
                            err_llvm,
                            err_val
                        );
                        self.fn_ctx.record_emitted_type(&casted, "i64");
                        casted
                    } else {
                        let err_size = self.compute_sizeof(&err_ty);
                        if err_size <= 8 {
                            let cast_ptr = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = bitcast i64* {} to {}*",
                                cast_ptr,
                                result_payload_ptr,
                                err_llvm
                            );
                            let store_val = if actual == format!("{}*", err_llvm)
                                || actual == "ptr"
                                || actual.ends_with('*')
                            {
                                let src_ptr = if actual == format!("{}*", err_llvm) {
                                    err_val.clone()
                                } else {
                                    let casted = self.next_temp(counter);
                                    write_ir!(
                                        ir,
                                        "  {} = bitcast {} {} to {}*",
                                        casted,
                                        actual,
                                        err_val,
                                        err_llvm
                                    );
                                    self.fn_ctx
                                        .record_emitted_type(&casted, &format!("{}*", err_llvm));
                                    casted
                                };
                                let loaded = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = load {}, {}* {}",
                                    loaded,
                                    err_llvm,
                                    err_llvm,
                                    src_ptr
                                );
                                loaded
                            } else {
                                err_val.clone()
                            };
                            write_ir!(
                                ir,
                                "  store {} {}, {}* {}",
                                err_llvm,
                                store_val,
                                err_llvm,
                                cast_ptr
                            );
                            String::new()
                        } else {
                            let heap_ptr = self.next_temp(counter);
                            write_ir!(ir, "  {} = call i8* @malloc(i64 {})", heap_ptr, err_size);
                            self.fn_ctx.record_emitted_type(&heap_ptr, "i8*");
                            let typed_ptr = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = bitcast i8* {} to {}*",
                                typed_ptr,
                                heap_ptr,
                                err_llvm
                            );
                            self.fn_ctx
                                .record_emitted_type(&typed_ptr, &format!("{}*", err_llvm));
                            let store_val = if actual == format!("{}*", err_llvm)
                                || actual == "ptr"
                                || actual.ends_with('*')
                            {
                                let src_ptr = if actual == format!("{}*", err_llvm) {
                                    err_val.clone()
                                } else {
                                    let casted = self.next_temp(counter);
                                    write_ir!(
                                        ir,
                                        "  {} = bitcast {} {} to {}*",
                                        casted,
                                        actual,
                                        err_val,
                                        err_llvm
                                    );
                                    self.fn_ctx
                                        .record_emitted_type(&casted, &format!("{}*", err_llvm));
                                    casted
                                };
                                let loaded = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = load {}, {}* {}",
                                    loaded,
                                    err_llvm,
                                    err_llvm,
                                    src_ptr
                                );
                                loaded
                            } else {
                                err_val.clone()
                            };
                            write_ir!(
                                ir,
                                "  store {} {}, {}* {}",
                                err_llvm,
                                store_val,
                                err_llvm,
                                typed_ptr
                            );
                            let ptr_i64 = self.next_temp(counter);
                            write_ir!(ir, "  {} = ptrtoint i8* {} to i64", ptr_i64, heap_ptr);
                            self.fn_ctx.record_emitted_type(&ptr_i64, "i64");
                            ptr_i64
                        }
                    }
                };
                if !err_payload.is_empty() {
                    write_ir!(
                        ir,
                        "  store i64 {}, i64* {}",
                        err_payload,
                        result_payload_ptr
                    );
                }
                write_ir!(ir, "  br label %{}", merge_label);

                write_ir!(ir, "{}:", merge_label);
                self.fn_ctx.current_block = merge_label;
                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = load {}, {}* {}",
                    result,
                    result_llvm,
                    result_llvm,
                    result_ptr
                );
                self.fn_ctx.register_temp_type(&result, result_ty);
                self.fn_ctx.record_emitted_type(&result, &result_llvm);
                return Ok((result, ir));
            }
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
            if matches!(
                inner_recv,
                ResolvedType::Slice(_) | ResolvedType::SliceMut(_)
            ) {
                // Slice clone has Rust-like Vec<T> ownership semantics in the
                // type checker, so let the slice materialization path below
                // allocate and copy the elements instead of returning the
                // borrowed fat pointer unchanged.
            } else {
                if matches!(inner_recv, ResolvedType::Named { .. }) {
                    let llvm_ty = self.type_to_llvm(inner_recv);
                    let actual_recv_ty = self
                        .llvm_type_of_checked(&recv_val)
                        .unwrap_or_else(|| self.llvm_type_of(&recv_val));
                    let expected_ptr_ty = format!("{}*", llvm_ty);
                    // Check if the receiver is already a pointer. This covers
                    // locals/fields and derived lvalues such as `vec[idx]`.
                    let is_ptr_receiver = actual_recv_ty == expected_ptr_ty
                        || actual_recv_ty == "ptr"
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
                        let loaded = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = load {}, {}* {}",
                            loaded,
                            llvm_ty,
                            llvm_ty,
                            recv_val
                        );
                        self.fn_ctx.record_emitted_type(&loaded, &llvm_ty);
                        return Ok((loaded, ir));
                    }
                }
                return Ok((recv_val, ir));
            }
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

        // Slice .to_vec() — materialize an owned Vec<T> from the slice fat pointer.
        if (method_name == "to_vec" || method_name == "clone") && args.is_empty() {
            let slice_elem = match &recv_type {
                ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => {
                    Some(elem.as_ref().clone())
                }
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => match inner.as_ref() {
                    ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => {
                        Some(elem.as_ref().clone())
                    }
                    _ => None,
                },
                _ => None,
            };
            if let Some(elem_ty) = slice_elem {
                let vec_resolved = ResolvedType::Named {
                    name: "Vec".to_string(),
                    generics: vec![elem_ty.clone()],
                };
                let vec_llvm = self.type_to_llvm(&vec_resolved);
                let with_capacity =
                    vais_types::mangle_name("Vec_with_capacity", &[elem_ty.clone()]);

                let data_i8 = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                    data_i8,
                    recv_val
                );
                self.fn_ctx.record_emitted_type(&data_i8, "i8*");
                let len = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {{ i8*, i64 }} {}, 1",
                    len,
                    recv_val
                );
                self.fn_ctx.record_emitted_type(&len, "i64");

                let vec_val = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call {} @{}(i64 {})",
                    vec_val,
                    vec_llvm,
                    with_capacity,
                    len
                );
                self.fn_ctx
                    .register_temp_type(&vec_val, vec_resolved.clone());

                let dst_i64 = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {} {}, 0",
                    dst_i64,
                    vec_llvm,
                    vec_val
                );
                self.fn_ctx.record_emitted_type(&dst_i64, "i64");
                let byte_len = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = mul i64 {}, {}",
                    byte_len,
                    len,
                    self.compute_sizeof(&elem_ty)
                );
                self.fn_ctx.record_emitted_type(&byte_len, "i64");
                let dst_i8 = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i8*", dst_i8, dst_i64);
                self.fn_ctx.record_emitted_type(&dst_i8, "i8*");
                self.needs_llvm_memcpy = true;
                write_ir!(
                    ir,
                    "  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {}, i8* {}, i64 {}, i1 false)",
                    dst_i8,
                    data_i8,
                    byte_len
                );

                let vec_with_len = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = insertvalue {} {}, i64 {}, 1",
                    vec_with_len,
                    vec_llvm,
                    vec_val,
                    len
                );
                self.fn_ctx.register_temp_type(&vec_with_len, vec_resolved);
                return Ok((vec_with_len, ir));
            }
        }

        // ByteBuffer.as_bytes() is a zero-copy slice view over (data, len).
        // The type checker exposes it as `&[u8]`; emit the fat pointer directly
        // instead of calling a std method that would need to manufacture a slice.
        let inner_recv_for_builtin = match &recv_type {
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref(),
            other => other,
        };
        if method_name == "as_bytes" && args.is_empty() {
            if matches!(inner_recv_for_builtin, ResolvedType::Str) {
                let recv_llvm = self.llvm_type_of(&recv_val);
                let fat_val = if recv_llvm == "{ i8*, i64 }" {
                    recv_val.clone()
                } else if recv_llvm == "{ i8*, i64 }*" {
                    let loaded = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = load {{ i8*, i64 }}, {{ i8*, i64 }}* {}",
                        loaded,
                        recv_val
                    );
                    self.fn_ctx.record_emitted_type(&loaded, "{ i8*, i64 }");
                    loaded
                } else if recv_llvm == "ptr" {
                    let loaded = self.next_temp(counter);
                    write_ir!(ir, "  {} = load {{ i8*, i64 }}, ptr {}", loaded, recv_val);
                    self.fn_ctx.record_emitted_type(&loaded, "{ i8*, i64 }");
                    loaded
                } else {
                    recv_val.clone()
                };

                let data_i8 = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                    data_i8,
                    fat_val
                );
                self.fn_ctx.record_emitted_type(&data_i8, "i8*");
                let data_i64 = self.next_temp(counter);
                write_ir!(ir, "  {} = ptrtoint i8* {} to i64", data_i64, data_i8);
                self.fn_ctx.record_emitted_type(&data_i64, "i64");
                let len = self.next_temp(counter);
                write_ir!(ir, "  {} = extractvalue {{ i8*, i64 }} {}, 1", len, fat_val);
                self.fn_ctx.record_emitted_type(&len, "i64");

                let vec_ty = "%Vec$u8";
                let v0 = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = insertvalue {} undef, i64 {}, 0",
                    v0,
                    vec_ty,
                    data_i64
                );
                let v1 = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = insertvalue {} {}, i64 {}, 1",
                    v1,
                    vec_ty,
                    v0,
                    len
                );
                let v2 = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = insertvalue {} {}, i64 {}, 2",
                    v2,
                    vec_ty,
                    v1,
                    len
                );
                let v3 = self.next_temp(counter);
                write_ir!(ir, "  {} = insertvalue {} {}, i64 1, 3", v3, vec_ty, v2);
                let vec = self.next_temp(counter);
                write_ir!(ir, "  {} = insertvalue {} {}, i64 0, 4", vec, vec_ty, v3);
                self.fn_ctx.record_emitted_type(&vec, vec_ty);
                self.fn_ctx.register_temp_type(
                    &vec,
                    ResolvedType::Named {
                        name: "Vec".to_string(),
                        generics: vec![ResolvedType::U8],
                    },
                );
                return Ok((vec, ir));
            }

            if let ResolvedType::Named { name, .. } = inner_recv_for_builtin {
                let resolved = self.resolve_struct_name(name);
                if name == "ByteBuffer" || resolved == "ByteBuffer" {
                    let recv_llvm = self.llvm_type_of(&recv_val);
                    let (data_i64, len_i64) = if recv_llvm == "%ByteBuffer"
                        || recv_llvm == "%ByteBuffer = type { i64, i64, i64, i64 }"
                    {
                        let data_i64 = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = extractvalue %ByteBuffer {}, 0",
                            data_i64,
                            recv_val
                        );
                        let len_i64 = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = extractvalue %ByteBuffer {}, 1",
                            len_i64,
                            recv_val
                        );
                        (data_i64, len_i64)
                    } else {
                        let data_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr %ByteBuffer, %ByteBuffer* {}, i32 0, i32 0",
                            data_ptr,
                            recv_val
                        );
                        let data_i64 = self.next_temp(counter);
                        write_ir!(ir, "  {} = load i64, i64* {}", data_i64, data_ptr);
                        let len_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr %ByteBuffer, %ByteBuffer* {}, i32 0, i32 1",
                            len_ptr,
                            recv_val
                        );
                        let len_i64 = self.next_temp(counter);
                        write_ir!(ir, "  {} = load i64, i64* {}", len_i64, len_ptr);
                        (data_i64, len_i64)
                    };
                    let data_i8 = self.next_temp(counter);
                    write_ir!(ir, "  {} = inttoptr i64 {} to i8*", data_i8, data_i64);
                    let slice0 = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
                        slice0,
                        data_i8
                    );
                    let slice = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1",
                        slice,
                        slice0,
                        len_i64
                    );
                    self.fn_ctx.record_emitted_type(&slice, "{ i8*, i64 }");
                    return Ok((slice, ir));
                }
            }
        }

        // Box<T>.as_ref() is represented as an i64 heap pointer in LLVM. Emit
        // the typed pointer directly instead of routing through a method
        // declaration that treats the receiver as an `i64*`.
        if method_name == "as_ref" && args.is_empty() {
            if let ResolvedType::Named { name, generics } = inner_recv_for_builtin {
                if name == "Box" && generics.len() == 1 {
                    let inner_llvm = self.type_to_llvm(&generics[0]);
                    let recv_llvm = self.llvm_type_of(&recv_val);
                    let raw_ptr = if recv_llvm == "i64" {
                        recv_val.clone()
                    } else if recv_llvm == "i64*" {
                        let loaded = self.next_temp(counter);
                        write_ir!(ir, "  {} = load i64, i64* {}", loaded, recv_val);
                        loaded
                    } else {
                        recv_val.clone()
                    };
                    let typed_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = inttoptr i64 {} to {}*",
                        typed_ptr,
                        raw_ptr,
                        inner_llvm
                    );
                    self.fn_ctx
                        .record_emitted_type(&typed_ptr, &format!("{}*", inner_llvm));
                    return Ok((typed_ptr, ir));
                }
            }
        }

        // Use resolve_struct_name to match definition naming (e.g., Pair → Pair$i64)
        // For generic structs with type args, try mangled name first (e.g., Vec_push$GraphNode)
        // Unwrap Ref/RefMut to get the inner Named type
        let inner_recv_type_owned = match &recv_type {
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref().clone(),
            other => other.clone(),
        };
        let inner_recv_type = &inner_recv_type_owned;
        let guard_forward_inner = match inner_recv_type {
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
        let effective_recv_type_owned = guard_forward_inner
            .clone()
            .unwrap_or_else(|| inner_recv_type_owned.clone());
        let effective_recv_type = &effective_recv_type_owned;
        let full_method_name = if let ResolvedType::Named { name, generics } = effective_recv_type {
            let resolved = self.resolve_struct_name(name);
            // Method names are mangled as `<base>_<method>$<typeargs>` (e.g.
            // `Vec_push$u8`), not `<specialized>_<method>` (e.g.
            // `Vec$u8_push`). Use the base struct name here even though
            // `resolved` may already be specialized — `mangle_name` below
            // appends the type-arg suffix.
            let base = format!("{}_{}", name, method_name);

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
                        // `try_generate_vec_specialization` expects the base
                        // struct name (e.g. "Vec") — passing the specialized
                        // form ("Vec$u8") would miss the method-body lookup
                        // and silently emit an unmangled callsite.
                        self.try_generate_vec_specialization(name, method_name, generics, counter)
                            .unwrap_or_else(|| mangled.clone())
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
            // Receiver type is not Named (e.g., I64 from fallback inference,
            // or `&dyn Trait`/`&mut dyn Trait` parameter — see STEP7_FINDINGS
            // F-23 for the silent-corruption case the latter triggers in the
            // text-IR path; the inkwell-path twin is in inkwell/gen_aggregate.rs).
            //
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
                candidates
                    .into_iter()
                    .next()
                    .expect("invariant: candidates.len() == 1 checked immediately before iteration")
            } else {
                method_name.clone()
            }
        };

        // Look up function info for parameter types
        let fn_info = self.types.functions.get(&full_method_name).cloned();

        if method_name == "push" {
            let pushed_elem_ty = full_method_name
                .strip_prefix("Vec_push$")
                .map(|type_suffix| self.resolve_type_suffix_to_resolved(type_suffix))
                .or_else(|| {
                    if args.len() == 1
                        && matches!(
                            effective_recv_type,
                            ResolvedType::Named { name, .. } if name == "Vec"
                        )
                    {
                        Some(self.infer_expr_type(&args[0]))
                    } else {
                        None
                    }
                });

            if let (Some(elem_ty), Expr::Ident(recv_name)) = (pushed_elem_ty, &receiver.node) {
                if !matches!(
                    elem_ty,
                    ResolvedType::Var(_) | ResolvedType::Generic(_) | ResolvedType::Never
                ) {
                    if let Some(local) = self.fn_ctx.locals.get_mut(recv_name) {
                        if let ResolvedType::Named { name, generics } = &local.ty {
                            let needs_update = name == "Vec"
                                && (generics.is_empty()
                                    || generics.first().is_some_and(|g| {
                                        matches!(
                                            g,
                                            ResolvedType::Var(_)
                                                | ResolvedType::Generic(_)
                                                | ResolvedType::Never
                                                | ResolvedType::Unknown
                                        )
                                    }));
                            if needs_update {
                                let updated_ty = ResolvedType::Named {
                                    name: "Vec".to_string(),
                                    generics: vec![elem_ty],
                                };
                                local.ty = updated_ty.clone();
                                recv_type = updated_ty;
                            }
                        }
                    }
                }
            }
        }

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
        let mut recv_val = if recv_llvm_ty.ends_with('*')
            && !matches!(&receiver.node, Expr::SelfCall)
            && self.is_expr_value(receiver)
            && matches!(&recv_type, ResolvedType::Named { .. })
        {
            let struct_llvm = self.type_to_llvm(&recv_type);
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

        let recv_is_dyn_trait = match &recv_type {
            ResolvedType::DynTrait { .. } => true,
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                matches!(inner.as_ref(), ResolvedType::DynTrait { .. })
            }
            _ => false,
        };
        if recv_is_dyn_trait
            && recv_llvm_ty.ends_with('*')
            && recv_llvm_ty != crate::vtable::TRAIT_OBJECT_TYPE
        {
            let actual_recv_ty = self.llvm_type_of_checked(&recv_val);
            if actual_recv_ty
                .as_deref()
                .map_or(true, |ty| ty == crate::vtable::TRAIT_OBJECT_TYPE)
            {
                let data_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {} {}, 0",
                    data_ptr,
                    crate::vtable::TRAIT_OBJECT_TYPE,
                    recv_val
                );
                self.fn_ctx.record_emitted_type(&data_ptr, "i8*");
                let typed_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = bitcast i8* {} to {}",
                    typed_ptr,
                    data_ptr,
                    recv_llvm_ty
                );
                self.fn_ctx.record_emitted_type(&typed_ptr, &recv_llvm_ty);
                recv_val = typed_ptr;
            }
        }
        if let Some(inner_ty) = &guard_forward_inner {
            let guard_llvm = self.type_to_llvm(&recv_type);
            let guard_ptr = match self.llvm_type_of_checked(&recv_val).as_deref() {
                Some(actual) if actual == format!("{}*", guard_llvm) || actual == "ptr" => {
                    recv_val.clone()
                }
                Some(actual) if actual.ends_with('*') => {
                    let casted = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = bitcast {} {} to {}*",
                        casted,
                        actual,
                        recv_val,
                        guard_llvm
                    );
                    self.fn_ctx
                        .record_emitted_type(&casted, &format!("{}*", guard_llvm));
                    casted
                }
                _ => recv_val.clone(),
            };
            let mutex_word_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                mutex_word_ptr,
                guard_llvm,
                guard_llvm,
                guard_ptr
            );
            self.fn_ctx.record_emitted_type(&mutex_word_ptr, "i64*");
            let mutex_word = self.next_temp(counter);
            write_ir!(ir, "  {} = load i64, i64* {}", mutex_word, mutex_word_ptr);
            self.fn_ctx.record_emitted_type(&mutex_word, "i64");
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
            self.fn_ctx
                .record_emitted_type(&mutex_ptr, &format!("{}*", mutex_llvm));
            let inner_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                inner_ptr,
                mutex_llvm,
                mutex_llvm,
                mutex_ptr
            );
            self.fn_ctx
                .record_emitted_type(&inner_ptr, &format!("{}*", self.type_to_llvm(inner_ty)));
            recv_val = if recv_llvm_ty == self.llvm_type_of(&inner_ptr) {
                inner_ptr
            } else {
                let casted = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = bitcast {} {} to {}",
                    casted,
                    self.llvm_type_of(&inner_ptr),
                    inner_ptr,
                    recv_llvm_ty
                );
                self.fn_ctx.record_emitted_type(&casted, &recv_llvm_ty);
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
            let mut did_vec_to_slice = false;
            let arg_llvm_ty = if let Some(ref pt) = param_ty {
                if let ResolvedType::Generic(ref name) = pt {
                    // Check if a concrete substitution exists for this generic parameter
                    if let Some(concrete_ty) = self.get_generic_substitution(name) {
                        // After substitution, the substituted type may be a Slice
                        // while the actual arg is a Vec — handle that coercion
                        // here too (mirrors generate_static_method_call_expr path).
                        let sub = concrete_ty.clone();
                        if Self::is_vec_to_slice_coercion(&sub, &inferred_ty) {
                            let actual_ty = self.llvm_type_of(&val);
                            if actual_ty != "{ i8*, i64 }" {
                                let vec_struct_ty = actual_ty
                                    .strip_suffix('*')
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| "%Vec".to_string());
                                let data_gep = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                                    data_gep,
                                    vec_struct_ty,
                                    vec_struct_ty,
                                    val
                                );
                                let data_i64 = self.next_temp(counter);
                                write_ir!(ir, "  {} = load i64, i64* {}", data_i64, data_gep);
                                self.fn_ctx.record_emitted_type(&data_i64, "i64");
                                let data_i8 = self.next_temp(counter);
                                write_ir!(ir, "  {} = inttoptr i64 {} to i8*", data_i8, data_i64);
                                let len_gep = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = getelementptr {}, {}* {}, i32 0, i32 1",
                                    len_gep,
                                    vec_struct_ty,
                                    vec_struct_ty,
                                    val
                                );
                                let len_val = self.next_temp(counter);
                                write_ir!(ir, "  {} = load i64, i64* {}", len_val, len_gep);
                                self.fn_ctx.record_emitted_type(&len_val, "i64");
                                let fat1 = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
                                    fat1,
                                    data_i8
                                );
                                self.fn_ctx.record_emitted_type(&fat1, "{ i8*, i64 }");
                                let fat2 = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1",
                                    fat2,
                                    fat1,
                                    len_val
                                );
                                self.fn_ctx.record_emitted_type(&fat2, "{ i8*, i64 }");
                                val = fat2;
                            }
                            did_vec_to_slice = true;
                            "{ i8*, i64 }".to_string()
                        } else {
                            self.type_to_llvm(&sub)
                        }
                    } else {
                        // Generic params are erased to i64 in LLVM IR
                        // Use i64 as the LLVM type, and coerce if needed
                        "i64".to_string()
                    }
                } else if Self::is_vec_to_slice_coercion(pt, &inferred_ty) {
                    // Non-generic param explicitly typed as Slice but arg is a Vec.
                    let actual_ty = self.llvm_type_of(&val);
                    if actual_ty != "{ i8*, i64 }" {
                        let vec_struct_ty = actual_ty
                            .strip_suffix('*')
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "%Vec".to_string());
                        let data_gep = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                            data_gep,
                            vec_struct_ty,
                            vec_struct_ty,
                            val
                        );
                        let data_i64 = self.next_temp(counter);
                        write_ir!(ir, "  {} = load i64, i64* {}", data_i64, data_gep);
                        self.fn_ctx.record_emitted_type(&data_i64, "i64");
                        let data_i8 = self.next_temp(counter);
                        write_ir!(ir, "  {} = inttoptr i64 {} to i8*", data_i8, data_i64);
                        let len_gep = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr {}, {}* {}, i32 0, i32 1",
                            len_gep,
                            vec_struct_ty,
                            vec_struct_ty,
                            val
                        );
                        let len_val = self.next_temp(counter);
                        write_ir!(ir, "  {} = load i64, i64* {}", len_val, len_gep);
                        self.fn_ctx.record_emitted_type(&len_val, "i64");
                        let fat1 = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
                            fat1,
                            data_i8
                        );
                        self.fn_ctx.record_emitted_type(&fat1, "{ i8*, i64 }");
                        let fat2 = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1",
                            fat2,
                            fat1,
                            len_val
                        );
                        self.fn_ctx.record_emitted_type(&fat2, "{ i8*, i64 }");
                        val = fat2;
                    }
                    did_vec_to_slice = true;
                    "{ i8*, i64 }".to_string()
                } else {
                    self.type_to_llvm(pt)
                }
            } else {
                self.type_to_llvm(&inferred_ty)
            };

            // Skip downstream coercions when Vec→slice has produced final fat ptr.
            if did_vec_to_slice {
                arg_vals.push(format!("{} {}", arg_llvm_ty, val));
                continue;
            }

            // Integer width coercion: coerce to match arg_llvm_ty.
            // Phase 17.H4.4: prefer the **actual** LLVM type of `val`
            // (i.e., what the previous instruction emitted) over the
            // inferred type. Codegen's generic-erasure inference often
            // returns I64 for values that were actually loaded as i8/i16
            // (e.g., `bytes[i]` from a Vec<u8>), and using I64 as src
            // bits made the coerce a no-op even though LLVM saw i8.
            {
                let src_bits_from_val = {
                    let actual = self.llvm_type_of(&val);
                    if let Some(rest) = actual.strip_prefix('i') {
                        rest.parse::<u32>().unwrap_or(0)
                    } else {
                        0
                    }
                };
                let src_bits = if src_bits_from_val > 0 {
                    src_bits_from_val
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
                } else if (dst_is_float || dst_is_double)
                    && matches!(
                        inferred_ty,
                        ResolvedType::I8
                            | ResolvedType::I16
                            | ResolvedType::I32
                            | ResolvedType::I64
                            | ResolvedType::U8
                            | ResolvedType::U16
                            | ResolvedType::U32
                            | ResolvedType::U64
                    )
                {
                    // Integer value passed where a float param is expected
                    // (e.g., bare integer literal `2` into `Vec<f32>::push`).
                    let src_bits = self.get_integer_bits(&inferred_ty);
                    let src_ty = if src_bits > 0 {
                        format!("i{}", src_bits)
                    } else {
                        "i64".to_string()
                    };
                    let conv_tmp = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = sitofp {} {} to {}",
                        conv_tmp,
                        src_ty,
                        val,
                        arg_llvm_ty
                    );
                    val = conv_tmp;
                }
            }

            // For struct types: handle generic erasure (struct → i64) and
            // pointer-to-value loading for non-generic struct params.
            let arg_inferred = self.infer_expr_type(arg);
            if matches!(&arg_inferred, ResolvedType::Named { .. }) {
                let struct_llvm = self.type_to_llvm(&arg_inferred);
                if arg_llvm_ty == "i64" && struct_llvm.starts_with('%') {
                    // Generic param (T→i64): check if a specialized version exists
                    let skip_erasure = {
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
                            std::slice::from_ref(&arg_inferred),
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
                    };
                    if !skip_erasure {
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
                    }
                } else {
                    // Non-generic struct param: load only when the generated
                    // argument is a pointer. Shape-based value inference can be
                    // stale for method calls such as `params.clone()`, where the
                    // receiver expression is not syntactically a value but codegen
                    // has already materialized a by-value struct.
                    let actual_val_ty = self.llvm_type_of_checked(&val);
                    let expected_ptr_ty = format!("{}*", arg_llvm_ty);
                    let known_named_pointer_local = actual_val_ty.is_none()
                        && self.fn_ctx.locals.values().any(|local| {
                            let local_llvm = if local.llvm_name.starts_with('%') {
                                local.llvm_name.clone()
                            } else {
                                format!("%{}", local.llvm_name)
                            };
                            local_llvm == val
                                && matches!(local.ty, ResolvedType::Named { .. })
                                && (local.is_ssa() || local.is_alloca() || local.is_param())
                        });
                    let actual_is_named_ptr = actual_val_ty
                        .as_deref()
                        .is_some_and(|ty| ty.ends_with('*') && arg_llvm_ty.starts_with('%'));
                    let needs_load = actual_val_ty.as_deref() == Some(expected_ptr_ty.as_str())
                        || actual_val_ty.as_deref() == Some("ptr")
                        || actual_is_named_ptr
                        || known_named_pointer_local
                        || (actual_val_ty.is_none() && !self.is_expr_value(arg));
                    if needs_load {
                        let load_ptr = if let Some(actual_ptr_ty) = actual_val_ty.as_deref() {
                            if actual_ptr_ty != expected_ptr_ty && actual_ptr_ty.ends_with('*') {
                                let casted = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = bitcast {} {} to {}",
                                    casted,
                                    actual_ptr_ty,
                                    val,
                                    expected_ptr_ty
                                );
                                self.fn_ctx.record_emitted_type(&casted, &expected_ptr_ty);
                                casted
                            } else {
                                val.clone()
                            }
                        } else {
                            val.clone()
                        };
                        let loaded = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = load {}, {}* {}",
                            loaded,
                            arg_llvm_ty,
                            arg_llvm_ty,
                            load_ptr
                        );
                        self.fn_ctx.record_emitted_type(&loaded, &arg_llvm_ty);
                        val = loaded;
                    }
                }
            }

            // Coerce struct pointer → i64 when arg_ty is i64 but value is a Named type
            if arg_llvm_ty == "i64" {
                match self.llvm_type_of_checked(&val).as_deref() {
                    Some("i64*") => {
                        let loaded = self.next_temp(counter);
                        write_ir!(ir, "  {} = load i64, i64* {}", loaded, val);
                        self.fn_ctx.record_emitted_type(&loaded, "i64");
                        val = loaded;
                    }
                    Some("ptr")
                        if matches!(
                            inferred_ty,
                            ResolvedType::Ref(_) | ResolvedType::RefMut(_)
                        ) =>
                    {
                        let loaded = self.next_temp(counter);
                        write_ir!(ir, "  {} = load i64, ptr {}", loaded, val);
                        self.fn_ctx.record_emitted_type(&loaded, "i64");
                        val = loaded;
                    }
                    _ => {}
                }
                // Same scalar-shape guard as `generate_expr_call.rs` —
                // if the AST is a Binary/Unary/literal/Cast, the LLVM
                // result is scalar by construction. A cross-module span
                // collision can otherwise make `infer_expr_type` return
                // `Named { Vec<u8> }` for an `i64 add`, producing
                // invalid `ptrtoint %Vec$u8* %t to i64` IR (vaisdb
                // bytebuffer.vais hit this in `__store_byte(data + off + 1, …)`).
                let scalar_shape = matches!(
                    &arg.node,
                    Expr::Binary { .. }
                        | Expr::Unary { .. }
                        | Expr::Int(_)
                        | Expr::Float(_)
                        | Expr::Bool(_)
                        | Expr::Cast { .. }
                );
                if !scalar_shape {
                    let inferred = self.infer_expr_type(arg);
                    if matches!(inferred, ResolvedType::Named { .. }) {
                        let struct_llvm = self.type_to_llvm(&inferred);
                        let tmp = self.next_temp(counter);
                        write_ir!(ir, "  {} = ptrtoint {}* {} to i64", tmp, struct_llvm, val);
                        self.fn_ctx.record_emitted_type(&tmp, "i64");
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
                    // Value is i64 (generic erasure) but param expects struct type
                    // Coerce via inttoptr + load: i64 → struct_ptr → struct
                    // Guard: if val was emitted as a narrow integer (i1/i8/i16/i32),
                    // zext to i64 first — `inttoptr` requires the source to be i64
                    // here. Without this, vaisdb prefix.vais hits
                    //   `%t34 = inttoptr i64 %t33 to %CompressedKey*`
                    // where %t33 is `load i8` (Vec<CompressedKey>::push(u8) misuse).
                    let actual = self.llvm_type_of(&val);
                    if matches!(actual.as_str(), "i1" | "i8" | "i16" | "i32") {
                        let widened = self.next_temp(counter);
                        write_ir!(ir, "  {} = zext {} {} to i64", widened, actual, val);
                        self.fn_ctx.record_emitted_type(&widened, "i64");
                        val = widened;
                    }
                    let ptr_tmp = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = inttoptr i64 {} to {}*",
                        ptr_tmp,
                        val,
                        arg_llvm_ty
                    );
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

            // Phase E: skip Unit args (see generate_expr_call.rs).
            if arg_llvm_ty == "void" {
                continue;
            }

            // ADR 0001 §1 (Mini Pillar 1 — call-arg invariant):
            //   "When param expects { i8*, i64 } (slice fat-ptr) and val's
            //    LLVM type is %Vec* / %Vec$T*, emit Vec→fat-ptr coercion."
            //
            // This catches cases where inferred_ty was erased (Generic) or
            // mistakenly inferred as Ref(scalar) (e.g. vaisdb node.ll:1848:
            // `key_refs.push(&keys_owned[i])` where keys_owned[i] is a
            // generic-erased Vec<u8> indexed as i64). The earlier
            // is_vec_to_slice_coercion path uses inferred_ty which doesn't
            // see post-erasure LLVM truth, so we add a structural guard here.
            //
            // Tracker: vaisdb Task #10 (test_btree_node.ll:1848).
            // Tests: ret_invariant_test covers ret class; call-arg class
            //        will get its own invariant test once stable.
            if !did_vec_to_slice && arg_llvm_ty == "{ i8*, i64 }" && val.starts_with('%') {
                let val_actual = self.llvm_type_of(&val);
                if val_actual.starts_with("%Vec") && val_actual.ends_with('*') {
                    let vec_struct_ty = val_actual.trim_end_matches('*').to_string();
                    let data_gep = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr {}, {} {}, i32 0, i32 0",
                        data_gep,
                        vec_struct_ty,
                        val_actual,
                        val
                    );
                    let data_i64 = self.next_temp(counter);
                    write_ir!(ir, "  {} = load i64, i64* {}", data_i64, data_gep);
                    self.fn_ctx.record_emitted_type(&data_i64, "i64");
                    let data_i8 = self.next_temp(counter);
                    write_ir!(ir, "  {} = inttoptr i64 {} to i8*", data_i8, data_i64);
                    let len_gep = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr {}, {} {}, i32 0, i32 1",
                        len_gep,
                        vec_struct_ty,
                        val_actual,
                        val
                    );
                    let len_val = self.next_temp(counter);
                    write_ir!(ir, "  {} = load i64, i64* {}", len_val, len_gep);
                    self.fn_ctx.record_emitted_type(&len_val, "i64");
                    let fat1 = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
                        fat1,
                        data_i8
                    );
                    self.fn_ctx.record_emitted_type(&fat1, "{ i8*, i64 }");
                    let fat2 = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1",
                        fat2,
                        fat1,
                        len_val
                    );
                    self.fn_ctx.record_emitted_type(&fat2, "{ i8*, i64 }");
                    val = fat2;
                }
            }
            if arg_llvm_ty == "{ i8*, i64 }" {
                match self.llvm_type_of_checked(&val).as_deref() {
                    Some("{ i8*, i64 }*") => {
                        let loaded = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = load {{ i8*, i64 }}, {{ i8*, i64 }}* {}",
                            loaded,
                            val
                        );
                        self.fn_ctx.record_emitted_type(&loaded, "{ i8*, i64 }");
                        val = loaded;
                    }
                    Some("ptr") => {
                        let loaded = self.next_temp(counter);
                        write_ir!(ir, "  {} = load {{ i8*, i64 }}, ptr {}", loaded, val);
                        self.fn_ctx.record_emitted_type(&loaded, "{ i8*, i64 }");
                        val = loaded;
                    }
                    _ => {}
                }
            }

            if let Some(ResolvedType::Ref(_) | ResolvedType::RefMut(_)) = param_ty.as_ref() {
                if let Some(pointee_ty) = arg_llvm_ty.strip_suffix('*') {
                    if arg_llvm_ty != "ptr" {
                        let actual_ty = self.llvm_type_of_checked(&val);
                        if actual_ty
                            .as_deref()
                            .is_some_and(|actual| actual != "ptr" && !actual.ends_with('*'))
                        {
                            let ref_tmp = self.next_temp(counter);
                            self.emit_entry_alloca(&ref_tmp, pointee_ty);
                            write_ir!(
                                ir,
                                "  store {} {}, {}* {}",
                                pointee_ty,
                                val,
                                pointee_ty,
                                ref_tmp
                            );
                            self.fn_ctx.record_emitted_type(&ref_tmp, &arg_llvm_ty);
                            val = ref_tmp;
                        }
                    }
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
                // Phase 17.H4 iter 20: TC expr_types fallback. When no
                // FunctionInfo / resolved_function_sigs / trait impl is
                // registered, consult the call expression's span in TC's
                // expr_types map. For vaisdb `buf.to_vec()`, `x.to_bytes()`,
                // etc. where the method isn't in any registry, TC often
                // stamped the call with the correct ResolvedType from
                // surrounding context (`Vec<u64>`, `Vec<u8>`). Emit that
                // instead of the `i64` default so downstream `store
                // %Vec$T %t` typechecks at clang link.
                let tc_fallback = call_span
                    .and_then(|s| {
                        let file_id = if s.file_id != 0 {
                            s.file_id
                        } else {
                            self.current_file_id
                        };
                        self.expr_types
                            .get(&(file_id, s.start, s.end))
                            .cloned()
                            .or_else(|| {
                                let mut iter = self
                                    .expr_types
                                    .iter()
                                    .filter(|((_, ss, ee), _)| *ss == s.start && *ee == s.end);
                                let first = iter.next();
                                let second = iter.next();
                                match (first, second) {
                                    (Some((_, ty)), None) => Some(ty.clone()),
                                    _ => None,
                                }
                            })
                    })
                    .filter(|ty| !matches!(ty, ResolvedType::Var(_)));
                if let Some(ty) = tc_fallback {
                    (self.type_to_llvm(&ty), ty)
                } else {
                    ("i64".to_string(), ResolvedType::I64)
                }
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
                        .map(|_| {
                            self.compute_sizeof(&ResolvedType::Named {
                                name: type_suffix.to_string(),
                                generics: vec![],
                            })
                        })
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
                        self.fn_ctx.record_emitted_type(&old_es, "i64");
                        write_ir!(ir, "  store i64 {}, i64* {}", elem_size, es_ptr);
                        let needs_adjust = self.next_temp(counter);
                        write_ir!(ir, "  {} = icmp eq i64 {}, 8", needs_adjust, old_es);
                        self.fn_ctx.record_emitted_type(&needs_adjust, "i1");
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
                        self.fn_ctx.record_emitted_type(&old_cap, "i64");
                        let bytes = self.next_temp(counter);
                        write_ir!(ir, "  {} = mul i64 {}, 8", bytes, old_cap);
                        let new_cap = self.next_temp(counter);
                        write_ir!(ir, "  {} = sdiv i64 {}, {}", new_cap, bytes, elem_size);
                        write_ir!(ir, "  store i64 {}, i64* {}", new_cap, cap_ptr);
                        write_ir!(ir, "  br label %{}", lbl_done);
                        write_ir!(ir, "{}:", lbl_done);
                        // Subsequent IR emission belongs to lbl_done — keep
                        // current_block accurate so enclosing phi nodes read
                        // the correct predecessor.
                        self.fn_ctx.current_block = lbl_done;
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
                    self.fn_ctx.record_emitted_type(&cur_len, "i64");
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

        // F-23 dispatch (text-IR side, A4-12 step 2a-C-3 — DEFERRED #19):
        // if the receiver was originally `dyn Trait` / `&dyn Trait` /
        // `&mut dyn Trait`, dispatch through the vtable instead of the
        // static @<MangledName> emit below.
        //
        // This is now safe (DEFERRED #19 2a-C-1 fix established):
        //   - sorted_method_names ensures emission/dispatch slot order
        //     agreement across vtable_struct_type / generate_vtable /
        //     generate_dyn_method_call.
        //   - typed ABI (2a-A) ensures Result/Option/struct returns
        //     lower precisely (no i64 widening).
        // Together these remove the regression risks that revert-ed
        // earlier wiring attempts.
        //
        // recv_is_dyn_trait was decided from var_resolved_types — the
        // type checker's source-level view. The lowered LLVM type for
        // `dyn Trait` / `&dyn Trait` / `&mut dyn Trait` is the fat
        // pointer { i8*, i8* } per type_to_llvm. Trust the type-checker
        // view; an LLVM-shape guard would miss cases where the
        // recv_val was loaded directly from a function-parameter
        // alloca without going through record_emitted_type (verified
        // empirically on LIVING_SPEC dyn_trait_param.vais).
        if recv_is_dyn_trait {
            let trait_name_opt = match &recv_type {
                ResolvedType::DynTrait { trait_name, .. } => Some(trait_name.clone()),
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                    if let ResolvedType::DynTrait { trait_name, .. } = inner.as_ref() {
                        Some(trait_name.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(trait_name) = trait_name_opt {
                // Re-evaluate receiver to recover the original fat
                // pointer (recv_val above may have been bitcast to a
                // typed pointer earlier in this function).
                let (fat_val, fat_ir) = self.generate_expr(receiver, counter)?;
                ir.push_str(&fat_ir);

                // arg_vals[0] is recv (formatted "<ty> <val>"); skip
                // it. Remaining values are also "<ty> <val>"; strip the
                // type prefix. trait_dispatch::generate_dyn_method_call
                // re-derives types from method_sig (typed ABI 2a-A).
                let dyn_args: Vec<String> = arg_vals
                    .iter()
                    .skip(1)
                    .map(|av| {
                        av.split_once(' ')
                            .map(|(_, v)| v.to_string())
                            .unwrap_or_else(|| av.clone())
                    })
                    .collect();

                let (call_ir, call_val) = self.generate_dyn_method_call(
                    &fat_val,
                    &trait_name,
                    method_name,
                    &dyn_args,
                    counter,
                )?;
                ir.push_str(&call_ir);

                if call_val.is_empty() {
                    return Ok(("void".to_string(), ir));
                }
                self.fn_ctx.register_temp_type(&call_val, ret_resolved);
                return Ok((call_val, ir));
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
        call_span: Option<vais_ast::Span>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Phase 16 A2.5: when we know the overall call expression's span, read
        // the TC-resolved return type so generic parameters that only appear
        // in the return type can be inferred via resolve_generic_call_with_hint.
        let expected_ret = call_span.and_then(|s| {
            let file_id = if s.file_id != 0 {
                s.file_id
            } else {
                self.current_file_id
            };
            if let Some(ty) = self.expr_types.get(&(file_id, s.start, s.end)).cloned() {
                return Some(ty);
            }
            // Phase 17.H1 fallback: serial TC path uses a single file_id.
            // Match (start, end) only if unique.
            let mut iter = self
                .expr_types
                .iter()
                .filter(|((_, st, en), _)| *st == s.start && *en == s.end);
            let first = iter.next();
            let second = iter.next();
            match (first, second) {
                (Some((_, ty)), None) => Some(ty.clone()),
                _ => None,
            }
        });
        // Phase 0 bug C4 fix: when this call is inside a specialized generic
        // function body (e.g. inside `Mutex_lock$i64`), TC stamped the call's
        // return type with the original generic params (e.g. `MutexGuard<T>`).
        // Substitute those params using the current spec context so the
        // mangled-name lookup below picks the right specialization
        // (`MutexGuard_new$i64` instead of bare `MutexGuard_new`).
        let expected_ret = expected_ret.map(|ty| {
            if self.generics.substitutions.is_empty() {
                ty
            } else {
                vais_types::substitute_type(&ty, &self.generics.substitutions)
            }
        });
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

        let has_user_box_new = self.types.functions.contains_key("Box_new")
            || self.generics.fn_instantiations.contains_key("Box_new")
            || self
                .generics
                .generic_method_bodies
                .contains_key(&("Box".to_string(), "new".to_string()));
        if type_name.node == "Box" && method.node == "new" && args.len() == 1 && !has_user_box_new {
            let mut ir = String::new();
            let (val, arg_ir) = self.generate_expr(&args[0], counter)?;
            ir.push_str(&arg_ir);

            let value_ty = self.infer_expr_type(&args[0]);
            let value_llvm = self.type_to_llvm(&value_ty);
            let value_size = self.compute_sizeof(&value_ty).max(1);
            let heap_ptr = self.next_temp(counter);
            write_ir!(ir, "  {} = call i8* @malloc(i64 {})", heap_ptr, value_size);
            self.fn_ctx.record_emitted_type(&heap_ptr, "i8*");

            if value_llvm != "void" {
                let typed_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = bitcast i8* {} to {}*",
                    typed_ptr,
                    heap_ptr,
                    value_llvm
                );
                self.fn_ctx
                    .record_emitted_type(&typed_ptr, &format!("{}*", value_llvm));

                let actual = self
                    .llvm_type_of_checked(&val)
                    .unwrap_or_else(|| self.llvm_type_of(&val));
                let store_val = if actual == format!("{}*", value_llvm)
                    || actual == "ptr"
                    || actual.ends_with('*')
                {
                    let src_ptr = if actual == format!("{}*", value_llvm) {
                        val.clone()
                    } else {
                        let cast_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = bitcast {} {} to {}*",
                            cast_ptr,
                            actual,
                            val,
                            value_llvm
                        );
                        self.fn_ctx
                            .record_emitted_type(&cast_ptr, &format!("{}*", value_llvm));
                        cast_ptr
                    };
                    let loaded = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}",
                        loaded,
                        value_llvm,
                        value_llvm,
                        src_ptr
                    );
                    self.fn_ctx.record_emitted_type(&loaded, &value_llvm);
                    loaded
                } else {
                    self.coerce_int_width(&val, &actual, &value_llvm, counter, &mut ir)
                };
                write_ir!(
                    ir,
                    "  store {} {}, {}* {}",
                    value_llvm,
                    store_val,
                    value_llvm,
                    typed_ptr
                );
            }

            let ptr_i64 = self.next_temp(counter);
            write_ir!(ir, "  {} = ptrtoint i8* {} to i64", ptr_i64, heap_ptr);
            self.fn_ctx.record_emitted_type(&ptr_i64, "i64");
            self.fn_ctx.register_temp_type(
                &ptr_i64,
                ResolvedType::Named {
                    name: "Box".to_string(),
                    generics: vec![value_ty],
                },
            );
            return Ok((ptr_i64, ir));
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
        // Phase 17.H4.15: when expected_ret has concrete generic args and
        // zero-arg call (e.g. Vec.new() with a known target Vec<T>), only
        // accept a specialization whose type args match EXACTLY. Otherwise
        // fall through so branch C can generate on-demand. Without this,
        // resolve_generic_call_with_hint's "last resort: first instantiation"
        // fallback picks an unrelated specialization (e.g. Vec_new$MigrationRecord
        // when the target was actually Vec<i64>).
        let expected_has_concrete_generics = matches!(
            &expected_ret,
            Some(ResolvedType::Named { generics, .. }) if !generics.is_empty()
                && generics.iter().all(|g| !matches!(g,
                    ResolvedType::Generic(_) | ResolvedType::Var(_)))
        );
        let expect_zero_arg_container = args.is_empty() && expected_has_concrete_generics;
        let expected_inst_types: Option<Vec<ResolvedType>> = if expect_zero_arg_container {
            match &expected_ret {
                Some(ResolvedType::Named { generics, .. }) => Some(generics.clone()),
                _ => None,
            }
        } else {
            None
        };

        // Zero-arg + concrete expected: if the exact match is NOT present in
        // fn_instantiations, skip branches A+B so branch C can try to
        // generate the specialization on demand.
        let skip_ab_for_expected = if let Some(expected_types) = &expected_inst_types {
            let inst_list_opt = self.generics.fn_instantiations.get(&base_method_name);
            match inst_list_opt {
                Some(inst_list) => !inst_list.iter().any(|(types, _)| types == expected_types),
                None => true,
            }
        } else {
            false
        };

        let full_method_name = if !skip_ab_for_expected
            && self
                .generics
                .fn_instantiations
                .contains_key(&base_method_name)
        {
            let inst_list = self
                .generics
                .fn_instantiations
                .get(&base_method_name)
                .cloned()
                .expect("invariant: fn_instantiations contains_key checked immediately before get");
            let arg_types: Vec<ResolvedType> =
                args.iter().map(|a| self.infer_expr_type(a)).collect();
            let resolved = self.resolve_generic_call_with_hint(
                &base_method_name,
                &arg_types,
                &inst_list,
                expected_ret.as_ref(),
            );
            if self.types.functions.contains_key(&resolved) {
                resolved
            } else {
                base_method_name.clone()
            }
        } else if !skip_ab_for_expected
            && self.types.functions.contains_key(&base_method_name)
            && !(base_method_name == "Vec_new" && args.is_empty()
                && matches!(&expected_ret, Some(ResolvedType::Named { generics, .. }) if !generics.is_empty()))
            // Phase 0 bug C4 fix (part 2): when this call is inside a
            // specialized generic body (substitutions non-empty), don't take
            // the bare-base-name shortcut if the type carries generics — try
            // to specialize on demand using either expected_ret or struct's
            // type params. Without this skip, calls like `MutexGuard::new(&self)`
            // inside `Mutex_lock$i64` resolve to bare `MutexGuard_new` (which
            // returns the unspecialized `%MutexGuard`) and fail link with a
            // type mismatch against the specialized return slot.
            && !(self.generics.struct_defs.contains_key(&type_name.node)
                 && !self.generics.substitutions.is_empty()
                 && matches!(
                     &expected_ret,
                     Some(ResolvedType::Named { generics, .. }) if !generics.is_empty()
                         && generics.iter().all(|g| !matches!(g,
                             ResolvedType::Generic(_) | ResolvedType::Var(_)))
                 ))
        {
            // Already found directly — use as-is.
            // Phase 17.H4.11: but skip this shortcut for `Vec.new()` when
            // we know T from expected_ret — prefer specialization so the
            // return type matches struct field layout. Without this override
            // the base generic (signature returns i64) wins and subsequent
            // store-into-struct-field fails.
            base_method_name.clone()
        } else if self.generics.struct_defs.contains_key(&type_name.node)
            && args.is_empty()
            && matches!(&expected_ret, Some(ResolvedType::Named { generics, .. }) if !generics.is_empty()
                && generics.iter().all(|g| !matches!(g,
                    ResolvedType::Generic(_) | ResolvedType::Var(_))))
        {
            // Phase 17.H4.11 (+ H4.15): zero-arg generic static methods (Vec.new(),
            // HashMap.new(), Option.None()) cannot infer T from args. Use
            // the TC-resolved expected return type's generic args as the
            // concrete T vector. H4.15: drop the I64 exclusion — I64 is a
            // valid concrete element type (e.g., Vec<i64>), and the new
            // struct-literal / expected-type hint path in TC now provides
            // a trustworthy concrete hint rather than the legacy "I64 means
            // unresolved fallback" sentinel.
            let type_args: Vec<ResolvedType> = match &expected_ret {
                Some(ResolvedType::Named { generics, .. }) => generics.clone(),
                _ => vec![],
            };
            let mangled = vais_types::mangle_name(&base_method_name, &type_args);
            if self.types.functions.contains_key(&mangled) {
                mangled
            } else if let Some(spec) = self.try_generate_vec_specialization(
                &type_name.node,
                &method.node,
                &type_args,
                counter,
            ) {
                spec
            } else {
                base_method_name.clone()
            }
        } else if self.generics.struct_defs.contains_key(&type_name.node) && !args.is_empty() {
            // Phase 0 bug C4 fix (part 3): when expected_ret carries concrete
            // generics (e.g. `MutexGuard<i64>`), trust those over arg-type
            // inference. Argument types like `&Mutex<i64>` for `MutexGuard.new`
            // would otherwise mangle to `MutexGuard_new$ref_Mutex_i64` which
            // doesn't match the spec key the rest of codegen expects.
            let expected_concrete: Option<Vec<ResolvedType>> = match &expected_ret {
                Some(ResolvedType::Named { generics, .. })
                    if !generics.is_empty()
                        && generics.iter().all(|g| {
                            !matches!(g, ResolvedType::Generic(_) | ResolvedType::Var(_))
                        }) =>
                {
                    Some(generics.clone())
                }
                _ => None,
            };

            // Infer type args from arguments for generic struct static methods
            let arg_types: Vec<ResolvedType> =
                args.iter().map(|a| self.infer_expr_type(a)).collect();
            let capacity_only_ctor = matches!(
                (type_name.node.as_str(), method.node.as_str()),
                ("Vec", "new")
                    | ("Vec", "with_capacity")
                    | ("HashMap", "new")
                    | ("HashMap", "with_capacity")
                    | ("HashSet", "new")
                    | ("HashSet", "with_capacity")
            );
            let informative_args: Vec<&ResolvedType> = if capacity_only_ctor {
                Vec::new()
            } else {
                arg_types
                    .iter()
                    .filter(|t| {
                        !matches!(
                            t,
                            ResolvedType::I64 | ResolvedType::Generic(_) | ResolvedType::Var(_)
                        )
                    })
                    .collect()
            };
            if let Some(concrete_args) = expected_concrete {
                let mangled = vais_types::mangle_name(&base_method_name, &concrete_args);
                if self.types.functions.contains_key(&mangled) {
                    mangled
                } else if let Some(spec) = self.try_generate_vec_specialization(
                    &type_name.node,
                    &method.node,
                    &concrete_args,
                    counter,
                ) {
                    spec
                } else {
                    base_method_name.clone()
                }
            } else if !informative_args.is_empty() {
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
            let mut did_vec_to_slice = false;
            let arg_llvm_ty = if let Some(ref pt) = param_ty {
                if matches!(pt, ResolvedType::Generic(_)) {
                    self.type_to_llvm(&inferred_ty)
                } else if Self::is_vec_to_slice_coercion(pt, &inferred_ty) {
                    // Vec<T> → Slice(T) coercion at static method call boundary.
                    // Materialize a real `{ i8*, i64 }` fat pointer from the Vec
                    // by extracting data (field 0) and len (field 1).
                    let vec_ptr_ty = self.llvm_type_of(&val);
                    let vec_struct_ty = vec_ptr_ty
                        .strip_suffix('*')
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "%Vec".to_string());
                    let data_gep = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                        data_gep,
                        vec_struct_ty,
                        vec_struct_ty,
                        val
                    );
                    let data_i64 = self.next_temp(counter);
                    write_ir!(ir, "  {} = load i64, i64* {}", data_i64, data_gep);
                    self.fn_ctx.record_emitted_type(&data_i64, "i64");
                    let data_i8 = self.next_temp(counter);
                    write_ir!(ir, "  {} = inttoptr i64 {} to i8*", data_i8, data_i64);
                    let len_gep = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr {}, {}* {}, i32 0, i32 1",
                        len_gep,
                        vec_struct_ty,
                        vec_struct_ty,
                        val
                    );
                    let len_val = self.next_temp(counter);
                    write_ir!(ir, "  {} = load i64, i64* {}", len_val, len_gep);
                    self.fn_ctx.record_emitted_type(&len_val, "i64");
                    let fat1 = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
                        fat1,
                        data_i8
                    );
                    self.fn_ctx.record_emitted_type(&fat1, "{ i8*, i64 }");
                    let fat2 = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1",
                        fat2,
                        fat1,
                        len_val
                    );
                    self.fn_ctx.record_emitted_type(&fat2, "{ i8*, i64 }");
                    val = fat2;
                    did_vec_to_slice = true;
                    "{ i8*, i64 }".to_string()
                } else {
                    self.type_to_llvm(pt)
                }
            } else {
                self.type_to_llvm(&inferred_ty)
            };

            if did_vec_to_slice {
                arg_vals.push(format!("{} {}", arg_llvm_ty, val));
                continue;
            }

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
                Some(ResolvedType::Generic(_)) => inferred_ty.clone(),
                Some(ty) => ty.clone(),
                None => inferred_ty.clone(),
            };
            if matches!(type_to_check, ResolvedType::Named { .. }) {
                let actual_val_ty = self.llvm_type_of_checked(&val);
                let expected_ptr_ty = format!("{}*", arg_llvm_ty);
                let known_named_pointer_local = actual_val_ty.is_none()
                    && self.fn_ctx.locals.values().any(|local| {
                        let local_llvm = if local.llvm_name.starts_with('%') {
                            local.llvm_name.clone()
                        } else {
                            format!("%{}", local.llvm_name)
                        };
                        local_llvm == val
                            && matches!(local.ty, ResolvedType::Named { .. })
                            && (local.is_ssa() || local.is_alloca() || local.is_param())
                    });
                let actual_is_named_ptr = actual_val_ty
                    .as_deref()
                    .is_some_and(|ty| ty.ends_with('*') && arg_llvm_ty.starts_with('%'));
                let needs_load = actual_val_ty.as_deref() == Some(expected_ptr_ty.as_str())
                    || actual_val_ty.as_deref() == Some("ptr")
                    || actual_is_named_ptr
                    || known_named_pointer_local
                    || (actual_val_ty.is_none() && !self.is_expr_value(arg));
                if needs_load {
                    let load_ptr = if let Some(actual_ptr_ty) = actual_val_ty.as_deref() {
                        if actual_ptr_ty != expected_ptr_ty && actual_ptr_ty.ends_with('*') {
                            let casted = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = bitcast {} {} to {}",
                                casted,
                                actual_ptr_ty,
                                val,
                                expected_ptr_ty
                            );
                            self.fn_ctx.record_emitted_type(&casted, &expected_ptr_ty);
                            casted
                        } else {
                            val.clone()
                        }
                    } else {
                        val.clone()
                    };
                    let loaded = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}",
                        loaded,
                        arg_llvm_ty,
                        arg_llvm_ty,
                        load_ptr
                    );
                    self.fn_ctx.record_emitted_type(&loaded, &arg_llvm_ty);
                    val = loaded;
                }
            }
            if arg_llvm_ty == "{ i8*, i64 }" {
                match self.llvm_type_of_checked(&val).as_deref() {
                    Some("{ i8*, i64 }*") => {
                        let loaded = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = load {{ i8*, i64 }}, {{ i8*, i64 }}* {}",
                            loaded,
                            val
                        );
                        self.fn_ctx.record_emitted_type(&loaded, "{ i8*, i64 }");
                        val = loaded;
                    }
                    Some("ptr") => {
                        let loaded = self.next_temp(counter);
                        write_ir!(ir, "  {} = load {{ i8*, i64 }}, ptr {}", loaded, val);
                        self.fn_ctx.record_emitted_type(&loaded, "{ i8*, i64 }");
                        val = loaded;
                    }
                    _ => {}
                }
            }

            // Phase E: skip Unit args (see generate_expr_call.rs).
            if arg_llvm_ty == "void" {
                continue;
            }
            arg_vals.push(format!("{} {}", arg_llvm_ty, val));
        }

        let ret_resolved: Option<ResolvedType> = fn_info
            .as_ref()
            .map(|info| info.signature.ret.clone())
            .or_else(|| {
                self.types
                    .resolved_function_sigs
                    .get(&full_method_name)
                    .map(|sig| sig.ret.clone())
            })
            .or_else(|| {
                // Phase 17.H4 iter 20: TC expr_types fallback when the static
                // method has no registered signature (vaisdb calls like
                // `Str.new()`, `Foo.bar()` where Foo is not a real struct or
                // the method is unregistered). Without this, codegen emits
                // `call i64 @Foo_bar()` then use sites hitting
                // `store %Expected %t, %Expected* %slot` fail at link.
                // `expected_ret` is already resolved from call_span above.
                expected_ret.clone().and_then(|ty| {
                    if matches!(&ty, ResolvedType::Var(_)) {
                        None
                    } else {
                        Some(ty)
                    }
                })
            });
        let ret_type = ret_resolved
            .as_ref()
            .map(|ty| self.type_to_llvm(ty))
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
            // Phase B4: register the temp with the ACTUAL emitted return
            // type (from signature), not the caller-inferred type. This
            // stops the generate_expr catch-all from registering a Named
            // type on an SSA temp whose emission used `i64` fallback.
            if let Some(resolved) = ret_resolved {
                self.fn_ctx.register_temp_type(&tmp, resolved);
            } else {
                self.fn_ctx.register_temp_type(&tmp, ResolvedType::I64);
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
            let saved_actual_llvm_type = std::mem::take(&mut self.fn_ctx.actual_llvm_type);
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
            self.fn_ctx.actual_llvm_type = saved_actual_llvm_type;
            self.fn_ctx.scope_stack = saved_scope_stack;
            self.fn_ctx.scope_str_stack = saved_scope_str_stack;
            self.fn_ctx.scope_drop_label_counter = saved_scope_drop_label_counter;
            self.fn_ctx.entry_allocas = saved_entry_allocas;

            match result {
                Ok(ir_code) => {
                    self.fn_ctx.pending_specialized_ir.push(ir_code);
                    Some(mangled)
                }
                Err(_) => None,
            }
        } else {
            None
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
