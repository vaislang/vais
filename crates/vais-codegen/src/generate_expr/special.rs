//! Special expression code generation.
//!
//! Handles Spawn, Comptime, and Range expressions.
//! Note: Try (?) and Unwrap (!) are in expr_helpers_misc.rs.

use vais_ast::*;

use crate::{CodeGenerator, CodegenError, CodegenResult};

impl CodeGenerator {
    /// Generate code for spawn expression: create a concurrent task.
    /// For async function calls, the inner expression already returns a state pointer.
    /// For sync expressions, spawn wraps the value in a Future struct:
    ///   malloc {i64 state=-1, i64 result=value}, return pointer as i64.
    pub(crate) fn generate_spawn_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let inner_type = self.infer_expr_type(inner);
        let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;

        // If inner is already a Future (async call), pass through
        let is_async_call = if let Expr::Call { func, .. } = &inner.node {
            if let Expr::Ident(name) = &func.node {
                self.types
                    .functions
                    .get(name.as_str())
                    .is_some_and(|info| info.signature.is_async)
            } else {
                false
            }
        } else {
            false
        };
        if matches!(inner_type, vais_types::ResolvedType::Future(_)) || is_async_call {
            return Ok((inner_val, inner_ir));
        }

        // Sync value: wrap in an immediate Future state struct {i64 state, i64 result}
        let mut ir = inner_ir;
        let state_ptr = self.next_temp(counter);
        ir.push_str(&format!("  {} = call i64 @malloc(i64 16)\n", state_ptr));
        let typed_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = inttoptr i64 {} to {{i64, i64}}*\n",
            typed_ptr, state_ptr
        ));
        // Store state = -1 (completed)
        let state_field = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {{i64, i64}}, {{i64, i64}}* {}, i32 0, i32 0\n",
            state_field, typed_ptr
        ));
        ir.push_str(&format!("  store i64 -1, i64* {}\n", state_field));
        // Store result value
        let result_field = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {{i64, i64}}, {{i64, i64}}* {}, i32 0, i32 1\n",
            result_field, typed_ptr
        ));
        ir.push_str(&format!(
            "  store i64 {}, i64* {}\n",
            inner_val, result_field
        ));

        self.needs_sync_spawn_poll = true;
        Ok((state_ptr, ir))
    }

    /// Generate code for comptime expression: evaluate at compile time and emit constant.
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
                // Create a global string constant
                let name = self.make_string_name();
                self.strings.counter += 1;
                self.strings.constants.push((name.clone(), s.clone()));
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

                ir.push_str(&format!("  {} = alloca [{} x i64]\n", array_name, len));

                for (i, elem_val) in elements.iter().enumerate() {
                    let elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr [{} x i64], [{} x i64]* {}, i64 0, i64 {}\n",
                        elem_ptr, len, len, array_name, i
                    ));
                    ir.push_str(&format!("  store i64 {}, i64* {}\n", elem_val, elem_ptr));
                }

                Ok((array_name, ir))
            }
            vais_types::ComptimeValue::Unit => Ok(("void".to_string(), String::new())),
        }
    }

    /// Generate code for range expression: `start..end` or `start..=end`
    /// Produces `{ i64 start, i64 end, i1 inclusive }` struct.
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
        ir.push_str(&format!(
            "  {} = insertvalue {} undef, i64 {}, 0\n",
            t1, range_type, start_val
        ));
        let t2 = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = insertvalue {} {}, i64 {}, 1\n",
            t2, range_type, t1, end_val
        ));
        let t3 = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = insertvalue {} {}, i1 {}, 2\n",
            t3, range_type, t2, incl_val
        ));

        Ok((t3, ir))
    }
}
