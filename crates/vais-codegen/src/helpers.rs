//! Miscellaneous code generation helper methods

use std::borrow::Cow;

use super::*;

/// Block labels that are hardcoded in codegen (not generated via `next_label`).
/// Parameter names matching these must be renamed to avoid LLVM IR collisions.
const RESERVED_BLOCK_LABELS: &[&str] = &["entry"];

/// Characters that are invalid in LLVM IR identifiers and must be replaced.
/// LLVM allows alphanumeric, '.', '_', and '$' in identifiers.
#[allow(dead_code)] // Used by sanitize_llvm_name below; callers will migrate incrementally
const LLVM_INVALID_CHARS: &[char] = &[
    '<', '>', ',', ' ', ':', ';', '(', ')', '{', '}', '[', ']', '!', '@', '#', '%', '^', '&', '*',
    '+', '=', '|', '\\', '/', '?', '~', '`', '"', '\'',
];

/// Check whether a resolved type represents void/Unit.
///
/// Returns `true` for `ResolvedType::Unit` and when the LLVM type string
/// is `"void"`. Used to decide whether phi nodes should be replaced with
/// [`void_placeholder_ir`].
pub(crate) fn is_void_result(llvm_type: &str, resolved: &vais_types::ResolvedType) -> bool {
    llvm_type == "void" || *resolved == vais_types::ResolvedType::Unit
}

/// Generate an LLVM IR instruction that acts as a void/Unit placeholder.
///
/// LLVM IR does not allow `phi void` — void is not a first-class type. When
/// an if/else or match expression produces a Unit/void result, we still need
/// an SSA variable to satisfy the codegen infrastructure that expects every
/// expression to yield a result name. This helper generates a well-documented
/// `add i64 0, 0` instruction that is dead (its result is never used).
///
/// The Inkwell backend uses `struct_type(&[], false).const_zero()` (a
/// zero-sized struct) for the same purpose — both approaches are valid LLVM
/// idioms for representing "no meaningful value."
pub(crate) fn void_placeholder_ir(result: &str) -> String {
    format!("  {} = add i64 0, 0  ; void/Unit placeholder\n", result)
}

/// Estimate LLVM type size in bytes from the type string.
/// For struct types like `{ i8*, i64 }`, sums field sizes.
#[allow(dead_code)]
pub(crate) fn estimate_llvm_type_size(ty: &str) -> usize {
    match ty {
        "i1" | "i8" => 1,
        "i16" => 2,
        "i32" | "float" => 4,
        "i64" | "double" | "ptr" => 8,
        t if t.ends_with('*') => 8, // pointer
        t if t.starts_with("{ ") && t.ends_with(" }") => {
            // Struct type: sum of field sizes
            let inner = &t[2..t.len() - 2];
            inner
                .split(',')
                .map(|f| estimate_llvm_type_size(f.trim()))
                .sum()
        }
        t if t.starts_with('%') => 8, // named type, assume pointer-sized
        _ => 8,
    }
}

/// Sanitize a parameter name to avoid collision with LLVM block labels.
/// Returns `Cow::Borrowed` when no rename is needed (zero allocation).
pub(crate) fn sanitize_param_name(name: &str) -> Cow<'_, str> {
    if RESERVED_BLOCK_LABELS.contains(&name) {
        Cow::Owned(format!("{}.param", name))
    } else {
        Cow::Borrowed(name)
    }
}

/// Sanitize an arbitrary identifier for use as an LLVM IR name.
///
/// Replaces characters that are invalid in LLVM identifiers with underscores,
/// and appends a disambiguating suffix counter to prevent collisions when
/// two different source names sanitize to the same LLVM name.
///
/// # Arguments
/// * `name` - The source identifier to sanitize
/// * `disambiguation_suffix` - Optional numeric suffix to append for uniqueness
///
/// # Returns
/// A valid LLVM IR identifier string.
#[allow(dead_code)] // Review #10: available for callers that generate LLVM names from user identifiers
pub(crate) fn sanitize_llvm_name(name: &str, disambiguation_suffix: Option<usize>) -> String {
    let mut result = String::with_capacity(name.len() + 8);
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '$' {
            result.push(ch);
        } else if LLVM_INVALID_CHARS.contains(&ch) {
            result.push('_');
        } else {
            // Non-ASCII or other chars: encode as _uXXXX
            write_ir_no_newline!(result, "_u{:04X}", ch as u32);
        }
    }
    if let Some(suffix) = disambiguation_suffix {
        write_ir_no_newline!(result, "__{}", suffix);
    }
    result
}

