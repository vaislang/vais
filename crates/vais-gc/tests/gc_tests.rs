//! Unit tests for vais-gc

use vais_gc::{GcHeap, GcStats};

#[test]
fn test_gc_heap_creation() {
    let heap = GcHeap::new();
    let stats = heap.stats();

    assert_eq!(stats.bytes_allocated, 0);
    assert_eq!(stats.objects_count, 0);
    assert_eq!(stats.collections, 0);
}

#[test]
fn test_basic_allocation() {
    let mut heap = GcHeap::new();

    // Allocate 100 bytes
    let ptr1 = heap.alloc(100, 1);
    assert!(!ptr1.is_null());

    let stats = heap.stats();
    assert!(stats.bytes_allocated >= 100);
    assert_eq!(stats.objects_count, 1);
}

#[test]
fn test_multiple_allocations() {
    let mut heap = GcHeap::new();

    let ptr1 = heap.alloc(100, 1);
    let ptr2 = heap.alloc(200, 2);
    let ptr3 = heap.alloc(300, 3);

    assert!(!ptr1.is_null());
    assert!(!ptr2.is_null());
    assert!(!ptr3.is_null());

    let stats = heap.stats();
    assert_eq!(stats.objects_count, 3);
}

#[test]
fn test_gc_collection() {
    let mut heap = GcHeap::with_threshold(10000);

    // Allocate without roots - should be collected
    let _ptr1 = heap.alloc(100, 1);
    let _ptr2 = heap.alloc(200, 2);

    let before = heap.stats();
    assert_eq!(before.objects_count, 2);

    // Force collection
    heap.force_collect();

    let after = heap.stats();
    assert_eq!(after.collections, 1);
    // Objects without roots should be freed
    assert_eq!(after.objects_count, 0);
}

#[test]
fn test_root_preservation() {
    let mut heap = GcHeap::new();

    // Allocate and register as root
    let ptr = heap.alloc(100, 1) as usize;
    heap.add_root(ptr);

    // Force collection
    heap.force_collect();

    let stats = heap.stats();
    // Object should survive because it's rooted
    assert_eq!(stats.objects_count, 1);

    // Remove root and collect again
    heap.remove_root(ptr);
    heap.force_collect();

    let stats = heap.stats();
    // Object should now be freed
    assert_eq!(stats.objects_count, 0);
}

#[test]
fn test_threshold_behavior() {
    let mut heap = GcHeap::with_threshold(1000);

    let stats = heap.stats();
    assert_eq!(stats.gc_threshold, 1000);

    // Set new threshold
    heap.set_threshold(2000);
    let stats = heap.stats();
    assert_eq!(stats.gc_threshold, 2000);
}

#[test]
fn test_large_allocation() {
    let mut heap = GcHeap::new();

    // Allocate 1 MB
    let ptr = heap.alloc(1024 * 1024, 1);
    assert!(!ptr.is_null());

    let stats = heap.stats();
    assert!(stats.bytes_allocated >= 1024 * 1024);
}

#[test]
fn test_stress_allocation() {
    let mut heap = GcHeap::with_threshold(10000);

    // Allocate many small objects
    for i in 0..100 {
        let ptr = heap.alloc(256, i);
        assert!(!ptr.is_null());
    }

    let stats = heap.stats();
    assert!(stats.objects_count > 0);
    assert!(stats.bytes_allocated > 0);
}

#[test]
fn test_ffi_integration() {
    use vais_gc::{vais_gc_init, vais_gc_alloc, vais_gc_collect,
                  vais_gc_bytes_allocated, vais_gc_objects_count};

    // Initialize GC
    vais_gc_init();

    // Allocate
    let ptr1 = vais_gc_alloc(100, 1);
    let ptr2 = vais_gc_alloc(200, 2);

    assert_ne!(ptr1 as usize, 0);
    assert_ne!(ptr2 as usize, 0);

    // Check stats
    let bytes = vais_gc_bytes_allocated();
    let objects = vais_gc_objects_count();

    assert!(bytes >= 300);
    assert_eq!(objects, 2);

    // Force collection
    vais_gc_collect();
}
