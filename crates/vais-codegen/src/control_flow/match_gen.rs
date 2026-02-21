use super::*;
use std::fmt::Write;

impl CodeGenerator {
    /// Generate code for match expression
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
            writeln!(ir, "  {} = alloca {}", stack_ptr, llvm_type).unwrap();
            writeln!(
                ir,
                "  store {} {}, {}* {}",
                llvm_type, match_val_raw, llvm_type, stack_ptr
            )
            .unwrap();
            stack_ptr
        } else {
            match_val_raw
        };

        let merge_label = self.next_label("match.merge");
        let mut arm_labels: Vec<String> = Vec::with_capacity(arms.len());
        let mut arm_values: Vec<(String, String)> = Vec::with_capacity(arms.len()); // (value, label)

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

            // Generate switch instruction
            writeln!(ir, "  switch i64 {}, label %{} [", match_val, default_label).unwrap();
            for (val, label) in &switch_cases {
                writeln!(ir, "    i64 {}, label %{}", val, label).unwrap();
            }
            ir.push_str("  ]\n");

            // Generate arm bodies for integer cases
            let mut case_idx = 0;
            for arm in arms {
                if let Pattern::Literal(Literal::Int(_)) = &arm.pattern.node {
                    let label = &arm_labels[case_idx];
                    writeln!(ir, "{}:", label).unwrap();

                    // Check guard if present
                    if let Some(guard) = &arm.guard {
                        let guard_pass = self.next_label("match.guard.pass");
                        let guard_fail = self.next_label("match.guard.fail");

                        let (guard_val, guard_ir) = self.generate_expr(guard, counter)?;
                        ir.push_str(&guard_ir);
                        writeln!(
                            ir,
                            "  br i1 {}, label %{}, label %{}",
                            guard_val, guard_pass, guard_fail
                        )
                        .unwrap();

                        // Guard passed - execute body
                        writeln!(ir, "{}:", guard_pass).unwrap();
                        self.fn_ctx.current_block = guard_pass.clone();
                        let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                        ir.push_str(&body_ir);
                        arm_values.push((body_val, self.fn_ctx.current_block.clone()));
                        writeln!(ir, "  br label %{}", merge_label).unwrap();

                        // Guard failed - go to default
                        writeln!(ir, "{}:", guard_fail).unwrap();
                        writeln!(ir, "  br label %{}", default_label).unwrap();
                    } else {
                        self.fn_ctx.current_block.clone_from(label);
                        let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                        ir.push_str(&body_ir);
                        arm_values.push((body_val, self.fn_ctx.current_block.clone()));
                        writeln!(ir, "  br label %{}", merge_label).unwrap();
                    }

                    case_idx += 1;
                }
            }

            // Generate default arm
            writeln!(ir, "{}:", default_label).unwrap();
            if let Some(arm) = default_arm {
                let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                ir.push_str(&body_ir);
                arm_values.push((body_val, default_label.clone()));
            } else {
                // No default arm - unreachable or return 0
                arm_values.push(("0".to_string(), default_label.clone()));
            }
            writeln!(ir, "  br label %{}", merge_label).unwrap();
        } else {
            // Fall back to chained conditional branches for complex patterns
            let mut current_label = self.next_label("match.check");
            // Create a default fallthrough block so all check-fail paths have a phi entry
            let default_label = self.next_label("match.default");
            writeln!(ir, "  br label %{}", current_label).unwrap();

            for (i, arm) in arms.iter().enumerate() {
                let is_last = i == arms.len() - 1;
                let next_label = if is_last {
                    default_label.clone()
                } else {
                    self.next_label("match.check")
                };
                let arm_body_label = self.next_label("match.arm");

                writeln!(ir, "{}:", current_label).unwrap();

                // Generate pattern check — pass match_type for correct nested tuple type inference
                let (check_val, check_ir) = self.generate_pattern_check_typed(
                    &arm.pattern,
                    &match_val,
                    counter,
                    &match_type,
                )?;
                ir.push_str(&check_ir);

                // Handle guard - need to bind variables first so guard can use them
                if let Some(guard) = &arm.guard {
                    let guard_bind = self.next_label("match.guard.bind");
                    let guard_check = self.next_label("match.guard.check");

                    // First check pattern
                    writeln!(
                        ir,
                        "  br i1 {}, label %{}, label %{}",
                        check_val, guard_bind, next_label
                    )
                    .unwrap();

                    // Bind pattern variables for guard to use
                    writeln!(ir, "{}:", guard_bind).unwrap();
                    let bind_ir = self.generate_pattern_bindings_typed(
                        &arm.pattern,
                        &match_val,
                        counter,
                        &match_type,
                    )?;
                    ir.push_str(&bind_ir);
                    writeln!(ir, "  br label %{}", guard_check).unwrap();

                    // Then check guard
                    writeln!(ir, "{}:", guard_check).unwrap();
                    let (guard_val, guard_ir) = self.generate_expr(guard, counter)?;
                    ir.push_str(&guard_ir);
                    // Guard value is i64 (0 or 1), convert to i1 for branch
                    let guard_bool = self.next_temp(counter);
                    writeln!(ir, "  {} = icmp ne i64 {}, 0", guard_bool, guard_val).unwrap();
                    writeln!(
                        ir,
                        "  br i1 {}, label %{}, label %{}",
                        guard_bool, arm_body_label, next_label
                    )
                    .unwrap();

                    // Generate arm body (bindings already done)
                    writeln!(ir, "{}:", arm_body_label).unwrap();
                    self.fn_ctx.current_block.clone_from(&arm_body_label);
                } else {
                    writeln!(
                        ir,
                        "  br i1 {}, label %{}, label %{}",
                        check_val, arm_body_label, next_label
                    )
                    .unwrap();

                    // Generate arm body
                    writeln!(ir, "{}:", arm_body_label).unwrap();
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

                let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                ir.push_str(&body_ir);
                // Use actual current block (may differ from arm_body_label if body
                // inserted intermediate labels, e.g., division-by-zero guard)
                let actual_block = self.fn_ctx.current_block.clone();
                arm_values.push((body_val, actual_block));
                writeln!(ir, "  br label %{}", merge_label).unwrap();

                current_label = next_label;
            }

            // Default fallthrough block (no arm matched)
            writeln!(ir, "{}:", default_label).unwrap();
            // Use appropriate default value based on first arm's body type
            let default_val = if !arms.is_empty() {
                let arm_body_type = self.infer_expr_type(&arms[0].body);
                match &arm_body_type {
                    ResolvedType::Named { .. }
                    | ResolvedType::Str
                    | ResolvedType::Ref(_)
                    | ResolvedType::RefMut(_) => "null".to_string(),
                    ResolvedType::F64 => "0.0".to_string(),
                    ResolvedType::Bool => "false".to_string(),
                    _ => "0".to_string(),
                }
            } else {
                "0".to_string()
            };
            arm_values.push((default_val, default_label.clone()));
            writeln!(ir, "  br label %{}", merge_label).unwrap();
        }

        // Merge block with phi node
        writeln!(ir, "{}:", merge_label).unwrap();

        if arm_values.is_empty() {
            Ok(("0".to_string(), ir))
        } else {
            // Determine phi node type from the first arm's body expression type
            let arm_body_type = if !arms.is_empty() {
                self.infer_expr_type(&arms[0].body)
            } else {
                ResolvedType::I64
            };

            let is_named_type = matches!(&arm_body_type, ResolvedType::Named { .. });
            let phi_type = match &arm_body_type {
                ResolvedType::Named { .. } => {
                    // Enum/struct types are returned as pointers in text codegen
                    let llvm_ty = self.type_to_llvm(&arm_body_type);
                    format!("{}*", llvm_ty)
                }
                ResolvedType::Str => "i8*".to_string(),
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                    let inner_ty = self.type_to_llvm(inner);
                    format!("{}*", inner_ty)
                }
                ResolvedType::F64 => "double".to_string(),
                ResolvedType::Bool => "i1".to_string(),
                _ => "i64".to_string(),
            };

            let phi_result = self.next_temp(counter);
            let phi_args: Vec<String> = arm_values
                .iter()
                .map(|(val, label)| format!("[ {}, %{} ]", val, label))
                .collect();
            writeln!(
                ir,
                "  {} = phi {} {}",
                phi_result,
                phi_type,
                phi_args.join(", ")
            )
            .unwrap();

            // For Named types (enum/struct), the phi gives us a pointer.
            // Load the value so it can be used directly (e.g., as a return value).
            if is_named_type {
                let llvm_ty = self.type_to_llvm(&arm_body_type);
                let loaded = self.next_temp(counter);
                writeln!(
                    ir,
                    "  {} = load {}, {}* {}",
                    loaded, llvm_ty, llvm_ty, phi_result
                )
                .unwrap();
                Ok((loaded, ir))
            } else {
                Ok((phi_result, ir))
            }
        }
    }
}
