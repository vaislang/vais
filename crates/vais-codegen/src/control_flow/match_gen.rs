use super::*;

impl CodeGenerator {
    fn informative_expected_match_type(&self, fallback_ty: &ResolvedType) -> Option<ResolvedType> {
        if !matches!(
            fallback_ty,
            ResolvedType::I64 | ResolvedType::Unknown | ResolvedType::Never
        ) {
            return None;
        }

        let expected = self.fn_ctx.expected_expr_types.last()?;
        if matches!(
            expected,
            ResolvedType::Named { .. }
                | ResolvedType::Str
                | ResolvedType::Ref(_)
                | ResolvedType::RefMut(_)
                | ResolvedType::Bool
                | ResolvedType::F32
                | ResolvedType::F64
                | ResolvedType::I32
                | ResolvedType::U32
                | ResolvedType::I16
                | ResolvedType::U16
                | ResolvedType::I8
                | ResolvedType::U8
        ) {
            Some(expected.clone())
        } else {
            None
        }
    }

    fn informative_non_i64_match_arm_type(ty: &ResolvedType) -> bool {
        matches!(
            ty,
            ResolvedType::Str
                | ResolvedType::Ref(_)
                | ResolvedType::RefMut(_)
                | ResolvedType::Bool
                | ResolvedType::F32
                | ResolvedType::F64
                | ResolvedType::I32
                | ResolvedType::U32
                | ResolvedType::I16
                | ResolvedType::U16
                | ResolvedType::I8
                | ResolvedType::U8
        )
    }

    fn match_arm_body_definitely_returns(body: &Spanned<Expr>) -> bool {
        match &body.node {
            Expr::Block(stmts) => Self::stmt_list_tail_definitely_returns(stmts),
            Expr::Match { arms, .. } if !arms.is_empty() => arms
                .iter()
                .all(|arm| Self::match_arm_body_definitely_returns(&arm.body)),
            _ => false,
        }
    }

    fn stmt_list_tail_definitely_returns(stmts: &[Spanned<Stmt>]) -> bool {
        matches!(
            stmts.last().map(|stmt| &stmt.node),
            Some(vais_ast::Stmt::Return(_))
        )
    }

