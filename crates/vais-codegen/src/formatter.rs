//! VAIS Code Formatter
//!
//! Pretty-prints VAIS AST to formatted source code.

use vais_ast::*;

/// Formatter configuration
#[derive(Clone)]
pub struct FormatConfig {
    /// Indentation size in spaces
    pub indent_size: usize,
    /// Maximum line length
    pub max_line_length: usize,
    /// Use tabs instead of spaces
    pub use_tabs: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_size: 4,
            max_line_length: 100,
            use_tabs: false,
        }
    }
}

/// Code formatter that converts AST to formatted source code
pub struct Formatter {
    config: FormatConfig,
    output: String,
    indent_level: usize,
}

impl Formatter {
    pub fn new(config: FormatConfig) -> Self {
        Self {
            config,
            output: String::new(),
            indent_level: 0,
        }
    }

    /// Format a module
    pub fn format_module(&mut self, module: &Module) -> String {
        self.output.clear();
        self.indent_level = 0;

        let mut first = true;
        for item in &module.items {
            if !first {
                self.output.push('\n');
            }
            first = false;
            self.format_item(&item.node);
        }

        self.output.clone()
    }

    /// Get indentation string
    fn indent(&self) -> String {
        if self.config.use_tabs {
            "\t".repeat(self.indent_level)
        } else {
            " ".repeat(self.indent_level * self.config.indent_size)
        }
    }

    /// Push indentation
    fn push_indent(&mut self) {
        self.indent_level += 1;
    }

