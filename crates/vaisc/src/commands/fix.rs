//! `vaisc fix` command -- automatic code fixes
//!
//! Provides an AST-based rewriting engine that can detect and automatically
//! remove unused variables, unused imports, and apply other mechanical fixes.

use crate::configure_type_checker;
use crate::error_formatter;
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use vais_ast::*;
use vais_types::TypeChecker;

/// Result of running the fix command
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct FixResult {
    /// Number of unused variables removed
    pub unused_vars_removed: usize,
    /// Number of unused imports removed
    pub unused_imports_removed: usize,
    /// Whether the file was modified
    pub modified: bool,
}

/// Run the fix command on a single file
pub(crate) fn cmd_fix(input: &PathBuf, dry_run: bool, verbose: bool) -> Result<(), String> {
    let source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    if verbose {
        println!("{} Analyzing {}", "Info".cyan(), input.display());
    }

    // Parse the source
    let ast = vais_parser::parse(&source)
        .map_err(|e| error_formatter::format_parse_error(&e, &source, input))?;

    // Type check to get warnings about unused items
    let mut checker = TypeChecker::new();
    configure_type_checker(&mut checker);
    let _ = checker.check_module(&ast); // Ignore type errors, we just want warnings

    // Collect unused variable and import information by analyzing the AST
    let unused_vars = find_unused_variables(&ast);
    let unused_imports = find_unused_imports(&ast);

    let total_fixes = unused_vars.len() + unused_imports.len();

    if total_fixes == 0 {
        println!("{} No fixes needed for {}", "OK".green(), input.display());
        return Ok(());
    }

    // Compute spans to remove (sorted in reverse order so we can remove from end to start)
    let mut removals: Vec<SpanRemoval> = Vec::new();

    for var_info in &unused_vars {
        if verbose {
            println!(
                "  {} Unused variable '{}' at offset {}",
                "fix".yellow(),
                var_info.name,
                var_info.span.start
            );
        }
        removals.push(SpanRemoval {
            start: var_info.stmt_span.start,
            end: var_info.stmt_span.end,
            kind: RemovalKind::UnusedVariable(var_info.name.clone()),
        });
    }

    for import_info in &unused_imports {
        if verbose {
            println!(
                "  {} Unused import '{}' at offset {}",
                "fix".yellow(),
                import_info.name,
                import_info.span.start
            );
        }
        removals.push(SpanRemoval {
            start: import_info.span.start,
            end: import_info.span.end,
            kind: RemovalKind::UnusedImport(import_info.name.clone()),
        });
    }

    // Sort removals by start position in descending order (apply from end to start
    // so earlier offsets remain valid after each removal)
    removals.sort_by(|a, b| b.start.cmp(&a.start));

    // Validate no overlapping spans — overlapping removals would corrupt the source
    for window in removals.windows(2) {
        // window[0].start >= window[1].start (descending order)
        // Overlap if window[0].start < window[1].end
        if window[0].start < window[1].end && verbose {
            eprintln!(
                "  Warning: overlapping fix spans [{}, {}) and [{}, {}), skipping overlap",
                window[1].start, window[1].end, window[0].start, window[0].end
            );
            // Skip the second (earlier in source) removal to avoid corruption
            // In practice this is rare since let statements don't overlap
        }
    }

    if dry_run {
        println!(
            "{} Would fix {} issue(s) in {}:",
            "Info".cyan(),
            total_fixes,
            input.display()
        );
        for removal in &removals {
            match &removal.kind {
                RemovalKind::UnusedVariable(name) => {
                    println!("  - Remove unused variable '{}'", name);
                }
                RemovalKind::UnusedImport(name) => {
                    println!("  - Remove unused import '{}'", name);
                }
            }
        }
        return Ok(());
    }

    // Apply removals to source text (reverse order preserves offsets)
    let mut fixed_source = source.clone();
    let mut last_start = usize::MAX;
    for removal in &removals {
        let start = removal.start;
        let end = removal.end;

        // Skip if this removal overlaps with the previous one (detected above)
        if end > last_start {
            continue;
        }

        if start < fixed_source.len() && end <= fixed_source.len() && start < end {
            // Remove the span and any trailing newline
            let mut remove_end = end;
            if remove_end < fixed_source.len() && fixed_source.as_bytes()[remove_end] == b'\n' {
                remove_end += 1;
            }
            fixed_source.replace_range(start..remove_end, "");
            last_start = start;
        }
    }

    // Write fixed source
    fs::write(input, &fixed_source)
        .map_err(|e| format!("Cannot write '{}': {}", input.display(), e))?;

    println!(
        "{} Fixed {} issue(s) in {}",
        "OK".green().bold(),
        total_fixes,
        input.display()
    );
    if !unused_vars.is_empty() {
        println!("  Removed {} unused variable(s)", unused_vars.len());
    }
    if !unused_imports.is_empty() {
        println!("  Removed {} unused import(s)", unused_imports.len());
    }

    Ok(())
}