    fn normalize_match_arm_value_for_phi(
        &mut self,
        mut body_val: String,
        arm_body_type: &ResolvedType,
        counter: &mut usize,
        ir: &mut String,
    ) -> String {
        match arm_body_type {
            ResolvedType::Named { .. } => {
                if body_val == "null" {
                    return body_val;
                }
                let llvm_ty = self.type_to_llvm(arm_body_type);
                let actual = self
                    .llvm_type_of_checked(&body_val)
                    .unwrap_or_else(|| self.llvm_type_of(&body_val));
                let target_ptr = format!("{}*", llvm_ty);
                if actual == "ptr" || actual == target_ptr || actual.ends_with('*') {
                    return body_val;
                }
                if actual == llvm_ty {
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
                } else if actual == "i64" {
                    let casted = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = inttoptr i64 {} to {}*",
                        casted,
                        body_val,
                        llvm_ty
                    );
                    self.fn_ctx.record_emitted_type(&casted, &target_ptr);
                    body_val = casted;
                }
                body_val
            }
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                if body_val == "null" {
                    return body_val;
                }
                let inner_llvm = self.type_to_llvm(inner);
                let target = format!("{}*", inner_llvm);
                let actual = self
                    .llvm_type_of_checked(&body_val)
                    .unwrap_or_else(|| self.llvm_type_of(&body_val));
                if actual == "i64" {
                    let casted = self.next_temp(counter);
                    write_ir!(ir, "  {} = inttoptr i64 {} to {}", casted, body_val, target);
                    self.fn_ctx.record_emitted_type(&casted, &target);
                    casted
                } else {
                    body_val
                }
            }
            ResolvedType::Str => {
                if body_val == "void" || body_val == "0" {
                    "{ i8* null, i64 0 }".to_string()
                } else {
                    body_val
                }
            }
            ResolvedType::Bool => {
                let actual = self
                    .llvm_type_of_checked(&body_val)
                    .unwrap_or_else(|| self.llvm_type_of(&body_val));
                if actual == "i1" {
                    let widened = self.next_temp(counter);
                    write_ir!(ir, "  {} = zext i1 {} to i64", widened, body_val);
                    self.fn_ctx.record_emitted_type(&widened, "i64");
                    widened
                } else {
                    body_val
                }
            }
            _ => body_val,
        }
    }

    fn ir_tail_terminates(ir: &str) -> bool {
        let Some(last) = ir
            .lines()
            .rev()
            .map(str::trim)
            .find(|line| !line.is_empty() && !line.starts_with(';'))
        else {
            return false;
        };

        last == "unreachable" || last.starts_with("ret ") || last.starts_with("br ")
    }

    fn type_contains_codegen_hole(ty: &ResolvedType) -> bool {
        match ty {
            ResolvedType::Var(_)
            | ResolvedType::Generic(_)
            | ResolvedType::Never
            | ResolvedType::Unknown => true,
            ResolvedType::Array(inner)
            | ResolvedType::Optional(inner)
            | ResolvedType::Pointer(inner)
            | ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Slice(inner)
            | ResolvedType::SliceMut(inner)
            | ResolvedType::Range(inner)
            | ResolvedType::Future(inner) => Self::type_contains_codegen_hole(inner),
            ResolvedType::ConstArray { element, .. } => Self::type_contains_codegen_hole(element),
            ResolvedType::Map(k, v) => {
                Self::type_contains_codegen_hole(k) || Self::type_contains_codegen_hole(v)
            }
            ResolvedType::Result(ok, err) => {
                Self::type_contains_codegen_hole(ok) || Self::type_contains_codegen_hole(err)
            }
            ResolvedType::Tuple(items) => items.iter().any(Self::type_contains_codegen_hole),
            ResolvedType::Named { generics, .. } => {
                generics.iter().any(Self::type_contains_codegen_hole)
            }
            ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
                params.iter().any(Self::type_contains_codegen_hole)
                    || Self::type_contains_codegen_hole(ret)
            }
            _ => false,
        }
    }

    fn type_is_more_specific(before: &ResolvedType, after: &ResolvedType) -> bool {
        if before == after || Self::type_contains_codegen_hole(after) {
            return false;
        }

        match (before, after) {
            (
                ResolvedType::Named {
                    name: before_name,
                    generics: before_generics,
                },
                ResolvedType::Named {
                    name: after_name,
                    generics: after_generics,
                },
            ) if before_name == after_name => {
                (before_generics.is_empty() && !after_generics.is_empty())
                    || (before_generics.len() == after_generics.len()
                        && before_generics
                            .iter()
                            .zip(after_generics.iter())
                            .any(|(b, a)| Self::type_is_more_specific(b, a)))
            }
            (ResolvedType::Var(_) | ResolvedType::Generic(_) | ResolvedType::Never, _) => true,
            (ResolvedType::Optional(b), ResolvedType::Optional(a))
            | (ResolvedType::Ref(b), ResolvedType::Ref(a))
            | (ResolvedType::RefMut(b), ResolvedType::RefMut(a))
            | (ResolvedType::Pointer(b), ResolvedType::Pointer(a))
            | (ResolvedType::Slice(b), ResolvedType::Slice(a))
            | (ResolvedType::SliceMut(b), ResolvedType::SliceMut(a))
            | (ResolvedType::Array(b), ResolvedType::Array(a))
            | (ResolvedType::Future(b), ResolvedType::Future(a))
            | (ResolvedType::Range(b), ResolvedType::Range(a)) => Self::type_is_more_specific(b, a),
            (ResolvedType::Tuple(b_items), ResolvedType::Tuple(a_items))
                if b_items.len() == a_items.len() =>
            {
                b_items
                    .iter()
                    .zip(a_items.iter())
                    .any(|(b, a)| Self::type_is_more_specific(b, a))
            }
            (ResolvedType::Result(b_ok, b_err), ResolvedType::Result(a_ok, a_err)) => {
                Self::type_is_more_specific(b_ok, a_ok) || Self::type_is_more_specific(b_err, a_err)
            }
            (ResolvedType::Map(b_k, b_v), ResolvedType::Map(a_k, a_v)) => {
                Self::type_is_more_specific(b_k, a_k) || Self::type_is_more_specific(b_v, a_v)
            }
            _ => false,
        }
    }

    pub(crate) fn infer_match_arm_result_type(
        &self,
        arm: &MatchArm,
        match_type: &ResolvedType,
    ) -> ResolvedType {
        let inferred = self.infer_expr_type(&arm.body);
        if !matches!(inferred, ResolvedType::I64 | ResolvedType::Unknown) {
            return inferred;
        }

        if let Expr::Ident(name) = &arm.body.node {
            self.resolve_pattern_binding_result_type(&arm.pattern, name, match_type)
                .unwrap_or(inferred)
        } else {
            inferred
        }
    }

    fn resolve_pattern_binding_result_type(
        &self,
        pattern: &Spanned<Pattern>,
        target: &str,
        match_type: &ResolvedType,
    ) -> Option<ResolvedType> {
        match &pattern.node {
            Pattern::Ident(name) => {
                if name == target
                    && !self.is_unit_enum_variant(name)
                    && !self.is_known_constant(name)
                {
                    Some(match_type.clone())
                } else {
                    None
                }
            }
            Pattern::Tuple(patterns) => {
                if let ResolvedType::Tuple(types) = match_type {
                    patterns.iter().zip(types.iter()).find_map(|(pat, ty)| {
                        self.resolve_pattern_binding_result_type(pat, target, ty)
                    })
                } else {
                    None
                }
            }
            Pattern::Variant { name, fields } => {
                let variant_name = &name.node;
                let field_types = match (variant_name.as_str(), match_type) {
                    ("Ok", ResolvedType::Result(ok, _)) => vec![(**ok).clone()],
                    ("Err", ResolvedType::Result(_, err)) => vec![(**err).clone()],
                    ("Some", ResolvedType::Optional(inner)) => vec![(**inner).clone()],
                    _ => {
                        let enum_name = match match_type {
                            ResolvedType::Named { name, .. } => Some(name.clone()),
                            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                                if let ResolvedType::Named { name, .. } = inner.as_ref() {
                                    Some(name.clone())
                                } else {
                                    None
                                }
                            }
                            _ => self.get_enum_name_for_variant(variant_name),
                        }?;
                        self.resolve_variant_field_types(&enum_name, variant_name, match_type)
                    }
                };

                fields
                    .iter()
                    .zip(field_types.iter())
                    .find_map(|(pat, ty)| self.resolve_pattern_binding_result_type(pat, target, ty))
            }
            Pattern::Struct {
                name,
                fields,
                enum_name,
            } => {
                let struct_or_variant = &name.node;
                if let Some(struct_info) = self.types.structs.get(struct_or_variant) {
                    for (field_name, field_pat) in fields {
                        if let Some((_, field_ty)) = struct_info
                            .fields
                            .iter()
                            .find(|(n, _)| n == &field_name.node)
                        {
                            if let Some(pat) = field_pat {
                                if let Some(found) =
                                    self.resolve_pattern_binding_result_type(pat, target, field_ty)
                                {
                                    return Some(found);
                                }
                            } else if field_name.node == target {
                                return Some(field_ty.clone());
                            }
                        }
                    }
                    None
                } else {
                    let enum_name = enum_name.clone().or_else(|| match match_type {
                        ResolvedType::Named { name, .. } => Some(name.clone()),
                        ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                            if let ResolvedType::Named { name, .. } = inner.as_ref() {
                                Some(name.clone())
                            } else {
                                None
                            }
                        }
                        _ => self.get_enum_name_for_variant(struct_or_variant),
                    })?;
                    let enum_lookup = enum_name
                        .split_once('$')
                        .map(|(base, _)| base)
                        .unwrap_or(enum_name.as_str());
                    let enum_info = self
                        .types
                        .enums
                        .get(enum_lookup)
                        .or_else(|| self.types.enums.get(enum_name.as_str()))?;
                    let variant = enum_info
                        .variants
                        .iter()
                        .find(|variant| variant.name == *struct_or_variant)?;
                    let field_types =
                        self.resolve_variant_field_types(&enum_name, struct_or_variant, match_type);
                    if let crate::types::EnumVariantFields::Struct(variant_fields) = &variant.fields
                    {
                        for (field_name, field_pat) in fields {
                            if let Some(field_idx) = variant_fields
                                .iter()
                                .position(|(n, _)| n == &field_name.node)
                            {
                                if let Some(field_ty) = field_types.get(field_idx) {
                                    if let Some(pat) = field_pat {
                                        if let Some(found) = self
                                            .resolve_pattern_binding_result_type(
                                                pat, target, field_ty,
                                            )
                                        {
                                            return Some(found);
                                        }
                                    } else if field_name.node == target {
                                        return Some(field_ty.clone());
                                    }
                                }
                            }
                        }
                    }
                    None
                }
            }
            Pattern::Alias { name, pattern } => {
                if name == target {
                    Some(match_type.clone())
                } else {
                    self.resolve_pattern_binding_result_type(pattern, target, match_type)
                }
            }
            Pattern::Or(patterns) => patterns
                .iter()
                .find_map(|pat| self.resolve_pattern_binding_result_type(pat, target, match_type)),
            _ => None,
        }
    }

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
        let match_raw_actual_ty = self.llvm_type_of_checked(&match_val_raw);
        let is_enum_or_struct = matches!(
            &match_type,
            ResolvedType::Named { .. } | ResolvedType::Optional(_) | ResolvedType::Result(_, _)
        );
        let emitted_aggregate_value = match_raw_actual_ty
            .as_deref()
            .map(|ty| ty.starts_with('%') && !ty.ends_with('*'))
            .unwrap_or(false);
        let is_value = self.is_expr_value(match_expr);

        // If it's an enum/struct value from a function call, store it on the stack
        let match_val = if (is_enum_or_struct || emitted_aggregate_value) && is_value {
            let llvm_type = if is_enum_or_struct {
                self.type_to_llvm(&match_type)
            } else {
                match_raw_actual_ty.unwrap_or_else(|| self.type_to_llvm(&match_type))
            };
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
            self.fn_ctx
                .record_emitted_type(&stack_ptr, &format!("{}*", llvm_type));

            stack_ptr
        } else {
            match_val_raw
        };

        // For str match values, extract the raw i8* pointer from the fat ptr { i8*, i64 }
        // so that strcmp in pattern matching receives the correct type. Use
        // emitted LLVM ground truth as a fallback because method calls such as
        // `.as_str()` can currently infer too weakly while still emitting a
        // concrete fat string value.
        let match_actual_ty = self.llvm_type_of_checked(&match_val);
        let match_val = if matches!(&match_type, ResolvedType::Str)
            || matches!(match_actual_ty.as_deref(), Some("{ i8*, i64 }"))
        {
            let raw_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                raw_ptr,
                match_val
            );
            self.fn_ctx.record_emitted_type(&raw_ptr, "i8*");
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
            let non_returning_arms: Vec<&MatchArm> = arms
                .iter()
                .filter(|arm| !Self::match_arm_body_definitely_returns(&arm.body))
                .collect();
            let type_arms: Vec<&MatchArm> = if non_returning_arms.is_empty() {
                arms.iter().collect()
            } else {
                non_returning_arms
            };
            let first_arm_ty = self.infer_match_arm_result_type(type_arms[0], &match_type);
            if matches!(first_arm_ty, ResolvedType::Named { .. }) {
                first_arm_ty
            } else {
                let named_from_arms = type_arms.iter().skip(1).find_map(|arm| {
                    let ty = self.infer_match_arm_result_type(arm, &match_type);
                    if matches!(ty, ResolvedType::Named { .. }) {
                        Some(ty)
                    } else {
                        None
                    }
                });
                named_from_arms.unwrap_or_else(|| {
                    if matches!(first_arm_ty, ResolvedType::I64 | ResolvedType::Unknown) {
                        type_arms
                            .iter()
                            .skip(1)
                            .find_map(|arm| {
                                let ty = self.infer_match_arm_result_type(arm, &match_type);
                                if Self::informative_non_i64_match_arm_type(&ty) {
                                    Some(ty)
                                } else {
                                    None
                                }
                            })
                            .or_else(|| self.informative_expected_match_type(&first_arm_ty))
                            .unwrap_or(first_arm_ty)
                    } else {
                        first_arm_ty
                    }
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
                        let (mut body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                        ir.push_str(&body_ir);
                        if !Self::ir_tail_terminates(&body_ir) {
                            body_val = self.normalize_match_arm_value_for_phi(
                                body_val,
                                &arm_body_type,
                                counter,
                                &mut ir,
                            );
                            arm_values.push((body_val, self.fn_ctx.current_block.clone()));
                            write_ir!(ir, "  br label %{}", merge_label);
                        }

                        // Guard failed - go to default
                        write_ir!(ir, "{}:", guard_fail);
                        write_ir!(ir, "  br label %{}", default_label);
                    } else {
                        self.fn_ctx.current_block.clone_from(label);
                        let (mut body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                        ir.push_str(&body_ir);
                        if !Self::ir_tail_terminates(&body_ir) {
                            body_val = self.normalize_match_arm_value_for_phi(
                                body_val,
                                &arm_body_type,
                                counter,
                                &mut ir,
                            );
                            arm_values.push((body_val, self.fn_ctx.current_block.clone()));
                            write_ir!(ir, "  br label %{}", merge_label);
                        }
                    }

                    case_idx += 1;
                }
            }

            // Generate default arm
            write_ir!(ir, "{}:", default_label);
            self.fn_ctx.current_block = default_label.clone();
            if let Some(arm) = default_arm {
                let (mut body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                ir.push_str(&body_ir);
                if !Self::ir_tail_terminates(&body_ir) {
                    body_val = self.normalize_match_arm_value_for_phi(
                        body_val,
                        &arm_body_type,
                        counter,
                        &mut ir,
                    );
                    // Body may have introduced new basic blocks — the branch to
                    // merge originates from whichever block is current now.
                    arm_values.push((body_val, self.fn_ctx.current_block.clone()));
                    write_ir!(ir, "  br label %{}", merge_label);
                }
            } else {
                // No default arm - unreachable or return 0
                let default_value = match &arm_body_type {
                    ResolvedType::Named { .. } | ResolvedType::Ref(_) | ResolvedType::RefMut(_) => {
                        "null".to_string()
                    }
                    ResolvedType::Str => "{ i8* null, i64 0 }".to_string(),
                    _ => "0".to_string(),
                };
                arm_values.push((default_value, self.fn_ctx.current_block.clone()));
                write_ir!(ir, "  br label %{}", merge_label);
            }
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
                    self.fn_ctx
                        .locals
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();

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
                    self.fn_ctx.record_emitted_type(&guard_bool, "i1");
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
                let post_arm_locals = self.fn_ctx.locals.clone();
                let pre_arm_keys: std::collections::HashSet<&String> =
                    pre_arm_locals.keys().collect();
                self.fn_ctx.locals.retain(|k, _| pre_arm_keys.contains(k));
                for (k, v) in &pre_arm_locals {
                    let mut restored = v.clone();
                    if let Some(after) = post_arm_locals.get(k) {
                        if after.llvm_name == v.llvm_name
                            && Self::type_is_more_specific(&v.ty, &after.ty)
                        {
                            restored.ty = after.ty.clone();
                        }
                    }
                    self.fn_ctx.locals.insert(k.clone(), restored);
                }

                // Coerce arm values to match the phi type.
                // Skip if the body_val is a placeholder (void/ret arms).
                let arm_terminated = Self::ir_tail_terminates(&body_ir);
                if !arm_terminated && !body_val.is_empty() && body_val != "void" {
                    let arm_inferred = self.infer_expr_type(&arm.body);
                    // Phase 17.H4.5: don't zext i1 when body_val is a Unit
                    // placeholder (`add i64 0, 0`). The arm might have been
                    // inferred as Bool at the TC level but this specific
                    // code path fell through to the placeholder (e.g., an
                    // early `ret` in one branch and no value in the other).
                    // llvm_type_of returns "i64" for the placeholder, so
                    // we can detect and skip the zext.
                    let body_actual_llvm = self.llvm_type_of(&body_val);
                    let looks_like_placeholder = body_actual_llvm == "i64"
                        && body_ir.trim_end().ends_with("void/Unit placeholder");
                    if (matches!(arm_inferred, ResolvedType::Bool)
                        || matches!(arm_body_type, ResolvedType::Bool))
                        && !looks_like_placeholder
                    {
                        let actual = self
                            .llvm_type_of_checked(&body_val)
                            .unwrap_or_else(|| self.llvm_type_of(&body_val));
                        if actual == "i1" {
                            let coerced = self.next_temp(counter);
                            write_ir!(ir, "  {} = zext i1 {} to i64", coerced, body_val);
                            self.fn_ctx.record_emitted_type(&coerced, "i64");
                            body_val = coerced;
                        }
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
                            write_ir!(ir, "  {} = trunc i64 {} to {}", narrowed, body_val, target);
                            body_val = narrowed;
                        }
                    } else if matches!(arm_body_type, ResolvedType::Str)
                        && (body_val == "void" || body_val == "0" || looks_like_placeholder)
                    {
                        body_val = "{ i8* null, i64 0 }".to_string();
                    } else if let ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) =
                        &arm_body_type
                    {
                        let actual = self.llvm_type_of(&body_val);
                        let target = self.type_to_llvm(&arm_body_type);
                        if actual == "i64" && target.ends_with('*') {
                            let casted = self.next_temp(counter);
                            write_ir!(ir, "  {} = inttoptr i64 {} to {}", casted, body_val, target);
                            self.fn_ctx.record_emitted_type(&casted, &target);
                            body_val = casted;
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
                            write_ir!(ir, "  {} = trunc i64 {} to {}", narrowed, body_val, target);
                            body_val = narrowed;
                        } else if actual == "i1" && target != "i1" {
                            // i1 → wider int (e.g., bool-typed pattern binding
                            // flowing into i32-typed phi from `order_by_compare`
                            // style match arms). zext preserves 0/1.
                            let widened = self.next_temp(counter);
                            write_ir!(ir, "  {} = zext i1 {} to {}", widened, body_val, target);
                            body_val = widened;
                        } else if actual.starts_with('i')
                            && target.starts_with('i')
                            && actual != target
                        {
                            // General int-width mismatch (e.g., i8 → i32 for
                            // pattern-bound narrow locals). Use trunc/sext.
                            body_val = self
                                .coerce_int_width(&body_val, &actual, &target, counter, &mut ir);
                        }
                    } else if matches!(arm_body_type, ResolvedType::F32 | ResolvedType::F64) {
                        let actual = self.llvm_type_of(&body_val);
                        let target = self.type_to_llvm(&arm_body_type);
                        if actual != target {
                            let coerced = self
                                .coerce_float_width(&body_val, &actual, &target, counter, &mut ir);
                            if coerced != body_val {
                                self.fn_ctx.record_emitted_type(&coerced, &target);
                                body_val = coerced;
                            }
                        }
                    } else if matches!(&arm_body_type, ResolvedType::Named { .. }) {
                        // Named type (struct/enum): phi uses pointer type (%T*).
                        // Normalize each arm to that pointer type in the arm
                        // block. Some arm bodies are inferred as I64/Unknown
                        // even though the enclosing match LUB is Named (for
                        // example a nested if-expression returning `%T`).
                        let llvm_ty = self.type_to_llvm(&arm_body_type);
                        let actual = self
                            .llvm_type_of_checked(&body_val)
                            .unwrap_or_else(|| self.llvm_type_of(&body_val));
                        let actual_is_ptr = actual == "ptr" || actual.ends_with('*');
                        if actual_is_ptr {
                            self.fn_ctx
                                .record_emitted_type(&body_val, &format!("{}*", llvm_ty));
                        } else if actual == llvm_ty {
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
                        } else if actual == "i64" {
                            let casted = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = inttoptr i64 {} to {}*",
                                casted,
                                body_val,
                                llvm_ty
                            );
                            self.fn_ctx
                                .record_emitted_type(&casted, &format!("{}*", llvm_ty));
                            body_val = casted;
                        }
                    } else if matches!(&arm_body_type, ResolvedType::Tuple(_)) {
                        // Tuple aggregate phi: arm value must already be a
                        // `{ ... }` value matching arm_body_type. If an arm
                        // happened to leave the value as i64 (placeholder /
                        // wide-load fallback), reload it through a typed
                        // pointer so the phi incoming has the correct
                        // aggregate type. Keeps tuple-of-struct payloads
                        // (e.g., `Ok((Meta, Ptr))`) intact end-to-end.
                        let tuple_llvm = self.type_to_llvm(&arm_body_type);
                        let actual = self
                            .llvm_type_of_checked(&body_val)
                            .unwrap_or_else(|| self.llvm_type_of(&body_val));
                        if actual == tuple_llvm {
                            self.fn_ctx.record_emitted_type(&body_val, &tuple_llvm);
                        } else if actual == format!("{}*", tuple_llvm) || actual == "ptr" {
                            let loaded = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = load {}, {}* {}",
                                loaded,
                                tuple_llvm,
                                tuple_llvm,
                                body_val
                            );
                            self.fn_ctx.record_emitted_type(&loaded, &tuple_llvm);
                            body_val = loaded;
                        }
                    }
                }

                // Use actual current block (may differ from arm_body_label if body
                // inserted intermediate labels, e.g., division-by-zero guard).
                //
                // Phase B5: if the arm body terminates the current block
                // (ret / unreachable), do not add a phi incoming from it and
                // skip the fallthrough `br` — emitting `br` after `ret` leaves
                // an unreachable block whose phi entry in the merge mismatches
                // the actual predecessor set.
                if !arm_terminated {
                    let actual_block = self.fn_ctx.current_block.clone();
                    arm_values.push((body_val, actual_block));
                    write_ir!(ir, "  br label %{}", merge_label);
                }

                current_label = next_label;
            }

            // Default fallthrough block (no arm matched)
            write_ir!(ir, "{}:", default_label);
            self.fn_ctx.current_block.clone_from(&default_label);
            // Use appropriate default value based on arm types or function return type
            let default_val = {
                let mut resolved = if matches!(arm_body_type, ResolvedType::Unknown) {
                    if !arms.is_empty() {
                        self.infer_expr_type(&arms[0].body)
                    } else {
                        ResolvedType::I64
                    }
                } else {
                    arm_body_type.clone()
                };
                // If this match is in a typed expression context and arm
                // inference only produced a fallback, use the explicit context.
                // Do not fall back to the function return type here: nested
                // matches inside a Result-returning function may legitimately
                // produce str/i32/etc. values for local initializers.
                if !matches!(resolved, ResolvedType::Named { .. }) {
                    if let Some(expected) = self.informative_expected_match_type(&resolved) {
                        resolved = expected;
                    }
                }
                // Phase α.1 fix: when the inferred resolved type is wider than
                // the actual phi (e.g., Named wrapper but inner phi is i32),
                // first check the actual LLVM type registered for arm values.
                // Fall back to resolved type only when actual is unknown.
                let phi_llvm_actual = arm_values
                    .iter()
                    .find_map(|(v, _)| self.fn_ctx.get_emitted_type(v).map(|s| s.to_string()))
                    .or_else(|| {
                        arm_values
                            .iter()
                            .find_map(|(v, _)| self.llvm_type_of_checked(v))
                    });
                // B-58 fix (Phase 1 100% Gap, iter 62): when actual phi LLVM type is a
                // value (e.g. `{ i8*, i64 }` fat-pointer for &str / Str return), the
                // arm_body_type-based "null" default produces a pointer-typed default
                // and forces phi to pointer type — clang rejects with type mismatch
                // (vaisdb types.ll:2412 cascade). Honor actual emitted type first.
                let actual_override: Option<String> =
                    phi_llvm_actual.as_deref().and_then(|actual| {
                        if actual == "{ i8*, i64 }" {
                            let zinit = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = insertvalue {{ i8*, i64 }} {{ i8* null, i64 0 }}, i64 0, 1",
                                zinit
                            );
                            Some(zinit)
                        } else if !actual.ends_with('*') && actual.starts_with('{') {
                            Some("zeroinitializer".to_string())
                        } else {
                            None
                        }
                    });
                if let Some(v) = actual_override {
                    v
                } else if matches!(arm_body_type, ResolvedType::Named { .. }) {
                    "null".to_string()
                } else if matches!(
                    arm_body_type,
                    ResolvedType::Ref(_) | ResolvedType::RefMut(_)
                ) {
                    let ref_llvm = self.type_to_llvm(&arm_body_type);
                    if ref_llvm.ends_with('*') {
                        "null".to_string()
                    } else if ref_llvm == "{ i8*, i64 }" {
                        let zinit = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = insertvalue {{ i8*, i64 }} {{ i8* null, i64 0 }}, i64 0, 1",
                            zinit
                        );
                        zinit
                    } else {
                        "0".to_string()
                    }
                } else if matches!(arm_body_type, ResolvedType::Str) {
                    let zinit = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = insertvalue {{ i8*, i64 }} {{ i8* null, i64 0 }}, i64 0, 1",
                        zinit
                    );
                    zinit
                } else if matches!(arm_body_type, ResolvedType::Tuple(_)) {
                    // Aggregate tuple phi: zeroinitializer matches `{ ... }`
                    // exactly without inventing per-element defaults.
                    "zeroinitializer".to_string()
                } else if let Some(actual) = phi_llvm_actual {
                    // Use actual phi type to pick a type-correct default.
                    if actual.ends_with('*') {
                        "null".to_string()
                    } else if actual == "{ i8*, i64 }" {
                        let zinit = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = insertvalue {{ i8*, i64 }} {{ i8* null, i64 0 }}, i64 0, 1",
                            zinit
                        );
                        zinit
                    } else if actual == "double" || actual == "float" {
                        "0.0".to_string()
                    } else {
                        "0".to_string()
                    }
                } else {
                    // Phase α.1 fix: when Named/Ref resolved but actual is
                    // unknown, the phi may still be a narrow integer (e.g.,
                    // when match is over an Option inner i32). Use literal "0"
                    // as a safe default — only emit "null" when the IR string
                    // form of the resolved type is actually a pointer.
                    let resolved_llvm = self.type_to_llvm(&resolved);
                    match &resolved {
                        ResolvedType::Str => {
                            let zinit = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = insertvalue {{ i8*, i64 }} {{ i8* null, i64 0 }}, i64 0, 1",
                                zinit
                            );
                            zinit
                        }
                        ResolvedType::Named { .. } => "null".to_string(),
                        ResolvedType::Ref(_) | ResolvedType::RefMut(_) => {
                            if resolved_llvm.ends_with('*') {
                                "null".to_string()
                            } else if resolved_llvm == "{ i8*, i64 }" {
                                let zinit = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = insertvalue {{ i8*, i64 }} {{ i8* null, i64 0 }}, i64 0, 1",
                                    zinit
                                );
                                zinit
                            } else {
                                "0".to_string()
                            }
                        }
                        ResolvedType::F64 => "0.0".to_string(),
                        ResolvedType::Bool => "0".to_string(),
                        _ => "0".to_string(),
                    }
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
                ResolvedType::Ref(_) | ResolvedType::RefMut(_) => self.type_to_llvm(&arm_body_type),
                ResolvedType::F64 => "double".to_string(),
                ResolvedType::F32 => "float".to_string(),
                ResolvedType::Bool => "i64".to_string(), // Bool is zext'd to i64 in codegen
                ResolvedType::I8 | ResolvedType::U8 => "i8".to_string(),
                ResolvedType::I16 | ResolvedType::U16 => "i16".to_string(),
                ResolvedType::I32 | ResolvedType::U32 => "i32".to_string(),
                ResolvedType::Tuple(_) => self.type_to_llvm(&arm_body_type),
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
                    let safe = if val == "void" {
                        void_substitute
                    } else if phi_type == "{ i8*, i64 }" && val == "0" {
                        void_substitute
                    } else {
                        val.as_str()
                    };
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
            self.fn_ctx.record_emitted_type(&phi_result, &phi_type);

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
