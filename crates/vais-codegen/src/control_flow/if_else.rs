use super::*;

impl CodeGenerator {
    /// Generate code for if-else branches with termination tracking
    /// Returns (value, ir, is_terminated, last_block_name)
    /// last_block_name is the block that actually branches to the outer merge
    #[inline(never)]
    pub(crate) fn generate_if_else_with_term(
        &mut self,
        if_else: &IfElse,
        counter: &mut usize,
        _merge_label: &str,
    ) -> CodegenResult<(String, String, bool, String)> {
        match if_else {
            IfElse::Else(stmts) => {
                let (val, ir, terminated) = self.generate_block_stmts(stmts, counter)?;
                // For plain else block, the "last block" is empty (caller handles it)
                Ok((val, ir, terminated, String::new()))
            }
            IfElse::ElseIf(cond, then_stmts, else_branch) => {
                // Generate nested if-else
                let then_label = self.next_label("elseif.then");
                let else_label = self.next_label("elseif.else");
                let local_merge = self.next_label("elseif.merge");

                // Infer the type of the then block for phi node
                let then_type = self.infer_block_type(then_stmts);
                let else_type_resolved = if let Some(eb) = else_branch {
                    match eb.as_ref() {
                        IfElse::Else(stmts) => self.infer_block_type(stmts),
                        IfElse::ElseIf(_, nested_then, _) => self.infer_block_type(nested_then),
                    }
                } else {
                    vais_types::ResolvedType::I64
                };
                let block_type = if self.type_to_llvm(&then_type) != self.type_to_llvm(&else_type_resolved) {
                    vais_types::ResolvedType::I64
                } else {
                    then_type
                };
                let llvm_type = self.type_to_llvm(&block_type);

                // Check each branch independently for struct pointer vs value
                let is_named_phi = matches!(&block_type, ResolvedType::Named { .. });
                let then_is_ptr = is_named_phi && !self.is_block_result_value(then_stmts);

                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let mut ir = cond_ir;

                // Convert to i1 for branch (type-aware: skips for bool/i1)
                let (cond_bool, conv_ir) = self.generate_cond_to_i1(cond, &cond_val, counter);
                ir.push_str(&conv_ir);

                write_ir!(
                    ir,
                    "  br i1 {}, label %{}, label %{}",
                    cond_bool,
                    then_label,
                    else_label
                );

                // Then branch
                write_ir!(ir, "{}:", then_label);
                self.fn_ctx.current_block = then_label; // move: label not used after
                let (then_val, then_ir, then_terminated) =
                    self.generate_block_stmts(then_stmts, counter)?;
                ir.push_str(&then_ir);

                // For struct results, load the value before branch if it's a pointer
                let then_val_for_phi = if then_is_ptr && !then_terminated {
                    let loaded = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}",
                        loaded,
                        llvm_type,
                        llvm_type,
                        then_val
                    );

                    loaded
                } else if !then_terminated {
                    // Coerce integer width if the value type differs from the phi type
                    let actual_ty = self.llvm_type_of(&then_val);
                    let coerced =
                        self.coerce_int_width(&then_val, &actual_ty, &llvm_type, counter, &mut ir);
                    // Also coerce float width (e.g., float→double or double→float for phi)
                    let actual_after = self.llvm_type_of(&coerced);
                    if actual_after != llvm_type
                        && (actual_after == "float" || actual_after == "double")
                        && (llvm_type == "float" || llvm_type == "double")
                    {
                        let tmp = self.next_temp(counter);
                        if actual_after == "float" {
                            write_ir!(ir, "  {} = fpext float {} to double", tmp, coerced);
                        } else {
                            write_ir!(ir, "  {} = fptrunc double {} to float", tmp, coerced);
                        }
                        tmp
                    } else {
                        coerced
                    }
                } else {
                    then_val // move: not used after
                };

                let then_actual_block = std::mem::take(&mut self.fn_ctx.current_block); // take ownership
                let then_from_label = if !then_terminated {
                    write_ir!(ir, "  br label %{}", local_merge);
                    then_actual_block
                } else {
                    String::new()
                };

                // Else branch
                write_ir!(ir, "{}:", else_label);
                self.fn_ctx.current_block = else_label; // move: not used after
                let has_else = else_branch.is_some();
                let (else_val, else_ir, else_terminated, nested_last_block) =
                    if let Some(nested) = else_branch {
                        self.generate_if_else_with_term(nested, counter, &local_merge)?
                    } else {
                        ("0".to_string(), String::new(), false, String::new())
                    };
                ir.push_str(&else_ir);

                // For struct results, load the value before branch if it's a pointer
                // But if else_val comes from a nested if-else (indicated by non-empty nested_last_block),
                // it's already a phi node value (not a pointer), so don't load it
                // Check else branch independently (may differ from then branch)
                let else_is_ptr = is_named_phi
                    && has_else
                    && nested_last_block.is_empty()
                    && else_branch
                        .as_ref()
                        .map(|eb| match eb.as_ref() {
                            vais_ast::IfElse::Else(stmts) => !self.is_block_result_value(stmts),
                            vais_ast::IfElse::ElseIf(..) => false, // nested produces phi value
                        })
                        .unwrap_or(false);
                let else_val_for_phi = if else_is_ptr
                    && !else_terminated
                    && has_else
                    && nested_last_block.is_empty()
                {
                    let loaded = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}",
                        loaded,
                        llvm_type,
                        llvm_type,
                        else_val
                    );