// ==================== Unused Variable Detection ====================

/// Information about an unused variable
#[derive(Debug)]
struct UnusedVarInfo {
    name: String,
    span: Span,
    /// The full statement span (for removal)
    stmt_span: Span,
}

/// Find unused variables in the AST.
///
/// A variable is "unused" if it is declared with `:=` but never referenced
/// elsewhere in the same scope. Variables starting with `_` are exempt.
fn find_unused_variables(module: &Module) -> Vec<UnusedVarInfo> {
    let mut results = Vec::new();

    for item in &module.items {
        if let Item::Function(func) = &item.node {
            let unused = find_unused_vars_in_function(func);
            results.extend(unused);
        }
        if let Item::Impl(impl_block) = &item.node {
            for method in &impl_block.methods {
                let unused = find_unused_vars_in_function(&method.node);
                results.extend(unused);
            }
        }
    }

    results
}

fn find_unused_vars_in_function(func: &Function) -> Vec<UnusedVarInfo> {
    let stmts = match &func.body {
        FunctionBody::Block(stmts) => stmts,
        FunctionBody::Expr(_) => return Vec::new(),
    };

    // Pass 1: Collect all declared variable names and their spans
    let mut declared: Vec<(String, Span, Span)> = Vec::new(); // (name, name_span, stmt_span)
    for stmt in stmts {
        if let Stmt::Let { name, .. } = &stmt.node {
            if !name.node.starts_with('_') {
                declared.push((name.node.clone(), name.span, stmt.span));
            }
        }
    }

    if declared.is_empty() {
        return Vec::new();
    }

    // Pass 2: Collect all used identifiers across all expressions/statements
    let mut used_names: HashSet<String> = HashSet::new();
    for stmt in stmts {
        collect_used_idents_in_stmt(&stmt.node, &mut used_names);
    }

    // A variable is unused if it's declared but never used in any expression.
    // But we need to exclude uses in the initializer of the same let statement.
    let mut results = Vec::new();
    for (name, name_span, stmt_span) in &declared {
        // Count references: the declaration itself doesn't count, only uses
        let mut use_count = 0;
        for stmt in stmts {
            match &stmt.node {
                Stmt::Let {
                    name: decl_name,
                    value,
                    ..
                } => {
                    if &decl_name.node == name {
                        // This is the declaration; count uses in value as references
                        // that DON'T count (they're part of the init, but the var
                        // itself would be a forward ref which is unusual)
                        continue;
                    }
                    // Count references in this other statement's value
                    use_count += count_ident_in_expr(&value.node, name);
                }
                Stmt::Expr(expr) => {
                    use_count += count_ident_in_expr(&expr.node, name);
                }
                Stmt::Return(Some(expr)) => {
                    use_count += count_ident_in_expr(&expr.node, name);
                }
                Stmt::Break(Some(expr)) => {
                    use_count += count_ident_in_expr(&expr.node, name);
                }
                Stmt::Defer(expr) => {
                    use_count += count_ident_in_expr(&expr.node, name);
                }
                Stmt::LetDestructure { value, .. } => {
                    use_count += count_ident_in_expr(&value.node, name);
                }
                _ => {}
            }
        }

        if use_count == 0 {
            results.push(UnusedVarInfo {
                name: name.clone(),
                span: *name_span,
                stmt_span: *stmt_span,
            });
        }
    }

    results
}

