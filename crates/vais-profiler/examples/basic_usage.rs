use vais_profiler::{Profiler, ProfilerConfig, ProfilerMode};
use vais_profiler::reporter::{TextReport, ProfileStats, CompactReport};
use std::time::Duration;

fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fn main() {
    let mut config = ProfilerConfig::default();
    config.mode = ProfilerMode::All;
    config.max_samples = 100_000;

    let profiler = Profiler::new(config);

    println!("Starting profiler...");
    profiler.start().unwrap();

    // Simulate some work with profiling
    for i in 0..1000 {
        profiler.record_sample("main_loop", 0x1000 + i);
    }

    // Simulate function calls
    profiler.record_call("main", "fibonacci");
    for _ in 0..10 {
        let _ = fibonacci(20);
        profiler.record_sample("fibonacci", 0x2000);
    }

    profiler.record_call("main", "process_data");
    for i in 0..500 {
        profiler.record_sample("process_data", 0x3000 + i);
    }

    // Simulate memory allocations
    for i in 0..100 {
        profiler.record_allocation(1024 + i * 64, 0x10000 + i * 1024);
    }

    // Simulate some deallocations
    for i in 0..50 {
        profiler.record_deallocation(0x10000 + i * 1024);
    }

    std::thread::sleep(Duration::from_millis(100));

    println!("Stopping profiler...");
    profiler.stop().unwrap();

    // Get snapshot
    let snapshot = profiler.snapshot();

    // Generate different reports
    println!("\n{}", "=".repeat(70));
    println!("TEXT REPORT");
    println!("{}", "=".repeat(70));
    let text_report = TextReport::new(snapshot.clone());
    println!("{}", text_report);

    println!("{}", "=".repeat(70));
    println!("COMPACT REPORT");
    println!("{}", "=".repeat(70));
    let compact_report = CompactReport::new(snapshot.clone());
    println!("{}", compact_report);

    println!("\n{}", "=".repeat(70));
    println!("JSON STATS");
    println!("{}", "=".repeat(70));
    let stats = ProfileStats::from_snapshot(&snapshot);
    println!("{}", stats.to_json().unwrap());

    // Display basic statistics
    println!("\n{}", "=".repeat(70));
    println!("SUMMARY");
    println!("{}", "=".repeat(70));
    println!("Total samples:          {}", profiler.get_sample_count());
    println!("Total allocations:      {}", profiler.get_total_allocations());
    println!(
        "Current memory:         {:.2} KB",
        profiler.get_current_allocated_bytes() as f64 / 1024.0
    );
    println!(
        "Peak memory:            {:.2} KB",
        profiler.get_peak_allocated_bytes() as f64 / 1024.0
    );
    if let Some(duration) = profiler.get_duration() {
        println!("Duration:               {:.3}s", duration.as_secs_f64());
    }
}
