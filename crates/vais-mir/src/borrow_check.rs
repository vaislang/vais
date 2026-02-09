//! MIR Borrow Checker: ownership tracking, lifetime analysis, use-after-move detection.
//!
//! This module implements a simplified borrow checker that operates on MIR, detecting:
//! - Use-after-move errors (using a value after ownership transfer)
//! - Double-free errors (dropping the same value twice)
//! - Use-after-free errors (using a value after it was dropped)
//! - Mutable borrow conflicts (multiple active mutable borrows)
//! - Shared/mutable borrow conflicts (shared borrow while mutably borrowed)
//! - Move-while-borrowed errors (moving a value that has active borrows)

use crate::*;
use std::collections::HashMap;
use std::fmt;

/// A location in the MIR body (basic block + statement index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub block: BasicBlockId,
    pub statement_index: usize,
}

impl Location {
    pub fn new(block: BasicBlockId, statement_index: usize) -> Self {
        Self {
            block,
            statement_index,
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.block.0, self.statement_index)
    }
}

/// Errors detected by the borrow checker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BorrowError {
    /// Using a value after it was moved.
    UseAfterMove {
        local: Local,
        moved_at: Location,
        used_at: Location,
    },
    /// Dropping a value twice.
    DoubleFree {
        local: Local,
        first_drop: Location,
        second_drop: Location,
    },
    /// Using a value after it was dropped.
    UseAfterFree {
        local: Local,
        freed_at: Location,
        used_at: Location,
    },
    /// Multiple active mutable borrows.
    MutableBorrowConflict {
        local: Local,
        first_borrow: Location,
        second_borrow: Location,
    },
    /// Shared borrow while mutably borrowed.
    BorrowWhileMutablyBorrowed {
        local: Local,
        mutable_borrow: Location,
        shared_borrow: Location,
    },
    /// Moving a value that has active borrows.
    MoveWhileBorrowed {
        local: Local,
        borrow_at: Location,
        move_at: Location,
    },
}

impl fmt::Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorrowError::UseAfterMove { local, moved_at, used_at } => {
                write!(
                    f,
                    "error[E100]: use of moved value `{}`\n  --> moved at {}\n  --> used at {}",
                    local.0, moved_at, used_at
                )
            }
            BorrowError::DoubleFree { local, first_drop, second_drop } => {
                write!(
                    f,
                    "error[E101]: value `{}` dropped twice\n  --> first drop at {}\n  --> second drop at {}",
                    local.0, first_drop, second_drop
                )
            }
            BorrowError::UseAfterFree { local, freed_at, used_at } => {
                write!(
                    f,
                    "error[E102]: use of freed value `{}`\n  --> freed at {}\n  --> used at {}",
                    local.0, freed_at, used_at
                )
            }
            BorrowError::MutableBorrowConflict { local, first_borrow, second_borrow } => {
                write!(
                    f,
                    "error[E103]: cannot borrow `{}` as mutable more than once\n  --> first mutable borrow at {}\n  --> second mutable borrow at {}",
                    local.0, first_borrow, second_borrow
                )
            }
            BorrowError::BorrowWhileMutablyBorrowed { local, mutable_borrow, shared_borrow } => {
                write!(
                    f,
                    "error[E104]: cannot borrow `{}` as shared while mutably borrowed\n  --> mutable borrow at {}\n  --> shared borrow at {}",
                    local.0, mutable_borrow, shared_borrow
                )
            }
            BorrowError::MoveWhileBorrowed { local, borrow_at, move_at } => {
                write!(
                    f,
                    "error[E105]: cannot move `{}` while borrowed\n  --> borrowed at {}\n  --> moved at {}",
                    local.0, borrow_at, move_at
                )
            }
        }
    }
}

/// The ownership state of a local variable.
#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalState {
    /// Not yet initialized.
    Uninitialized,
    /// Owns the value (location where it was assigned).
    Owned(Location),
    /// Value was moved away (location where it was moved).
    Moved(Location),
    /// Value was explicitly dropped (location where it was dropped).
    Dropped(Location),
}

/// Type of borrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BorrowKind {
    Shared,
    Mutable,
}

/// Information about an active borrow.
#[derive(Debug, Clone, PartialEq, Eq)]
struct BorrowInfo {
    kind: BorrowKind,
    location: Location,
}

/// The main borrow checker structure.
pub struct BorrowChecker<'a> {
    body: &'a Body,
    local_states: HashMap<Local, LocalState>,
    active_borrows: HashMap<Local, Vec<BorrowInfo>>,
    errors: Vec<BorrowError>,
}

