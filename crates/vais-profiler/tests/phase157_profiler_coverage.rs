//! Phase 157: Additional unit tests for vais-profiler
//!
//! Covers: collector (SampleCollector, MemoryTracker, CallGraph),
//!         reporter (TextReport, CompactReport, ProfileStats),
//!         lib (ProfilerConfig variants, Profiler edge cases)

use std::time::Duration;
use vais_profiler::{
    collector::{CallGraph, MemoryTracker, SampleCollector},
    reporter::{CompactReport, FunctionStats, MemoryStatsReport, ProfileStats, TextReport},
    MemoryStats, ProfileSnapshot, Profiler, ProfilerConfig, ProfilerError, ProfilerMode,
};

// ============================================================
// Helper
// ============================================================

fn empty_snapshot() -> ProfileSnapshot {
    ProfileSnapshot {
        duration: None,
        samples: vec![],
        memory_stats: MemoryStats::default(),
        call_graph: vec![],
    }
}

fn full_snapshot() -> ProfileSnapshot {
    ProfileSnapshot {
        duration: Some(Duration::from_millis(500)),
        samples: vec![
            ("hotfn".to_string(), 80),
            ("coldfn".to_string(), 20),
        ],
        memory_stats: MemoryStats {
            total_allocations: 200,
            total_deallocations: 150,
            total_allocated_bytes: 2_097_152, // 2 MB
            current_allocated_bytes: 524_288,  // 0.5 MB
            peak_allocated_bytes: 1_048_576,   // 1 MB
        },
        call_graph: vec![
            ("main".to_string(), "hotfn".to_string(), 80),
            ("main".to_string(), "coldfn".to_string(), 20),
        ],
    }
}

// ============================================================
// SampleCollector
// ============================================================

#[test]
fn test_sample_collector_empty() {
    let collector = SampleCollector::new(100);
    assert_eq!(collector.sample_count(), 0);
    assert!(collector.get_function_samples().is_empty());
}

#[test]
fn test_sample_collector_single_function_multiple_ips() {
    let mut collector = SampleCollector::new(100);
    collector.add_sample("main", 0x1000);
    collector.add_sample("main", 0x1004);
    collector.add_sample("main", 0x1008);
    assert_eq!(collector.sample_count(), 3);
    let samples = collector.get_function_samples();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0], ("main".to_string(), 3));
}

#[test]
fn test_sample_collector_sorted_by_count_descending() {
    let mut collector = SampleCollector::new(1000);
    for _ in 0..10 {
        collector.add_sample("slow", 0x1000);
    }
    for _ in 0..5 {
        collector.add_sample("fast", 0x2000);
    }
    let samples = collector.get_function_samples();
    assert_eq!(samples[0].0, "slow");
    assert_eq!(samples[1].0, "fast");
}

#[test]
fn test_sample_collector_max_samples_boundary() {
    let mut collector = SampleCollector::new(2);
    collector.add_sample("a", 0x1000);
    collector.add_sample("b", 0x2000);
    // This one should be dropped
    collector.add_sample("c", 0x3000);
    assert_eq!(collector.sample_count(), 2);
    let samples = collector.get_function_samples();
    assert_eq!(samples.len(), 2);
}

#[test]
fn test_sample_collector_max_samples_zero() {
    let mut collector = SampleCollector::new(0);
    collector.add_sample("fn", 0x1000);
    assert_eq!(collector.sample_count(), 0);
}

#[test]
fn test_sample_collector_clear() {
    let mut collector = SampleCollector::new(100);
    collector.add_sample("main", 0x1000);
    collector.add_sample("foo", 0x2000);
    collector.clear();
    assert_eq!(collector.sample_count(), 0);
    assert!(collector.get_function_samples().is_empty());
}

#[test]
fn test_sample_collector_hot_functions_top_1() {
    let mut collector = SampleCollector::new(1000);
    for _ in 0..60 {
        collector.add_sample("a", 0x1000);
    }
    for _ in 0..40 {
        collector.add_sample("b", 0x2000);
    }
    let hot = collector.get_hot_functions(1);
    assert_eq!(hot.len(), 1);
    assert_eq!(hot[0].0, "a");
    assert_eq!(hot[0].1, 60);
    assert!((hot[0].2 - 60.0).abs() < 0.01);
}

