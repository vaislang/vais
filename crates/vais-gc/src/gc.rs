//! Core GC implementation - Mark and Sweep algorithm

use std::collections::{HashMap, HashSet};
use std::ptr;

/// GC Object header
///
/// Each GC-allocated object has a header containing metadata
#[repr(C)]
#[derive(Debug, Clone)]
pub struct GcObjectHeader {
    /// Size of the object in bytes (excluding header)
    pub size: usize,
    /// Mark bit for mark-and-sweep
    pub marked: bool,
    /// Reference count for cycle detection
    pub ref_count: usize,
    /// Type information (for debugging)
    pub type_id: u32,
}

impl GcObjectHeader {
    pub fn new(size: usize, type_id: u32) -> Self {
        Self {
            size,
            marked: false,
            ref_count: 0,
            type_id,
        }
    }
}

/// GC Object - header + data
#[derive(Debug)]
pub struct GcObject {
    pub header: GcObjectHeader,
    pub data: Vec<u8>,
}

impl GcObject {
    pub fn new(size: usize, type_id: u32) -> Self {
        Self {
            header: GcObjectHeader::new(size, type_id),
            data: vec![0u8; size],
        }
    }

    /// Get pointer to data (excluding header)
    pub fn data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Get mutable pointer to data
    pub fn data_ptr_mut(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }
}

/// GC Root - represents a pointer on the stack or in global data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GcRoot {
    pub ptr: usize,
}

impl GcRoot {
    pub fn new(ptr: usize) -> Self {
        Self { ptr }
    }
}

/// GC Heap - manages all GC objects
pub struct GcHeap {
    /// All allocated objects (key = data pointer)
    objects: HashMap<usize, GcObject>,
    /// Root set (stack pointers, globals)
    roots: HashSet<GcRoot>,
    /// Total bytes allocated
    bytes_allocated: usize,
    /// Threshold for triggering GC
    gc_threshold: usize,
    /// Statistics
    collections: usize,
    objects_freed: usize,
}

impl GcHeap {
    /// Create a new GC heap
    pub fn new() -> Self {
        Self::with_threshold(1024 * 1024) // 1 MB default threshold
    }

    /// Create GC heap with custom threshold
    pub fn with_threshold(threshold: usize) -> Self {
        Self {
            objects: HashMap::new(),
            roots: HashSet::new(),
            bytes_allocated: 0,
            gc_threshold: threshold,
            collections: 0,
            objects_freed: 0,
        }
    }

    /// Allocate a new GC object
    pub fn alloc(&mut self, size: usize, type_id: u32) -> *mut u8 {
        // Check if we should trigger GC
        if self.bytes_allocated >= self.gc_threshold {
            self.collect();
        }

        let mut obj = GcObject::new(size, type_id);
        let data_ptr = obj.data_ptr_mut() as usize;

        self.bytes_allocated += size + std::mem::size_of::<GcObjectHeader>();
        self.objects.insert(data_ptr, obj);

        data_ptr as *mut u8
    }

    /// Register a root
    pub fn add_root(&mut self, ptr: usize) {
        if ptr != 0 {
            self.roots.insert(GcRoot::new(ptr));
        }
    }

    /// Unregister a root
    pub fn remove_root(&mut self, ptr: usize) {
        self.roots.remove(&GcRoot::new(ptr));
    }

    /// Mark phase - mark all reachable objects
    fn mark(&mut self) {
        // Clear all marks
        for obj in self.objects.values_mut() {
            obj.header.marked = false;
        }

        // Collect root pointers to avoid borrowing issues
        let root_ptrs: Vec<usize> = self.roots.iter().map(|r| r.ptr).collect();

        // Mark from roots
        for ptr in root_ptrs {
            self.mark_object(ptr);
        }
    }