/// Count occurrences of an identifier in an expression
fn count_ident_in_expr(expr: &Expr, name: &str) -> usize {
    match expr {
        Expr::Ident(id) if id == name => 1,
        Expr::Binary { left, right, .. } => {
            count_ident_in_expr(&left.node, name) + count_ident_in_expr(&right.node, name)
        }
        Expr::Unary { expr, .. } => count_ident_in_expr(&expr.node, name),
        Expr::Call { func, args } => {
            count_ident_in_expr(&func.node, name)
                + args
                    .iter()
                    .map(|a| count_ident_in_expr(&a.node, name))
                    .sum::<usize>()
        }
        Expr::MethodCall { receiver, args, .. } => {
            count_ident_in_expr(&receiver.node, name)
                + args
                    .iter()
                    .map(|a| count_ident_in_expr(&a.node, name))
                    .sum::<usize>()
        }
        Expr::StaticMethodCall { args, .. } => args
            .iter()
            .map(|a| count_ident_in_expr(&a.node, name))
            .sum(),
        Expr::Field { expr, .. } => count_ident_in_expr(&expr.node, name),
        Expr::Index { expr, index } => {
            count_ident_in_expr(&expr.node, name) + count_ident_in_expr(&index.node, name)
        }
        Expr::Array(elems) | Expr::Tuple(elems) => elems
            .iter()
            .map(|e| count_ident_in_expr(&e.node, name))
            .sum(),
        Expr::If { cond, then, else_ } => {
            let mut count = count_ident_in_expr(&cond.node, name);
            for s in then {
                count += count_ident_in_stmt_expr(&s.node, name);
            }
            if let Some(else_branch) = else_ {
                count += count_ident_in_if_else(else_branch, name);
            }
            count
        }
        Expr::Block(stmts) => stmts
            .iter()
            .map(|s| count_ident_in_stmt_expr(&s.node, name))
            .sum(),
        Expr::Ternary { cond, then, else_ } => {
            count_ident_in_expr(&cond.node, name)
                + count_ident_in_expr(&then.node, name)
                + count_ident_in_expr(&else_.node, name)
        }
        Expr::Assign { target, value } => {
            count_ident_in_expr(&target.node, name) + count_ident_in_expr(&value.node, name)
        }
        Expr::AssignOp { target, value, .. } => {
            count_ident_in_expr(&target.node, name) + count_ident_in_expr(&value.node, name)
        }
        Expr::Lambda { body, .. } => count_ident_in_expr(&body.node, name),
        Expr::Loop { iter, body, .. } => {
            let mut count = 0;
            if let Some(iter_expr) = iter {
                count += count_ident_in_expr(&iter_expr.node, name);
            }
            for s in body {
                count += count_ident_in_stmt_expr(&s.node, name);
            }
            count
        }
        Expr::While { condition, body } => {
            let mut count = count_ident_in_expr(&condition.node, name);
            for s in body {
                count += count_ident_in_stmt_expr(&s.node, name);
            }
            count
        }
        Expr::Match { expr, arms } => {
            let mut count = count_ident_in_expr(&expr.node, name);
            for arm in arms {
                count += count_ident_in_expr(&arm.body.node, name);
            }
            count
        }
        Expr::Ref(e)
        | Expr::Deref(e)
        | Expr::Spread(e)
        | Expr::Await(e)
        | Expr::Try(e)
        | Expr::Unwrap(e)
        | Expr::Lazy(e)
        | Expr::Force(e)
        | Expr::Old(e)
        | Expr::Assume(e) => count_ident_in_expr(&e.node, name),
        Expr::Cast { expr, .. } => count_ident_in_expr(&expr.node, name),
        Expr::Range { start, end, .. } => {
            let mut count = 0;
            if let Some(s) = start {
                count += count_ident_in_expr(&s.node, name);
            }
            if let Some(e) = end {
                count += count_ident_in_expr(&e.node, name);
            }
            count
        }
        Expr::StructLit { fields, .. } => fields
            .iter()
            .map(|(_, v)| count_ident_in_expr(&v.node, name))
            .sum(),
        Expr::StringInterp(parts) => parts
            .iter()
            .map(|p| {
                if let StringInterpPart::Expr(e) = p {
                    count_ident_in_expr(&e.node, name)
                } else {
                    0
                }
            })
            .sum(),
        Expr::MapLit(pairs) => pairs
            .iter()
            .map(|(k, v)| count_ident_in_expr(&k.node, name) + count_ident_in_expr(&v.node, name))
            .sum(),
        Expr::Assert {
            condition, message, ..
        } => {
            let mut count = count_ident_in_expr(&condition.node, name);
            if let Some(msg) = message {
                count += count_ident_in_expr(&msg.node, name);
            }
            count
        }
        Expr::Comptime { body } => count_ident_in_expr(&body.node, name),
        _ => 0,
    }
}