impl CodeGenerator {
    /// Record a static-size alloca to be hoisted into the function entry block.
    ///
    /// Instead of emitting `alloca` inline (which may land inside an if/else or loop block),
    /// this method records the instruction and returns the variable name. The collected
    /// allocas are spliced into the entry block by [`Self::splice_entry_allocas`] after
    /// the full function body IR has been generated.
    ///
    /// # Arguments
    /// * `var_name` - The LLVM temporary name (e.g., `%tmp.5`)
    /// * `llvm_type` - The LLVM type to allocate (e.g., `%MyStruct`, `[10 x i64]`)
    #[inline(never)]
    pub(crate) fn emit_entry_alloca(&mut self, var_name: &str, llvm_type: &str) {
        self.fn_ctx
            .entry_allocas
            .push(format!("  {} = alloca {}", var_name, llvm_type));
    }

    /// Splice collected entry-block allocas into function IR.
    ///
    /// Searches for the `entry:` label in the given IR string and inserts all
    /// collected allocas immediately after it. This ensures LLVM can optimize
    /// all stack allocations and avoids domination errors from non-entry allocas.
    ///
    /// Must be called after the function body has been fully generated but before
    /// the IR is returned.
    #[inline(never)]
    pub(crate) fn splice_entry_allocas(&mut self, ir: &mut String) {
        if self.fn_ctx.entry_allocas.is_empty() {
            return;
        }

        // Build the alloca block to insert
        let mut alloca_block = String::new();
        for alloca_line in &self.fn_ctx.entry_allocas {
            alloca_block.push_str(alloca_line);
            alloca_block.push('\n');
        }

        // Find "entry:\n" and insert allocas right after it
        if let Some(pos) = ir.find("entry:\n") {
            let insert_pos = pos + "entry:\n".len();
            ir.insert_str(insert_pos, &alloca_block);
        }

        self.fn_ctx.entry_allocas.clear();
    }

    /// Generate a unique string constant name, with optional module prefix
    #[inline(never)]
    pub(crate) fn make_string_name(&self) -> String {
        use std::fmt::Write;
        let mut name = String::with_capacity(16);
        if let Some(ref prefix) = self.strings.prefix {
            let _ = write!(name, "{}.str.{}", prefix, self.strings.counter);
        } else {
            let _ = write!(name, ".str.{}", self.strings.counter);
        }
        name
    }

    /// Get or create a string constant, deduplicating identical string values.
    ///
    /// If the same string value has already been registered as a global constant,
    /// returns the existing constant name without creating a new one.
    /// This reduces IR size and binary size when the same string literal
    /// appears multiple times in source code.
    #[inline(never)]
    pub(crate) fn get_or_create_string_constant(&mut self, value: &str) -> String {
        // Check dedup cache first
        if let Some(existing_name) = self.strings.dedup_cache.get(value) {
            return existing_name.clone();
        }

        // Create new constant
        let name = self.make_string_name();
        self.strings.counter += 1;
        self.strings
            .constants
            .push((name.clone(), value.to_string()));
        self.strings
            .dedup_cache
            .insert(value.to_string(), name.clone());
        name
    }

    /// Generate a unique label with the given prefix
    #[inline(never)]
    pub(crate) fn next_label(&mut self, prefix: &str) -> String {
        debug_assert!(
            !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_'),
            "Invalid label prefix: '{}'. Must be non-empty and contain only alphanumeric, '.', or '_' characters.",
            prefix
        );
        use std::fmt::Write;
        let mut label = String::with_capacity(prefix.len() + 6);
        let _ = write!(label, "{}{}", prefix, self.fn_ctx.label_counter);
        self.fn_ctx.label_counter += 1;
        label
    }

