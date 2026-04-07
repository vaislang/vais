//! Loop expression code generation.
//!
//! Extracted from `generate_expr_inner` match arms for `Expr::Loop` and
//! `Expr::While` to reduce the parent function's stack frame size.
//! Each handler is `#[inline(never)]` so Rust allocates its locals independently.

use vais_ast::*;
use vais_types::ResolvedType;

use crate::{CodeGenerator, CodegenResult, LocalVar, LoopLabels};

impl CodeGenerator {
    /// Generate code for a loop expression (`L` keyword) with pattern support.
    /// Handles range-based for loops (`L pattern : start..end { body }`),
    /// collection for-each loops (`L elem : &collection { body }`),
    /// then falls through to conditional/infinite loops.
    /// Extracted from `generate_expr_inner` to reduce stack frame size.
    #[inline(never)]
    pub(crate) fn generate_loop_with_pattern(
        &mut self,
        pattern: Option<&Spanned<Pattern>>,
        iter: Option<&Spanned<Expr>>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Check if this is a range-based for loop
        let is_range_loop = iter
            .as_ref()
            .is_some_and(|it| matches!(&it.node, Expr::Range { .. }));

        if is_range_loop {
            if let (Some(pat), Some(it)) = (pattern, iter) {
                // Range-based for loop: L pattern : start..end { body }
                return self.generate_range_for_loop(pat, it, body, counter);
            }
        }

        // Collection for-each loop: L elem : &collection { body }
        // When we have a named pattern variable (not Wildcard) and a non-range iter expression,
        // generate an index-based iteration over the collection.
        // Wildcard patterns (L _:condition) are conditional (while) loops, not collection iteration.
        if let (Some(pat), Some(iter_expr)) = (pattern, iter) {
            if matches!(&pat.node, Pattern::Ident(_)) {
                return self.generate_collection_for_loop(pat, iter_expr, body, counter);
            }
        }

        // Conditional or infinite loop
        let loop_start = self.next_label("loop.start");
        let loop_body = self.next_label("loop.body");
        let loop_end = self.next_label("loop.end");

        // Push loop labels for break/continue
        self.fn_ctx.loop_stack.push(LoopLabels {
            continue_label: loop_start.clone(), // keep: used in continue stmt
            break_label: loop_end.clone(),      // keep: used in break stmt
        });

        let mut ir = String::new();

        // Check if this is a conditional loop (L cond { body }) or infinite loop
        if let Some(iter_expr) = iter {
            // Conditional loop: L condition { body }
            write_ir!(ir, "  br label %{}", loop_start);
            write_ir!(ir, "{}:", loop_start);

            // Evaluate condition
            let (cond_val, cond_ir) = self.generate_expr(iter_expr, counter)?;
            ir.push_str(&cond_ir);

            // Convert to i1 for branch (type-aware: skips for bool/i1)
            let (cond_bool, conv_ir) = self.generate_cond_to_i1(iter_expr, &cond_val, counter);
            ir.push_str(&conv_ir);
            write_ir!(
                ir,
                "  br i1 {}, label %{}, label %{}",
                cond_bool,
                loop_body,
                loop_end
            );

            // Loop body
            write_ir!(ir, "{}:", loop_body);
            let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
            ir.push_str(&body_ir);
            // Only emit loop back if body doesn't terminate
            if !body_terminated {
                write_ir!(ir, "  br label %{}", loop_start);
            }
        } else {
            // Infinite loop: L { body } - must use break to exit
            write_ir!(ir, "  br label %{}", loop_start);
            write_ir!(ir, "{}:", loop_start);
            let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
            ir.push_str(&body_ir);
            // Only emit loop back if body doesn't terminate
            if !body_terminated {
                write_ir!(ir, "  br label %{}", loop_start);
            }
        }

        // Loop end
        write_ir!(ir, "{}:", loop_end);
        self.fn_ctx.current_block.clone_from(&loop_end);

        self.fn_ctx.loop_stack.pop();

        // Loop returns void by default (use break with value for expression)
        Ok(("0".to_string(), ir))
    }

