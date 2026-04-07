//! Reference and dereference expression code generation.
//!
//! Extracted from `generate_expr_inner` match arms for `Expr::Ref` and
//! `Expr::Deref` to reduce the parent function's stack frame size.
//! Each handler is `#[inline(never)]` so Rust allocates its locals independently.

use vais_ast::*;
use vais_types::ResolvedType;

use crate::{CodeGenerator, CodegenResult};

impl CodeGenerator {
    /// Generate code for a reference expression (`&expr`).
    /// Handles array-to-slice conversion, ident references, and general expressions.
    #[inline(never)]
    pub(crate) fn generate_ref_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Special case: &[elem, ...] array literal -> slice fat pointer { i8*, i64 }
        if let Expr::Array(elements) = &inner.node {
            return self.generate_ref_array_slice(elements, counter);
        }

        // For simple references, just return the address
        if let Expr::Ident(name) = &inner.node {
            if let Some(local) = self.fn_ctx.locals.get(name.as_str()).cloned() {
                if local.is_alloca() {
                    // Alloca variables already have an address.
                    return Ok((format!("%{}", local.llvm_name), String::new()));
                } else {
                    // SSA/Param values need to be spilled to stack to take their address
                    return self.generate_ref_spill(&local, inner, counter);
                }
            }
        }
        // For complex expressions, evaluate and return
        self.generate_expr(inner, counter)
    }

    /// Generate a slice fat pointer `{ i8*, i64 }` from `&[elem, ...]`.
    #[inline(never)]
    fn generate_ref_array_slice(
        &mut self,
        elements: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let len = elements.len();
        let mut ir = String::new();

        // Infer element type
        let elem_ty = if let Some(first) = elements.first() {
            let resolved = self.infer_expr_type(first);
            self.type_to_llvm(&resolved)
        } else {
            "i64".to_string()
        };
        let arr_ty = format!("[{}  x {}]", len, elem_ty);

        // Allocate array on stack (hoisted to entry block)
        let arr_ptr = self.next_temp(counter);
        self.emit_entry_alloca(&arr_ptr, &arr_ty);

        // Store each element
        for (i, elem) in elements.iter().enumerate() {
            let (val, elem_ir) = self.generate_expr(elem, counter)?;
            ir.push_str(&elem_ir);

            let elem_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {}* {}, i64 0, i64 {}",
                elem_ptr,
                arr_ty,
                arr_ty,
                arr_ptr,
                i
            );
            write_ir!(ir, "  store {} {}, {}* {}", elem_ty, val, elem_ty, elem_ptr);
        }

        // Get pointer to first element
        let data_ptr = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = getelementptr {}, {}* {}, i64 0, i64 0",
            data_ptr,
            arr_ty,
            arr_ty,
            arr_ptr
        );

        // Bitcast to i8*
        let data_i8 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = bitcast {}* {} to i8*",
            data_i8,
            elem_ty,
            data_ptr
        );

        // Build fat pointer: { i8*, i64 }
        let fat1 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
            fat1,
            data_i8
        );
        let fat2 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1",
            fat2,
            fat1,
            len
        );

        Ok((fat2, ir))
    }

    /// Spill an SSA/Param value to stack to take its address.
    #[inline(never)]
    fn generate_ref_spill(
        &mut self,
        local: &crate::types::LocalVar,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let llvm_ty = self.type_to_llvm(&local.ty);
        let (val, val_ir) = self.generate_expr(inner, counter)?;
        ir.push_str(&val_ir);

        // If the local is a struct-typed SSA value whose LLVM name is an alloca
        // pointer (e.g., %__test_ptr from function entry, or %self for methods),
        // the value is already a pointer to the struct — return it directly.
        // The function entry code creates alloca + store for struct params, and
        // registers them as SSA with Named type but the value is %StructType*.
        // Also handles Param locals (e.g., `self` in methods passed as %Struct*).
        if matches!(&local.ty, ResolvedType::Named { .. }) && (local.is_ssa() || local.is_param()) {
            // The value is already a pointer (alloca result or self param pointer)
            return Ok((val, ir));
        }

        let tmp_alloca = self.next_temp(counter);
        self.emit_entry_alloca(&tmp_alloca, &llvm_ty);
        write_ir!(
            ir,
            "  store {} {}, {}* {}",
            llvm_ty,
            val,
            llvm_ty,
            tmp_alloca
        );
        Ok((tmp_alloca, ir))
    }

    /// Generate code for a dereference expression (`*expr`).
    #[inline(never)]
    pub(crate) fn generate_deref_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (ptr_val, ptr_ir) = self.generate_expr(inner, counter)?;
        let mut ir = ptr_ir;

        // Infer the pointee type from the pointer expression
        let ptr_type = self.infer_expr_type(inner);
        let pointee_llvm = match &ptr_type {
            ResolvedType::Pointer(inner) => self.type_to_llvm(inner),
            ResolvedType::Ref(inner) => self.type_to_llvm(inner),
            ResolvedType::RefMut(inner) => self.type_to_llvm(inner),
            _ => "i64".to_string(),
        };

        let result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = load {}, {}* {}",
            result,
            pointee_llvm,
            pointee_llvm,
            ptr_val
        );

        Ok((result, ir))
    }
}