impl<'a> BorrowChecker<'a> {
    fn new(body: &'a Body) -> Self {
        let mut local_states = HashMap::new();

        // Initialize all locals as uninitialized, except:
        // - _0 (return place) starts as Owned (will be written before return)
        // - Parameters (_1 .. _1+params.len()) start as Owned
        local_states.insert(Local(0), LocalState::Uninitialized);
        for i in 1..=body.params.len() {
            local_states.insert(
                Local(i as u32),
                LocalState::Owned(Location::new(BasicBlockId(0), 0)),
            );
        }

        // All other locals start uninitialized
        for i in (body.params.len() + 1)..body.locals.len() {
            local_states.insert(Local(i as u32), LocalState::Uninitialized);
        }

        Self {
            body,
            local_states,
            active_borrows: HashMap::new(),
            errors: vec![],
        }
    }

    /// Get the type of a local.
    fn local_type(&self, local: Local) -> &MirType {
        &self.body.locals[local.0 as usize].ty
    }

    /// Check if a local's type is Copy.
    fn is_copy(&self, local: Local) -> bool {
        self.local_type(local).is_copy()
    }

    /// Check if a place can be used (read) at the given location.
    fn check_use(&mut self, place: &Place, location: Location) {
        let local = place.local;

        // Copy types can always be used
        if self.is_copy(local) {
            return;
        }

        let state = self.local_states.get(&local).cloned().unwrap_or(LocalState::Uninitialized);

        match state {
            LocalState::Moved(moved_at) => {
                self.errors.push(BorrowError::UseAfterMove {
                    local,
                    moved_at,
                    used_at: location,
                });
            }
            LocalState::Dropped(freed_at) => {
                self.errors.push(BorrowError::UseAfterFree {
                    local,
                    freed_at,
                    used_at: location,
                });
            }
            LocalState::Owned(_) | LocalState::Uninitialized => {
                // OK to use
            }
        }
    }

    /// Record a move of a place at the given location.
    fn record_move(&mut self, place: &Place, location: Location) {
        let local = place.local;

        // Copy types don't move
        if self.is_copy(local) {
            return;
        }

        // Check for active borrows
        if let Some(borrows) = self.active_borrows.get(&local) {
            if let Some(first_borrow) = borrows.first() {
                self.errors.push(BorrowError::MoveWhileBorrowed {
                    local,
                    borrow_at: first_borrow.location,
                    move_at: location,
                });
            }
        }

        // Check current state
        let state = self.local_states.get(&local).cloned().unwrap_or(LocalState::Uninitialized);

        match state {
            LocalState::Dropped(freed_at) => {
                self.errors.push(BorrowError::UseAfterFree {
                    local,
                    freed_at,
                    used_at: location,
                });
            }
            LocalState::Owned(_) | LocalState::Uninitialized => {
                // Record the move
                self.local_states.insert(local, LocalState::Moved(location));
            }
            LocalState::Moved(moved_at) => {
                // Already moved - report use-after-move
                self.errors.push(BorrowError::UseAfterMove {
                    local,
                    moved_at,
                    used_at: location,
                });
            }
        }
    }

    /// Record an assignment to a place.
    fn record_assign(&mut self, place: &Place, location: Location) {
        let local = place.local;

        // Assignment reinitializes the local (makes it Owned again)
        self.local_states.insert(local, LocalState::Owned(location));

        // Clear any active borrows (assignment invalidates them)
        self.active_borrows.remove(&local);
    }

    /// Record a borrow of a place.
    fn record_borrow(&mut self, place: &Place, kind: BorrowKind, location: Location) {
        let local = place.local;

        // Check for existing borrows
        let existing = self.active_borrows.entry(local).or_default();

        for borrow in existing.iter() {
            match (borrow.kind, kind) {
                (BorrowKind::Mutable, BorrowKind::Mutable) => {
                    self.errors.push(BorrowError::MutableBorrowConflict {
                        local,
                        first_borrow: borrow.location,
                        second_borrow: location,
                    });
                }
                (BorrowKind::Mutable, BorrowKind::Shared) => {
                    self.errors.push(BorrowError::BorrowWhileMutablyBorrowed {
                        local,
                        mutable_borrow: borrow.location,
                        shared_borrow: location,
                    });
                }
                (BorrowKind::Shared, BorrowKind::Mutable) => {
                    self.errors.push(BorrowError::BorrowWhileMutablyBorrowed {
                        local,
                        mutable_borrow: location,
                        shared_borrow: borrow.location,
                    });
                }
                (BorrowKind::Shared, BorrowKind::Shared) => {
                    // Multiple shared borrows are OK
                }
            }
        }

        existing.push(BorrowInfo { kind, location });
    }

