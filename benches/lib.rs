//! Vais Benchmark Suite
//!
//! This crate provides benchmarks for measuring compiler and runtime performance.
//!
//! ## Running Benchmarks
//!
//! ```bash
//! # Run all benchmarks
//! cargo bench -p vais-benches
//!
//! # Run specific benchmark
//! cargo bench -p vais-benches --bench compile_bench
//!
//! # Run with HTML report
//! cargo bench -p vais-benches -- --save-baseline current
//! ```

pub mod utils {
    use std::fs;

    /// Load a benchmark fixture file
    pub fn load_fixture(name: &str) -> String {
        let path = format!("benches/fixtures/{}.vais", name);
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {}", path))
    }

    /// Generate synthetic Vais code with N functions
    pub fn generate_code(num_funcs: usize) -> String {
        let mut code = String::new();

        for i in 0..num_funcs {
            code.push_str(&format!(
                "F func{}(x: i64)->i64 = x * {} + {}\n",
                i,
                i % 10,
                i
            ));
        }

        code.push_str("F main()->i64 = func0(42)\n");
        code
    }
}
