//! Format expressions

use super::*;

impl Formatter {
    /// Format an expression
    pub(crate) fn format_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(n) => n.to_string(),
            Expr::Float(n) => {
                let s = n.to_string();
                if s.contains('.') {
                    s
                } else {
                    format!("{}.0", s)
                }
            }
            Expr::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            Expr::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            Expr::StringInterp(parts) => {
                let mut result = String::from("\"");
                for part in parts {
                    match part {
                        StringInterpPart::Lit(s) => {
                            result.push_str(&s.replace('\\', "\\\\").replace('"', "\\\""));
                        }
                        StringInterpPart::Expr(e) => {
                            result.push('{');
                            result.push_str(&self.format_expr(&e.node));
                            result.push('}');
                        }
                    }
                }
                result.push('"');
                result
            }
            Expr::Unit => "()".to_string(),
            Expr::Ident(name) => name.to_string(),
            Expr::SelfCall => "@".to_string(),

            Expr::Binary { op, left, right } => {
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Eq => "==",
                    BinOp::Neq => "!=",
                    BinOp::Lt => "<",
                    BinOp::Lte => "<=",
                    BinOp::Gt => ">",
                    BinOp::Gte => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                    BinOp::BitAnd => "&",
                    BinOp::BitOr => "|",
                    BinOp::BitXor => "^",
                    BinOp::Shl => "<<",
                    BinOp::Shr => ">>",
                };
                format!(
                    "{} {} {}",
                    self.format_expr(&left.node),
                    op_str,
                    self.format_expr(&right.node)
                )
            }

            Expr::Unary { op, expr } => {
                let op_str = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                    UnaryOp::BitNot => "~",
                };
                format!("{}{}", op_str, self.format_expr(&expr.node))
            }

            Expr::Ternary { cond, then, else_ } => {
                format!(
                    "{} ? {} : {}",
                    self.format_expr(&cond.node),
                    self.format_expr(&then.node),
                    self.format_expr(&else_.node)
                )
            }

            Expr::If { cond, then, else_ } => {
                let mut s = format!("I {} {{\n", self.format_expr(&cond.node));
                for stmt in then {
                    s.push_str("    ");
                    s.push_str(&self.format_stmt_inline(&stmt.node));
                    s.push('\n');
                }
                s.push('}');

                if let Some(else_branch) = else_ {
                    s.push_str(&self.format_if_else(else_branch));
                }
                s
            }

            Expr::Loop {
                pattern,
                iter,
                body,
            } => {
                let mut s = String::from("L");
                if let Some(pat) = pattern {
                    s.push(' ');
                    s.push_str(&self.format_pattern(&pat.node));
                }
                if let Some(it) = iter {
                    s.push(':');
                    s.push_str(&self.format_expr(&it.node));
                }
                s.push_str(" {\n");
                for stmt in body {
                    s.push_str("    ");
                    s.push_str(&self.format_stmt_inline(&stmt.node));
                    s.push('\n');
                }
                s.push('}');
                s
            }

            Expr::While { condition, body } => {
                let mut s = String::from("L ");
                s.push_str(&self.format_expr(&condition.node));
                s.push_str(" {\n");
                for stmt in body {
                    s.push_str("    ");
                    s.push_str(&self.format_stmt_inline(&stmt.node));
                    s.push('\n');
                }
                s.push('}');
                s
            }

            Expr::Match { expr, arms } => {
                let mut s = format!("M {} {{\n", self.format_expr(&expr.node));
                for arm in arms {
                    s.push_str("    ");
                    s.push_str(&self.format_pattern(&arm.pattern.node));
                    if let Some(guard) = &arm.guard {
                        s.push_str(" I ");
                        s.push_str(&self.format_expr(&guard.node));
                    }
                    s.push_str(" => ");
                    s.push_str(&self.format_expr(&arm.body.node));
                    s.push_str(",\n");
                }
                s.push('}');
                s
            }

            Expr::Call { func, args } => {
                let args_str: Vec<String> =
                    args.iter().map(|a| self.format_expr(&a.node)).collect();
                format!("{}({})", self.format_expr(&func.node), args_str.join(", "))
            }

            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                let args_str: Vec<String> =
                    args.iter().map(|a| self.format_expr(&a.node)).collect();
                format!(
                    "{}.{}({})",
                    self.format_expr(&receiver.node),
                    method.node,
                    args_str.join(", ")
                )
            }

            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => {
                let args_str: Vec<String> =
                    args.iter().map(|a| self.format_expr(&a.node)).collect();
                format!(
                    "{}.{}({})",
                    type_name.node,
                    method.node,
                    args_str.join(", ")
                )
            }

            Expr::Field { expr, field } => {
                format!("{}.{}", self.format_expr(&expr.node), field.node)
            }

            Expr::Index { expr, index } => {
                format!(
                    "{}[{}]",
                    self.format_expr(&expr.node),
                    self.format_expr(&index.node)
                )
            }

            Expr::Array(items) => {
                let items_str: Vec<String> =
                    items.iter().map(|i| self.format_expr(&i.node)).collect();
                format!("[{}]", items_str.join(", "))
            }

            Expr::Tuple(items) => {
                let items_str: Vec<String> =
                    items.iter().map(|i| self.format_expr(&i.node)).collect();
                format!("({})", items_str.join(", "))
            }

            Expr::StructLit { name, fields } => {
                let fields_str: Vec<String> = fields
                    .iter()
                    .map(|(n, v)| format!("{}: {}", n.node, self.format_expr(&v.node)))
                    .collect();
                format!("{} {{ {} }}", name.node, fields_str.join(", "))
            }

            Expr::MapLit(pairs) => {
                let pairs_str: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}: {}",
                            self.format_expr(&k.node),
                            self.format_expr(&v.node)
                        )
                    })
                    .collect();
                format!("{{{}}}", pairs_str.join(", "))
            }

            Expr::Range {
                start,
                end,
                inclusive,
            } => {
                let mut s = String::with_capacity(64);
                if let Some(st) = start {
                    s.push_str(&self.format_expr(&st.node));
                }
                s.push_str(if *inclusive { "..=" } else { ".." });
                if let Some(en) = end {
                    s.push_str(&self.format_expr(&en.node));
                }
                s
            }

            Expr::Block(stmts) => {
                let mut s = String::from("{\n");
                for stmt in stmts {
                    s.push_str("    ");
                    s.push_str(&self.format_stmt_inline(&stmt.node));
                    s.push('\n');
                }
                s.push('}');
                s
            }

            Expr::Await(expr) => format!("{}.Y", self.format_expr(&expr.node)),
            Expr::Try(expr) => format!("{}?", self.format_expr(&expr.node)),
            Expr::Unwrap(expr) => format!("{}!", self.format_expr(&expr.node)),
            Expr::Spread(expr) => format!("..{}", self.format_expr(&expr.node)),
            Expr::Ref(expr) => format!("&{}", self.format_expr(&expr.node)),
            Expr::Deref(expr) => format!("*{}", self.format_expr(&expr.node)),

            Expr::Cast { expr, ty } => {
                format!(
                    "{} as {}",
                    self.format_expr(&expr.node),
                    self.format_type(&ty.node)
                )
            }

            Expr::Assign { target, value } => {
                format!(
                    "{} = {}",
                    self.format_expr(&target.node),
                    self.format_expr(&value.node)
                )
            }

            Expr::AssignOp { op, target, value } => {
                let op_str = match op {
                    BinOp::Add => "+=",
                    BinOp::Sub => "-=",
                    BinOp::Mul => "*=",
                    BinOp::Div => "/=",
                    _ => "?=",
                };
                format!(
                    "{} {} {}",
                    self.format_expr(&target.node),
                    op_str,
                    self.format_expr(&value.node)
                )
            }

            Expr::Lambda { params, body, .. } => {
                let params_str: Vec<String> = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name.node, self.format_type(&p.ty.node)))
                    .collect();
                format!(
                    "|{}| {}",
                    params_str.join(", "),
                    self.format_expr(&body.node)
                )
            }

            Expr::Spawn(expr) => format!("spawn {{ {} }}", self.format_expr(&expr.node)),
            Expr::Yield(expr) => format!("yield {}", self.format_expr(&expr.node)),
            Expr::Comptime { body } => format!("comptime {{ {} }}", self.format_expr(&body.node)),
            Expr::Old(inner) => format!("old({})", self.format_expr(&inner.node)),
            Expr::Assert { condition, message } => {
                if let Some(msg) = message {
                    format!(
                        "assert({}, {})",
                        self.format_expr(&condition.node),
                        self.format_expr(&msg.node)
                    )
                } else {
                    format!("assert({})", self.format_expr(&condition.node))
                }
            }
            Expr::Assume(inner) => format!("assume({})", self.format_expr(&inner.node)),
            Expr::MacroInvoke(invoke) => {
                let delim = match invoke.delimiter {
                    Delimiter::Paren => ('(', ')'),
                    Delimiter::Bracket => ('[', ']'),
                    Delimiter::Brace => ('{', '}'),
                };
                let tokens_str = self.format_macro_tokens(&invoke.tokens);
                format!("{}!{}{}{}", invoke.name.node, delim.0, tokens_str, delim.1)
            }

            Expr::Error { message, .. } => {
                // Format error expressions as comments
                format!("/* ERROR: {} */", message)
            }
            Expr::Lazy(inner) => format!("lazy {}", self.format_expr(&inner.node)),
            Expr::Force(inner) => format!("force {}", self.format_expr(&inner.node)),
        }
    }

    /// Format if-else branch
    pub(crate) fn format_if_else(&self, if_else: &IfElse) -> String {
        match if_else {
            IfElse::ElseIf(cond, stmts, else_) => {
                let mut s = format!(" E I {} {{\n", self.format_expr(&cond.node));
                for stmt in stmts {
                    s.push_str("    ");
                    s.push_str(&self.format_stmt_inline(&stmt.node));
                    s.push('\n');
                }
                s.push('}');
                if let Some(next) = else_ {
                    s.push_str(&self.format_if_else(next));
                }
                s
            }
            IfElse::Else(stmts) => {
                let mut s = String::from(" E {\n");
                for stmt in stmts {
                    s.push_str("    ");
                    s.push_str(&self.format_stmt_inline(&stmt.node));
                    s.push('\n');
                }
                s.push('}');
                s
            }
        }
    }
}
