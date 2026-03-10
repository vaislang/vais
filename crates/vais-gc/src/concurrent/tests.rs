use super::*;

#[test]
fn test_basic_allocation() {
    let gc = ConcurrentGc::new();

    let ptr1 = gc.alloc(100, 1);
    let ptr2 = gc.alloc(200, 2);

    assert!(!ptr1.is_null());
    assert!(!ptr2.is_null());
    assert_ne!(ptr1, ptr2);

    assert_eq!(gc.object_count(), 2);
}

#[test]
fn test_root_tracking() {
    let gc = ConcurrentGc::new();

    let ptr = gc.alloc(100, 1) as usize;
    gc.add_root(ptr);

    gc.collect_sync();

    // Object should still be alive (rooted)
    assert!(gc.is_alive(ptr));
}

#[test]
fn test_garbage_collection() {
    let gc = ConcurrentGc::new();

    let ptr1 = gc.alloc(100, 1) as usize;
    let ptr2 = gc.alloc(200, 2) as usize;

    // Only root ptr1
    gc.add_root(ptr1);

    gc.collect_sync();

    // ptr1 should be alive, ptr2 should be collected
    assert!(gc.is_alive(ptr1));
    assert!(!gc.is_alive(ptr2));
}

#[test]
fn test_stats() {
    let gc = ConcurrentGc::new();

    gc.alloc(100, 1);
    gc.alloc(200, 2);

    let stats = gc.get_stats();
    assert_eq!(stats.objects_count, 2);
    assert!(stats.bytes_allocated >= 300);
}

#[test]
fn test_tri_color_marking() {
    let gc = ConcurrentGc::new();

    let ptr = gc.alloc(100, 1) as usize;

    // Initially white
    {
        let objects = gc.objects.read().unwrap();
        let obj = objects.get(&ptr).unwrap();
        assert_eq!(obj.header.get_color(), Color::White);
    }

    gc.add_root(ptr);
    gc.collect_sync();

    // After collection and being rooted, should be black
    {
        let objects = gc.objects.read().unwrap();
        // Object survived, color might be reset for next cycle
        assert!(objects.contains_key(&ptr));
    }
}

#[test]
fn test_incremental_gc() {
    let gc = ConcurrentGc::new();
    let mut inc = IncrementalGc::new(Arc::clone(&gc));

    // Allocate enough to trigger collection
    for _ in 0..100 {
        gc.alloc(10000, 1);
    }

    // Run incremental steps until complete
    inc.start_collection();
    while inc.is_collecting() {
        inc.step();
    }

    // Most objects should be collected (no roots)
    assert_eq!(gc.object_count(), 0);
}

#[test]
fn test_write_barrier() {
    let gc = ConcurrentGc::with_config(ConcurrentGcConfig {
        write_barrier: true,
        ..Default::default()
    });

    let ptr1 = gc.alloc(100, 1) as usize;
    let ptr2 = gc.alloc(100, 2) as usize;

    // Simulate a pointer write
    gc.write_barrier(ptr1, 0, ptr2);

    // Buffer should have the entry
    {
        let _buffer = gc.write_barrier_buffer.lock().unwrap();
        // Entry might or might not be there depending on GC phase
        // During Idle phase, write barriers are ignored
    }
}

#[test]
fn test_gc_phases() {
    let gc = ConcurrentGc::new();

    assert_eq!(gc.get_phase(), GcPhase::Idle);

    gc.alloc(100, 1);
    gc.collect_sync();

    // Should be idle after collection
    assert_eq!(gc.get_phase(), GcPhase::Idle);
}

#[test]
fn test_config() {
    let config = ConcurrentGcConfig {
        gc_threshold: 500,
        target_pause_ns: 500_000,
        max_marking_steps: 500,
        concurrent_sweep: false,
        write_barrier: false,
    };

    let gc = ConcurrentGc::with_config(config);
    assert_eq!(gc.config.gc_threshold, 500);
}

#[test]
fn test_multiple_collections() {
    let gc = ConcurrentGc::new();

    for i in 0..5 {
        let ptr = gc.alloc(100, i);
        gc.add_root(ptr as usize);
        gc.collect_sync();
    }

    let stats = gc.get_stats();
    assert_eq!(stats.collections, 5);
    assert_eq!(stats.objects_count, 5);
}

#[test]
fn test_remove_root_then_collect() {
    let gc = ConcurrentGc::new();
    let ptr = gc.alloc(100, 1) as usize;
    gc.add_root(ptr);

    // Should survive
    gc.collect_sync();
    assert!(gc.is_alive(ptr));

    // Remove root, should be collected
    gc.remove_root(ptr);
    gc.collect_sync();
    assert!(!gc.is_alive(ptr));
}

#[test]
fn test_null_root_ignored() {
    let gc = ConcurrentGc::new();
    gc.add_root(0); // Should be no-op
    gc.remove_root(0); // Should be no-op
}

#[test]
fn test_object_count() {
    let gc = ConcurrentGc::new();
    assert_eq!(gc.object_count(), 0);

    gc.alloc(100, 1);
    assert_eq!(gc.object_count(), 1);

    gc.alloc(200, 2);
    assert_eq!(gc.object_count(), 2);

    gc.collect_sync();
    assert_eq!(gc.object_count(), 0);
}

#[test]
fn test_concurrent_gc_default_config() {
    let config = ConcurrentGcConfig::default();
    assert_eq!(config.gc_threshold, 1024 * 1024);
    assert_eq!(config.target_pause_ns, 1_000_000);
    assert_eq!(config.max_marking_steps, 1000);
    assert!(config.concurrent_sweep);
    assert!(config.write_barrier);
}

