//! AST checking for ownership violations

use super::{OwnershipChecker, OwnershipState};
use crate::types::{ResolvedType, TypeError, TypeResult};
use vais_ast::*;

impl OwnershipChecker {
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

        // Check if function returns a reference type
        let returns_ref = f
            .ret_type
            .as_ref()
            .is_some_and(|rt| self.is_ref_ast_type(&rt.node));
        let prev_returns_ref = self.function_returns_ref;
        self.function_returns_ref = returns_ref;

        // Register parameters (at function scope depth, treated as "parameter" scope)
        for param in &f.params {
            let ty = self.ast_type_to_resolved(&param.ty.node);
            self.define_var(&param.name.node, ty, param.is_mut, Some(param.name.span));
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
                // If function returns a reference, check the return expression
                if returns_ref {
                    self.check_return_ref(expr, Some(expr.span))?;
                }
            }
        }

        self.function_returns_ref = prev_returns_ref;
        self.pop_scope();
        Ok(())
    }

    /// Check if an AST type is a reference type
    fn is_ref_ast_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Ref(_) | Type::RefMut(_))
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
                    // Infer type from value expression for ownership purposes
                    self.infer_type_from_expr(value)
                };

                let is_move_ownership = matches!(ownership, Ownership::Move);
                self.define_var(
                    &name.node,
                    var_ty,
                    *is_mut || is_move_ownership,
                    Some(name.span),
                );

                // Track reference sources for dangling pointer detection
                if let Expr::Ref(inner) = &value.node {
                    if let Expr::Ident(source_name) = &inner.node {
                        self.register_reference(&name.node, source_name, false);
                    }
                }

                Ok(())
            }
            Stmt::LetDestructure { value, .. } => {
                self.check_expr_ownership(value)?;
                self.check_move_from_expr(value)?;
                Ok(())
            }
            Stmt::Expr(expr) => self.check_expr_ownership(expr),
            Stmt::Return(Some(expr)) => {
                self.check_expr_ownership(expr)?;
                self.check_move_from_expr(expr)?;
                // Check for returning references to locals
                if self.function_returns_ref {
                    self.check_return_ref(expr, Some(stmt.span))?;
                }
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

            Expr::If { cond, then, else_ } => {
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

            Expr::Loop {
                body,
                pattern,
                iter,
                ..
            } => {
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

            Expr::Match {
                expr: scrutinee,
                arms,
            } => {
                // check_expr_ownership already handles the move via use_var,
                // so we don't need check_move_from_expr here (which would
                // incorrectly report E022 since the value is already marked as moved)
                self.check_expr_ownership(scrutinee)?;
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
                // Field access borrows the struct, it doesn't move it.
                // Only check that the variable hasn't been moved/partially-moved yet.
                if let Expr::Ident(name) = &object.node {
                    if let Some(info) = self.lookup_var(name) {
                        match &info.state {
                            OwnershipState::Moved { moved_at, .. } => {
                                let err = TypeError::UseAfterMove {
                                    var_name: name.to_string(),
                                    moved_at: *moved_at,
                                    use_at: Some(object.span),
                                };
                                return self.report_error(err);
                            }
                            OwnershipState::PartiallyMoved { moved_fields } => {
                                let err = TypeError::UseAfterPartialMove {
                                    var_name: name.to_string(),
                                    moved_fields: moved_fields.iter().cloned().collect(),
                                    use_at: Some(object.span),
                                };
                                return self.report_error(err);
                            }
                            _ => {}
                        }
                    }
                } else {
                    self.check_expr_ownership(object)?;
                }
                Ok(())
            }

            Expr::Index {
                expr: object,
                index,
            } => {
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
            Expr::Int(_)
            | Expr::Float(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::Unit
            | Expr::SelfCall => Ok(()),

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
}