    /// Pop indentation
    fn pop_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Format a top-level item
    fn format_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.format_function(f),
            Item::Struct(s) => self.format_struct(s),
            Item::Enum(e) => self.format_enum(e),
            Item::TypeAlias(t) => self.format_type_alias(t),
            Item::Use(u) => self.format_use(u),
            Item::Trait(t) => self.format_trait(t),
            Item::Impl(i) => self.format_impl(i),
        }
    }

    /// Format a function
    fn format_function(&mut self, f: &Function) {
        let indent = self.indent();

        // Attributes
        for attr in &f.attributes {
            self.output.push_str(&indent);
            self.output.push_str("#[");
            self.output.push_str(&attr.name);
            if !attr.args.is_empty() {
                self.output.push('(');
                self.output.push_str(&attr.args.join(", "));
                self.output.push(')');
            }
            self.output.push_str("]\n");
        }

        self.output.push_str(&indent);

        // pub
        if f.is_pub {
            self.output.push_str("pub ");
        }

        // async
        if f.is_async {
            self.output.push_str("async ");
        }

        // F name
        self.output.push_str("F ");
        self.output.push_str(&f.name.node);

        // Generics
        if !f.generics.is_empty() {
            self.output.push('<');
            let mut first = true;
            for g in &f.generics {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                if g.bounds.is_empty() {
                    self.output.push_str(&g.name.node);
                } else {
                    self.output.push_str(&g.name.node);
                    self.output.push_str(": ");
                    let bounds: Vec<&str> = g.bounds.iter().map(|b| b.node.as_str()).collect();
                    self.output.push_str(&bounds.join(" + "));
                }
            }
            self.output.push('>');
        }

        // Parameters
        self.output.push('(');
        let params: Vec<String> = f.params.iter().map(|p| {
            let mut s = String::new();
            if p.is_mut {
                s.push_str("mut ");
            }
            s.push_str(&p.name.node);
            s.push_str(": ");
            s.push_str(&self.format_type(&p.ty.node));
            s
        }).collect();
        self.output.push_str(&params.join(", "));
        self.output.push(')');

        // Return type
        if let Some(ret) = &f.ret_type {
            self.output.push_str(" -> ");
            self.output.push_str(&self.format_type(&ret.node));
        }

        // Body
        match &f.body {
            FunctionBody::Expr(expr) => {
                self.output.push_str(" = ");
                self.output.push_str(&self.format_expr(&expr.node));
                self.output.push('\n');
            }
            FunctionBody::Block(stmts) => {
                self.output.push_str(" {\n");
                self.push_indent();
                for stmt in stmts {
                    self.format_stmt(&stmt.node);
                }
                self.pop_indent();
                self.output.push_str(&self.indent());
                self.output.push_str("}\n");
            }
        }
    }

    /// Format a struct
    fn format_struct(&mut self, s: &Struct) {
        let indent = self.indent();

        self.output.push_str(&indent);
        if s.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("S ");
        self.output.push_str(&s.name.node);

        // Generics
        if !s.generics.is_empty() {
            self.output.push('<');
            let mut first = true;
            for g in &s.generics {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                self.output.push_str(&g.name.node);
            }
            self.output.push('>');
        }

        self.output.push_str(" {\n");
        self.push_indent();

        // Fields
        for field in &s.fields {
            self.output.push_str(&self.indent());
            if field.is_pub {
                self.output.push_str("pub ");
            }
            self.output.push_str(&field.name.node);
            self.output.push_str(": ");
            self.output.push_str(&self.format_type(&field.ty.node));
            self.output.push_str(",\n");
        }

        // Methods
        if !s.methods.is_empty() && !s.fields.is_empty() {
            self.output.push('\n');
        }
        for method in &s.methods {
            self.format_function(&method.node);
        }

        self.pop_indent();
        self.output.push_str(&self.indent());
        self.output.push_str("}\n");
    }

    /// Format an enum
    fn format_enum(&mut self, e: &Enum) {
        let indent = self.indent();

        self.output.push_str(&indent);
        if e.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("E ");
        self.output.push_str(&e.name.node);

        // Generics
        if !e.generics.is_empty() {
            self.output.push('<');
            let mut first = true;
            for g in &e.generics {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                self.output.push_str(&g.name.node);
            }
            self.output.push('>');
        }

        self.output.push_str(" {\n");
        self.push_indent();

        for variant in &e.variants {
            self.output.push_str(&self.indent());
            self.output.push_str(&variant.name.node);

            match &variant.fields {
                VariantFields::Unit => {}
                VariantFields::Tuple(types) => {
                    self.output.push('(');
                    let types: Vec<String> = types.iter().map(|t| self.format_type(&t.node)).collect();
                    self.output.push_str(&types.join(", "));
                    self.output.push(')');
                }
                VariantFields::Struct(fields) => {
                    self.output.push_str(" {\n");
                    self.push_indent();
                    for field in fields {
                        self.output.push_str(&self.indent());
                        self.output.push_str(&field.name.node);
                        self.output.push_str(": ");
                        self.output.push_str(&self.format_type(&field.ty.node));
                        self.output.push_str(",\n");
                    }
                    self.pop_indent();
                    self.output.push_str(&self.indent());
                    self.output.push('}');
                }
            }
            self.output.push_str(",\n");
        }

        self.pop_indent();
        self.output.push_str(&self.indent());
        self.output.push_str("}\n");
    }

    /// Format a type alias
    fn format_type_alias(&mut self, t: &TypeAlias) {
        let indent = self.indent();

        self.output.push_str(&indent);
        if t.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("T ");
        self.output.push_str(&t.name.node);

        if !t.generics.is_empty() {
            self.output.push('<');
            let mut first = true;
            for g in &t.generics {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                self.output.push_str(&g.name.node);
            }
            self.output.push('>');
        }

        self.output.push_str(" = ");
        self.output.push_str(&self.format_type(&t.ty.node));
        self.output.push('\n');
    }

    /// Format a use statement
    fn format_use(&mut self, u: &Use) {
        let indent = self.indent();

        self.output.push_str(&indent);
        self.output.push_str("U ");
        let path: Vec<&str> = u.path.iter().map(|p| p.node.as_str()).collect();
        self.output.push_str(&path.join("::"));

        if let Some(alias) = &u.alias {
            self.output.push_str(" as ");
            self.output.push_str(&alias.node);
        }

        self.output.push('\n');
    }

    /// Format a trait
    fn format_trait(&mut self, t: &Trait) {
        let indent = self.indent();

        self.output.push_str(&indent);
        if t.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("W ");
        self.output.push_str(&t.name.node);

        if !t.generics.is_empty() {
            self.output.push('<');
            let mut first = true;
            for g in &t.generics {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                self.output.push_str(&g.name.node);
            }
            self.output.push('>');
        }

        if !t.super_traits.is_empty() {
            self.output.push_str(": ");
            let traits: Vec<&str> = t.super_traits.iter().map(|s| s.node.as_str()).collect();
            self.output.push_str(&traits.join(" + "));
        }

        self.output.push_str(" {\n");
        self.push_indent();

        // Associated types
        for at in &t.associated_types {
            self.output.push_str(&self.indent());
            self.output.push_str("T ");
            self.output.push_str(&at.name.node);
            if !at.bounds.is_empty() {
                self.output.push_str(": ");
                let bounds: Vec<&str> = at.bounds.iter().map(|b| b.node.as_str()).collect();
                self.output.push_str(&bounds.join(" + "));
            }
            if let Some(default) = &at.default {
                self.output.push_str(" = ");
                self.output.push_str(&self.format_type(&default.node));
            }
            self.output.push('\n');
        }

        // Methods
        for method in &t.methods {
            self.output.push_str(&self.indent());
            self.output.push_str("F ");
            self.output.push_str(&method.name.node);
            self.output.push('(');
            let params: Vec<String> = method.params.iter().map(|p| {
                format!("{}: {}", p.name.node, self.format_type(&p.ty.node))
            }).collect();
            self.output.push_str(&params.join(", "));
            self.output.push(')');

            if let Some(ret) = &method.ret_type {
                self.output.push_str(" -> ");
                self.output.push_str(&self.format_type(&ret.node));
            }

            if let Some(body) = &method.default_body {
                match body {
                    FunctionBody::Expr(expr) => {
                        self.output.push_str(" = ");
                        self.output.push_str(&self.format_expr(&expr.node));
                        self.output.push('\n');
                    }
                    FunctionBody::Block(stmts) => {
                        self.output.push_str(" {\n");
                        self.push_indent();
                        for stmt in stmts {
                            self.format_stmt(&stmt.node);
                        }
                        self.pop_indent();
                        self.output.push_str(&self.indent());
                        self.output.push_str("}\n");
                    }
                }
            } else {
                self.output.push('\n');
            }
        }

        self.pop_indent();
        self.output.push_str(&self.indent());
        self.output.push_str("}\n");
    }

    /// Format an impl block
    fn format_impl(&mut self, i: &Impl) {
        let indent = self.indent();

        self.output.push_str(&indent);
        self.output.push_str("X ");
        self.output.push_str(&self.format_type(&i.target_type.node));

        if let Some(trait_name) = &i.trait_name {
            self.output.push_str(": ");
            self.output.push_str(&trait_name.node);
        }

        self.output.push_str(" {\n");
        self.push_indent();

        for method in &i.methods {
            self.format_function(&method.node);
        }

        self.pop_indent();
        self.output.push_str(&self.indent());
        self.output.push_str("}\n");
    }

    /// Format a type
    fn format_type(&self, ty: &Type) -> String {
        match ty {
            Type::Named { name, generics } => {
                if generics.is_empty() {
                    name.to_string()
                } else {
                    let gens: Vec<String> = generics.iter().map(|g| self.format_type(&g.node)).collect();
                    format!("{}<{}>", name, gens.join(", "))
                }
            }
            Type::Array(inner) => format!("[{}]", self.format_type(&inner.node)),
            Type::Map(key, value) => {
                format!("[{}:{}]", self.format_type(&key.node), self.format_type(&value.node))
            }
            Type::Tuple(types) => {
                let ts: Vec<String> = types.iter().map(|t| self.format_type(&t.node)).collect();
                format!("({})", ts.join(", "))
            }
            Type::Optional(inner) => format!("{}?", self.format_type(&inner.node)),
            Type::Result(inner) => format!("{}!", self.format_type(&inner.node)),
            Type::Pointer(inner) => format!("*{}", self.format_type(&inner.node)),
            Type::Ref(inner) => format!("&{}", self.format_type(&inner.node)),
            Type::RefMut(inner) => format!("&mut {}", self.format_type(&inner.node)),
            Type::Fn { params, ret } => {
                let ps: Vec<String> = params.iter().map(|p| self.format_type(&p.node)).collect();
                format!("({}) -> {}", ps.join(", "), self.format_type(&ret.node))
            }
            Type::Unit => "()".to_string(),
            Type::Infer => "_".to_string(),
        }
    }

    /// Format a statement
    fn format_stmt(&mut self, stmt: &Stmt) {
        let indent = self.indent();

        match stmt {
            Stmt::Let { name, ty, value, is_mut } => {
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
            Stmt::Expr(expr) => {
                // Handle if/loop/match/block specially for proper indentation
                match &expr.node {
                    Expr::If { cond, then, else_ } => {
                        self.format_if_expr(&indent, cond, then, else_.as_ref());
                    }
                    Expr::Loop { pattern, iter, body } => {
                        self.format_loop_expr(&indent, pattern, iter.as_ref().map(|e| &**e), body);
                    }
                    Expr::Match { expr: match_expr, arms } => {
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
                self.output.push_str("R");
                if let Some(e) = expr {
                    self.output.push(' ');
                    self.output.push_str(&self.format_expr(&e.node));
                }
                self.output.push('\n');
            }
            Stmt::Break(expr) => {
                self.output.push_str(&indent);
                self.output.push_str("B");
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
        }
    }

    /// Format an if expression with proper indentation
    fn format_if_expr(&mut self, indent: &str, cond: &Spanned<Expr>, then: &[Spanned<Stmt>], else_: Option<&IfElse>) {
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
    fn format_if_else_branch(&mut self, if_else: &IfElse) {
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
    fn format_loop_expr(&mut self, indent: &str, pattern: &Option<Spanned<Pattern>>, iter: Option<&Spanned<Expr>>, body: &[Spanned<Stmt>]) {
        self.output.push_str(indent);
        self.output.push_str("L");
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

    /// Format a match expression with proper indentation
    fn format_match_expr(&mut self, indent: &str, expr: &Spanned<Expr>, arms: &[MatchArm]) {
        self.output.push_str(indent);
        self.output.push_str("M ");
        self.output.push_str(&self.format_expr(&expr.node));
        self.output.push_str(" {\n");
        self.push_indent();
        for arm in arms {
            self.output.push_str(&self.indent());
            self.output.push_str(&self.format_pattern(&arm.pattern.node));
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

    /// Format an expression
    fn format_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(n) => n.to_string(),
            Expr::Float(n) => {
                let s = n.to_string();
                if s.contains('.') { s } else { format!("{}.0", s) }
            }
            Expr::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            Expr::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
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
                format!("{} {} {}", self.format_expr(&left.node), op_str, self.format_expr(&right.node))
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

            Expr::Loop { pattern, iter, body } => {
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
                let args_str: Vec<String> = args.iter().map(|a| self.format_expr(&a.node)).collect();
                format!("{}({})", self.format_expr(&func.node), args_str.join(", "))
            }

            Expr::MethodCall { receiver, method, args } => {
                let args_str: Vec<String> = args.iter().map(|a| self.format_expr(&a.node)).collect();
                format!("{}.{}({})", self.format_expr(&receiver.node), method.node, args_str.join(", "))
            }

            Expr::StaticMethodCall { type_name, method, args } => {
                let args_str: Vec<String> = args.iter().map(|a| self.format_expr(&a.node)).collect();
                format!("{}.{}({})", type_name.node, method.node, args_str.join(", "))
            }

            Expr::Field { expr, field } => {
                format!("{}.{}", self.format_expr(&expr.node), field.node)
            }

            Expr::Index { expr, index } => {
                format!("{}[{}]", self.format_expr(&expr.node), self.format_expr(&index.node))
            }

            Expr::Array(items) => {
                let items_str: Vec<String> = items.iter().map(|i| self.format_expr(&i.node)).collect();
                format!("[{}]", items_str.join(", "))
            }

            Expr::Tuple(items) => {
                let items_str: Vec<String> = items.iter().map(|i| self.format_expr(&i.node)).collect();
                format!("({})", items_str.join(", "))
            }

            Expr::StructLit { name, fields } => {
                let fields_str: Vec<String> = fields.iter().map(|(n, v)| {
                    format!("{}: {}", n.node, self.format_expr(&v.node))
                }).collect();
                format!("{} {{ {} }}", name.node, fields_str.join(", "))
            }

            Expr::Range { start, end, inclusive } => {
                let mut s = String::new();
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

            Expr::Await(expr) => format!("{}.A", self.format_expr(&expr.node)),
            Expr::Try(expr) => format!("{}?", self.format_expr(&expr.node)),
            Expr::Unwrap(expr) => format!("{}!", self.format_expr(&expr.node)),
            Expr::Ref(expr) => format!("&{}", self.format_expr(&expr.node)),
            Expr::Deref(expr) => format!("*{}", self.format_expr(&expr.node)),

            Expr::Assign { target, value } => {
                format!("{} = {}", self.format_expr(&target.node), self.format_expr(&value.node))
            }

            Expr::AssignOp { op, target, value } => {
                let op_str = match op {
                    BinOp::Add => "+=",
                    BinOp::Sub => "-=",
                    BinOp::Mul => "*=",
                    BinOp::Div => "/=",
                    _ => "?=",
                };
                format!("{} {} {}", self.format_expr(&target.node), op_str, self.format_expr(&value.node))
            }

            Expr::Lambda { params, body, .. } => {
                let params_str: Vec<String> = params.iter().map(|p| {
                    format!("{}: {}", p.name.node, self.format_type(&p.ty.node))
                }).collect();
                format!("|{}| {}", params_str.join(", "), self.format_expr(&body.node))
            }

            Expr::Spawn(expr) => format!("spawn {{ {} }}", self.format_expr(&expr.node)),
        }
    }

    /// Format statement inline (without leading indent)
    fn format_stmt_inline(&self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Let { name, ty, value, is_mut } => {
                let mut s = String::new();
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
        }
    }

    /// Format if-else branch
    fn format_if_else(&self, if_else: &IfElse) -> String {
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

    /// Format a pattern
    fn format_pattern(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Wildcard => "_".to_string(),
            Pattern::Ident(name) => name.to_string(),
            Pattern::Literal(lit) => match lit {
                Literal::Int(n) => n.to_string(),
                Literal::Float(n) => n.to_string(),
                Literal::Bool(b) => b.to_string(),
                Literal::String(s) => format!("\"{}\"", s),
            },
            Pattern::Tuple(patterns) => {
                let ps: Vec<String> = patterns.iter().map(|p| self.format_pattern(&p.node)).collect();
                format!("({})", ps.join(", "))
            }
            Pattern::Struct { name, fields } => {
                let fs: Vec<String> = fields.iter().map(|(n, p)| {
                    if let Some(pat) = p {
                        format!("{}: {}", n.node, self.format_pattern(&pat.node))
                    } else {
                        n.node.to_string()
                    }
                }).collect();
                format!("{} {{ {} }}", name.node, fs.join(", "))
            }
            Pattern::Variant { name, fields } => {
                if fields.is_empty() {
                    name.node.to_string()
                } else {
                    let fs: Vec<String> = fields.iter().map(|p| self.format_pattern(&p.node)).collect();
                    format!("{}({})", name.node, fs.join(", "))
                }
            }
            Pattern::Range { start, end, inclusive } => {
                let mut s = String::new();
                if let Some(st) = start {
                    s.push_str(&self.format_pattern(&st.node));
                }
                s.push_str(if *inclusive { "..=" } else { ".." });
                if let Some(en) = end {
                    s.push_str(&self.format_pattern(&en.node));
                }
                s
            }
            Pattern::Or(patterns) => {
                let ps: Vec<String> = patterns.iter().map(|p| self.format_pattern(&p.node)).collect();
                ps.join(" | ")
            }
        }
    }
}