    /// Generate a unique temporary register name
    #[inline(never)]
    pub(crate) fn next_temp(&self, counter: &mut usize) -> String {
        use std::fmt::Write;
        let mut tmp = String::with_capacity(8); // "%t" + up to 6 digits
        let _ = write!(tmp, "%t{}", counter);
        *counter += 1;
        tmp
    }

    /// Check if a function call is recursive (calls the current function with decreases clause)
    #[inline(never)]
    pub(crate) fn is_recursive_call(&self, fn_name: &str) -> bool {
        // Check if we have a decreases clause for this function
        if let Some(ref decreases_info) = self.contracts.current_decreases_info {
            // A recursive call is when the called function matches the function with decreases
            decreases_info.function_name == fn_name
        } else {
            false
        }
    }

    /// Check if a function has the #[gc] attribute
    #[inline(never)]
    pub(crate) fn _has_gc_attribute(attributes: &[Attribute]) -> bool {
        attributes.iter().any(|attr| attr.name == "gc")
    }

    /// Enter a type recursion level and check depth limit
    /// Returns an error if recursion limit is exceeded
    #[inline(never)]
    pub(crate) fn enter_type_recursion(&self, context: &str) -> CodegenResult<()> {
        let depth = self.type_recursion_depth.get();
        if depth >= MAX_TYPE_RECURSION_DEPTH {
            return Err(CodegenError::RecursionLimitExceeded(format!(
                "Type recursion depth limit ({}) exceeded in {}",
                MAX_TYPE_RECURSION_DEPTH, context
            )));
        }
        self.type_recursion_depth.set(depth + 1);
        Ok(())
    }

    /// Exit a type recursion level
    #[inline(never)]
    pub(crate) fn exit_type_recursion(&self) {
        let depth = self.type_recursion_depth.get();
        self.type_recursion_depth.set(depth.saturating_sub(1));
    }

    /// Get the size of a type in bytes (for generic operations)
    #[inline(never)]
    pub(crate) fn _type_size(&self, ty: &ResolvedType) -> usize {
        // Track recursion depth
        if self.enter_type_recursion("type_size").is_err() {
            // On recursion limit, return default size
            debug_assert!(false, "ICE: Type recursion limit exceeded in type_size");
            return 8;
        }

        let size = match ty {
            ResolvedType::I8 | ResolvedType::U8 | ResolvedType::Bool => 1,
            ResolvedType::I16 | ResolvedType::U16 => 2,
            ResolvedType::I32 | ResolvedType::U32 | ResolvedType::F32 => 4,
            ResolvedType::I64 | ResolvedType::U64 | ResolvedType::F64 => 8,
            ResolvedType::I128 | ResolvedType::U128 => 16,
            ResolvedType::Str => 8, // Pointer size
            ResolvedType::Pointer(_) | ResolvedType::Ref(_) | ResolvedType::RefMut(_) => 8,
            ResolvedType::Named { name, .. } => {
                // Calculate struct size
                if let Some(info) = self.types.structs.get(name) {
                    info.fields.iter().map(|(_, t)| self._type_size(t)).sum()
                } else {
                    8 // Default to pointer size
                }
            }
            ResolvedType::Generic(param) => {
                // Try to get concrete type from substitutions
                if let Some(concrete) = self.generics.substitutions.get(param) {
                    self._type_size(concrete)
                } else {
                    8 // Default to i64 size
                }
            }
            ResolvedType::DynTrait { .. } => 16, // Fat pointer: data + vtable
            _ => 8,                              // Default
        };

        // Always exit recursion
        self.exit_type_recursion();
        size
    }

