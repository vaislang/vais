#!/bin/bash
# Vais vs Python Benchmark Runner
# ===============================

cd "$(dirname "$0")/.."

echo "============================================================"
echo "Vais vs Python Performance Benchmark"
echo "============================================================"
echo ""

# Build Vais in release mode
echo "Building Vais (release mode)..."
cargo build -p vais-cli --release -q 2>/dev/null
echo ""

# Run Python benchmark
echo "Running Python benchmark..."
echo ""
python3 benchmark/python_bench.py
echo ""

# Run Vais benchmarks individually with timing
echo "============================================================"
echo "Vais Benchmark Results"
echo "============================================================"
echo ""

VAIS="./target/release/vais"

run_vais_bench() {
    local name="$1"
    local code="$2"
    local iterations=5
    local total=0

    for i in $(seq 1 $iterations); do
        # Use gtime if available, otherwise use built-in time
        start=$(python3 -c "import time; print(time.perf_counter())")
        result=$($VAIS run -e "$code" 2>/dev/null)
        end=$(python3 -c "import time; print(time.perf_counter())")
        elapsed=$(python3 -c "print(($end - $start) * 1000000)")
        total=$(python3 -c "print($total + $elapsed)")
    done

    avg=$(python3 -c "print($total / $iterations)")
    printf "%-18s = %12s  |  %10.2f µs\n" "$name" "$result" "$avg"
}

echo "--- Recursive Functions ---"
run_vais_bench "factorial(10)" "factorial(n) = n <= 1 ? 1 : n * factorial(n - 1); factorial(10)"
run_vais_bench "factorial(20)" "factorial(n) = n <= 1 ? 1 : n * factorial(n - 1); factorial(20)"
run_vais_bench "fibonacci(20)" "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2); fib(20)"
run_vais_bench "fibonacci(30)" "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2); fib(30)"
run_vais_bench "sum_to_n(100)" "sum(n) = n <= 0 ? 0 : n + sum(n - 1); sum(100)"

echo ""
echo "--- Collection Operations (1000 elements) ---"
run_vais_bench "map (* 2)" "double(arr) = arr.@(_ * 2); double(1..1000)./+"
run_vais_bench "filter (even)" "evens(arr) = arr.?(_ % 2 == 0); #evens(1..1000)"
run_vais_bench "reduce (sum)" "total(arr) = arr./+; total(1..1000)"

echo ""
echo "--- Chained Operations ---"
run_vais_bench "filter>map>sum" "[1, 2, 3, 4, 5].?(_ > 2).@(_ * 2)./+"

echo ""
echo "============================================================"
