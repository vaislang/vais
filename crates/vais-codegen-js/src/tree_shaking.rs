//! Tree shaking (dead code elimination) for Vais JavaScript codegen.
//!
//! This module implements a simple name-based reachability analysis to remove
//! unreferenced functions, structs, enums, and other items from the final output.

use std::collections::{HashMap, HashSet};
use vais_ast::*;

/// Tree shaker for eliminating unreachable code.
///
/// Performs reachability analysis starting from entry points (main function
/// and all public items), then filters the AST to only include reachable items.
#[derive(Debug, Clone)]
pub struct TreeShaker {
    /// Set of reachable item names (functions, structs, enums, etc.)
    reachable: HashSet<String>,
    /// Dependency graph: item_name → set of referenced item_names
    deps: HashMap<String, HashSet<String>>,
}

impl TreeShaker {
    /// Analyze a module and build the dependency graph.
    ///
    /// Walks through all items and collects what other items each one references.
    pub fn analyze(module: &Module) -> Self {
        let mut shaker = TreeShaker {
            reachable: HashSet::new(),
            deps: HashMap::new(),
        };

        for item in &module.items {
            shaker.analyze_item(&item.node);
        }

        shaker
    }

    /// Analyze a single item and record its dependencies.
    fn analyze_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => {
                let name = f.name.node.clone();
                let mut deps = HashSet::new();

                // Analyze function body for references
                match &f.body {
                    FunctionBody::Expr(expr) => {
                        Self::collect_expr_deps(&expr.node, &mut deps);
                    }
                    FunctionBody::Block(stmts) => {
                        for stmt in stmts {
                            Self::collect_stmt_deps(&stmt.node, &mut deps);
                        }
                    }
                }

                // Analyze return type for type references
                if let Some(ret_type) = &f.ret_type {
                    Self::collect_type_deps(&ret_type.node, &mut deps);
                }

                // Analyze parameter types
                for param in &f.params {
                    Self::collect_type_deps(&param.ty.node, &mut deps);
                }

                self.deps.insert(name, deps);
            }
            Item::Struct(s) => {
                let name = s.name.node.clone();
                let mut deps = HashSet::new();

                // Analyze field types
                for field in &s.fields {
                    Self::collect_type_deps(&field.ty.node, &mut deps);
                }

                // Analyze methods
                for method in &s.methods {
                    match &method.node.body {
                        FunctionBody::Expr(expr) => {
                            Self::collect_expr_deps(&expr.node, &mut deps);
                        }
                        FunctionBody::Block(stmts) => {
                            for stmt in stmts {
                                Self::collect_stmt_deps(&stmt.node, &mut deps);
                            }
                        }
                    }

                    if let Some(ret_type) = &method.node.ret_type {
                        Self::collect_type_deps(&ret_type.node, &mut deps);
                    }

                    for param in &method.node.params {
                        Self::collect_type_deps(&param.ty.node, &mut deps);
                    }
                }

                self.deps.insert(name, deps);
            }
            Item::Enum(e) => {
                let name = e.name.node.clone();
                let mut deps = HashSet::new();

                // Analyze variant field types
                for variant in &e.variants {
                    match &variant.fields {
                        VariantFields::Tuple(types) => {
                            for ty in types {
                                Self::collect_type_deps(&ty.node, &mut deps);
                            }
                        }
                        VariantFields::Struct(fields) => {
                            for field in fields {
                                Self::collect_type_deps(&field.ty.node, &mut deps);
                            }
                        }
                        VariantFields::Unit => {}
                    }
                }

                self.deps.insert(name, deps);
            }
            Item::TypeAlias(t) => {
                let name = t.name.node.clone();
                let mut deps = HashSet::new();
                Self::collect_type_deps(&t.ty.node, &mut deps);
                self.deps.insert(name, deps);
            }
            Item::Const(c) => {
                let name = c.name.node.clone();
                let mut deps = HashSet::new();
                Self::collect_type_deps(&c.ty.node, &mut deps);
                Self::collect_expr_deps(&c.value.node, &mut deps);
                self.deps.insert(name, deps);
            }
            Item::Global(g) => {
                let name = g.name.node.clone();
                let mut deps = HashSet::new();
                Self::collect_type_deps(&g.ty.node, &mut deps);
                Self::collect_expr_deps(&g.value.node, &mut deps);
                self.deps.insert(name, deps);
            }
            Item::Trait(t) => {
                let name = t.name.node.clone();
                let mut deps = HashSet::new();

                for method in &t.methods {
                    if let Some(ret_type) = &method.ret_type {
                        Self::collect_type_deps(&ret_type.node, &mut deps);
                    }
                    for param in &method.params {
                        Self::collect_type_deps(&param.ty.node, &mut deps);
                    }
                    if let Some(body) = &method.default_body {
                        match body {
                            FunctionBody::Expr(expr) => {
                                Self::collect_expr_deps(&expr.node, &mut deps);
                            }
                            FunctionBody::Block(stmts) => {
                                for stmt in stmts {
                                    Self::collect_stmt_deps(&stmt.node, &mut deps);
                                }
                            }
                        }
                    }
                }

                self.deps.insert(name, deps);
            }
            Item::Impl(impl_block) => {
                // Impl blocks don't have a single name, but we track their type
                if let Type::Named { name, .. } = &impl_block.target_type.node {
                    let mut deps = HashSet::new();
                    Self::collect_type_deps(&impl_block.target_type.node, &mut deps);

                    if let Some(trait_name) = &impl_block.trait_name {
                        deps.insert(trait_name.node.clone());
                    }

                    for method in &impl_block.methods {
                        match &method.node.body {
                            FunctionBody::Expr(expr) => {
                                Self::collect_expr_deps(&expr.node, &mut deps);
                            }
                            FunctionBody::Block(stmts) => {
                                for stmt in stmts {
                                    Self::collect_stmt_deps(&stmt.node, &mut deps);
                                }
                            }
                        }

                        if let Some(ret_type) = &method.node.ret_type {
                            Self::collect_type_deps(&ret_type.node, &mut deps);
                        }

                        for param in &method.node.params {
                            Self::collect_type_deps(&param.ty.node, &mut deps);
                        }
                    }

                    self.deps.insert(name.clone(), deps);
                }
            }
            // Other items don't produce dependencies we track
            Item::Use(_)
            | Item::ExternBlock(_)
            | Item::Union(_)
            | Item::Macro(_)
            | Item::Error { .. } => {}
        }
    }

    /// Collect all type names referenced in a type expression.
    fn collect_type_deps(ty: &Type, deps: &mut HashSet<String>) {
        match ty {
            Type::Named { name, generics } => {
                // Skip built-in types
                if !Self::is_builtin_type(name) {
                    deps.insert(name.clone());
                }
                for generic in generics {
                    Self::collect_type_deps(&generic.node, deps);
                }
            }
            Type::Tuple(types) => {
                for ty in types {
                    Self::collect_type_deps(&ty.node, deps);
                }
            }
            Type::Array(elem_type)
            | Type::Optional(elem_type)
            | Type::Result(elem_type)
            | Type::Pointer(elem_type)
            | Type::Ref(elem_type)
            | Type::RefMut(elem_type)
            | Type::Slice(elem_type)
            | Type::SliceMut(elem_type)
            | Type::Lazy(elem_type) => {
                Self::collect_type_deps(&elem_type.node, deps);
            }
            Type::ConstArray { element, .. } => {
                Self::collect_type_deps(&element.node, deps);
            }
            Type::Map(key, value) => {
                Self::collect_type_deps(&key.node, deps);
                Self::collect_type_deps(&value.node, deps);
            }
            Type::RefLifetime { inner, .. } | Type::RefMutLifetime { inner, .. } => {
                Self::collect_type_deps(&inner.node, deps);
            }
            Type::Fn { params, ret } | Type::FnPtr { params, ret, .. } => {
                for param in params {
                    Self::collect_type_deps(&param.node, deps);
                }
                Self::collect_type_deps(&ret.node, deps);
            }
            Type::DynTrait {
                trait_name,
                generics,
            } => {
                // Track the trait name
                if !Self::is_builtin_type(trait_name) {
                    deps.insert(trait_name.clone());
                }
                for generic in generics {
                    Self::collect_type_deps(&generic.node, deps);
                }
            }
            Type::Associated {
                base,
                trait_name,
                generics,
                ..
            } => {
                Self::collect_type_deps(&base.node, deps);
                if let Some(trait_n) = trait_name {
                    if !Self::is_builtin_type(trait_n) {
                        deps.insert(trait_n.clone());
                    }
                }
                for generic in generics {
                    Self::collect_type_deps(&generic.node, deps);
                }
            }
            Type::Linear(inner) | Type::Affine(inner) => {
                Self::collect_type_deps(&inner.node, deps);
            }
            Type::Dependent {
                base, predicate, ..
            } => {
                Self::collect_type_deps(&base.node, deps);
                Self::collect_expr_deps(&predicate.node, deps);
            }
            Type::Infer | Type::Unit => {}
        }
    }

    /// Collect all item names referenced in an expression.
    fn collect_expr_deps(expr: &Expr, deps: &mut HashSet<String>) {
        match expr {
            Expr::Ident(name) => {
                // Track all identifiers as potential references
                // This includes functions, constants, and types
                // We can't reliably distinguish local variables from global items
                // at this simple analysis level, so we track all
                deps.insert(name.clone());
            }
            Expr::Call { func, args } => {
                Self::collect_expr_deps(&func.node, deps);
                for arg in args {
                    Self::collect_expr_deps(&arg.node, deps);
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                Self::collect_expr_deps(&receiver.node, deps);
                for arg in args {
                    Self::collect_expr_deps(&arg.node, deps);
                }
            }
            Expr::StaticMethodCall {
                type_name, args, ..
            } => {
                deps.insert(type_name.node.clone());
                for arg in args {
                    Self::collect_expr_deps(&arg.node, deps);
                }
            }
            Expr::StructLit { name, fields } => {
                deps.insert(name.node.clone());
                for (_, value) in fields {
                    Self::collect_expr_deps(&value.node, deps);
                }
            }
            Expr::Binary { left, right, .. } => {
                Self::collect_expr_deps(&left.node, deps);
                Self::collect_expr_deps(&right.node, deps);
            }
            Expr::Unary { expr, .. } => {
                Self::collect_expr_deps(&expr.node, deps);
            }
            Expr::Ternary { cond, then, else_ } => {
                Self::collect_expr_deps(&cond.node, deps);
                Self::collect_expr_deps(&then.node, deps);
                Self::collect_expr_deps(&else_.node, deps);
            }
            Expr::If { cond, then, else_ } => {
                Self::collect_expr_deps(&cond.node, deps);
                for stmt in then {
                    Self::collect_stmt_deps(&stmt.node, deps);
                }
                if let Some(else_block) = else_ {
                    match else_block {
                        IfElse::ElseIf(cond, stmts, next) => {
                            Self::collect_expr_deps(&cond.node, deps);
                            for stmt in stmts {
                                Self::collect_stmt_deps(&stmt.node, deps);
                            }
                            if let Some(next_else) = next {
                                // Recursively handle nested else-if
                                match next_else.as_ref() {
                                    IfElse::ElseIf(c, s, n) => {
                                        Self::collect_expr_deps(&c.node, deps);
                                        for stmt in s {
                                            Self::collect_stmt_deps(&stmt.node, deps);
                                        }
                                        if let Some(nn) = n {
                                            // Continue recursion if needed
                                            if let IfElse::Else(stmts) = nn.as_ref() {
                                                for stmt in stmts {
                                                    Self::collect_stmt_deps(&stmt.node, deps);
                                                }
                                            }
                                        }
                                    }
                                    IfElse::Else(stmts) => {
                                        for stmt in stmts {
                                            Self::collect_stmt_deps(&stmt.node, deps);
                                        }
                                    }
                                }
                            }
                        }
                        IfElse::Else(stmts) => {
                            for stmt in stmts {
                                Self::collect_stmt_deps(&stmt.node, deps);
                            }
                        }
                    }
                }
            }
            Expr::Loop { iter, body, .. } => {
                if let Some(iter) = iter {
                    Self::collect_expr_deps(&iter.node, deps);
                }
                for stmt in body {
                    Self::collect_stmt_deps(&stmt.node, deps);
                }
            }
            Expr::While { condition, body } => {
                Self::collect_expr_deps(&condition.node, deps);
                for stmt in body {
                    Self::collect_stmt_deps(&stmt.node, deps);
                }
            }
            Expr::Match { expr, arms } => {
                Self::collect_expr_deps(&expr.node, deps);
                for arm in arms {
                    Self::collect_pattern_deps(&arm.pattern.node, deps);
                    if let Some(guard) = &arm.guard {
                        Self::collect_expr_deps(&guard.node, deps);
                    }
                    Self::collect_expr_deps(&arm.body.node, deps);
                }
            }
            Expr::Field { expr, .. } => {
                Self::collect_expr_deps(&expr.node, deps);
            }
            Expr::Index { expr, index } => {
                Self::collect_expr_deps(&expr.node, deps);
                Self::collect_expr_deps(&index.node, deps);
            }
            Expr::Array(exprs) | Expr::Tuple(exprs) => {
                for expr in exprs {
                    Self::collect_expr_deps(&expr.node, deps);
                }
            }
            Expr::Range { start, end, .. } => {
                if let Some(start) = start {
                    Self::collect_expr_deps(&start.node, deps);
                }
                if let Some(end) = end {
                    Self::collect_expr_deps(&end.node, deps);
                }
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    Self::collect_stmt_deps(&stmt.node, deps);
                }
            }
            Expr::Await(expr) | Expr::Try(expr) | Expr::Unwrap(expr) => {
                Self::collect_expr_deps(&expr.node, deps);
            }
            Expr::Ref(expr) | Expr::Deref(expr) => {
                Self::collect_expr_deps(&expr.node, deps);
            }
            Expr::Spawn(expr) => {
                Self::collect_expr_deps(&expr.node, deps);
            }
            Expr::Lambda { params, body, .. } => {
                for param in params {
                    Self::collect_type_deps(&param.ty.node, deps);
                }
                Self::collect_expr_deps(&body.node, deps);
            }
            Expr::Cast { expr, ty } => {
                Self::collect_expr_deps(&expr.node, deps);
                Self::collect_type_deps(&ty.node, deps);
            }
            Expr::Yield(expr) => {
                Self::collect_expr_deps(&expr.node, deps);
            }
            Expr::MapLit(pairs) => {
                for (key, value) in pairs {
                    Self::collect_expr_deps(&key.node, deps);
                    Self::collect_expr_deps(&value.node, deps);
                }
            }
            Expr::Spread(expr) => {
                Self::collect_expr_deps(&expr.node, deps);
            }
            Expr::Assign { target, value } | Expr::AssignOp { target, value, .. } => {
                Self::collect_expr_deps(&target.node, deps);
                Self::collect_expr_deps(&value.node, deps);
            }
            Expr::Comptime { body } => {
                Self::collect_expr_deps(&body.node, deps);
            }
            Expr::MacroInvoke(_) => {
                // Macro invocations are expanded during parsing, tokens don't carry dep info
            }
            Expr::Old(expr) | Expr::Assume(expr) | Expr::Lazy(expr) | Expr::Force(expr) => {
                Self::collect_expr_deps(&expr.node, deps);
            }
            Expr::Assert { condition, message } => {
                Self::collect_expr_deps(&condition.node, deps);
                if let Some(msg) = message {
                    Self::collect_expr_deps(&msg.node, deps);
                }
            }
            // Literals don't reference other items
            Expr::Int(_)
            | Expr::Float(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::StringInterp(_)
            | Expr::Unit
            | Expr::SelfCall
            | Expr::Error { .. } => {}
        }
    }

    /// Collect dependencies from a statement.
    fn collect_stmt_deps(stmt: &Stmt, deps: &mut HashSet<String>) {
        match stmt {
            Stmt::Expr(expr) => {
                Self::collect_expr_deps(&expr.node, deps);
            }
            Stmt::Let { ty, value, .. } => {
                if let Some(ty) = ty {
                    Self::collect_type_deps(&ty.node, deps);
                }
                Self::collect_expr_deps(&value.node, deps);
            }
            Stmt::LetDestructure { pattern, value, .. } => {
                Self::collect_pattern_deps(&pattern.node, deps);
                Self::collect_expr_deps(&value.node, deps);
            }
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    Self::collect_expr_deps(&expr.node, deps);
                }
            }
            Stmt::Break(expr) => {
                if let Some(expr) = expr {
                    Self::collect_expr_deps(&expr.node, deps);
                }
            }
            Stmt::Defer(expr) => {
                Self::collect_expr_deps(&expr.node, deps);
            }
            Stmt::Continue | Stmt::Error { .. } => {}
        }
    }

    /// Collect dependencies from a pattern.
    fn collect_pattern_deps(pattern: &Pattern, deps: &mut HashSet<String>) {
        match pattern {
            Pattern::Ident(_) | Pattern::Wildcard => {}
            Pattern::Literal(lit) => {
                Self::collect_literal_deps(lit, deps);
            }
            Pattern::Tuple(patterns) => {
                for pat in patterns {
                    Self::collect_pattern_deps(&pat.node, deps);
                }
            }
            Pattern::Struct { name, fields } => {
                deps.insert(name.node.clone());
                for (_, pat_opt) in fields {
                    if let Some(pat) = pat_opt {
                        Self::collect_pattern_deps(&pat.node, deps);
                    }
                }
            }
            Pattern::Variant { name, fields } => {
                deps.insert(name.node.clone());
                for pat in fields {
                    Self::collect_pattern_deps(&pat.node, deps);
                }
            }
            Pattern::Range { start, end, .. } => {
                if let Some(start) = start {
                    Self::collect_pattern_deps(&start.node, deps);
                }
                if let Some(end) = end {
                    Self::collect_pattern_deps(&end.node, deps);
                }
            }
            Pattern::Or(patterns) => {
                for pat in patterns {
                    Self::collect_pattern_deps(&pat.node, deps);
                }
            }
            Pattern::Alias { pattern, .. } => {
                // For pattern alias, collect deps from inner pattern
                Self::collect_pattern_deps(&pattern.node, deps);
            }
        }
    }

    /// Collect dependencies from a literal (used in patterns).
    fn collect_literal_deps(_lit: &Literal, _deps: &mut HashSet<String>) {
        // Literals don't reference other items
    }

    /// Check if a type name is a built-in type.
    fn is_builtin_type(name: &str) -> bool {
        matches!(
            name,
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
                | "bool"
                | "char"
                | "str"
                | "String"
                | "()"
                | "unit"
        )
    }

    /// Mark items as reachable starting from entry points.
    ///
    /// Entry points are:
    /// - The "main" function (if present)
    /// - All public items
    ///
    /// This performs a flood-fill through the dependency graph.
    pub fn mark_reachable(&mut self, entry_points: &[&str]) {
        let mut queue: Vec<String> = entry_points.iter().map(|s| s.to_string()).collect();

        while let Some(name) = queue.pop() {
            if self.reachable.contains(&name) {
                continue;
            }

            self.reachable.insert(name.clone());

            // Add dependencies to queue
            if let Some(deps) = self.deps.get(&name) {
                for dep in deps {
                    if !self.reachable.contains(dep) {
                        queue.push(dep.clone());
                    }
                }
            }
        }
    }

    /// Check if an item name is reachable.
    pub fn is_reachable(&self, name: &str) -> bool {
        self.reachable.contains(name)
    }

    /// Filter a module to only include reachable items.
    ///
    /// Returns a new module with the same structure but with unreachable
    /// items removed.
    pub fn filter_module(&self, module: &Module) -> Module {
        let mut filtered_items = Vec::new();

        for item in &module.items {
            if self.should_keep_item(&item.node) {
                filtered_items.push(item.clone());
            }
        }

        Module {
            items: filtered_items,
            modules_map: module.modules_map.clone(),
        }
    }

    /// Check if an item should be kept based on reachability.
    fn should_keep_item(&self, item: &Item) -> bool {
        match item {
            Item::Function(f) => self.is_reachable(&f.name.node),
            Item::Struct(s) => self.is_reachable(&s.name.node),
            Item::Enum(e) => self.is_reachable(&e.name.node),
            Item::TypeAlias(t) => self.is_reachable(&t.name.node),
            Item::Const(c) => self.is_reachable(&c.name.node),
            Item::Global(g) => self.is_reachable(&g.name.node),
            Item::Trait(t) => self.is_reachable(&t.name.node),
            Item::Impl(impl_block) => {
                if let Type::Named { name, .. } = &impl_block.target_type.node {
                    self.is_reachable(name)
                } else {
                    true // Keep if we can't determine the name
                }
            }
            // Always keep imports, extern blocks, unions, and macros
            Item::Use(_)
            | Item::ExternBlock(_)
            | Item::Union(_)
            | Item::Macro(_)
            | Item::Error { .. } => true,
        }
    }

    /// Convenience method: analyze and shake a module in one step.
    ///
    /// Uses default entry points (main + all public items).
    pub fn shake(module: &Module) -> Module {
        let mut shaker = Self::analyze(module);

        // Find entry points: main + public items
        let mut entry_points = vec!["main"];

        for item in &module.items {
            match &item.node {
                Item::Function(f) if f.is_pub => {
                    entry_points.push(&f.name.node);
                }
                Item::Struct(s) if s.is_pub => {
                    entry_points.push(&s.name.node);
                }
                Item::Enum(e) if e.is_pub => {
                    entry_points.push(&e.name.node);
                }
                Item::Trait(t) if t.is_pub => {
                    entry_points.push(&t.name.node);
                }
                _ => {}
            }
        }

        shaker.mark_reachable(&entry_points);
        shaker.filter_module(module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_function(name: &str, is_pub: bool, body: Expr) -> Spanned<Item> {
        Spanned::new(
            Item::Function(Function {
                name: Spanned::new(name.to_string(), Span::new(0, name.len())),
                generics: vec![],
                params: vec![],
                ret_type: None,
                body: FunctionBody::Expr(Box::new(Spanned::new(body, Span::new(0, 1)))),
                is_pub,
                is_async: false,
                attributes: vec![],
            where_clause: vec![],
            }),
            Span::new(0, 1),
        )
    }

    fn make_call(func_name: &str) -> Expr {
        Expr::Call {
            func: Box::new(Spanned::new(
                Expr::Ident(func_name.to_string()),
                Span::new(0, func_name.len()),
            )),
            args: vec![],
        }
    }

    #[test]
    fn test_unreferenced_private_function_removed() {
        let module = Module {
            items: vec![
                make_function("main", false, Expr::Int(42)),
                make_function("unused", false, Expr::Int(0)),
            ],
            modules_map: None,
        };

        let shaken = TreeShaker::shake(&module);

        assert_eq!(shaken.items.len(), 1);
        match &shaken.items[0].node {
            Item::Function(f) => assert_eq!(f.name.node, "main"),
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_transitively_referenced_functions_kept() {
        let module = Module {
            items: vec![
                make_function("main", false, make_call("helper1")),
                make_function("helper1", false, make_call("helper2")),
                make_function("helper2", false, Expr::Int(42)),
                make_function("unused", false, Expr::Int(0)),
            ],
            modules_map: None,
        };

        let shaken = TreeShaker::shake(&module);

        assert_eq!(shaken.items.len(), 3);
        let names: Vec<String> = shaken
            .items
            .iter()
            .filter_map(|item| match &item.node {
                Item::Function(f) => Some(f.name.node.clone()),
                _ => None,
            })
            .collect();

        assert!(names.contains(&"main".to_string()));
        assert!(names.contains(&"helper1".to_string()));
        assert!(names.contains(&"helper2".to_string()));
        assert!(!names.contains(&"unused".to_string()));
    }

    #[test]
    fn test_public_functions_always_kept() {
        let module = Module {
            items: vec![
                make_function("main", false, Expr::Int(42)),
                make_function("public_api", true, Expr::Int(100)),
                make_function("unused_private", false, Expr::Int(0)),
            ],
            modules_map: None,
        };

        let shaken = TreeShaker::shake(&module);

        assert_eq!(shaken.items.len(), 2);
        let names: Vec<String> = shaken
            .items
            .iter()
            .filter_map(|item| match &item.node {
                Item::Function(f) => Some(f.name.node.clone()),
                _ => None,
            })
            .collect();

        assert!(names.contains(&"main".to_string()));
        assert!(names.contains(&"public_api".to_string()));
        assert!(!names.contains(&"unused_private".to_string()));
    }

    #[test]
    fn test_struct_lit_marks_struct_reachable() {
        let module = Module {
            items: vec![
                make_function(
                    "main",
                    false,
                    Expr::StructLit {
                        name: Spanned::new("Point".to_string(), Span::new(0, 5)),
                        fields: vec![],
                    },
                ),
                Spanned::new(
                    Item::Struct(Struct {
                        name: Spanned::new("Point".to_string(), Span::new(0, 5)),
                        generics: vec![],
                        fields: vec![],
                        methods: vec![],
                        is_pub: false,
                        attributes: vec![],
            where_clause: vec![],
                    }),
                    Span::new(0, 1),
                ),
                Spanned::new(
                    Item::Struct(Struct {
                        name: Spanned::new("Unused".to_string(), Span::new(0, 6)),
                        generics: vec![],
                        fields: vec![],
                        methods: vec![],
                        is_pub: false,
                        attributes: vec![],
            where_clause: vec![],
                    }),
                    Span::new(0, 1),
                ),
            ],
            modules_map: None,
        };

        let shaken = TreeShaker::shake(&module);

        assert_eq!(shaken.items.len(), 2);
        let has_point = shaken.items.iter().any(|item| match &item.node {
            Item::Struct(s) => s.name.node == "Point",
            _ => false,
        });
        let has_unused = shaken.items.iter().any(|item| match &item.node {
            Item::Struct(s) => s.name.node == "Unused",
            _ => false,
        });

        assert!(has_point);
        assert!(!has_unused);
    }

    #[test]
    fn test_no_main_keeps_public_only() {
        let module = Module {
            items: vec![
                make_function("private1", false, Expr::Int(1)),
                make_function("public1", true, Expr::Int(2)),
                make_function("private2", false, Expr::Int(3)),
            ],
            modules_map: None,
        };

        let shaken = TreeShaker::shake(&module);

        // Only main (not present) and public items are entry points
        // So only public1 should remain
        assert_eq!(shaken.items.len(), 1);
        match &shaken.items[0].node {
            Item::Function(f) => assert_eq!(f.name.node, "public1"),
            _ => panic!("Expected function"),
        }
    }
}