    /// Generate allocation call (malloc or gc_alloc depending on GC mode)
    ///
    /// Returns: (result_register, IR code)
    #[inline(never)]
    pub(crate) fn _generate_alloc(
        &self,
        size_arg: &str,
        counter: &mut usize,
        type_id: u32,
    ) -> (String, String) {
        let mut ir = String::new();

        if self.gc_enabled {
            // Use GC allocation
            let ptr_tmp = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = call i8* @vais_gc_alloc(i64 {}, i32 {})",
                ptr_tmp,
                size_arg,
                type_id
            );
            let result = self.next_temp(counter);
            write_ir!(ir, "  {} = ptrtoint i8* {} to i64", result, ptr_tmp);
            (result, ir)
        } else {
            // Use manual malloc
            let ptr_tmp = self.next_temp(counter);
            write_ir!(ir, "  {} = call i8* @malloc(i64 {})", ptr_tmp, size_arg);
            let result = self.next_temp(counter);
            write_ir!(ir, "  {} = ptrtoint i8* {} to i64", result, ptr_tmp);
            (result, ir)
        }
    }

    /// Generate code for a block expression (used in if/else branches)
    #[inline(never)]
    pub(crate) fn _generate_block_expr(
        &mut self,
        expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &expr.node {
            Expr::Block(stmts) => {
                let (val, ir, _terminated) = self.generate_block_stmts(stmts, counter)?;
                Ok((val, ir))
            }
            _ => self.generate_expr(expr, counter),
        }
    }

    /// Generate code for a block of statements
    /// Returns (value, ir_code, is_terminated)
    #[inline(never)]
    pub(crate) fn generate_block_stmts(
        &mut self,
        stmts: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String, bool)> {
        // Use StmtVisitor trait for statement code generation
        use crate::visitor::StmtVisitor;
        self.visit_block_stmts(stmts, counter)
    }

    /// Generate code for array slicing: arr[start..end]
    /// Returns a new array (allocated on heap) containing the slice
    #[inline(never)]
    pub(crate) fn generate_slice(
        &mut self,
        array_expr: &Spanned<Expr>,
        start: Option<&Spanned<Expr>>,
        end: Option<&Spanned<Expr>>,
        inclusive: bool,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (arr_val, arr_ir) = self.generate_expr(array_expr, counter)?;
        let mut ir = arr_ir;

        // Determine if the source is a Slice/SliceMut (fat pointer)
        // Also handle Ref(Slice(_)) and RefMut(SliceMut(_)) — &[T] and &mut [T]
        let arr_type = self.infer_expr_type(array_expr);
        let is_slice_source =
            matches!(arr_type, ResolvedType::Slice(_) | ResolvedType::SliceMut(_))
                || matches!(
                    &arr_type,
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
                    if matches!(inner.as_ref(), ResolvedType::Slice(_) | ResolvedType::SliceMut(_))
                );

        // Detect Vec source: Vec<T>, &Vec<T>, or &mut Vec<T>
        let is_vec_source =
            matches!(&arr_type, ResolvedType::Named { name, .. } if name == "Vec")
                || matches!(
                    &arr_type,
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
                    if matches!(inner.as_ref(), ResolvedType::Named { name, .. } if name == "Vec")
                );

        // Get start index (default 0)
        let start_val = if let Some(start_expr) = start {
            let (val, start_ir) = self.generate_expr(start_expr, counter)?;
            ir.push_str(&start_ir);
            val
        } else {
            "0".to_string()
        };

        // Get end index
        let end_val = if let Some(end_expr) = end {
            let (val, end_ir) = self.generate_expr(end_expr, counter)?;
            ir.push_str(&end_ir);

            // If inclusive (..=), add 1 to end
            if inclusive {
                let adj_end = self.next_temp(counter);
                write_ir!(ir, "  {} = add i64 {}, 1", adj_end, val);
                adj_end
            } else {
                val
            }
        } else {
            // Open-end slice: arr[start..]
            if is_slice_source {
                // Extract length from fat pointer (second field)
                let length = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {{ i8*, i64 }} {}, 1",
                    length,
                    arr_val
                );
                length
            } else if is_vec_source {
                // Vec<T>: extract length from Vec struct field 1 (len)
                // arr_val is a %Vec* pointer
                let len_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr %Vec, %Vec* {}, i32 0, i32 1",
                    len_ptr,
                    arr_val
                );
                let length = self.next_temp(counter);
                write_ir!(ir, "  {} = load i64, i64* {}", length, len_ptr);
                length
            } else if let ResolvedType::ConstArray { size, .. } = &arr_type {
                // ConstArray has a known compile-time size; use it as i64 literal
                if let Some(n) = size.try_evaluate() {
                    n.to_string()
                } else {
                    return Err(CodegenError::Unsupported(
                        "Open-end slicing on ConstArray requires a concrete size".to_string(),
                    ));
                }
            } else {
                // Array/Pointer source doesn't have length information
                return Err(CodegenError::Unsupported(
                    "Open-end slicing requires a slice source; array length is unknown".to_string(),
                ));
            }
        };

        // If source is a slice or Vec, extract the data pointer
        let src_arr_ptr = if is_slice_source {
            let data_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                data_ptr,
                arr_val
            );
            let typed_ptr = self.next_temp(counter);
            write_ir!(ir, "  {} = bitcast i8* {} to i64*", typed_ptr, data_ptr);
            typed_ptr
        } else if is_vec_source {
            // Vec<T>: extract data pointer from Vec struct field 0 (data)
            let data_field = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr %Vec, %Vec* {}, i32 0, i32 0",
                data_field,
                arr_val
            );
            let data_i64 = self.next_temp(counter);
            write_ir!(ir, "  {} = load i64, i64* {}", data_i64, data_field);
            let data_ptr = self.next_temp(counter);
            write_ir!(ir, "  {} = inttoptr i64 {} to i64*", data_ptr, data_i64);
            data_ptr
        } else {
            // For arrays/pointers, use directly
            arr_val.clone()
        };

        // Calculate slice length: end - start
        let length = self.next_temp(counter);
        write_ir!(ir, "  {} = sub i64 {}, {}", length, end_val, start_val);

        // Allocate new array for slice
        let byte_size = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = mul i64 {}, 8", // 8 bytes per i64 element
            byte_size,
            length
        );

        let raw_ptr = self.next_temp(counter);
        write_ir!(ir, "  {} = call i8* @malloc(i64 {})", raw_ptr, byte_size);
        // Track allocation for automatic cleanup at scope exit
        self.track_alloc(raw_ptr.clone());

        let slice_ptr = self.next_temp(counter);
        write_ir!(ir, "  {} = bitcast i8* {} to i64*", slice_ptr, raw_ptr);

        // Copy elements using a loop
        let loop_idx_ptr = self.next_temp(counter);
        self.emit_entry_alloca(&loop_idx_ptr, "i64");
        write_ir!(ir, "  store i64 0, i64* {}", loop_idx_ptr);

        let loop_start = self.next_label("slice_loop");
        let loop_body = self.next_label("slice_body");
        let loop_end = self.next_label("slice_end");

        write_ir!(ir, "  br label %{}", loop_start);
        write_ir!(ir, "{}:", loop_start);

        let loop_idx = self.next_temp(counter);
        write_ir!(ir, "  {} = load i64, i64* {}", loop_idx, loop_idx_ptr);

        let cmp = self.next_temp(counter);
        write_ir!(ir, "  {} = icmp slt i64 {}, {}", cmp, loop_idx, length);
        write_ir!(
            ir,
            "  br i1 {}, label %{}, label %{}",
            cmp,
            loop_body,
            loop_end
        );

        write_ir!(ir, "{}:", loop_body);

        // Calculate source index: start + loop_idx
        let src_idx = self.next_temp(counter);
        write_ir!(ir, "  {} = add i64 {}, {}", src_idx, start_val, loop_idx);

        // Get source element pointer
        let src_ptr = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = getelementptr i64, i64* {}, i64 {}",
            src_ptr,
            src_arr_ptr,
            src_idx
        );

        // Load source element
        let elem = self.next_temp(counter);
        write_ir!(ir, "  {} = load i64, i64* {}", elem, src_ptr);

        // Get destination element pointer
        let dst_ptr = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = getelementptr i64, i64* {}, i64 {}",
            dst_ptr,
            slice_ptr,
            loop_idx
        );

        // Store element
        write_ir!(ir, "  store i64 {}, i64* {}", elem, dst_ptr);

        // Increment loop index
        let next_idx = self.next_temp(counter);
        write_ir!(ir, "  {} = add i64 {}, 1", next_idx, loop_idx);
        write_ir!(ir, "  store i64 {}, i64* {}", next_idx, loop_idx_ptr);
        write_ir!(ir, "  br label %{}", loop_start);

        write_ir!(ir, "{}:", loop_end);

        Ok((slice_ptr, ir))
    }
}
