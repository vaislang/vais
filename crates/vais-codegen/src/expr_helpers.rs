//! Expression generation helper methods for CodeGenerator
//!
//! This module contains core expression helpers: enum variants,
//! binary/unary operations, and cast operations.
//! Assignment and identifier expression helpers are in expr_helpers_assign.

use crate::{CodeGenerator, CodegenError, CodegenResult};
use vais_ast::{BinOp, Expr, Span, Spanned, Type, UnaryOp};
use vais_types::ResolvedType;

/// Phase E: if `val` looks like a decimal float literal (contains '.' or
/// 'e' or 'E' and starts with a digit / minus), parse it, round to f32,
/// expand back to f64 bit pattern, and emit as 0x-prefixed hex. LLVM
/// requires that a `float` constant's double bit pattern round-trip
/// exactly to f32 — the simplest way to guarantee this is to go
/// f64 → f32 → f64 before emitting.
fn normalize_float_literal_for_float(val: &str) -> String {
    let first = val.chars().next().unwrap_or(' ');
    let is_number_like = first.is_ascii_digit() || first == '-' || first == '+';
    let looks_like_float = is_number_like
        && (val.contains('.') || val.contains('e') || val.contains('E'))
        && !val.starts_with("0x");
    if !looks_like_float {
        return val.to_string();
    }
    match val.parse::<f64>() {
        Ok(n) if n.is_finite() => {
            // Round to f32 first, then expand back to f64 to get a bit
            // pattern LLVM will accept for `float` context.
            let as_f32 = n as f32;
            let round_tripped = as_f32 as f64;
            let bits = round_tripped.to_bits();
            format!("0x{:016X}", bits)
        }
        _ => val.to_string(),
    }
}

