use super::*;
use std::fmt::Write;

impl CodeGenerator {
    /// Generate code for if-else branches with termination tracking
    /// Returns (value, ir, is_terminated, last_block_name)
    /// last_block_name is the block that actually branches to the outer merge
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
                let block_type = self.infer_block_type(then_stmts);
                let llvm_type = self.type_to_llvm(&block_type);

                // Check if the result is a struct type (returned as pointer from struct literals)
                let is_struct_result = matches!(&block_type, ResolvedType::Named { .. })
                    && !self.is_block_result_value(then_stmts);

                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let mut ir = cond_ir;

                // Convert to i1 for branch (type-aware: skips for bool/i1)
                let (cond_bool, conv_ir) = self.generate_cond_to_i1(cond, &cond_val, counter);
                ir.push_str(&conv_ir);

                writeln!(
                    ir,
                    "  br i1 {}, label %{}, label %{}",
                    cond_bool, then_label, else_label
                )
                .unwrap();

                // Then branch
                writeln!(ir, "{}:", then_label).unwrap();
                self.fn_ctx.current_block = then_label; // move: label not used after
                let (then_val, then_ir, then_terminated) =
                    self.generate_block_stmts(then_stmts, counter)?;
                ir.push_str(&then_ir);

                // For struct results, load the value before branch if it's a pointer
                let then_val_for_phi = if is_struct_result && !then_terminated {
                    let loaded = self.next_temp(counter);
                    writeln!(
                        ir,
                        "  {} = load {}, {}* {}",
                        loaded, llvm_type, llvm_type, then_val
                    )
                    .unwrap();
                    loaded
                } else {
                    then_val // move: not used after
                };

                let then_actual_block = std::mem::take(&mut self.fn_ctx.current_block); // take ownership
                let then_from_label = if !then_terminated {
                    writeln!(ir, "  br label %{}", local_merge).unwrap();
                    then_actual_block
                } else {
                    String::new()
                };

                // Else branch
                writeln!(ir, "{}:", else_label).unwrap();
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
                let else_val_for_phi = if is_struct_result
                    && !else_terminated
                    && has_else
                    && nested_last_block.is_empty()
                {
                    let loaded = self.next_temp(counter);
                    writeln!(
                        ir,
                        "  {} = load {}, {}* {}",
                        loaded, llvm_type, llvm_type, else_val
                    )
                    .unwrap();
                    loaded
                } else {
                    else_val // move: not used after
                };

                let else_from_label = if !else_terminated {
                    writeln!(ir, "  br label %{}", local_merge).unwrap();
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
                writeln!(ir, "{}:", local_merge).unwrap();
                self.fn_ctx.current_block = local_merge.clone();
                let result = self.next_temp(counter);

                // Check if the block type is void/unit - if so, don't generate phi nodes
                // (phi nodes cannot have void type in LLVM IR)
                let is_void_type = matches!(block_type, ResolvedType::Unit);

                // Build phi node only from non-terminated predecessors and non-void types
                if is_void_type {
                    // Void type: value is not used, just use 0
                    writeln!(ir, "  {} = add i64 0, 0", result).unwrap();
                } else if !then_from_label.is_empty() && !else_from_label.is_empty() {
                    writeln!(
                        ir,
                        "  {} = phi {} [ {}, %{} ], [ {}, %{} ]",
                        result,
                        llvm_type,
                        then_val_for_phi,
                        then_from_label,
                        else_val_for_phi,
                        else_from_label
                    )
                    .unwrap();
                } else if !then_from_label.is_empty() {
                    writeln!(
                        ir,
                        "  {} = phi {} [ {}, %{} ]",
                        result, llvm_type, then_val_for_phi, then_from_label
                    )
                    .unwrap();
                } else if !else_from_label.is_empty() {
                    writeln!(
                        ir,
                        "  {} = phi {} [ {}, %{} ]",
                        result, llvm_type, else_val_for_phi, else_from_label
                    )
                    .unwrap();
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
