//! Ownership and borrow checker for the Vais type system
//!
//! Implements Rust-style ownership semantics:
//! - Move semantics: values are moved by default for non-Copy types
//! - Borrow checking: at most one mutable reference OR any number of immutable references
//! - Scope-based invalidation: references cannot outlive their referents
//!
//! The checker runs as a second pass after type checking, operating on the typed AST.

use std::collections::{HashMap, HashSet};
use vais_ast::*;
use crate::types::{ResolvedType, TypeError, TypeResult};

/// Tracks the state of a value's ownership
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipState {
    /// Value is owned and valid
    Owned,
    /// Value has been moved to another binding
    Moved {
        moved_to: String,
        moved_at: Option<Span>,
    },
    /// Value has been partially moved (some fields moved)
    PartiallyMoved {
        moved_fields: HashSet<String>,
    },
    /// Value is borrowed (immutably)
    Borrowed {
        borrow_count: usize,
    },
    /// Value is mutably borrowed
    MutBorrowed {
        borrower: String,
    },
}

/// Information about an active borrow
#[derive(Debug, Clone)]
pub struct BorrowInfo {
    /// The variable being borrowed
    pub borrowed_from: String,
    /// Whether this is a mutable borrow
    pub is_mut: bool,
    /// The scope in which the borrow was created
    pub scope_id: u32,
    /// Where the borrow was created
    pub borrow_at: Option<Span>,
}

/// Information about a tracked variable in the ownership system
#[derive(Debug, Clone)]
pub struct OwnershipInfo {
    /// Current ownership state
    pub state: OwnershipState,
    /// The resolved type of the variable
    pub ty: ResolvedType,
    /// Whether the variable is declared mutable
    pub is_mut: bool,
    /// Whether the type is Copy (primitives, references)
    pub is_copy: bool,
    /// The scope where this variable was defined
    pub defined_in_scope: u32,
    /// Where the variable was defined
    pub defined_at: Option<Span>,
}

/// The ownership and borrow checker
pub struct OwnershipChecker {
    /// Stack of scopes, each mapping variable names to ownership info
    scopes: Vec<HashMap<String, OwnershipInfo>>,
    /// Active borrows: borrower variable -> borrow info
    active_borrows: HashMap<String, BorrowInfo>,
    /// Current scope ID counter
    next_scope_id: u32,
    /// Current scope ID
    current_scope: u32,
    /// Collected errors (non-fatal mode)
    errors: Vec<TypeError>,
    /// Whether to collect errors instead of returning immediately
    collect_errors: bool,
}