impl CodeGenerator {
    #[inline(never)]
    pub(crate) fn generate_unit_enum_variant(
        &mut self,
        name: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Clone enum info to avoid borrow conflict with self.next_temp/emit_entry_alloca
        let mut found = None;
        for enum_info in self.types.enums.values() {
            for (tag, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == name {
                    found = Some((enum_info.name.clone(), tag));
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        if let Some((enum_name, tag)) = found {
            let mut ir = String::new();
            let enum_ptr = self.next_temp(counter);
            self.emit_entry_alloca(&enum_ptr, &format!("%{}", enum_name));
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
            return Ok((enum_ptr, ir));
        }
        // Fallback if not found (shouldn't happen)
        Ok((format!("@{}", name), String::new()))
    }

    /// Generate binary expression
    #[inline(never)]
    pub(crate) fn generate_binary_expr(
        &mut self,
        op: &BinOp,
        left: &Spanned<Expr>,
        right: &Spanned<Expr>,
        counter: &mut usize,
        span: Span,
    ) -> CodegenResult<(String, String)> {
        let (left_val, left_ir) = self.generate_expr(left, counter)?;
        let (right_val, right_ir) = self.generate_expr(right, counter)?;

        let mut ir = left_ir;
        ir.push_str(&right_ir);

        // Handle string operations. Unwrap Ref/RefMut on the LHS because
        // `&str` and `str` share the same fat-pointer LLVM layout and the
        // string helpers (concat, strcmp, str_contains, ...) operate on
        // the data pointer either way. Without this, comparisons like
        // `s == "..."` where `s: &str` fall through to i64 compare.
        let left_type_raw = self.infer_expr_type(left);
        let left_type = match &left_type_raw {
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
                if matches!(inner.as_ref(), ResolvedType::Str) =>
            {
                ResolvedType::Str
            }
            _ => left_type_raw,
        };
        if matches!(left_type, ResolvedType::Str) {
            // `str + int` is pointer-arithmetic, not string concatenation.
            // load_byte(s + i) / similar low-level helpers rely on this form.
            let right_type_for_strop = self.infer_expr_type(right);
            let right_is_int = self.get_integer_bits(&right_type_for_strop) > 0
                && !matches!(right_type_for_strop, ResolvedType::Str | ResolvedType::Bool);
            if matches!(op, BinOp::Add | BinOp::Sub) && right_is_int {
                // Extract raw i8* from left fat pointer, ptrtoint to i64, then add/sub.
                let ptr_tmp = self.next_temp(counter);
                let i64_tmp = self.next_temp(counter);
                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                    ptr_tmp,
                    left_val
                );
                write_ir!(ir, "  {} = ptrtoint i8* {} to i64", i64_tmp, ptr_tmp);
                self.fn_ctx.record_emitted_type(&i64_tmp, "i64");
                let op_str = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    _ => unreachable!(),
                };
                // Right operand may be narrower than i64; widen if so.
                let rbits = self.get_integer_bits(&right_type_for_strop);
                let right_use = if rbits < 64 {
                    let widened = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = sext i{} {} to i64",
                        widened,
                        rbits,
                        right_val
                    );
                    widened
                } else {
                    right_val.clone()
                };
                write_ir!(
                    ir,
                    "  {} = {} i64 {}, {}",
                    result,
                    op_str,
                    i64_tmp,
                    right_use
                );
                return Ok((result, ir));
            }
            return self.generate_string_binary_op(op, &left_val, &right_val, ir, counter);
        }

        // Handle comparison and logical operations (result is i1)
        let is_comparison = matches!(
            op,
            BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte | BinOp::Eq | BinOp::Neq
        );
        let is_logical = matches!(op, BinOp::And | BinOp::Or);

        if is_logical {
            // For logical And/Or, convert operands to i1 first, then perform operation
            // If operand is already i1 (bool), skip the conversion
            let left_bool = if matches!(left_type, ResolvedType::Bool) {
                left_val.clone()
            } else {
                let tmp = self.next_temp(counter);
                let left_llvm = self.type_to_llvm(&left_type);
                write_ir!(ir, "  {} = icmp ne {} {}, 0", tmp, left_llvm, left_val);
                self.fn_ctx.record_emitted_type(&tmp, "i1");
                tmp
            };
            let right_type = self.infer_expr_type(right);
            let right_bool = if matches!(right_type, ResolvedType::Bool) {
                right_val.clone()
            } else {
                let tmp = self.next_temp(counter);
                let right_llvm = self.type_to_llvm(&right_type);
                write_ir!(ir, "  {} = icmp ne {} {}, 0", tmp, right_llvm, right_val);
                self.fn_ctx.record_emitted_type(&tmp, "i1");
                tmp
            };

            let op_str = match op {
                BinOp::And => "and",
                BinOp::Or => "or",
                _ => {
                    return Err(CodegenError::InternalError(format!(
                        "BinOp {:?} in logical codegen path",
                        op
                    )))
                }
            };

            let result_bool = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);
            write_ir!(
                ir,
                "  {} = {} i1 {}, {}{}",
                result_bool,
                op_str,
                left_bool,
                right_bool,
                dbg_info
            );

            // Extend back to i64 for consistency
            let result = self.next_temp(counter);
            write_ir!(ir, "  {} = zext i1 {} to i64", result, result_bool);
            self.fn_ctx.record_emitted_type(&result, "i64");
            Ok((result, ir))
        } else if is_comparison {
            // Comparison returns i1, extend to i64
            let right_type = self.infer_expr_type(right);
            let is_float_cmp = matches!(left_type, ResolvedType::F64 | ResolvedType::F32)
                || matches!(right_type, ResolvedType::F64 | ResolvedType::F32);

            let cmp_tmp = self.next_temp(counter);
            let dbg_info = self.debug_info.dbg_ref_from_span(span);

            if is_float_cmp {
                let op_str = match op {
                    BinOp::Lt => "fcmp olt",
                    BinOp::Lte => "fcmp ole",
                    BinOp::Gt => "fcmp ogt",
                    BinOp::Gte => "fcmp oge",
                    BinOp::Eq => "fcmp oeq",
                    BinOp::Neq => "fcmp one",
                    _ => {
                        return Err(CodegenError::InternalError(format!(
                            "BinOp {:?} in float_cmp codegen path",
                            op
                        )))
                    }
                };
                // Handle mixed f32/f64 comparisons
                let left_is_f32 = matches!(left_type, ResolvedType::F32);
                let right_is_f32 = matches!(right_type, ResolvedType::F32);
                let left_is_f64 = matches!(left_type, ResolvedType::F64);
                let right_is_f64 = matches!(right_type, ResolvedType::F64);

                let float_llvm;
                let mut actual_left = left_val.clone();
                let mut actual_right = right_val.clone();

                if (left_is_f32 && right_is_f64) || (left_is_f64 && right_is_f32) {
                    float_llvm = "double";
                    if left_is_f32 {
                        let ext = self.next_temp(counter);
                        write_ir!(ir, "  {} = fpext float {} to double", ext, left_val);
                        actual_left = ext;
                    }
                    if right_is_f32 {
                        let ext = self.next_temp(counter);
                        write_ir!(ir, "  {} = fpext float {} to double", ext, right_val);
                        actual_right = ext;
                    }
                } else if left_is_f32 || right_is_f32 {
                    float_llvm = "float";
                } else {
                    float_llvm = "double";
                }

                // Phase E: convert decimal float literal constants to
                // IEEE-754 hex form when target is `float`. LLVM rejects
                // decimal forms that don't round-trip exactly through f32.
                if float_llvm == "float" {
                    actual_left = normalize_float_literal_for_float(&actual_left);
                    actual_right = normalize_float_literal_for_float(&actual_right);
                }

                write_ir!(
                    ir,
                    "  {} = {} {} {}, {}{}",
                    cmp_tmp,
                    op_str,
                    float_llvm,
                    actual_left,
                    actual_right,
                    dbg_info
                );
            } else {
                let op_str = match op {
                    BinOp::Lt => "icmp slt",
                    BinOp::Lte => "icmp sle",
                    BinOp::Gt => "icmp sgt",
                    BinOp::Gte => "icmp sge",
                    BinOp::Eq => "icmp eq",
                    BinOp::Neq => "icmp ne",
                    _ => {
                        return Err(CodegenError::InternalError(format!(
                            "BinOp {:?} in int_cmp codegen path",
                            op
                        )))
                    }
                };
                // Use inferred type for integer comparison width
                let cmp_llvm = match &left_type {
                    ResolvedType::I8 | ResolvedType::U8 => "i8",
                    ResolvedType::I16 | ResolvedType::U16 => "i16",
                    ResolvedType::I32 | ResolvedType::U32 => "i32",
                    ResolvedType::I128 | ResolvedType::U128 => "i128",
                    ResolvedType::Bool => "i1",
                    _ => "i64",
                };
                // Phase 17.H4.10: ptrtoint pointer operands first. When
                // registers hold `%T*` (e.g., field-access GEP results),
                // `coerce_int_width` is a no-op and leaves operands as
                // ptr while icmp is emitted as `icmp eq i64 ptr, ptr`,
                // which LLVM rejects. Only trigger when `llvm_type_of`
                // explicitly returns a pointer-ending string — this is
                // reliable after H4.10's GEP-register upgrade.
                let actual_left_ty = self.llvm_type_of(&left_val);
                let actual_right_ty = self.llvm_type_of(&right_val);
                let (left_norm, left_norm_ty) = if actual_left_ty.ends_with('*') {
                    let t = self.next_temp(counter);
                    write_ir!(ir, "  {} = ptrtoint {} {} to i64", t, actual_left_ty, left_val);
                    self.fn_ctx.record_emitted_type(&t, "i64");
                    (t, "i64".to_string())
                } else {
                    (left_val.clone(), actual_left_ty)
                };
                let (right_norm, right_norm_ty) = if actual_right_ty.ends_with('*') {
                    let t = self.next_temp(counter);
                    write_ir!(ir, "  {} = ptrtoint {} {} to i64", t, actual_right_ty, right_val);
                    self.fn_ctx.record_emitted_type(&t, "i64");
                    (t, "i64".to_string())
                } else {
                    (right_val.clone(), actual_right_ty)
                };
                let coerced_left =
                    self.coerce_int_width(&left_norm, &left_norm_ty, cmp_llvm, counter, &mut ir);
                let coerced_right =
                    self.coerce_int_width(&right_norm, &right_norm_ty, cmp_llvm, counter, &mut ir);
                write_ir!(
                    ir,
                    "  {} = {} {} {}, {}{}",
                    cmp_tmp,
                    op_str,
                    cmp_llvm,
                    coerced_left,
                    coerced_right,
                    dbg_info
                );
            }

            // Extend i1 to i64
            let result = self.next_temp(counter);
            write_ir!(ir, "  {} = zext i1 {} to i64", result, cmp_tmp);
            self.fn_ctx.record_emitted_type(&result, "i64");
            Ok((result, ir))
        } else {
            // Arithmetic and bitwise operations
            let tmp = self.next_temp(counter);

            // Check if either operand is a float type
            let right_type = self.infer_expr_type(right);
            let is_float = matches!(left_type, ResolvedType::F64 | ResolvedType::F32)
                || matches!(right_type, ResolvedType::F64 | ResolvedType::F32);

            let dbg_info = self.debug_info.dbg_ref_from_span(span);

            if is_float
                && matches!(
                    op,
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod
                )
            {
                let op_str = match op {
                    BinOp::Add => "fadd",
                    BinOp::Sub => "fsub",
                    BinOp::Mul => "fmul",
                    BinOp::Div => "fdiv",
                    BinOp::Mod => "frem",
                    _ => {
                        return Err(CodegenError::InternalError(format!(
                            "BinOp {:?} in float_arith codegen path",
                            op
                        )))
                    }
                };
                // Determine target float type
                let left_is_f32 = matches!(left_type, ResolvedType::F32);
                let right_is_f32 = matches!(right_type, ResolvedType::F32);
                let left_is_f64 = matches!(left_type, ResolvedType::F64);
                let right_is_f64 = matches!(right_type, ResolvedType::F64);

                let float_llvm;
                let mut actual_left = left_val.clone();
                let mut actual_right = right_val.clone();

                // Check for int operands that need sitofp conversion
                let left_is_int = !left_is_f32 && !left_is_f64;
                let right_is_int = !right_is_f32 && !right_is_f64;

                if (left_is_f32 && right_is_f64) || (left_is_f64 && right_is_f32) {
                    // Mixed f32/f64 — promote f32 to f64
                    float_llvm = "double";
                    if left_is_f32 {
                        let ext = self.next_temp(counter);
                        write_ir!(ir, "  {} = fpext float {} to double", ext, left_val);
                        actual_left = ext;
                    }
                    if right_is_f32 {
                        let ext = self.next_temp(counter);
                        write_ir!(ir, "  {} = fpext float {} to double", ext, right_val);
                        actual_right = ext;
                    }
                } else if left_is_f32 || right_is_f32 {
                    float_llvm = "float";
                } else {
                    float_llvm = "double";
                }

                // Convert int operands to float (sitofp) for mixed int*float arithmetic
                if left_is_int {
                    let conv = self.next_temp(counter);
                    let int_llvm = self.type_to_llvm(&left_type);
                    write_ir!(ir, "  {} = sitofp {} {} to {}", conv, int_llvm, actual_left, float_llvm);
                    actual_left = conv;
                }
                if right_is_int {
                    let conv = self.next_temp(counter);
                    let int_llvm = self.type_to_llvm(&right_type);
                    write_ir!(ir, "  {} = sitofp {} {} to {}", conv, int_llvm, actual_right, float_llvm);
                    actual_right = conv;
                }

                write_ir!(
                    ir,
                    "  {} = {} {} {}, {}{}",
                    tmp,
                    op_str,
                    float_llvm,
                    actual_left,
                    actual_right,
                    dbg_info
                );
            } else {
                let op_str = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "mul",
                    BinOp::Div => "sdiv",
                    BinOp::Mod => "srem",
                    BinOp::BitAnd => "and",
                    BinOp::BitOr => "or",
                    BinOp::BitXor => "xor",
                    BinOp::Shl => "shl",
                    BinOp::Shr => "ashr",
                    _ => {
                        return Err(CodegenError::InternalError(format!(
                            "BinOp {:?} in int_arith codegen path",
                            op
                        )))
                    }
                };
                // Use MAX of both operand widths as the target so that narrower
                // operands are promoted before the instruction (fixes P3: e.g.,
                // shl i16 %t20, %t23 where %t20 is actually i8).
                let left_bits = self.get_integer_bits(&left_type);
                let right_bits = self.get_integer_bits(&right_type);
                let target_bits = if left_bits > 0 && right_bits > 0 {
                    std::cmp::max(left_bits, right_bits)
                } else if left_bits > 0 {
                    left_bits
                } else if right_bits > 0 {
                    right_bits
                } else {
                    64 // default
                };
                let int_llvm_owned = format!("i{}", target_bits);
                let int_llvm: &str = &int_llvm_owned;

                // Coerce both operands to the target width (using inferred types, not llvm_type_of)
                let left_ty_str = format!("i{}", if left_bits > 0 { left_bits } else { 64 });
                let right_ty_str = format!("i{}", if right_bits > 0 { right_bits } else { 64 });
                let coerced_left =
                    self.coerce_int_width(&left_val, &left_ty_str, int_llvm, counter, &mut ir);
                let coerced_right =
                    self.coerce_int_width(&right_val, &right_ty_str, int_llvm, counter, &mut ir);
                write_ir!(
                    ir,
                    "  {} = {} {} {}, {}{}",
                    tmp,
                    op_str,
                    int_llvm,
                    coerced_left,
                    coerced_right,
                    dbg_info
                );
            }
            Ok((tmp, ir))
        }
    }

    /// Generate unary expression
    #[inline(never)]
    pub(crate) fn generate_unary_expr(
        &mut self,
        op: &UnaryOp,
        expr: &Spanned<Expr>,
        counter: &mut usize,
        span: Span,
    ) -> CodegenResult<(String, String)> {
        let (val, val_ir) = self.generate_expr(expr, counter)?;
        let tmp = self.next_temp(counter);

        let mut ir = val_ir;
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let expr_type = self.infer_expr_type(expr);
        let int_llvm = match &expr_type {
            ResolvedType::I8 | ResolvedType::U8 => "i8",
            ResolvedType::I16 | ResolvedType::U16 => "i16",
            ResolvedType::I32 | ResolvedType::U32 => "i32",
            ResolvedType::I128 | ResolvedType::U128 => "i128",
            ResolvedType::Bool => "i1",
            _ => "i64",
        };
        match op {
            UnaryOp::Neg => {
                // Float negation must use fsub with a matching-width 0.0 constant.
                // `sub iN 0, <float>` is invalid IR.
                match &expr_type {
                    ResolvedType::F32 => {
                        write_ir!(
                            ir,
                            "  {} = fsub float 0.000000e+00, {}{}",
                            tmp,
                            val,
                            dbg_info
                        );
                    }
                    ResolvedType::F64 => {
                        write_ir!(
                            ir,
                            "  {} = fsub double 0.000000e+00, {}{}",
                            tmp,
                            val,
                            dbg_info
                        );
                    }
                    _ => {
                        write_ir!(ir, "  {} = sub {} 0, {}{}", tmp, int_llvm, val, dbg_info);
                    }
                }
            }
            UnaryOp::Not => {
                // Logical NOT: convert to i1 via icmp ne, then xor to flip
                let val_ty = self.llvm_type_of(&val);
                let bool_val = if val_ty == "i1" {
                    val.clone()
                } else {
                    let to_bool = self.next_temp(counter);
                    write_ir!(ir, "  {} = icmp ne {} {}, 0", to_bool, val_ty, val);
                    self.fn_ctx.record_emitted_type(&to_bool, "i1");
                    to_bool
                };
                // xor i1 produces i1 — keep as i1 and let callers zext if needed
                write_ir!(ir, "  {} = xor i1 {}, 1{}", tmp, bool_val, dbg_info);
            }
            UnaryOp::BitNot => {
                write_ir!(ir, "  {} = xor {} {}, -1{}", tmp, int_llvm, val, dbg_info);
            }
        }

        Ok((tmp, ir))
    }

    /// Generate ternary expression
    #[inline(never)]
    pub(crate) fn generate_cast_expr(
        &mut self,
        expr: &Spanned<Expr>,
        ty: &Spanned<Type>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Infer the source type BEFORE generate_expr so registry pollution
        // from earlier expressions doesn't mask the true source width.
        let src_type_static = self.infer_expr_type(expr);
        let (val, val_ir) = self.generate_expr(expr, counter)?;
        let mut ir = val_ir;

        let target_type = self.ast_type_to_resolved(&ty.node);
        let llvm_type = self.type_to_llvm(&target_type);

        // Check source type for str→i64 cast: extract data pointer from fat pointer.
        //
        // Phase E: prefer the statically-inferred source type over
        // llvm_type_of(&val). The latter reads temp_var_types which can be
        // polluted by catch-all registrations that run on outer
        // expressions. For an explicit `as` cast, the user's intent is
        // clear from the AST and shouldn't be overridden by an
        // accidentally-registered Vec<T> or similar.
        let static_src_llvm = self.type_to_llvm(&src_type_static);
        let src_llvm_ty = if static_src_llvm.starts_with('i')
            || static_src_llvm == "float"
            || static_src_llvm == "double"
        {
            static_src_llvm.clone()
        } else {
            self.llvm_type_of(&val)
        };
        if src_llvm_ty == "{ i8*, i64 }" && llvm_type == "i64" {
            // str → i64: extract the data pointer (field 0) and ptrtoint
            let ptr_val = self.next_temp(counter);
            let result = self.next_temp(counter);
            write_ir!(ir, "  {} = extractvalue {{ i8*, i64 }} {}, 0", ptr_val, val);
            write_ir!(ir, "  {} = ptrtoint i8* {} to i64", result, ptr_val);
            self.fn_ctx.record_emitted_type(&result, "i64");
            return Ok((result, ir));
        }

        // i64 → str cast: convert pointer-as-i64 to fat pointer { i8*, i64 }
        if src_llvm_ty == "i64" && llvm_type == "{ i8*, i64 }" {
            let ptr_val = self.next_temp(counter);
            let fat1 = self.next_temp(counter);
            let result = self.next_temp(counter);
            write_ir!(ir, "  {} = inttoptr i64 {} to i8*", ptr_val, val);
            write_ir!(
                ir,
                "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
                fat1,
                ptr_val
            );
            self.fn_ctx.record_emitted_type(&fat1, "{ i8*, i64 }");
            write_ir!(
                ir,
                "  {} = insertvalue {{ i8*, i64 }} {}, i64 0, 1",
                result,
                fat1
            );
            self.fn_ctx.record_emitted_type(&result, "{ i8*, i64 }");
            return Ok((result, ir));
        }

        // pointer/struct → i64 cast: ptrtoint
        if llvm_type == "i64" && (src_llvm_ty.ends_with('*') || src_llvm_ty.starts_with('%')) {
            let result = self.next_temp(counter);
            if src_llvm_ty.ends_with('*') {
                write_ir!(ir, "  {} = ptrtoint {} {} to i64", result, src_llvm_ty, val);
                self.fn_ctx.record_emitted_type(&result, "i64");
            } else {
                // Named struct type — the SSA value is actually a pointer (from alloca)
                write_ir!(
                    ir,
                    "  {} = ptrtoint {}* {} to i64",
                    result,
                    src_llvm_ty,
                    val
                );
                self.fn_ctx.record_emitted_type(&result, "i64");
            }
            return Ok((result, ir));
        }

        // Integer width coercion.
        //
        // Phase E: explicit `as` cast is a user directive — always honor
        // trunc/sext for int→int width mismatches rather than gating on
        // `has_known_type`. The previous guard protected against an
        // unregistered `%tN` falsely reporting i64 and then being
        // truncated; but in an `as` cast the target width is a pure user
        // statement of intent and the source width is either registered
        // (temps produced by the current generator, which now tracks
        // types more thoroughly after the A1 SSA registry work) or a
        // fallback i64 that is actually the alloca-load width in
        // practice. Unconditional coercion fixes `body_size as u32`
        // returning i64 without trunc.
        {
            if src_llvm_ty.starts_with('i')
                && llvm_type.starts_with('i')
                && src_llvm_ty != llvm_type
            {
                let src_bits: u32 = src_llvm_ty[1..].parse().unwrap_or(0);
                let dst_bits: u32 = llvm_type[1..].parse().unwrap_or(0);
                if src_bits > 0 && dst_bits > 0 && src_bits != dst_bits {
                    let result = self.next_temp(counter);
                    if src_bits > dst_bits {
                        write_ir!(
                            ir,
                            "  {} = trunc {} {} to {}",
                            result,
                            src_llvm_ty,
                            val,
                            llvm_type
                        );
                    } else {
                        write_ir!(
                            ir,
                            "  {} = sext {} {} to {}",
                            result,
                            src_llvm_ty,
                            val,
                            llvm_type
                        );
                    }
                    return Ok((result, ir));
                }
            }
        }

        // Float literal → integer: a float literal (e.g., "3.140000e+00") typed as i64 by
        // the "everything is i64" fallback needs fptosi when cast to i64. Without this,
        // `ret i64 3.140000e+00` is emitted which is invalid LLVM IR.
        if src_llvm_ty.starts_with('i') && llvm_type.starts_with('i') {
            let is_float_literal =
                !val.starts_with('%') && (val.contains("e+") || val.contains("e-"));
            if is_float_literal {
                let result = self.next_temp(counter);
                write_ir!(ir, "  {} = fptosi double {} to {}", result, val, llvm_type);
                return Ok((result, ir));
            }
        }

        // Float width coercion: f32 ↔ f64 (fpext/fptrunc)
        if (src_llvm_ty == "float" && llvm_type == "double")
            || (src_llvm_ty == "double" && llvm_type == "float")
        {
            let result = self.next_temp(counter);
            if src_llvm_ty == "float" {
                write_ir!(ir, "  {} = fpext float {} to double", result, val);
            } else {
                write_ir!(ir, "  {} = fptrunc double {} to float", result, val);
            }
            return Ok((result, ir));
        }

        // Integer ↔ float coercion (as f64, as f32 from int, as i64 from float)
        if src_llvm_ty.starts_with('i') && (llvm_type == "float" || llvm_type == "double") {
            // Check if the value is actually a float literal (e.g., "5.000000e+00")
            // that was given i64 type by the "everything is i64" fallback.
            // Float literals don't start with '%' and contain 'e+' or 'e-' (scientific notation).
            let is_float_literal =
                !val.starts_with('%') && (val.contains("e+") || val.contains("e-"));
            if is_float_literal {
                if llvm_type == "float" {
                    // f32 target: parse double literal → truncate to f32 → emit as LLVM hex.
                    // LLVM requires float constants to be exactly representable or in hex form.
                    // Hex format uses the double-precision encoding of the f32 value.
                    if let Ok(d) = val.parse::<f64>() {
                        let f = d as f32;
                        let f_as_double = f as f64;
                        let bits = f_as_double.to_bits();
                        return Ok((format!("0x{:016X}", bits), ir));
                    }
                }
                // double target: return the literal directly
                return Ok((val.clone(), ir));
            }
            let result = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = sitofp {} {} to {}",
                result,
                src_llvm_ty,
                val,
                llvm_type
            );
            return Ok((result, ir));
        }
        if (src_llvm_ty == "float" || src_llvm_ty == "double") && llvm_type.starts_with('i') {
            let result = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = fptosi {} {} to {}",
                result,
                src_llvm_ty,
                val,
                llvm_type
            );
            return Ok((result, ir));
        }

        // Simple cast - in many cases just bitcast or pass through
        let result = self.next_temp(counter);
        match (&target_type, llvm_type.as_str()) {
            // Integer to pointer cast
            (ResolvedType::Pointer(_), _)
            | (ResolvedType::Ref(_), _)
            | (ResolvedType::RefMut(_), _) => {
                write_ir!(ir, "  {} = inttoptr i64 {} to {}", result, val, llvm_type);
            }
            // Default: just use the value as-is (same size types)
            _ => {
                return Ok((val, ir));
            }
        }

        Ok((result, ir))
    }
}
