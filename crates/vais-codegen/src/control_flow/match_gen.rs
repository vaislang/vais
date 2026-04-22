use super::*;

impl CodeGenerator {
    /// Generate code for match expression
    #[inline(never)]
    pub(crate) fn generate_match(
        &mut self,
        match_expr: &Spanned<Expr>,
        arms: &[MatchArm],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Generate the expression to match against
        let (match_val_raw, mut ir) = self.generate_expr(match_expr, counter)?;

        // Check if match expression is a struct/enum value (not a pointer)
        // If it's a function call returning enum, we need to store it first
        let match_type = self.infer_expr_type(match_expr);
        let is_enum_or_struct = matches!(&match_type, ResolvedType::Named { .. });
        let is_value = self.is_expr_value(match_expr);

        // If it's an enum/struct value from a function call, store it on the stack
        let match_val = if is_enum_or_struct && is_value {
            let llvm_type = self.type_to_llvm(&match_type);
            let stack_ptr = self.next_temp(counter);
            self.emit_entry_alloca(&stack_ptr, &llvm_type);
            write_ir!(
                ir,
                "  store {} {}, {}* {}",
                llvm_type,
                match_val_raw,
                llvm_type,
                stack_ptr
            );

            stack_ptr
        } else {
            match_val_raw
        };

        // For str match values, extract the raw i8* pointer from the fat ptr { i8*, i64 }
        // so that strcmp in pattern matching receives the correct type.
        let match_val = if matches!(&match_type, ResolvedType::Str) {
            let raw_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                raw_ptr,
                match_val
            );
            raw_ptr
        } else {
            match_val
        };

        let merge_label = self.next_label("match.merge");
        let mut arm_labels: Vec<String> = Vec::with_capacity(arms.len());
        let mut arm_values: Vec<(String, String)> = Vec::with_capacity(arms.len()); // (value, label)

        // Pre-compute the phi type (shared by both switch and fallback paths)
        // so per-arm coercion can use it before branching to merge, and the
        // later merge-block logic doesn't re-derive a different value.
        let arm_body_type: ResolvedType = if !arms.is_empty() {
            let first_arm_ty = self.infer_expr_type(&arms[0].body);
            if matches!(first_arm_ty, ResolvedType::Named { .. }) {
                first_arm_ty
            } else {
                let named_from_arms = arms.iter().skip(1).find_map(|arm| {
                    let ty = self.infer_expr_type(&arm.body);
                    if matches!(ty, ResolvedType::Named { .. }) {
                        Some(ty)
                    } else {
                        None
                    }
                });
                named_from_arms.unwrap_or_else(|| {
                    if let Some(ret_ty) = &self.fn_ctx.current_return_type {
                        let ret_informative = matches!(
                            ret_ty,
                            ResolvedType::Named { .. }
                                | ResolvedType::I8
                                | ResolvedType::U8
                                | ResolvedType::I16
                                | ResolvedType::U16
                                | ResolvedType::I32
                                | ResolvedType::U32
                                | ResolvedType::F32
                                | ResolvedType::F64
                        );
                        let first_is_i64 = matches!(first_arm_ty, ResolvedType::I64);
                        if ret_informative && first_is_i64 {
                            return ret_ty.clone();
                        }
                    }
                    first_arm_ty
                })
            }
        } else {
            ResolvedType::I64
        };

        // Check if all arms are simple integer literals (can use switch)
        let all_int_literals = arms.iter().all(|arm| {
            matches!(
                &arm.pattern.node,
                Pattern::Literal(Literal::Int(_)) | Pattern::Wildcard
            )
        });

