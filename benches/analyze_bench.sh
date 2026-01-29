#!/bin/bash
# Benchmark Analysis Script
# Compares benchmark results and detects performance regressions

set -e

THRESHOLD=${BENCH_THRESHOLD:-10}  # Default 10% regression threshold
BASELINE=${BASELINE:-main}
CURRENT=${CURRENT:-current}

echo "🔍 Analyzing benchmark results..."
echo "Baseline: $BASELINE"
echo "Current: $CURRENT"
echo "Regression threshold: ${THRESHOLD}%"
echo ""

# Check if criterion results exist
if [ ! -d "target/criterion" ]; then
    echo "❌ No criterion results found. Run 'cargo bench' first."
    exit 1
fi

# Function to extract timing from criterion output
extract_time() {
    local bench_name=$1
    local baseline=$2

    # Look for the benchmark results in criterion directory
    local result_file="target/criterion/$bench_name/$baseline/estimates.json"

    if [ -f "$result_file" ]; then
        # Extract mean time in nanoseconds
        jq -r '.mean.point_estimate' "$result_file" 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Compare benchmarks
HAS_REGRESSION=false
SUMMARY=""

# List all benchmark names
BENCHES=$(find target/criterion -name "estimates.json" -type f | sed 's|target/criterion/||' | sed 's|/[^/]*/estimates.json||' | sort -u)

for bench in $BENCHES; do
    baseline_time=$(extract_time "$bench" "$BASELINE")
    current_time=$(extract_time "$bench" "$CURRENT")

    if [ "$baseline_time" != "0" ] && [ "$current_time" != "0" ]; then
        # Calculate percentage change
        change=$(echo "scale=2; (($current_time - $baseline_time) / $baseline_time) * 100" | bc)

        # Check if regression exceeds threshold
        if (( $(echo "$change > $THRESHOLD" | bc -l) )); then
            echo "⚠️  REGRESSION: $bench: ${change}% slower"
            HAS_REGRESSION=true
            SUMMARY="${SUMMARY}\n- ⚠️  $bench: +${change}%"
        elif (( $(echo "$change < -5" | bc -l) )); then
            echo "✅ IMPROVEMENT: $bench: ${change#-}% faster"
            SUMMARY="${SUMMARY}\n- ✅ $bench: ${change}%"
        else
            echo "➡️  $bench: ${change}% (within threshold)"
            SUMMARY="${SUMMARY}\n- ➡️  $bench: ${change}%"
        fi
    fi
done

echo ""
echo "📊 Summary:"
echo -e "$SUMMARY"
echo ""

if [ "$HAS_REGRESSION" = true ]; then
    echo "❌ Performance regressions detected (>${THRESHOLD}%)"
    exit 1
else
    echo "✅ No significant performance regressions detected"
    exit 0
fi