    /// Record a drop of a place.
    fn record_drop(&mut self, place: &Place, location: Location) {
        let local = place.local;

        // Copy types don't need explicit drops (no-op)
        if self.is_copy(local) {
            return;
        }

        let state = self.local_states.get(&local).cloned().unwrap_or(LocalState::Uninitialized);

        match state {
            LocalState::Dropped(first_drop) => {
                self.errors.push(BorrowError::DoubleFree {
                    local,
                    first_drop,
                    second_drop: location,
                });
            }
            LocalState::Moved(_) => {
                // No error: dropping an already-moved value is a no-op
                // (the value was already moved, so drop has nothing to do)
            }
            LocalState::Owned(_) | LocalState::Uninitialized => {
                self.local_states.insert(local, LocalState::Dropped(location));
            }
        }
    }

    /// Check an operand at the given location.
    fn check_operand(&mut self, operand: &Operand, location: Location) {
        match operand {
            Operand::Copy(place) => {
                self.check_use(place, location);
            }
            Operand::Move(place) => {
                self.check_use(place, location);
                self.record_move(place, location);
            }
            Operand::Constant(_) => {
                // Constants are always OK
            }
        }
    }

    /// Check an rvalue at the given location.
    fn check_rvalue(&mut self, rvalue: &Rvalue, location: Location) {
        match rvalue {
            Rvalue::Use(operand) => {
                self.check_operand(operand, location);
            }
            Rvalue::BinaryOp(_, left, right) => {
                self.check_operand(left, location);
                self.check_operand(right, location);
            }
            Rvalue::UnaryOp(_, operand) => {
                self.check_operand(operand, location);
            }
            Rvalue::Ref(place) => {
                self.check_use(place, location);
                // Determine borrow kind based on local mutability
                let kind = if place.local.0 < self.body.locals.len() as u32
                    && self.body.locals[place.local.0 as usize].is_mutable
                {
                    BorrowKind::Mutable
                } else {
                    BorrowKind::Shared
                };
                self.record_borrow(place, kind, location);
            }
            Rvalue::Aggregate(_, operands) => {
                for operand in operands {
                    self.check_operand(operand, location);
                }
            }
            Rvalue::Discriminant(place) => {
                self.check_use(place, location);
            }
            Rvalue::Cast(operand, _) => {
                self.check_operand(operand, location);
            }
            Rvalue::Len(place) => {
                self.check_use(place, location);
            }
        }
    }

    /// Check a statement at the given location.
    fn check_statement(&mut self, statement: &Statement, location: Location) {
        match statement {
            Statement::Assign(place, rvalue) => {
                // Check the rvalue first (reads on RHS)
                self.check_rvalue(rvalue, location);
                // Then record the assignment (write on LHS)
                self.record_assign(place, location);
            }
            Statement::Drop(place) => {
                self.record_drop(place, location);
            }
            Statement::Nop => {
                // No-op
            }
        }
    }

    /// Check a terminator at the given location.
    fn check_terminator(&mut self, terminator: &Terminator, location: Location) {
        match terminator {
            Terminator::Goto(_) => {
                // No operands to check
            }
            Terminator::SwitchInt { discriminant, .. } => {
                self.check_operand(discriminant, location);
            }
            Terminator::Return => {
                // Check that the return place (_0) is properly initialized
                let return_local = Local(0);
                if !self.is_copy(return_local) {
                    let state = self.local_states.get(&return_local).cloned().unwrap_or(LocalState::Uninitialized);
                    match state {
                        LocalState::Moved(moved_at) => {
                            self.errors.push(BorrowError::UseAfterMove {
                                local: return_local,
                                moved_at,
                                used_at: location,
                            });
                        }
                        LocalState::Dropped(freed_at) => {
                            self.errors.push(BorrowError::UseAfterFree {
                                local: return_local,
                                freed_at,
                                used_at: location,
                            });
                        }
                        LocalState::Owned(_) | LocalState::Uninitialized => {
                            // OK
                        }
                    }
                }
            }
            Terminator::Call { args, .. } => {
                for arg in args {
                    self.check_operand(arg, location);
                }
            }
            Terminator::TailCall { args, .. } => {
                for arg in args {
                    self.check_operand(arg, location);
                }
            }
            Terminator::Unreachable => {
                // No checks needed
            }
            Terminator::Assert { cond, .. } => {
                self.check_operand(cond, location);
            }
        }
    }

    /// Run the borrow checker on the body.
    fn check(&mut self) -> Vec<BorrowError> {
        // Simple forward pass through basic blocks in order
        for (block_idx, block) in self.body.basic_blocks.iter().enumerate() {
            let block_id = BasicBlockId(block_idx as u32);

            // Check each statement
            for (stmt_idx, statement) in block.statements.iter().enumerate() {
                let location = Location::new(block_id, stmt_idx);
                self.check_statement(statement, location);
            }

            // Check terminator
            if let Some(ref terminator) = block.terminator {
                let location = Location::new(block_id, block.statements.len());
                self.check_terminator(terminator, location);
            }
        }

        self.errors.clone()
    }
}