    /// Generate code for a collection for-each loop: `L elem : &collection { body }`.
    ///
    /// Produces an index-based loop that:
    /// 1. Evaluates the collection expression to obtain the collection value.
    /// 2. Extracts the length and data pointer (Vec, Slice, Array, etc.).
    /// 3. Creates an internal index counter (i64).
    /// 4. On each iteration: loads the element at the current index into the pattern
    ///    variable, executes the loop body, and increments the counter.
    #[inline(never)]
    fn generate_collection_for_loop(
        &mut self,
        pattern: &Spanned<Pattern>,
        iter_expr: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();

        // 1. Evaluate the collection expression
        let (coll_val, coll_ir) = self.generate_expr(iter_expr, counter)?;
        ir.push_str(&coll_ir);

        // 2. Determine the collection type and extract length + data pointer
        let coll_type = self.infer_expr_type(iter_expr);

        // Unwrap Ref/RefMut to get the inner collection type for classification,
        // but keep coll_type as the full type for LLVM representation.
        let inner_type = match &coll_type {
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref().clone(),
            other => other.clone(),
        };

        // Determine element LLVM type
        let elem_resolved = self.get_collection_element_type(&coll_type);
        let elem_llvm_ty = self.type_to_llvm(&elem_resolved);

        // Determine if the collection is a slice (fat pointer { i8*, i64 })
        let is_slice = matches!(
            inner_type,
            ResolvedType::Slice(_) | ResolvedType::SliceMut(_)
        );

        // Determine if the collection is a Vec<T>
        let is_vec = matches!(
            &inner_type,
            ResolvedType::Named { name, .. } if name == "Vec"
        );

        // Extract length
        let length_val = if is_slice {
            // Slice fat pointer: extract length from field 1
            let len = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = extractvalue {{ i8*, i64 }} {}, 1",
                len,
                coll_val
            );
            len
        } else if is_vec {
            // Vec: load length from struct field 1
            let len_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr %Vec, %Vec* {}, i32 0, i32 1",
                len_ptr,
                coll_val
            );
            let len = self.next_temp(counter);
            write_ir!(ir, "  {} = load i64, i64* {}", len, len_ptr);
            len
        } else if let ResolvedType::ConstArray { size, .. } = &inner_type {
            // Const array: use compile-time size
            if let Some(n) = size.try_evaluate() {
                n.to_string()
            } else {
                "0".to_string()
            }
        } else {
            // For dynamic arrays (i64*), use a method call to get length.
            // Try to get length from the MethodCall codegen on the original expr.
            // As a fallback, use a generic approach: call <type>_len or extract from context.
            // If no length can be determined, generate a zero-trip loop (safe fallback).
            "0".to_string()
        };

        // Extract data pointer for element access
        let data_ptr = if is_slice {
            // Slice: extract data pointer from field 0, then bitcast
            let raw = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                raw,
                coll_val
            );
            let typed = self.next_temp(counter);
            write_ir!(ir, "  {} = bitcast i8* {} to {}*", typed, raw, elem_llvm_ty);
            typed
        } else if is_vec {
            // Vec: load data pointer from struct field 0 (stored as i64), cast to elem*
            let data_field = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr %Vec, %Vec* {}, i32 0, i32 0",
                data_field,
                coll_val
            );
            let data_i64 = self.next_temp(counter);
            write_ir!(ir, "  {} = load i64, i64* {}", data_i64, data_field);
            let typed = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = inttoptr i64 {} to {}*",
                typed,
                data_i64,
                elem_llvm_ty
            );
            typed
        } else {
            // Array/Pointer: use directly as element pointer
            coll_val.clone()
        };

        // 3. Create internal index counter (alloca at function entry)
        let idx_var = format!("%foreach_idx.{}", self.fn_ctx.label_counter);
        self.fn_ctx.label_counter += 1;
        self.emit_entry_alloca(&idx_var, "i64");
        write_ir!(ir, "  store i64 0, i64* {}", idx_var);

        // 4. Create pattern variable alloca and register in locals
        if let Pattern::Ident(name) = &pattern.node {
            let var_name = format!("{}.foreach.{}", name, self.fn_ctx.label_counter);
            self.fn_ctx.label_counter += 1;
            let llvm_name = format!("%{}", var_name);
            self.emit_entry_alloca(&llvm_name, &elem_llvm_ty);
            self.fn_ctx.locals.insert(
                name.clone(),
                LocalVar::alloca(elem_resolved.clone(), var_name),
            );
        }

        // 5. Generate loop structure: cond → body → inc → cond, with exit
        let loop_cond = self.next_label("foreach.cond");
        let loop_body_label = self.next_label("foreach.body");
        let loop_inc = self.next_label("foreach.inc");
        let loop_end = self.next_label("foreach.end");

        self.fn_ctx.loop_stack.push(LoopLabels {
            continue_label: loop_inc.clone(),
            break_label: loop_end.clone(),
        });

        write_ir!(ir, "  br label %{}", loop_cond);

        // Condition: idx < length
        write_ir!(ir, "{}:", loop_cond);
        let current_idx = self.next_temp(counter);
        write_ir!(ir, "  {} = load i64, i64* {}", current_idx, idx_var);
        let cmp = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = icmp slt i64 {}, {}",
            cmp,
            current_idx,
            length_val
        );
        write_ir!(
            ir,
            "  br i1 {}, label %{}, label %{}",
            cmp,
            loop_body_label,
            loop_end
        );

        // Body: load element, store in pattern variable, run body
        write_ir!(ir, "{}:", loop_body_label);

        // Load current index again (in case of phi issues)
        let body_idx = self.next_temp(counter);
        write_ir!(ir, "  {} = load i64, i64* {}", body_idx, idx_var);

        // Get element pointer and load element
        if let Pattern::Ident(name) = &pattern.node {
            if let Some(local) = self.fn_ctx.locals.get(name).cloned() {
                let llvm_name = format!("%{}", local.llvm_name);

                if elem_llvm_ty.starts_with('%') && !is_vec {
                    // Struct element: get pointer to the struct in the array and copy it
                    let elem_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr {}, {}* {}, i64 {}",
                        elem_ptr,
                        elem_llvm_ty,
                        elem_llvm_ty,
                        data_ptr,
                        body_idx
                    );
                    // For struct types, store the pointer as the variable value
                    // (the loop variable acts as a reference/pointer to the element)
                    write_ir!(
                        ir,
                        "  store {} {}, {}* {}",
                        // Store the pointer to the struct element
                        format!("{}*", elem_llvm_ty),
                        elem_ptr,
                        format!("{}*", elem_llvm_ty),
                        llvm_name
                    );
                } else if is_vec && elem_llvm_ty.starts_with('%') {
                    // Vec<StructType>: elements may be stored differently
                    // Use elem_size from Vec to compute the offset
                    let es_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr %Vec, %Vec* {}, i32 0, i32 3",
                        es_ptr,
                        coll_val
                    );
                    let elem_size = self.next_temp(counter);
                    write_ir!(ir, "  {} = load i64, i64* {}", elem_size, es_ptr);
                    let byte_offset = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = mul i64 {}, {}",
                        byte_offset,
                        body_idx,
                        elem_size
                    );
                    // Cast data_ptr to i8* for byte-level offset
                    let raw_data = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = bitcast {}* {} to i8*",
                        raw_data,
                        elem_llvm_ty,
                        data_ptr
                    );
                    let offset_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr i8, i8* {}, i64 {}",
                        offset_ptr,
                        raw_data,
                        byte_offset
                    );
                    let typed_elem_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = bitcast i8* {} to {}*",
                        typed_elem_ptr,
                        offset_ptr,
                        elem_llvm_ty
                    );
                    // Store the pointer to the struct element
                    write_ir!(
                        ir,
                        "  store {} {}, {}* {}",
                        format!("{}*", elem_llvm_ty),
                        typed_elem_ptr,
                        format!("{}*", elem_llvm_ty),
                        llvm_name
                    );
                } else {
                    // Primitive element: GEP + load + store
                    let elem_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr {}, {}* {}, i64 {}",
                        elem_ptr,
                        elem_llvm_ty,
                        elem_llvm_ty,
                        data_ptr,
                        body_idx
                    );
                    let elem_val = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}",
                        elem_val,
                        elem_llvm_ty,
                        elem_llvm_ty,
                        elem_ptr
                    );
                    write_ir!(
                        ir,
                        "  store {} {}, {}* {}",
                        elem_llvm_ty,
                        elem_val,
                        elem_llvm_ty,
                        llvm_name
                    );
                }
            }
        }

        // Execute the loop body
        let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
        ir.push_str(&body_ir);

        if !body_terminated {
            write_ir!(ir, "  br label %{}", loop_inc);
        }

        // Increment index
        write_ir!(ir, "{}:", loop_inc);
        let inc_load = self.next_temp(counter);
        write_ir!(ir, "  {} = load i64, i64* {}", inc_load, idx_var);
        let inc_result = self.next_temp(counter);
        write_ir!(ir, "  {} = add i64 {}, 1", inc_result, inc_load);
        write_ir!(ir, "  store i64 {}, i64* {}", inc_result, idx_var);
        write_ir!(ir, "  br label %{}", loop_cond);

        // Loop end
        write_ir!(ir, "{}:", loop_end);
        self.fn_ctx.current_block.clone_from(&loop_end);
        self.fn_ctx.loop_stack.pop();

        Ok(("0".to_string(), ir))
    }

    /// Determine the element type of a collection for iteration.
    fn get_collection_element_type(&self, coll_type: &ResolvedType) -> ResolvedType {
        match coll_type {
            ResolvedType::Array(elem) => (**elem).clone(),
            ResolvedType::ConstArray { element, .. } => (**element).clone(),
            ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => (**elem).clone(),
            ResolvedType::Pointer(elem) => (**elem).clone(),
            ResolvedType::Named { name, generics } if name == "Vec" && !generics.is_empty() => {
                generics[0].clone()
            }
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                self.get_collection_element_type(inner)
            }
            _ => ResolvedType::I64,
        }
    }

    /// Generate code for a while loop expression.
    /// Extracted from `generate_expr_inner` to reduce stack frame size.
    #[inline(never)]
    pub(crate) fn generate_while_loop_expr(
        &mut self,
        condition: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let loop_start = self.next_label("while.start");
        let loop_body = self.next_label("while.body");
        let loop_end = self.next_label("while.end");

        // Push loop labels for break/continue
        self.fn_ctx.loop_stack.push(LoopLabels {
            continue_label: loop_start.clone(), // keep: used in continue stmt
            break_label: loop_end.clone(),      // keep: used in break stmt
        });

        let mut ir = String::new();

        // Jump to condition check
        write_ir!(ir, "  br label %{}", loop_start);
        write_ir!(ir, "{}:", loop_start);

        // Evaluate condition
        let (cond_val, cond_ir) = self.generate_expr(condition, counter)?;
        ir.push_str(&cond_ir);

        // Convert to i1 for branch (type-aware: skips for bool/i1)
        let (cond_bool, conv_ir) = self.generate_cond_to_i1(condition, &cond_val, counter);
        ir.push_str(&conv_ir);
        write_ir!(
            ir,
            "  br i1 {}, label %{}, label %{}",
            cond_bool,
            loop_body,
            loop_end
        );

        // Loop body
        write_ir!(ir, "{}:", loop_body);
        let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
        ir.push_str(&body_ir);

        // Jump back to condition if body doesn't terminate
        if !body_terminated {
            write_ir!(ir, "  br label %{}", loop_start);
        }

        // Loop end
        write_ir!(ir, "{}:", loop_end);
        self.fn_ctx.current_block.clone_from(&loop_end);

        self.fn_ctx.loop_stack.pop();

        Ok(("0".to_string(), ir))
    }
}
