//! Control flow code generation for Vais
//!
//! This module contains functions for generating LLVM IR for control flow
//! constructs: if-else, match expressions, and pattern matching.

use crate::types::LocalVar;
use crate::{CodeGenerator, CodegenResult};
use std::fmt::Write;
use vais_ast::{Expr, IfElse, Literal, MatchArm, Pattern, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Generate code for if-else branches.
    /// Legacy API for backward compatibility; use generate_if_else_with_term for new code.
    #[allow(dead_code)]
    pub(crate) fn generate_if_else(
        &mut self,
        if_else: &IfElse,
        counter: &mut usize,
        merge_label: &str,
    ) -> CodegenResult<(String, String)> {
        let (val, ir, _terminated, _last_block) =
            self.generate_if_else_with_term(if_else, counter, merge_label)?;
        Ok((val, ir))
    }

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

                writeln!(ir, "  br i1 {}, label %{}, label %{}", cond_bool, then_label, else_label).unwrap();

                // Then branch
                writeln!(ir, "{}:", then_label).unwrap();
                self.fn_ctx.current_block = then_label; // move: label not used after
                let (then_val, then_ir, then_terminated) =
                    self.generate_block_stmts(then_stmts, counter)?;
                ir.push_str(&then_ir);

                // For struct results, load the value before branch if it's a pointer
                let then_val_for_phi = if is_struct_result && !then_terminated {
                    let loaded = self.next_temp(counter);
                    writeln!(ir, "  {} = load {}, {}* {}", loaded, llvm_type, llvm_type, then_val).unwrap();
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
                    writeln!(ir, "  {} = load {}, {}* {}", loaded, llvm_type, llvm_type, else_val).unwrap();
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
                    writeln!(ir, "  {} = phi {} [ {}, %{} ], [ {}, %{} ]", result, llvm_type, then_val_for_phi, then_from_label, else_val_for_phi, else_from_label).unwrap();
                } else if !then_from_label.is_empty() {
                    writeln!(ir, "  {} = phi {} [ {}, %{} ]", result, llvm_type, then_val_for_phi, then_from_label).unwrap();
                } else if !else_from_label.is_empty() {
                    writeln!(ir, "  {} = phi {} [ {}, %{} ]", result, llvm_type, else_val_for_phi, else_from_label).unwrap();
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
            writeln!(ir, "  store {} {}, {}* {}", llvm_type, match_val_raw, llvm_type, stack_ptr).unwrap();
            stack_ptr
        } else {
            match_val_raw
        };

        let merge_label = self.next_label("match.merge");
        let mut arm_labels: Vec<String> = Vec::new();
        let mut arm_values: Vec<(String, String)> = Vec::new(); // (value, label)

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
            let mut switch_cases: Vec<(i64, String)> = Vec::new();
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
                        writeln!(ir, "  br i1 {}, label %{}, label %{}", guard_val, guard_pass, guard_fail).unwrap();

                        // Guard passed - execute body
                        writeln!(ir, "{}:", guard_pass).unwrap();
                        let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                        ir.push_str(&body_ir);
                        arm_values.push((body_val, guard_pass.clone()));
                        writeln!(ir, "  br label %{}", merge_label).unwrap();

                        // Guard failed - go to default
                        writeln!(ir, "{}:", guard_fail).unwrap();
                        writeln!(ir, "  br label %{}", default_label).unwrap();
                    } else {
                        let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                        ir.push_str(&body_ir);
                        arm_values.push((body_val, label.clone()));
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

                // Generate pattern check
                let (check_val, check_ir) =
                    self.generate_pattern_check(&arm.pattern, &match_val, counter)?;
                ir.push_str(&check_ir);

                // Handle guard - need to bind variables first so guard can use them
                if let Some(guard) = &arm.guard {
                    let guard_bind = self.next_label("match.guard.bind");
                    let guard_check = self.next_label("match.guard.check");

                    // First check pattern
                    writeln!(ir, "  br i1 {}, label %{}, label %{}", check_val, guard_bind, next_label).unwrap();

                    // Bind pattern variables for guard to use
                    writeln!(ir, "{}:", guard_bind).unwrap();
                    let bind_ir =
                        self.generate_pattern_bindings_typed(&arm.pattern, &match_val, counter, &match_type)?;
                    ir.push_str(&bind_ir);
                    writeln!(ir, "  br label %{}", guard_check).unwrap();

                    // Then check guard
                    writeln!(ir, "{}:", guard_check).unwrap();
                    let (guard_val, guard_ir) = self.generate_expr(guard, counter)?;
                    ir.push_str(&guard_ir);
                    // Guard value is i64 (0 or 1), convert to i1 for branch
                    let guard_bool = self.next_temp(counter);
                    writeln!(ir, "  {} = icmp ne i64 {}, 0", guard_bool, guard_val).unwrap();
                    writeln!(ir, "  br i1 {}, label %{}, label %{}", guard_bool, arm_body_label, next_label).unwrap();

                    // Generate arm body (bindings already done)
                    writeln!(ir, "{}:", arm_body_label).unwrap();
                } else {
                    writeln!(ir, "  br i1 {}, label %{}, label %{}", check_val, arm_body_label, next_label).unwrap();

                    // Generate arm body
                    writeln!(ir, "{}:", arm_body_label).unwrap();

                    // Bind pattern variables if needed
                    let bind_ir =
                        self.generate_pattern_bindings_typed(&arm.pattern, &match_val, counter, &match_type)?;
                    ir.push_str(&bind_ir);
                }

                let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                ir.push_str(&body_ir);
                arm_values.push((body_val, arm_body_label.clone()));
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
            writeln!(ir, "  {} = phi {} {}", phi_result, phi_type, phi_args.join(", ")).unwrap();

            // For Named types (enum/struct), the phi gives us a pointer.
            // Load the value so it can be used directly (e.g., as a return value).
            if is_named_type {
                let llvm_ty = self.type_to_llvm(&arm_body_type);
                let loaded = self.next_temp(counter);
                writeln!(ir, "  {} = load {}, {}* {}", loaded, llvm_ty, llvm_ty, phi_result).unwrap();
                Ok((loaded, ir))
            } else {
                Ok((phi_result, ir))
            }
        }
    }

    /// Generate code to check if a pattern matches
    pub(crate) fn generate_pattern_check(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &pattern.node {
            Pattern::Wildcard => {
                // Wildcard always matches
                Ok(("1".to_string(), String::new()))
            }
            Pattern::Ident(name) => {
                // Check if this is a unit enum variant (like None, Some without args)
                // If so, we need to check the discriminant, not just match anything
                if self.is_unit_enum_variant(name) {
                    // Generate discriminant check for unit enum variant
                    let mut ir = String::new();

                    // Get the enum type name for proper LLVM type reference
                    let enum_name = self
                        .get_enum_name_for_variant(name)
                        .unwrap_or_else(|| "Unknown".to_string());

                    // Get the tag from the enum value (first field at index 0)
                    let tag_ptr = self.next_temp(counter);
                    writeln!(ir, "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0", tag_ptr, enum_name, enum_name, match_val).unwrap();

                    let tag_val = self.next_temp(counter);
                    writeln!(ir, "  {} = load i32, i32* {}", tag_val, tag_ptr).unwrap();

                    // Find the expected tag value for this variant
                    let expected_tag = self.get_enum_variant_tag(name);

                    // Compare tag
                    let result = self.next_temp(counter);
                    writeln!(ir, "  {} = icmp eq i32 {}, {}", result, tag_val, expected_tag).unwrap();

                    Ok((result, ir))
                } else {
                    // Identifier pattern always matches (binding)
                    Ok(("1".to_string(), String::new()))
                }
            }
            Pattern::Literal(lit) => match lit {
                Literal::Int(n) => {
                    let result = self.next_temp(counter);
                    let ir = format!("  {} = icmp eq i64 {}, {}\n", result, match_val, n);
                    Ok((result, ir))
                }
                Literal::Bool(b) => {
                    let lit_val = if *b { "1" } else { "0" };
                    let result = self.next_temp(counter);
                    let ir = format!("  {} = icmp eq i64 {}, {}\n", result, match_val, lit_val);
                    Ok((result, ir))
                }
                Literal::Float(f) => {
                    let result = self.next_temp(counter);
                    let ir = format!("  {} = fcmp oeq double {}, {:e}\n", result, match_val, f);
                    Ok((result, ir))
                }
                Literal::String(s) => {
                    // String comparison using strcmp
                    let mut ir = String::new();

                    // Create string constant for the pattern
                    let const_name = format!(".str_pat.{}", self.strings.counter);
                    self.strings.counter += 1;
                    self.strings.constants.push((const_name.clone(), s.clone()));

                    // Get pointer to the constant string
                    let str_ptr = self.next_temp(counter);
                    let str_len = s.len() + 1;
                    writeln!(ir, "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i32 0, i32 0", str_ptr, str_len, str_len, const_name).unwrap();

                    // Call strcmp: int strcmp(const char* s1, const char* s2)
                    // Returns 0 if strings are equal
                    let cmp_result = self.next_temp(counter);
                    writeln!(ir, "  {} = call i32 @strcmp(i8* {}, i8* {})", cmp_result, match_val, str_ptr).unwrap();

                    // Check if strcmp returned 0 (equal)
                    let result = self.next_temp(counter);
                    writeln!(ir, "  {} = icmp eq i32 {}, 0", result, cmp_result).unwrap();

                    Ok((result, ir))
                }
            },
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let mut ir = String::new();

                // Check lower bound
                let lower_check = if let Some(start_pat) = start {
                    if let Pattern::Literal(Literal::Int(n)) = &start_pat.node {
                        let tmp = self.next_temp(counter);
                        writeln!(ir, "  {} = icmp sge i64 {}, {}", tmp, match_val, n).unwrap();
                        tmp
                    } else {
                        "1".to_string()
                    }
                } else {
                    "1".to_string()
                };

                // Check upper bound
                let upper_check = if let Some(end_pat) = end {
                    if let Pattern::Literal(Literal::Int(n)) = &end_pat.node {
                        let tmp = self.next_temp(counter);
                        let cmp = if *inclusive { "icmp sle" } else { "icmp slt" };
                        writeln!(ir, "  {} = {} i64 {}, {}", tmp, cmp, match_val, n).unwrap();
                        tmp
                    } else {
                        "1".to_string()
                    }
                } else {
                    "1".to_string()
                };

                // Combine checks
                let result = self.next_temp(counter);
                writeln!(ir, "  {} = and i1 {}, {}", result, lower_check, upper_check).unwrap();

                Ok((result, ir))
            }
            Pattern::Or(patterns) => {
                let mut ir = String::new();
                let mut checks: Vec<String> = Vec::new();

                for pat in patterns {
                    let (check, check_ir) = self.generate_pattern_check(pat, match_val, counter)?;
                    ir.push_str(&check_ir);
                    checks.push(check);
                }

                // OR all checks together
                let mut result = checks[0].clone();
                for check in checks.iter().skip(1) {
                    let tmp = self.next_temp(counter);
                    writeln!(ir, "  {} = or i1 {}, {}", tmp, result, check).unwrap();
                    result = tmp;
                }

                Ok((result, ir))
            }
            Pattern::Tuple(patterns) => {
                // For tuple patterns, we need to extract and check each element
                let mut ir = String::new();
                let mut checks: Vec<String> = Vec::new();

                for (i, pat) in patterns.iter().enumerate() {
                    // Extract tuple element
                    let elem = self.next_temp(counter);
                    writeln!(ir, "  {} = extractvalue {{ {} }} {}, {}", elem, vec!["i64"; patterns.len()].join(", "), match_val, i).unwrap();

                    let (check, check_ir) = self.generate_pattern_check(pat, &elem, counter)?;
                    ir.push_str(&check_ir);
                    checks.push(check);
                }

                // AND all checks together
                if checks.is_empty() {
                    Ok(("1".to_string(), ir))
                } else {
                    let mut result = checks[0].clone();
                    for check in checks.iter().skip(1) {
                        let tmp = self.next_temp(counter);
                        writeln!(ir, "  {} = and i1 {}, {}", tmp, result, check).unwrap();
                        result = tmp;
                    }
                    Ok((result, ir))
                }
            }
            Pattern::Variant { name, fields: _ } => {
                // Enum variant pattern: check the tag matches
                // Enum value is a struct %EnumName { i32 tag, ... payload }
                // Extract the tag and compare
                let mut ir = String::new();
                let variant_name = &name.node;

                // Get the enum type name for proper LLVM type reference
                let enum_name = self
                    .get_enum_name_for_variant(variant_name)
                    .unwrap_or_else(|| "Unknown".to_string());

                // Get the tag from the enum value (first field at index 0)
                let tag_ptr = self.next_temp(counter);
                writeln!(ir, "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0", tag_ptr, enum_name, enum_name, match_val).unwrap();

                let tag_val = self.next_temp(counter);
                writeln!(ir, "  {} = load i32, i32* {}", tag_val, tag_ptr).unwrap();

                // Find the expected tag value for this variant
                let expected_tag = self.get_enum_variant_tag(variant_name);

                // Compare tag
                let result = self.next_temp(counter);
                writeln!(ir, "  {} = icmp eq i32 {}, {}", result, tag_val, expected_tag).unwrap();

                Ok((result, ir))
            }
            Pattern::Struct { name, fields } => {
                // Struct pattern: always matches if type is correct, but we check field patterns
                let struct_name = &name.node;
                let mut ir = String::new();
                let mut checks: Vec<String> = Vec::new();

                if let Some(struct_info) = self.types.structs.get(struct_name).cloned() {
                    for (field_name, field_pat) in fields {
                        // Find field index
                        if let Some(field_idx) = struct_info
                            .fields
                            .iter()
                            .position(|(n, _)| n == &field_name.node)
                        {
                            if let Some(pat) = field_pat {
                                // Extract field value and check pattern
                                let field_ptr = self.next_temp(counter);
                                writeln!(ir, "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}", field_ptr, struct_name, struct_name, match_val, field_idx).unwrap();

                                let field_val = self.next_temp(counter);
                                let field_ty = &struct_info.fields[field_idx].1;
                                let llvm_ty = self.type_to_llvm(field_ty);
                                writeln!(ir, "  {} = load {}, {}* {}", field_val, llvm_ty, llvm_ty, field_ptr).unwrap();

                                let (check, check_ir) =
                                    self.generate_pattern_check(pat, &field_val, counter)?;
                                ir.push_str(&check_ir);
                                checks.push(check);
                            }
                        }
                    }
                }

                // AND all checks together
                if checks.is_empty() {
                    Ok(("1".to_string(), ir))
                } else {
                    let mut result = checks[0].clone();
                    for check in checks.iter().skip(1) {
                        let tmp = self.next_temp(counter);
                        writeln!(ir, "  {} = and i1 {}, {}", tmp, result, check).unwrap();
                        result = tmp;
                    }
                    Ok((result, ir))
                }
            }
            Pattern::Alias { pattern, .. } => {
                // For pattern alias, check the inner pattern
                self.generate_pattern_check(pattern, match_val, counter)
            }
        }
    }

    /// Get the tag value for an enum variant
    pub(crate) fn get_enum_variant_tag(&self, variant_name: &str) -> i32 {
        for enum_info in self.types.enums.values() {
            for (i, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == variant_name {
                    return i as i32;
                }
            }
        }
        0 // Default to 0 if not found
    }

    /// Get the enum name that contains a given variant
    pub(crate) fn get_enum_name_for_variant(&self, variant_name: &str) -> Option<String> {
        for enum_info in self.types.enums.values() {
            for variant in &enum_info.variants {
                if variant.name == variant_name {
                    return Some(enum_info.name.clone());
                }
            }
        }
        None
    }

    /// Check if a name is a unit enum variant (not a binding)
    pub(crate) fn is_unit_enum_variant(&self, name: &str) -> bool {
        use crate::types::EnumVariantFields;
        for enum_info in self.types.enums.values() {
            for variant in &enum_info.variants {
                if variant.name == name {
                    return matches!(variant.fields, EnumVariantFields::Unit);
                }
            }
        }
        false
    }

    /// Check if a name is a tuple enum variant and get its enum name and tag
    pub(crate) fn get_tuple_variant_info(&self, name: &str) -> Option<(String, i32)> {
        use crate::types::EnumVariantFields;
        for enum_info in self.types.enums.values() {
            for (tag, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == name && matches!(variant.fields, EnumVariantFields::Tuple(_)) {
                    return Some((enum_info.name.clone(), tag as i32));
                }
            }
        }
        None
    }

    /// Generate pattern bindings (assign matched values to pattern variables)
    pub(crate) fn generate_pattern_bindings(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &str,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        self.generate_pattern_bindings_typed(pattern, match_val, counter, &ResolvedType::I64)
    }

    /// Generate pattern bindings with explicit match type for correct type propagation.
    /// Called from generate_match where the matched expression type is known.
    pub(crate) fn generate_pattern_bindings_typed(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &str,
        counter: &mut usize,
        match_type: &ResolvedType,
    ) -> CodegenResult<String> {
        match &pattern.node {
            Pattern::Ident(name) => {
                // Check if this is a unit enum variant (like None)
                // Unit variants don't bind anything
                if self.is_unit_enum_variant(name) {
                    return Ok(String::new());
                }

                // Bind the matched value to the identifier
                let ir = String::new();
                let ty = match_type.clone();

                // Generate unique LLVM name for pattern binding
                let _llvm_name = format!("{}.{}", name, counter);
                *counter += 1;

                self.fn_ctx.locals.insert(
                    name.clone(),
                    LocalVar::ssa(ty.clone(), match_val.to_string()),
                );

                // SSA style - no alloca needed, we just alias the match value

                Ok(ir)
            }
            Pattern::Tuple(patterns) => {
                let mut ir = String::new();

                for (i, pat) in patterns.iter().enumerate() {
                    // Extract tuple element
                    let elem = self.next_temp(counter);
                    writeln!(ir, "  {} = extractvalue {{ {} }} {}, {}", elem, vec!["i64"; patterns.len()].join(", "), match_val, i).unwrap();

                    let bind_ir = self.generate_pattern_bindings(pat, &elem, counter)?;
                    ir.push_str(&bind_ir);
                }

                Ok(ir)
            }
            Pattern::Variant { name, fields } => {
                // Bind fields from enum variant payload
                let mut ir = String::new();
                let variant_name = &name.node;

                // Get the enum type name for proper LLVM type reference
                let enum_name = self
                    .get_enum_name_for_variant(variant_name)
                    .unwrap_or_else(|| "Unknown".to_string());

                for (i, field_pat) in fields.iter().enumerate() {
                    // Extract payload field from enum variant
                    // Enum layout: %EnumName = type { i32 tag, { payload_types... } }
                    // First get pointer to payload struct (index 1), then get field within it

                    if match_val.starts_with('%') {
                        // match_val is a pointer to the enum - use getelementptr
                        // Access payload field: first get to the payload (index 1),
                        // then get the specific field within the payload
                        let payload_ptr = self.next_temp(counter);
                        writeln!(ir, "  {} = getelementptr %{}, %{}* {}, i32 0, i32 1, i32 {}", payload_ptr, enum_name, enum_name, match_val, i).unwrap();
                        let field_val = self.next_temp(counter);
                        writeln!(ir, "  {} = load i64, i64* {}", field_val, payload_ptr).unwrap();
                        let bind_ir =
                            self.generate_pattern_bindings(field_pat, &field_val, counter)?;
                        ir.push_str(&bind_ir);
                    } else {
                        // It's a value - use extractvalue
                        // Extract from payload sub-struct: index 1 for payload, then i for field
                        let payload_val = self.next_temp(counter);
                        writeln!(ir, "  {} = extractvalue %{} {}, 1, {}", payload_val, enum_name, match_val, i).unwrap();
                        let bind_ir =
                            self.generate_pattern_bindings(field_pat, &payload_val, counter)?;
                        ir.push_str(&bind_ir);
                    }
                }

                Ok(ir)
            }
            Pattern::Struct { name, fields } => {
                // Bind fields from struct
                let struct_name = &name.node;
                let mut ir = String::new();

                if let Some(struct_info) = self.types.structs.get(struct_name).cloned() {
                    for (field_name, field_pat) in fields {
                        // If field_pat is None, bind the field to its own name
                        if let Some(field_idx) = struct_info
                            .fields
                            .iter()
                            .position(|(n, _)| n == &field_name.node)
                        {
                            // Extract field value
                            let field_ptr = self.next_temp(counter);
                            writeln!(ir, "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}", field_ptr, struct_name, struct_name, match_val, field_idx).unwrap();

                            let field_val = self.next_temp(counter);
                            let field_ty = &struct_info.fields[field_idx].1;
                            let llvm_ty = self.type_to_llvm(field_ty);
                            writeln!(ir, "  {} = load {}, {}* {}", field_val, llvm_ty, llvm_ty, field_ptr).unwrap();

                            if let Some(pat) = field_pat {
                                // Bind to pattern
                                let bind_ir =
                                    self.generate_pattern_bindings(pat, &field_val, counter)?;
                                ir.push_str(&bind_ir);
                            } else {
                                // Bind to field name directly using SSA style
                                self.fn_ctx.locals.insert(
                                    field_name.node.clone(),
                                    LocalVar::ssa(field_ty.clone(), field_val.clone()),
                                );
                                // SSA style - no alloca needed
                            }
                        }
                    }
                }

                Ok(ir)
            }
            Pattern::Alias { name, pattern } => {
                // Bind the whole value to name, then bind variables from inner pattern
                let mut ir = String::new();

                // First, bind the whole matched value to the alias name using the actual match type
                self.fn_ctx.locals.insert(
                    name.clone(),
                    LocalVar::ssa(match_type.clone(), match_val.to_string()),
                );

                // Then bind variables from the inner pattern
                let inner_ir = self.generate_pattern_bindings_typed(pattern, match_val, counter, match_type)?;
                ir.push_str(&inner_ir);

                Ok(ir)
            }
            _ => Ok(String::new()),
        }
    }
}
