//! Example usage of the large-scale project generator
//!
//! This demonstrates how to use the generator functions from benches/lib.rs
//! to create synthetic Vais projects for benchmarking and testing.
//!
//! ## Usage
//!
//! ```bash
//! # Run this example
//! cargo run --example gen_large_example
//!
//! # Or use in benchmarks
//! cargo bench -p vais-benches
//! ```

use std::fs;
use vais_benches::utils::{
    generate_large_project, generate_multi_module_project, generate_distributed_project,
};

fn main() {
    println!("=== Vais Large-Scale Project Generator ===\n");

    // Example 1: Single-file project with 10K lines
    println!("1. Generating 10K-line single-file project...");
    let project_10k = generate_large_project(10_000);
    let actual_lines_10k = project_10k.lines().count();
    println!("   Generated: {} lines ({} bytes)", actual_lines_10k, project_10k.len());

    // Save to file
    fs::write("benches/fixtures/generated_10k.vais", &project_10k)
        .expect("Failed to write 10K project");
    println!("   Saved to: benches/fixtures/generated_10k.vais\n");

    // Example 2: 50K-line single-file project
    println!("2. Generating 50K-line single-file project...");
    let project_50k = generate_large_project(50_000);
    let actual_lines_50k = project_50k.lines().count();
    println!("   Generated: {} lines ({} bytes)", actual_lines_50k, project_50k.len());

    fs::write("benches/fixtures/generated_50k.vais", &project_50k)
        .expect("Failed to write 50K project");
    println!("   Saved to: benches/fixtures/generated_50k.vais\n");

    // Example 3: 100K-line single-file project
    println!("3. Generating 100K-line single-file project...");
    let project_100k = generate_large_project(100_000);
    let actual_lines_100k = project_100k.lines().count();
    println!("   Generated: {} lines ({} bytes)", actual_lines_100k, project_100k.len());

    fs::write("benches/fixtures/generated_100k.vais", &project_100k)
        .expect("Failed to write 100K project");
    println!("   Saved to: benches/fixtures/generated_100k.vais\n");

    // Example 4: Multi-module project (10 modules, 1K lines each)
    println!("4. Generating multi-module project (10 modules × 1K lines)...");
    let multi_modules = generate_multi_module_project(10, 1_000);
    let total_lines: usize = multi_modules.iter().map(|(_, code)| code.lines().count()).sum();
    println!("   Generated: {} modules, {} total lines", multi_modules.len(), total_lines);

    for (filename, code) in &multi_modules {
        let path = format!("benches/fixtures/multi_module_{}", filename);
        fs::write(&path, code).expect("Failed to write module");
        println!("   Saved: {} ({} lines)", path, code.lines().count());
    }
    println!();

    // Example 5: Distributed project (20K lines across 5 modules)
    println!("5. Generating distributed project (20K lines / 5 modules)...");
    let distributed = generate_distributed_project(20_000, 5);
    let dist_total_lines: usize = distributed.iter().map(|(_, code)| code.lines().count()).sum();
    println!("   Generated: {} modules, {} total lines", distributed.len(), dist_total_lines);

    for (filename, code) in &distributed {
        let path = format!("benches/fixtures/distributed_{}", filename);
        fs::write(&path, code).expect("Failed to write module");
        println!("   Saved: {} ({} lines)", path, code.lines().count());
    }
    println!();

    // Summary
    println!("=== Summary ===");
    println!("Single-file projects:");
    println!("  - 10K:  {} lines", actual_lines_10k);
    println!("  - 50K:  {} lines", actual_lines_50k);
    println!("  - 100K: {} lines", actual_lines_100k);
    println!("\nMulti-module projects:");
    println!("  - 10 modules: {} total lines", total_lines);
    println!("  - 5 modules:  {} total lines", dist_total_lines);
    println!("\nAll files saved to benches/fixtures/");
}
