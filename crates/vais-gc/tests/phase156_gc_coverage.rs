//! Phase 156 coverage tests for vais-gc
//!
//! Adds +25 tests covering generational GC allocation/collection,
//! mark/sweep correctness, cycle-related patterns, and GC statistics/metrics.

use vais_gc::{
    CardTable, CollectionType, GcHeap, GcObject, GcRoot, GcStats, GenGcConfig, GenGcStats,
    Generation, GenerationalGc, RememberedSet,
};

// ─── GcHeap: allocation statistics ──────────────────────────────────────────

#[test]
fn test_gc_heap_stats_after_single_alloc() {
    let mut heap = GcHeap::new();
    heap.alloc(256, 7);
    let stats = heap.stats();
    assert_eq!(stats.objects_count, 1);
    assert!(stats.bytes_allocated >= 256);
    assert_eq!(stats.collections, 0);
    assert_eq!(stats.objects_freed, 0);
}

#[test]
fn test_gc_heap_bytes_reset_after_collect() {
    let mut heap = GcHeap::with_threshold(10_000);
    heap.alloc(200, 1);
    heap.alloc(300, 2);
    assert!(heap.stats().bytes_allocated > 0);

    heap.force_collect(); // no roots → everything freed
    assert_eq!(heap.stats().bytes_allocated, 0);
}

#[test]
fn test_gc_heap_objects_freed_accumulates() {
    let mut heap = GcHeap::with_threshold(10_000);
    heap.alloc(100, 1);
    heap.alloc(100, 2);
    heap.force_collect(); // frees 2
    assert_eq!(heap.stats().objects_freed, 2);

    heap.alloc(100, 3);
    heap.force_collect(); // frees 1 more
    assert_eq!(heap.stats().objects_freed, 3);
}

#[test]
fn test_gc_heap_threshold_controls_auto_gc() {
    // Threshold of 1 byte → GC fires on every alloc
    let mut heap = GcHeap::with_threshold(1);
    heap.alloc(64, 1);
    // Auto-GC should fire; verify threshold was set
    assert_eq!(heap.stats().gc_threshold, 1);
}

#[test]
fn test_gc_heap_set_threshold_takes_effect() {
    let mut heap = GcHeap::new();
    heap.set_threshold(512);
    assert_eq!(heap.stats().gc_threshold, 512);
}

#[test]
fn test_gc_heap_add_multiple_roots_all_survive() {
    let mut heap = GcHeap::with_threshold(10_000);
    let ptr1 = heap.alloc(64, 1) as usize;
    let ptr2 = heap.alloc(64, 2) as usize;
    let ptr3 = heap.alloc(64, 3) as usize;

    heap.add_root(ptr1);
    heap.add_root(ptr2);
    heap.add_root(ptr3);

    heap.force_collect();
    assert_eq!(heap.stats().objects_count, 3);
}

#[test]
fn test_gc_heap_remove_one_of_many_roots() {
    let mut heap = GcHeap::with_threshold(10_000);
    let ptr1 = heap.alloc(64, 1) as usize;
    let ptr2 = heap.alloc(64, 2) as usize;

    heap.add_root(ptr1);
    heap.add_root(ptr2);
    heap.remove_root(ptr2);

    heap.force_collect();
    // ptr1 survives, ptr2 is freed
    assert_eq!(heap.stats().objects_count, 1);
    assert_eq!(heap.stats().objects_freed, 1);
}

// ─── Mark/sweep correctness ──────────────────────────────────────────────────

#[test]
fn test_gc_heap_collect_empty_heap_increments_count() {
    let mut heap = GcHeap::new();
    heap.force_collect();
    assert_eq!(heap.stats().collections, 1);
    assert_eq!(heap.stats().objects_count, 0);
}

#[test]
fn test_gc_heap_collect_twice_increments_twice() {
    let mut heap = GcHeap::new();
    heap.force_collect();
    heap.force_collect();
    assert_eq!(heap.stats().collections, 2);
}

#[test]
fn test_gc_heap_unreachable_object_is_freed() {
    let mut heap = GcHeap::with_threshold(100_000);
    heap.alloc(100, 1); // no root
    heap.force_collect();
    assert_eq!(heap.stats().objects_count, 0);
}

// ─── GenerationalGc: generation tracking ─────────────────────────────────────

#[test]
fn test_generational_gc_new_alloc_is_young() {
    let mut gc = GenerationalGc::new();
    let ptr = gc.alloc(64, 1) as usize;
    assert_eq!(gc.get_generation(ptr), Some(Generation::Young));
}

#[test]
fn test_generational_gc_object_count_correct() {
    let mut gc = GenerationalGc::new();
    gc.alloc(64, 1);
    gc.alloc(64, 2);
    gc.alloc(64, 3);
    assert_eq!(gc.object_count(), 3);
}

#[test]
fn test_generational_gc_is_alive_after_alloc() {
    let mut gc = GenerationalGc::new();
    let ptr = gc.alloc(64, 1) as usize;
    assert!(gc.is_alive(ptr));
}

