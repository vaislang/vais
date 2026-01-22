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
                self.bytes_allocated = self.bytes_allocated.saturating_sub(
                    obj.header.size + std::mem::size_of::<GcObjectHeader>()
                );
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
