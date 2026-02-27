//! Expression code generation: Vais Expr → JavaScript expression string

use crate::{JsCodeGenerator, Result};
use vais_ast::*;

impl JsCodeGenerator {
    /// Generate a JavaScript expression from a Vais Expr
    pub(crate) fn generate_expr(&mut self, expr: &Expr) -> Result<String> {
        match expr {
            // --- Literals ---
            Expr::Int(n) => Ok(n.to_string()),
            Expr::Float(f) => {
                let s = f.to_string();
                // Ensure it looks like a float in JS
                if s.contains('.') || s.contains('e') || s.contains('E') {
                    Ok(s)
                } else {
                    Ok(format!("{s}.0"))
                }
            }
            Expr::Bool(b) => Ok(b.to_string()),
            Expr::String(s) => Ok(format!("\"{}\"", escape_js_string(s))),
            Expr::StringInterp(parts) => self.generate_string_interp(parts),
            Expr::Unit => Ok("undefined".to_string()),

            // --- Identifiers ---
            Expr::Ident(name) => Ok(sanitize_js_ident(name)),
            Expr::SelfCall => {
                if let Some(ref func_name) = self.current_function {
                    Ok(func_name.clone())
                } else {
                    Ok("arguments.callee".to_string())
                }
            }

            // --- Operators ---
            Expr::Binary { op, left, right } => {
                let l = self.generate_expr(&left.node)?;
                let r = self.generate_expr(&right.node)?;
                let op_str = binop_to_js(op);
                Ok(format!("({l} {op_str} {r})"))
            }
            Expr::Unary { op, expr } => {
                let e = self.generate_expr(&expr.node)?;
                let op_str = unaryop_to_js(op);
                Ok(format!("{op_str}{e}"))
            }
            Expr::Ternary { cond, then, else_ } => {
                let c = self.generate_expr(&cond.node)?;
                let t = self.generate_expr(&then.node)?;
                let e = self.generate_expr(&else_.node)?;
                Ok(format!("({c} ? {t} : {e})"))
            }

            // --- Control flow expressions ---
            Expr::If { cond, then, else_ } => self.generate_if_expr(cond, then, else_),
            Expr::Loop {
                pattern,
                iter,
                body,
            } => self.generate_loop_expr(pattern, iter, body),
            Expr::While { condition, body } => self.generate_while_expr(condition, body),
            Expr::Match { expr, arms } => self.generate_match_expr(expr, arms),
            Expr::Block(stmts) => self.generate_block_expr(stmts),

            // --- Function calls ---
            Expr::Call { func, args } => {
                // Struct tuple literal: `Response(200, 1)` → desugar to StructLit
                if let Expr::Ident(name) = &func.node {
                    if let Some(field_defs) = self.structs.get(name.as_str()).cloned() {
                        let field_strs: std::result::Result<Vec<String>, _> = field_defs
                            .iter()
                            .zip(args.iter())
                            .map(|((fname, _), fval)| {
                                let v = self.generate_expr(&fval.node)?;
                                Ok(format!("{}: {v}", sanitize_js_ident(fname)))
                            })
                            .collect();
                        return Ok(format!(
                            "new {}({{{}}})",
                            sanitize_js_ident(name),
                            field_strs?.join(", ")
                        ));
                    }
                }
                let f = self.generate_expr(&func.node)?;
                let args_str = self.generate_args(args)?;
                Ok(format!("{f}({args_str})"))
            }
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                let recv = self.generate_expr(&receiver.node)?;
                let args_str = self.generate_args(args)?;
                Ok(format!(
                    "{recv}.{}({args_str})",
                    sanitize_js_ident(&method.node)
                ))
            }
            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => {
                let args_str = self.generate_args(args)?;
                Ok(format!(
                    "{}.{}({args_str})",
                    sanitize_js_ident(&type_name.node),
                    sanitize_js_ident(&method.node)
                ))
            }

            // --- Data access ---
            Expr::Field { expr, field } => {
                let e = self.generate_expr(&expr.node)?;
                Ok(format!("{e}.{}", sanitize_js_ident(&field.node)))
            }
            Expr::Index { expr, index } => {
                let e = self.generate_expr(&expr.node)?;
                let i = self.generate_expr(&index.node)?;
                Ok(format!("{e}[{i}]"))
            }

            // --- Collection literals ---
            Expr::Array(items) => {
                let parts: std::result::Result<Vec<String>, _> =
                    items.iter().map(|e| self.generate_expr(&e.node)).collect();
                Ok(format!("[{}]", parts?.join(", ")))
            }
            Expr::Tuple(items) => {
                let parts: std::result::Result<Vec<String>, _> =
                    items.iter().map(|e| self.generate_expr(&e.node)).collect();
                Ok(format!("[{}]", parts?.join(", ")))
            }
            Expr::StructLit { name, fields } => {
                let field_strs: std::result::Result<Vec<String>, _> = fields
                    .iter()
                    .map(|(fname, fval)| {
                        let v = self.generate_expr(&fval.node)?;
                        Ok(format!("{}: {v}", sanitize_js_ident(&fname.node)))
                    })
                    .collect();
                Ok(format!(
                    "new {}({{{}}})",
                    sanitize_js_ident(&name.node),
                    field_strs?.join(", ")
                ))
            }
            Expr::MapLit(pairs) => {
                let pair_strs: std::result::Result<Vec<String>, _> = pairs
                    .iter()
                    .map(|(k, v)| {
                        let ks = self.generate_expr(&k.node)?;
                        let vs = self.generate_expr(&v.node)?;
                        Ok(format!("[{ks}, {vs}]"))
                    })
                    .collect();
                Ok(format!("new Map([{}])", pair_strs?.join(", ")))
            }

            // --- Range ---
            Expr::Range {
                start,
                end,
                inclusive,
            } => {
                self.ensure_range_helper();
                let s = match start {
                    Some(e) => self.generate_expr(&e.node)?,
                    None => "0".to_string(),
                };
                let e = match end {
                    Some(e) => self.generate_expr(&e.node)?,
                    None => "Infinity".to_string(),
                };
                if *inclusive {
                    Ok(format!("__range({s}, {e} + 1)"))
                } else {
                    Ok(format!("__range({s}, {e})"))
                }
            }

            // --- Lambda ---
            Expr::Lambda { params, body, .. } => {
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| sanitize_js_ident(&p.name.node))
                    .collect();
                let body_js = self.generate_expr(&body.node)?;
                Ok(format!("({}) => {body_js}", param_strs.join(", ")))
            }

