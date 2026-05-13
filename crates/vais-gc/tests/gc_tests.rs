//! Unit tests for vais-gc

use vais_gc::GcHeap;

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
    use vais_gc::{
        vais_gc_alloc, vais_gc_bytes_allocated, vais_gc_collect, vais_gc_init,
        vais_gc_objects_count,
    };

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

// ============================================
// New Integration Tests (10)
// ============================================

// ConcurrentGc Integration Tests (3)

#[test]
fn test_concurrent_gc_complete_lifecycle() {
    use vais_gc::ConcurrentGc;

    let gc = ConcurrentGc::new();

    // Allocate objects
    let ptr1 = gc.alloc(100, 1) as usize;
    let ptr2 = gc.alloc(200, 2) as usize;
    let ptr3 = gc.alloc(300, 3) as usize;

    assert_eq!(gc.object_count(), 3);

    // Root only ptr1 and ptr3
    gc.add_root(ptr1);
    gc.add_root(ptr3);

    // Run full collection cycle
    gc.collect_sync();

    // ptr1 and ptr3 should survive, ptr2 should be collected
    assert!(gc.is_alive(ptr1));
    assert!(!gc.is_alive(ptr2));
    assert!(gc.is_alive(ptr3));

    let stats = gc.get_stats();
    assert_eq!(stats.collections, 1);
    assert_eq!(stats.last_freed, 1);
    assert_eq!(stats.objects_count, 2);

    // Remove a root and collect again
    gc.remove_root(ptr3);
    gc.collect_sync();

    assert!(gc.is_alive(ptr1));
    assert!(!gc.is_alive(ptr3));
    assert_eq!(gc.object_count(), 1);
}

#[test]
fn test_concurrent_gc_custom_config() {
    use vais_gc::{ConcurrentGc, ConcurrentGcConfig};

    let config = ConcurrentGcConfig {
        gc_threshold: 2000,
        target_pause_ns: 500_000,
        max_marking_steps: 100,
        concurrent_sweep: true,
        write_barrier: true,
    };

    let gc = ConcurrentGc::with_config(config.clone());

    // Allocate and verify behavior with custom config
    let ptr1 = gc.alloc(100, 1);
    assert!(!ptr1.is_null());

    let stats = gc.get_stats();
    assert_eq!(stats.objects_count, 1);
    assert!(stats.bytes_allocated >= 100);

    // Manually trigger collection to verify config is working
    gc.collect_sync();

    let stats = gc.get_stats();
    assert_eq!(stats.collections, 1);

    // Allocate a rooted object and verify it survives
    let ptr2 = gc.alloc(200, 2) as usize;
    gc.add_root(ptr2);
    gc.collect_sync();

    assert!(gc.is_alive(ptr2));
    let stats = gc.get_stats();
    assert_eq!(stats.collections, 2);
}

#[test]
fn test_concurrent_gc_all_phases() {
    use vais_gc::{ConcurrentGc, GcPhase};

    let gc = ConcurrentGc::new();

    // Initially idle
    assert_eq!(gc.get_phase(), GcPhase::Idle);

    // Allocate and root an object
    let ptr = gc.alloc(100, 1) as usize;
    gc.add_root(ptr);

    // Run full collection and check phase transitions
    gc.collect_sync();

    // After collection, should return to idle
    assert_eq!(gc.get_phase(), GcPhase::Idle);

    // Object should still be alive
    assert!(gc.is_alive(ptr));

    let stats = gc.get_stats();
    assert_eq!(stats.collections, 1);
    assert!(stats.total_pause_time_ns > 0);
    assert!(stats.marking_steps > 0);
}

// GenerationalGc Integration Tests (3)

