//! Format statements

use super::*;

impl Formatter {
    /// Format a statement
    pub(crate) fn format_stmt(&mut self, stmt: &Stmt) {
        let indent = self.indent();

        match stmt {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut,
                ..
            } => {
                self.output.push_str(&indent);
                if *is_mut {
                    self.output.push_str("mut ");
                }
                self.output.push_str(&name.node);
                if let Some(t) = ty {
                    self.output.push_str(": ");
                    self.output.push_str(&self.format_type(&t.node));
                    self.output.push_str(" = ");
                } else {
                    self.output.push_str(" := ");
                }
                self.output.push_str(&self.format_expr(&value.node));
                self.output.push('\n');
            }
            Stmt::LetDestructure {
                pattern,
                value,
                is_mut,
            } => {
                self.output.push_str(&indent);
                if *is_mut {
                    self.output.push_str("mut ");
                }
                self.output.push_str(&self.format_pattern(&pattern.node));
                self.output.push_str(" := ");
                self.output.push_str(&self.format_expr(&value.node));
                self.output.push('\n');
            }
            Stmt::Expr(expr) => {
                // Handle if/loop/match/block specially for proper indentation
                match &expr.node {
                    Expr::If { cond, then, else_ } => {
                        self.format_if_expr(&indent, cond, then, else_.as_ref());
                    }
                    Expr::Loop {
                        pattern,
                        iter,
                        body,
                    } => {
                        self.format_loop_expr(&indent, pattern, iter.as_ref().map(|e| &**e), body);
                    }
                    Expr::While { condition, body } => {
                        self.format_while_expr(&indent, condition, body);
                    }
                    Expr::Match {
                        expr: match_expr,
                        arms,
                    } => {
                        self.format_match_expr(&indent, match_expr, arms);
                    }
                    Expr::Block(stmts) => {
                        self.output.push_str(&indent);
                        self.output.push_str("{\n");
                        self.push_indent();
                        for s in stmts {
                            self.format_stmt(&s.node);
                        }
                        self.pop_indent();
                        self.output.push_str(&self.indent());
                        self.output.push_str("}\n");
                    }
                    _ => {
                        self.output.push_str(&indent);
                        self.output.push_str(&self.format_expr(&expr.node));
                        self.output.push('\n');
                    }
                }
            }
            Stmt::Return(expr) => {
                self.output.push_str(&indent);
                self.output.push('R');
                if let Some(e) = expr {
                    self.output.push(' ');
                    self.output.push_str(&self.format_expr(&e.node));
                }
                self.output.push('\n');
            }
            Stmt::Break(expr) => {
                self.output.push_str(&indent);
                self.output.push('B');
                if let Some(e) = expr {
                    self.output.push(' ');
                    self.output.push_str(&self.format_expr(&e.node));
                }
                self.output.push('\n');
            }
            Stmt::Continue => {
                self.output.push_str(&indent);
                self.output.push_str("C\n");
            }
            Stmt::Defer(expr) => {
                self.output.push_str(&indent);
                self.output.push_str("D ");
                self.output.push_str(&self.format_expr(&expr.node));
                self.output.push('\n');
            }
            Stmt::Error { message, .. } => {
                // Format error nodes as comments
                self.output.push_str(&indent);
                self.output.push_str(&format!("# ERROR: {}\n", message));
            }
        }
    }

    /// Format an if expression with proper indentation
    pub(crate) fn format_if_expr(
        &mut self,
        indent: &str,
        cond: &Spanned<Expr>,
        then: &[Spanned<Stmt>],
        else_: Option<&IfElse>,
    ) {
        self.output.push_str(indent);
        self.output.push_str("I ");
        self.output.push_str(&self.format_expr(&cond.node));
        self.output.push_str(" {\n");
        self.push_indent();
        for stmt in then {
            self.format_stmt(&stmt.node);
        }
        self.pop_indent();
        self.output.push_str(&self.indent());
        self.output.push('}');

        if let Some(else_branch) = else_ {
            self.format_if_else_branch(else_branch);
        }
        self.output.push('\n');
    }

    /// Format else/else-if branches
    pub(crate) fn format_if_else_branch(&mut self, if_else: &IfElse) {
        match if_else {
            IfElse::ElseIf(cond, stmts, else_) => {
                self.output.push_str(" E I ");
                self.output.push_str(&self.format_expr(&cond.node));
                self.output.push_str(" {\n");
                self.push_indent();
                for stmt in stmts {
                    self.format_stmt(&stmt.node);
                }
                self.pop_indent();
                self.output.push_str(&self.indent());
                self.output.push('}');
                if let Some(next) = else_ {
                    self.format_if_else_branch(next);
                }
            }
            IfElse::Else(stmts) => {
                self.output.push_str(" E {\n");
                self.push_indent();
                for stmt in stmts {
                    self.format_stmt(&stmt.node);
                }
                self.pop_indent();
                self.output.push_str(&self.indent());
                self.output.push('}');
            }
        }
    }

    /// Format a loop expression with proper indentation
    pub(crate) fn format_loop_expr(
        &mut self,
        indent: &str,
        pattern: &Option<Spanned<Pattern>>,
        iter: Option<&Spanned<Expr>>,
        body: &[Spanned<Stmt>],
    ) {
        self.output.push_str(indent);
        self.output.push('L');
        if let Some(pat) = pattern {
            self.output.push(' ');
            self.output.push_str(&self.format_pattern(&pat.node));
        }
        if let Some(it) = iter {
            self.output.push(':');
            self.output.push_str(&self.format_expr(&it.node));
        }
        self.output.push_str(" {\n");
        self.push_indent();
        for stmt in body {
            self.format_stmt(&stmt.node);
        }
        self.pop_indent();
        self.output.push_str(&self.indent());
        self.output.push_str("}\n");
    }

    /// Format a while loop expression with proper indentation
    pub(crate) fn format_while_expr(
        &mut self,
        indent: &str,
        condition: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
    ) {
        self.output.push_str(indent);
        self.output.push_str("L ");
        self.output.push_str(&self.format_expr(&condition.node));
        self.output.push_str(" {\n");
        self.push_indent();
        for stmt in body {
            self.format_stmt(&stmt.node);
        }
        self.pop_indent();
        self.output.push_str(&self.indent());
        self.output.push_str("}\n");
    }

    /// Format a match expression with proper indentation
    pub(crate) fn format_match_expr(&mut self, indent: &str, expr: &Spanned<Expr>, arms: &[MatchArm]) {
        self.output.push_str(indent);
        self.output.push_str("M ");
        self.output.push_str(&self.format_expr(&expr.node));
        self.output.push_str(" {\n");
        self.push_indent();
        for arm in arms {
            self.output.push_str(&self.indent());
            self.output
                .push_str(&self.format_pattern(&arm.pattern.node));
            if let Some(guard) = &arm.guard {
                self.output.push_str(" I ");
                self.output.push_str(&self.format_expr(&guard.node));
            }
            self.output.push_str(" => ");
            self.output.push_str(&self.format_expr(&arm.body.node));
            self.output.push_str(",\n");
        }
        self.pop_indent();
        self.output.push_str(&self.indent());
        self.output.push_str("}\n");
    }

    /// Format statement inline (without leading indent)
    pub(crate) fn format_stmt_inline(&self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut,
                ..
            } => {
                let mut s = String::with_capacity(64);
                if *is_mut {
                    s.push_str("mut ");
                }
                s.push_str(&name.node);
                if let Some(t) = ty {
                    s.push_str(": ");
                    s.push_str(&self.format_type(&t.node));
                    s.push_str(" = ");
                } else {
                    s.push_str(" := ");
                }
                s.push_str(&self.format_expr(&value.node));
                s
            }
            Stmt::LetDestructure {
                pattern,
                value,
                is_mut,
            } => {
                let mut s = String::with_capacity(64);
                if *is_mut {
                    s.push_str("mut ");
                }
                s.push_str(&self.format_pattern(&pattern.node));
                s.push_str(" := ");
                s.push_str(&self.format_expr(&value.node));
                s
            }
            Stmt::Expr(expr) => self.format_expr(&expr.node),
            Stmt::Return(expr) => {
                let mut s = String::from("R");
                if let Some(e) = expr {
                    s.push(' ');
                    s.push_str(&self.format_expr(&e.node));
                }
                s
            }
            Stmt::Break(expr) => {
                let mut s = String::from("B");
                if let Some(e) = expr {
                    s.push(' ');
                    s.push_str(&self.format_expr(&e.node));
                }
                s
            }
            Stmt::Continue => String::from("C"),
            Stmt::Defer(expr) => {
                format!("D {}", self.format_expr(&expr.node))
            }
            Stmt::Error { message, .. } => {
                format!("# ERROR: {}", message)
            }
        }
    }
}