        if all_int_literals && !arms.is_empty() {
            // Use LLVM switch instruction for integer pattern matching
            let default_label = self.next_label("match.default");
            let mut switch_cases: Vec<(i64, String)> = Vec::with_capacity(arms.len());
            let mut default_arm: Option<&MatchArm> = None;

            // First pass: collect labels and find default
            for arm in arms {
                match &arm.pattern.node {
                    Pattern::Literal(Literal::Int(n)) => {
                        let label = self.next_label("match.arm");
                        switch_cases.push((*n, label.clone()));
                        arm_labels.push(label);
                    }
                    Pattern::Wildcard => {
                        default_arm = Some(arm);
                    }
                    _ => {}
                }
            }

            // Coerce match value to i64 for switch (e.g., i8 param → i64)
            let match_val_ty = self.llvm_type_of(&match_val);
            let switch_val =
                self.coerce_int_width(&match_val, &match_val_ty, "i64", counter, &mut ir);
            // Generate switch instruction
            write_ir!(
                ir,
                "  switch i64 {}, label %{} [",
                switch_val,
                default_label
            );
            for (val, label) in &switch_cases {
                write_ir!(ir, "    i64 {}, label %{}", val, label);
            }
            ir.push_str("  ]\n");

            // Generate arm bodies for integer cases
            let mut case_idx = 0;
            for arm in arms {
                if let Pattern::Literal(Literal::Int(_)) = &arm.pattern.node {
                    let label = &arm_labels[case_idx];
                    write_ir!(ir, "{}:", label);

                    // Check guard if present
                    if let Some(guard) = &arm.guard {
                        let guard_pass = self.next_label("match.guard.pass");
                        let guard_fail = self.next_label("match.guard.fail");

                        let (guard_val, guard_ir) = self.generate_expr(guard, counter)?;
                        ir.push_str(&guard_ir);
                        write_ir!(
                            ir,
                            "  br i1 {}, label %{}, label %{}",
                            guard_val,
                            guard_pass,
                            guard_fail
                        );

                        // Guard passed - execute body
                        write_ir!(ir, "{}:", guard_pass);
                        self.fn_ctx.current_block = guard_pass.clone();
                        let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                        ir.push_str(&body_ir);
                        arm_values.push((body_val, self.fn_ctx.current_block.clone()));
                        write_ir!(ir, "  br label %{}", merge_label);

                        // Guard failed - go to default
                        write_ir!(ir, "{}:", guard_fail);
                        write_ir!(ir, "  br label %{}", default_label);
                    } else {
                        self.fn_ctx.current_block.clone_from(label);
                        let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                        ir.push_str(&body_ir);
                        arm_values.push((body_val, self.fn_ctx.current_block.clone()));
                        write_ir!(ir, "  br label %{}", merge_label);
                    }

                    case_idx += 1;
                }
            }

            // Generate default arm
            write_ir!(ir, "{}:", default_label);
            self.fn_ctx.current_block = default_label.clone();
            if let Some(arm) = default_arm {
                let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                ir.push_str(&body_ir);
                // Body may have introduced new basic blocks — the branch to
                // merge originates from whichever block is current now.
                arm_values.push((body_val, self.fn_ctx.current_block.clone()));
            } else {
                // No default arm - unreachable or return 0
                arm_values.push(("0".to_string(), self.fn_ctx.current_block.clone()));
            }
            write_ir!(ir, "  br label %{}", merge_label);
        } else {
            // Fall back to chained conditional branches for complex patterns
            let mut current_label = self.next_label("match.check");
            // Create a default fallthrough block so all check-fail paths have a phi entry
            let default_label = self.next_label("match.default");
            write_ir!(ir, "  br label %{}", current_label);

            for (i, arm) in arms.iter().enumerate() {
                let is_last = i == arms.len() - 1;
                let next_label = if is_last {
                    default_label.clone()
                } else {
                    self.next_label("match.check")
                };
                let arm_body_label = self.next_label("match.arm");

                write_ir!(ir, "{}:", current_label);

                // Generate pattern check — pass match_type for correct nested tuple type inference
                let (check_val, check_ir) = self.generate_pattern_check_typed(
                    &arm.pattern,
                    &match_val,
                    counter,
                    &match_type,
                )?;
                ir.push_str(&check_ir);

                // Phase B5: snapshot outer-scope locals so arm-local pattern
                // bindings don't leak beyond this arm. Shadowed outer names
                // are reverted to their pre-arm value after body generation.
                let pre_arm_locals: std::collections::HashMap<String, crate::types::LocalVar> =
                    self.fn_ctx.locals.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

                // Handle guard - need to bind variables first so guard can use them
                if let Some(guard) = &arm.guard {
                    let guard_bind = self.next_label("match.guard.bind");
                    let guard_check = self.next_label("match.guard.check");

                    // First check pattern
                    write_ir!(
                        ir,
                        "  br i1 {}, label %{}, label %{}",
                        check_val,
                        guard_bind,
                        next_label
                    );

                    // Bind pattern variables for guard to use
                    write_ir!(ir, "{}:", guard_bind);
                    let bind_ir = self.generate_pattern_bindings_typed(
                        &arm.pattern,
                        &match_val,
                        counter,
                        &match_type,
                    )?;
                    ir.push_str(&bind_ir);
                    write_ir!(ir, "  br label %{}", guard_check);

                    // Then check guard
                    write_ir!(ir, "{}:", guard_check);
                    let (guard_val, guard_ir) = self.generate_expr(guard, counter)?;
                    ir.push_str(&guard_ir);
                    // Guard value is i64 (0 or 1), convert to i1 for branch
                    let guard_bool = self.next_temp(counter);
                    write_ir!(ir, "  {} = icmp ne i64 {}, 0", guard_bool, guard_val);
                    write_ir!(
                        ir,
                        "  br i1 {}, label %{}, label %{}",
                        guard_bool,
                        arm_body_label,
                        next_label
                    );

                    // Generate arm body (bindings already done)
                    write_ir!(ir, "{}:", arm_body_label);
                    self.fn_ctx.current_block.clone_from(&arm_body_label);
                } else {
                    write_ir!(
                        ir,
                        "  br i1 {}, label %{}, label %{}",
                        check_val,
                        arm_body_label,
                        next_label
                    );

                    // Generate arm body
                    write_ir!(ir, "{}:", arm_body_label);
                    self.fn_ctx.current_block.clone_from(&arm_body_label);

                    // Bind pattern variables if needed
                    let bind_ir = self.generate_pattern_bindings_typed(
                        &arm.pattern,
                        &match_val,
                        counter,
                        &match_type,
                    )?;
                    ir.push_str(&bind_ir);
                }

                let (mut body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                ir.push_str(&body_ir);

                // Phase B5: restore pre-arm locals. Any binding present only
                // after the arm is a pattern binding; drop it. Any binding
                // present before is reverted to the pre-arm value (handles
                // outer-name shadowing). Other fn_ctx state untouched.
                let pre_arm_keys: std::collections::HashSet<&String> =
                    pre_arm_locals.keys().collect();
                self.fn_ctx
                    .locals
                    .retain(|k, _| pre_arm_keys.contains(k));
                for (k, v) in &pre_arm_locals {
                    self.fn_ctx.locals.insert(k.clone(), v.clone());
                }

                // Coerce arm values to match the phi type.
                // Skip if the body_val is a placeholder (void/ret arms).
                if !body_val.is_empty()
                    && body_val != "void"
                    && !body_ir.trim_end().ends_with("unreachable")
                {
                    let arm_inferred = self.infer_expr_type(&arm.body);
                    if matches!(arm_inferred, ResolvedType::Bool) {
                        let coerced = self.next_temp(counter);
                        write_ir!(ir, "  {} = zext i1 {} to i64", coerced, body_val);
                        body_val = coerced;
                        // Coerce to arm_body_type if it's a Named pointer or
                        // narrow int — the phi incoming must match the phi's
                        // declared type.
                        if matches!(&arm_body_type, ResolvedType::Named { .. }) {
                            let llvm_ty = self.type_to_llvm(&arm_body_type);
                            let casted = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = inttoptr i64 {} to {}*",
                                casted,
                                body_val,
                                llvm_ty
                            );
                            body_val = casted;
                        } else if matches!(
                            &arm_body_type,
                            ResolvedType::I32
                                | ResolvedType::U32
                                | ResolvedType::I16
                                | ResolvedType::U16
                                | ResolvedType::I8
                                | ResolvedType::U8
                        ) {
                            let target = self.type_to_llvm(&arm_body_type);
                            let narrowed = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = trunc i64 {} to {}",
                                narrowed,
                                body_val,
                                target
                            );
                            body_val = narrowed;
                        }
                    } else if matches!(
                        arm_body_type,
                        ResolvedType::I32
                            | ResolvedType::U32
                            | ResolvedType::I16
                            | ResolvedType::U16
                            | ResolvedType::I8
                            | ResolvedType::U8
                    ) {
                        // arm_body_type (phi type) is narrow int — trunc the
                        // arm's (possibly i64-wide) value to match.
                        let actual = self.llvm_type_of(&body_val);
                        let target = self.type_to_llvm(&arm_body_type);
                        if actual == "i64" && target != "i64" {
                            let narrowed = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = trunc i64 {} to {}",
                                narrowed,
                                body_val,
                                target
                            );
                            body_val = narrowed;
                        } else if actual == "i1" && target != "i1" {
                            // i1 → wider int (e.g., bool-typed pattern binding
                            // flowing into i32-typed phi from `order_by_compare`
                            // style match arms). zext preserves 0/1.
                            let widened = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = zext i1 {} to {}",
                                widened,
                                body_val,
                                target
                            );
                            body_val = widened;
                        } else if actual.starts_with('i')
                            && target.starts_with('i')
                            && actual != target
                        {
                            // General int-width mismatch (e.g., i8 → i32 for
                            // pattern-bound narrow locals). Use trunc/sext.
                            body_val =
                                self.coerce_int_width(&body_val, &actual, &target, counter, &mut ir);
                        }
                    } else if matches!(arm_inferred, ResolvedType::Named { .. }) {
                        // Named type (struct/enum): phi uses pointer type (%T*).
                        // If this arm body produced a value (e.g., function return),
                        // we must alloca+store it to get a pointer for the phi node.
                        // If the arm body already produced a pointer (e.g., struct literal,
                        // local variable, enum constructor), use it as-is.
                        if self.is_expr_value(&arm.body) {
                            let llvm_ty = self.type_to_llvm(&arm_inferred);
                            let alloca = self.next_temp(counter);
                            self.emit_entry_alloca(&alloca, &llvm_ty);
                            write_ir!(
                                ir,
                                "  store {} {}, {}* {}",
                                llvm_ty,
                                body_val,
                                llvm_ty,
                                alloca
                            );
                            body_val = alloca;
                        }
                        // else: already a pointer, use as-is
                    } else if !matches!(arm_inferred, ResolvedType::Named { .. }) {
                        // Arm produces i64 (e.g., closure call) but function returns
                        // Named type (struct/enum) — inttoptr to match phi pointer type
                        if let Some(ret_ty) = &self.fn_ctx.current_return_type {
                            if matches!(ret_ty, ResolvedType::Named { .. }) {
                                let llvm_ty = self.type_to_llvm(ret_ty);
                                let coerced = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = inttoptr i64 {} to {}*",
                                    coerced,
                                    body_val,
                                    llvm_ty
                                );
                                body_val = coerced;
                            }
                        }
                    }
                }

                // Use actual current block (may differ from arm_body_label if body
                // inserted intermediate labels, e.g., division-by-zero guard)
                let actual_block = self.fn_ctx.current_block.clone();
                arm_values.push((body_val, actual_block));
                write_ir!(ir, "  br label %{}", merge_label);

                current_label = next_label;
            }

            // Default fallthrough block (no arm matched)
            write_ir!(ir, "{}:", default_label);
            self.fn_ctx.current_block.clone_from(&default_label);
            // Use appropriate default value based on arm types or function return type
            let default_val = {
                let mut resolved = if !arms.is_empty() {
                    self.infer_expr_type(&arms[0].body)
                } else {
                    ResolvedType::I64
                };
                // If first arm type is i64 but function returns Named, use Named
                if !matches!(resolved, ResolvedType::Named { .. }) {
                    if let Some(ret_ty) = &self.fn_ctx.current_return_type {
                        if matches!(ret_ty, ResolvedType::Named { .. }) {
                            resolved = ret_ty.clone();
                        }
                    }
                }
                match &resolved {
                    ResolvedType::Str => {
                        // Str PHI uses fat-pointer { i8*, i64 } — emit a zero
                        // constant for the unreachable default fallthrough.
                        let zinit = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = insertvalue {{ i8*, i64 }} {{ i8* null, i64 0 }}, i64 0, 1",
                            zinit
                        );
                        zinit
                    }
                    ResolvedType::Named { .. }
                    | ResolvedType::Ref(_)
                    | ResolvedType::RefMut(_) => "null".to_string(),
                    ResolvedType::F64 => "0.0".to_string(),
                    ResolvedType::Bool => "0".to_string(), // Bool → i64 in phi
                    _ => "0".to_string(),
                }
            };
            arm_values.push((default_val, default_label.clone()));
            write_ir!(ir, "  br label %{}", merge_label);
        }

        // Merge block with phi node
        write_ir!(ir, "{}:", merge_label);
        // Subsequent emission (phi, and whatever the caller appends) lives in
        // the merge block — track that so nested match phi predecessors are
        // recorded correctly.
        self.fn_ctx.current_block = merge_label.clone();

        if arm_values.is_empty() {
            Ok(("0".to_string(), ir))
        } else {
            // Phi node type was already determined above (needed for per-arm
            // coercion before branching to merge).

            // Check if all arms produce void/Unit — phi void is invalid in LLVM IR.
            // Use a placeholder add instruction instead, same pattern as if_else.rs.
            let llvm_type_str = self.type_to_llvm(&arm_body_type);
            let is_void_type = crate::helpers::is_void_result(&llvm_type_str, &arm_body_type);

            let phi_result = self.next_temp(counter);

            if is_void_type {
                ir.push_str(&crate::helpers::void_placeholder_ir(&phi_result));
                return Ok((phi_result, ir));
            }

            let is_named_type = matches!(&arm_body_type, ResolvedType::Named { .. });
            let phi_type = match &arm_body_type {
                ResolvedType::Named { .. } => {
                    // Enum/struct types are returned as pointers in text codegen
                    let llvm_ty = self.type_to_llvm(&arm_body_type);
                    format!("{}*", llvm_ty)
                }
                ResolvedType::Str => "{ i8*, i64 }".to_string(),
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                    let inner_ty = self.type_to_llvm(inner);
                    format!("{}*", inner_ty)
                }
                ResolvedType::F64 => "double".to_string(),
                ResolvedType::F32 => "float".to_string(),
                ResolvedType::Bool => "i64".to_string(), // Bool is zext'd to i64 in codegen
                ResolvedType::I8 | ResolvedType::U8 => "i8".to_string(),
                ResolvedType::I16 | ResolvedType::U16 => "i16".to_string(),
                ResolvedType::I32 | ResolvedType::U32 => "i32".to_string(),
                _ => "i64".to_string(),
            };

            // Substitute "void" placeholders with a phi-type-appropriate default
            // (null for pointer types, 0 otherwise). Placing a literal `void` in
            // a phi incoming is invalid LLVM IR.
            let void_substitute = if phi_type.ends_with('*') {
                "null"
            } else if phi_type == "{ i8*, i64 }" {
                "{ i8* null, i64 0 }"
            } else if phi_type == "double" || phi_type == "float" {
                "0.0"
            } else {
                "0"
            };

            // "void" arm values are replaced with a phi-type appropriate
            // default, but no other coercion is performed here — the coercion
            // would need to live in the arm's block (for dominance), not in
            // the merge block. Arm-body coercion already handled during
            // generate_expr of each arm above.
            let phi_args: Vec<String> = arm_values
                .iter()
                .map(|(val, label)| {
                    let safe = if val == "void" { void_substitute } else { val.as_str() };
                    format!("[ {}, %{} ]", safe, label)
                })
                .collect();
            write_ir!(
                ir,
                "  {} = phi {} {}",
                phi_result,
                phi_type,
                phi_args.join(", ")
            );

            // For Named types (enum/struct), the phi gives us a pointer.
            // Load the value so it can be used directly (e.g., as a return value).
            if is_named_type {
                let llvm_ty = self.type_to_llvm(&arm_body_type);
                let loaded = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = load {}, {}* {}",
                    loaded,
                    llvm_ty,
                    llvm_ty,
                    phi_result
                );

                Ok((loaded, ir))
            } else {
                // Register the actual phi IR type so downstream coercion
                // uses the correct source type instead of the default i64
                // fallback in llvm_type_of.
                if matches!(arm_body_type, ResolvedType::Bool) {
                    self.fn_ctx
                        .register_temp_type(&phi_result, ResolvedType::I64);
                } else if !matches!(arm_body_type, ResolvedType::I64) {
                    self.fn_ctx
                        .register_temp_type(&phi_result, arm_body_type.clone());
                }
                // Str PHI ownership merge (Phase 191 #9, mirrors if-expr at
                // expr_helpers_control.rs:344-371). If any arm value owns a
                // tracked alloc_slot, register all such slots against the PHI
                // SSA so a subsequent return / let-binding can transfer
                // ownership and skip the would-be cleanup.
                if matches!(arm_body_type, ResolvedType::Str) {
                    let mut slots: Vec<String> = Vec::new();
                    for (val, _label) in &arm_values {
                        let key = val.trim().to_string();
                        if let Some(slot) = self.fn_ctx.string_value_slot.get(&key).cloned() {
                            if !slots.contains(&slot) {
                                slots.push(slot);
                            }
                        }
                    }
                    if !slots.is_empty() {
                        self.fn_ctx
                            .string_value_slot
                            .insert(phi_result.clone(), slots[0].clone());
                        if slots.len() > 1 {
                            self.fn_ctx
                                .phi_extra_slots
                                .insert(phi_result.clone(), slots[1..].to_vec());
                        }
                    }
                }
                Ok((phi_result, ir))
            }
        }
    }
}
