use std::ffi::CString;
use vais_profiler::ffi::*;

#[test]
fn test_ffi_profiler_lifecycle() {
    let profiler = vais_profiler_create(std::ptr::null());
    assert!(!profiler.is_null());

    assert!(!vais_profiler_is_running(profiler));
    assert!(vais_profiler_start(profiler));
    assert!(vais_profiler_is_running(profiler));
    assert!(vais_profiler_stop(profiler));
    assert!(!vais_profiler_is_running(profiler));

    vais_profiler_destroy(profiler);
}

#[test]
fn test_ffi_double_start() {
    let profiler = vais_profiler_create(std::ptr::null());
    assert!(vais_profiler_start(profiler));
    assert!(!vais_profiler_start(profiler));
    vais_profiler_destroy(profiler);
}

#[test]
fn test_ffi_record_samples() {
    let profiler = vais_profiler_create(std::ptr::null());
    vais_profiler_start(profiler);

    let func1 = CString::new("main").unwrap();
    let func2 = CString::new("foo").unwrap();

    vais_profiler_record_sample(profiler, func1.as_ptr(), 0x1000);
    vais_profiler_record_sample(profiler, func2.as_ptr(), 0x2000);
    vais_profiler_record_sample(profiler, func1.as_ptr(), 0x1100);

    let stats = vais_profiler_get_stats(profiler);
    assert_eq!(stats.sample_count, 3);

    vais_profiler_stop(profiler);
    vais_profiler_destroy(profiler);
}

#[test]
fn test_ffi_memory_tracking() {
    let profiler = vais_profiler_create(std::ptr::null());
    vais_profiler_start(profiler);

    vais_profiler_record_allocation(profiler, 1024, 0x1000);
    vais_profiler_record_allocation(profiler, 2048, 0x2000);
    vais_profiler_record_allocation(profiler, 512, 0x3000);

    let stats = vais_profiler_get_stats(profiler);
    assert_eq!(stats.total_allocations, 3);
    assert_eq!(stats.total_allocated_bytes, 3584);
    assert_eq!(stats.current_allocated_bytes, 3584);

    vais_profiler_record_deallocation(profiler, 0x2000);

    let stats = vais_profiler_get_stats(profiler);
    assert_eq!(stats.current_allocated_bytes, 1536);
    assert_eq!(stats.peak_allocated_bytes, 3584);

    vais_profiler_stop(profiler);
    vais_profiler_destroy(profiler);
}

#[test]
fn test_ffi_call_graph() {
    let profiler = vais_profiler_create(std::ptr::null());
    vais_profiler_start(profiler);

    let main = CString::new("main").unwrap();
    let foo = CString::new("foo").unwrap();
    let bar = CString::new("bar").unwrap();

    vais_profiler_record_call(profiler, main.as_ptr(), foo.as_ptr());
    vais_profiler_record_call(profiler, main.as_ptr(), bar.as_ptr());
    vais_profiler_record_call(profiler, foo.as_ptr(), bar.as_ptr());

    let stats = vais_profiler_get_stats(profiler);
    assert_eq!(stats.call_graph_edges, 3);

    vais_profiler_stop(profiler);
    vais_profiler_destroy(profiler);
}

#[test]
fn test_ffi_custom_config() {
    let config = VaisProfilerConfig {
        sample_interval_ms: 10,
        track_memory: true,
        build_call_graph: false,
        max_samples: 5000,
    };

    let profiler = vais_profiler_create(&config as *const _);
    assert!(!profiler.is_null());

    vais_profiler_start(profiler);

    let func = CString::new("test").unwrap();
    vais_profiler_record_sample(profiler, func.as_ptr(), 0x1000);
    vais_profiler_record_allocation(profiler, 100, 0x1000);

    let main = CString::new("main").unwrap();
    let foo = CString::new("foo").unwrap();
    vais_profiler_record_call(profiler, main.as_ptr(), foo.as_ptr());

    let stats = vais_profiler_get_stats(profiler);
    assert_eq!(stats.sample_count, 1);
    assert_eq!(stats.total_allocations, 1);

    vais_profiler_stop(profiler);
    vais_profiler_destroy(profiler);
}