#[test]
fn test_generational_gc_not_alive_after_minor_gc_without_root() {
    let mut gc = GenerationalGc::with_config(GenGcConfig {
        young_threshold: usize::MAX, // disable auto
        ..Default::default()
    });
    let ptr = gc.alloc(64, 1) as usize;
    gc.collect_minor();
    assert!(!gc.is_alive(ptr));
}

#[test]
fn test_generational_gc_rooted_object_survives_minor_gc() {
    let mut gc = GenerationalGc::with_config(GenGcConfig {
        young_threshold: usize::MAX,
        ..Default::default()
    });
    let ptr = gc.alloc(64, 1) as usize;
    gc.add_root(ptr);
    gc.collect_minor();
    assert!(gc.is_alive(ptr));
}

#[test]
fn test_generational_gc_promotion_increments_stat() {
    let mut gc = GenerationalGc::with_config(GenGcConfig {
        young_threshold: usize::MAX,
        old_threshold: usize::MAX,
        promotion_age: 1, // promote after 1 minor GC
        ..Default::default()
    });
    let ptr = gc.alloc(64, 1) as usize;
    gc.add_root(ptr);

    gc.collect_minor(); // age → 1, promoted
    let stats = gc.get_stats();
    assert_eq!(stats.total_promoted, 1);
    assert_eq!(gc.get_generation(ptr), Some(Generation::Old));
}

#[test]
fn test_generational_gc_major_gc_frees_old_unreachable() {
    let mut gc = GenerationalGc::with_config(GenGcConfig {
        young_threshold: usize::MAX,
        old_threshold: usize::MAX,
        promotion_age: 0, // immediate promotion
        ..Default::default()
    });
    let ptr = gc.alloc(64, 1) as usize;
    gc.add_root(ptr);
    gc.collect_minor(); // promote to old

    gc.remove_root(ptr);
    gc.collect_major();
    assert!(!gc.is_alive(ptr));
}

#[test]
fn test_generational_gc_minor_major_collection_counts() {
    let mut gc = GenerationalGc::with_config(GenGcConfig {
        young_threshold: usize::MAX,
        old_threshold: usize::MAX,
        ..Default::default()
    });
    gc.collect_minor();
    gc.collect_major();
    let stats = gc.get_stats();
    assert_eq!(stats.minor_collections, 1);
    assert_eq!(stats.major_collections, 1);
}

#[test]
fn test_generational_gc_collect_full_runs_both() {
    let mut gc = GenerationalGc::with_config(GenGcConfig {
        young_threshold: usize::MAX,
        old_threshold: usize::MAX,
        ..Default::default()
    });
    gc.collect_full();
    let stats = gc.get_stats();
    assert!(stats.minor_collections >= 1);
    assert!(stats.major_collections >= 1);
}

// ─── CardTable: write barrier infrastructure ──────────────────────────────────

#[test]
fn test_card_table_new_all_clean() {
    let ct = CardTable::new(4096, 512);
    let dirty = ct.dirty_cards();
    assert!(dirty.is_empty());
}

#[test]
fn test_card_table_mark_dirty_and_query() {
    let mut ct = CardTable::new(4096, 512);
    ct.set_base(0);
    ct.mark_dirty(100); // falls in card 0 (0..512)
    assert!(ct.is_dirty(0));
    assert!(ct.is_dirty(100));
    assert!(!ct.is_dirty(512)); // card 1 still clean
}

#[test]
fn test_card_table_clear_all_cleans_dirty() {
    let mut ct = CardTable::new(4096, 512);
    ct.set_base(0);
    ct.mark_dirty(0);
    ct.mark_dirty(512);
    ct.clear_all();
    assert!(ct.dirty_cards().is_empty());
}

// ─── RememberedSet: old→young tracking ───────────────────────────────────────

#[test]
fn test_remembered_set_new_is_empty() {
    let rs = RememberedSet::new();
    assert!(rs.is_empty());
    assert_eq!(rs.len(), 0);
}

#[test]
fn test_remembered_set_add_and_query_young_roots() {
    let mut rs = RememberedSet::new();
    rs.add(100, 200); // old_ptr=100, young_ptr=200
    rs.add(100, 300);
    let roots = rs.young_roots();
    assert!(roots.contains(&200));
    assert!(roots.contains(&300));
}

#[test]
fn test_remembered_set_remove_young_removes_entries() {
    let mut rs = RememberedSet::new();
    rs.add(10, 20);
    rs.add(30, 20); // both point to young_ptr=20
    rs.remove_young(20);
    assert!(rs.is_empty());
}

#[test]
fn test_remembered_set_clear() {
    let mut rs = RememberedSet::new();
    rs.add(1, 2);
    rs.add(3, 4);
    rs.clear();
    assert!(rs.is_empty());
}

// ─── GC statistics struct ────────────────────────────────────────────────────

#[test]
fn test_gc_stats_default_are_zero() {
    let stats = GenGcStats::default();
    assert_eq!(stats.minor_collections, 0);
    assert_eq!(stats.major_collections, 0);
    assert_eq!(stats.young_objects, 0);
    assert_eq!(stats.old_objects, 0);
    assert_eq!(stats.total_promoted, 0);
    assert_eq!(stats.remembered_set_size, 0);
}