impl Default for OwnershipChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl OwnershipChecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            active_borrows: HashMap::new(),
            next_scope_id: 1,
            current_scope: 0,
            errors: Vec::new(),
            collect_errors: false,
        }
    }

    /// Create a checker that collects all errors instead of stopping at first
    pub fn new_collecting() -> Self {
        let mut checker = Self::new();
        checker.collect_errors = true;
        checker
    }

    /// Get collected errors
    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }

    /// Take collected errors
    pub fn take_errors(&mut self) -> Vec<TypeError> {
        std::mem::take(&mut self.errors)
    }

    // --- Scope management ---

    fn push_scope(&mut self) {
        let _id = self.next_scope_id;
        self.next_scope_id += 1;
        self.current_scope = _id;
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        // Invalidate borrows from this scope
        let scope_id = self.current_scope;
        self.active_borrows.retain(|_, info| info.scope_id != scope_id);

        // Remove variables from the scope
        self.scopes.pop();
        if self.current_scope > 0 {
            self.current_scope -= 1;
        }
    }

    // --- Variable registration ---

    /// Register a new variable with its ownership info
    pub fn define_var(
        &mut self,
        name: &str,
        ty: ResolvedType,
        is_mut: bool,
        defined_at: Option<Span>,
    ) {
        let is_copy = Self::is_copy_type(&ty);
        let info = OwnershipInfo {
            state: OwnershipState::Owned,
            ty,
            is_mut,
            is_copy,
            defined_in_scope: self.current_scope,
            defined_at,
        };
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), info);
        }
    }

    /// Look up a variable's ownership info
    fn lookup_var(&self, name: &str) -> Option<&OwnershipInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    /// Look up a variable's ownership info mutably
    fn lookup_var_mut(&mut self, name: &str) -> Option<&mut OwnershipInfo> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                return Some(info);
            }
        }
        None
    }

    // --- Copy type determination ---

    /// Determine if a type is Copy (can be implicitly copied rather than moved)
    pub fn is_copy_type(ty: &ResolvedType) -> bool {
        match ty {
            // Primitives are always Copy
            ResolvedType::I8
            | ResolvedType::I16
            | ResolvedType::I32
            | ResolvedType::I64
            | ResolvedType::I128
            | ResolvedType::U8
            | ResolvedType::U16
            | ResolvedType::U32
            | ResolvedType::U64
            | ResolvedType::U128
            | ResolvedType::F32
            | ResolvedType::F64
            | ResolvedType::Bool
            | ResolvedType::Unit
            | ResolvedType::Never => true,

            // References are Copy (the reference itself, not the referent)
            ResolvedType::Ref(_)
            | ResolvedType::RefLifetime { .. } => true,

            // Mutable references are NOT Copy (uniqueness requirement)
            ResolvedType::RefMut(_)
            | ResolvedType::RefMutLifetime { .. } => false,

            // Tuples are Copy if all elements are Copy
            ResolvedType::Tuple(elems) => elems.iter().all(|t| Self::is_copy_type(t)),

            // Const arrays are Copy if element type is Copy
            ResolvedType::ConstArray { element, .. } => Self::is_copy_type(element),

            // Dynamic arrays, strings, maps, and other heap-allocated types are NOT Copy
            ResolvedType::Array(_)
            | ResolvedType::Str
            | ResolvedType::Map(_, _) => false,

            // Named structs/enums: not Copy by default
            // (In a full implementation, we'd check for Copy trait impl)
            ResolvedType::Named { .. } => false,

            // Generic types: conservative - not Copy
            ResolvedType::Generic(_) => false,

            // Function types are Copy
            ResolvedType::Fn { .. } => true,

            // Pointer types are Copy
            ResolvedType::Pointer(_) => true,

            // Optional/Result: Copy if inner is Copy
            ResolvedType::Optional(inner) => Self::is_copy_type(inner),
            ResolvedType::Result(inner) => Self::is_copy_type(inner),

            // Linear/Affine types are explicitly NOT Copy
            ResolvedType::Linear(_) | ResolvedType::Affine(_) => false,

            // Everything else: conservative default
            _ => false,
        }
    }

    // --- Move checking ---

    /// Record a use of a variable (may trigger move)
    pub fn use_var(&mut self, name: &str, use_at: Option<Span>) -> TypeResult<()> {
        let info = match self.lookup_var(name) {
            Some(info) => info.clone(),
            None => return Ok(()), // Variable not tracked (e.g., builtin)
        };

        // Check if already moved
        if let OwnershipState::Moved { moved_at, .. } = &info.state {
            let err = TypeError::UseAfterMove {
                var_name: name.to_string(),
                moved_at: *moved_at,
                use_at,
            };
            return self.report_error(err);
        }

        // Check if partially moved
        if let OwnershipState::PartiallyMoved { moved_fields } = &info.state {
            let err = TypeError::UseAfterPartialMove {
                var_name: name.to_string(),
                moved_fields: moved_fields.iter().cloned().collect(),
                use_at,
            };
            return self.report_error(err);
        }

        // If not Copy, this use moves the value
        if !info.is_copy {
            if let Some(owner_info) = self.lookup_var_mut(name) {
                owner_info.state = OwnershipState::Moved {
                    moved_to: "<used>".to_string(),
                    moved_at: use_at,
                };
            }
        }

        Ok(())
    }

    /// Record an assignment to a variable (resets ownership state)
    pub fn assign_var(&mut self, name: &str, new_ty: ResolvedType, assign_at: Option<Span>) -> TypeResult<()> {
        // Check for active borrows on this variable
        if let Some(borrow) = self.find_active_borrow_of(name) {
            let err = TypeError::AssignWhileBorrowed {
                var_name: name.to_string(),
                borrow_at: borrow.borrow_at,
                assign_at,
                is_mut_borrow: borrow.is_mut,
            };
            return self.report_error(err);
        }

        if let Some(owner_info) = self.lookup_var_mut(name) {
            let is_copy = Self::is_copy_type(&new_ty);
            owner_info.state = OwnershipState::Owned;
            owner_info.ty = new_ty;
            owner_info.is_copy = is_copy;
        }

        Ok(())
    }

    // --- Borrow checking ---

    /// Record an immutable borrow of a variable
    pub fn borrow_var(&mut self, borrower: &str, borrowed_from: &str, borrow_at: Option<Span>) -> TypeResult<()> {
        let info = match self.lookup_var(borrowed_from) {
            Some(info) => info.clone(),
            None => return Ok(()),
        };

        // Cannot borrow a moved value
        if let OwnershipState::Moved { moved_at, .. } = &info.state {
            let err = TypeError::BorrowAfterMove {
                var_name: borrowed_from.to_string(),
                moved_at: *moved_at,
                borrow_at,
            };
            return self.report_error(err);
        }

        // Cannot immutably borrow while mutably borrowed
        if let Some(existing) = self.find_active_mut_borrow_of(borrowed_from) {
            let err = TypeError::BorrowConflict {
                var_name: borrowed_from.to_string(),
                existing_borrow_at: existing.borrow_at,
                new_borrow_at: borrow_at,
                existing_is_mut: true,
                new_is_mut: false,
            };
            return self.report_error(err);
        }

        // Record the borrow
        self.active_borrows.insert(
            borrower.to_string(),
            BorrowInfo {
                borrowed_from: borrowed_from.to_string(),
                is_mut: false,
                scope_id: self.current_scope,
                borrow_at,
            },
        );

        Ok(())
    }

    /// Record a mutable borrow of a variable
    pub fn borrow_var_mut(&mut self, borrower: &str, borrowed_from: &str, borrow_at: Option<Span>) -> TypeResult<()> {
        let info = match self.lookup_var(borrowed_from) {
            Some(info) => info.clone(),
            None => return Ok(()),
        };

        // Cannot borrow a moved value
        if let OwnershipState::Moved { moved_at, .. } = &info.state {
            let err = TypeError::BorrowAfterMove {
                var_name: borrowed_from.to_string(),
                moved_at: *moved_at,
                borrow_at,
            };
            return self.report_error(err);
        }

        // Check the source variable is mutable
        if !info.is_mut {
            let err = TypeError::MutBorrowOfImmutable {
                var_name: borrowed_from.to_string(),
                borrow_at,
            };
            return self.report_error(err);
        }

        // Cannot mutably borrow while any other borrow is active
        if let Some(existing) = self.find_any_active_borrow_of(borrowed_from) {
            let err = TypeError::BorrowConflict {
                var_name: borrowed_from.to_string(),
                existing_borrow_at: existing.borrow_at,
                new_borrow_at: borrow_at,
                existing_is_mut: existing.is_mut,
                new_is_mut: true,
            };
            return self.report_error(err);
        }

        // Record the mutable borrow
        self.active_borrows.insert(
            borrower.to_string(),
            BorrowInfo {
                borrowed_from: borrowed_from.to_string(),
                is_mut: true,
                scope_id: self.current_scope,
                borrow_at,
            },
        );

        Ok(())
    }

    /// Release a borrow (when the borrower goes out of scope or is reassigned)
    pub fn release_borrow(&mut self, borrower: &str) {
        self.active_borrows.remove(borrower);
    }

    // --- Borrow query helpers ---

    fn find_active_borrow_of(&self, var_name: &str) -> Option<&BorrowInfo> {
        self.active_borrows.values().find(|b| b.borrowed_from == var_name)
    }

    fn find_active_mut_borrow_of(&self, var_name: &str) -> Option<&BorrowInfo> {
        self.active_borrows.values().find(|b| b.borrowed_from == var_name && b.is_mut)
    }

    fn find_any_active_borrow_of(&self, var_name: &str) -> Option<&BorrowInfo> {
        self.active_borrows.values().find(|b| b.borrowed_from == var_name)
    }

    // --- Error reporting ---

    fn report_error(&mut self, err: TypeError) -> TypeResult<()> {
        if self.collect_errors {
            self.errors.push(err);
            Ok(())
        } else {
            Err(err)
        }
    }

    // --- AST checking ---

    /// Check a module for ownership violations
    pub fn check_module(&mut self, module: &Module) -> TypeResult<()> {
        for item in &module.items {
            match &item.node {
                Item::Function(f) => self.check_function(f)?,
                Item::Impl(impl_block) => {
                    for method in &impl_block.methods {
                        self.check_function(&method.node)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Check a function body for ownership violations
    fn check_function(&mut self, f: &Function) -> TypeResult<()> {
        self.push_scope();

        // Register parameters
        for param in &f.params {
            let ty = self.ast_type_to_resolved(&param.ty.node);
            self.define_var(
                &param.name.node,
                ty,
                param.is_mut,
                Some(param.name.span),
            );
        }

        // Check body
        match &f.body {
            FunctionBody::Block(stmts) => {
                for stmt in stmts {
                    self.check_stmt(stmt)?;
                }
            }
            FunctionBody::Expr(expr) => {
                self.check_expr_ownership(expr)?;
            }
        }

        self.pop_scope();
        Ok(())
    }

    /// Check a statement for ownership violations
    fn check_stmt(&mut self, stmt: &Spanned<Stmt>) -> TypeResult<()> {
        match &stmt.node {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut,
                ownership,
            } => {
                // Check the value expression
                self.check_expr_ownership(value)?;

                // Determine if this is a move or copy from the value
                self.check_move_from_expr(value)?;

                // Register the new variable
                let var_ty = if let Some(ty) = ty {
                    self.ast_type_to_resolved(&ty.node)
                } else {
                    // Use a placeholder - actual type was resolved by type checker
                    ResolvedType::Unknown
                };

                let is_move_ownership = matches!(ownership, Ownership::Move);
                self.define_var(
                    &name.node,
                    var_ty,
                    *is_mut || is_move_ownership,
                    Some(name.span),
                );

                Ok(())
            }
            Stmt::Expr(expr) => self.check_expr_ownership(expr),
            Stmt::Return(Some(expr)) => {
                self.check_expr_ownership(expr)?;
                self.check_move_from_expr(expr)?;
                Ok(())
            }
            Stmt::Return(None) => Ok(()),
            Stmt::Break(_) | Stmt::Continue => Ok(()),
            Stmt::Defer(expr) => self.check_expr_ownership(expr),
            Stmt::Error { .. } => Ok(()),
        }
    }

    /// Check an expression for ownership violations
    fn check_expr_ownership(&mut self, expr: &Spanned<Expr>) -> TypeResult<()> {
        match &expr.node {
            Expr::Ident(name) => {
                self.use_var(name, Some(expr.span))?;
                Ok(())
            }

            Expr::Binary { left, right, .. } => {
                self.check_expr_ownership(left)?;
                self.check_expr_ownership(right)?;
                Ok(())
            }

            Expr::Unary { expr: inner, .. } => {
                self.check_expr_ownership(inner)?;
                Ok(())
            }

            Expr::Call { func, args } => {
                self.check_expr_ownership(func)?;
                for arg in args {
                    self.check_expr_ownership(arg)?;
                    // Function arguments move non-Copy values
                    self.check_move_from_expr(arg)?;
                }
                Ok(())
            }

            Expr::MethodCall { receiver, args, .. } => {
                self.check_expr_ownership(receiver)?;
                for arg in args {
                    self.check_expr_ownership(arg)?;
                    self.check_move_from_expr(arg)?;
                }
                Ok(())
            }

            Expr::Ref(inner) => {
                // Immutable borrow
                if let Expr::Ident(name) = &inner.node {
                    let borrower = format!("__ref_{}", name);
                    self.borrow_var(&borrower, name, Some(expr.span))?;
                }
                self.check_expr_ownership(inner)?;
                Ok(())
            }

            Expr::Deref(inner) => {
                self.check_expr_ownership(inner)?;
                Ok(())
            }

            Expr::Assign { target, value } => {
                self.check_expr_ownership(value)?;
                self.check_move_from_expr(value)?;

                if let Expr::Ident(name) = &target.node {
                    // Check for active borrows before assigning
                    if let Some(borrow) = self.find_active_borrow_of(name).cloned() {
                        let err = TypeError::AssignWhileBorrowed {
                            var_name: name.clone(),
                            borrow_at: borrow.borrow_at,
                            assign_at: Some(expr.span),
                            is_mut_borrow: borrow.is_mut,
                        };
                        self.report_error(err)?;
                    }
                }

                Ok(())
            }

            Expr::AssignOp { target, value, .. } => {
                self.check_expr_ownership(value)?;
                self.check_expr_ownership(target)?;
                Ok(())
            }

            Expr::If {
                cond,
                then,
                else_,
            } => {
                self.check_expr_ownership(cond)?;
                self.push_scope();
                for stmt in then {
                    self.check_stmt(stmt)?;
                }
                self.pop_scope();
                if let Some(else_branch) = else_ {
                    self.check_if_else(else_branch)?;
                }
                Ok(())
            }

            Expr::Block(stmts) => {
                self.push_scope();
                for stmt in stmts {
                    self.check_stmt(stmt)?;
                }
                self.pop_scope();
                Ok(())
            }

            Expr::Loop { body, pattern, iter, .. } => {
                if let Some(iter_expr) = iter {
                    self.check_expr_ownership(iter_expr)?;
                }
                self.push_scope();
                if let Some(pat) = pattern {
                    // Register pattern variable
                    if let Pattern::Ident(name) = &pat.node {
                        self.define_var(name, ResolvedType::Unknown, false, Some(pat.span));
                    }
                }
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                self.pop_scope();
                Ok(())
            }

            Expr::While { condition, body } => {
                self.check_expr_ownership(condition)?;
                self.push_scope();
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                self.pop_scope();
                Ok(())
            }

            Expr::Match { expr: scrutinee, arms } => {
                self.check_expr_ownership(scrutinee)?;
                self.check_move_from_expr(scrutinee)?;
                for arm in arms {
                    self.push_scope();
                    self.check_expr_ownership(&arm.body)?;
                    self.pop_scope();
                }
                Ok(())
            }

            Expr::Lambda { body, params, .. } => {
                self.push_scope();
                for p in params {
                    let ty = self.ast_type_to_resolved(&p.ty.node);
                    self.define_var(&p.name.node, ty, p.is_mut, Some(p.name.span));
                }
                self.check_expr_ownership(body)?;
                self.pop_scope();
                Ok(())
            }

            Expr::Tuple(elems) => {
                for e in elems {
                    self.check_expr_ownership(e)?;
                }
                Ok(())
            }

            Expr::Array(elems) => {
                for e in elems {
                    self.check_expr_ownership(e)?;
                }
                Ok(())
            }

            Expr::StructLit { fields, .. } => {
                for (_, e) in fields {
                    self.check_expr_ownership(e)?;
                    self.check_move_from_expr(e)?;
                }
                Ok(())
            }

            Expr::Field { expr: object, .. } => {
                self.check_expr_ownership(object)?;
                Ok(())
            }

            Expr::Index { expr: object, index } => {
                self.check_expr_ownership(object)?;
                self.check_expr_ownership(index)?;
                Ok(())
            }

            Expr::Spawn(inner) | Expr::Await(inner) | Expr::Try(inner) | Expr::Unwrap(inner) => {
                self.check_expr_ownership(inner)?;
                Ok(())
            }

            Expr::Cast { expr: inner, .. } => {
                self.check_expr_ownership(inner)?;
                Ok(())
            }

            // Literals and other simple expressions don't have ownership concerns
            Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) | Expr::String(_)
            | Expr::Unit | Expr::SelfCall => Ok(()),

            // Catch-all for other expression types
            _ => Ok(()),
        }
    }

    /// Check an if-else branch for ownership violations
    fn check_if_else(&mut self, if_else: &IfElse) -> TypeResult<()> {
        match if_else {
            IfElse::ElseIf(cond, stmts, else_branch) => {
                self.check_expr_ownership(cond)?;
                self.push_scope();
                for stmt in stmts {
                    self.check_stmt(stmt)?;
                }
                self.pop_scope();
                if let Some(else_b) = else_branch {
                    self.check_if_else(else_b)?;
                }
                Ok(())
            }
            IfElse::Else(stmts) => {
                self.push_scope();
                for stmt in stmts {
                    self.check_stmt(stmt)?;
                }
                self.pop_scope();
                Ok(())
            }
        }
    }

    /// Check if an expression causes a move (for non-Copy types)
    fn check_move_from_expr(&mut self, expr: &Spanned<Expr>) -> TypeResult<()> {
        if let Expr::Ident(name) = &expr.node {
            // This variable is being used as a value (e.g., passed to function, assigned)
            // For non-Copy types, this is a move
            if let Some(info) = self.lookup_var(name) {
                if !info.is_copy {
                    match &info.state {
                        OwnershipState::Moved { moved_at, .. } => {
                            let err = TypeError::UseAfterMove {
                                var_name: name.clone(),
                                moved_at: *moved_at,
                                use_at: Some(expr.span),
                            };
                            return self.report_error(err);
                        }
                        OwnershipState::Owned => {
                            // Mark as moved
                            if let Some(owner_info) = self.lookup_var_mut(name) {
                                owner_info.state = OwnershipState::Moved {
                                    moved_to: "<consumed>".to_string(),
                                    moved_at: Some(expr.span),
                                };
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    // --- Type conversion helper ---

    /// Convert AST type to a simplified ResolvedType for ownership tracking
    fn ast_type_to_resolved(&self, ty: &Type) -> ResolvedType {
        match ty {
            Type::Named { name, .. } => match name.as_str() {
                "i8" => ResolvedType::I8,
                "i16" => ResolvedType::I16,
                "i32" => ResolvedType::I32,
                "i64" | "int" => ResolvedType::I64,
                "i128" => ResolvedType::I128,
                "u8" => ResolvedType::U8,
                "u16" => ResolvedType::U16,
                "u32" => ResolvedType::U32,
                "u64" => ResolvedType::U64,
                "u128" => ResolvedType::U128,
                "f32" => ResolvedType::F32,
                "f64" | "float" => ResolvedType::F64,
                "bool" => ResolvedType::Bool,
                "str" | "String" => ResolvedType::Str,
                _ => ResolvedType::Named { name: name.clone(), generics: vec![] },
            },
            Type::Ref(inner) => ResolvedType::Ref(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::RefMut(inner) => ResolvedType::RefMut(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::Array(inner) => ResolvedType::Array(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::Tuple(elems) => ResolvedType::Tuple(
                elems.iter().map(|e| self.ast_type_to_resolved(&e.node)).collect(),
            ),
            Type::Unit => ResolvedType::Unit,
            Type::Infer => ResolvedType::Unknown,
            _ => ResolvedType::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_span() -> Span {
        Span { start: 0, end: 0 }
    }

    #[test]
    fn test_copy_types_are_not_moved() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::I64, false, Some(make_span()));

        // Using a Copy type multiple times should be fine
        assert!(checker.use_var("x", Some(make_span())).is_ok());
        assert!(checker.use_var("x", Some(make_span())).is_ok());
        assert!(checker.use_var("x", Some(make_span())).is_ok());
    }

    #[test]
    fn test_non_copy_type_moved_on_use() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("s", ResolvedType::Str, false, Some(make_span()));

        // First use moves the string
        assert!(checker.use_var("s", Some(make_span())).is_ok());
        // Second use should fail - value was moved
        assert!(checker.use_var("s", Some(make_span())).is_err());
    }

    #[test]
    fn test_reassign_after_move_is_ok() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("s", ResolvedType::Str, true, Some(make_span()));

        // Move the value
        assert!(checker.use_var("s", Some(make_span())).is_ok());
        // Reassign restores ownership
        assert!(checker.assign_var("s", ResolvedType::Str, Some(make_span())).is_ok());
        // Now we can use it again
        assert!(checker.use_var("s", Some(make_span())).is_ok());
    }

    #[test]
    fn test_immutable_borrow_allows_multiple() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::Named { name: "Vec".to_string(), generics: vec![] }, false, Some(make_span()));

        // Multiple immutable borrows are fine
        assert!(checker.borrow_var("r1", "x", Some(make_span())).is_ok());
        assert!(checker.borrow_var("r2", "x", Some(make_span())).is_ok());
    }

    #[test]
    fn test_mutable_borrow_exclusive() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::Named { name: "Vec".to_string(), generics: vec![] }, true, Some(make_span()));

        // First mutable borrow is fine
        assert!(checker.borrow_var_mut("r1", "x", Some(make_span())).is_ok());
        // Second borrow (even immutable) conflicts
        assert!(checker.borrow_var("r2", "x", Some(make_span())).is_err());
    }

    #[test]
    fn test_mutable_borrow_after_release() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::Named { name: "Vec".to_string(), generics: vec![] }, true, Some(make_span()));

        // Borrow and release
        assert!(checker.borrow_var_mut("r1", "x", Some(make_span())).is_ok());
        checker.release_borrow("r1");

        // Now we can borrow again
        assert!(checker.borrow_var_mut("r2", "x", Some(make_span())).is_ok());
    }

    #[test]
    fn test_cannot_mut_borrow_immutable_var() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::Named { name: "Vec".to_string(), generics: vec![] }, false, Some(make_span()));

        // Cannot mutably borrow an immutable variable
        assert!(checker.borrow_var_mut("r1", "x", Some(make_span())).is_err());
    }

    #[test]
    fn test_cannot_borrow_moved_value() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("s", ResolvedType::Str, false, Some(make_span()));

        // Move the value
        assert!(checker.use_var("s", Some(make_span())).is_ok());
        // Cannot borrow a moved value
        assert!(checker.borrow_var("r1", "s", Some(make_span())).is_err());
    }

    #[test]
    fn test_assign_while_borrowed_fails() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::Named { name: "Vec".to_string(), generics: vec![] }, true, Some(make_span()));

        // Borrow x
        assert!(checker.borrow_var("r1", "x", Some(make_span())).is_ok());
        // Cannot assign while borrowed
        assert!(checker.assign_var("x", ResolvedType::Named { name: "Vec".to_string(), generics: vec![] }, Some(make_span())).is_err());
    }

    #[test]
    fn test_scope_releases_borrows() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::Named { name: "Vec".to_string(), generics: vec![] }, true, Some(make_span()));

        // Borrow in inner scope
        checker.push_scope();
        assert!(checker.borrow_var_mut("r1", "x", Some(make_span())).is_ok());
        checker.pop_scope(); // Borrow released

        // Now we can borrow again
        assert!(checker.borrow_var_mut("r2", "x", Some(make_span())).is_ok());
    }

    #[test]
    fn test_is_copy_type() {
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::I64));
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::Bool));
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::F64));
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::Unit));
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::Ref(Box::new(ResolvedType::I64))));

        assert!(!OwnershipChecker::is_copy_type(&ResolvedType::Str));
        assert!(!OwnershipChecker::is_copy_type(&ResolvedType::Array(Box::new(ResolvedType::I64))));
        assert!(!OwnershipChecker::is_copy_type(&ResolvedType::Named { name: "Vec".to_string(), generics: vec![] }));
        assert!(!OwnershipChecker::is_copy_type(&ResolvedType::RefMut(Box::new(ResolvedType::I64))));
    }

    #[test]
    fn test_collecting_mode() {
        let mut checker = OwnershipChecker::new_collecting();
        checker.define_var("s1", ResolvedType::Str, false, Some(make_span()));
        checker.define_var("s2", ResolvedType::Str, false, Some(make_span()));

        // Move s1
        assert!(checker.use_var("s1", Some(make_span())).is_ok());
        // Use after move - error collected but doesn't fail
        assert!(checker.use_var("s1", Some(make_span())).is_ok());

        // Move s2
        assert!(checker.use_var("s2", Some(make_span())).is_ok());
        // Use after move - another error collected
        assert!(checker.use_var("s2", Some(make_span())).is_ok());

        // Should have collected 2 errors
        assert_eq!(checker.errors().len(), 2);
    }
}
