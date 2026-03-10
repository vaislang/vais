//! Built-in function evaluation and assertion/indexing for compile-time expressions

use super::*;

impl ComptimeEvaluator {
    pub(crate) fn eval_function_call(
        &mut self,
        func: &Spanned<Expr>,
        args: &[Spanned<Expr>],
        span: Span,
    ) -> TypeResult<ComptimeValue> {
        // Extract function name
        let func_name = match &func.node {
            Expr::Ident(name) => name.as_str(),
            _ => {
                return Err(TypeError::Mismatch {
                    expected: "function name".to_string(),
                    found: format!("{:?}", func.node),
                    span: Some(span),
                });
            }
        };

        // Evaluate arguments
        let mut arg_vals = Vec::new();
        for arg in args {
            arg_vals.push(self.eval(arg)?);
        }

        // Call built-in functions
        match func_name {
            "abs" => {
                if arg_vals.len() != 1 {
                    return Err(TypeError::Mismatch {
                        expected: "1 argument".to_string(),
                        found: format!("{} arguments", arg_vals.len()),
                        span: Some(span),
                    });
                }
                match &arg_vals[0] {
                    ComptimeValue::Int(n) => Ok(ComptimeValue::Int(n.abs())),
                    ComptimeValue::Float(f) => Ok(ComptimeValue::Float(f.abs())),
                    _ => Err(TypeError::Mismatch {
                        expected: "numeric value".to_string(),
                        found: format!("{:?}", arg_vals[0]),
                        span: Some(span),
                    }),
                }
            }
            "min" => {
                if arg_vals.len() != 2 {
                    return Err(TypeError::Mismatch {
                        expected: "2 arguments".to_string(),
                        found: format!("{} arguments", arg_vals.len()),
                        span: Some(span),
                    });
                }
                match (&arg_vals[0], &arg_vals[1]) {
                    (ComptimeValue::Int(a), ComptimeValue::Int(b)) => {
                        Ok(ComptimeValue::Int(*a.min(b)))
                    }
                    (ComptimeValue::Float(a), ComptimeValue::Float(b)) => {
                        Ok(ComptimeValue::Float(a.min(*b)))
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "two numeric values".to_string(),
                        found: format!("{:?}, {:?}", arg_vals[0], arg_vals[1]),
                        span: Some(span),
                    }),
                }
            }
            "max" => {
                if arg_vals.len() != 2 {
                    return Err(TypeError::Mismatch {
                        expected: "2 arguments".to_string(),
                        found: format!("{} arguments", arg_vals.len()),
                        span: Some(span),
                    });
                }
                match (&arg_vals[0], &arg_vals[1]) {
                    (ComptimeValue::Int(a), ComptimeValue::Int(b)) => {
                        Ok(ComptimeValue::Int(*a.max(b)))
                    }
                    (ComptimeValue::Float(a), ComptimeValue::Float(b)) => {
                        Ok(ComptimeValue::Float(a.max(*b)))
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "two numeric values".to_string(),
                        found: format!("{:?}, {:?}", arg_vals[0], arg_vals[1]),
                        span: Some(span),
                    }),
                }
            }
            "pow" => {
                if arg_vals.len() != 2 {
                    return Err(TypeError::Mismatch {
                        expected: "2 arguments".to_string(),
                        found: format!("{} arguments", arg_vals.len()),
                        span: Some(span),
                    });
                }
                match (&arg_vals[0], &arg_vals[1]) {
                    (ComptimeValue::Int(base), ComptimeValue::Int(exp)) => {
                        if *exp < 0 {
                            return Err(TypeError::Mismatch {
                                expected: "non-negative exponent".to_string(),
                                found: format!("{}", exp),
                                span: Some(span),
                            });
                        }
                        Ok(ComptimeValue::Int(base.pow(*exp as u32)))
                    }
                    (ComptimeValue::Float(base), ComptimeValue::Float(exp)) => {
                        Ok(ComptimeValue::Float(base.powf(*exp)))
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "two numeric values".to_string(),
                        found: format!("{:?}, {:?}", arg_vals[0], arg_vals[1]),
                        span: Some(span),
                    }),
                }
            }
            "len" => {
                if arg_vals.len() != 1 {
                    return Err(TypeError::Mismatch {
                        expected: "1 argument".to_string(),
                        found: format!("{} arguments", arg_vals.len()),
                        span: Some(span),
                    });
                }
                match &arg_vals[0] {
                    ComptimeValue::String(s) => Ok(ComptimeValue::Int(s.chars().count() as i64)),
                    ComptimeValue::Array(arr) => Ok(ComptimeValue::Int(arr.len() as i64)),
                    _ => Err(TypeError::Mismatch {
                        expected: "string or array".to_string(),
                        found: format!("{:?}", arg_vals[0]),
                        span: Some(span),
                    }),
                }
            }
            _ => Err(TypeError::Mismatch {
                expected: "built-in comptime function (abs, min, max, pow, len)".to_string(),
                found: func_name.to_string(),
                span: Some(span),
            }),
        }
    }

    pub(crate) fn eval_assert(
        &mut self,
        condition: &Spanned<Expr>,
        message: Option<&Spanned<Expr>>,
        span: Span,
    ) -> TypeResult<ComptimeValue> {
        let cond_val = self.eval(condition)?;
        let cond_bool = cond_val.as_bool()?;

        if !cond_bool {
            let msg = if let Some(msg_expr) = message {
                let msg_val = self.eval(msg_expr)?;
                match msg_val {
                    ComptimeValue::String(s) => s,
                    _ => format!("{}", msg_val),
                }
            } else {
                "comptime assertion failed".to_string()
            };

            return Err(TypeError::Mismatch {
                expected: "true".to_string(),
                found: format!("assertion failed: {}", msg),
                span: Some(span),
            });
        }

        Ok(ComptimeValue::Unit)
    }

    pub(crate) fn eval_index(
        &self,
        array: ComptimeValue,
        index: ComptimeValue,
        span: Span,
    ) -> TypeResult<ComptimeValue> {
        let idx = index.as_i64()?;

        match array {
            ComptimeValue::Array(arr) => {
                if idx < 0 || idx >= arr.len() as i64 {
                    return Err(TypeError::Mismatch {
                        expected: format!("index in range 0..{}", arr.len()),
                        found: format!("{}", idx),
                        span: Some(span),
                    });
                }
                Ok(arr[idx as usize].clone())
            }
            ComptimeValue::String(s) => {
                let char_count = s.chars().count();
                if idx < 0 || idx >= char_count as i64 {
                    return Err(TypeError::Mismatch {
                        expected: format!("index in range 0..{}", char_count),
                        found: format!("{}", idx),
                        span: Some(span),
                    });
                }
                // safe: bounds check above guarantees idx is valid for byte length,
                // but chars() iterates codepoints. Use unwrap_or for safety.
                let ch = s
                    .chars()
                    .nth(idx as usize)
                    .ok_or_else(|| TypeError::Mismatch {
                        expected: format!("valid char index in range 0..{}", s.chars().count()),
                        found: format!("{}", idx),
                        span: Some(span),
                    })?;
                Ok(ComptimeValue::String(ch.to_string()))
            }
            _ => Err(TypeError::Mismatch {
                expected: "array or string".to_string(),
                found: format!("{:?}", array),
                span: Some(span),
            }),
        }
    }
}