#[test]
fn test_generational_gc_promotion_flow() {
    use vais_gc::{GenGcConfig, Generation, GenerationalGc};

    let config = GenGcConfig {
        young_threshold: 1024 * 1024, // Don't auto-trigger
        old_threshold: 4 * 1024 * 1024,
        promotion_age: 2,
        card_size: 512,
        max_heap_size: 64 * 1024 * 1024,
    };

    let mut gc = GenerationalGc::with_config(config);

    // Allocate in young generation
    let ptr1 = gc.alloc(100, 1) as usize;
    let ptr2 = gc.alloc(200, 2) as usize;

    assert_eq!(gc.get_generation(ptr1), Some(Generation::Young));
    assert_eq!(gc.get_generation(ptr2), Some(Generation::Young));

    // Root ptr1 only
    gc.add_root(ptr1);

    // First minor GC - age becomes 1
    gc.collect_minor();
    assert_eq!(gc.get_generation(ptr1), Some(Generation::Young));
    assert!(!gc.is_alive(ptr2)); // ptr2 collected

    // Second minor GC - age becomes 2, triggers promotion
    gc.collect_minor();
    assert_eq!(gc.get_generation(ptr1), Some(Generation::Old));

    let stats = gc.get_stats();
    assert_eq!(stats.total_promoted, 1);
    assert_eq!(stats.old_objects, 1);
    assert_eq!(stats.young_objects, 0);
    assert_eq!(stats.minor_collections, 2);
}

#[test]
fn test_generational_gc_custom_config() {
    use vais_gc::{GenGcConfig, GenerationalGc};

    let config = GenGcConfig {
        young_threshold: 128 * 1024,
        old_threshold: 2 * 1024 * 1024,
        promotion_age: 5,
        card_size: 256,
        max_heap_size: 32 * 1024 * 1024,
    };

    let mut gc = GenerationalGc::with_config(config);

    // Allocate objects
    let ptr = gc.alloc(100, 1);
    assert!(!ptr.is_null());

    let stats = gc.get_stats();
    assert_eq!(stats.young_objects, 1);
    assert_eq!(stats.old_objects, 0);

    // Change thresholds and promotion age
    gc.set_young_threshold(64 * 1024);
    gc.set_old_threshold(1024 * 1024);
    gc.set_promotion_age(3);

    // Verify behavior with new settings
    // Allocate object and root it
    let ptr2 = gc.alloc(100, 2) as usize;
    gc.add_root(ptr2);

    // With promotion_age=3, need 3 minor GCs to promote
    gc.collect_minor();
    gc.collect_minor();
    gc.collect_minor();

    let stats = gc.get_stats();
    // At least one object should have been promoted
    assert!(stats.total_promoted >= 1);
}

#[test]
fn test_generational_gc_remembered_set_integration() {
    use vais_gc::{GenGcConfig, Generation, GenerationalGc};

    let mut gc = GenerationalGc::with_config(GenGcConfig {
        young_threshold: 1024 * 1024,
        old_threshold: 4 * 1024 * 1024,
        promotion_age: 0, // Immediate promotion
        ..Default::default()
    });

    // Create and promote an old object
    let old_ptr = gc.alloc(100, 1) as usize;
    gc.add_root(old_ptr);
    gc.collect_minor(); // Promotes to old

    assert_eq!(gc.get_generation(old_ptr), Some(Generation::Old));

    // Create a young object (not rooted)
    let young_ptr = gc.alloc(200, 2) as usize;
    assert_eq!(gc.get_generation(young_ptr), Some(Generation::Young));

    // Simulate old→young pointer via write barrier
    gc.write_barrier(old_ptr, 0, young_ptr);

    // Minor GC should keep young_ptr alive via remembered set
    gc.collect_minor();
    assert!(gc.is_alive(young_ptr));

    let stats = gc.get_stats();
    assert!(stats.remembered_set_size > 0 || gc.get_generation(young_ptr) == Some(Generation::Old));
}

// GcHeap Advanced Tests (2)