fn count_ident_in_stmt_expr(stmt: &Stmt, name: &str) -> usize {
    match stmt {
        Stmt::Let { value, .. } => count_ident_in_expr(&value.node, name),
        Stmt::Expr(expr) => count_ident_in_expr(&expr.node, name),
        Stmt::Return(Some(expr)) | Stmt::Break(Some(expr)) | Stmt::Defer(expr) => {
            count_ident_in_expr(&expr.node, name)
        }
        Stmt::LetDestructure { value, .. } => count_ident_in_expr(&value.node, name),
        _ => 0,
    }
}

fn count_ident_in_if_else(if_else: &IfElse, name: &str) -> usize {
    match if_else {
        IfElse::ElseIf(cond, then, next) => {
            let mut count = count_ident_in_expr(&cond.node, name);
            for s in then {
                count += count_ident_in_stmt_expr(&s.node, name);
            }
            if let Some(next_branch) = next {
                count += count_ident_in_if_else(next_branch, name);
            }
            count
        }
        IfElse::Else(stmts) => stmts
            .iter()
            .map(|s| count_ident_in_stmt_expr(&s.node, name))
            .sum(),
    }
}

/// Collect all used identifier names in a statement (for quick lookup)
fn collect_used_idents_in_stmt(stmt: &Stmt, used: &mut HashSet<String>) {
    match stmt {
        Stmt::Let { value, .. } => collect_used_idents_in_expr(&value.node, used),
        Stmt::Expr(expr) => collect_used_idents_in_expr(&expr.node, used),
        Stmt::Return(Some(expr)) | Stmt::Break(Some(expr)) | Stmt::Defer(expr) => {
            collect_used_idents_in_expr(&expr.node, used)
        }
        Stmt::LetDestructure { value, .. } => collect_used_idents_in_expr(&value.node, used),
        _ => {}
    }
}