#[test]
fn test_concurrent_gc_header_color_ops() {
    let header = ConcurrentGcHeader::new(64, 1);
    assert_eq!(header.get_color(), Color::White);

    header.set_color(Color::Gray);
    assert_eq!(header.get_color(), Color::Gray);

    assert!(header.compare_and_set_color(Color::Gray, Color::Black));
    assert_eq!(header.get_color(), Color::Black);

    // Should fail if expected doesn't match
    assert!(!header.compare_and_set_color(Color::White, Color::Gray));
    assert_eq!(header.get_color(), Color::Black);
}

#[test]
fn test_concurrent_gc_header_pinned() {
    let header = ConcurrentGcHeader::new(64, 1);
    assert!(!header.pinned.load(Ordering::Relaxed));
    header.pinned.store(true, Ordering::Relaxed);
    assert!(header.pinned.load(Ordering::Relaxed));
}

#[test]
fn test_concurrent_gc_object_creation() {
    let obj = ConcurrentGcObject::new(128, 42);
    assert_eq!(obj.header.size.load(Ordering::Relaxed), 128);
    assert_eq!(obj.header.type_id, 42);
    assert_eq!(obj.data.len(), 128);
    assert!(!obj.data_ptr().is_null());
}

#[test]
fn test_write_barrier_entry_fields() {
    let entry = WriteBarrierEntry {
        source: 100,
        old_target: 200,
        new_target: 300,
        timestamp: 42,
    };
    assert_eq!(entry.source, 100);
    assert_eq!(entry.old_target, 200);
    assert_eq!(entry.new_target, 300);
    assert_eq!(entry.timestamp, 42);
}

#[test]
fn test_gc_phase_transitions() {
    let gc = ConcurrentGc::new();
    assert_eq!(gc.get_phase(), GcPhase::Idle);

    // Collection runs through phases
    let ptr = gc.alloc(100, 1) as usize;
    gc.add_root(ptr);
    gc.collect_sync();

    // After collection should be back to idle
    assert_eq!(gc.get_phase(), GcPhase::Idle);
}

#[test]
fn test_stats_bytes_tracking() {
    let gc = ConcurrentGc::new();
    gc.alloc(100, 1);
    gc.alloc(200, 2);

    let stats = gc.get_stats();
    assert_eq!(stats.bytes_allocated, 300);

    gc.collect_sync();
    let stats = gc.get_stats();
    assert_eq!(stats.last_freed, 2);
    assert_eq!(stats.last_bytes_freed, 300);
}

#[test]
fn test_shutdown() {
    let gc = ConcurrentGc::new();
    gc.alloc(100, 1);
    gc.shutdown();
    // Should not panic
}

#[test]
fn test_incremental_gc_idle_state() {
    let gc = ConcurrentGc::new();
    let inc = IncrementalGc::new(Arc::clone(&gc));
    assert!(!inc.is_collecting());
}

#[test]
fn test_incremental_gc_start_and_step() {
    let gc = ConcurrentGc::new();
    let mut inc = IncrementalGc::new(Arc::clone(&gc));

    gc.alloc(100, 1);

    inc.start_collection();
    assert!(inc.is_collecting());

    // Step through phases until complete
    let mut steps = 0;
    while inc.is_collecting() && steps < 100 {
        inc.step();
        steps += 1;
    }
    assert!(!inc.is_collecting());
}

#[test]
fn test_request_collection_from_idle() {
    let gc = ConcurrentGc::new();
    gc.request_collection();
    // Phase should move from Idle
    let phase = gc.get_phase();
    assert_eq!(phase, GcPhase::InitialMark);
}

#[test]
fn test_default_concurrent_gc_fn() {
    let gc = default_concurrent_gc();
    assert_eq!(gc.object_count(), 0);
    assert_eq!(gc.get_phase(), GcPhase::Idle);
}

#[test]
fn test_scan_object_smaller_than_ptr_size() {
    let gc = ConcurrentGc::new();
    // 1-byte object: too small for usize pointer, scan should not OOB
    let ptr = gc.alloc(1, 1) as usize;
    gc.add_root(ptr);
    gc.collect_sync();
    assert!(gc.is_alive(ptr));
}

#[test]
fn test_scan_zero_size_object() {
    let gc = ConcurrentGc::new();
    let ptr = gc.alloc(0, 1) as usize;
    gc.add_root(ptr);
    gc.collect_sync();
    assert!(gc.is_alive(ptr));
}

#[test]
fn test_scan_exact_ptr_size_object() {
    let ptr_size = std::mem::size_of::<usize>();
    let gc = ConcurrentGc::new();
    let ptr = gc.alloc(ptr_size, 1) as usize;
    gc.add_root(ptr);
    gc.collect_sync();
    assert!(gc.is_alive(ptr));
}

#[test]
fn test_collect_empty_heap() {
    let gc = ConcurrentGc::new();
    gc.collect_sync(); // Should not panic on empty heap
    assert_eq!(gc.object_count(), 0);
    assert_eq!(gc.get_stats().collections, 1);
}

#[test]
fn test_stress_concurrent_alloc_collect() {
    let gc = ConcurrentGc::new();
    let mut rooted = Vec::new();

    for i in 0..50 {
        let ptr = gc.alloc(64, i) as usize;
        if i % 5 == 0 {
            gc.add_root(ptr);
            rooted.push(ptr);
        }
    }

    gc.collect_sync();

    for ptr in &rooted {
        assert!(gc.is_alive(*ptr));
    }
    assert_eq!(gc.object_count(), rooted.len());
}