#[test]
fn test_sample_collector_hot_functions_percentage_sums_to_100() {
    let mut collector = SampleCollector::new(1000);
    for _ in 0..50 {
        collector.add_sample("x", 0x1000);
    }
    for _ in 0..50 {
        collector.add_sample("y", 0x2000);
    }
    let hot = collector.get_hot_functions(2);
    let total_pct: f64 = hot.iter().map(|(_, _, pct)| pct).sum();
    assert!((total_pct - 100.0).abs() < 0.01);
}

#[test]
fn test_sample_collector_hot_functions_more_than_available() {
    let mut collector = SampleCollector::new(100);
    collector.add_sample("only_one", 0x1000);
    let hot = collector.get_hot_functions(10);
    assert_eq!(hot.len(), 1);
}

#[test]
fn test_sample_collector_add_after_clear() {
    let mut collector = SampleCollector::new(100);
    collector.add_sample("fn", 0x1000);
    collector.clear();
    collector.add_sample("fn2", 0x2000);
    assert_eq!(collector.sample_count(), 1);
}

// ============================================================
// MemoryTracker
// ============================================================

#[test]
fn test_memory_tracker_initial_state() {
    let tracker = MemoryTracker::new();
    assert_eq!(tracker.total_allocations(), 0);
    assert_eq!(tracker.total_deallocations(), 0);
    assert_eq!(tracker.total_allocated_bytes(), 0);
    assert_eq!(tracker.current_allocated_bytes(), 0);
    assert_eq!(tracker.peak_allocated_bytes(), 0);
}

#[test]
fn test_memory_tracker_default_equals_new() {
    let a = MemoryTracker::default();
    assert_eq!(a.total_allocations(), 0);
}

#[test]
fn test_memory_tracker_single_allocation() {
    let mut tracker = MemoryTracker::new();
    tracker.record_allocation(512, 0xABCD);
    assert_eq!(tracker.total_allocations(), 1);
    assert_eq!(tracker.total_allocated_bytes(), 512);
    assert_eq!(tracker.current_allocated_bytes(), 512);
    assert_eq!(tracker.peak_allocated_bytes(), 512);
}

#[test]
fn test_memory_tracker_alloc_then_free() {
    let mut tracker = MemoryTracker::new();
    tracker.record_allocation(256, 0x1000);
    tracker.record_deallocation(0x1000);
    assert_eq!(tracker.total_allocations(), 1);
    assert_eq!(tracker.total_deallocations(), 1);
    assert_eq!(tracker.current_allocated_bytes(), 0);
    assert_eq!(tracker.peak_allocated_bytes(), 256);
    assert_eq!(tracker.total_allocated_bytes(), 256);
}

#[test]
fn test_memory_tracker_unknown_deallocation_is_noop() {
    let mut tracker = MemoryTracker::new();
    tracker.record_deallocation(0x9999); // address never allocated
    assert_eq!(tracker.total_deallocations(), 0);
}

#[test]
fn test_memory_tracker_peak_does_not_decrease() {
    let mut tracker = MemoryTracker::new();
    tracker.record_allocation(1000, 0x1000);
    tracker.record_deallocation(0x1000);
    tracker.record_allocation(100, 0x2000);
    assert_eq!(tracker.peak_allocated_bytes(), 1000);
    assert_eq!(tracker.current_allocated_bytes(), 100);
}

#[test]
fn test_memory_tracker_live_allocations_sorted_by_size_desc() {
    let mut tracker = MemoryTracker::new();
    tracker.record_allocation(100, 0x1000);
    tracker.record_allocation(500, 0x2000);
    tracker.record_allocation(50, 0x3000);
    let live = tracker.get_live_allocations();
    assert_eq!(live.len(), 3);
    assert_eq!(live[0].1, 500);
    assert_eq!(live[1].1, 100);
    assert_eq!(live[2].1, 50);
}