fn collect_used_idents_in_expr(expr: &Expr, used: &mut HashSet<String>) {
    match expr {
        Expr::Ident(name) => {
            used.insert(name.clone());
        }
        Expr::Binary { left, right, .. } => {
            collect_used_idents_in_expr(&left.node, used);
            collect_used_idents_in_expr(&right.node, used);
        }
        Expr::Unary { expr, .. } => collect_used_idents_in_expr(&expr.node, used),
        Expr::Call { func, args } => {
            collect_used_idents_in_expr(&func.node, used);
            for arg in args {
                collect_used_idents_in_expr(&arg.node, used);
            }
        }
        Expr::MethodCall { receiver, args, .. } => {
            collect_used_idents_in_expr(&receiver.node, used);
            for arg in args {
                collect_used_idents_in_expr(&arg.node, used);
            }
        }
        Expr::StaticMethodCall { args, .. } => {
            for arg in args {
                collect_used_idents_in_expr(&arg.node, used);
            }
        }
        Expr::Field { expr, .. } => collect_used_idents_in_expr(&expr.node, used),
        Expr::Index { expr, index } => {
            collect_used_idents_in_expr(&expr.node, used);
            collect_used_idents_in_expr(&index.node, used);
        }
        Expr::Block(stmts) => {
            for s in stmts {
                collect_used_idents_in_stmt(&s.node, used);
            }
        }
        Expr::If { cond, then, else_ } => {
            collect_used_idents_in_expr(&cond.node, used);
            for s in then {
                collect_used_idents_in_stmt(&s.node, used);
            }
            if let Some(else_branch) = else_ {
                collect_used_idents_in_if_else_ids(else_branch, used);
            }
        }
        Expr::Ternary { cond, then, else_ } => {
            collect_used_idents_in_expr(&cond.node, used);
            collect_used_idents_in_expr(&then.node, used);
            collect_used_idents_in_expr(&else_.node, used);
        }
        Expr::Loop { iter, body, .. } => {
            if let Some(iter_expr) = iter {
                collect_used_idents_in_expr(&iter_expr.node, used);
            }
            for s in body {
                collect_used_idents_in_stmt(&s.node, used);
            }
        }
        Expr::While { condition, body } => {
            collect_used_idents_in_expr(&condition.node, used);
            for s in body {
                collect_used_idents_in_stmt(&s.node, used);
            }
        }
        Expr::Match { expr, arms } => {
            collect_used_idents_in_expr(&expr.node, used);
            for arm in arms {
                collect_used_idents_in_expr(&arm.body.node, used);
            }
        }
        Expr::Array(elems) | Expr::Tuple(elems) => {
            for e in elems {
                collect_used_idents_in_expr(&e.node, used);
            }
        }
        Expr::Assign { target, value } | Expr::AssignOp { target, value, .. } => {
            collect_used_idents_in_expr(&target.node, used);
            collect_used_idents_in_expr(&value.node, used);
        }
        Expr::Lambda { body, .. } => collect_used_idents_in_expr(&body.node, used),
        Expr::Ref(e)
        | Expr::Deref(e)
        | Expr::Spread(e)
        | Expr::Await(e)
        | Expr::Try(e)
        | Expr::Unwrap(e)
        | Expr::Lazy(e)
        | Expr::Force(e)
        | Expr::Old(e)
        | Expr::Assume(e) => collect_used_idents_in_expr(&e.node, used),
        Expr::Cast { expr, .. } => collect_used_idents_in_expr(&expr.node, used),
        Expr::Range { start, end, .. } => {
            if let Some(s) = start {
                collect_used_idents_in_expr(&s.node, used);
            }
            if let Some(e) = end {
                collect_used_idents_in_expr(&e.node, used);
            }
        }
        Expr::StructLit { fields, .. } => {
            for (_, v) in fields {
                collect_used_idents_in_expr(&v.node, used);
            }
        }
        Expr::StringInterp(parts) => {
            for p in parts {
                if let StringInterpPart::Expr(e) = p {
                    collect_used_idents_in_expr(&e.node, used);
                }
            }
        }
        Expr::MapLit(pairs) => {
            for (k, v) in pairs {
                collect_used_idents_in_expr(&k.node, used);
                collect_used_idents_in_expr(&v.node, used);
            }
        }
        Expr::Assert {
            condition, message, ..
        } => {
            collect_used_idents_in_expr(&condition.node, used);
            if let Some(msg) = message {
                collect_used_idents_in_expr(&msg.node, used);
            }
        }
        Expr::Comptime { body } => collect_used_idents_in_expr(&body.node, used),
        _ => {}
    }
}

fn collect_used_idents_in_if_else_ids(if_else: &IfElse, used: &mut HashSet<String>) {
    match if_else {
        IfElse::ElseIf(cond, then, next) => {
            collect_used_idents_in_expr(&cond.node, used);
            for s in then {
                collect_used_idents_in_stmt(&s.node, used);
            }
            if let Some(next_branch) = next {
                collect_used_idents_in_if_else_ids(next_branch, used);
            }
        }
        IfElse::Else(stmts) => {
            for s in stmts {
                collect_used_idents_in_stmt(&s.node, used);
            }
        }
    }
}

// ==================== Unused Import Detection ====================

/// Information about an unused import
#[derive(Debug)]
struct UnusedImportInfo {
    name: String,
    span: Span,
}