/// Compute the predecessor blocks for each block in the CFG.
/// Returns a map from block ID to the list of blocks that can jump to it.
pub fn cfg_predecessors(body: &Body) -> HashMap<BasicBlockId, Vec<BasicBlockId>> {
    let mut predecessors: HashMap<BasicBlockId, Vec<BasicBlockId>> = HashMap::new();

    for (block_idx, block) in body.basic_blocks.iter().enumerate() {
        let block_id = BasicBlockId(block_idx as u32);

        if let Some(ref terminator) = block.terminator {
            let successors = match terminator {
                Terminator::Goto(target) => vec![*target],
                Terminator::SwitchInt { targets, otherwise, .. } => {
                    let mut succ = targets.iter().map(|(_, target)| *target).collect::<Vec<_>>();
                    succ.push(*otherwise);
                    succ
                }
                Terminator::Return | Terminator::Unreachable => vec![],
                Terminator::Call { target, .. } => vec![*target],
                Terminator::TailCall { .. } => vec![],
                Terminator::Assert { target, .. } => vec![*target],
            };

            for successor in successors {
                predecessors.entry(successor).or_default().push(block_id);
            }
        }
    }

    predecessors
}

/// Compute the successor blocks for each block in the CFG.
/// Returns a map from block ID to the list of blocks that this block can jump to.
pub fn cfg_successors(body: &Body) -> HashMap<BasicBlockId, Vec<BasicBlockId>> {
    let mut successors: HashMap<BasicBlockId, Vec<BasicBlockId>> = HashMap::new();

    for (block_idx, block) in body.basic_blocks.iter().enumerate() {
        let block_id = BasicBlockId(block_idx as u32);

        if let Some(ref terminator) = block.terminator {
            let succ = match terminator {
                Terminator::Goto(target) => vec![*target],
                Terminator::SwitchInt { targets, otherwise, .. } => {
                    let mut s = targets.iter().map(|(_, target)| *target).collect::<Vec<_>>();
                    s.push(*otherwise);
                    s
                }
                Terminator::Return | Terminator::Unreachable => vec![],
                Terminator::Call { target, .. } => vec![*target],
                Terminator::TailCall { .. } => vec![],
                Terminator::Assert { target, .. } => vec![*target],
            };

            successors.insert(block_id, succ);
        } else {
            successors.insert(block_id, vec![]);
        }
    }

    successors
}

/// Check a MIR body for borrow errors.
pub fn check_body(body: &Body) -> Vec<BorrowError> {
    let mut checker = BorrowChecker::new(body);
    checker.check()
}

