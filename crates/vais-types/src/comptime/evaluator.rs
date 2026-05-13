//! Core evaluation logic for compile-time expressions

use super::*;

impl ComptimeEvaluator {
    /// Evaluate a comptime expression to a constant value
    pub fn eval(&mut self, expr: &Spanned<Expr>) -> TypeResult<ComptimeValue> {
        match &expr.node {
            Expr::Int(n) => Ok(ComptimeValue::Int(*n)),
            Expr::Float(f) => Ok(ComptimeValue::Float(*f)),
            Expr::Bool(b) => Ok(ComptimeValue::Bool(*b)),
            Expr::String(s) => Ok(ComptimeValue::String(s.clone())),
            Expr::Unit => Ok(ComptimeValue::Unit),

            Expr::Array(elements) => {
                let mut arr = Vec::new();
                for elem in elements {
                    arr.push(self.eval(elem)?);
                }
                Ok(ComptimeValue::Array(arr))
            }

            Expr::Ident(name) => {
                self.vars
                    .get(name)
                    .cloned()
                    .ok_or_else(|| TypeError::UndefinedVar {
                        name: name.clone(),
                        span: Some(expr.span),
                        suggestion: None,
                    })
            }

            Expr::Binary { op, left, right } => {
                let lval = self.eval(left)?;
                let rval = self.eval(right)?;
                self.eval_binary_op(*op, lval, rval, expr.span)
            }

            Expr::Unary { op, expr: inner } => {
                let val = self.eval(inner)?;
                self.eval_unary_op(*op, val, expr.span)
            }

            Expr::If { cond, then, else_ } => {
                let cond_val = self.eval(cond)?;
                if cond_val.as_bool()? {
                    self.eval_block(then)
                } else if let Some(else_branch) = else_ {
                    self.eval_else_branch(else_branch)
                } else {
                    Ok(ComptimeValue::Unit)
                }
            }

            Expr::Loop {
                pattern,
                iter,
                body,
            } => {
                if let (Some(pattern), Some(iter)) = (pattern, iter) {
                    self.eval_for_loop(pattern, iter, body)
                } else {
                    Err(TypeError::Mismatch {
                        expected: "for loop with iterator".to_string(),
                        found: "infinite loop".to_string(),
                        span: Some(expr.span),
                    })
                }
            }

            Expr::Block(stmts) => self.eval_block(stmts),

            Expr::Ternary { cond, then, else_ } => {
                let cond_val = self.eval(cond)?;
                if cond_val.as_bool()? {
                    self.eval(then)
                } else {
                    self.eval(else_)
                }
            }

            Expr::Call { func, args } => self.eval_function_call(func, args, expr.span),

            Expr::Assert { condition, message } => {
                self.eval_assert(condition, message.as_deref(), expr.span)
            }

            Expr::Index { expr: array, index } => {
                let arr_val = self.eval(array)?;
                let idx_val = self.eval(index)?;
                self.eval_index(arr_val, idx_val, expr.span)
            }

            Expr::Comptime { body } => {
                // Nested comptime blocks
                self.eval(body)
            }

            _ => Err(TypeError::Mismatch {
                expected: "comptime-evaluable expression".to_string(),
                found: format!("{:?}", expr.node),
                span: Some(expr.span),
            }),
        }
    }

    pub(crate) fn eval_block(&mut self, stmts: &[Spanned<Stmt>]) -> TypeResult<ComptimeValue> {
        self.push_scope();
        let mut last_value = ComptimeValue::Unit;

        for stmt in stmts {
            match &stmt.node {
                Stmt::Let { name, value, .. } => {
                    let val = self.eval(value)?;
                    self.vars.insert(name.node.clone(), val);
                }
                Stmt::Expr(expr) => {
                    last_value = self.eval(expr)?;
                    // Check for control flow signals
                    if !matches!(self.control_flow, ControlFlow::None) {
                        self.pop_scope();
                        return Ok(last_value);
                    }
                }
                Stmt::Return(Some(expr)) => {
                    last_value = self.eval(expr)?;
                    self.pop_scope();
                    return Ok(last_value);
                }
                Stmt::Return(None) => {
                    self.pop_scope();
                    return Ok(ComptimeValue::Unit);
                }
                Stmt::Break(value_expr) => {
                    let value = if let Some(expr) = value_expr {
                        Some(self.eval(expr)?)
                    } else {
                        None
                    };
                    self.control_flow = ControlFlow::Break(value);
                    self.pop_scope();
                    return Ok(ComptimeValue::Unit);
                }
                Stmt::Continue => {
                    self.control_flow = ControlFlow::Continue;
                    self.pop_scope();
                    return Ok(ComptimeValue::Unit);
                }
                _ => {
                    self.pop_scope();
                    return Err(TypeError::Mismatch {
                        expected: "comptime-evaluable statement".to_string(),
                        found: format!("{:?}", stmt.node),
                        span: Some(stmt.span),
                    });
                }
            }
        }

        self.pop_scope();
        Ok(last_value)
    }

