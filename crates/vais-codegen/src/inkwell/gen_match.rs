//! Pattern matching code generation.
//!
//! Handles match expressions, pattern checking, pattern bindings,
//! and assertions.

use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, IntValue};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};

use vais_ast::{Expr, Literal, MatchArm, Pattern, Spanned};

use super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn infer_struct_name(&self, expr: &Expr) -> CodegenResult<String> {
        match expr {
            Expr::Ident(name) => {
                // Look up in var_struct_types
                if let Some(struct_name) = self.var_struct_types.get(name) {
                    return Ok(struct_name.clone());
                }
                Err(CodegenError::Unsupported(format!(
                    "Cannot infer struct type for variable: {}. Consider adding type annotations.",
                    name
                )))
            }
            Expr::StructLit { name, .. } => Ok(name.node.clone()),
            Expr::Field { expr: inner, field } => {
                // Recursively infer the inner expression's struct type,
                // then look up the field's type (if it's also a struct)
                let parent_struct = self.infer_struct_name(&inner.node)?;
                // Look up the field's type name from struct_field_type_names
                if let Some(field_types) = self.struct_field_type_names.get(&parent_struct) {
                    for (fname, ftype) in field_types {
                        if fname == &field.node && !ftype.is_empty() {
                            // Check if this type name is a known struct
                            if self.generated_structs.contains_key(ftype) {
                                return Ok(ftype.clone());
                            }
                        }
                    }
                }
                Err(CodegenError::Unsupported(format!(
                    "Cannot infer struct type for nested field access: {}.{}",
                    parent_struct, field.node
                )))
            }
            Expr::Call { func, .. } => {
                // Try to infer return type as struct from function name
                if let Expr::Ident(fn_name) = &func.node {
                    // Check if the function name matches a struct constructor pattern
                    if self.generated_structs.contains_key(fn_name.as_str()) {
                        return Ok(fn_name.clone());
                    }
                }
                Err(CodegenError::Unsupported(
                    "Cannot infer struct type for call expression".to_string(),
                ))
            }
            Expr::SelfCall => {
                // @ in a method context refers to the current struct instance
                // Look up "self" in var_struct_types
                if let Some(struct_name) = self.var_struct_types.get("self") {
                    return Ok(struct_name.clone());
                }
                Err(CodegenError::Unsupported(
                    "SelfCall (@) used outside of method context".to_string(),
                ))
            }
            _ => Err(CodegenError::Unsupported(format!(
                "Cannot infer struct type for expression: {:?}",
                expr
            ))),
        }
    }

    /// Gets the field index by name for a struct.
    pub(super) fn get_field_index(
        &self,
        struct_name: &str,
        field_name: &str,
    ) -> CodegenResult<u32> {
        if let Some(fields) = self.struct_fields.get(struct_name) {
            for (idx, name) in fields.iter().enumerate() {
                if name == field_name {
                    return Ok(idx as u32);
                }
            }
            Err(CodegenError::UndefinedVar(format!(
                "Field '{}' not found in struct '{}'",
                field_name, struct_name
            )))
        } else {
            Err(CodegenError::UndefinedVar(format!(
                "Struct '{}' not found",
                struct_name
            )))
        }
    }

    /// Generates a fresh label name.
    pub(super) fn fresh_label(&mut self, prefix: &str) -> String {
        let label = format!("{}{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Returns a default/zero value for a given LLVM type.
    pub(super) fn get_default_value(&self, ty: BasicTypeEnum<'ctx>) -> BasicValueEnum<'ctx> {
        match ty {
            BasicTypeEnum::IntType(it) => it.const_int(0, false).into(),
            BasicTypeEnum::FloatType(ft) => ft.const_float(0.0).into(),
            BasicTypeEnum::PointerType(pt) => pt.const_null().into(),
            BasicTypeEnum::StructType(st) => st.const_zero().into(),
            BasicTypeEnum::ArrayType(at) => at.const_zero().into(),
            BasicTypeEnum::VectorType(vt) => vt.const_zero().into(),
        }
    }

    // ========== Match Expression ==========

    pub(super) fn generate_match(
        &mut self,
        match_expr: &Spanned<Expr>,
        arms: &[MatchArm],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self.current_function.ok_or_else(|| {
            CodegenError::LlvmError("No current function for match expression".to_string())
        })?;

        // Generate the expression to match against
        let match_val = self.generate_expr(&match_expr.node)?;

        // Create merge block
        let merge_block = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("match.merge"));

        // Track arm results for phi node: (value, block)
        let mut arm_results: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> =
            Vec::new();

        // Check if all arms are simple integer literals (can use switch)
        let all_int_literals = arms.iter().all(|arm| {
            matches!(
                &arm.pattern.node,
                Pattern::Literal(Literal::Int(_)) | Pattern::Wildcard
            )
        });

        if all_int_literals && !arms.is_empty() && match_val.is_int_value() {
            // Use LLVM switch instruction for integer pattern matching
            let default_block = self
                .context
                .append_basic_block(fn_value, &self.fresh_label("match.default"));

            // Collect cases and find default arm
            let mut cases: Vec<(IntValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> =
                Vec::new();
            let mut default_arm: Option<&MatchArm> = None;
            let mut case_arms: Vec<(&MatchArm, inkwell::basic_block::BasicBlock<'ctx>)> =
                Vec::new();

            for arm in arms {
                match &arm.pattern.node {
                    Pattern::Literal(Literal::Int(n)) => {
                        let arm_block = self
                            .context
                            .append_basic_block(fn_value, &self.fresh_label("match.arm"));
                        let case_val = self.context.i64_type().const_int(*n as u64, true);
                        cases.push((case_val, arm_block));
                        case_arms.push((arm, arm_block));
                    }
                    Pattern::Wildcard => {
                        default_arm = Some(arm);
                    }
                    _ => {}
                }
            }

            // Build switch instruction
            let switch_val = match_val.into_int_value();
            let switch = self
                .builder
                .build_switch(switch_val, default_block, &cases)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            let _ = switch; // Suppress unused variable warning

            // Generate arm bodies for integer cases
            for (arm, arm_block) in case_arms {
                self.builder.position_at_end(arm_block);

                // Handle guard if present
                if let Some(guard) = &arm.guard {
                    let guard_pass = self
                        .context
                        .append_basic_block(fn_value, &self.fresh_label("match.guard.pass"));
                    let guard_fail = self
                        .context
                        .append_basic_block(fn_value, &self.fresh_label("match.guard.fail"));

                    let guard_val = self.generate_expr(&guard.node)?;
                    let guard_bool = if guard_val.is_int_value() {
                        guard_val.into_int_value()
                    } else {
                        self.context.bool_type().const_int(1, false) // Truthy fallback
                    };

                    self.builder
                        .build_conditional_branch(guard_bool, guard_pass, guard_fail)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Guard passed - execute body
                    self.builder.position_at_end(guard_pass);
                    let body_val = self.generate_expr(&arm.body.node)?;
                    let body_end = self.builder.get_insert_block().unwrap();
                    if body_end.get_terminator().is_none() {
                        self.builder
                            .build_unconditional_branch(merge_block)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        arm_results.push((body_val, body_end));
                    }

                    // Guard failed - go to default
                    self.builder.position_at_end(guard_fail);
                    self.builder
                        .build_unconditional_branch(default_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                } else {
                    let body_val = self.generate_expr(&arm.body.node)?;
                    let body_end = self.builder.get_insert_block().unwrap();
                    if body_end.get_terminator().is_none() {
                        self.builder
                            .build_unconditional_branch(merge_block)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        arm_results.push((body_val, body_end));
                    }
                }
            }

            // Generate default arm
            self.builder.position_at_end(default_block);
            if let Some(arm) = default_arm {
                let body_val = self.generate_expr(&arm.body.node)?;
                let default_end = self.builder.get_insert_block().unwrap();
                if default_end.get_terminator().is_none() {
                    self.builder
                        .build_unconditional_branch(merge_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    arm_results.push((body_val, default_end));
                }
            } else {
                // No default arm - return default value (0)
                let default_val = self.context.i64_type().const_int(0, false);
                self.builder
                    .build_unconditional_branch(merge_block)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                arm_results.push((default_val.into(), default_block));
            }
        } else {
            // Fall back to chained conditional branches for complex patterns
            let mut current_block = self.builder.get_insert_block().unwrap();

            for (i, arm) in arms.iter().enumerate() {
                let is_last = i == arms.len() - 1;
                // For the last arm, create a fallthrough block instead of using merge_block
                // directly. This prevents invalid phi nodes when the last arm's check has
                // a false branch that reaches merge_block without a phi entry.
                let next_block = if is_last {
                    let fallthrough = self
                        .context
                        .append_basic_block(fn_value, &self.fresh_label("match.fallthrough"));
                    fallthrough
                } else {
                    self.context
                        .append_basic_block(fn_value, &self.fresh_label("match.check"))
                };
                let arm_body_block = self
                    .context
                    .append_basic_block(fn_value, &self.fresh_label("match.arm"));

                self.builder.position_at_end(current_block);

                // Generate pattern check
                let check_result = self.generate_pattern_check(&arm.pattern, &match_val)?;

                // Handle guard
                if let Some(guard) = &arm.guard {
                    let guard_bind_block = self
                        .context
                        .append_basic_block(fn_value, &self.fresh_label("match.guard.bind"));
                    let guard_check_block = self
                        .context
                        .append_basic_block(fn_value, &self.fresh_label("match.guard.check"));

                    // First check pattern
                    self.builder
                        .build_conditional_branch(check_result, guard_bind_block, next_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Bind pattern variables for guard to use
                    self.builder.position_at_end(guard_bind_block);
                    self.generate_pattern_bindings(&arm.pattern, &match_val)?;
                    self.builder
                        .build_unconditional_branch(guard_check_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Then check guard
                    self.builder.position_at_end(guard_check_block);
                    let guard_val = self.generate_expr(&guard.node)?;
                    let guard_bool = if guard_val.is_int_value() {
                        let int_val = guard_val.into_int_value();
                        // Convert i64 to i1 if needed
                        if int_val.get_type().get_bit_width() > 1 {
                            self.builder
                                .build_int_compare(
                                    IntPredicate::NE,
                                    int_val,
                                    self.context.i64_type().const_int(0, false),
                                    "guard_bool",
                                )
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                        } else {
                            int_val
                        }
                    } else {
                        self.context.bool_type().const_int(1, false)
                    };
                    self.builder
                        .build_conditional_branch(guard_bool, arm_body_block, next_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Generate arm body (bindings already done)
                    self.builder.position_at_end(arm_body_block);
                } else {
                    // No guard - branch based on pattern check
                    self.builder
                        .build_conditional_branch(check_result, arm_body_block, next_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Generate arm body
                    self.builder.position_at_end(arm_body_block);

                    // Bind pattern variables if needed
                    self.generate_pattern_bindings(&arm.pattern, &match_val)?;
                }

                let body_val = self.generate_expr(&arm.body.node)?;
                let body_end_block = self.builder.get_insert_block().unwrap();
                if body_end_block.get_terminator().is_none() {
                    self.builder
                        .build_unconditional_branch(merge_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    arm_results.push((body_val, body_end_block));
                }

                // For the last arm's fallthrough block, add a default value and jump to merge
                if is_last {
                    self.builder.position_at_end(next_block);
                    let default_val = self.context.i64_type().const_int(0, false);
                    self.builder
                        .build_unconditional_branch(merge_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    arm_results.push((default_val.into(), next_block));
                }

                current_block = next_block;
            }
        }

        // Merge block with phi node
        self.builder.position_at_end(merge_block);

        if arm_results.is_empty() {
            // All arms terminated (return/break) - merge is unreachable
            self.builder
                .build_unreachable()
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(self.context.struct_type(&[], false).const_zero().into())
        } else if arm_results.len() == 1 {
            // Only one arm reaches merge - no phi needed
            Ok(arm_results[0].0)
        } else {
            // Build phi node
            let first_type = arm_results[0].0.get_type();
            let phi = self
                .builder
                .build_phi(first_type, "match_result")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            for (val, block) in &arm_results {
                phi.add_incoming(&[(val, *block)]);
            }

            Ok(phi.as_basic_value())
        }
    }

    /// Generates code to check if a pattern matches the given value.
    /// Returns an i1 (boolean) value.
    pub(super) fn generate_pattern_check(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &BasicValueEnum<'ctx>,
    ) -> CodegenResult<IntValue<'ctx>> {
        match &pattern.node {
            Pattern::Wildcard => {
                // Wildcard always matches
                Ok(self.context.bool_type().const_int(1, false))
            }
            Pattern::Ident(name) => {
                // Check if this identifier is a known enum variant (simple variant without data)
                let is_enum_variant = self.enum_variants.iter().any(|((_, v), _)| v == name);
                if is_enum_variant && match_val.is_struct_value() {
                    // Compare the tag value
                    let struct_val = match_val.into_struct_value();
                    let tag_val = self
                        .builder
                        .build_extract_value(struct_val, 0, "enum_tag")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                        .into_int_value();
                    let expected_tag = self.get_enum_variant_tag(name);
                    let result = self
                        .builder
                        .build_int_compare(
                            IntPredicate::EQ,
                            tag_val,
                            self.context.i8_type().const_int(expected_tag as u64, false),
                            "variant_check",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    Ok(result)
                } else {
                    // Regular binding - always matches
                    Ok(self.context.bool_type().const_int(1, false))
                }
            }
            Pattern::Literal(lit) => {
                match lit {
                    Literal::Int(n) => {
                        let lit_val = self.context.i64_type().const_int(*n as u64, true);
                        let cmp = self
                            .builder
                            .build_int_compare(
                                IntPredicate::EQ,
                                match_val.into_int_value(),
                                lit_val,
                                "pat_eq",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(cmp)
                    }
                    Literal::Bool(b) => {
                        let lit_val = self.context.bool_type().const_int(*b as u64, false);
                        let match_int = match_val.into_int_value();
                        // Convert to same bit width if needed
                        let cmp = self
                            .builder
                            .build_int_compare(IntPredicate::EQ, match_int, lit_val, "pat_eq")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(cmp)
                    }
                    Literal::Float(f) => {
                        let lit_val = self.context.f64_type().const_float(*f);
                        let cmp = self
                            .builder
                            .build_float_compare(
                                FloatPredicate::OEQ,
                                match_val.into_float_value(),
                                lit_val,
                                "pat_eq",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(cmp)
                    }
                    Literal::String(s) => {
                        // String comparison using strcmp
                        // First, create the pattern string constant
                        let pattern_str = self.generate_string_literal(s)?;

                        // Get strcmp function
                        let strcmp_fn = self.module.get_function("strcmp").unwrap_or_else(|| {
                            let i32_type = self.context.i32_type();
                            let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
                            let fn_type =
                                i32_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
                            self.module.add_function("strcmp", fn_type, None)
                        });

                        // Call strcmp
                        let cmp_result = self
                            .builder
                            .build_call(
                                strcmp_fn,
                                &[(*match_val).into(), pattern_str.into()],
                                "strcmp_result",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                        let cmp_int = cmp_result
                            .try_as_basic_value()
                            .left()
                            .ok_or_else(|| {
                                CodegenError::LlvmError("strcmp returned void".to_string())
                            })?
                            .into_int_value();

                        // Check if strcmp returned 0 (equal)
                        let result = self
                            .builder
                            .build_int_compare(
                                IntPredicate::EQ,
                                cmp_int,
                                self.context.i32_type().const_int(0, false),
                                "str_eq",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                        Ok(result)
                    }
                }
            }
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let mut lower_check = self.context.bool_type().const_int(1, false);
                let mut upper_check = self.context.bool_type().const_int(1, false);

                // Check lower bound
                if let Some(start_pat) = start {
                    if let Pattern::Literal(Literal::Int(n)) = &start_pat.node {
                        let start_val = self.context.i64_type().const_int(*n as u64, true);
                        lower_check = self
                            .builder
                            .build_int_compare(
                                IntPredicate::SGE,
                                match_val.into_int_value(),
                                start_val,
                                "range_lower",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }

                // Check upper bound
                if let Some(end_pat) = end {
                    if let Pattern::Literal(Literal::Int(n)) = &end_pat.node {
                        let end_val = self.context.i64_type().const_int(*n as u64, true);
                        let cmp = if *inclusive {
                            IntPredicate::SLE
                        } else {
                            IntPredicate::SLT
                        };
                        upper_check = self
                            .builder
                            .build_int_compare(
                                cmp,
                                match_val.into_int_value(),
                                end_val,
                                "range_upper",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }

                // Combine checks
                let result = self
                    .builder
                    .build_and(lower_check, upper_check, "range_check")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                Ok(result)
            }
            Pattern::Or(patterns) => {
                if patterns.is_empty() {
                    return Ok(self.context.bool_type().const_int(0, false));
                }

                let mut result = self.generate_pattern_check(&patterns[0], match_val)?;
                for pat in patterns.iter().skip(1) {
                    let check = self.generate_pattern_check(pat, match_val)?;
                    result = self
                        .builder
                        .build_or(result, check, "or_pat")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }

                Ok(result)
            }
            Pattern::Tuple(patterns) => {
                if patterns.is_empty() {
                    return Ok(self.context.bool_type().const_int(1, false));
                }

                let struct_val = match_val.into_struct_value();
                let mut result = self.context.bool_type().const_int(1, false);

                for (i, pat) in patterns.iter().enumerate() {
                    let elem_val = self
                        .builder
                        .build_extract_value(struct_val, i as u32, &format!("tuple_{}", i))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    let check = self.generate_pattern_check(pat, &elem_val)?;
                    result = self
                        .builder
                        .build_and(result, check, "tuple_check")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }

                Ok(result)
            }
            Pattern::Variant { name, fields: _ } => {
                // Enum variant pattern: check the tag matches
                // Enum is represented as { i8 tag, i64 data }
                let struct_val = match_val.into_struct_value();
                let tag_val = self
                    .builder
                    .build_extract_value(struct_val, 0, "enum_tag")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_int_value();

                // Find expected tag value
                let expected_tag = self.get_enum_variant_tag(&name.node);

                let result = self
                    .builder
                    .build_int_compare(
                        IntPredicate::EQ,
                        tag_val,
                        self.context.i8_type().const_int(expected_tag as u64, false),
                        "variant_check",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                Ok(result)
            }
            Pattern::Struct { name, fields } => {
                // Struct pattern: check field patterns
                let struct_name = &name.node;
                let struct_val = match_val.into_struct_value();
                let mut result = self.context.bool_type().const_int(1, false);

                for (field_name, field_pat) in fields {
                    if let Some(pat) = field_pat {
                        // Get field index
                        let field_idx = self.get_field_index(struct_name, &field_name.node)?;
                        let field_val = self
                            .builder
                            .build_extract_value(struct_val, field_idx, &field_name.node)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        let check = self.generate_pattern_check(pat, &field_val)?;
                        result = self
                            .builder
                            .build_and(result, check, "struct_check")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }

                Ok(result)
            }
        }
    }

    /// Generates code to bind pattern variables to their matched values.
    pub(super) fn generate_pattern_bindings(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &BasicValueEnum<'ctx>,
    ) -> CodegenResult<()> {
        match &pattern.node {
            Pattern::Wildcard => {
                // Nothing to bind
                Ok(())
            }
            Pattern::Ident(name) => {
                // Bind identifier to the matched value
                let var_type = match_val.get_type();
                let alloca = self
                    .builder
                    .build_alloca(var_type, name)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder
                    .build_store(alloca, *match_val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.locals.insert(name.clone(), (alloca, var_type));
                Ok(())
            }
            Pattern::Literal(_) => {
                // Literals don't bind anything
                Ok(())
            }
            Pattern::Range { .. } => {
                // Ranges don't bind anything
                Ok(())
            }
            Pattern::Or(patterns) => {
                // For or patterns, bind the first pattern (all alternatives should bind the same names)
                if let Some(first) = patterns.first() {
                    self.generate_pattern_bindings(first, match_val)?;
                }
                Ok(())
            }
            Pattern::Tuple(patterns) => {
                let struct_val = match_val.into_struct_value();
                for (i, pat) in patterns.iter().enumerate() {
                    let elem_val = self
                        .builder
                        .build_extract_value(struct_val, i as u32, &format!("tuple_{}", i))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    self.generate_pattern_bindings(pat, &elem_val)?;
                }
                Ok(())
            }
            Pattern::Variant { name: _, fields } => {
                // Bind variant fields
                // Enum is { i8 tag, i64 data } - extract data and bind
                let struct_val = match_val.into_struct_value();
                let data_val = self
                    .builder
                    .build_extract_value(struct_val, 1, "variant_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // For now, assume single field variant
                if let Some(first_field) = fields.first() {
                    self.generate_pattern_bindings(first_field, &data_val)?;
                }
                Ok(())
            }
            Pattern::Struct { name, fields } => {
                let struct_name = &name.node;
                let struct_val = match_val.into_struct_value();

                for (field_name, field_pat) in fields {
                    let field_idx = self.get_field_index(struct_name, &field_name.node)?;
                    let field_val = self
                        .builder
                        .build_extract_value(struct_val, field_idx, &field_name.node)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    if let Some(pat) = field_pat {
                        // Pattern specified - bind according to pattern
                        self.generate_pattern_bindings(pat, &field_val)?;
                    } else {
                        // Shorthand: `{x}` means `{x: x}`
                        let var_type = field_val.get_type();
                        let alloca = self
                            .builder
                            .build_alloca(var_type, &field_name.node)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        self.builder
                            .build_store(alloca, field_val)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        self.locals
                            .insert(field_name.node.clone(), (alloca, var_type));
                    }
                }
                Ok(())
            }
        }
    }

    /// Gets the tag value for an enum variant by searching all registered enums.
    ///
    /// This method searches through all registered enum variants to find the tag
    /// for the given variant name. If multiple enums have the same variant name,
    /// the first match is returned.
    pub(super) fn get_enum_variant_tag(&self, variant_name: &str) -> i32 {
        // Search through all registered enum variants
        for ((_, v_name), tag) in &self.enum_variants {
            if v_name == variant_name {
                return *tag;
            }
        }
        // Variant not found - this could happen for built-in types like Option/Result
        // In such cases, we use a simple heuristic: Some=1, None=0, Ok=0, Err=1
        match variant_name {
            "None" => 0,
            "Some" => 1,
            "Ok" => 0,
            "Err" => 1,
            _ => 0, // Default to 0 for unknown variants
        }
    }

    /// Gets the tag value for an enum variant with explicit enum name.
    ///
    /// This method provides more precise lookup when the enum name is known.
    #[allow(dead_code)]
    pub(super) fn get_enum_variant_tag_with_enum(
        &self,
        enum_name: &str,
        variant_name: &str,
    ) -> i32 {
        self.enum_variants
            .get(&(enum_name.to_string(), variant_name.to_string()))
            .copied()
            .unwrap_or_else(|| self.get_enum_variant_tag(variant_name))
    }

    // ========== Assert ==========

    pub(super) fn generate_assert(
        &mut self,
        condition: &Expr,
        _message: Option<&Spanned<Expr>>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self
            .current_function
            .ok_or_else(|| CodegenError::LlvmError("No current function for assert".to_string()))?;

        let cond_val = self.generate_expr(condition)?;
        let cond_bool = if cond_val.is_int_value() {
            let int_val = cond_val.into_int_value();
            if int_val.get_type().get_bit_width() > 1 {
                self.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        int_val,
                        int_val.get_type().const_int(0, false),
                        "assert_cond",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                int_val
            }
        } else {
            self.context.bool_type().const_int(1, false)
        };

        let pass_block = self.context.append_basic_block(fn_value, "assert_pass");
        let fail_block = self.context.append_basic_block(fn_value, "assert_fail");

        self.builder
            .build_conditional_branch(cond_bool, pass_block, fail_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Fail: call abort
        self.builder.position_at_end(fail_block);
        if let Some(abort_fn) = self.module.get_function("abort") {
            self.builder
                .build_call(abort_fn, &[], "")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }
        self.builder
            .build_unreachable()
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Pass: continue
        self.builder.position_at_end(pass_block);
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    // ========== String Interpolation ==========
}