/// Find unused imports in the AST.
///
/// An import is "unused" if none of its imported names appear as
/// identifiers in the rest of the module.
fn find_unused_imports(module: &Module) -> Vec<UnusedImportInfo> {
    // Collect all identifiers used in the module (excluding Use items themselves)
    let mut all_used_idents: HashSet<String> = HashSet::new();

    for item in &module.items {
        match &item.node {
            Item::Use(_) => {} // Skip imports themselves
            Item::Function(func) => collect_used_idents_in_function(func, &mut all_used_idents),
            Item::Impl(impl_block) => {
                for method in &impl_block.methods {
                    collect_used_idents_in_function(&method.node, &mut all_used_idents);
                }
            }
            Item::Struct(s) => {
                // Struct field types might reference imported types
                for field in &s.fields {
                    collect_type_idents(&field.ty.node, &mut all_used_idents);
                }
            }
            Item::Const(c) => {
                collect_used_idents_in_expr(&c.value.node, &mut all_used_idents);
            }
            Item::Global(g) => {
                collect_used_idents_in_expr(&g.value.node, &mut all_used_idents);
            }
            _ => {}
        }
    }

    // Check each import
    let mut unused = Vec::new();

    for item in &module.items {
        if let Item::Use(use_stmt) = &item.node {
            // Get the imported names
            let imported_names = get_import_names(use_stmt);

            // An import is unused if none of its imported names are used
            let is_used = imported_names.iter().any(|n| all_used_idents.contains(n));

            if !is_used && !imported_names.is_empty() {
                let display_name = imported_names.join(", ");
                unused.push(UnusedImportInfo {
                    name: display_name,
                    span: item.span,
                });
            }
        }
    }

    unused
}

/// Get the names that an import brings into scope
fn get_import_names(use_stmt: &Use) -> Vec<String> {
    if let Some(items) = &use_stmt.items {
        // Selective import: `U mod.{A, B}`
        items.iter().map(|i| i.node.clone()).collect()
    } else if let Some(alias) = &use_stmt.alias {
        // Aliased import: `U mod as alias`
        vec![alias.node.clone()]
    } else {
        // Module import: `U mod.Item` -- the last path component
        use_stmt
            .path
            .last()
            .map(|p| vec![p.node.clone()])
            .unwrap_or_default()
    }
}

fn collect_used_idents_in_function(func: &Function, used: &mut HashSet<String>) {
    // Collect from parameter types
    for param in &func.params {
        collect_type_idents(&param.ty.node, used);
    }
    // Collect from return type
    if let Some(ret) = &func.ret_type {
        collect_type_idents(&ret.node, used);
    }
    // Collect from body
    match &func.body {
        FunctionBody::Expr(expr) => collect_used_idents_in_expr(&expr.node, used),
        FunctionBody::Block(stmts) => {
            for s in stmts {
                collect_used_idents_in_stmt(&s.node, used);
            }
        }
    }
}

