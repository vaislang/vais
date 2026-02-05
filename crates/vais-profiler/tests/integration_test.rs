use std::time::Duration;
use vais_profiler::reporter::{CompactReport, ProfileStats, TextReport};
use vais_profiler::{Profiler, ProfilerConfig, ProfilerMode};

#[test]
fn test_end_to_end_profiling() {
    let config = ProfilerConfig {
        mode: ProfilerMode::All,
        max_samples: 10000,
        ..ProfilerConfig::default()
    };

    let profiler = Profiler::new(config);

    profiler.start().unwrap();

    for i in 0..1000 {
        profiler.record_sample("main", 0x1000 + i);
    }

    for i in 0..500 {
        profiler.record_sample("foo", 0x2000 + i);
    }

    for i in 0..300 {
        profiler.record_sample("bar", 0x3000 + i);
    }

    for i in 0..100 {
        profiler.record_allocation(1024, 0x10000 + i * 1024);
    }

    for i in 0..50 {
        profiler.record_deallocation(0x10000 + i * 1024);
    }

    profiler.record_call("main", "foo");
    profiler.record_call("main", "bar");
    profiler.record_call("foo", "baz");

    std::thread::sleep(Duration::from_millis(100));

    profiler.stop().unwrap();

    assert_eq!(profiler.get_sample_count(), 1800);
    assert_eq!(profiler.get_total_allocations(), 100);
    assert_eq!(profiler.get_total_allocated_bytes(), 102400);
    assert_eq!(profiler.get_current_allocated_bytes(), 51200);

    let snapshot = profiler.snapshot();
    assert!(snapshot.duration.is_some());
    assert!(snapshot.duration.unwrap() >= Duration::from_millis(100));

    assert_eq!(snapshot.samples.len(), 3);
    assert_eq!(snapshot.samples[0].0, "main");
    assert_eq!(snapshot.samples[0].1, 1000);
    assert_eq!(snapshot.samples[1].0, "foo");
    assert_eq!(snapshot.samples[1].1, 500);

    assert_eq!(snapshot.call_graph.len(), 3);
}

#[test]
fn test_text_report_generation() {
    let profiler = Profiler::default();
    profiler.start().unwrap();

    profiler.record_sample("compute", 0x1000);
    profiler.record_sample("compute", 0x1100);
    profiler.record_sample("render", 0x2000);

    profiler.record_allocation(4096, 0x10000);
    profiler.record_call("main", "compute");

    profiler.stop().unwrap();

    let snapshot = profiler.snapshot();
    let report = TextReport::new(snapshot);
    let text = report.generate();

    assert!(text.contains("Vais Performance Profile"));
    assert!(text.contains("CPU Profile"));
    assert!(text.contains("Memory Profile"));
    assert!(text.contains("Call Graph"));
    assert!(text.contains("compute"));
    assert!(text.contains("render"));
}

#[test]
fn test_profile_stats_generation() {
    let profiler = Profiler::default();
    profiler.start().unwrap();

    for _ in 0..100 {
        profiler.record_sample("main", 0x1000);
    }

    for _ in 0..50 {
        profiler.record_sample("foo", 0x2000);
    }

    profiler.record_allocation(1024, 0x10000);

    profiler.stop().unwrap();

    let snapshot = profiler.snapshot();
    let stats = ProfileStats::from_snapshot(&snapshot);

    assert_eq!(stats.total_samples, 150);
    assert_eq!(stats.hot_functions.len(), 2);
    assert_eq!(stats.hot_functions[0].name, "main");
    assert_eq!(stats.hot_functions[0].samples, 100);
    assert!((stats.hot_functions[0].percentage - 66.66).abs() < 0.1);

    let json = stats.to_json().unwrap();
    assert!(json.contains("\"duration_secs\""));
    assert!(json.contains("\"total_samples\""));
    assert!(json.contains("\"hot_functions\""));
}

#[test]
fn test_compact_report() {
    let profiler = Profiler::default();
    profiler.start().unwrap();

    profiler.record_sample("main", 0x1000);
    profiler.record_allocation(1048576, 0x10000);

    std::thread::sleep(Duration::from_millis(10));
    profiler.stop().unwrap();

    let snapshot = profiler.snapshot();
    let report = CompactReport::new(snapshot);
    let text = report.generate();

    assert!(text.contains("Duration:"));
    assert!(text.contains("Samples:"));
    assert!(text.contains("Memory:"));
    assert!(text.contains("Hottest: main"));
}

