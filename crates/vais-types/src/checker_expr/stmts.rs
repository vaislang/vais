//! Statement type checking

use vais_ast::*;
use crate::TypeChecker;
use crate::types::{Linearity, ResolvedType, TypeError, TypeResult};

impl TypeChecker {
    /// Check a block of statements
    pub(crate) fn check_block(&mut self, stmts: &[Spanned<Stmt>]) -> TypeResult<ResolvedType> {
        let mut last_type = ResolvedType::Unit;

        for stmt in stmts {
            last_type = self.check_stmt(stmt)?;
        }

        Ok(last_type)
    }

    /// Check a statement
    pub(crate) fn check_stmt(&mut self, stmt: &Spanned<Stmt>) -> TypeResult<ResolvedType> {
        match &stmt.node {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut,
                ownership,
            } => {
                let value_type = self.check_expr(value)?;
                let var_type = if let Some(ty) = ty {
                    let expected = self.resolve_type(&ty.node);
                    self.unify(&expected, &value_type)?;
                    expected
                } else {
                    value_type
                };

                // Convert AST Ownership to type system Linearity
                let linearity = match ownership {
                    Ownership::Linear => Linearity::Linear,
                    Ownership::Affine => Linearity::Affine,
                    Ownership::Move => Linearity::Affine, // Move semantics act like affine
                    Ownership::Regular => {
                        // Check if the type itself is linear/affine
                        match &var_type {
                            ResolvedType::Linear(_) => Linearity::Linear,
                            ResolvedType::Affine(_) => Linearity::Affine,
                            _ => Linearity::Unrestricted,
                        }
                    }
                };

                self.define_var_with_linearity(
                    &name.node,
                    var_type,
                    *is_mut,
                    linearity,
                    Some(name.span),
                );
                Ok(ResolvedType::Unit)
            }
            Stmt::LetDestructure {
                pattern,
                value,
                is_mut,
            } => {
                let value_type = self.check_expr(value)?;
                self.check_destructure_pattern(pattern, &value_type, *is_mut)?;
                Ok(ResolvedType::Unit)
            }
            Stmt::Expr(expr) => self.check_expr(expr),
            Stmt::Return(expr) => {
                let ret_type = if let Some(expr) = expr {
                    self.check_expr(expr)?
                } else {
                    ResolvedType::Unit
                };
                if let Some(expected) = self.current_fn_ret.clone() {
                    // Auto-deref: if returning &T but expected is T, allow implicit deref
                    let ret_type_deref = if let ResolvedType::Ref(inner) = &ret_type {
                        if self.unify(&expected, inner).is_ok() {
                            *inner.clone()
                        } else {
                            ret_type.clone()
                        }
                    } else {
                        ret_type.clone()
                    };
                    self.unify(&expected, &ret_type_deref)?;
                }
                // Return has "Never" type because execution doesn't continue past it
                Ok(ResolvedType::Never)
            }
            // Break and Continue have "Never" type because execution doesn't continue past them
            Stmt::Break(_) | Stmt::Continue => Ok(ResolvedType::Never),

            Stmt::Defer(expr) => {
                // Type check the deferred expression
                // Defer expressions typically should be function calls that return unit
                self.check_expr(expr)?;
                // Defer itself doesn't produce a value in the control flow
                Ok(ResolvedType::Unit)
            }
            Stmt::Error { .. } => {
                // Error nodes from recovery mode are treated as having Unknown type.
                // The parsing error has already been reported.
                Ok(ResolvedType::Unknown)
            }
        }
    }

    /// Check a destructuring pattern against a type and bind variables
    pub(crate) fn check_destructure_pattern(
        &mut self,
        pattern: &Spanned<Pattern>,
        ty: &ResolvedType,
        is_mut: bool,
    ) -> TypeResult<()> {
        match &pattern.node {
            Pattern::Ident(name) => {
                self.define_var_with_linearity(
                    name,
                    ty.clone(),
                    is_mut,
                    Linearity::Unrestricted,
                    Some(pattern.span),
                );
                Ok(())
            }
            Pattern::Tuple(patterns) => {
                if let ResolvedType::Tuple(types) = ty {
                    if patterns.len() != types.len() {
                        return Err(TypeError::Mismatch {
                            expected: format!("tuple of {} elements", patterns.len()),
                            found: format!("tuple of {} elements", types.len()),
                            span: Some(pattern.span),
                        });
                    }
                    for (pat, elem_ty) in patterns.iter().zip(types.iter()) {
                        self.check_destructure_pattern(pat, elem_ty, is_mut)?;
                    }
                    Ok(())
                } else {
                    Err(TypeError::Mismatch {
                        expected: "tuple".to_string(),
                        found: format!("{}", ty),
                        span: Some(pattern.span),
                    })
                }
            }
            Pattern::Wildcard => Ok(()),
            _ => Err(TypeError::Mismatch {
                expected: "identifier or tuple pattern".to_string(),
                found: "unsupported pattern in destructuring".to_string(),
                span: Some(pattern.span),
            }),
        }
    }
}
