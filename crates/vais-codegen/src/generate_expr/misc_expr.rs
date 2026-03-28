//! Miscellaneous expression code generation.
//!
//! Extracted from `generate_expr_inner` match arms for `Expr::Old`, `Expr::Assume`,
//! `Expr::Yield`, and `Expr::EnumAccess` (with data) to reduce the parent function's
//! stack frame size.
//! Each handler is `#[inline(never)]` so Rust allocates its locals independently.

use vais_ast::*;

use crate::{CodeGenerator, CodegenResult};

impl CodeGenerator {
    /// Generate code for `old(expr)` — references a pre-snapshot value in contract ensures.
    #[inline(never)]
    pub(crate) fn generate_old_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // old(expr) references a pre-snapshot value
        let old_var_name = format!("__old_{}", counter);
        *counter += 1;

        if let Some(snapshot_var) = self.contracts.old_snapshots.get(&old_var_name) {
            let ty = self.infer_expr_type(inner);
            let llvm_ty = self.type_to_llvm(&ty);
            let result = self.next_temp(counter);
            let ir = format!(
                "  {} = load {}, {}* %{}\n",
                result, llvm_ty, llvm_ty, snapshot_var
            );
            Ok((result, ir))
        } else {
            // Fallback: just evaluate the expression (for non-ensures contexts)
            self.generate_expr(inner, counter)
        }
    }

    /// Generate code for `assume(expr)` — verification hint, no runtime effect in release.
    #[inline(never)]
    pub(crate) fn generate_assume_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        if self.release_mode {
            // In release mode, assume is a no-op
            Ok(("0".to_string(), String::new()))
        } else {
            // In debug mode, assume acts like assert but with different error message
            self.generate_assume(inner, counter)
        }
    }

    /// Generate code for `yield expr` — yields a value from a generator.
    #[inline(never)]
    pub(crate) fn generate_yield_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (val, ir) = self.generate_expr(inner, counter)?;
        Ok((val, ir))
    }

    /// Generate code for `EnumName::Variant(data)` — enum variant with data payload.
    #[inline(never)]
    pub(crate) fn generate_enum_access_data_expr(
        &mut self,
        variant: &str,
        data_expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let callee = Spanned::new(Expr::Ident(variant.to_string()), data_expr.span);
        let args = vec![data_expr.clone()];
        self.generate_call_expr(&callee, &args, counter, data_expr.span)
    }
}
