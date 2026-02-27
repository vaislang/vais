//! Type definitions for ownership and borrow checking

use crate::types::ResolvedType;
use std::collections::HashSet;
use vais_ast::Span;

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
    PartiallyMoved { moved_fields: HashSet<String> },
    /// Value is borrowed (immutably)
    Borrowed { borrow_count: usize },
    /// Value is mutably borrowed
    MutBorrowed { borrower: String },
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

/// Information about a reference variable and what it points to
#[derive(Debug, Clone)]
pub struct ReferenceInfo {
    /// The variable being referenced
    pub source_var: String,
    /// The scope depth where the source lives
    pub source_scope_depth: u32,
    /// Where the source was defined
    pub source_defined_at: Option<Span>,
    /// Whether the reference is mutable
    pub is_mut: bool,
}