            // --- Async ---
            Expr::Await(inner) => {
                let e = self.generate_expr(&inner.node)?;
                Ok(format!("(await {e})"))
            }
            Expr::Spawn(inner) => {
                let e = self.generate_expr(&inner.node)?;
                Ok(format!("Promise.resolve().then(() => {e})"))
            }

            // --- Error handling ---
            Expr::Try(inner) => {
                // ?  operator: extract value or return early
                let e = self.generate_expr(&inner.node)?;
                Ok(format!("__unwrapOrThrow({e})",))
            }
            Expr::Unwrap(inner) => {
                let e = self.generate_expr(&inner.node)?;
                Ok(format!("__unwrap({e})"))
            }

            // --- References (no-op in JS) ---
            Expr::Ref(inner) | Expr::Deref(inner) => self.generate_expr(&inner.node),

            // --- Spread ---
            Expr::Spread(inner) => {
                let e = self.generate_expr(&inner.node)?;
                Ok(format!("...{e}"))
            }

            // --- Cast (JS has no real casts, use coercion) ---
            Expr::Cast { expr, ty } => {
                let e = self.generate_expr(&expr.node)?;
                match &ty.node {
                    Type::Named { name, .. } => match name.as_str() {
                        "i8" | "i16" | "i32" | "u8" | "u16" | "u32" => Ok(format!("({e} | 0)")),
                        "i64" | "u64" | "i128" | "u128" => Ok(format!("Number({e})")),
                        "f32" | "f64" => Ok(format!("Number({e})")),
                        "bool" => Ok(format!("Boolean({e})")),
                        "str" | "String" => Ok(format!("String({e})")),
                        _ => Ok(e),
                    },
                    _ => Ok(e),
                }
            }

            // --- Assignment ---
            Expr::Assign { target, value } => {
                let t = self.generate_expr(&target.node)?;
                let v = self.generate_expr(&value.node)?;
                Ok(format!("{t} = {v}"))
            }
            Expr::AssignOp { op, target, value } => {
                let t = self.generate_expr(&target.node)?;
                let v = self.generate_expr(&value.node)?;
                let op_str = binop_to_js(op);
                Ok(format!("{t} {op_str}= {v}"))
            }

            // --- Yield ---
            Expr::Yield(inner) => {
                let e = self.generate_expr(&inner.node)?;
                Ok(format!("yield {e}"))
            }

            // --- Lazy / Force ---
            Expr::Lazy(inner) => {
                let e = self.generate_expr(&inner.node)?;
                Ok(format!("(() => {e})"))
            }
            Expr::Force(inner) => {
                let e = self.generate_expr(&inner.node)?;
                Ok(format!("{e}()"))
            }

            // --- Assert ---
            Expr::Assert { condition, message } => {
                let c = self.generate_expr(&condition.node)?;
                match message {
                    Some(msg) => {
                        let m = self.generate_expr(&msg.node)?;
                        Ok(format!("console.assert({c}, {m})"))
                    }
                    None => Ok(format!("console.assert({c})")),
                }
            }