    /// Mark a single object and its children
    fn mark_object(&mut self, ptr: usize) {
        // Check if object exists and is not marked
        let should_scan = if let Some(obj) = self.objects.get_mut(&ptr) {
            if obj.header.marked {
                false
            } else {
                obj.header.marked = true;
                true
            }
        } else {
            false
        };

        if should_scan {
            // Get size first
            let size = self.objects.get(&ptr).map(|o| o.header.size).unwrap_or(0);
            // Scan for child pointers
            self.scan_for_pointers(ptr, size);
        }
    }

    /// Conservative pointer scanning
    fn scan_for_pointers(&mut self, base_ptr: usize, size: usize) {
        // Get data pointer without holding a borrow
        let data_vec = if let Some(obj) = self.objects.get(&base_ptr) {
            obj.data.clone()
        } else {
            return;
        };

        // Collect potential child pointers
        let mut child_ptrs = Vec::new();
        let ptr_size = std::mem::size_of::<usize>();

        for offset in (0..size).step_by(ptr_size) {
            if offset + ptr_size <= size {
                unsafe {
                    let potential_ptr = ptr::read(data_vec.as_ptr().add(offset) as *const usize);

                    // Check if this looks like a pointer to a GC object
                    if self.objects.contains_key(&potential_ptr) {
                        child_ptrs.push(potential_ptr);
                    }
                }
            }
        }

        // Mark children
        for child_ptr in child_ptrs {
            self.mark_object(child_ptr);
        }
    }

    /// Sweep phase - free unmarked objects
    fn sweep(&mut self) {
        let mut to_remove = Vec::new();

        for (ptr, obj) in &self.objects {
            if !obj.header.marked {
                to_remove.push(*ptr);
                self.bytes_allocated = self
                    .bytes_allocated
                    .saturating_sub(obj.header.size + std::mem::size_of::<GcObjectHeader>());
                self.objects_freed += 1;
            }
        }

        for ptr in to_remove {
            self.objects.remove(&ptr);
        }
    }

    /// Run garbage collection
    pub fn collect(&mut self) {
        self.collections += 1;
        self.mark();
        self.sweep();
    }

    /// Force collection (for testing/debugging)
    pub fn force_collect(&mut self) {
        self.collect();
    }

    /// Get statistics
    pub fn stats(&self) -> GcStats {
        GcStats {
            bytes_allocated: self.bytes_allocated,
            objects_count: self.objects.len(),
            collections: self.collections,
            objects_freed: self.objects_freed,
            gc_threshold: self.gc_threshold,
        }
    }

    /// Set GC threshold
    pub fn set_threshold(&mut self, threshold: usize) {
        self.gc_threshold = threshold;
    }
}

impl Default for GcHeap {
    fn default() -> Self {
        Self::new()
    }
}

/// GC Statistics
#[derive(Debug, Clone, Copy)]
pub struct GcStats {
    pub bytes_allocated: usize,
    pub objects_count: usize,
    pub collections: usize,
    pub objects_freed: usize,
    pub gc_threshold: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    // === GcObjectHeader tests ===

    #[test]
    fn test_gc_object_header_new() {
        let header = GcObjectHeader::new(128, 42);
        assert_eq!(header.size, 128);
        assert!(!header.marked);
        assert_eq!(header.ref_count, 0);
        assert_eq!(header.type_id, 42);
    }

    #[test]
    fn test_gc_object_header_zero_size() {
        let header = GcObjectHeader::new(0, 0);
        assert_eq!(header.size, 0);
        assert_eq!(header.type_id, 0);
    }

    // === GcObject tests ===

