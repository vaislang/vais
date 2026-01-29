//! Revision tracking for incremental computation.

use std::sync::atomic::{AtomicU64, Ordering};

/// A monotonically increasing revision counter.
/// Each time an input changes, the global revision increments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision(pub u64);

impl Revision {
    pub const INITIAL: Revision = Revision(0);

    pub fn next(self) -> Revision {
        Revision(self.0 + 1)
    }
}

/// Global revision counter shared across the database.
pub(crate) struct RevisionCounter {
    current: AtomicU64,
}

impl RevisionCounter {
    pub fn new() -> Self {
        Self {
            current: AtomicU64::new(1),
        }
    }

    pub fn current(&self) -> Revision {
        Revision(self.current.load(Ordering::SeqCst))
    }

    pub fn increment(&self) -> Revision {
        let new = self.current.fetch_add(1, Ordering::SeqCst) + 1;
        Revision(new)
    }
}

impl Default for RevisionCounter {
    fn default() -> Self {
        Self::new()
    }
}
