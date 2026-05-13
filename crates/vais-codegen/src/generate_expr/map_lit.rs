//! Map literal expression code generation.
//!
//! Extracted from `generate_expr_inner` match arm for `Expr::MapLit` to reduce
//! the parent function's stack frame size.
//! This handler is `#[inline(never)]` so Rust allocates its locals independently.

use vais_ast::*;

use crate::{CodeGenerator, CodegenResult};

impl CodeGenerator {
    /// Generate code for a map literal: `{k: v, ...}`.
    /// Stored as parallel arrays of keys and values on the stack.
    #[inline(never)]
    pub(crate) fn generate_map_lit_expr(
        &mut self,
        pairs: &[(Spanned<Expr>, Spanned<Expr>)],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let len = pairs.len();

        // Infer key/value types
        let (key_ty, val_ty) = if let Some((k, v)) = pairs.first() {
            let kt = self.type_to_llvm(&self.infer_expr_type(k));
            let vt = self.type_to_llvm(&self.infer_expr_type(v));
            (kt, vt)
        } else {
            ("i64".to_string(), "i64".to_string())
        };

        let keys_arr_ty = format!("[{} x {}]", len, key_ty);
        let vals_arr_ty = format!("[{} x {}]", len, val_ty);

        // Allocate key and value arrays on stack (hoisted to entry block)
        let keys_ptr = self.next_temp(counter);
        self.emit_entry_alloca(&keys_ptr, &keys_arr_ty);
        let vals_ptr = self.next_temp(counter);
        self.emit_entry_alloca(&vals_ptr, &vals_arr_ty);

        // Store each key-value pair
        for (i, (k, v)) in pairs.iter().enumerate() {
            let (kval, k_ir) = self.generate_expr(k, counter)?;
            ir.push_str(&k_ir);
            let k_elem_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {}* {}, i64 0, i64 {}",
                k_elem_ptr,
                keys_arr_ty,
                keys_arr_ty,
                keys_ptr,
                i
            );
            write_ir!(
                ir,
                "  store {} {}, {}* {}",
                key_ty,
                kval,
                key_ty,
                k_elem_ptr
            );

            let (vval, v_ir) = self.generate_expr(v, counter)?;
            ir.push_str(&v_ir);
            let v_elem_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {}* {}, i64 0, i64 {}",
                v_elem_ptr,
                vals_arr_ty,
                vals_arr_ty,
                vals_ptr,
                i
            );
            write_ir!(
                ir,
                "  store {} {}, {}* {}",
                val_ty,
                vval,
                val_ty,
                v_elem_ptr
            );
        }

        // Return pointer to keys array (map is represented as parallel arrays)
        let result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = getelementptr {}, {}* {}, i64 0, i64 0",
            result,
            keys_arr_ty,
            keys_arr_ty,
            keys_ptr
        );

        Ok((result, ir))
    }
}
