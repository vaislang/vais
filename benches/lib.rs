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

    /// Generate a large-scale Vais project with diverse constructs.
    /// Targets a specific line count with a mix of functions, structs, enums, traits, and generics.
    ///
    /// # Arguments
    /// * `target_lines` - Approximate number of lines to generate
    ///
    /// # Returns
    /// A String containing valid Vais source code
    pub fn generate_large_project(target_lines: usize) -> String {
        let mut code = String::with_capacity(target_lines * 50);
        let mut lines = 0;

        // Calculate module distribution
        let items_per_module = 30; // functions + structs + enums + traits
        let module_count = (target_lines / (items_per_module * 5)).max(1);

        for m in 0..module_count {
            if lines >= target_lines {
                break;
            }

            // Module header
            code.push_str(&format!("# Module {} — Large-scale benchmark\n\n", m));
            lines += 2;

            // Generate structs (5 per module)
            for s in 0..5 {
                if lines >= target_lines {
                    break;
                }
                code.push_str(&format!("S Point{}_{}_{} {{\n", m, s, lines));
                code.push_str("    x: i64,\n");
                code.push_str("    y: i64,\n");
                code.push_str("    z: i64\n");
                code.push_str("}\n\n");
                lines += 6;
            }

            // Generate enums (3 per module)
            for e in 0..3 {
                if lines >= target_lines {
                    break;
                }
                code.push_str(&format!("E Result{}_{}_{} {{\n", m, e, lines));
                code.push_str("    Ok(i64),\n");
                code.push_str("    Err(i64),\n");
                code.push_str("    Pending\n");
                code.push_str("}\n\n");
                lines += 6;
            }

            // Generate generic structs (2 per module)
            for g in 0..2 {
                if lines >= target_lines {
                    break;
                }
                code.push_str(&format!("S Container{}_{}_{}<T> {{\n", m, g, lines));
                code.push_str("    value: T,\n");
                code.push_str("    count: i64\n");
                code.push_str("}\n\n");
                lines += 5;
            }

            // Generate functions with varied patterns
            let funcs_per_module = 20;
            for f in 0..funcs_per_module {
                if lines >= target_lines {
                    break;
                }

                match f % 8 {
                    0 => {
                        // Simple arithmetic
                        code.push_str(&format!(
                            "F mod{}_arithmetic_{}(x: i64, y: i64) -> i64 {{\n",
                            m, f
                        ));
                        code.push_str(&format!("    a := x * {} + y\n", f + 1));
                        code.push_str(&format!("    b := a - {} * x\n", f % 7 + 1));
                        code.push_str(&format!("    c := b + {} * y\n", f % 5 + 2));
                        code.push_str("    R a + b + c\n");
                        code.push_str("}\n\n");
                        lines += 7;
                    }
                    1 => {
                        // Recursive factorial-like
                        code.push_str(&format!("F mod{}_recursive_{}(n: i64) -> i64 {{\n", m, f));
                        code.push_str("    I n <= 1 {\n");
                        code.push_str("        R 1\n");
                        code.push_str("    }\n");
                        code.push_str(&format!("    R n * @(n - {})\n", f % 3 + 1));
                        code.push_str("}\n\n");
                        lines += 7;
                    }
                    2 => {
                        // Conditional chain
                        code.push_str(&format!("F mod{}_conditional_{}(x: i64) -> i64 {{\n", m, f));
                        code.push_str(&format!("    I x < {} {{\n", f * 5));
                        code.push_str(&format!("        R x * {}\n", f % 4 + 2));
                        code.push_str(&format!("    }} E I x < {} {{\n", f * 10));
                        code.push_str(&format!("        R x + {}\n", f));
                        code.push_str("    } E {\n");
                        code.push_str("        R x\n");
                        code.push_str("    }\n");
                        code.push_str("}\n\n");
                        lines += 10;
                    }
                    3 => {
                        // Loop with accumulator
                        code.push_str(&format!("F mod{}_loop_{}(n: i64) -> i64 {{\n", m, f));
                        code.push_str("    sum := mut 0\n");
                        code.push_str("    i := mut 0\n");
                        code.push_str("    L {\n");
                        code.push_str("        I i >= n {\n");
                        code.push_str("            B\n");
                        code.push_str("        }\n");
                        code.push_str(&format!("        sum = sum + i * {}\n", f % 6 + 1));
                        code.push_str("        i = i + 1\n");
                        code.push_str("    }\n");
                        code.push_str("    R sum\n");
                        code.push_str("}\n\n");
                        lines += 13;
                    }
                    4 => {
                        // Match expression on simple integer
                        code.push_str(&format!("F mod{}_match_{}(x: i64) -> i64 {{\n", m, f));
                        code.push_str("    M x {\n");
                        code.push_str(&format!("        {} => x * 2,\n", f % 10));
                        code.push_str(&format!("        {} => x * 3,\n", (f + 1) % 10));
                        code.push_str(&format!("        {} => x * 4,\n", (f + 2) % 10));
                        code.push_str("        _ => x\n");
                        code.push_str("    }\n");
                        code.push_str("}\n\n");
                        lines += 8;
                    }
                    5 => {
                        // Ternary operator chain
                        code.push_str(&format!(
                            "F mod{}_ternary_{}(a: i64, b: i64) -> i64 {{\n",
                            m, f
                        ));
                        code.push_str(&format!(
                            "    x := a > b ? a * {} : b * {}\n",
                            f + 1,
                            f + 2
                        ));
                        code.push_str(&format!(
                            "    y := x > {} ? x - {} : x + {}\n",
                            f * 10,
                            f,
                            f * 2
                        ));
                        code.push_str("    R x + y\n");
                        code.push_str("}\n\n");
                        lines += 6;
                    }
                    6 => {
                        // Struct construction and field access
                        code.push_str(&format!("F mod{}_struct_{}(v: i64) -> i64 {{\n", m, f));
                        code.push_str(&format!(
                            "    p := Point{}_0_{} {{ x: v, y: v * {}, z: v + {} }}\n",
                            m,
                            m * 6,
                            f % 5 + 1,
                            f
                        ));
                        code.push_str("    R p.x + p.y + p.z\n");
                        code.push_str("}\n\n");
                        lines += 5;
                    }
                    _ => {
                        // Multi-variable computation
                        code.push_str(&format!(
                            "F mod{}_compute_{}(a: i64, b: i64, c: i64) -> i64 {{\n",
                            m, f
                        ));
                        code.push_str(&format!("    x := a * {} + b * {}\n", f + 1, f % 3 + 1));
                        code.push_str(&format!("    y := b * {} - c * {}\n", f % 4 + 1, f % 2 + 1));
                        code.push_str(&format!("    z := c * {} + x\n", f % 5 + 1));
                        code.push_str("    w := x + y - z\n");
                        code.push_str("    R (x + y + z + w) / 4\n");
                        code.push_str("}\n\n");
                        lines += 8;
                    }
                }
            }

            // Add a module-level aggregator function
            if lines < target_lines {
                code.push_str(&format!("F mod{}_aggregate() -> i64 {{\n", m));
                code.push_str(&format!("    a := mod{}_arithmetic_0(10, 20)\n", m));
                code.push_str(&format!("    b := mod{}_recursive_1(5)\n", m));
                code.push_str(&format!("    c := mod{}_conditional_2(15)\n", m));
                code.push_str("    R a + b + c\n");
                code.push_str("}\n\n");
                lines += 7;
            }
        }

        // Main entry point
        code.push_str("F main() -> i64 {\n");
        code.push_str("    result := mod0_arithmetic_0(42, 13)\n");
        if module_count > 1 {
            code.push_str("    result2 := mod0_aggregate()\n");
            code.push_str("    R result + result2\n");
        } else {
            code.push_str("    R result\n");
        }
        code.push_str("}\n");

        code
    }

    /// Generate a multi-module Vais project.
    /// Returns a vector of (module_name, source_code) tuples.
    ///
    /// # Arguments
    /// * `num_modules` - Number of modules to generate
    /// * `lines_per_module` - Approximate lines per module
    ///
    /// # Returns
    /// Vec of (filename, source_code) tuples
    pub fn generate_multi_module_project(
        num_modules: usize,
        lines_per_module: usize,
    ) -> Vec<(String, String)> {
        let mut modules = Vec::new();

        for m in 0..num_modules {
            let mut code = String::with_capacity(lines_per_module * 50);
            let mut lines = 0;

            // Module header
            code.push_str(&format!("# Module {} — Part of multi-module benchmark\n\n", m));
            lines += 2;

            // Import from previous module (if not first)
            if m > 0 {
                code.push_str(&format!("U module{}\n\n", m - 1));
                lines += 2;
            }

            // Generate structs
            let structs_count = 3;
            for s in 0..structs_count {
                if lines >= lines_per_module {
                    break;
                }
                code.push_str(&format!("P S Data{}_{} {{\n", m, s));
                code.push_str("    value: i64,\n");
                code.push_str("    status: bool\n");
                code.push_str("}\n\n");
                lines += 5;
            }

            // Generate enums
            let enums_count = 2;
            for e in 0..enums_count {
                if lines >= lines_per_module {
                    break;
                }
                code.push_str(&format!("P E Status{}_{} {{\n", m, e));
                code.push_str("    Active,\n");
                code.push_str("    Inactive,\n");
                code.push_str("    Error(i64)\n");
                code.push_str("}\n\n");
                lines += 6;
            }

            // Generate public functions
            let funcs_count = (lines_per_module / 10).max(5);
            for f in 0..funcs_count {
                if lines >= lines_per_module {
                    break;
                }

                match f % 4 {
                    0 => {
                        code.push_str(&format!("P F process_{}(x: i64) -> i64 {{\n", f));
                        code.push_str(&format!("    result := x * {} + {}\n", f + 1, m));
                        code.push_str("    I result > 100 {\n");
                        code.push_str("        R result / 2\n");
                        code.push_str("    }\n");
                        code.push_str("    R result\n");
                        code.push_str("}\n\n");
                        lines += 8;
                    }
                    1 => {
                        code.push_str(&format!("P F calculate_{}(a: i64, b: i64) -> i64 {{\n", f));
                        code.push_str("    L {\n");
                        code.push_str("        I a <= b {\n");
                        code.push_str("            R a + b\n");
                        code.push_str("        }\n");
                        code.push_str(&format!("        a := a - {}\n", f % 5 + 1));
                        code.push_str("    }\n");
                        code.push_str("}\n\n");
                        lines += 9;
                    }
                    2 => {
                        code.push_str(&format!("P F recursive_{}(n: i64) -> i64 {{\n", f));
                        code.push_str(&format!("    I n <= {} {{ R n }}\n", f % 3 + 1));
                        code.push_str("    R @(n - 1) + @(n - 2)\n");
                        code.push_str("}\n\n");
                        lines += 5;
                    }
                    _ => {
                        code.push_str(&format!("P F transform_{}(x: i64) -> i64 {{\n", f));
                        code.push_str(&format!("    a := x > {} ? x * 2 : x + {}\n", f * 10, f));
                        code.push_str(&format!("    b := a < {} ? a - {} : a\n", f * 20, f * 2));
                        code.push_str("    R a + b\n");
                        code.push_str("}\n\n");
                        lines += 6;
                    }
                }
            }

            // Add cross-module reference if not first module
            if m > 0 && lines < lines_per_module {
                code.push_str("P F call_previous() -> i64 {\n");
                code.push_str(&format!("    R module{}::process_0(42)\n", m - 1));
                code.push_str("}\n\n");
            }

            let filename = format!("module{}.vais", m);
            modules.push((filename, code));
        }

        // Create main module that imports and uses others
        if num_modules > 0 {
            let mut main_code = String::new();
            main_code.push_str("# Main module — Entry point for multi-module benchmark\n\n");

            for m in 0..num_modules {
                main_code.push_str(&format!("U module{}\n", m));
            }
            main_code.push('\n');

            main_code.push_str("F main() -> i64 {\n");
            main_code.push_str("    sum := mut 0\n");
            for m in 0..num_modules.min(5) {
                main_code.push_str(&format!(
                    "    sum = sum + module{}::process_0({})\n",
                    m,
                    m * 10
                ));
            }
            main_code.push_str("    R sum\n");
            main_code.push_str("}\n");

            modules.push(("main.vais".to_string(), main_code));
        }

        modules
    }

    /// Generate a project targeting a specific total line count across multiple modules.
    ///
    /// # Arguments
    /// * `target_total_lines` - Total lines to generate across all modules
    /// * `num_modules` - Number of modules to split across (min 1)
    ///
    /// # Returns
    /// Vec of (filename, source_code) tuples
    pub fn generate_distributed_project(
        target_total_lines: usize,
        num_modules: usize,
    ) -> Vec<(String, String)> {
        let num_modules = num_modules.max(1);
        let lines_per_module = target_total_lines / num_modules;
        generate_multi_module_project(num_modules, lines_per_module)
    }
}
