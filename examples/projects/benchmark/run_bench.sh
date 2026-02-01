#!/bin/bash

# Performance Benchmark Runner - Vais vs C comparison
# This script compiles and runs both versions, measuring execution time

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo "======================================"
echo "Performance Benchmark: Vais vs C"
echo "======================================"
echo ""

# Compile Vais version
echo "Compiling Vais version..."
cd "$PROJECT_ROOT"
cargo run --bin vaisc -- examples/projects/benchmark/benchmark.vais > /dev/null 2>&1
echo "✓ Vais compilation complete"
echo ""

# Compile C version with optimization
echo "Compiling C version (with -O2 optimization)..."
clang -O2 -o examples/projects/benchmark/benchmark_c examples/projects/benchmark/benchmark.c
echo "✓ C compilation complete"
echo ""

# Run Vais benchmark
echo "======================================"
echo "Running Vais Benchmark"
echo "======================================"
echo ""
{ time "$SCRIPT_DIR/benchmark" ; } 2>&1 | tee /tmp/vais_bench_time.txt
VAIS_TIME=$(grep "^real" /tmp/vais_bench_time.txt | tail -1)
echo ""

# Run C benchmark
echo "======================================"
echo "Running C Benchmark"
echo "======================================"
echo ""
{ time "$SCRIPT_DIR/benchmark_c" ; } 2>&1 | tee /tmp/c_bench_time.txt
C_TIME=$(grep "^real" /tmp/c_bench_time.txt | tail -1)
echo ""

echo "======================================"
echo "Benchmark Complete!"
echo "======================================"
echo ""
echo "Performance Comparison:"
echo "  Vais: $VAIS_TIME"
echo "  C:    $C_TIME"
echo ""
echo "Lower 'real' time = better performance."