#[test]
fn test_memory_tracker_get_stats_matches_fields() {
    let mut tracker = MemoryTracker::new();
    tracker.record_allocation(100, 0x1000);
    tracker.record_allocation(200, 0x2000);
    tracker.record_deallocation(0x1000);
    let stats = tracker.get_stats();
    assert_eq!(stats.total_allocations, 2);
    assert_eq!(stats.total_deallocations, 1);
    assert_eq!(stats.total_allocated_bytes, 300);
    assert_eq!(stats.current_allocated_bytes, 200);
    assert_eq!(stats.peak_allocated_bytes, 300);
}

#[test]
fn test_memory_tracker_clear_resets_all() {
    let mut tracker = MemoryTracker::new();
    tracker.record_allocation(1024, 0x1000);
    tracker.clear();
    assert_eq!(tracker.total_allocations(), 0);
    assert_eq!(tracker.total_allocated_bytes(), 0);
    assert_eq!(tracker.peak_allocated_bytes(), 0);
    assert!(tracker.get_live_allocations().is_empty());
}

// ============================================================
// CallGraph
// ============================================================

#[test]
fn test_call_graph_empty() {
    let graph = CallGraph::new();
    assert!(graph.get_edges().is_empty());
    assert_eq!(graph.get_call_count("main", "foo"), 0);
}

#[test]
fn test_call_graph_default_equals_new() {
    let graph = CallGraph::default();
    assert!(graph.get_edges().is_empty());
}

#[test]
fn test_call_graph_increment_existing_edge() {
    let mut graph = CallGraph::new();
    graph.record_call("main", "foo");
    graph.record_call("main", "foo");
    graph.record_call("main", "foo");
    assert_eq!(graph.get_call_count("main", "foo"), 3);
}

#[test]
fn test_call_graph_distinct_edges() {
    let mut graph = CallGraph::new();
    graph.record_call("a", "b");
    graph.record_call("b", "c");
    graph.record_call("c", "d");
    assert_eq!(graph.get_edges().len(), 3);
}

#[test]
fn test_call_graph_get_callers_empty() {
    let graph = CallGraph::new();
    assert!(graph.get_callers("nobody").is_empty());
}

#[test]
fn test_call_graph_get_callees_empty() {
    let graph = CallGraph::new();
    assert!(graph.get_callees("nobody").is_empty());
}

#[test]
fn test_call_graph_get_callers_correct() {
    let mut graph = CallGraph::new();
    graph.record_call("x", "target");
    graph.record_call("y", "target");
    graph.record_call("z", "other");
    let callers = graph.get_callers("target");
    assert_eq!(callers.len(), 2);
    assert!(callers.iter().any(|(c, _)| c == "x"));
    assert!(callers.iter().any(|(c, _)| c == "y"));
}

#[test]
fn test_call_graph_get_callees_correct() {
    let mut graph = CallGraph::new();
    graph.record_call("main", "alpha");
    graph.record_call("main", "beta");
    graph.record_call("main", "gamma");
    let callees = graph.get_callees("main");
    assert_eq!(callees.len(), 3);
}

#[test]
fn test_call_graph_hot_edges_sorted_descending() {
    let mut graph = CallGraph::new();
    for _ in 0..5 {
        graph.record_call("a", "b");
    }
    for _ in 0..15 {
        graph.record_call("c", "d");
    }
    for _ in 0..10 {
        graph.record_call("e", "f");
    }
    let hot = graph.get_hot_edges(3);
    assert_eq!(hot.len(), 3);
    assert_eq!(hot[0].2, 15);
    assert_eq!(hot[1].2, 10);
    assert_eq!(hot[2].2, 5);
}

#[test]
fn test_call_graph_hot_edges_fewer_than_n() {
    let mut graph = CallGraph::new();
    graph.record_call("a", "b");
    let hot = graph.get_hot_edges(10);
    assert_eq!(hot.len(), 1);
}