    pub(crate) fn eval_else_branch(&mut self, else_branch: &IfElse) -> TypeResult<ComptimeValue> {
        match else_branch {
            IfElse::Else(stmts) => self.eval_block(stmts),
            IfElse::ElseIf(cond, then, else_) => {
                let cond_val = self.eval(cond)?;
                if cond_val.as_bool()? {
                    self.eval_block(then)
                } else if let Some(else_branch) = else_ {
                    self.eval_else_branch(else_branch)
                } else {
                    Ok(ComptimeValue::Unit)
                }
            }
        }
    }

    pub(crate) fn eval_for_loop(
        &mut self,
        pattern: &Spanned<Pattern>,
        iter: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
    ) -> TypeResult<ComptimeValue> {
        // For now, support Range expressions and Arrays
        // Don't call self.eval(iter) for Range - handle it directly
        match &iter.node {
            Expr::Range {
                start,
                end,
                inclusive,
            } => {
                let start_val = if let Some(s) = start {
                    self.eval(s)?.as_i64()?
                } else {
                    0
                };

                let end_val = if let Some(e) = end {
                    self.eval(e)?.as_i64()?
                } else {
                    return Err(TypeError::Mismatch {
                        expected: "range with end bound".to_string(),
                        found: "unbounded range".to_string(),
                        span: Some(iter.span),
                    });
                };

                // Extract loop variable name from pattern
                let var_name = match &pattern.node {
                    Pattern::Ident(name) => name.clone(),
                    _ => {
                        return Err(TypeError::Mismatch {
                            expected: "simple identifier pattern".to_string(),
                            found: format!("{:?}", pattern.node),
                            span: Some(pattern.span),
                        });
                    }
                };

                let mut last_value = ComptimeValue::Unit;
                let range: Box<dyn Iterator<Item = i64>> = if *inclusive {
                    Box::new(start_val..=end_val)
                } else {
                    Box::new(start_val..end_val)
                };

                for i in range {
                    self.vars.insert(var_name.clone(), ComptimeValue::Int(i));
                    last_value = self.eval_block(body)?;

                    // Handle break/continue
                    match &self.control_flow {
                        ControlFlow::Break(val) => {
                            if let Some(v) = val {
                                last_value = v.clone();
                            }
                            self.control_flow = ControlFlow::None;
                            break;
                        }
                        ControlFlow::Continue => {
                            self.control_flow = ControlFlow::None;
                            continue;
                        }
                        ControlFlow::None => {}
                    }
                }

                Ok(last_value)
            }
            _ => {
                // Try to evaluate the iterator and iterate over it if it's an array
                let iter_val = self.eval(iter)?;
                if let ComptimeValue::Array(arr) = iter_val {
                    let var_name = match &pattern.node {
                        Pattern::Ident(name) => name.clone(),
                        _ => {
                            return Err(TypeError::Mismatch {
                                expected: "simple identifier pattern".to_string(),
                                found: format!("{:?}", pattern.node),
                                span: Some(pattern.span),
                            });
                        }
                    };

                    let mut last_value = ComptimeValue::Unit;
                    for item in arr {
                        self.vars.insert(var_name.clone(), item);
                        last_value = self.eval_block(body)?;

                        // Handle break/continue
                        match &self.control_flow {
                            ControlFlow::Break(val) => {
                                if let Some(v) = val {
                                    last_value = v.clone();
                                }
                                self.control_flow = ControlFlow::None;
                                break;
                            }
                            ControlFlow::Continue => {
                                self.control_flow = ControlFlow::None;
                                continue;
                            }
                            ControlFlow::None => {}
                        }
                    }

                    Ok(last_value)
                } else {
                    Err(TypeError::Mismatch {
                        expected: "range or array expression".to_string(),
                        found: format!("{:?}", iter_val),
                        span: Some(iter.span),
                    })
                }
            }
        }
    }

    pub(crate) fn push_scope(&mut self) {
        self.scope_stack.push(self.vars.clone());
    }

    pub(crate) fn pop_scope(&mut self) {
        if let Some(vars) = self.scope_stack.pop() {
            self.vars = vars;
        }
    }
}
