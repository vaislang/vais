//! GC Benchmarks for Vais Generational Garbage Collector
//!
//! Measures allocation throughput, minor/major GC latency, and promotion behavior
//! under various workloads and tuning configurations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use vais_gc::{GenGcConfig, GenerationalGc};

/// Benchmark: allocation-intensive workload (many short-lived objects)
fn bench_allocation_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_allocation_throughput");

    for count in [1000, 5000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("young_alloc", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let mut gc = GenerationalGc::new();
                    for i in 0..count {
                        let ptr = gc.alloc(black_box(64), i as u32);
                        black_box(ptr);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: minor GC with varying young generation sizes
fn bench_minor_gc(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_minor_collection");

    for young_count in [100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("minor_gc", young_count),
            &young_count,
            |b, &young_count| {
                b.iter(|| {
                    let config = GenGcConfig {
                        // Set high threshold so we control when GC happens
                        young_threshold: 1024 * 1024 * 64,
                        ..GenGcConfig::default()
                    };
                    let mut gc = GenerationalGc::with_config(config);

                    // Allocate objects, keep some as roots
                    let mut ptrs = Vec::new();
                    for i in 0..young_count {
                        let ptr = gc.alloc(64, i as u32) as usize;
                        if i % 3 == 0 {
                            // Keep ~33% as roots (reachable)
                            gc.add_root(ptr);
                        }
                        ptrs.push(ptr);
                    }

                    // Trigger minor GC
                    gc.collect_minor();
                    black_box(gc.get_stats());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: major GC with mixed young+old generation
fn bench_major_gc(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_major_collection");

    for total_count in [200, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("major_gc", total_count),
            &total_count,
            |b, &total_count| {
                b.iter(|| {
                    let config = GenGcConfig {
                        young_threshold: 1024 * 1024 * 64,
                        old_threshold: 1024 * 1024 * 64,
                        promotion_age: 1, // Promote after 1 minor GC
                        ..GenGcConfig::default()
                    };
                    let mut gc = GenerationalGc::with_config(config);

                    // Phase 1: Allocate and root objects
                    for i in 0..total_count {
                        let ptr = gc.alloc(64, i as u32) as usize;
                        gc.add_root(ptr);
                    }

                    // Phase 2: Minor GC to promote rooted objects to old gen
                    gc.collect_minor();

                    // Phase 3: Allocate more young objects (unreachable)
                    for i in 0..total_count / 2 {
                        gc.alloc(64, (total_count + i) as u32);
                    }

                    // Phase 4: Major GC (collects both generations)
                    gc.collect_major();
                    black_box(gc.get_stats());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: promotion behavior under sustained allocation
fn bench_promotion(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_promotion");

    for promotion_age in [1u8, 3, 5] {
        group.bench_with_input(
            BenchmarkId::new("promotion_age", promotion_age),
            &promotion_age,
            |b, &promotion_age| {
                b.iter(|| {
                    let config = GenGcConfig {
                        young_threshold: 4096, // Small threshold to trigger frequent minor GCs
                        old_threshold: 1024 * 1024 * 64,
                        promotion_age,
                        ..GenGcConfig::default()
                    };
                    let mut gc = GenerationalGc::with_config(config);

                    // Allocate 500 objects, keep all rooted so they get promoted
                    let mut ptrs = Vec::new();
                    for i in 0..500 {
                        let ptr = gc.alloc(64, i as u32) as usize;
                        gc.add_root(ptr);
                        ptrs.push(ptr);
                    }

                    let stats = gc.get_stats();
                    black_box(stats.total_promoted);
                    black_box(stats.minor_collections);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: tuning presets comparison (low-latency vs throughput vs balanced)
fn bench_tuning_presets(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_tuning_presets");

    let presets = vec![
        (
            "low_latency",
            GenGcConfig {
                young_threshold: 64 * 1024,     // 64 KB
                old_threshold: 2 * 1024 * 1024, // 2 MB
                promotion_age: 2,
                ..GenGcConfig::default()
            },
        ),
        (
            "throughput",
            GenGcConfig {
                young_threshold: 1024 * 1024,    // 1 MB
                old_threshold: 16 * 1024 * 1024, // 16 MB
                promotion_age: 5,
                ..GenGcConfig::default()
            },
        ),
        ("balanced", GenGcConfig::default()),
    ];

    for (name, config) in &presets {
        group.bench_with_input(
            BenchmarkId::new("mixed_workload", name),
            config,
            |b, config: &GenGcConfig| {
                b.iter(|| {
                    let mut gc = GenerationalGc::with_config(config.clone());

                    // Simulate realistic workload: mix of short-lived and long-lived objects
                    let mut roots = Vec::new();
                    for i in 0..2000 {
                        let size = if i % 10 == 0 { 256 } else { 64 };
                        let ptr = gc.alloc(size, i as u32) as usize;

                        // 20% of objects are long-lived (rooted)
                        if i % 5 == 0 {
                            gc.add_root(ptr);
                            roots.push(ptr);
                        }
                    }

                    let stats = gc.get_stats();
                    black_box((
                        stats.minor_collections,
                        stats.major_collections,
                        stats.total_promoted,
                    ));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: write barrier overhead
fn bench_write_barrier(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_write_barrier");

    for barrier_count in [100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("barriers", barrier_count),
            &barrier_count,
            |b, &barrier_count| {
                b.iter(|| {
                    let config = GenGcConfig {
                        young_threshold: 1024 * 1024 * 64,
                        old_threshold: 1024 * 1024 * 64,
                        promotion_age: 1,
                        ..GenGcConfig::default()
                    };
                    let mut gc = GenerationalGc::with_config(config);

                    // Create old generation objects
                    let mut old_ptrs = Vec::new();
                    for i in 0..100 {
                        let ptr = gc.alloc(64, i as u32) as usize;
                        gc.add_root(ptr);
                        old_ptrs.push(ptr);
                    }
                    gc.collect_minor(); // Promote to old gen

                    // Create young generation objects
                    let mut young_ptrs = Vec::new();
                    for i in 0..barrier_count {
                        let ptr = gc.alloc(64, (100 + i) as u32) as usize;
                        young_ptrs.push(ptr);
                    }

                    // Trigger write barriers (old → young)
                    for (i, &young_ptr) in young_ptrs.iter().enumerate() {
                        let old_ptr = old_ptrs[i % old_ptrs.len()];
                        gc.write_barrier(old_ptr, 0, young_ptr);
                    }

                    black_box(gc.get_stats());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    gc_benches,
    bench_allocation_throughput,
    bench_minor_gc,
    bench_major_gc,
    bench_promotion,
    bench_tuning_presets,
    bench_write_barrier,
);
criterion_main!(gc_benches);
