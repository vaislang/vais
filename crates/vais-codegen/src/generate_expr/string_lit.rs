//! String literal expression code generation.
//!
//! Extracted from `generate_expr_inner` match arms for `Expr::String` and
//! `Expr::StringInterp` to reduce the parent function's stack frame size.
//! Each handler is `#[inline(never)]` so Rust allocates its locals independently.

use vais_ast::*;

use crate::{CodeGenerator, CodegenResult};

impl CodeGenerator {
    /// Generate code for a string literal: creates a global string constant
    /// and wraps it in a fat pointer `{ i8*, i64 }`.
    #[inline(never)]
    pub(crate) fn generate_string_literal_expr(
        &mut self,
        s: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Create a global string constant and wrap in fat pointer { i8*, i64 }
        // Uses dedup cache: identical string literals share the same global constant
        let name = self.get_or_create_string_constant(s);
        let byte_len = s.len() + 1; // includes null terminator for global
        let str_len = s.len() as i64; // actual string length (no null)
        let gep = format!(
            "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
            byte_len, byte_len, name
        );
        // Build fat pointer: { i8* ptr, i64 len }
        let t0 = self.next_temp(counter);
        let t1 = self.next_temp(counter);
        let ir = format!(
            "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0\n  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1\n",
            t0, gep, t1, t0, str_len
        );
        Ok((t1, ir))
    }

    /// Generate code for string interpolation: desugars into a `format()` call.
    #[inline(never)]
    pub(crate) fn generate_string_interp_expr(
        &mut self,
        parts: &[StringInterpPart],
        span: Span,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Desugar string interpolation into a format() call.
        let mut format_str_parts = Vec::with_capacity(parts.len());
        let mut interp_args = Vec::with_capacity(parts.len());
        for part in parts {
            match part {
                StringInterpPart::Lit(s) => {
                    format_str_parts.push(s.clone());
                }
                StringInterpPart::Expr(e) => {
                    format_str_parts.push("{}".to_string());
                    interp_args.push(e.as_ref().clone());
                }
            }
        }
        let fmt_string = format_str_parts.join("");
        let mut args: Vec<Spanned<Expr>> = Vec::with_capacity(interp_args.len() + 1);
        args.push(Spanned::new(Expr::String(fmt_string), span));
        args.extend(interp_args);
        self.generate_format_call(&args, counter, span)
    }
}