#[test]
fn test_call_graph_clear() {
    let mut graph = CallGraph::new();
    graph.record_call("a", "b");
    graph.clear();
    assert!(graph.get_edges().is_empty());
    assert_eq!(graph.get_call_count("a", "b"), 0);
}

// ============================================================
// reporter — TextReport
// ============================================================

#[test]
fn test_text_report_header_present() {
    let report = TextReport::new(empty_snapshot());
    let output = report.generate();
    assert!(output.contains("=== Vais Performance Profile ==="));
}

#[test]
fn test_text_report_no_duration_when_none() {
    let report = TextReport::new(empty_snapshot());
    let output = report.generate();
    assert!(!output.contains("Duration:"));
}

#[test]
fn test_text_report_duration_present_when_some() {
    let snapshot = ProfileSnapshot {
        duration: Some(Duration::from_secs(2)),
        ..empty_snapshot()
    };
    let report = TextReport::new(snapshot);
    let output = report.generate();
    assert!(output.contains("Duration: 2.000s"));
}

#[test]
fn test_text_report_no_samples_message() {
    let report = TextReport::new(empty_snapshot());
    let output = report.generate();
    assert!(output.contains("No samples collected"));
}

#[test]
fn test_text_report_samples_listed() {
    let report = TextReport::new(full_snapshot());
    let output = report.generate();
    assert!(output.contains("hotfn"));
    assert!(output.contains("coldfn"));
    assert!(output.contains("Total samples: 100"));
}

#[test]
fn test_text_report_memory_section() {
    let report = TextReport::new(full_snapshot());
    let output = report.generate();
    assert!(output.contains("--- Memory Profile ---"));
    assert!(output.contains("Total allocations:     200"));
    assert!(output.contains("Total deallocations:   150"));
}

#[test]
fn test_text_report_no_call_graph_message() {
    let report = TextReport::new(empty_snapshot());
    let output = report.generate();
    assert!(output.contains("No call graph data"));
}

#[test]
fn test_text_report_call_graph_present() {
    let report = TextReport::new(full_snapshot());
    let output = report.generate();
    assert!(output.contains("main"));
}

#[test]
fn test_text_report_display_trait() {
    let report = TextReport::new(full_snapshot());
    let display_str = format!("{}", report);
    assert!(!display_str.is_empty());
}

#[test]
fn test_text_report_memory_average_allocation() {
    let snapshot = ProfileSnapshot {
        memory_stats: MemoryStats {
            total_allocations: 4,
            total_allocated_bytes: 400,
            ..MemoryStats::default()
        },
        ..empty_snapshot()
    };
    let report = TextReport::new(snapshot);
    let output = report.generate();
    assert!(output.contains("Average allocation:    100 bytes"));
}

#[test]
fn test_text_report_no_average_when_zero_allocs() {
    let report = TextReport::new(empty_snapshot());
    let output = report.generate();
    assert!(!output.contains("Average allocation:"));
}

// ============================================================
// reporter — CompactReport
// ============================================================

#[test]
fn test_compact_report_basic_format() {
    let report = CompactReport::new(full_snapshot());
    let output = report.generate();
    assert!(output.contains("Samples: 100"));
    assert!(output.contains("Edges: 2"));
}

#[test]
fn test_compact_report_duration() {
    let report = CompactReport::new(full_snapshot());
    let output = report.generate();
    assert!(output.contains("Duration: 0.500s"));
}

#[test]
fn test_compact_report_no_duration_without_timing() {
    let report = CompactReport::new(empty_snapshot());
    let output = report.generate();
    assert!(!output.contains("Duration:"));
}

#[test]
fn test_compact_report_hottest_function() {
    let report = CompactReport::new(full_snapshot());
    let output = report.generate();
    assert!(output.contains("Hottest: hotfn"));
    assert!(output.contains("(80.0%)"));
}

#[test]
fn test_compact_report_no_hottest_when_no_samples() {
    let report = CompactReport::new(empty_snapshot());
    let output = report.generate();
    assert!(!output.contains("Hottest:"));
}

