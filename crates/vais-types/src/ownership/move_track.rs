//! Move semantics tracking

use crate::types::{ResolvedType, TypeError, TypeResult};
use super::{OwnershipChecker, OwnershipState};
use vais_ast::{Expr, Span, Spanned};

impl OwnershipChecker {
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
    pub fn assign_var(
        &mut self,
        name: &str,
        new_ty: ResolvedType,
        assign_at: Option<Span>,
    ) -> TypeResult<()> {
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

    /// Check if an expression causes a move (for non-Copy types)
    pub(super) fn check_move_from_expr(&mut self, expr: &Spanned<Expr>) -> TypeResult<()> {
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
}