/// Collect type identifiers (for checking if imported types are used)
fn collect_type_idents(ty: &Type, used: &mut HashSet<String>) {
    match ty {
        Type::Named { name, generics } => {
            used.insert(name.clone());
            for g in generics {
                collect_type_idents(&g.node, used);
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
            collect_type_idents(&inner.node, used);
        }
        Type::ConstArray { element, .. } => {
            collect_type_idents(&element.node, used);
        }
        Type::Map(key, val) => {
            collect_type_idents(&key.node, used);
            collect_type_idents(&val.node, used);
        }
        Type::Tuple(elems) => {
            for elem in elems {
                collect_type_idents(&elem.node, used);
            }
        }
        Type::FnPtr { params, ret, .. } | Type::Fn { params, ret } => {
            for param in params {
                collect_type_idents(&param.node, used);
            }
            collect_type_idents(&ret.node, used);
        }
        Type::RefLifetime { inner, .. } | Type::RefMutLifetime { inner, .. } => {
            collect_type_idents(&inner.node, used);
        }
        Type::DynTrait {
            trait_name,
            generics,
        } => {
            used.insert(trait_name.clone());
            for g in generics {
                collect_type_idents(&g.node, used);
            }
        }
        Type::Associated {
            base,
            trait_name,
            assoc_name,
            generics,
        } => {
            collect_type_idents(&base.node, used);
            if let Some(tn) = trait_name {
                used.insert(tn.clone());
            }
            used.insert(assoc_name.clone());
            for g in generics {
                collect_type_idents(&g.node, used);
            }
        }
        Type::ImplTrait { bounds } => {
            for b in bounds {
                used.insert(b.node.clone());
            }
        }
        Type::Dependent { base, .. } => {
            collect_type_idents(&base.node, used);
        }
        Type::Unit | Type::Infer => {}
    }
}

// ==================== Span Removal ====================

#[derive(Debug)]
struct SpanRemoval {
    start: usize,
    end: usize,
    kind: RemovalKind,
}

#[derive(Debug)]
enum RemovalKind {
    UnusedVariable(String),
    UnusedImport(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_and_find_unused_vars(source: &str) -> Vec<String> {
        let ast = vais_parser::parse(source).expect("parse failed");
        let unused = find_unused_variables(&ast);
        unused.into_iter().map(|v| v.name).collect()
    }

    fn parse_and_find_unused_imports(source: &str) -> Vec<String> {
        let ast = vais_parser::parse(source).expect("parse failed");
        let unused = find_unused_imports(&ast);
        unused.into_iter().map(|v| v.name).collect()
    }

    #[test]
    fn test_unused_var_detection() {
        let source = r#"
F main() -> i64 {
    x := 5
    y := 10
    y
}
"#;
        let unused = parse_and_find_unused_vars(source);
        assert!(unused.contains(&"x".to_string()));
        assert!(!unused.contains(&"y".to_string()));
    }

    #[test]
    fn test_no_unused_vars() {
        let source = r#"
F main() -> i64 {
    x := 5
    x + 1
}
"#;
        let unused = parse_and_find_unused_vars(source);
        assert!(unused.is_empty());
    }

    #[test]
    fn test_underscore_prefix_exempt() {
        let source = r#"
F main() -> i64 {
    _unused := 5
    0
}
"#;
        let unused = parse_and_find_unused_vars(source);
        assert!(unused.is_empty());
    }

    #[test]
    fn test_unused_import_detection() {
        // Use items that import names not used in functions
        let source = r#"
U std.io
F main() -> i64 {
    0
}
"#;
        let unused = parse_and_find_unused_imports(source);
        assert!(unused.contains(&"io".to_string()));
    }

    #[test]
    fn test_removal_kind_display() {
        let removal = SpanRemoval {
            start: 0,
            end: 10,
            kind: RemovalKind::UnusedVariable("x".to_string()),
        };
        assert!(matches!(removal.kind, RemovalKind::UnusedVariable(_)));
    }

    #[test]
    fn test_fix_result_default() {
        let result = FixResult::default();
        assert_eq!(result.unused_vars_removed, 0);
        assert_eq!(result.unused_imports_removed, 0);
        assert!(!result.modified);
    }

    #[test]
    fn test_get_import_names_path() {
        let use_stmt = Use {
            path: vec![
                Spanned {
                    node: "std".to_string(),
                    span: Span { start: 0, end: 3 },
                },
                Spanned {
                    node: "io".to_string(),
                    span: Span { start: 4, end: 6 },
                },
            ],
            alias: None,
            items: None,
        };
        let names = get_import_names(&use_stmt);
        assert_eq!(names, vec!["io"]);
    }

    #[test]
    fn test_get_import_names_selective() {
        let use_stmt = Use {
            path: vec![Spanned {
                node: "std".to_string(),
                span: Span { start: 0, end: 3 },
            }],
            alias: None,
            items: Some(vec![
                Spanned {
                    node: "File".to_string(),
                    span: Span { start: 5, end: 9 },
                },
                Spanned {
                    node: "Dir".to_string(),
                    span: Span { start: 11, end: 14 },
                },
            ]),
        };
        let names = get_import_names(&use_stmt);
        assert_eq!(names, vec!["File", "Dir"]);
    }

    #[test]
    fn test_get_import_names_alias() {
        let use_stmt = Use {
            path: vec![Spanned {
                node: "long_module".to_string(),
                span: Span { start: 0, end: 11 },
            }],
            alias: Some(Spanned {
                node: "lm".to_string(),
                span: Span { start: 15, end: 17 },
            }),
            items: None,
        };
        let names = get_import_names(&use_stmt);
        assert_eq!(names, vec!["lm"]);
    }
}
