//! Static analysis lint rules for Vais code quality
//!
//! Provides AST-based lint checks for:
//! - Dead code (unreachable functions, unused struct fields)
//! - Unused imports
//! - Naming conventions
//! - Complexity warnings
//! - Unsafe code audit (pointer ops, extern blocks)

use std::collections::{HashMap, HashSet};
use vais_ast::*;

/// Lint severity levels (separate from security Severity for lint-specific use)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LintLevel {
    /// Informational hint
    Hint,
    /// Warning — code compiles but is suspicious
    Warning,
    /// Error — lint rule violation that should be fixed
    Error,
}

impl std::fmt::Display for LintLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintLevel::Hint => write!(f, "hint"),
            LintLevel::Warning => write!(f, "warning"),
            LintLevel::Error => write!(f, "error"),
        }
    }
}

/// Lint rule categories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LintCategory {
    /// Unreachable / never-called code
    DeadCode,
    /// Imported symbol never used
    UnusedImport,
    /// Variable declared but never read
    UnusedVariable,
    /// Naming convention violation
    NamingConvention,
    /// Function or block too complex
    Complexity,
    /// Unsafe operation that warrants review
    UnsafeAudit,
    /// Unreachable code after return/break
    UnreachableCode,
}

impl std::fmt::Display for LintCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintCategory::DeadCode => write!(f, "dead-code"),
            LintCategory::UnusedImport => write!(f, "unused-import"),
            LintCategory::UnusedVariable => write!(f, "unused-variable"),
            LintCategory::NamingConvention => write!(f, "naming-convention"),
            LintCategory::Complexity => write!(f, "complexity"),
            LintCategory::UnsafeAudit => write!(f, "unsafe-audit"),
            LintCategory::UnreachableCode => write!(f, "unreachable-code"),
        }
    }
}

/// A lint diagnostic produced by the analyzer
#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    /// Lint rule code (e.g., "L100", "L200")
    pub code: String,
    /// Severity
    pub level: LintLevel,
    /// Category
    pub category: LintCategory,
    /// Human-readable message
    pub message: String,
    /// Source location
    pub span: Span,
    /// Optional suggestion for fixing
    pub suggestion: Option<String>,
}

impl std::fmt::Display for LintDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {} ({}..{})",
            self.code, self.level, self.message, self.span.start, self.span.end
        )?;
        if let Some(ref sug) = self.suggestion {
            write!(f, "\n  suggestion: {}", sug)?;
        }
        Ok(())
    }
}

/// AST-based lint analyzer
pub struct LintAnalyzer {
    diagnostics: Vec<LintDiagnostic>,
    /// All function names defined in the module (name -> span)
    defined_functions: HashMap<String, Span>,
    /// All function names that are called somewhere
    called_functions: HashSet<String>,
    /// All identifiers used in expressions/statements
    used_idents: HashSet<String>,
    /// Import names (name -> span of use statement)
    imported_names: HashMap<String, Span>,
    /// Struct names defined
    defined_structs: HashSet<String>,
    /// Struct names used (in type annotations, constructors, etc.)
    used_types: HashSet<String>,
}

