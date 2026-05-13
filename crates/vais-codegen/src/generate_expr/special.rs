//! Special expression code generation.
//!
//! Handles Comptime and Range expressions.
//! Note: Try (?) and Unwrap (!) are in expr_helpers_misc.rs.

use vais_ast::*;

use crate::{CodeGenerator, CodegenError, CodegenResult};

impl CodeGenerator {
    /// Generate code for comptime expression: evaluate at compile time and emit constant.
    #[inline(never)]
    pub(crate) fn generate_comptime_expr(
        &mut self,
        body: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Evaluate at compile time
        let mut evaluator = vais_types::ComptimeEvaluator::new();
        let value = evaluator
            .eval(body)
            .map_err(|e| CodegenError::TypeError(format!("Comptime evaluation failed: {}", e)))?;

        // Return the evaluated constant
        match value {
            vais_types::ComptimeValue::Int(n) => Ok((n.to_string(), String::new())),
            vais_types::ComptimeValue::Float(f) => {
                Ok((crate::types::format_llvm_float(f), String::new()))
            }
            vais_types::ComptimeValue::Bool(b) => {
                Ok((if b { "1" } else { "0" }.to_string(), String::new()))
            }
            vais_types::ComptimeValue::String(s) => {
                // Create a global string constant (deduplicated)
                let name = self.get_or_create_string_constant(&s);
                let len = s.len() + 1;
                Ok((
                    format!(
                        "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                        len, len, name
                    ),
                    String::new(),
                ))
            }
            vais_types::ComptimeValue::Array(arr) => {
                // Generate array literal from comptime array
                let mut elements = Vec::new();
                let mut ir = String::new();

                for elem in arr {
                    match elem {
                        vais_types::ComptimeValue::Int(n) => elements.push(n.to_string()),
                        vais_types::ComptimeValue::Float(f) => {
                            elements.push(crate::types::format_llvm_float(f))
                        }
                        vais_types::ComptimeValue::Bool(b) => {
                            elements.push(if b { "1" } else { "0" }.to_string())
                        }
                        _ => {
                            return Err(CodegenError::TypeError(
                                "Comptime arrays can only contain simple values (int, float, bool)"
                                    .to_string(),
                            ));
                        }
                    }
                }

                // Create array on the stack
                let array_name = format!("%comptime_array_{}", counter);
                *counter += 1;
                let len = elements.len();

                self.emit_entry_alloca(&array_name, &format!("[{} x i64]", len));

                for (i, elem_val) in elements.iter().enumerate() {
                    let elem_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr [{} x i64], [{} x i64]* {}, i64 0, i64 {}",
                        elem_ptr,
                        len,
                        len,
                        array_name,
                        i
                    );
                    write_ir!(ir, "  store i64 {}, i64* {}", elem_val, elem_ptr);
                }

                Ok((array_name, ir))
            }
            vais_types::ComptimeValue::Unit => Ok(("void".to_string(), String::new())),
        }
    }

    /// Generate code for range expression: `start..end` or `start..=end`
    /// Produces `{ i64 start, i64 end, i1 inclusive }` struct.
    #[inline(never)]
    pub(crate) fn generate_range_expr(
        &mut self,
        start: &Option<Box<Spanned<Expr>>>,
        end: &Option<Box<Spanned<Expr>>>,
        inclusive: bool,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();

        let (start_val, start_ir) = if let Some(s) = start {
            self.generate_expr(s, counter)?
        } else {
            ("0".to_string(), String::new())
        };
        ir.push_str(&start_ir);

        let (end_val, end_ir) = if let Some(e) = end {
            self.generate_expr(e, counter)?
        } else {
            (format!("{}", i64::MAX), String::new())
        };
        ir.push_str(&end_ir);

        let incl_val = if inclusive { "1" } else { "0" };

        // Build struct via insertvalue chain
        let range_type = "{ i64, i64, i1 }";
        let t1 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {} undef, i64 {}, 0",
            t1,
            range_type,
            start_val
        );
        let t2 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {} {}, i64 {}, 1",
            t2,
            range_type,
            t1,
            end_val
        );
        let t3 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {} {}, i1 {}, 2",
            t3,
            range_type,
            t2,
            incl_val
        );

        Ok((t3, ir))
    }
}
