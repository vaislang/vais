//! Miscellaneous code generation helper methods

use super::*;

impl CodeGenerator {
    /// Generate a unique string constant name, with optional module prefix
    pub(crate) fn make_string_name(&self) -> String {
        if let Some(ref prefix) = self.strings.prefix {
            format!("{}.str.{}", prefix, self.strings.counter)
        } else {
            format!(".str.{}", self.strings.counter)
        }
    }

    /// Generate a unique label with the given prefix
    pub(crate) fn next_label(&mut self, prefix: &str) -> String {
        debug_assert!(
            !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_'),
            "Invalid label prefix: '{}'. Must be non-empty and contain only alphanumeric, '.', or '_' characters.",
            prefix
        );
        let label = format!("{}{}", prefix, self.fn_ctx.label_counter);
        self.fn_ctx.label_counter += 1;
        label
    }

    /// Generate a unique temporary register name
    pub(crate) fn next_temp(&self, counter: &mut usize) -> String {
        let tmp = format!("%t{}", counter);
        *counter += 1;
        tmp
    }

    /// Check if a function call is recursive (calls the current function with decreases clause)
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
    pub(crate) fn _has_gc_attribute(attributes: &[Attribute]) -> bool {
        attributes.iter().any(|attr| attr.name == "gc")
    }

    /// Enter a type recursion level and check depth limit
    /// Returns an error if recursion limit is exceeded
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
    pub(crate) fn exit_type_recursion(&self) {
        let depth = self.type_recursion_depth.get();
        self.type_recursion_depth.set(depth.saturating_sub(1));
    }

    /// Get the size of a type in bytes (for generic operations)
    pub(crate) fn _type_size(&self, ty: &ResolvedType) -> usize {
        // Track recursion depth
        if self.enter_type_recursion("type_size").is_err() {
            // On recursion limit, return default size
            #[cfg(debug_assertions)]
            eprintln!("Warning: Type recursion limit exceeded in type_size");
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
            ir.push_str(&format!(
                "  {} = call i8* @vais_gc_alloc(i64 {}, i32 {})\n",
                ptr_tmp, size_arg, type_id
            ));
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, ptr_tmp));
            (result, ir)
        } else {
            // Use manual malloc
            let ptr_tmp = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = call i8* @malloc(i64 {})\n",
                ptr_tmp, size_arg
            ));
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, ptr_tmp));
            (result, ir)
        }
    }

    /// Generate code for a block expression (used in if/else branches)
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
        let arr_type = self.infer_expr_type(array_expr);
        let is_slice_source =
            matches!(arr_type, ResolvedType::Slice(_) | ResolvedType::SliceMut(_));

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
                ir.push_str(&format!("  {} = add i64 {}, 1\n", adj_end, val));
                adj_end
            } else {
                val
            }
        } else {
            // Open-end slice: arr[start..]
            if is_slice_source {
                // Extract length from fat pointer (second field)
                let length = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {{ i8*, i64 }} {}, 1\n",
                    length, arr_val
                ));
                length
            } else {
                // Array/Pointer source doesn't have length information
                return Err(CodegenError::Unsupported(
                    "Open-end slicing requires a slice source; array length is unknown".to_string(),
                ));
            }
        };

        // If source is a slice, extract the data pointer
        let src_arr_ptr = if is_slice_source {
            let data_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = extractvalue {{ i8*, i64 }} {}, 0\n",
                data_ptr, arr_val
            ));
            let typed_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = bitcast i8* {} to i64*\n",
                typed_ptr, data_ptr
            ));
            typed_ptr
        } else {
            // For arrays/pointers, use directly
            arr_val.clone()
        };

        // Calculate slice length: end - start
        let length = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = sub i64 {}, {}\n",
            length, end_val, start_val
        ));

        // Allocate new array for slice
        let byte_size = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = mul i64 {}, 8\n", // 8 bytes per i64 element
            byte_size, length
        ));

        let raw_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i8* @malloc(i64 {})\n",
            raw_ptr, byte_size
        ));

        let slice_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = bitcast i8* {} to i64*\n",
            slice_ptr, raw_ptr
        ));

        // Copy elements using a loop
        let loop_idx_ptr = self.next_temp(counter);
        ir.push_str(&format!("  {} = alloca i64\n", loop_idx_ptr));
        ir.push_str(&format!("  store i64 0, i64* {}\n", loop_idx_ptr));

        let loop_start = self.next_label("slice_loop");
        let loop_body = self.next_label("slice_body");
        let loop_end = self.next_label("slice_end");

        ir.push_str(&format!("  br label %{}\n", loop_start));
        ir.push_str(&format!("{}:\n", loop_start));

        let loop_idx = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load i64, i64* {}\n",
            loop_idx, loop_idx_ptr
        ));

        let cmp = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = icmp slt i64 {}, {}\n",
            cmp, loop_idx, length
        ));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cmp, loop_body, loop_end
        ));

        ir.push_str(&format!("{}:\n", loop_body));

        // Calculate source index: start + loop_idx
        let src_idx = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = add i64 {}, {}\n",
            src_idx, start_val, loop_idx
        ));

        // Get source element pointer
        let src_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr i64, i64* {}, i64 {}\n",
            src_ptr, src_arr_ptr, src_idx
        ));

        // Load source element
        let elem = self.next_temp(counter);
        ir.push_str(&format!("  {} = load i64, i64* {}\n", elem, src_ptr));

        // Get destination element pointer
        let dst_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr i64, i64* {}, i64 {}\n",
            dst_ptr, slice_ptr, loop_idx
        ));

        // Store element
        ir.push_str(&format!("  store i64 {}, i64* {}\n", elem, dst_ptr));

        // Increment loop index
        let next_idx = self.next_temp(counter);
        ir.push_str(&format!("  {} = add i64 {}, 1\n", next_idx, loop_idx));
        ir.push_str(&format!(
            "  store i64 {}, i64* {}\n",
            next_idx, loop_idx_ptr
        ));
        ir.push_str(&format!("  br label %{}\n", loop_start));

        ir.push_str(&format!("{}:\n", loop_end));

        Ok((slice_ptr, ir))
    }
}