#[test]
fn test_compact_report_memory_peak_mb() {
    let report = CompactReport::new(full_snapshot());
    let output = report.generate();
    assert!(output.contains("Memory: peak"));
    assert!(output.contains("MB"));
}

#[test]
fn test_compact_report_display_trait() {
    let report = CompactReport::new(empty_snapshot());
    let display_str = format!("{}", report);
    assert!(!display_str.is_empty());
}

// ============================================================
// reporter — ProfileStats
// ============================================================

#[test]
fn test_profile_stats_no_duration() {
    let stats = ProfileStats::from_snapshot(&empty_snapshot());
    assert_eq!(stats.duration_secs, 0.0);
    assert_eq!(stats.total_samples, 0);
    assert!(stats.hot_functions.is_empty());
    assert_eq!(stats.call_graph_edges, 0);
}

#[test]
fn test_profile_stats_duration_converted() {
    let stats = ProfileStats::from_snapshot(&full_snapshot());
    assert!((stats.duration_secs - 0.5).abs() < 0.001);
}

#[test]
fn test_profile_stats_hot_functions_max_10() {
    let mut samples = vec![];
    for i in 0..15 {
        samples.push((format!("fn_{}", i), 10 - (i % 10)));
    }
    let snapshot = ProfileSnapshot {
        samples,
        ..empty_snapshot()
    };
    let stats = ProfileStats::from_snapshot(&snapshot);
    assert!(stats.hot_functions.len() <= 10);
}

#[test]
fn test_profile_stats_average_allocation_size() {
    let snapshot = ProfileSnapshot {
        memory_stats: MemoryStats {
            total_allocations: 10,
            total_allocated_bytes: 1000,
            ..MemoryStats::default()
        },
        ..empty_snapshot()
    };
    let stats = ProfileStats::from_snapshot(&snapshot);
    assert_eq!(stats.memory.average_allocation_size, 100);
}

#[test]
fn test_profile_stats_average_allocation_zero_when_no_allocs() {
    let stats = ProfileStats::from_snapshot(&empty_snapshot());
    assert_eq!(stats.memory.average_allocation_size, 0);
}

#[test]
fn test_profile_stats_to_json_valid() {
    let stats = ProfileStats::from_snapshot(&full_snapshot());
    let json = stats.to_json().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["duration_secs"].is_number());
    assert!(parsed["total_samples"].is_number());
    assert!(parsed["hot_functions"].is_array());
    assert!(parsed["memory"].is_object());
    assert!(parsed["call_graph_edges"].is_number());
}

#[test]
fn test_profile_stats_json_contains_memory_fields() {
    let stats = ProfileStats::from_snapshot(&full_snapshot());
    let json = stats.to_json().unwrap();
    assert!(json.contains("total_allocations"));
    assert!(json.contains("peak_allocated_bytes"));
    assert!(json.contains("average_allocation_size"));
}

#[test]
fn test_function_stats_serde_roundtrip() {
    let fs = FunctionStats {
        name: "my_fn".to_string(),
        samples: 42,
        percentage: 33.33,
    };
    let json = serde_json::to_string(&fs).unwrap();
    let parsed: FunctionStats = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "my_fn");
    assert_eq!(parsed.samples, 42);
    assert!((parsed.percentage - 33.33).abs() < 0.001);
}

#[test]
fn test_memory_stats_report_serde_roundtrip() {
    let report = MemoryStatsReport {
        total_allocations: 100,
        total_deallocations: 80,
        total_allocated_bytes: 5000,
        current_allocated_bytes: 1000,
        peak_allocated_bytes: 4000,
        average_allocation_size: 50,
    };
    let json = serde_json::to_string(&report).unwrap();
    let parsed: MemoryStatsReport = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.total_allocations, 100);
    assert_eq!(parsed.average_allocation_size, 50);
}

// ============================================================
// lib — ProfilerConfig & Profiler edge cases
// ============================================================

#[test]
fn test_profiler_config_default() {
    let cfg = ProfilerConfig::default();
    assert_eq!(cfg.mode, ProfilerMode::All);
    assert_eq!(cfg.sample_interval, Duration::from_millis(1));
    assert!(cfg.track_memory);
    assert!(cfg.build_call_graph);
    assert_eq!(cfg.max_samples, 1_000_000);
}

