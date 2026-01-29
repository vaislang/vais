//! Static security analyzer for Vais AST

use vais_ast::*;
use crate::findings::*;
use std::collections::HashSet;

/// Security analyzer that walks the AST looking for security issues
pub struct SecurityAnalyzer {
    /// Accumulated security findings
    findings: Vec<SecurityFinding>,
    /// Track allocated pointers (malloc calls)
    allocated_vars: HashSet<String>,
    /// Track freed pointers
    freed_vars: HashSet<String>,
}

impl SecurityAnalyzer {
    /// Creates a new security analyzer
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            allocated_vars: HashSet::new(),
            freed_vars: HashSet::new(),
        }
    }

    /// Analyzes a module and returns all security findings
    pub fn analyze(&mut self, module: &Module) -> Vec<SecurityFinding> {
        self.findings.clear();
        self.allocated_vars.clear();
        self.freed_vars.clear();

        for item in &module.items {
            self.analyze_item(&item.node, item.span);
        }

        self.findings.clone()
    }

    /// Analyzes a top-level item
    fn analyze_item(&mut self, item: &Item, _span: Span) {
        match item {
            Item::Function(func) => self.analyze_function(func),
            Item::Struct(_) => {},
            Item::Enum(_) => {},
            Item::Union(_) => {},
            Item::TypeAlias(_) => {},
            Item::Use(_) => {},
            Item::Trait(_) => {},
            Item::Impl(impl_block) => {
                for method in &impl_block.methods {
                    self.analyze_function(&method.node);
                }
            },
            Item::Macro(_) => {},
            Item::ExternBlock(extern_block) => {
                self.analyze_extern_block(extern_block);
            },
            Item::Const(const_def) => {
                self.analyze_expr(&const_def.value.node, const_def.value.span);
            },
            Item::Global(global_def) => {
                self.analyze_expr(&global_def.value.node, global_def.value.span);
            },
            Item::Error { .. } => {},
        }
    }

    /// Analyzes a function
    fn analyze_function(&mut self, func: &Function) {
        // Reset per-function state
        self.allocated_vars.clear();
        self.freed_vars.clear();

        match &func.body {
            FunctionBody::Expr(expr) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            FunctionBody::Block(stmts) => {
                for stmt in stmts {
                    self.analyze_stmt(&stmt.node, stmt.span);
                }
            },
        }
    }

    /// Analyzes an extern block
    fn analyze_extern_block(&mut self, block: &ExternBlock) {
        // Check for unsafe extern functions
        for func in &block.functions {
            let func_name = &func.name.node;

            // Flag dangerous C functions
            if matches!(func_name.as_str(), "strcpy" | "strcat" | "gets" | "sprintf" | "scanf") {
                self.findings.push(SecurityFinding::buffer_overflow(
                    format!("Extern function '{}' is unsafe and prone to buffer overflows", func_name),
                    func.name.span,
                ));
            }

            // Flag system/exec functions
            if matches!(func_name.as_str(), "system" | "exec" | "popen") {
                self.findings.push(SecurityFinding::injection(
                    format!("Extern function '{}' can execute commands - high injection risk", func_name),
                    func.name.span,
                ));
            }
        }
    }

    /// Analyzes a statement
    fn analyze_stmt(&mut self, stmt: &Stmt, _span: Span) {
        match stmt {
            Stmt::Let { name, value, .. } => {
                // Check for suspicious variable names with string values
                if let Expr::String(s) = &value.node {
                    let var_lower = name.node.to_lowercase();
                    if (var_lower.contains("key") || var_lower.contains("token") ||
                        var_lower.contains("secret") || var_lower.contains("password")) && s.len() > 10 {
                        self.findings.push(SecurityFinding::hardcoded_secret(
                            format!("Suspicious string literal assigned to '{}': '{}'",
                                    name.node, self.truncate_string(s, 40)),
                            value.span,
                            Severity::High,
                        ));
                    }
                }

                self.analyze_expr(&value.node, value.span);

                // Track malloc allocations
                if let Expr::Call { func, .. } = &value.node {
                    if let Expr::Ident(func_name) = &func.node {
                        if func_name == "malloc" {
                            self.allocated_vars.insert(name.node.clone());
                        }
                    }
                }
            },
            Stmt::Expr(expr) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Stmt::Return(Some(expr)) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Stmt::Return(None) => {},
            Stmt::Break(Some(expr)) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Stmt::Break(None) => {},
            Stmt::Continue => {},
            Stmt::Defer(expr) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Stmt::Error { .. } => {},
        }
    }

    /// Analyzes an expression
    fn analyze_expr(&mut self, expr: &Expr, span: Span) {
        match expr {
            Expr::String(s) => {
                self.check_hardcoded_secrets(s, span);
            },
            Expr::Binary { op, left, right } => {
                self.analyze_expr(&left.node, left.span);
                self.analyze_expr(&right.node, right.span);

                // Check for unsafe pointer arithmetic
                if matches!(op, BinOp::Add | BinOp::Sub)
                    && (self.is_pointer_expr(&left.node) || self.is_pointer_expr(&right.node)) {
                    self.findings.push(SecurityFinding::unsafe_pointer(
                        "Pointer arithmetic detected - may lead to buffer overflows",
                        span,
                    ));
                }

                // Check for integer overflow risks in arithmetic
                if matches!(op, BinOp::Add | BinOp::Mul | BinOp::Sub)
                    && (self.operates_on_user_input(&left.node) || self.operates_on_user_input(&right.node)) {
                    self.findings.push(SecurityFinding::integer_overflow(
                        "Arithmetic operation on potentially unchecked input may overflow".to_string(),
                        span,
                    ));
                }
            },
            Expr::Unary { expr, .. } => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Expr::Ternary { cond, then, else_ } => {
                self.analyze_expr(&cond.node, cond.span);
                self.analyze_expr(&then.node, then.span);
                self.analyze_expr(&else_.node, else_.span);
            },
            Expr::If { cond, then, else_ } => {
                self.analyze_expr(&cond.node, cond.span);
                for stmt in then {
                    self.analyze_stmt(&stmt.node, stmt.span);
                }
                if let Some(else_branch) = else_ {
                    self.analyze_if_else(else_branch);
                }
            },
            Expr::Loop { pattern: _, iter, body } => {
                if let Some(iter_expr) = iter {
                    self.analyze_expr(&iter_expr.node, iter_expr.span);
                }
                for stmt in body {
                    self.analyze_stmt(&stmt.node, stmt.span);
                }
            },
            Expr::While { condition, body } => {
                self.analyze_expr(&condition.node, condition.span);
                for stmt in body {
                    self.analyze_stmt(&stmt.node, stmt.span);
                }
            },
            Expr::Match { expr, arms } => {
                self.analyze_expr(&expr.node, expr.span);
                for arm in arms {
                    self.analyze_expr(&arm.body.node, arm.body.span);
                }
            },
            Expr::Call { func, args } => {
                self.analyze_expr(&func.node, func.span);
                for arg in args {
                    self.analyze_expr(&arg.node, arg.span);
                }

                self.analyze_call(func, args, span);
            },
            Expr::MethodCall { receiver, method: _, args } => {
                self.analyze_expr(&receiver.node, receiver.span);
                for arg in args {
                    self.analyze_expr(&arg.node, arg.span);
                }
            },
            Expr::StaticMethodCall { args, .. } => {
                for arg in args {
                    self.analyze_expr(&arg.node, arg.span);
                }
            },
            Expr::Field { expr, .. } => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Expr::Index { expr, index } => {
                self.analyze_expr(&expr.node, expr.span);
                self.analyze_expr(&index.node, index.span);

                // Check for potential out-of-bounds access
                self.findings.push(SecurityFinding::buffer_overflow(
                    "Array indexing without bounds checking may cause buffer overflow",
                    span,
                ));
            },
            Expr::Array(elements) => {
                for elem in elements {
                    self.analyze_expr(&elem.node, elem.span);
                }
            },
            Expr::Tuple(elements) => {
                for elem in elements {
                    self.analyze_expr(&elem.node, elem.span);
                }
            },
            Expr::StructLit { fields, .. } => {
                for (_, value) in fields {
                    self.analyze_expr(&value.node, value.span);
                }
            },
            Expr::Range { start, end, .. } => {
                if let Some(start_expr) = start {
                    self.analyze_expr(&start_expr.node, start_expr.span);
                }
                if let Some(end_expr) = end {
                    self.analyze_expr(&end_expr.node, end_expr.span);
                }
            },
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.analyze_stmt(&stmt.node, stmt.span);
                }
            },
            Expr::Await(expr) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Expr::Try(expr) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Expr::Unwrap(expr) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Expr::Ref(expr) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Expr::Deref(expr) => {
                self.analyze_expr(&expr.node, expr.span);

                // Check for dereferencing potentially invalid pointers
                if self.is_potentially_freed(&expr.node) {
                    self.findings.push(SecurityFinding::use_after_free(
                        "Dereferencing a potentially freed pointer",
                        span,
                    ));
                }
            },
            Expr::Cast { expr, .. } => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Expr::Assign { target, value } => {
                self.analyze_expr(&target.node, target.span);
                self.analyze_expr(&value.node, value.span);
            },
            Expr::AssignOp { target, value, .. } => {
                self.analyze_expr(&target.node, target.span);
                self.analyze_expr(&value.node, value.span);
            },
            Expr::Lambda { body, .. } => {
                self.analyze_expr(&body.node, body.span);
            },
            Expr::Spawn(expr) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Expr::Comptime { body } => {
                self.analyze_expr(&body.node, body.span);
            },
            Expr::Assert { condition, message } => {
                self.analyze_expr(&condition.node, condition.span);
                if let Some(msg) = message {
                    self.analyze_expr(&msg.node, msg.span);
                }
            },
            Expr::Assume(expr) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Expr::Old(expr) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Expr::Lazy(expr) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            Expr::Force(expr) => {
                self.analyze_expr(&expr.node, expr.span);
            },
            // Literals and simple expressions
            Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) | Expr::Unit |
            Expr::Ident(_) | Expr::SelfCall | Expr::MacroInvoke(_) | Expr::Error { .. } => {},
        }
    }

    /// Analyzes a function call for security issues
    fn analyze_call(&mut self, func: &Spanned<Expr>, args: &[Spanned<Expr>], span: Span) {
        if let Expr::Ident(func_name) = &func.node {
            match func_name.as_str() {
                // Memory operations - buffer overflow risks
                "malloc" => {
                    self.findings.push(SecurityFinding::buffer_overflow(
                        "Manual memory allocation with malloc - ensure proper size calculation and bounds checking",
                        span,
                    ));
                },
                "free" => {
                    // Track freed variables
                    if let Some(arg) = args.first() {
                        if let Expr::Ident(var_name) = &arg.node {
                            self.freed_vars.insert(var_name.clone());
                        }
                    }
                    self.findings.push(SecurityFinding::use_after_free(
                        "Manual memory deallocation - ensure no use-after-free",
                        span,
                    ));
                },
                "load_byte" | "store_byte" | "load_i64" | "store_i64" => {
                    self.findings.push(SecurityFinding::buffer_overflow(
                        format!("Direct memory operation '{}' without bounds checking", func_name),
                        span,
                    ));
                },
                "memcpy" | "memmove" | "memset" => {
                    self.findings.push(SecurityFinding::buffer_overflow(
                        format!("Unsafe memory operation '{}' - verify buffer sizes", func_name),
                        span,
                    ));
                },
                // Command execution - injection risks
                "system" | "exec" | "execve" | "popen" => {
                    // Check if string concatenation is used in arguments
                    for arg in args {
                        if self.contains_string_concat(&arg.node) {
                            self.findings.push(SecurityFinding::injection(
                                format!("Command injection risk in '{}' - string concatenation detected", func_name),
                                span,
                            ));
                        }
                    }
                    self.findings.push(SecurityFinding::injection(
                        format!("Command execution function '{}' - validate and sanitize inputs", func_name),
                        span,
                    ));
                },
                // SQL-like operations
                "query" | "execute" | "sql" | "db_query" => {
                    for arg in args {
                        if self.contains_string_concat(&arg.node) {
                            self.findings.push(SecurityFinding::injection(
                                "SQL injection risk - use parameterized queries instead of string concatenation",
                                span,
                            ));
                        }
                    }
                },
                // Unchecked operations
                "unwrap" => {
                    self.findings.push(SecurityFinding::unchecked_error(
                        "Using 'unwrap' may panic - consider proper error handling",
                        span,
                    ));
                },
                _ => {},
            }
        }
    }

    /// Analyzes if-else branches
    fn analyze_if_else(&mut self, else_branch: &IfElse) {
        match else_branch {
            IfElse::ElseIf(cond, then, next) => {
                self.analyze_expr(&cond.node, cond.span);
                for stmt in then {
                    self.analyze_stmt(&stmt.node, stmt.span);
                }
                if let Some(next_branch) = next {
                    self.analyze_if_else(next_branch);
                }
            },
            IfElse::Else(stmts) => {
                for stmt in stmts {
                    self.analyze_stmt(&stmt.node, stmt.span);
                }
            },
        }
    }

    /// Checks if an expression involves pointer types
    fn is_pointer_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(name) => {
                // Check if variable is from malloc (pointer type)
                self.allocated_vars.contains(name)
            },
            Expr::Call { func, .. } => {
                if let Expr::Ident(func_name) = &func.node {
                    func_name == "malloc"
                } else {
                    false
                }
            },
            _ => false,
        }
    }

    /// Checks if an expression is potentially a freed pointer
    fn is_potentially_freed(&self, expr: &Expr) -> bool {
        if let Expr::Ident(name) = expr {
            self.freed_vars.contains(name)
        } else {
            false
        }
    }

    /// Checks if an expression contains string concatenation
    fn contains_string_concat(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Binary { op: BinOp::Add, left, right } => {
                self.is_string_expr(&left.node) || self.is_string_expr(&right.node) ||
                self.contains_string_concat(&left.node) || self.contains_string_concat(&right.node)
            },
            Expr::Call { func, .. } => {
                if let Expr::Ident(name) = &func.node {
                    matches!(name.as_str(), "format" | "concat" | "join")
                } else {
                    false
                }
            },
            _ => false,
        }
    }

    /// Checks if an expression is a string type
    fn is_string_expr(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::String(_))
    }

    /// Checks if an expression operates on potentially unchecked user input
    fn operates_on_user_input(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call { func, .. } => {
                if let Expr::Ident(name) = &func.node {
                    // Functions that typically return user input
                    matches!(name.as_str(), "read" | "readln" | "input" | "getline" |
                             "recv" | "read_file" | "get_arg" | "parse")
                } else {
                    false
                }
            },
            Expr::Ident(name) => {
                // Variable names that suggest user input
                name.contains("input") || name.contains("user") || name.contains("arg")
            },
            _ => false,
        }
    }

    /// Checks for hardcoded secrets in strings
    fn check_hardcoded_secrets(&mut self, s: &str, span: Span) {
        let lower = s.to_lowercase();

        // Check for high-entropy strings (potential tokens/keys)
        if s.len() > 20 && self.is_high_entropy(s)
            && (lower.contains("key") || lower.contains("token") || lower.contains("secret") ||
               lower.contains("password") || lower.contains("api")) {
            self.findings.push(SecurityFinding::hardcoded_secret(
                format!("Potential hardcoded secret detected: '{}'", self.truncate_string(s, 40)),
                span,
                Severity::Critical,
            ));
            return;
        }

        // Check for common secret patterns
        if lower.contains("password") && s.len() > 3 {
            self.findings.push(SecurityFinding::hardcoded_secret(
                format!("Hardcoded password detected: '{}'", self.truncate_string(s, 40)),
                span,
                Severity::High,
            ));
        } else if lower.contains("api_key") || lower.contains("apikey") {
            self.findings.push(SecurityFinding::hardcoded_secret(
                format!("Hardcoded API key detected: '{}'", self.truncate_string(s, 40)),
                span,
                Severity::High,
            ));
        } else if lower.contains("secret") && s.len() > 10 {
            self.findings.push(SecurityFinding::hardcoded_secret(
                format!("Hardcoded secret detected: '{}'", self.truncate_string(s, 40)),
                span,
                Severity::Medium,
            ));
        } else if s.starts_with("sk-") || s.starts_with("pk_") {
            // Common API key prefixes
            self.findings.push(SecurityFinding::hardcoded_secret(
                format!("Hardcoded token detected: '{}'", self.truncate_string(s, 40)),
                span,
                Severity::Critical,
            ));
        }
    }

    /// Calculates entropy of a string (for detecting random tokens)
    fn is_high_entropy(&self, s: &str) -> bool {
        use std::collections::HashMap;

        if s.len() < 20 {
            return false;
        }

        let mut freq: HashMap<char, usize> = HashMap::new();
        for c in s.chars() {
            *freq.entry(c).or_insert(0) += 1;
        }

        let len = s.len() as f64;
        let mut entropy = 0.0;
        for count in freq.values() {
            let p = *count as f64 / len;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }

        // High entropy threshold (above 4.0 bits per character)
        // Random base64: ~6 bits, repeated chars: ~0 bits, normal text: ~2-3 bits
        entropy > 4.0
    }

    /// Truncates a string for display
    fn truncate_string(&self, s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
    }
}

impl Default for SecurityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_entropy_detection() {
        let analyzer = SecurityAnalyzer::new();

        // High entropy string (random token)
        assert!(analyzer.is_high_entropy("xK7mP9qR2sL5nW8tV3gH4jF6bN1cM0dZ"));

        // Low entropy string (repeated characters)
        assert!(!analyzer.is_high_entropy("aaaaaaaaaaaaaaaaaaaaaaa"));

        // Medium entropy (words)
        assert!(!analyzer.is_high_entropy("this is a normal sentence"));
    }
}
