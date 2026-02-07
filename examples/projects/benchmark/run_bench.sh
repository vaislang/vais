#!/bin/bash

# Performance Benchmark Runner - Vais vs C vs Rust vs Python
# Compiles and runs all versions, measuring execution time

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo "============================================"
echo "Performance Benchmark: Vais vs C vs Rust vs Python"
echo "============================================"
echo ""

# Compile Vais version
echo "Compiling Vais version (-O2)..."
cd "$PROJECT_ROOT"
cargo run --release --bin vaisc -- build -O2 examples/projects/benchmark/benchmark.vais -o examples/projects/benchmark/benchmark > /dev/null 2>&1
echo "  Vais compilation complete"

# Compile C version
echo "Compiling C version (-O2)..."
clang -O2 -o examples/projects/benchmark/benchmark_c examples/projects/benchmark/benchmark.c
echo "  C compilation complete"

# Compile Rust version
echo "Compiling Rust version (-O3)..."
rustc -C opt-level=3 -o examples/projects/benchmark/benchmark_rs examples/projects/benchmark/benchmark.rs
echo "  Rust compilation complete"

echo ""
echo "============================================"
echo "Running Benchmarks (5 runs each, user time)"
echo "============================================"
echo ""

run_bench() {
    local name="$1"
    shift
    local cmd="$@"
    echo "--- $name ---"
    for i in 1 2 3 4 5; do
        /usr/bin/time -p $cmd 2>&1 | grep "^user" | sed "s/^/  Run $i: /"
    done
    echo ""
}

run_bench "Vais (-O2)" "$SCRIPT_DIR/benchmark"
run_bench "C (-O2)" "$SCRIPT_DIR/benchmark_c"
run_bench "Rust (-O3)" "$SCRIPT_DIR/benchmark_rs"
run_bench "Python 3" python3 "$SCRIPT_DIR/benchmark.py"

echo "============================================"
echo "Benchmark Complete!"
echo "============================================"
echo ""
echo "Lower 'user' time = better performance."
echo "See docs/benchmarks.md for detailed analysis."