#[test]
fn test_profiler_mode_variants_eq() {
    assert_eq!(ProfilerMode::Sampling, ProfilerMode::Sampling);
    assert_eq!(ProfilerMode::Instrumentation, ProfilerMode::Instrumentation);
    assert_eq!(ProfilerMode::Memory, ProfilerMode::Memory);
    assert_eq!(ProfilerMode::All, ProfilerMode::All);
    assert_ne!(ProfilerMode::Sampling, ProfilerMode::Memory);
}

#[test]
fn test_profiler_error_display_already_running() {
    let e = ProfilerError::AlreadyRunning;
    assert_eq!(e.to_string(), "Profiler already running");
}

#[test]
fn test_profiler_error_display_not_running() {
    let e = ProfilerError::NotRunning;
    assert_eq!(e.to_string(), "Profiler not running");
}

#[test]
fn test_profiler_error_display_collection_error() {
    let e = ProfilerError::CollectionError("oom".to_string());
    assert!(e.to_string().contains("oom"));
}

#[test]
fn test_profiler_error_display_report_error() {
    let e = ProfilerError::ReportError("io error".to_string());
    assert!(e.to_string().contains("io error"));
}

#[test]
fn test_profiler_default() {
    let profiler = Profiler::default();
    assert!(!profiler.is_running());
    assert_eq!(profiler.get_sample_count(), 0);
}

#[test]
fn test_profiler_start_stop_cycle() {
    let profiler = Profiler::default();
    assert!(!profiler.is_running());
    profiler.start().unwrap();
    assert!(profiler.is_running());
    profiler.stop().unwrap();
    assert!(!profiler.is_running());
}

#[test]
fn test_profiler_double_start_returns_error() {
    let profiler = Profiler::default();
    profiler.start().unwrap();
    assert!(matches!(profiler.start(), Err(ProfilerError::AlreadyRunning)));
    profiler.stop().unwrap();
}

#[test]
fn test_profiler_stop_without_start_returns_error() {
    let profiler = Profiler::default();
    assert!(matches!(profiler.stop(), Err(ProfilerError::NotRunning)));
}

#[test]
fn test_profiler_record_sample_only_when_running() {
    let profiler = Profiler::default();
    profiler.record_sample("fn", 0x1000);
    assert_eq!(profiler.get_sample_count(), 0);

    profiler.start().unwrap();
    profiler.record_sample("fn", 0x1000);
    assert_eq!(profiler.get_sample_count(), 1);

    profiler.stop().unwrap();
    profiler.record_sample("fn", 0x1000);
    assert_eq!(profiler.get_sample_count(), 1); // still 1, not running
}

#[test]
fn test_profiler_allocation_not_tracked_when_not_running() {
    let profiler = Profiler::default();
    profiler.record_allocation(100, 0x1000);
    assert_eq!(profiler.get_total_allocations(), 0);
}

#[test]
fn test_profiler_allocation_tracked_when_running() {
    let profiler = Profiler::default();
    profiler.start().unwrap();
    profiler.record_allocation(100, 0x1000);
    profiler.record_allocation(200, 0x2000);
    assert_eq!(profiler.get_total_allocations(), 2);
    assert_eq!(profiler.get_total_allocated_bytes(), 300);
    assert_eq!(profiler.get_current_allocated_bytes(), 300);
    profiler.stop().unwrap();
}

#[test]
fn test_profiler_deallocation_reduces_current() {
    let profiler = Profiler::default();
    profiler.start().unwrap();
    profiler.record_allocation(500, 0xA000);
    profiler.record_deallocation(0xA000);
    assert_eq!(profiler.get_current_allocated_bytes(), 0);
    assert_eq!(profiler.get_peak_allocated_bytes(), 500);
    profiler.stop().unwrap();
}

