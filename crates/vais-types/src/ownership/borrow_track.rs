//! Borrow checking and tracking

use crate::types::{TypeError, TypeResult};
use super::{BorrowInfo, OwnershipChecker, OwnershipState};
use vais_ast::Span;

impl OwnershipChecker {
    /// Record an immutable borrow of a variable
    pub fn borrow_var(
        &mut self,
        borrower: &str,
        borrowed_from: &str,
        borrow_at: Option<Span>,
    ) -> TypeResult<()> {
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
    pub fn borrow_var_mut(
        &mut self,
        borrower: &str,
        borrowed_from: &str,
        borrow_at: Option<Span>,
    ) -> TypeResult<()> {
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

    pub(super) fn find_active_borrow_of(&self, var_name: &str) -> Option<&BorrowInfo> {
        self.active_borrows
            .values()
            .find(|b| b.borrowed_from == var_name)
    }

    fn find_active_mut_borrow_of(&self, var_name: &str) -> Option<&BorrowInfo> {
        self.active_borrows
            .values()
            .find(|b| b.borrowed_from == var_name && b.is_mut)
    }

    fn find_any_active_borrow_of(&self, var_name: &str) -> Option<&BorrowInfo> {
        self.active_borrows
            .values()
            .find(|b| b.borrowed_from == var_name)
    }
}
