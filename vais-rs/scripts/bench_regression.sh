#!/bin/bash
# Performance Regression Test Script for Vais
# This script runs benchmarks and compares against a baseline

set -e

BASELINE_FILE="${BASELINE_FILE:-target/criterion/baseline.json}"
THRESHOLD="${THRESHOLD:-10}"  # Percentage threshold for regression

echo "=== Vais Performance Regression Test ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Run benchmarks
echo "Running benchmarks..."
cargo bench --bench benchmarks -- --noplot --save-baseline current 2>/dev/null

# Check if baseline exists
if [ ! -d "target/criterion" ]; then
    echo -e "${YELLOW}No baseline found. Saving current run as baseline.${NC}"
    cargo bench --bench benchmarks -- --noplot --save-baseline baseline 2>/dev/null
    echo -e "${GREEN}Baseline saved. Future runs will compare against this.${NC}"
    exit 0
fi

# Compare against baseline
echo ""
echo "Comparing against baseline..."
echo ""

# Parse criterion output for regression
REGRESSION_FOUND=0

for dir in target/criterion/*/; do
    if [ -d "$dir" ]; then
        GROUP=$(basename "$dir")

        # Skip non-benchmark directories
        if [ "$GROUP" == "report" ] || [ "$GROUP" == "baseline" ]; then
            continue
        fi

        echo "Group: $GROUP"

        for bench_dir in "$dir"*/; do
            if [ -d "$bench_dir" ]; then
                BENCH=$(basename "$bench_dir")
                ESTIMATE_FILE="$bench_dir/current/estimates.json"
                BASELINE_ESTIMATE="$bench_dir/baseline/estimates.json"

                if [ -f "$ESTIMATE_FILE" ] && [ -f "$BASELINE_ESTIMATE" ]; then
                    # Extract mean time from JSON (using jq if available, else grep)
                    if command -v jq &> /dev/null; then
                        CURRENT_TIME=$(jq -r '.mean.point_estimate' "$ESTIMATE_FILE" 2>/dev/null || echo "0")
                        BASELINE_TIME=$(jq -r '.mean.point_estimate' "$BASELINE_ESTIMATE" 2>/dev/null || echo "0")
                    else
                        # Fallback: simple grep-based extraction
                        CURRENT_TIME=$(grep -o '"point_estimate":[0-9.]*' "$ESTIMATE_FILE" | head -1 | cut -d: -f2)
                        BASELINE_TIME=$(grep -o '"point_estimate":[0-9.]*' "$BASELINE_ESTIMATE" | head -1 | cut -d: -f2)
                    fi

                    if [ -n "$CURRENT_TIME" ] && [ -n "$BASELINE_TIME" ] && [ "$BASELINE_TIME" != "0" ]; then
                        # Calculate percentage change
                        CHANGE=$(echo "scale=2; (($CURRENT_TIME - $BASELINE_TIME) / $BASELINE_TIME) * 100" | bc 2>/dev/null || echo "0")

                        if [ "$(echo "$CHANGE > $THRESHOLD" | bc 2>/dev/null || echo 0)" -eq 1 ]; then
                            echo -e "  ${RED}✗ $BENCH: +${CHANGE}% (regression detected)${NC}"
                            REGRESSION_FOUND=1
                        elif [ "$(echo "$CHANGE < -$THRESHOLD" | bc 2>/dev/null || echo 0)" -eq 1 ]; then
                            echo -e "  ${GREEN}✓ $BENCH: ${CHANGE}% (improvement!)${NC}"
                        else
                            echo -e "  ${GREEN}✓ $BENCH: ${CHANGE}% (within threshold)${NC}"
                        fi
                    fi
                fi
            fi
        done
    fi
done

echo ""

if [ $REGRESSION_FOUND -eq 1 ]; then
    echo -e "${RED}=== PERFORMANCE REGRESSION DETECTED ===${NC}"
    echo "One or more benchmarks exceeded the ${THRESHOLD}% regression threshold."
    exit 1
else
    echo -e "${GREEN}=== ALL BENCHMARKS PASSED ===${NC}"
    echo "No significant performance regressions detected."
    exit 0
fi