impl LintAnalyzer {
    /// Create a new lint analyzer
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            defined_functions: HashMap::new(),
            called_functions: HashSet::new(),
            used_idents: HashSet::new(),
            imported_names: HashMap::new(),
            defined_structs: HashSet::new(),
            used_types: HashSet::new(),
        }
    }

    /// Analyze a module and return all lint diagnostics
    pub fn analyze(&mut self, module: &Module) -> Vec<LintDiagnostic> {
        self.diagnostics.clear();
        self.defined_functions.clear();
        self.called_functions.clear();
        self.used_idents.clear();
        self.imported_names.clear();
        self.defined_structs.clear();
        self.used_types.clear();

        // Pass 1: Collect all definitions
        self.collect_definitions(module);

        // Pass 2: Collect all usages
        self.collect_usages(module);

        // Pass 3: Check rules
        self.check_dead_code();
        self.check_unused_imports();
        self.check_naming_conventions(module);
        self.check_complexity(module);
        self.check_unsafe_patterns(module);
        self.check_unreachable_code(module);

        self.diagnostics.clone()
    }

    // ==================== Pass 1: Collect Definitions ====================

    fn collect_definitions(&mut self, module: &Module) {
        for item in &module.items {
            match &item.node {
                Item::Function(func) => {
                    let name = &func.name.node;
                    // Skip main() — it's always the entry point
                    if name != "main" {
                        self.defined_functions.insert(name.clone(), func.name.span);
                    }
                }
                Item::Struct(s) => {
                    self.defined_structs.insert(s.name.node.clone());
                }
                Item::Use(use_stmt) => {
                    let span = item.span;
                    if let Some(items) = &use_stmt.items {
                        for sel_item in items {
                            self.imported_names.insert(sel_item.node.clone(), span);
                        }
                    } else if let Some(alias) = &use_stmt.alias {
                        self.imported_names.insert(alias.node.clone(), span);
                    } else if let Some(last) = use_stmt.path.last() {
                        self.imported_names.insert(last.node.clone(), span);
                    }
                }
                Item::Impl(impl_block) => {
                    // Methods in impl blocks are always considered "used" since they're
                    // dispatched via the type system, not directly called.
                    for method in &impl_block.methods {
                        self.called_functions.insert(method.node.name.node.clone());
                    }
                    // Track the target type as used
                    if let Type::Named { name, .. } = &impl_block.target_type.node {
                        self.used_types.insert(name.clone());
                    }
                }
                _ => {}
            }
        }
    }

    // ==================== Pass 2: Collect Usages ====================

    fn collect_usages(&mut self, module: &Module) {
        for item in &module.items {
            match &item.node {
                Item::Function(func) => {
                    self.collect_usages_in_function(func);
                }
                Item::Impl(impl_block) => {
                    // Impl target type is used
                    self.collect_usages_in_type(&impl_block.target_type.node);
                    for method in &impl_block.methods {
                        self.collect_usages_in_function(&method.node);
                    }
                }
                Item::Trait(trait_def) => {
                    for method in &trait_def.methods {
                        // TraitMethod has params, ret_type, and optional default_body
                        for param in &method.params {
                            self.collect_usages_in_type(&param.ty.node);
                        }
                        if let Some(ret) = &method.ret_type {
                            self.collect_usages_in_type(&ret.node);
                        }
                        if let Some(body) = &method.default_body {
                            match body {
                                FunctionBody::Expr(e) => self.collect_usages_in_expr(&e.node),
                                FunctionBody::Block(stmts) => {
                                    for s in stmts {
                                        self.collect_usages_in_stmt(&s.node);
                                    }
                                }
                            }
                        }
                    }
                }
                Item::Const(c) => {
                    self.collect_usages_in_expr(&c.value.node);
                }
                Item::Global(g) => {
                    self.collect_usages_in_expr(&g.value.node);
                }
                _ => {}
            }
        }
    }

    fn collect_usages_in_function(&mut self, func: &Function) {
        // Collect from parameter types
        for param in &func.params {
            self.collect_usages_in_type(&param.ty.node);
        }
        // Collect from return type
        if let Some(ret) = &func.ret_type {
            self.collect_usages_in_type(&ret.node);
        }
        // Collect from body
        match &func.body {
            FunctionBody::Expr(expr) => self.collect_usages_in_expr(&expr.node),
            FunctionBody::Block(stmts) => {
                for s in stmts {
                    self.collect_usages_in_stmt(&s.node);
                }
            }
        }
    }

    fn collect_usages_in_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { value, ty, .. } => {
                self.collect_usages_in_expr(&value.node);
                if let Some(ty_spanned) = ty {
                    self.collect_usages_in_type(&ty_spanned.node);
                }
            }
            Stmt::Expr(expr) => self.collect_usages_in_expr(&expr.node),
            Stmt::Return(Some(expr)) | Stmt::Break(Some(expr)) | Stmt::Defer(expr) => {
                self.collect_usages_in_expr(&expr.node);
            }
            Stmt::LetDestructure { value, .. } => {
                self.collect_usages_in_expr(&value.node);
            }
            _ => {}
        }
    }

    fn collect_usages_in_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(name) => {
                self.used_idents.insert(name.clone());
                self.called_functions.insert(name.clone()); // may be a function ref
                self.used_types.insert(name.clone());
            }
            Expr::Call { func, args } => {
                // Track direct function calls
                if let Expr::Ident(name) = &func.node {
                    self.called_functions.insert(name.clone());
                    self.used_idents.insert(name.clone());
                }
                self.collect_usages_in_expr(&func.node);
                for arg in args {
                    self.collect_usages_in_expr(&arg.node);
                }
            }
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                self.collect_usages_in_expr(&receiver.node);
                self.called_functions.insert(method.node.clone());
                self.used_idents.insert(method.node.clone());
                for arg in args {
                    self.collect_usages_in_expr(&arg.node);
                }
            }
            Expr::StaticMethodCall {
                type_name, args, ..
            } => {
                self.used_types.insert(type_name.node.clone());
                self.used_idents.insert(type_name.node.clone());
                for arg in args {
                    self.collect_usages_in_expr(&arg.node);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_usages_in_expr(&left.node);
                self.collect_usages_in_expr(&right.node);
            }
            Expr::Unary { expr, .. } => {
                self.collect_usages_in_expr(&expr.node);
            }
            Expr::If { cond, then, else_ } => {
                self.collect_usages_in_expr(&cond.node);
                for s in then {
                    self.collect_usages_in_stmt(&s.node);
                }
                if let Some(el) = else_ {
                    self.collect_usages_in_if_else(el);
                }
            }
            Expr::Ternary { cond, then, else_ } => {
                self.collect_usages_in_expr(&cond.node);
                self.collect_usages_in_expr(&then.node);
                self.collect_usages_in_expr(&else_.node);
            }
            Expr::Loop { iter, body, .. } => {
                if let Some(it) = iter {
                    self.collect_usages_in_expr(&it.node);
                }
                for s in body {
                    self.collect_usages_in_stmt(&s.node);
                }
            }
            Expr::While { condition, body } => {
                self.collect_usages_in_expr(&condition.node);
                for s in body {
                    self.collect_usages_in_stmt(&s.node);
                }
            }
            Expr::Match { expr, arms } => {
                self.collect_usages_in_expr(&expr.node);
                for arm in arms {
                    self.collect_usages_in_expr(&arm.body.node);
                    if let Some(guard) = &arm.guard {
                        self.collect_usages_in_expr(&guard.node);
                    }
                }
            }
            Expr::Lambda { body, .. } => {
                self.collect_usages_in_expr(&body.node);
            }
            Expr::Block(stmts) => {
                for s in stmts {
                    self.collect_usages_in_stmt(&s.node);
                }
            }
            Expr::Field { expr, .. } => {
                self.collect_usages_in_expr(&expr.node);
            }
            Expr::Index { expr, index } => {
                self.collect_usages_in_expr(&expr.node);
                self.collect_usages_in_expr(&index.node);
            }
            Expr::Array(elems) | Expr::Tuple(elems) => {
                for e in elems {
                    self.collect_usages_in_expr(&e.node);
                }
            }
            Expr::StructLit { name, fields, .. } => {
                self.used_types.insert(name.node.clone());
                self.used_idents.insert(name.node.clone());
                for (_, v) in fields {
                    self.collect_usages_in_expr(&v.node);
                }
            }
            Expr::Range { start, end, .. } => {
                if let Some(s) = start {
                    self.collect_usages_in_expr(&s.node);
                }
                if let Some(e) = end {
                    self.collect_usages_in_expr(&e.node);
                }
            }
            Expr::Assign { target, value } | Expr::AssignOp { target, value, .. } => {
                self.collect_usages_in_expr(&target.node);
                self.collect_usages_in_expr(&value.node);
            }
            Expr::Cast { expr, ty } => {
                self.collect_usages_in_expr(&expr.node);
                self.collect_usages_in_type(&ty.node);
            }
            Expr::Ref(inner) | Expr::Deref(inner) | Expr::Spread(inner) => {
                self.collect_usages_in_expr(&inner.node);
            }
            Expr::Await(inner) | Expr::Try(inner) | Expr::Unwrap(inner) => {
                self.collect_usages_in_expr(&inner.node);
            }
            Expr::Spawn(inner) | Expr::Lazy(inner) | Expr::Force(inner) => {
                self.collect_usages_in_expr(&inner.node);
            }
            Expr::Assert { condition, message } => {
                self.collect_usages_in_expr(&condition.node);
                if let Some(msg) = message {
                    self.collect_usages_in_expr(&msg.node);
                }
            }
            Expr::Assume(inner) | Expr::Old(inner) => {
                self.collect_usages_in_expr(&inner.node);
            }
            Expr::Comptime { body } => {
                self.collect_usages_in_expr(&body.node);
            }
            Expr::StringInterp(parts) => {
                for part in parts {
                    if let StringInterpPart::Expr(e) = part {
                        self.collect_usages_in_expr(&e.node);
                    }
                }
            }
            Expr::MapLit(pairs) => {
                for (k, v) in pairs {
                    self.collect_usages_in_expr(&k.node);
                    self.collect_usages_in_expr(&v.node);
                }
            }
            Expr::EnumAccess { data, .. } => {
                if let Some(d) = data {
                    self.collect_usages_in_expr(&d.node);
                }
            }
            Expr::String(_)
            | Expr::Int(_)
            | Expr::Float(_)
            | Expr::Bool(_)
            | Expr::Unit
            | Expr::SelfCall
            | Expr::MacroInvoke(_)
            | Expr::Yield(_)
            | Expr::Error { .. } => {}
        }
    }

    fn collect_usages_in_if_else(&mut self, branch: &IfElse) {
        match branch {
            IfElse::ElseIf(cond, stmts, next) => {
                self.collect_usages_in_expr(&cond.node);
                for s in stmts {
                    self.collect_usages_in_stmt(&s.node);
                }
                if let Some(n) = next {
                    self.collect_usages_in_if_else(n);
                }
            }
            IfElse::Else(stmts) => {
                for s in stmts {
                    self.collect_usages_in_stmt(&s.node);
                }
            }
        }
    }

    fn collect_usages_in_type(&mut self, ty: &Type) {
        match ty {
            Type::Named { name, generics } => {
                self.used_types.insert(name.clone());
                self.used_idents.insert(name.clone());
                for g in generics {
                    self.collect_usages_in_type(&g.node);
                }
            }
            Type::Array(inner)
            | Type::Slice(inner)
            | Type::SliceMut(inner)
            | Type::Ref(inner)
            | Type::RefMut(inner)
            | Type::Pointer(inner)
            | Type::Optional(inner)
            | Type::Result(inner)
            | Type::Lazy(inner)
            | Type::Linear(inner)
            | Type::Affine(inner) => {
                self.collect_usages_in_type(&inner.node);
            }
            Type::ConstArray { element, .. } => {
                self.collect_usages_in_type(&element.node);
            }
            Type::Map(k, v) => {
                self.collect_usages_in_type(&k.node);
                self.collect_usages_in_type(&v.node);
            }
            Type::Tuple(elems) => {
                for e in elems {
                    self.collect_usages_in_type(&e.node);
                }
            }
            Type::FnPtr { params, ret, .. } | Type::Fn { params, ret } => {
                for p in params {
                    self.collect_usages_in_type(&p.node);
                }
                self.collect_usages_in_type(&ret.node);
            }
            Type::RefLifetime { inner, .. } | Type::RefMutLifetime { inner, .. } => {
                self.collect_usages_in_type(&inner.node);
            }
            Type::DynTrait {
                trait_name,
                generics,
            } => {
                self.used_types.insert(trait_name.clone());
                self.used_idents.insert(trait_name.clone());
                for g in generics {
                    self.collect_usages_in_type(&g.node);
                }
            }
            Type::Associated {
                base,
                trait_name,
                assoc_name,
                generics,
            } => {
                self.collect_usages_in_type(&base.node);
                if let Some(tn) = trait_name {
                    self.used_types.insert(tn.clone());
                }
                self.used_idents.insert(assoc_name.clone());
                for g in generics {
                    self.collect_usages_in_type(&g.node);
                }
            }
            Type::ImplTrait { bounds } => {
                for b in bounds {
                    self.used_idents.insert(b.node.clone());
                }
            }
            Type::Dependent { base, .. } => {
                self.collect_usages_in_type(&base.node);
            }
            Type::Unit | Type::Infer => {}
        }
    }

    // ==================== Pass 3: Lint Rules ====================

    /// L100: Dead code — functions defined but never called
    fn check_dead_code(&mut self) {
        for (name, span) in &self.defined_functions {
            if !self.called_functions.contains(name) {
                // Skip pub functions (they may be used externally)
                // We'd need is_pub info, but conservatively flag non-underscore-prefixed
                if !name.starts_with('_') {
                    self.diagnostics.push(LintDiagnostic {
                        code: "L100".to_string(),
                        level: LintLevel::Warning,
                        category: LintCategory::DeadCode,
                        message: format!("function '{}' is defined but never called", name),
                        span: *span,
                        suggestion: Some(format!(
                            "Remove the function or prefix with '_' to suppress: '_{}'",
                            name
                        )),
                    });
                }
            }
        }
    }

    /// L200: Unused imports
    fn check_unused_imports(&mut self) {
        for (name, span) in &self.imported_names {
            if !self.used_idents.contains(name) && !self.used_types.contains(name) {
                self.diagnostics.push(LintDiagnostic {
                    code: "L200".to_string(),
                    level: LintLevel::Warning,
                    category: LintCategory::UnusedImport,
                    message: format!("imported name '{}' is never used", name),
                    span: *span,
                    suggestion: Some(format!("Remove the unused import of '{}'", name)),
                });
            }
        }
    }

    /// L300: Naming convention checks
    fn check_naming_conventions(&mut self, module: &Module) {
        for item in &module.items {
            match &item.node {
                Item::Function(func) => {
                    let name = &func.name.node;
                    // Functions should be snake_case
                    if name != "main" && !is_snake_case(name) && !name.starts_with('_') {
                        self.diagnostics.push(LintDiagnostic {
                            code: "L300".to_string(),
                            level: LintLevel::Hint,
                            category: LintCategory::NamingConvention,
                            message: format!("function '{}' should use snake_case naming", name),
                            span: func.name.span,
                            suggestion: Some(format!("Rename to '{}'", to_snake_case(name))),
                        });
                    }
                }
                Item::Struct(s) => {
                    let name = &s.name.node;
                    // Structs should be PascalCase
                    if !is_pascal_case(name) {
                        self.diagnostics.push(LintDiagnostic {
                            code: "L301".to_string(),
                            level: LintLevel::Hint,
                            category: LintCategory::NamingConvention,
                            message: format!("struct '{}' should use PascalCase naming", name),
                            span: s.name.span,
                            suggestion: None,
                        });
                    }
                }
                Item::Enum(e) => {
                    let name = &e.name.node;
                    if !is_pascal_case(name) {
                        self.diagnostics.push(LintDiagnostic {
                            code: "L302".to_string(),
                            level: LintLevel::Hint,
                            category: LintCategory::NamingConvention,
                            message: format!("enum '{}' should use PascalCase naming", name),
                            span: e.name.span,
                            suggestion: None,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    /// L400: Complexity warnings
    fn check_complexity(&mut self, module: &Module) {
        for item in &module.items {
            if let Item::Function(func) = &item.node {
                let stmt_count = match &func.body {
                    FunctionBody::Block(stmts) => count_statements(stmts),
                    FunctionBody::Expr(_) => 1,
                };

                // Flag functions with > 50 statements
                if stmt_count > 50 {
                    self.diagnostics.push(LintDiagnostic {
                        code: "L400".to_string(),
                        level: LintLevel::Warning,
                        category: LintCategory::Complexity,
                        message: format!(
                            "function '{}' has {} statements — consider splitting into smaller functions",
                            func.name.node, stmt_count
                        ),
                        span: func.name.span,
                        suggestion: Some("Extract logical sub-operations into helper functions".to_string()),
                    });
                }

                // Check nesting depth
                let max_depth = match &func.body {
                    FunctionBody::Block(stmts) => max_nesting_depth_stmts(stmts, 0),
                    FunctionBody::Expr(e) => max_nesting_depth_expr(&e.node, 0),
                };

                if max_depth > 5 {
                    self.diagnostics.push(LintDiagnostic {
                        code: "L401".to_string(),
                        level: LintLevel::Warning,
                        category: LintCategory::Complexity,
                        message: format!(
                            "function '{}' has nesting depth {} — consider refactoring",
                            func.name.node, max_depth
                        ),
                        span: func.name.span,
                        suggestion: Some(
                            "Use early returns or extract nested logic into helper functions"
                                .to_string(),
                        ),
                    });
                }
            }
        }
    }

    /// L500: Unsafe code audit — flags patterns that need review
    fn check_unsafe_patterns(&mut self, module: &Module) {
        for item in &module.items {
            match &item.node {
                Item::ExternBlock(block) => {
                    self.diagnostics.push(LintDiagnostic {
                        code: "L500".to_string(),
                        level: LintLevel::Warning,
                        category: LintCategory::UnsafeAudit,
                        message: format!(
                            "extern block declares {} FFI functions — review safety",
                            block.functions.len()
                        ),
                        span: item.span,
                        suggestion: Some(
                            "Wrap extern functions with safe Vais wrappers that validate inputs"
                                .to_string(),
                        ),
                    });
                }
                Item::Function(func) => {
                    self.check_unsafe_in_function(func);
                }
                Item::Impl(impl_block) => {
                    for method in &impl_block.methods {
                        self.check_unsafe_in_function(&method.node);
                    }
                }
                _ => {}
            }
        }
    }

    fn check_unsafe_in_function(&mut self, func: &Function) {
        match &func.body {
            FunctionBody::Block(stmts) => {
                for s in stmts {
                    self.check_unsafe_in_stmt(&s.node, s.span);
                }
            }
            FunctionBody::Expr(e) => {
                self.check_unsafe_in_expr(&e.node, e.span);
            }
        }
    }

    fn check_unsafe_in_stmt(&mut self, stmt: &Stmt, _span: Span) {
        match stmt {
            Stmt::Let { value, .. } => {
                self.check_unsafe_in_expr(&value.node, value.span);
            }
            Stmt::Expr(expr) => self.check_unsafe_in_expr(&expr.node, expr.span),
            Stmt::Return(Some(expr)) | Stmt::Break(Some(expr)) | Stmt::Defer(expr) => {
                self.check_unsafe_in_expr(&expr.node, expr.span);
            }
            Stmt::LetDestructure { value, .. } => {
                self.check_unsafe_in_expr(&value.node, value.span);
            }
            _ => {}
        }
    }

    fn check_unsafe_in_expr(&mut self, expr: &Expr, span: Span) {
        match expr {
            Expr::Deref(_) => {
                self.diagnostics.push(LintDiagnostic {
                    code: "L501".to_string(),
                    level: LintLevel::Warning,
                    category: LintCategory::UnsafeAudit,
                    message: "pointer dereference — ensure pointer validity".to_string(),
                    span,
                    suggestion: Some("Add null check before dereferencing".to_string()),
                });
            }
            Expr::Call { func, .. } => {
                if let Expr::Ident(name) = &func.node {
                    if matches!(
                        name.as_str(),
                        "malloc"
                            | "free"
                            | "memcpy"
                            | "memmove"
                            | "memset"
                            | "load_byte"
                            | "store_byte"
                            | "load_i64"
                            | "store_i64"
                    ) {
                        self.diagnostics.push(LintDiagnostic {
                            code: "L502".to_string(),
                            level: LintLevel::Warning,
                            category: LintCategory::UnsafeAudit,
                            message: format!("raw memory operation '{}' — review for safety", name),
                            span,
                            suggestion: Some(
                                "Prefer safe abstractions (Vec, slice) over raw memory ops"
                                    .to_string(),
                            ),
                        });
                    }
                }
            }
            Expr::If { cond, then, else_ } => {
                self.check_unsafe_in_expr(&cond.node, cond.span);
                for s in then {
                    self.check_unsafe_in_stmt(&s.node, s.span);
                }
                if let Some(el) = else_ {
                    self.check_unsafe_in_if_else(el);
                }
            }
            Expr::Loop { iter, body, .. } => {
                if let Some(it) = iter {
                    self.check_unsafe_in_expr(&it.node, it.span);
                }
                for s in body {
                    self.check_unsafe_in_stmt(&s.node, s.span);
                }
            }
            Expr::While { condition, body } => {
                self.check_unsafe_in_expr(&condition.node, condition.span);
                for s in body {
                    self.check_unsafe_in_stmt(&s.node, s.span);
                }
            }
            Expr::Match {
                expr: match_expr,
                arms,
            } => {
                self.check_unsafe_in_expr(&match_expr.node, match_expr.span);
                for arm in arms {
                    self.check_unsafe_in_expr(&arm.body.node, arm.body.span);
                }
            }
            Expr::Block(stmts) => {
                for s in stmts {
                    self.check_unsafe_in_stmt(&s.node, s.span);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.check_unsafe_in_expr(&left.node, left.span);
                self.check_unsafe_in_expr(&right.node, right.span);
            }
            _ => {}
        }
    }

    fn check_unsafe_in_if_else(&mut self, branch: &IfElse) {
        match branch {
            IfElse::ElseIf(cond, stmts, next) => {
                self.check_unsafe_in_expr(&cond.node, cond.span);
                for s in stmts {
                    self.check_unsafe_in_stmt(&s.node, s.span);
                }
                if let Some(n) = next {
                    self.check_unsafe_in_if_else(n);
                }
            }
            IfElse::Else(stmts) => {
                for s in stmts {
                    self.check_unsafe_in_stmt(&s.node, s.span);
                }
            }
        }
    }

    /// L600: Unreachable code after return/break
    fn check_unreachable_code(&mut self, module: &Module) {
        for item in &module.items {
            if let Item::Function(func) = &item.node {
                if let FunctionBody::Block(stmts) = &func.body {
                    self.check_unreachable_stmts(stmts);
                }
            }
        }
    }

    fn check_unreachable_stmts(&mut self, stmts: &[Spanned<Stmt>]) {
        let mut found_terminator = false;
        for stmt in stmts {
            if found_terminator {
                self.diagnostics.push(LintDiagnostic {
                    code: "L600".to_string(),
                    level: LintLevel::Warning,
                    category: LintCategory::UnreachableCode,
                    message: "unreachable code after return/break statement".to_string(),
                    span: stmt.span,
                    suggestion: Some("Remove the unreachable code".to_string()),
                });
                break; // Only report the first unreachable statement
            }
            match &stmt.node {
                Stmt::Return(_) | Stmt::Break(_) | Stmt::Continue => {
                    found_terminator = true;
                }
                _ => {}
            }
        }
    }
}

impl Default for LintAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Helper Functions ====================

/// Check if a name follows snake_case convention
fn is_snake_case(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }
    // Allow leading underscore(s)
    let stripped = name.trim_start_matches('_');
    if stripped.is_empty() {
        return true;
    }
    // Must not start with uppercase
    if stripped.starts_with(char::is_uppercase) {
        return false;
    }
    // Only lowercase letters, digits, and underscores
    stripped
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Check if a name follows PascalCase convention
fn is_pascal_case(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }
    // Must start with uppercase
    if !name.starts_with(char::is_uppercase) {
        return false;
    }
    // Must not contain underscores (except possible trailing ones)
    !name.contains('_')
}

/// Convert a name to snake_case
fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Count statements recursively (for complexity check)
fn count_statements(stmts: &[Spanned<Stmt>]) -> usize {
    let mut count = 0;
    for stmt in stmts {
        count += 1;
        match &stmt.node {
            Stmt::Expr(expr) => count += count_statements_in_expr(&expr.node),
            Stmt::Let { value, .. } => count += count_statements_in_expr(&value.node),
            _ => {}
        }
    }
    count
}

fn count_statements_in_expr(expr: &Expr) -> usize {
    match expr {
        Expr::If { then, else_, .. } => {
            let mut c = then.len();
            if let Some(el) = else_ {
                c += count_statements_in_if_else(el);
            }
            c
        }
        Expr::Loop { body, .. } | Expr::While { body, .. } => body.len(),
        Expr::Block(stmts) => stmts.len(),
        Expr::Match { arms, .. } => arms.len(),
        _ => 0,
    }
}

fn count_statements_in_if_else(branch: &IfElse) -> usize {
    match branch {
        IfElse::ElseIf(_, stmts, next) => {
            let mut c = stmts.len();
            if let Some(n) = next {
                c += count_statements_in_if_else(n);
            }
            c
        }
        IfElse::Else(stmts) => stmts.len(),
    }
}

/// Calculate maximum nesting depth for statements
fn max_nesting_depth_stmts(stmts: &[Spanned<Stmt>], current: usize) -> usize {
    let mut max_depth = current;
    for stmt in stmts {
        let d = match &stmt.node {
            Stmt::Expr(expr) => max_nesting_depth_expr(&expr.node, current),
            Stmt::Let { value, .. } => max_nesting_depth_expr(&value.node, current),
            _ => current,
        };
        if d > max_depth {
            max_depth = d;
        }
    }
    max_depth
}

/// Calculate maximum nesting depth for expressions
fn max_nesting_depth_expr(expr: &Expr, current: usize) -> usize {
    match expr {
        Expr::If { then, else_, .. } => {
            let mut d = max_nesting_depth_stmts(then, current + 1);
            if let Some(el) = else_ {
                let ed = max_nesting_depth_if_else(el, current + 1);
                if ed > d {
                    d = ed;
                }
            }
            d
        }
        Expr::Loop { body, .. } | Expr::While { body, .. } => {
            max_nesting_depth_stmts(body, current + 1)
        }
        Expr::Match { arms, .. } => {
            let mut d = current + 1;
            for arm in arms {
                let ad = max_nesting_depth_expr(&arm.body.node, current + 1);
                if ad > d {
                    d = ad;
                }
            }
            d
        }
        Expr::Block(stmts) => max_nesting_depth_stmts(stmts, current + 1),
        _ => current,
    }
}

fn max_nesting_depth_if_else(branch: &IfElse, current: usize) -> usize {
    match branch {
        IfElse::ElseIf(_, stmts, next) => {
            let mut d = max_nesting_depth_stmts(stmts, current);
            if let Some(n) = next {
                let nd = max_nesting_depth_if_else(n, current);
                if nd > d {
                    d = nd;
                }
            }
            d
        }
        IfElse::Else(stmts) => max_nesting_depth_stmts(stmts, current),
    }
}

#[cfg(test)]
#[path = "lint_tests.rs"]
mod lint_tests;