            // --- Comptime / Assume / Old / MacroInvoke / Error ---
            Expr::Comptime { body } => self.generate_expr(&body.node),
            Expr::Assume(_) => Ok("/* assume */".to_string()),
            Expr::Old(_) => Ok("/* old */".to_string()),
            Expr::MacroInvoke(inv) => {
                // Macro invocations are normally expanded during parsing
                // Generate as a function call placeholder
                Ok(format!(
                    "{}(/* macro */)",
                    sanitize_js_ident(&inv.name.node)
                ))
            }
            Expr::Error { message, .. } => Ok(format!("/* codegen error: {} */", message)),
        }
    }

    /// Generate comma-separated argument list
    fn generate_args(&mut self, args: &[Spanned<Expr>]) -> Result<String> {
        let parts: std::result::Result<Vec<String>, _> =
            args.iter().map(|a| self.generate_expr(&a.node)).collect();
        Ok(parts?.join(", "))
    }

    /// Generate string interpolation as template literal
    fn generate_string_interp(&mut self, parts: &[StringInterpPart]) -> Result<String> {
        let mut result = String::from("`");
        for part in parts {
            match part {
                StringInterpPart::Lit(s) => {
                    result.push_str(&escape_template_literal(s));
                }
                StringInterpPart::Expr(e) => {
                    result.push_str("${");
                    result.push_str(&self.generate_expr(&e.node)?);
                    result.push('}');
                }
            }
        }
        result.push('`');
        Ok(result)
    }

    /// Generate if expression as IIFE or ternary
    fn generate_if_expr(
        &mut self,
        cond: &Spanned<Expr>,
        then: &[Spanned<Stmt>],
        else_: &Option<IfElse>,
    ) -> Result<String> {
        let c = self.generate_expr(&cond.node)?;
        let indent = self.indent();

        let mut output = "(() => {\n".to_string();
        self.indent_level += 1;
        let inner_indent = self.indent();

        output.push_str(&format!("{inner_indent}if ({c}) {{\n"));
        self.indent_level += 1;
        let body_indent = self.indent();
        let then_js = self.generate_stmts_as_return(then)?;
        output.push_str(&format!("{body_indent}{then_js}\n"));
        self.indent_level -= 1;
        output.push_str(&format!("{inner_indent}}}"));

        if let Some(else_branch) = else_ {
            self.generate_else_branch(&mut output, else_branch)?;
        }

        output.push('\n');
        self.indent_level -= 1;
        output.push_str(&format!("{indent}}})()",));
        Ok(output)
    }

    fn generate_else_branch(&mut self, output: &mut String, else_: &IfElse) -> Result<()> {
        let inner_indent = self.indent();
        match else_ {
            IfElse::Else(stmts) => {
                output.push_str(" else {\n");
                self.indent_level += 1;
                let body_indent = self.indent();
                let body_js = self.generate_stmts_as_return(stmts)?;
                output.push_str(&format!("{body_indent}{body_js}\n"));
                self.indent_level -= 1;
                output.push_str(&format!("{inner_indent}}}"));
            }
            IfElse::ElseIf(cond, stmts, next_else) => {
                let c = self.generate_expr(&cond.node)?;
                output.push_str(&format!(" else if ({c}) {{\n"));
                self.indent_level += 1;
                let body_indent = self.indent();
                let body_js = self.generate_stmts_as_return(stmts)?;
                output.push_str(&format!("{body_indent}{body_js}\n"));
                self.indent_level -= 1;
                output.push_str(&format!("{inner_indent}}}"));
                if let Some(next) = next_else {
                    self.generate_else_branch(output, next)?;
                }
            }
        }
        Ok(())
    }

    /// Generate loop expression (for..of or infinite loop as IIFE)
    fn generate_loop_expr(
        &mut self,
        pattern: &Option<Spanned<Pattern>>,
        iter: &Option<Box<Spanned<Expr>>>,
        body: &[Spanned<Stmt>],
    ) -> Result<String> {
        let indent = self.indent();
        let mut output = String::new();

        match (pattern, iter) {
            (Some(pat), Some(it)) => {
                // for..of loop: `L x : collection { ... }`
                let pat_js = self.generate_pattern_binding(&pat.node);
                let iter_js = self.generate_expr(&it.node)?;
                output.push_str(&format!("for (const {pat_js} of {iter_js}) {{\n"));
                self.indent_level += 1;
                for stmt in body {
                    let s = self.generate_stmt(&stmt.node)?;
                    if !s.is_empty() {
                        output.push_str(&format!("{}{s}\n", self.indent()));
                    }
                }
                self.indent_level -= 1;
                output.push_str(&format!("{indent}}}"));
            }
            _ => {
                // Infinite loop: `L { ... }`
                output.push_str("while (true) {\n");
                self.indent_level += 1;
                for stmt in body {
                    let s = self.generate_stmt(&stmt.node)?;
                    if !s.is_empty() {
                        output.push_str(&format!("{}{s}\n", self.indent()));
                    }
                }
                self.indent_level -= 1;
                output.push_str(&format!("{indent}}}"));
            }
        }

        Ok(output)
    }

    /// Generate while loop
    fn generate_while_expr(
        &mut self,
        condition: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
    ) -> Result<String> {
        let indent = self.indent();
        let c = self.generate_expr(&condition.node)?;
        let mut output = format!("while ({c}) {{\n");
        self.indent_level += 1;
        for stmt in body {
            let s = self.generate_stmt(&stmt.node)?;
            if !s.is_empty() {
                output.push_str(&format!("{}{s}\n", self.indent()));
            }
        }
        self.indent_level -= 1;
        output.push_str(&format!("{indent}}}"));
        Ok(output)
    }

    /// Generate match expression as switch or if-else chain (as IIFE)
    fn generate_match_expr(&mut self, expr: &Spanned<Expr>, arms: &[MatchArm]) -> Result<String> {
        let indent = self.indent();
        let val = self.generate_expr(&expr.node)?;
        let match_var = self.next_label();

        let mut output = "(() => {\n".to_string();
        self.indent_level += 1;
        let inner = self.indent();
        output.push_str(&format!("{inner}const {match_var} = {val};\n"));

        for (i, arm) in arms.iter().enumerate() {
            let cond = self.generate_pattern_condition(&arm.pattern.node, &match_var)?;
            let bindings = self.generate_pattern_bindings(&arm.pattern.node, &match_var)?;
            let body = self.generate_expr(&arm.body.node)?;

            let keyword = if i == 0 { "if" } else { "else if" };

            if cond == "true" && i == arms.len() - 1 {
                // Wildcard as last arm → else
                output.push_str(&format!("{inner}else {{\n"));
            } else {
                let guard = if let Some(g) = &arm.guard {
                    let g_js = self.generate_expr(&g.node)?;
                    format!(" && ({g_js})")
                } else {
                    String::new()
                };
                output.push_str(&format!("{inner}{keyword} ({cond}{guard}) {{\n"));
            }

            self.indent_level += 1;
            let body_indent = self.indent();
            if !bindings.is_empty() {
                output.push_str(&format!("{body_indent}{bindings}\n"));
            }
            output.push_str(&format!("{body_indent}return {body};\n"));
            self.indent_level -= 1;
            output.push_str(&format!("{inner}}}\n"));
        }

        self.indent_level -= 1;
        output.push_str(&format!("{indent}}})()",));
        Ok(output)
    }

    /// Generate block expression as IIFE
    fn generate_block_expr(&mut self, stmts: &[Spanned<Stmt>]) -> Result<String> {
        if stmts.is_empty() {
            return Ok("undefined".to_string());
        }

        let indent = self.indent();
        let mut output = "(() => {\n".to_string();
        self.indent_level += 1;
        let body = self.generate_stmts_as_return(stmts)?;
        output.push_str(&format!("{}{body}\n", self.indent()));
        self.indent_level -= 1;
        output.push_str(&format!("{indent}}})()",));
        Ok(output)
    }

    /// Generate statements, turning the last expression into a return
    pub(crate) fn generate_stmts_as_return(&mut self, stmts: &[Spanned<Stmt>]) -> Result<String> {
        if stmts.is_empty() {
            return Ok(String::new());
        }

        let mut parts = Vec::new();
        let last_idx = stmts.len() - 1;
        let indent = self.indent();

        for (i, stmt) in stmts.iter().enumerate() {
            if i == last_idx {
                // Last statement: if it's an expression, make it a return
                match &stmt.node {
                    Stmt::Expr(expr) => {
                        let e = self.generate_expr(&expr.node)?;
                        parts.push(format!("return {e};"));
                    }
                    Stmt::Return(_) => {
                        let s = self.generate_stmt(&stmt.node)?;
                        parts.push(s);
                    }
                    _ => {
                        let s = self.generate_stmt(&stmt.node)?;
                        if !s.is_empty() {
                            parts.push(s);
                        }
                    }
                }
            } else {
                let s = self.generate_stmt(&stmt.node)?;
                if !s.is_empty() {
                    parts.push(s);
                }
            }
        }

        Ok(parts.join(&format!("\n{indent}")))
    }

    /// Generate a pattern as a JS binding (for for..of)
    pub(crate) fn generate_pattern_binding(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Ident(name) => sanitize_js_ident(name),
            Pattern::Wildcard => "_".to_string(),
            Pattern::Tuple(pats) => {
                let parts: Vec<String> = pats
                    .iter()
                    .map(|p| self.generate_pattern_binding(&p.node))
                    .collect();
                format!("[{}]", parts.join(", "))
            }
            _ => "_".to_string(),
        }
    }

    /// Generate a condition to check if a value matches a pattern
    pub(crate) fn generate_pattern_condition(
        &mut self,
        pattern: &Pattern,
        val_name: &str,
    ) -> Result<String> {
        match pattern {
            Pattern::Wildcard | Pattern::Ident(_) => Ok("true".to_string()),
            Pattern::Literal(lit) => {
                let lit_js = match lit {
                    Literal::Int(n) => n.to_string(),
                    Literal::Float(f) => f.to_string(),
                    Literal::Bool(b) => b.to_string(),
                    Literal::String(s) => format!("\"{}\"", escape_js_string(s)),
                };
                Ok(format!("{val_name} === {lit_js}"))
            }
            Pattern::Variant { name, .. } => Ok(format!("{val_name}.__tag === \"{}\"", name.node)),
            Pattern::Tuple(pats) => {
                let mut conditions = Vec::new();
                for (i, pat) in pats.iter().enumerate() {
                    let sub =
                        self.generate_pattern_condition(&pat.node, &format!("{val_name}[{i}]"))?;
                    if sub != "true" {
                        conditions.push(sub);
                    }
                }
                if conditions.is_empty() {
                    Ok("true".to_string())
                } else {
                    Ok(conditions.join(" && "))
                }
            }
            Pattern::Struct { name, .. } => Ok(format!(
                "{val_name} instanceof {}",
                sanitize_js_ident(&name.node)
            )),
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let mut conds = Vec::new();
                if let Some(s) = start {
                    let s_cond = self.generate_pattern_condition(&s.node, val_name)?;
                    if s_cond != "true" {
                        conds.push(format!("{val_name} >= {s_cond}"));
                    }
                }
                if let Some(e) = end {
                    let e_cond = self.generate_pattern_condition(&e.node, val_name)?;
                    if e_cond != "true" {
                        if *inclusive {
                            conds.push(format!("{val_name} <= {e_cond}"));
                        } else {
                            conds.push(format!("{val_name} < {e_cond}"));
                        }
                    }
                }
                if conds.is_empty() {
                    Ok("true".to_string())
                } else {
                    Ok(conds.join(" && "))
                }
            }
            Pattern::Or(pats) => {
                let parts: std::result::Result<Vec<String>, _> = pats
                    .iter()
                    .map(|p| self.generate_pattern_condition(&p.node, val_name))
                    .collect();
                Ok(format!("({})", parts?.join(" || ")))
            }
            Pattern::Alias { pattern, .. } => {
                // For pattern alias, check the inner pattern
                self.generate_pattern_condition(&pattern.node, val_name)
            }
        }
    }

    /// Generate variable bindings from a pattern match
    pub(crate) fn generate_pattern_bindings(
        &mut self,
        pattern: &Pattern,
        val_name: &str,
    ) -> Result<String> {
        match pattern {
            Pattern::Ident(name) => Ok(format!("const {} = {val_name};", sanitize_js_ident(name))),
            Pattern::Variant { fields, .. } => {
                let mut bindings = Vec::new();
                for (i, f) in fields.iter().enumerate() {
                    let b = self
                        .generate_pattern_bindings(&f.node, &format!("{val_name}.__data[{i}]"))?;
                    if !b.is_empty() {
                        bindings.push(b);
                    }
                }
                Ok(bindings.join(" "))
            }
            Pattern::Tuple(pats) => {
                let mut bindings = Vec::new();
                for (i, p) in pats.iter().enumerate() {
                    let b = self.generate_pattern_bindings(&p.node, &format!("{val_name}[{i}]"))?;
                    if !b.is_empty() {
                        bindings.push(b);
                    }
                }
                Ok(bindings.join(" "))
            }
            Pattern::Struct { fields, .. } => {
                let mut bindings = Vec::new();
                for (fname, _pat) in fields {
                    bindings.push(format!(
                        "const {} = {val_name}.{};",
                        sanitize_js_ident(&fname.node),
                        sanitize_js_ident(&fname.node)
                    ));
                }
                Ok(bindings.join(" "))
            }
            Pattern::Alias { name, pattern } => {
                // Bind the whole value to the alias name
                let mut bindings = vec![format!("const {} = {val_name};", sanitize_js_ident(name))];
                // Then bind variables from the inner pattern
                let inner = self.generate_pattern_bindings(&pattern.node, val_name)?;
                if !inner.is_empty() {
                    bindings.push(inner);
                }
                Ok(bindings.join(" "))
            }
            _ => Ok(String::new()),
        }
    }

    /// Ensure the __range helper is emitted
    fn ensure_range_helper(&mut self) {
        let helper = "function __range(start, end) {\n  const arr = [];\n  for (let i = start; i < end; i++) arr.push(i);\n  return arr;\n}".to_string();
        if !self.helpers.iter().any(|h| h.contains("__range")) {
            self.helpers.push(helper);
        }
    }
}

