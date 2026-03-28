//! Core OwnershipChecker struct and scope management

use super::{BorrowInfo, OwnershipInfo};
use crate::lifetime::LifetimeInferencer;
use crate::types::{TypeError, TypeResult};
use std::collections::HashMap;

/// The ownership and borrow checker
pub struct OwnershipChecker {
    /// Stack of scopes, each mapping variable names to ownership info
    pub(super) scopes: Vec<HashMap<String, OwnershipInfo>>,
    /// Active borrows: borrower variable -> borrow info
    pub(super) active_borrows: HashMap<String, BorrowInfo>,
    /// Reference tracking: ref variable name -> what it references
    pub(super) reference_sources: HashMap<String, super::ReferenceInfo>,
    /// Current scope ID counter
    pub(super) next_scope_id: u32,
    /// Current scope ID
    pub(super) current_scope: u32,
    /// Scope depth (increments on push, decrements on pop)
    pub(super) scope_depth: u32,
    /// Whether the current function returns a reference type
    pub(super) function_returns_ref: bool,
    /// Collected errors (non-fatal mode)
    pub(super) errors: Vec<TypeError>,
    /// Whether to collect errors instead of returning immediately
    pub(super) collect_errors: bool,
    /// Lifetime inferencer for lifetime bounds validation
    pub(super) lifetime_inferencer: LifetimeInferencer,
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
            reference_sources: HashMap::new(),
            next_scope_id: 1,
            current_scope: 0,
            scope_depth: 0,
            function_returns_ref: false,
            errors: Vec::new(),
            collect_errors: false,
            lifetime_inferencer: LifetimeInferencer::new(),
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

    /// Error reporting helper
    pub(super) fn report_error(&mut self, err: TypeError) -> TypeResult<()> {
        if self.collect_errors {
            self.errors.push(err);
            Ok(())
        } else {
            Err(err)
        }
    }

    // --- Scope management ---

    pub(super) fn push_scope(&mut self) {
        let _id = self.next_scope_id;
        self.next_scope_id += 1;
        self.current_scope = _id;
        self.scope_depth += 1;
        self.scopes.push(HashMap::new());
    }

    pub(super) fn pop_scope(&mut self) {
        use super::ReferenceInfo;
        use std::collections::HashSet;

        // Check for dangling references: references in outer scopes that point to
        // variables being dropped in this scope
        if let Some(dying_scope) = self.scopes.last() {
            let dying_vars: HashSet<String> = dying_scope.keys().cloned().collect();
            let current_depth = self.scope_depth;

            // Find references that point to variables in the dying scope
            let dangling: Vec<(String, ReferenceInfo)> = self
                .reference_sources
                .iter()
                .filter(|(_, ref_info)| {
                    dying_vars.contains(&ref_info.source_var)
                        && ref_info.source_scope_depth >= current_depth
                })
                .map(|(name, info)| (name.clone(), info.clone()))
                .collect();

            for (ref_var, ref_info) in dangling {
                // Only report if the reference itself is in an outer scope
                if let Some(ref_owner) = self.lookup_var(&ref_var) {
                    if ref_owner.defined_in_scope < self.current_scope {
                        let err = TypeError::DanglingReference {
                            ref_var: ref_var.clone(),
                            source_var: ref_info.source_var.clone(),
                            ref_scope_depth: ref_owner.defined_in_scope,
                            source_scope_depth: ref_info.source_scope_depth,
                            ref_at: ref_owner.defined_at,
                            source_defined_at: ref_info.source_defined_at,
                        };
                        let _ = self.report_error(err);
                    }
                }
                // Clean up the dangling reference tracking
                self.reference_sources.remove(&ref_var);
            }
        }

        // Invalidate borrows from this scope
        let scope_id = self.current_scope;
        self.active_borrows
            .retain(|_, info| info.scope_id != scope_id);

        // Clean up reference tracking for variables going out of scope
        if let Some(dying_scope) = self.scopes.last() {
            let dying_vars: HashSet<String> = dying_scope.keys().cloned().collect();
            self.reference_sources
                .retain(|name, _| !dying_vars.contains(name));
        }

        // Remove variables from the scope
        self.scopes.pop();
        if self.scope_depth > 0 {
            self.scope_depth -= 1;
        }
        if self.current_scope > 0 {
            self.current_scope -= 1;
        }
    }

    /// Save ownership states of all variables in all scopes (for branch analysis)
    pub(super) fn save_ownership_snapshot(&self) -> Vec<HashMap<String, OwnershipInfo>> {
        self.scopes.iter().map(|scope| scope.clone()).collect()
    }

    /// Restore ownership states from a snapshot (for branch analysis)
    pub(super) fn restore_ownership_snapshot(
        &mut self,
        snapshot: Vec<HashMap<String, OwnershipInfo>>,
    ) {
        // Restore only the ownership states that existed in the snapshot
        // (new variables declared in a branch are handled by push_scope/pop_scope)
        for (i, saved_scope) in snapshot.iter().enumerate() {
            if i < self.scopes.len() {
                for (name, saved_info) in saved_scope {
                    if let Some(current_info) = self.scopes[i].get_mut(name) {
                        current_info.state = saved_info.state.clone();
                    }
                }
            }
        }
    }

    /// Merge two ownership snapshots: a variable is "moved" only if BOTH branches moved it
    pub(super) fn merge_branch_ownership(
        &mut self,
        before: &[HashMap<String, OwnershipInfo>],
        after_then: &[HashMap<String, OwnershipInfo>],
        after_else: &[HashMap<String, OwnershipInfo>],
    ) {
        for (i, before_scope) in before.iter().enumerate() {
            if i >= self.scopes.len() {
                continue;
            }
            for (name, before_info) in before_scope {
                let then_moved = after_then
                    .get(i)
                    .and_then(|s| s.get(name))
                    .map(|info| matches!(info.state, super::OwnershipState::Moved { .. }))
                    .unwrap_or(false);

                let else_moved = after_else
                    .get(i)
                    .and_then(|s| s.get(name))
                    .map(|info| matches!(info.state, super::OwnershipState::Moved { .. }))
                    .unwrap_or(false);

                if let Some(current_info) = self.scopes[i].get_mut(name) {
                    if then_moved && else_moved {
                        // Both branches moved it → keep as moved (use the then-branch span)
                        if let Some(then_info) = after_then.get(i).and_then(|s| s.get(name)) {
                            current_info.state = then_info.state.clone();
                        }
                    } else {
                        // Only one or neither branch moved it → restore to before state
                        current_info.state = before_info.state.clone();
                    }
                }
            }
        }
    }

    /// Look up a variable's ownership info
    pub(super) fn lookup_var(&self, name: &str) -> Option<&OwnershipInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    /// Look up a variable's ownership info mutably
    pub(super) fn lookup_var_mut(&mut self, name: &str) -> Option<&mut OwnershipInfo> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                return Some(info);
            }
        }
        None
    }
}