#[test]
fn test_profiler_no_memory_tracking_when_disabled() {
    let cfg = ProfilerConfig {
        track_memory: false,
        ..ProfilerConfig::default()
    };
    let profiler = Profiler::new(cfg);
    profiler.start().unwrap();
    profiler.record_allocation(100, 0x1000);
    assert_eq!(profiler.get_total_allocations(), 0);
    profiler.stop().unwrap();
}

#[test]
fn test_profiler_no_call_graph_when_disabled() {
    let cfg = ProfilerConfig {
        build_call_graph: false,
        ..ProfilerConfig::default()
    };
    let profiler = Profiler::new(cfg);
    profiler.start().unwrap();
    profiler.record_call("main", "foo");
    profiler.stop().unwrap();
    let snapshot = profiler.snapshot();
    assert!(snapshot.call_graph.is_empty());
}

#[test]
fn test_profiler_get_duration_none_before_start() {
    let profiler = Profiler::default();
    assert!(profiler.get_duration().is_none());
}

#[test]
fn test_profiler_get_duration_available_while_running() {
    let profiler = Profiler::default();
    profiler.start().unwrap();
    std::thread::sleep(Duration::from_millis(5));
    let dur = profiler.get_duration();
    assert!(dur.is_some());
    assert!(dur.unwrap() >= Duration::from_millis(5));
    profiler.stop().unwrap();
}

#[test]
fn test_profiler_get_duration_available_after_stop() {
    let profiler = Profiler::default();
    profiler.start().unwrap();
    std::thread::sleep(Duration::from_millis(5));
    profiler.stop().unwrap();
    let dur = profiler.get_duration();
    assert!(dur.is_some());
}

#[test]
fn test_profiler_snapshot_zero_samples() {
    let profiler = Profiler::default();
    profiler.start().unwrap();
    profiler.stop().unwrap();
    let snapshot = profiler.snapshot();
    assert!(snapshot.samples.is_empty());
    assert!(snapshot.call_graph.is_empty());
}

#[test]
fn test_profiler_snapshot_contains_all_data() {
    let profiler = Profiler::default();
    profiler.start().unwrap();
    profiler.record_sample("main", 0x1000);
    profiler.record_sample("main", 0x1004);
    profiler.record_allocation(256, 0x2000);
    profiler.record_call("main", "helper");
    profiler.stop().unwrap();

    let snapshot = profiler.snapshot();
    assert_eq!(snapshot.samples.len(), 1);
    assert_eq!(snapshot.samples[0].0, "main");
    assert_eq!(snapshot.samples[0].1, 2);
    assert_eq!(snapshot.memory_stats.total_allocations, 1);
    assert_eq!(snapshot.call_graph.len(), 1);
}

#[test]
fn test_profiler_restart_resets_data() {
    let profiler = Profiler::default();
    profiler.start().unwrap();
    profiler.record_sample("fn", 0x1000);
    assert_eq!(profiler.get_sample_count(), 1);
    profiler.stop().unwrap();

    // Restart should clear previous data
    profiler.start().unwrap();
    assert_eq!(profiler.get_sample_count(), 0);
    profiler.stop().unwrap();
}

#[test]
fn test_profiler_sampling_mode_config() {
    let cfg = ProfilerConfig {
        mode: ProfilerMode::Sampling,
        max_samples: 500,
        track_memory: false,
        build_call_graph: false,
        ..ProfilerConfig::default()
    };
    let profiler = Profiler::new(cfg);
    profiler.start().unwrap();
    profiler.record_sample("fn", 0x1);
    assert_eq!(profiler.get_sample_count(), 1);
    profiler.stop().unwrap();
}

#[test]
fn test_profiler_memory_mode_config() {
    let cfg = ProfilerConfig {
        mode: ProfilerMode::Memory,
        track_memory: true,
        build_call_graph: false,
        ..ProfilerConfig::default()
    };
    let profiler = Profiler::new(cfg);
    profiler.start().unwrap();
    profiler.record_allocation(1024, 0xBEEF);
    assert_eq!(profiler.get_total_allocated_bytes(), 1024);
    profiler.stop().unwrap();
}
