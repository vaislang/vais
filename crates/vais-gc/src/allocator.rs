//! GC Allocator trait and implementations

pub use crate::gc::GcStats;

/// GC Allocator interface
pub trait GcAllocator {
    /// Allocate memory managed by GC
    fn gc_alloc(&mut self, size: usize, type_id: u32) -> *mut u8;

    /// Register a root pointer
    fn gc_add_root(&mut self, ptr: usize);

    /// Unregister a root pointer
    fn gc_remove_root(&mut self, ptr: usize);

    /// Force garbage collection
    fn gc_collect(&mut self);

    /// Get GC statistics
    fn gc_stats(&self) -> GcStats;
}