    #[test]
    fn test_gc_object_new() {
        let obj = GcObject::new(64, 1);
        assert_eq!(obj.header.size, 64);
        assert_eq!(obj.data.len(), 64);
        // Data should be zero-initialized
        assert!(obj.data.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_gc_object_data_ptr() {
        let obj = GcObject::new(32, 1);
        let ptr = obj.data_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_gc_object_data_ptr_mut() {
        let mut obj = GcObject::new(32, 1);
        let ptr = obj.data_ptr_mut();
        assert!(!ptr.is_null());
        // Write through pointer and verify
        unsafe {
            *ptr = 0xFF;
        }
        assert_eq!(obj.data[0], 0xFF);
    }

    // === GcRoot tests ===

    #[test]
    fn test_gc_root_new() {
        let root = GcRoot::new(0x1234);
        assert_eq!(root.ptr, 0x1234);
    }

    #[test]
    fn test_gc_root_equality() {
        let r1 = GcRoot::new(100);
        let r2 = GcRoot::new(100);
        let r3 = GcRoot::new(200);
        assert_eq!(r1, r2);
        assert_ne!(r1, r3);
    }

    #[test]
    fn test_gc_root_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(GcRoot::new(1));
        set.insert(GcRoot::new(1));
        set.insert(GcRoot::new(2));
        assert_eq!(set.len(), 2);
    }

    // === GcHeap allocation tests ===

    #[test]
    fn test_heap_new() {
        let heap = GcHeap::new();
        let stats = heap.stats();
        assert_eq!(stats.bytes_allocated, 0);
        assert_eq!(stats.objects_count, 0);
        assert_eq!(stats.collections, 0);
        assert_eq!(stats.gc_threshold, 1024 * 1024);
    }

    #[test]
    fn test_heap_with_threshold() {
        let heap = GcHeap::with_threshold(4096);
        let stats = heap.stats();
        assert_eq!(stats.gc_threshold, 4096);
    }

    #[test]
    fn test_heap_default() {
        let heap = GcHeap::default();
        let stats = heap.stats();
        assert_eq!(stats.gc_threshold, 1024 * 1024);
    }

    #[test]
    fn test_heap_alloc_single() {
        let mut heap = GcHeap::new();
        let ptr = heap.alloc(100, 1);
        assert!(!ptr.is_null());

        let stats = heap.stats();
        assert_eq!(stats.objects_count, 1);
        assert!(stats.bytes_allocated > 0);
    }

    #[test]
    fn test_heap_alloc_multiple() {
        let mut heap = GcHeap::new();
        let ptr1 = heap.alloc(100, 1);
        let ptr2 = heap.alloc(200, 2);
        let ptr3 = heap.alloc(300, 3);

        assert_ne!(ptr1, ptr2);
        assert_ne!(ptr2, ptr3);
        assert_ne!(ptr1, ptr3);

        let stats = heap.stats();
        assert_eq!(stats.objects_count, 3);
    }

    #[test]
    fn test_heap_alloc_zero_size() {
        let mut heap = GcHeap::new();
        let ptr = heap.alloc(0, 0);
        assert!(!ptr.is_null());
        assert_eq!(heap.stats().objects_count, 1);
    }

    // === Root management tests ===

    #[test]
    fn test_add_root() {
        let mut heap = GcHeap::new();
        let ptr = heap.alloc(100, 1) as usize;
        heap.add_root(ptr);

        // Root should protect from collection
        heap.collect();
        assert_eq!(heap.stats().objects_count, 1);
    }

    #[test]
    fn test_add_root_null() {
        let mut heap = GcHeap::new();
        heap.add_root(0); // Should be a no-op
    }

    #[test]
    fn test_remove_root() {
        let mut heap = GcHeap::new();
        let ptr = heap.alloc(100, 1) as usize;
        heap.add_root(ptr);
        heap.remove_root(ptr);

        heap.collect();
        assert_eq!(heap.stats().objects_count, 0);
    }

    #[test]
    fn test_remove_nonexistent_root() {
        let mut heap = GcHeap::new();
        heap.remove_root(12345); // Should not panic
    }

    // === Mark and sweep tests ===

    #[test]
    fn test_collect_frees_unreachable() {
        let mut heap = GcHeap::new();
        heap.alloc(100, 1);
        heap.alloc(200, 2);

        assert_eq!(heap.stats().objects_count, 2);
        heap.collect();
        assert_eq!(heap.stats().objects_count, 0);
        assert_eq!(heap.stats().objects_freed, 2);
    }

    #[test]
    fn test_collect_preserves_rooted() {
        let mut heap = GcHeap::new();
        let ptr1 = heap.alloc(100, 1) as usize;
        heap.alloc(200, 2); // unreachable
        heap.add_root(ptr1);

        heap.collect();
        assert_eq!(heap.stats().objects_count, 1);
        assert_eq!(heap.stats().objects_freed, 1);
    }

    #[test]
    fn test_force_collect() {
        let mut heap = GcHeap::new();
        heap.alloc(100, 1);
        heap.force_collect();
        assert_eq!(heap.stats().collections, 1);
    }

    #[test]
    fn test_multiple_collections() {
        let mut heap = GcHeap::new();

        // Round 1: allocate and collect
        heap.alloc(100, 1);
        heap.collect();
        assert_eq!(heap.stats().collections, 1);
        assert_eq!(heap.stats().objects_freed, 1);

        // Round 2: allocate and collect again
        let ptr = heap.alloc(200, 2) as usize;
        heap.add_root(ptr);
        heap.alloc(300, 3);
        heap.collect();
        assert_eq!(heap.stats().collections, 2);
        assert_eq!(heap.stats().objects_freed, 2);
        assert_eq!(heap.stats().objects_count, 1);
    }

    #[test]
    fn test_collection_updates_bytes_allocated() {
        let mut heap = GcHeap::new();
        heap.alloc(100, 1);
        let before = heap.stats().bytes_allocated;
        assert!(before > 0);

        heap.collect();
        let after = heap.stats().bytes_allocated;
        assert_eq!(after, 0);
    }

    #[test]
    fn test_set_threshold() {
        let mut heap = GcHeap::new();
        heap.set_threshold(2048);
        assert_eq!(heap.stats().gc_threshold, 2048);
    }

    #[test]
    fn test_auto_gc_on_threshold() {
        let mut heap = GcHeap::with_threshold(500);
        // Allocate enough to exceed threshold
        for _ in 0..10 {
            heap.alloc(100, 1);
        }
        // Auto-GC should have been triggered
        assert!(heap.stats().collections > 0);
    }

    #[test]
    fn test_stats_accuracy() {
        let mut heap = GcHeap::with_threshold(1024 * 1024);
        let ptr1 = heap.alloc(100, 1) as usize;
        let ptr2 = heap.alloc(200, 2) as usize;
        heap.add_root(ptr1);
        heap.add_root(ptr2);

        let stats = heap.stats();
        assert_eq!(stats.objects_count, 2);
        assert_eq!(stats.collections, 0);
        assert_eq!(stats.objects_freed, 0);
        let header_size = std::mem::size_of::<GcObjectHeader>();
        assert_eq!(stats.bytes_allocated, 100 + 200 + 2 * header_size);
    }

    // === Stress test ===

    #[test]
    fn test_stress_alloc_collect_cycles() {
        let mut heap = GcHeap::with_threshold(2000);
        let mut rooted = Vec::new();

        for i in 0..100 {
            let ptr = heap.alloc(64, i as u32) as usize;
            if i % 5 == 0 {
                heap.add_root(ptr);
                rooted.push(ptr);
            }
        }

        heap.collect();

        // All rooted objects should survive
        assert_eq!(heap.stats().objects_count, rooted.len());
    }

    #[test]
    fn test_repeated_root_add_remove() {
        let mut heap = GcHeap::new();
        let ptr = heap.alloc(100, 1) as usize;

        // Add and remove root multiple times
        for _ in 0..10 {
            heap.add_root(ptr);
            heap.remove_root(ptr);
        }

        // Object should be unreachable
        heap.collect();
        assert_eq!(heap.stats().objects_count, 0);
    }
}