#[test]
fn test_gc_allocator_trait_usage() {
    use vais_gc::GcHeap;

    let mut heap = GcHeap::new();

    // Test GcAllocator trait methods
    let ptr1 = heap.alloc(100, 1);
    let ptr2 = heap.alloc(200, 2);

    assert!(!ptr1.is_null());
    assert!(!ptr2.is_null());

    // Get stats via trait
    let stats = heap.stats();
    assert_eq!(stats.objects_count, 2);
    assert!(stats.bytes_allocated >= 300);
    assert_eq!(stats.collections, 0);

    // Test collection
    heap.collect();
    let stats = heap.stats();
    assert_eq!(stats.collections, 1);
    // Objects should be freed (no roots)
    assert_eq!(stats.objects_count, 0);
}

#[test]
fn test_gc_stats_initial_state() {
    use vais_gc::GcHeap;

    let heap = GcHeap::new();
    let stats = heap.stats();

    // Verify initial state is all zeros
    assert_eq!(stats.bytes_allocated, 0);
    assert_eq!(stats.objects_count, 0);
    assert_eq!(stats.collections, 0);
    assert_eq!(stats.objects_freed, 0);
    assert_eq!(stats.gc_threshold, 1024 * 1024); // Default 1 MB

    // Test with custom threshold
    let heap2 = GcHeap::with_threshold(2048);
    let stats2 = heap2.stats();
    assert_eq!(stats2.gc_threshold, 2048);
    assert_eq!(stats2.bytes_allocated, 0);
    assert_eq!(stats2.objects_count, 0);
}

// FFI Functions Test (1)

#[test]
fn test_generational_gc_ffi_functions() {
    use vais_gc::{
        vais_gen_gc_add_root, vais_gen_gc_alloc, vais_gen_gc_collect_minor, vais_gen_gc_init,
        vais_gen_gc_major_collections, vais_gen_gc_minor_collections, vais_gen_gc_old_objects,
        vais_gen_gc_remove_root, vais_gen_gc_total_promoted, vais_gen_gc_young_objects,
    };

    // Initialize generational GC
    vais_gen_gc_init();

    // Allocate objects
    let ptr1 = vais_gen_gc_alloc(100, 1);
    let ptr2 = vais_gen_gc_alloc(200, 2);

    assert_ne!(ptr1 as usize, 0);
    assert_ne!(ptr2 as usize, 0);

    // Check initial stats
    let young_objs = vais_gen_gc_young_objects();
    assert_eq!(young_objs, 2);

    let old_objs = vais_gen_gc_old_objects();
    assert_eq!(old_objs, 0);

    // Add root for ptr1
    vais_gen_gc_add_root(ptr1 as usize);

    // Minor collection
    vais_gen_gc_collect_minor();

    let minor_colls = vais_gen_gc_minor_collections();
    assert!(minor_colls >= 1);

    // Remove root
    vais_gen_gc_remove_root(ptr1 as usize);

    // Check promoted count - u64 is always >= 0, so just verify it's accessible
    let _promoted = vais_gen_gc_total_promoted();
    let _major_colls = vais_gen_gc_major_collections();
}

// Edge Cases Test (1)

#[test]
fn test_heavy_allocation_stress() {
    use vais_gc::GcHeap;

    let mut heap = GcHeap::with_threshold(50000);

    // Allocate many objects quickly
    let mut roots = Vec::new();
    for i in 0..500 {
        let ptr = heap.alloc(128, i as u32);
        assert!(!ptr.is_null());

        // Keep every 10th object rooted
        if i % 10 == 0 {
            heap.add_root(ptr as usize);
            roots.push(ptr as usize);
        }
    }

    let stats_before = heap.stats();
    assert!(stats_before.objects_count > 0);
    // Should have triggered at least one GC due to threshold
    assert!(stats_before.collections > 0);

    // Force collection
    heap.force_collect();

    let stats_after = heap.stats();
    // Should have performed more collections
    assert!(stats_after.collections > stats_before.collections);
    // Rooted objects should remain
    assert_eq!(stats_after.objects_count, roots.len());
    // Should have freed many objects
    assert!(stats_after.objects_freed > 0);

    // Verify bytes allocated is reasonable
    assert!(stats_after.bytes_allocated > 0);
    assert!(stats_after.bytes_allocated < stats_before.bytes_allocated);
}