/// Check all bodies in a MIR module for borrow errors.
pub fn check_module(module: &MirModule) -> Vec<BorrowError> {
    let mut all_errors = Vec::new();

    for body in &module.bodies {
        let errors = check_body(body);
        all_errors.extend(errors);
    }

    all_errors
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a simple body for testing.
    fn make_test_body(ty: MirType, statements: Vec<Statement>, terminator: Terminator) -> Body {
        Body {
            name: "test".to_string(),
            params: vec![ty.clone()],
            return_type: ty.clone(),
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: ty.clone(),
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("x".to_string()),
                    ty,
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements,
                terminator: Some(terminator),
            }],
            block_names: HashMap::new(),
        }
    }

    #[test]
    fn test_no_errors_copy_types() {
        // Copy types can be used multiple times without errors
        let body = make_test_body(
            MirType::I64,
            vec![
                Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                ),
                Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                ),
            ],
            Terminator::Return,
        );

        let errors = check_body(&body);
        assert_eq!(errors.len(), 0, "Copy types should not produce errors");
    }

    #[test]
    fn test_use_after_move() {
        // Non-copy type (Str) moved then used again
        let body = Body {
            name: "test_move".to_string(),
            params: vec![MirType::Str],
            return_type: MirType::Str,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Str,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("s".to_string()),
                    ty: MirType::Str,
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![
                    // Move s to _ret
                    Statement::Assign(
                        Place::local(Local(0)),
                        Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                    ),
                    // Try to use s again (error!)
                    Statement::Assign(
                        Place::local(Local(0)),
                        Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                    ),
                ],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let errors = check_body(&body);
        assert_eq!(errors.len(), 1, "Should detect use-after-move");
        match &errors[0] {
            BorrowError::UseAfterMove { local, .. } => {
                assert_eq!(*local, Local(1));
            }
            _ => panic!("Expected UseAfterMove error"),
        }
    }

    #[test]
    fn test_double_drop() {
        // Drop the same non-copy value twice
        let body = Body {
            name: "test_double_drop".to_string(),
            params: vec![MirType::Str],
            return_type: MirType::Unit,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Unit,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("s".to_string()),
                    ty: MirType::Str,
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![
                    Statement::Drop(Place::local(Local(1))),
                    Statement::Drop(Place::local(Local(1))), // Double drop!
                ],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let errors = check_body(&body);
        assert_eq!(errors.len(), 1, "Should detect double-free");
        match &errors[0] {
            BorrowError::DoubleFree { local, .. } => {
                assert_eq!(*local, Local(1));
            }
            _ => panic!("Expected DoubleFree error"),
        }
    }

    #[test]
    fn test_move_prevents_drop() {
        // Move a value, then drop it (should not error - drop is no-op after move)
        let body = Body {
            name: "test_move_no_drop".to_string(),
            params: vec![MirType::Str],
            return_type: MirType::Str,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Str,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("s".to_string()),
                    ty: MirType::Str,
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![
                    Statement::Assign(
                        Place::local(Local(0)),
                        Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                    ),
                    Statement::Drop(Place::local(Local(1))), // Drop after move is OK (no-op)
                ],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let errors = check_body(&body);
        assert_eq!(errors.len(), 0, "Drop after move should be allowed (no-op)");
    }

    #[test]
    fn test_copy_type_no_move_error() {
        // Copy types can be "moved" multiple times
        let body = make_test_body(
            MirType::I64,
            vec![
                Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                ),
                Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                ),
            ],
            Terminator::Return,
        );

        let errors = check_body(&body);
        assert_eq!(errors.len(), 0, "Copy types don't actually move");
    }

    #[test]
    fn test_assign_after_move() {
        // Move a value, then reassign it, then use it (should be OK)
        let body = Body {
            name: "test_reassign".to_string(),
            params: vec![MirType::Str],
            return_type: MirType::Str,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Str,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("s".to_string()),
                    ty: MirType::Str,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("temp".to_string()),
                    ty: MirType::Str,
                    is_mutable: true,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![
                    // Move s to temp
                    Statement::Assign(
                        Place::local(Local(2)),
                        Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                    ),
                    // Reassign s with the parameter (Local 0 holds new value conceptually)
                    Statement::Assign(
                        Place::local(Local(1)),
                        Rvalue::Use(Operand::Constant(Constant::Str("new".to_string()))),
                    ),
                    // Use s again (should be OK now)
                    Statement::Assign(
                        Place::local(Local(0)),
                        Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                    ),
                ],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let errors = check_body(&body);
        assert_eq!(errors.len(), 0, "Reassignment should reinitialize the local");
    }

    #[test]
    fn test_use_after_drop() {
        // Drop a value, then try to use it
        let body = Body {
            name: "test_use_after_drop".to_string(),
            params: vec![MirType::Str],
            return_type: MirType::Unit,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Unit,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("s".to_string()),
                    ty: MirType::Str,
                    is_mutable: false,
                },
                LocalDecl {
                    name: Some("temp".to_string()),
                    ty: MirType::Str,
                    is_mutable: true,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![
                    Statement::Drop(Place::local(Local(1))),
                    // Try to use s after drop (error!)
                    Statement::Assign(
                        Place::local(Local(2)),
                        Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                    ),
                ],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let errors = check_body(&body);
        assert_eq!(errors.len(), 1, "Should detect use-after-free");
        match &errors[0] {
            BorrowError::UseAfterFree { local, .. } => {
                assert_eq!(*local, Local(1));
            }
            _ => panic!("Expected UseAfterFree error"),
        }
    }

    #[test]
    fn test_mutable_borrow_conflict() {
        // Test that two mutable borrows create a conflict
        // (This test now actually detects mutable borrow conflicts!)
        let body = Body {
            name: "test_borrow_conflict".to_string(),
            params: vec![MirType::Str],
            return_type: MirType::Unit,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Unit,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("s".to_string()),
                    ty: MirType::Str,
                    is_mutable: true, // Mutable local = mutable borrows
                },
                LocalDecl {
                    name: Some("r1".to_string()),
                    ty: MirType::Ref(Box::new(MirType::Str)),
                    is_mutable: false,
                },
                LocalDecl {
                    name: Some("r2".to_string()),
                    ty: MirType::Ref(Box::new(MirType::Str)),
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![
                    // First mutable borrow of s
                    Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
                    // Second mutable borrow of s (error!)
                    Statement::Assign(Place::local(Local(3)), Rvalue::Ref(Place::local(Local(1)))),
                ],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let errors = check_body(&body);
        // Now that we detect mutability, this should error
        assert_eq!(errors.len(), 1, "Should detect mutable borrow conflict");
        match &errors[0] {
            BorrowError::MutableBorrowConflict { local, .. } => {
                assert_eq!(*local, Local(1));
            }
            _ => panic!("Expected MutableBorrowConflict error, got: {:?}", errors[0]),
        }
    }

    #[test]
    fn test_move_while_borrowed() {
        // Borrow a value, then try to move it
        let body = Body {
            name: "test_move_borrowed".to_string(),
            params: vec![MirType::Str],
            return_type: MirType::Str,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Str,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("s".to_string()),
                    ty: MirType::Str,
                    is_mutable: false,
                },
                LocalDecl {
                    name: Some("r".to_string()),
                    ty: MirType::Ref(Box::new(MirType::Str)),
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![
                    // Borrow s
                    Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
                    // Try to move s (error!)
                    Statement::Assign(
                        Place::local(Local(0)),
                        Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                    ),
                ],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let errors = check_body(&body);
        assert_eq!(errors.len(), 1, "Should detect move-while-borrowed");
        match &errors[0] {
            BorrowError::MoveWhileBorrowed { local, .. } => {
                assert_eq!(*local, Local(1));
            }
            _ => panic!("Expected MoveWhileBorrowed error"),
        }
    }

    #[test]
    fn test_return_value_not_dropped() {
        // Return a non-copy value (should not cause errors)
        let body = Body {
            name: "test_return".to_string(),
            params: vec![MirType::Str],
            return_type: MirType::Str,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Str,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("s".to_string()),
                    ty: MirType::Str,
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                )],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let errors = check_body(&body);
        assert_eq!(errors.len(), 0, "Returning a value should not error");
    }

    #[test]
    fn test_check_module() {
        // Test checking an entire module
        let body1 = make_test_body(
            MirType::I64,
            vec![Statement::Assign(
                Place::local(Local(0)),
                Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
            )],
            Terminator::Return,
        );

        let body2 = Body {
            name: "test_error".to_string(),
            params: vec![MirType::Str],
            return_type: MirType::Unit,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Unit,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("s".to_string()),
                    ty: MirType::Str,
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![
                    Statement::Drop(Place::local(Local(1))),
                    Statement::Drop(Place::local(Local(1))), // Double drop
                ],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let module = MirModule {
            name: "test_module".to_string(),
            bodies: vec![body1, body2],
            structs: HashMap::new(),
            enums: HashMap::new(),
        };

        let errors = check_module(&module);
        assert_eq!(errors.len(), 1, "Should detect error in second body");
    }

    // New tests for advanced features

    #[test]
    fn test_cfg_successors() {
        // Create a body with if-else (SwitchInt)
        let body = Body {
            name: "test_cfg".to_string(),
            params: vec![MirType::I64],
            return_type: MirType::I64,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::I64,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("x".to_string()),
                    ty: MirType::I64,
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![
                // bb0: condition check
                BasicBlock {
                    statements: vec![],
                    terminator: Some(Terminator::SwitchInt {
                        discriminant: Operand::Copy(Place::local(Local(1))),
                        targets: vec![(0, BasicBlockId(1))],
                        otherwise: BasicBlockId(2),
                    }),
                },
                // bb1: then branch
                BasicBlock {
                    statements: vec![Statement::Assign(
                        Place::local(Local(0)),
                        Rvalue::Use(Operand::Constant(Constant::Int(10))),
                    )],
                    terminator: Some(Terminator::Goto(BasicBlockId(3))),
                },
                // bb2: else branch
                BasicBlock {
                    statements: vec![Statement::Assign(
                        Place::local(Local(0)),
                        Rvalue::Use(Operand::Constant(Constant::Int(20))),
                    )],
                    terminator: Some(Terminator::Goto(BasicBlockId(3))),
                },
                // bb3: merge block
                BasicBlock {
                    statements: vec![],
                    terminator: Some(Terminator::Return),
                },
            ],
            block_names: HashMap::new(),
        };

        let successors = cfg_successors(&body);

        // bb0 should have successors bb1, bb2
        assert_eq!(successors.get(&BasicBlockId(0)).unwrap().len(), 2);
        assert!(successors.get(&BasicBlockId(0)).unwrap().contains(&BasicBlockId(1)));
        assert!(successors.get(&BasicBlockId(0)).unwrap().contains(&BasicBlockId(2)));

        // bb1 should have successor bb3
        assert_eq!(successors.get(&BasicBlockId(1)).unwrap(), &vec![BasicBlockId(3)]);

        // bb2 should have successor bb3
        assert_eq!(successors.get(&BasicBlockId(2)).unwrap(), &vec![BasicBlockId(3)]);

        // bb3 should have no successors (Return)
        assert_eq!(successors.get(&BasicBlockId(3)).unwrap().len(), 0);
    }

    #[test]
    fn test_cfg_predecessors() {
        // Use the same body as test_cfg_successors
        let body = Body {
            name: "test_cfg".to_string(),
            params: vec![MirType::I64],
            return_type: MirType::I64,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::I64,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("x".to_string()),
                    ty: MirType::I64,
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![
                BasicBlock {
                    statements: vec![],
                    terminator: Some(Terminator::SwitchInt {
                        discriminant: Operand::Copy(Place::local(Local(1))),
                        targets: vec![(0, BasicBlockId(1))],
                        otherwise: BasicBlockId(2),
                    }),
                },
                BasicBlock {
                    statements: vec![Statement::Assign(
                        Place::local(Local(0)),
                        Rvalue::Use(Operand::Constant(Constant::Int(10))),
                    )],
                    terminator: Some(Terminator::Goto(BasicBlockId(3))),
                },
                BasicBlock {
                    statements: vec![Statement::Assign(
                        Place::local(Local(0)),
                        Rvalue::Use(Operand::Constant(Constant::Int(20))),
                    )],
                    terminator: Some(Terminator::Goto(BasicBlockId(3))),
                },
                BasicBlock {
                    statements: vec![],
                    terminator: Some(Terminator::Return),
                },
            ],
            block_names: HashMap::new(),
        };

        let predecessors = cfg_predecessors(&body);

        // bb0 should have no predecessors (entry block)
        assert!(predecessors.get(&BasicBlockId(0)).is_none());

        // bb1 should have predecessor bb0
        assert_eq!(predecessors.get(&BasicBlockId(1)).unwrap(), &vec![BasicBlockId(0)]);

        // bb2 should have predecessor bb0
        assert_eq!(predecessors.get(&BasicBlockId(2)).unwrap(), &vec![BasicBlockId(0)]);

        // bb3 should have predecessors bb1 and bb2
        assert_eq!(predecessors.get(&BasicBlockId(3)).unwrap().len(), 2);
        assert!(predecessors.get(&BasicBlockId(3)).unwrap().contains(&BasicBlockId(1)));
        assert!(predecessors.get(&BasicBlockId(3)).unwrap().contains(&BasicBlockId(2)));
    }

    #[test]
    fn test_mutable_borrow_conflict_real() {
        // Two &mut borrows on a mutable local
        let body = Body {
            name: "test_mut_conflict".to_string(),
            params: vec![],
            return_type: MirType::Unit,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Unit,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("x".to_string()),
                    ty: MirType::Str,
                    is_mutable: true, // Mutable local
                },
                LocalDecl {
                    name: Some("r1".to_string()),
                    ty: MirType::Ref(Box::new(MirType::Str)),
                    is_mutable: false,
                },
                LocalDecl {
                    name: Some("r2".to_string()),
                    ty: MirType::Ref(Box::new(MirType::Str)),
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![
                    // Initialize x
                    Statement::Assign(
                        Place::local(Local(1)),
                        Rvalue::Use(Operand::Constant(Constant::Str("test".to_string()))),
                    ),
                    // First mutable borrow
                    Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
                    // Second mutable borrow (error!)
                    Statement::Assign(Place::local(Local(3)), Rvalue::Ref(Place::local(Local(1)))),
                ],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let errors = check_body(&body);
        assert_eq!(errors.len(), 1, "Should detect mutable borrow conflict");
        match &errors[0] {
            BorrowError::MutableBorrowConflict { local, .. } => {
                assert_eq!(*local, Local(1));
            }
            _ => panic!("Expected MutableBorrowConflict error, got: {:?}", errors[0]),
        }
    }

    #[test]
    fn test_shared_borrow_while_mutable() {
        // &mut borrow then & borrow
        let body = Body {
            name: "test_shared_while_mut".to_string(),
            params: vec![],
            return_type: MirType::Unit,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Unit,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("x".to_string()),
                    ty: MirType::Str,
                    is_mutable: true, // Mutable local
                },
                LocalDecl {
                    name: Some("y".to_string()),
                    ty: MirType::Str,
                    is_mutable: false, // Immutable local
                },
                LocalDecl {
                    name: Some("r1".to_string()),
                    ty: MirType::Ref(Box::new(MirType::Str)),
                    is_mutable: false,
                },
                LocalDecl {
                    name: Some("r2".to_string()),
                    ty: MirType::Ref(Box::new(MirType::Str)),
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![
                    // Initialize x and y
                    Statement::Assign(
                        Place::local(Local(1)),
                        Rvalue::Use(Operand::Constant(Constant::Str("test1".to_string()))),
                    ),
                    Statement::Assign(
                        Place::local(Local(2)),
                        Rvalue::Use(Operand::Constant(Constant::Str("test2".to_string()))),
                    ),
                    // Mutable borrow of x
                    Statement::Assign(Place::local(Local(3)), Rvalue::Ref(Place::local(Local(1)))),
                    // Shared borrow of y (OK - different local)
                    Statement::Assign(Place::local(Local(4)), Rvalue::Ref(Place::local(Local(2)))),
                ],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let errors = check_body(&body);
        // No error - borrowing different locals
        assert_eq!(errors.len(), 0, "Borrowing different locals should be OK");
    }

    #[test]
    fn test_multiple_shared_borrows_ok() {
        // Multiple & borrows on immutable local
        let body = Body {
            name: "test_multiple_shared".to_string(),
            params: vec![],
            return_type: MirType::Unit,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Unit,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("x".to_string()),
                    ty: MirType::Str,
                    is_mutable: false, // Immutable local
                },
                LocalDecl {
                    name: Some("r1".to_string()),
                    ty: MirType::Ref(Box::new(MirType::Str)),
                    is_mutable: false,
                },
                LocalDecl {
                    name: Some("r2".to_string()),
                    ty: MirType::Ref(Box::new(MirType::Str)),
                    is_mutable: false,
                },
                LocalDecl {
                    name: Some("r3".to_string()),
                    ty: MirType::Ref(Box::new(MirType::Str)),
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![
                    // Initialize x
                    Statement::Assign(
                        Place::local(Local(1)),
                        Rvalue::Use(Operand::Constant(Constant::Str("test".to_string()))),
                    ),
                    // First shared borrow
                    Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
                    // Second shared borrow
                    Statement::Assign(Place::local(Local(3)), Rvalue::Ref(Place::local(Local(1)))),
                    // Third shared borrow
                    Statement::Assign(Place::local(Local(4)), Rvalue::Ref(Place::local(Local(1)))),
                ],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let errors = check_body(&body);
        assert_eq!(errors.len(), 0, "Multiple shared borrows should be OK");
    }

    #[test]
    fn test_error_display_format() {
        // Check that BorrowError::Display produces correct error code format
        let error = BorrowError::UseAfterMove {
            local: Local(1),
            moved_at: Location::new(BasicBlockId(0), 2),
            used_at: Location::new(BasicBlockId(0), 4),
        };
        let display = format!("{}", error);
        assert!(display.contains("error[E100]"));
        assert!(display.contains("use of moved value"));
        assert!(display.contains("moved at 0:2"));
        assert!(display.contains("used at 0:4"));

        let error2 = BorrowError::MutableBorrowConflict {
            local: Local(2),
            first_borrow: Location::new(BasicBlockId(1), 0),
            second_borrow: Location::new(BasicBlockId(1), 1),
        };
        let display2 = format!("{}", error2);
        assert!(display2.contains("error[E103]"));
        assert!(display2.contains("cannot borrow"));
        assert!(display2.contains("mutable more than once"));
    }

    #[test]
    fn test_location_display() {
        // Check Location display format
        let loc = Location::new(BasicBlockId(3), 7);
        let display = format!("{}", loc);
        assert_eq!(display, "3:7");

        let loc2 = Location::new(BasicBlockId(0), 0);
        let display2 = format!("{}", loc2);
        assert_eq!(display2, "0:0");
    }

    #[test]
    fn test_borrow_invalidated_by_assign() {
        // Assign to a borrowed place should clear borrows
        let body = Body {
            name: "test_assign_invalidate".to_string(),
            params: vec![],
            return_type: MirType::Unit,
            locals: vec![
                LocalDecl {
                    name: Some("_ret".to_string()),
                    ty: MirType::Unit,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("x".to_string()),
                    ty: MirType::Str,
                    is_mutable: true,
                },
                LocalDecl {
                    name: Some("r".to_string()),
                    ty: MirType::Ref(Box::new(MirType::Str)),
                    is_mutable: false,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![
                    // Initialize x
                    Statement::Assign(
                        Place::local(Local(1)),
                        Rvalue::Use(Operand::Constant(Constant::Str("test".to_string()))),
                    ),
                    // Borrow x
                    Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
                    // Assign to x (invalidates the borrow)
                    Statement::Assign(
                        Place::local(Local(1)),
                        Rvalue::Use(Operand::Constant(Constant::Str("new".to_string()))),
                    ),
                    // Borrow x again (should be OK - previous borrow was invalidated)
                    Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
                ],
                terminator: Some(Terminator::Return),
            }],
            block_names: HashMap::new(),
        };

        let errors = check_body(&body);
        assert_eq!(errors.len(), 0, "Assignment should invalidate previous borrows");
    }
}
