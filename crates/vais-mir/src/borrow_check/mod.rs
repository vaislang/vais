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

mod cfg;
mod checker;
mod lifetime;

#[cfg(test)]
mod tests;

// Re-export public API
pub use cfg::{cfg_predecessors, cfg_successors};
pub use checker::BorrowChecker;
pub use lifetime::{check_body, check_module};

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
    /// Lifetime violation: 'a does not outlive 'b
    LifetimeViolation {
        shorter: String,
        longer: String,
        location: Location,
    },
}

impl fmt::Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorrowError::UseAfterMove {
                local,
                moved_at,
                used_at,
            } => {
                write!(
                    f,
                    "error[E100]: use of moved value `{}`\n  --> moved at {}\n  --> used at {}",
                    local.0, moved_at, used_at
                )
            }
            BorrowError::DoubleFree {
                local,
                first_drop,
                second_drop,
            } => {
                write!(
                    f,
                    "error[E101]: value `{}` dropped twice\n  --> first drop at {}\n  --> second drop at {}",
                    local.0, first_drop, second_drop
                )
            }
            BorrowError::UseAfterFree {
                local,
                freed_at,
                used_at,
            } => {
                write!(
                    f,
                    "error[E102]: use of freed value `{}`\n  --> freed at {}\n  --> used at {}",
                    local.0, freed_at, used_at
                )
            }
            BorrowError::MutableBorrowConflict {
                local,
                first_borrow,
                second_borrow,
            } => {
                write!(
                    f,
                    "error[E103]: cannot borrow `{}` as mutable more than once\n  --> first mutable borrow at {}\n  --> second mutable borrow at {}",
                    local.0, first_borrow, second_borrow
                )
            }
            BorrowError::BorrowWhileMutablyBorrowed {
                local,
                mutable_borrow,
                shared_borrow,
            } => {
                write!(
                    f,
                    "error[E104]: cannot borrow `{}` as shared while mutably borrowed\n  --> mutable borrow at {}\n  --> shared borrow at {}",
                    local.0, mutable_borrow, shared_borrow
                )
            }
            BorrowError::MoveWhileBorrowed {
                local,
                borrow_at,
                move_at,
            } => {
                write!(
                    f,
                    "error[E105]: cannot move `{}` while borrowed\n  --> borrowed at {}\n  --> moved at {}",
                    local.0, borrow_at, move_at
                )
            }
            BorrowError::LifetimeViolation {
                shorter,
                longer,
                location,
            } => {
                write!(
                    f,
                    "error[E106]: lifetime '{}' does not outlive '{}'\n  --> at {}",
                    shorter, longer, location
                )
            }
        }
    }
}

/// The ownership state of a local variable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LocalState {
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
pub(super) enum BorrowKind {
    Shared,
    Mutable,
    /// Two-phase mutable borrow (reserved but not yet activated).
    ReservedMutable,
}

/// Information about an active borrow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BorrowInfo {
    pub(super) kind: BorrowKind,
    pub(super) location: Location,
    /// The local being borrowed.
    pub(super) borrowed_local: Local,
    /// The local where the borrow result is stored (if any).
    pub(super) borrow_target: Option<Local>,
}

/// Liveness information for locals.
#[derive(Debug, Clone)]
pub(super) struct LivenessInfo {
    /// For each local, the last location where it's used (read/copied/moved).
    pub(super) last_use: HashMap<Local, Location>,
}

/// State information for a single basic block.
#[derive(Debug, Clone)]
pub(super) struct BlockState {
    pub(super) entry_states: HashMap<Local, LocalState>,
    pub(super) exit_states: HashMap<Local, LocalState>,
    pub(super) entry_borrows: HashMap<Local, Vec<BorrowInfo>>,
    pub(super) exit_borrows: HashMap<Local, Vec<BorrowInfo>>,
}