                    loaded
                } else if !else_terminated && has_else {
                    // Coerce integer width if the value type differs from the phi type
                    let actual_ty = self.llvm_type_of(&else_val);
                    let coerced =
                        self.coerce_int_width(&else_val, &actual_ty, &llvm_type, counter, &mut ir);
                    // Also coerce float width (e.g., float→double or double→float for phi)
                    let actual_after = self.llvm_type_of(&coerced);
                    if actual_after != llvm_type
                        && (actual_after == "float" || actual_after == "double")
                        && (llvm_type == "float" || llvm_type == "double")
                    {
                        let tmp = self.next_temp(counter);
                        if actual_after == "float" {
                            write_ir!(ir, "  {} = fpext float {} to double", tmp, coerced);
                        } else {
                            write_ir!(ir, "  {} = fptrunc double {} to float", tmp, coerced);
                        }
                        tmp
                    } else {
                        coerced
                    }
                } else {
                    else_val // move: not used after
                };

                let else_from_label = if !else_terminated {
                    write_ir!(ir, "  br label %{}", local_merge);
                    // If there was a nested if-else, use its merge block as the predecessor
                    if !nested_last_block.is_empty() {
                        nested_last_block
                    } else {
                        std::mem::take(&mut self.fn_ctx.current_block) // take ownership
                    }
                } else {
                    String::new()
                };

                // Both branches terminated = this whole if-else is terminated
                let all_terminated = then_terminated && else_terminated;

                // If both branches are terminated (e.g., both return/break),
                // skip the merge block entirely — it's unreachable
                if all_terminated {
                    // No merge block needed. Return a dummy value.
                    // The caller will see all_terminated=true and skip using this value.
                    let result = self.next_temp(counter);
                    return Ok((result, ir, true, String::new()));
                }

                // Merge
                write_ir!(ir, "{}:", local_merge);
                self.fn_ctx.current_block = local_merge.clone();
                let result = self.next_temp(counter);

                // Check if the block type is void/unit - if so, don't generate phi nodes
                // (phi nodes cannot have void type in LLVM IR)
                let is_void_type = matches!(block_type, ResolvedType::Unit);

                // Check for struct/non-struct type mismatch
                let phi_is_struct = llvm_type.starts_with('{') || llvm_type.starts_with('%');
                let then_actual_ty = self.llvm_type_of(&then_val_for_phi);
                let else_actual_ty = self.llvm_type_of(&else_val_for_phi);
                let phi_type_mismatch = if phi_is_struct {
                    (!then_from_label.is_empty() && then_actual_ty.starts_with('i') && !then_val_for_phi.starts_with("zeroinitializer"))
                        || (!else_from_label.is_empty() && else_actual_ty.starts_with('i') && else_val_for_phi != "0")
                } else {
                    (!then_from_label.is_empty() && (then_actual_ty.starts_with('{') || then_actual_ty.starts_with('%')))
                        || (!else_from_label.is_empty() && (else_actual_ty.starts_with('{') || else_actual_ty.starts_with('%')))
                };

                // Build phi node only from non-terminated predecessors and non-void types
                if is_void_type || phi_type_mismatch {
                    // When the phi type is str { i8*, i64 }, use a zeroinitializer instead
                    // of void placeholder (i64 0) to avoid type mismatch downstream.
                    if llvm_type == "{ i8*, i64 }" {
                        write_ir!(
                            ir,
                            "  {} = insertvalue {{ i8*, i64 }} {{ i8* null, i64 0 }}, i64 0, 1",
                            result
                        );
                        self.fn_ctx.register_temp_type(&result, vais_types::ResolvedType::Str);
                    } else {
                        ir.push_str(&crate::helpers::void_placeholder_ir(&result));
                        self.fn_ctx.register_temp_type(&result, vais_types::ResolvedType::I64);
                    }
                } else if !then_from_label.is_empty() && !else_from_label.is_empty() {
                    write_ir!(
                        ir,
                        "  {} = phi {} [ {}, %{} ], [ {}, %{} ]",
                        result,
                        llvm_type,
                        then_val_for_phi,
                        then_from_label,
                        else_val_for_phi,
                        else_from_label
                    );
                } else if !then_from_label.is_empty() {
                    write_ir!(
                        ir,
                        "  {} = phi {} [ {}, %{} ]",
                        result,
                        llvm_type,
                        then_val_for_phi,
                        then_from_label
                    );
                } else if !else_from_label.is_empty() {
                    write_ir!(
                        ir,
                        "  {} = phi {} [ {}, %{} ]",
                        result,
                        llvm_type,
                        else_val_for_phi,
                        else_from_label
                    );
                } else {
                    // Unreachable merge block — add terminator
                    ir.push_str("  unreachable\n");
                    return Ok((result, ir, true, local_merge));
                }

                // Return local_merge as the last block for this nested if-else
                Ok((result, ir, all_terminated, local_merge))
            }
        }
    }
}