/// Convert Vais BinOp to JavaScript operator string
fn binop_to_js(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Lt => "<",
        BinOp::Lte => "<=",
        BinOp::Gt => ">",
        BinOp::Gte => ">=",
        BinOp::Eq => "===",
        BinOp::Neq => "!==",
        BinOp::And => "&&",
        BinOp::Or => "||",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
    }
}

/// Convert Vais UnaryOp to JavaScript operator string
fn unaryop_to_js(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
        UnaryOp::BitNot => "~",
    }
}

/// Escape special characters in a JavaScript string
fn escape_js_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\0' => result.push_str("\\0"),
            _ => result.push(ch),
        }
    }
    result
}

/// Escape special characters in a template literal
fn escape_template_literal(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '`' => result.push_str("\\`"),
            '$' => result.push_str("\\$"),
            '\\' => result.push_str("\\\\"),
            _ => result.push(ch),
        }
    }
    result
}

/// Sanitize a Vais identifier for use in JavaScript
pub(crate) fn sanitize_js_ident(name: &str) -> String {
    // JS reserved words that need renaming
    match name {
        "class" => "_class".to_string(),
        "delete" => "_delete".to_string(),
        "export" => "_export".to_string(),
        "import" => "_import".to_string(),
        "new" => "_new".to_string(),
        "super" => "_super".to_string(),
        "switch" => "_switch".to_string(),
        "this" => "_this".to_string(),
        "throw" => "_throw".to_string(),
        "typeof" => "_typeof".to_string(),
        "var" => "_var".to_string(),
        "void" => "_void".to_string(),
        "with" => "_with".to_string(),
        "yield" => "_yield".to_string(),
        "await" => "_await".to_string(),
        "enum" => "_enum".to_string(),
        "implements" => "_implements".to_string(),
        "interface" => "_interface".to_string(),
        "package" => "_package".to_string(),
        "private" => "_private".to_string(),
        "protected" => "_protected".to_string(),
        "public" => "_public".to_string(),
        "static" => "_static".to_string(),
        "arguments" => "_arguments".to_string(),
        "eval" => "_eval".to_string(),
        _ => name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_js_string() {
        assert_eq!(escape_js_string("hello"), "hello");
        assert_eq!(escape_js_string("he\"llo"), "he\\\"llo");
        assert_eq!(escape_js_string("line\nnew"), "line\\nnew");
    }

    #[test]
    fn test_sanitize_js_ident() {
        assert_eq!(sanitize_js_ident("foo"), "foo");
        assert_eq!(sanitize_js_ident("class"), "_class");
        assert_eq!(sanitize_js_ident("yield"), "_yield");
    }

    #[test]
    fn test_binop_to_js() {
        assert_eq!(binop_to_js(&BinOp::Add), "+");
        assert_eq!(binop_to_js(&BinOp::Eq), "===");
        assert_eq!(binop_to_js(&BinOp::Neq), "!==");
        assert_eq!(binop_to_js(&BinOp::And), "&&");
    }

    #[test]
    fn test_binop_all_variants() {
        assert_eq!(binop_to_js(&BinOp::Sub), "-");
        assert_eq!(binop_to_js(&BinOp::Mul), "*");
        assert_eq!(binop_to_js(&BinOp::Div), "/");
        assert_eq!(binop_to_js(&BinOp::Mod), "%");
        assert_eq!(binop_to_js(&BinOp::Lt), "<");
        assert_eq!(binop_to_js(&BinOp::Lte), "<=");
        assert_eq!(binop_to_js(&BinOp::Gt), ">");
        assert_eq!(binop_to_js(&BinOp::Gte), ">=");
        assert_eq!(binop_to_js(&BinOp::Or), "||");
        assert_eq!(binop_to_js(&BinOp::BitAnd), "&");
        assert_eq!(binop_to_js(&BinOp::BitOr), "|");
        assert_eq!(binop_to_js(&BinOp::BitXor), "^");
        assert_eq!(binop_to_js(&BinOp::Shl), "<<");
        assert_eq!(binop_to_js(&BinOp::Shr), ">>");
    }

    #[test]
    fn test_unaryop_to_js() {
        assert_eq!(unaryop_to_js(&UnaryOp::Neg), "-");
        assert_eq!(unaryop_to_js(&UnaryOp::Not), "!");
        assert_eq!(unaryop_to_js(&UnaryOp::BitNot), "~");
    }

    #[test]
    fn test_escape_js_string_special_chars() {
        assert_eq!(escape_js_string("tab\there"), "tab\\there");
        assert_eq!(escape_js_string("ret\rhere"), "ret\\rhere");
        assert_eq!(escape_js_string("null\0here"), "null\\0here");
        assert_eq!(escape_js_string("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_escape_template_literal() {
        assert_eq!(escape_template_literal("hello"), "hello");
        assert_eq!(escape_template_literal("back`tick"), "back\\`tick");
        assert_eq!(escape_template_literal("dollar$sign"), "dollar\\$sign");
        assert_eq!(escape_template_literal("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_sanitize_all_reserved_words() {
        let reserved = vec![
            ("class", "_class"),
            ("delete", "_delete"),
            ("export", "_export"),
            ("import", "_import"),
            ("new", "_new"),
            ("super", "_super"),
            ("switch", "_switch"),
            ("this", "_this"),
            ("throw", "_throw"),
            ("typeof", "_typeof"),
            ("var", "_var"),
            ("void", "_void"),
            ("with", "_with"),
            ("yield", "_yield"),
            ("await", "_await"),
            ("enum", "_enum"),
            ("implements", "_implements"),
            ("interface", "_interface"),
            ("package", "_package"),
            ("private", "_private"),
            ("protected", "_protected"),
            ("public", "_public"),
            ("static", "_static"),
            ("arguments", "_arguments"),
            ("eval", "_eval"),
        ];
        for (input, expected) in reserved {
            assert_eq!(sanitize_js_ident(input), expected, "Failed for {input}");
        }
    }

    #[test]
    fn test_sanitize_non_reserved() {
        assert_eq!(sanitize_js_ident("my_var"), "my_var");
        assert_eq!(sanitize_js_ident("x"), "x");
        assert_eq!(sanitize_js_ident("counter"), "counter");
    }

    #[test]
    fn test_generate_int_literal() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(gen.generate_expr(&Expr::Int(42)).unwrap(), "42");
        assert_eq!(gen.generate_expr(&Expr::Int(0)).unwrap(), "0");
        assert_eq!(gen.generate_expr(&Expr::Int(-1)).unwrap(), "-1");
    }

    #[test]
    fn test_generate_float_literal() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(gen.generate_expr(&Expr::Float(3.14)).unwrap(), "3.14");
        // Integer-valued float should get .0 appended
        let result = gen.generate_expr(&Expr::Float(1.0)).unwrap();
        assert!(result.contains('.') || result.contains('e'));
    }

    #[test]
    fn test_generate_bool_literal() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(gen.generate_expr(&Expr::Bool(true)).unwrap(), "true");
        assert_eq!(gen.generate_expr(&Expr::Bool(false)).unwrap(), "false");
    }

    #[test]
    fn test_generate_string_literal() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(
            gen.generate_expr(&Expr::String("hello".to_string()))
                .unwrap(),
            "\"hello\""
        );
        assert_eq!(
            gen.generate_expr(&Expr::String("say \"hi\"".to_string()))
                .unwrap(),
            "\"say \\\"hi\\\"\""
        );
    }

    #[test]
    fn test_generate_unit() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(gen.generate_expr(&Expr::Unit).unwrap(), "undefined");
    }

    #[test]
    fn test_generate_ident() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(
            gen.generate_expr(&Expr::Ident("foo".to_string())).unwrap(),
            "foo"
        );
        // Reserved word gets sanitized
        assert_eq!(
            gen.generate_expr(&Expr::Ident("class".to_string()))
                .unwrap(),
            "_class"
        );
    }

    #[test]
    fn test_generate_self_call_with_function() {
        let mut gen = JsCodeGenerator::new();
        gen.current_function = Some("fibonacci".to_string());
        assert_eq!(gen.generate_expr(&Expr::SelfCall).unwrap(), "fibonacci");
    }

    #[test]
    fn test_generate_self_call_without_function() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(
            gen.generate_expr(&Expr::SelfCall).unwrap(),
            "arguments.callee"
        );
    }

    #[test]
    fn test_generate_binary_expr() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Binary {
            op: BinOp::Add,
            left: Box::new(Spanned::new(Expr::Int(1), Span::new(0, 1))),
            right: Box::new(Spanned::new(Expr::Int(2), Span::new(3, 4))),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "(1 + 2)");
    }

    #[test]
    fn test_generate_unary_expr() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Spanned::new(Expr::Int(5), Span::new(1, 2))),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "-5");

        let expr_not = Expr::Unary {
            op: UnaryOp::Not,
            expr: Box::new(Spanned::new(Expr::Bool(true), Span::new(1, 5))),
        };
        assert_eq!(gen.generate_expr(&expr_not).unwrap(), "!true");
    }

    #[test]
    fn test_generate_ternary_expr() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Ternary {
            cond: Box::new(Spanned::new(Expr::Bool(true), Span::new(0, 4))),
            then: Box::new(Spanned::new(Expr::Int(1), Span::new(7, 8))),
            else_: Box::new(Spanned::new(Expr::Int(0), Span::new(11, 12))),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "(true ? 1 : 0)");
    }

    #[test]
    fn test_generate_array_literal() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Array(vec![
            Spanned::new(Expr::Int(1), Span::new(1, 2)),
            Spanned::new(Expr::Int(2), Span::new(4, 5)),
            Spanned::new(Expr::Int(3), Span::new(7, 8)),
        ]);
        assert_eq!(gen.generate_expr(&expr).unwrap(), "[1, 2, 3]");
    }

    #[test]
    fn test_generate_empty_array() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Array(vec![]);
        assert_eq!(gen.generate_expr(&expr).unwrap(), "[]");
    }

    #[test]
    fn test_generate_tuple_as_array() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Tuple(vec![
            Spanned::new(Expr::Int(1), Span::new(1, 2)),
            Spanned::new(Expr::String("hello".to_string()), Span::new(4, 11)),
        ]);
        assert_eq!(gen.generate_expr(&expr).unwrap(), "[1, \"hello\"]");
    }

    #[test]
    fn test_generate_function_call() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Call {
            func: Box::new(Spanned::new(
                Expr::Ident("print".to_string()),
                Span::new(0, 5),
            )),
            args: vec![Spanned::new(Expr::Int(42), Span::new(6, 8))],
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "print(42)");
    }

    #[test]
    fn test_generate_method_call() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::MethodCall {
            receiver: Box::new(Spanned::new(
                Expr::Ident("vec".to_string()),
                Span::new(0, 3),
            )),
            method: Spanned::new("push".to_string(), Span::new(4, 8)),
            args: vec![Spanned::new(Expr::Int(5), Span::new(9, 10))],
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "vec.push(5)");
    }

    #[test]
    fn test_generate_static_method_call() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::StaticMethodCall {
            type_name: Spanned::new("Vec".to_string(), Span::new(0, 3)),
            method: Spanned::new("new".to_string(), Span::new(5, 8)),
            args: vec![],
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "Vec._new()");
    }

    #[test]
    fn test_generate_field_access() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Field {
            expr: Box::new(Spanned::new(
                Expr::Ident("point".to_string()),
                Span::new(0, 5),
            )),
            field: Spanned::new("x".to_string(), Span::new(6, 7)),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "point.x");
    }

    #[test]
    fn test_generate_index() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Index {
            expr: Box::new(Spanned::new(
                Expr::Ident("arr".to_string()),
                Span::new(0, 3),
            )),
            index: Box::new(Spanned::new(Expr::Int(0), Span::new(4, 5))),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "arr[0]");
    }

    #[test]
    fn test_generate_map_literal() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::MapLit(vec![(
            Spanned::new(Expr::String("key".to_string()), Span::new(0, 5)),
            Spanned::new(Expr::Int(1), Span::new(8, 9)),
        )]);
        assert_eq!(gen.generate_expr(&expr).unwrap(), "new Map([[\"key\", 1]])");
    }

    #[test]
    fn test_generate_struct_literal() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::StructLit {
            name: Spanned::new("Point".to_string(), Span::new(0, 5)),
            fields: vec![
                (
                    Spanned::new("x".to_string(), Span::new(6, 7)),
                    Spanned::new(Expr::Int(1), Span::new(9, 10)),
                ),
                (
                    Spanned::new("y".to_string(), Span::new(12, 13)),
                    Spanned::new(Expr::Int(2), Span::new(15, 16)),
                ),
            ],
        };
        let result = gen.generate_expr(&expr).unwrap();
        assert!(result.contains("new Point"));
        assert!(result.contains("x: 1"));
        assert!(result.contains("y: 2"));
    }

    #[test]
    fn test_generate_lambda() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Lambda {
            params: vec![Param {
                name: Spanned::new("x".to_string(), Span::new(1, 2)),
                ty: Spanned::new(Type::Infer, Span::new(0, 0)),
                is_mut: false,
                is_vararg: false,
                ownership: Ownership::Regular,
                default_value: None,
            }],
            body: Box::new(Spanned::new(
                Expr::Binary {
                    op: BinOp::Mul,
                    left: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(5, 6))),
                    right: Box::new(Spanned::new(Expr::Int(2), Span::new(9, 10))),
                },
                Span::new(5, 10),
            )),
            captures: vec![],
            capture_mode: CaptureMode::ByValue,
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "(x) => (x * 2)");
    }

    #[test]
    fn test_generate_await() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Await(Box::new(Spanned::new(
            Expr::Call {
                func: Box::new(Spanned::new(
                    Expr::Ident("fetch".to_string()),
                    Span::new(0, 5),
                )),
                args: vec![],
            },
            Span::new(0, 7),
        )));
        assert_eq!(gen.generate_expr(&expr).unwrap(), "(await fetch())");
    }

    #[test]
    fn test_generate_spawn() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Spawn(Box::new(Spanned::new(
            Expr::Ident("task".to_string()),
            Span::new(0, 4),
        )));
        assert_eq!(
            gen.generate_expr(&expr).unwrap(),
            "Promise.resolve().then(() => task)"
        );
    }

    #[test]
    fn test_generate_try_operator() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Try(Box::new(Spanned::new(
            Expr::Ident("result".to_string()),
            Span::new(0, 6),
        )));
        assert_eq!(gen.generate_expr(&expr).unwrap(), "__unwrapOrThrow(result)");
    }

    #[test]
    fn test_generate_unwrap() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Unwrap(Box::new(Spanned::new(
            Expr::Ident("opt".to_string()),
            Span::new(0, 3),
        )));
        assert_eq!(gen.generate_expr(&expr).unwrap(), "__unwrap(opt)");
    }

    #[test]
    fn test_generate_ref_deref_noop() {
        let mut gen = JsCodeGenerator::new();
        let inner = Expr::Ident("x".to_string());
        let ref_expr = Expr::Ref(Box::new(Spanned::new(inner.clone(), Span::new(0, 1))));
        assert_eq!(gen.generate_expr(&ref_expr).unwrap(), "x");

        let deref_expr = Expr::Deref(Box::new(Spanned::new(
            Expr::Ident("x".to_string()),
            Span::new(0, 1),
        )));
        assert_eq!(gen.generate_expr(&deref_expr).unwrap(), "x");
    }

    #[test]
    fn test_generate_spread() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Spread(Box::new(Spanned::new(
            Expr::Ident("args".to_string()),
            Span::new(0, 4),
        )));
        assert_eq!(gen.generate_expr(&expr).unwrap(), "...args");
    }

    #[test]
    fn test_generate_cast_i32() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Cast {
            expr: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
            ty: Spanned::new(
                Type::Named {
                    name: "i32".to_string(),
                    generics: vec![],
                },
                Span::new(5, 8),
            ),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "(x | 0)");
    }

    #[test]
    fn test_generate_cast_f64() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Cast {
            expr: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
            ty: Spanned::new(
                Type::Named {
                    name: "f64".to_string(),
                    generics: vec![],
                },
                Span::new(5, 8),
            ),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "Number(x)");
    }

    #[test]
    fn test_generate_cast_bool() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Cast {
            expr: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
            ty: Spanned::new(
                Type::Named {
                    name: "bool".to_string(),
                    generics: vec![],
                },
                Span::new(5, 9),
            ),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "Boolean(x)");
    }

    #[test]
    fn test_generate_cast_string() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Cast {
            expr: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
            ty: Spanned::new(
                Type::Named {
                    name: "String".to_string(),
                    generics: vec![],
                },
                Span::new(5, 11),
            ),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "String(x)");
    }

    #[test]
    fn test_generate_cast_unknown_type() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Cast {
            expr: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
            ty: Spanned::new(
                Type::Named {
                    name: "MyType".to_string(),
                    generics: vec![],
                },
                Span::new(5, 11),
            ),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "x");
    }

    #[test]
    fn test_generate_assign() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Assign {
            target: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
            value: Box::new(Spanned::new(Expr::Int(10), Span::new(4, 6))),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "x = 10");
    }

    #[test]
    fn test_generate_assign_op() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::AssignOp {
            op: BinOp::Add,
            target: Box::new(Spanned::new(Expr::Ident("x".to_string()), Span::new(0, 1))),
            value: Box::new(Spanned::new(Expr::Int(1), Span::new(5, 6))),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "x += 1");
    }

    #[test]
    fn test_generate_yield() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Yield(Box::new(Spanned::new(Expr::Int(42), Span::new(6, 8))));
        assert_eq!(gen.generate_expr(&expr).unwrap(), "yield 42");
    }

    #[test]
    fn test_generate_lazy() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Lazy(Box::new(Spanned::new(Expr::Int(42), Span::new(0, 2))));
        assert_eq!(gen.generate_expr(&expr).unwrap(), "(() => 42)");
    }

    #[test]
    fn test_generate_force() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Force(Box::new(Spanned::new(
            Expr::Ident("lazy_val".to_string()),
            Span::new(0, 8),
        )));
        assert_eq!(gen.generate_expr(&expr).unwrap(), "lazy_val()");
    }

    #[test]
    fn test_generate_assert_without_message() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Assert {
            condition: Box::new(Spanned::new(Expr::Bool(true), Span::new(0, 4))),
            message: None,
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "console.assert(true)");
    }

    #[test]
    fn test_generate_assert_with_message() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Assert {
            condition: Box::new(Spanned::new(Expr::Bool(false), Span::new(0, 5))),
            message: Some(Box::new(Spanned::new(
                Expr::String("failed".to_string()),
                Span::new(7, 15),
            ))),
        };
        assert_eq!(
            gen.generate_expr(&expr).unwrap(),
            "console.assert(false, \"failed\")"
        );
    }

    #[test]
    fn test_generate_comptime() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Comptime {
            body: Box::new(Spanned::new(Expr::Int(42), Span::new(0, 2))),
        };
        assert_eq!(gen.generate_expr(&expr).unwrap(), "42");
    }

    #[test]
    fn test_generate_assume() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Assume(Box::new(Spanned::new(Expr::Bool(true), Span::new(0, 4))));
        assert_eq!(gen.generate_expr(&expr).unwrap(), "/* assume */");
    }

    #[test]
    fn test_generate_old() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Old(Box::new(Spanned::new(
            Expr::Ident("x".to_string()),
            Span::new(0, 1),
        )));
        assert_eq!(gen.generate_expr(&expr).unwrap(), "/* old */");
    }

    #[test]
    fn test_generate_error_expr() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Error {
            message: "something went wrong".to_string(),
            skipped_tokens: vec![],
        };
        let result = gen.generate_expr(&expr).unwrap();
        assert!(result.contains("codegen error"));
        assert!(result.contains("something went wrong"));
    }

    #[test]
    fn test_generate_string_interp() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::StringInterp(vec![
            StringInterpPart::Lit("Hello, ".to_string()),
            StringInterpPart::Expr(Box::new(Spanned::new(
                Expr::Ident("name".to_string()),
                Span::new(0, 4),
            ))),
            StringInterpPart::Lit("!".to_string()),
        ]);
        assert_eq!(gen.generate_expr(&expr).unwrap(), "`Hello, ${name}!`");
    }

    #[test]
    fn test_generate_block_expr_empty() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Block(vec![]);
        assert_eq!(gen.generate_expr(&expr).unwrap(), "undefined");
    }

    #[test]
    fn test_generate_block_expr_single() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Block(vec![Spanned::new(
            Stmt::Expr(Box::new(Spanned::new(Expr::Int(42), Span::new(0, 2)))),
            Span::new(0, 3),
        )]);
        let result = gen.generate_expr(&expr).unwrap();
        assert!(result.contains("return 42;"));
    }

    #[test]
    fn test_generate_while_loop() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::While {
            condition: Box::new(Spanned::new(Expr::Bool(true), Span::new(0, 4))),
            body: vec![Spanned::new(Stmt::Break(None), Span::new(5, 11))],
        };
        let result = gen.generate_expr(&expr).unwrap();
        assert!(result.contains("while (true)"));
        assert!(result.contains("break;"));
    }

    #[test]
    fn test_generate_infinite_loop() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Loop {
            pattern: None,
            iter: None,
            body: vec![Spanned::new(Stmt::Continue, Span::new(0, 8))],
        };
        let result = gen.generate_expr(&expr).unwrap();
        assert!(result.contains("while (true)"));
        assert!(result.contains("continue;"));
    }

    #[test]
    fn test_generate_for_of_loop() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Loop {
            pattern: Some(Spanned::new(
                Pattern::Ident("x".to_string()),
                Span::new(0, 1),
            )),
            iter: Some(Box::new(Spanned::new(
                Expr::Ident("items".to_string()),
                Span::new(3, 8),
            ))),
            body: vec![],
        };
        let result = gen.generate_expr(&expr).unwrap();
        assert!(result.contains("for (const x of items)"));
    }

    #[test]
    fn test_generate_range_exclusive() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Range {
            start: Some(Box::new(Spanned::new(Expr::Int(0), Span::new(0, 1)))),
            end: Some(Box::new(Spanned::new(Expr::Int(10), Span::new(3, 5)))),
            inclusive: false,
        };
        let result = gen.generate_expr(&expr).unwrap();
        assert_eq!(result, "__range(0, 10)");
        // Verify helper was registered
        assert!(!gen.helpers.is_empty());
    }

    #[test]
    fn test_generate_range_inclusive() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Range {
            start: Some(Box::new(Spanned::new(Expr::Int(1), Span::new(0, 1)))),
            end: Some(Box::new(Spanned::new(Expr::Int(5), Span::new(3, 4)))),
            inclusive: true,
        };
        let result = gen.generate_expr(&expr).unwrap();
        assert_eq!(result, "__range(1, 5 + 1)");
    }

    #[test]
    fn test_generate_range_no_start() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Range {
            start: None,
            end: Some(Box::new(Spanned::new(Expr::Int(10), Span::new(0, 2)))),
            inclusive: false,
        };
        let result = gen.generate_expr(&expr).unwrap();
        assert_eq!(result, "__range(0, 10)");
    }

    #[test]
    fn test_generate_range_no_end() {
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::Range {
            start: Some(Box::new(Spanned::new(Expr::Int(5), Span::new(0, 1)))),
            end: None,
            inclusive: false,
        };
        let result = gen.generate_expr(&expr).unwrap();
        assert_eq!(result, "__range(5, Infinity)");
    }

    #[test]
    fn test_pattern_binding_ident() {
        let gen = JsCodeGenerator::new();
        assert_eq!(
            gen.generate_pattern_binding(&Pattern::Ident("x".to_string())),
            "x"
        );
    }

    #[test]
    fn test_pattern_binding_wildcard() {
        let gen = JsCodeGenerator::new();
        assert_eq!(gen.generate_pattern_binding(&Pattern::Wildcard), "_");
    }

    #[test]
    fn test_pattern_binding_tuple() {
        let gen = JsCodeGenerator::new();
        let pat = Pattern::Tuple(vec![
            Spanned::new(Pattern::Ident("a".to_string()), Span::new(0, 1)),
            Spanned::new(Pattern::Ident("b".to_string()), Span::new(3, 4)),
        ]);
        assert_eq!(gen.generate_pattern_binding(&pat), "[a, b]");
    }

    #[test]
    fn test_pattern_condition_wildcard() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(
            gen.generate_pattern_condition(&Pattern::Wildcard, "x")
                .unwrap(),
            "true"
        );
    }

    #[test]
    fn test_pattern_condition_literal() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(
            gen.generate_pattern_condition(&Pattern::Literal(Literal::Int(42)), "x")
                .unwrap(),
            "x === 42"
        );
        assert_eq!(
            gen.generate_pattern_condition(
                &Pattern::Literal(Literal::String("hello".to_string())),
                "x"
            )
            .unwrap(),
            "x === \"hello\""
        );
        assert_eq!(
            gen.generate_pattern_condition(&Pattern::Literal(Literal::Bool(true)), "x")
                .unwrap(),
            "x === true"
        );
    }

    #[test]
    fn test_pattern_condition_variant() {
        let mut gen = JsCodeGenerator::new();
        let pat = Pattern::Variant {
            name: Spanned::new("Ok".to_string(), Span::new(0, 2)),
            fields: vec![],
        };
        assert_eq!(
            gen.generate_pattern_condition(&pat, "val").unwrap(),
            "val.__tag === \"Ok\""
        );
    }

    #[test]
    fn test_pattern_bindings_ident() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(
            gen.generate_pattern_bindings(&Pattern::Ident("x".to_string()), "val")
                .unwrap(),
            "const x = val;"
        );
    }

    #[test]
    fn test_pattern_bindings_wildcard() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(
            gen.generate_pattern_bindings(&Pattern::Wildcard, "val")
                .unwrap(),
            ""
        );
    }

    #[test]
    fn test_macro_invoke() {
        use vais_ast::macros::{Delimiter, MacroInvoke};
        let mut gen = JsCodeGenerator::new();
        let expr = Expr::MacroInvoke(MacroInvoke {
            name: Spanned::new("println".to_string(), Span::new(0, 7)),
            delimiter: Delimiter::Paren,
            tokens: vec![],
        });
        let result = gen.generate_expr(&expr).unwrap();
        assert!(result.contains("println"));
        assert!(result.contains("macro"));
    }

    #[test]
    fn test_generate_stmts_as_return_empty() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(gen.generate_stmts_as_return(&[]).unwrap(), "");
    }
}