#[test]
fn test_ffi_global_profiler_all() {
    // All global profiler tests combined into one to avoid race conditions
    // (global state is shared across test threads)

    // Part 0: Init
    vais_profiler_global_destroy();
    assert!(vais_profiler_global_init(std::ptr::null()));
    assert!(!vais_profiler_global_init(std::ptr::null()));
    vais_profiler_global_destroy();

    // Part 1: Lifecycle
    vais_profiler_global_destroy();
    vais_profiler_global_init(std::ptr::null());
    assert!(vais_profiler_global_start());
    assert!(vais_profiler_global_stop());
    vais_profiler_global_destroy();

    // Part 2: Record samples
    vais_profiler_global_init(std::ptr::null());
    vais_profiler_global_start();

    let func1 = CString::new("main").unwrap();
    let func2 = CString::new("foo").unwrap();

    vais_profiler_global_record_sample(func1.as_ptr(), 0x1000);
    vais_profiler_global_record_sample(func2.as_ptr(), 0x2000);
    vais_profiler_global_record_sample(func1.as_ptr(), 0x1100);

    let stats = vais_profiler_global_get_stats();
    assert_eq!(stats.sample_count, 3);

    vais_profiler_global_stop();
    vais_profiler_global_destroy();

    // Part 3: Memory tracking
    vais_profiler_global_init(std::ptr::null());
    vais_profiler_global_start();

    vais_profiler_global_record_allocation(100, 0x1000);
    vais_profiler_global_record_allocation(200, 0x2000);

    let stats = vais_profiler_global_get_stats();
    assert_eq!(stats.total_allocations, 2);
    assert_eq!(stats.total_allocated_bytes, 300);

    vais_profiler_global_record_deallocation(0x1000);

    let stats = vais_profiler_global_get_stats();
    assert_eq!(stats.current_allocated_bytes, 200);

    vais_profiler_global_stop();
    vais_profiler_global_destroy();

    // Part 4: Call graph
    vais_profiler_global_init(std::ptr::null());
    vais_profiler_global_start();

    let main_fn = CString::new("main").unwrap();
    let foo_fn = CString::new("foo").unwrap();

    vais_profiler_global_record_call(main_fn.as_ptr(), foo_fn.as_ptr());
    vais_profiler_global_record_call(main_fn.as_ptr(), foo_fn.as_ptr());

    let stats = vais_profiler_global_get_stats();
    assert_eq!(stats.call_graph_edges, 1);

    vais_profiler_global_stop();
    vais_profiler_global_destroy();
}

#[test]
fn test_ffi_null_safety() {
    assert!(!vais_profiler_start(std::ptr::null_mut()));
    assert!(!vais_profiler_stop(std::ptr::null_mut()));
    assert!(!vais_profiler_is_running(std::ptr::null_mut()));

    vais_profiler_record_sample(std::ptr::null_mut(), std::ptr::null(), 0);
    vais_profiler_record_allocation(std::ptr::null_mut(), 0, 0);
    vais_profiler_record_deallocation(std::ptr::null_mut(), 0);
    vais_profiler_record_call(std::ptr::null_mut(), std::ptr::null(), std::ptr::null());

    let stats = vais_profiler_get_stats(std::ptr::null_mut());
    assert_eq!(stats.sample_count, 0);
    assert_eq!(stats.total_allocations, 0);
}

#[test]
fn test_ffi_stats_structure() {
    let profiler = vais_profiler_create(std::ptr::null());
    vais_profiler_start(profiler);

    let func = CString::new("test").unwrap();
    for _ in 0..10 {
        vais_profiler_record_sample(profiler, func.as_ptr(), 0x1000);
    }

    for i in 0..5 {
        vais_profiler_record_allocation(profiler, 100, 0x1000 + i);
    }

    let main = CString::new("main").unwrap();
    let foo = CString::new("foo").unwrap();
    vais_profiler_record_call(profiler, main.as_ptr(), foo.as_ptr());

    let stats = vais_profiler_get_stats(profiler);
    assert_eq!(stats.sample_count, 10);
    assert_eq!(stats.total_allocations, 5);
    assert_eq!(stats.total_allocated_bytes, 500);
    assert_eq!(stats.current_allocated_bytes, 500);
    assert_eq!(stats.peak_allocated_bytes, 500);
    assert_eq!(stats.call_graph_edges, 1);

    vais_profiler_stop(profiler);
    vais_profiler_destroy(profiler);
}

#[test]
fn test_ffi_multiple_profilers() {
    let profiler1 = vais_profiler_create(std::ptr::null());
    let profiler2 = vais_profiler_create(std::ptr::null());

    vais_profiler_start(profiler1);
    vais_profiler_start(profiler2);

    let func = CString::new("test").unwrap();
    vais_profiler_record_sample(profiler1, func.as_ptr(), 0x1000);
    vais_profiler_record_sample(profiler2, func.as_ptr(), 0x2000);
    vais_profiler_record_sample(profiler2, func.as_ptr(), 0x2100);

    let stats1 = vais_profiler_get_stats(profiler1);
    let stats2 = vais_profiler_get_stats(profiler2);

    assert_eq!(stats1.sample_count, 1);
    assert_eq!(stats2.sample_count, 2);

    vais_profiler_stop(profiler1);
    vais_profiler_stop(profiler2);
    vais_profiler_destroy(profiler1);
    vais_profiler_destroy(profiler2);
}