#[cfg(feature = "flamegraph")]
#[test]
fn test_flamegraph_data() {
    use vais_profiler::reporter::FlameGraphData;

    let profiler = Profiler::default();
    profiler.start().unwrap();

    profiler.record_sample("main", 0x1000);
    profiler.record_sample("foo", 0x2000);
    profiler.record_call("main", "foo");

    profiler.stop().unwrap();

    let snapshot = profiler.snapshot();
    let flamegraph = FlameGraphData::new(snapshot);
    let folded = flamegraph.generate_folded();

    assert!(folded.contains("main"));
    assert!(folded.contains("foo"));
    assert!(folded.contains("main;foo"));
}

#[test]
fn test_sampling_mode() {
    let config = ProfilerConfig {
        mode: ProfilerMode::Sampling,
        track_memory: false,
        build_call_graph: false,
        ..ProfilerConfig::default()
    };

    let profiler = Profiler::new(config);
    profiler.start().unwrap();

    profiler.record_sample("main", 0x1000);
    profiler.record_allocation(100, 0x1000);
    profiler.record_call("main", "foo");

    profiler.stop().unwrap();

    assert_eq!(profiler.get_sample_count(), 1);
}

#[test]
fn test_memory_mode() {
    let config = ProfilerConfig {
        mode: ProfilerMode::Memory,
        ..ProfilerConfig::default()
    };

    let profiler = Profiler::new(config);
    profiler.start().unwrap();

    profiler.record_allocation(100, 0x1000);
    profiler.record_allocation(200, 0x2000);
    profiler.record_allocation(300, 0x3000);

    assert_eq!(profiler.get_peak_allocated_bytes(), 600);

    profiler.record_deallocation(0x2000);
    assert_eq!(profiler.get_current_allocated_bytes(), 400);
    assert_eq!(profiler.get_peak_allocated_bytes(), 600);

    profiler.stop().unwrap();
}

#[test]
fn test_concurrent_profiling() {
    use std::sync::Arc;
    use std::thread;

    let profiler = Arc::new(Profiler::default());
    profiler.start().unwrap();

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let profiler = Arc::clone(&profiler);
            thread::spawn(move || {
                for j in 0..100 {
                    profiler.record_sample(&format!("thread_{}", i), 0x1000 + j);
                    profiler.record_allocation(64, 0x10000 + i * 1000 + j);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    profiler.stop().unwrap();

    assert_eq!(profiler.get_sample_count(), 400);
    assert_eq!(profiler.get_total_allocations(), 400);
}

#[test]
fn test_max_samples_limit() {
    let config = ProfilerConfig {
        max_samples: 100,
        ..ProfilerConfig::default()
    };

    let profiler = Profiler::new(config);
    profiler.start().unwrap();

    for i in 0..200 {
        profiler.record_sample("main", 0x1000 + i);
    }

    profiler.stop().unwrap();

    assert_eq!(profiler.get_sample_count(), 100);
}

#[test]
fn test_profiler_reset_on_restart() {
    let profiler = Profiler::default();

    profiler.start().unwrap();
    profiler.record_sample("main", 0x1000);
    profiler.record_allocation(100, 0x1000);
    profiler.stop().unwrap();

    assert_eq!(profiler.get_sample_count(), 1);
    assert_eq!(profiler.get_total_allocations(), 1);

    profiler.start().unwrap();
    assert_eq!(profiler.get_sample_count(), 0);
    assert_eq!(profiler.get_total_allocations(), 0);
    profiler.stop().unwrap();
}

#[test]
fn test_call_graph_analysis() {
    let profiler = Profiler::default();
    profiler.start().unwrap();

    profiler.record_call("main", "init");
    profiler.record_call("main", "process");
    profiler.record_call("main", "cleanup");
    profiler.record_call("process", "compute");
    profiler.record_call("process", "render");
    profiler.record_call("compute", "gpu_kernel");

    profiler.stop().unwrap();

    let snapshot = profiler.snapshot();
    assert_eq!(snapshot.call_graph.len(), 6);

    let main_calls: Vec<_> = snapshot
        .call_graph
        .iter()
        .filter(|(caller, _, _)| caller == "main")
        .collect();
    assert_eq!(main_calls.len(), 3);
}
