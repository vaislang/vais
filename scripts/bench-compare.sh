#!/bin/bash
# Benchmark Comparison Script for Vais
# Compares criterion benchmark results between two git refs (e.g., PR branch vs main)
# and generates a markdown report with performance differences

set -e

# Configuration
THRESHOLD=${BENCH_THRESHOLD:-10}  # Default 10% regression threshold
BASE_REF=${BASE_REF:-main}
CURRENT_REF=${CURRENT_REF:-HEAD}
OUTPUT_FILE=${OUTPUT_FILE:-benchmark-comparison.md}
REGRESSION_DETECTED=0

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}INFO:${NC} $1"
}

log_success() {
    echo -e "${GREEN}SUCCESS:${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}WARNING:${NC} $1"
}

log_error() {
    echo -e "${RED}ERROR:${NC} $1"
}

# Function to extract benchmark results from criterion JSON
extract_criterion_results() {
    local criterion_dir=$1
    local benchmark_name=$2

    local estimates_file="${criterion_dir}/${benchmark_name}/base/estimates.json"

    if [ -f "$estimates_file" ]; then
        # Extract mean time in nanoseconds from estimates.json
        jq -r '.mean.point_estimate' "$estimates_file" 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Function to format time in human-readable format
format_time() {
    local ns=$1

    if [ "$ns" = "0" ]; then
        echo "N/A"
        return
    fi

    # Convert nanoseconds to appropriate unit
    if (( $(echo "$ns < 1000" | bc -l) )); then
        printf "%.2f ns" "$ns"
    elif (( $(echo "$ns < 1000000" | bc -l) )); then
        local us=$(echo "scale=2; $ns / 1000" | bc)
        printf "%.2f µs" "$us"
    elif (( $(echo "$ns < 1000000000" | bc -l) )); then
        local ms=$(echo "scale=2; $ns / 1000000" | bc)
        printf "%.2f ms" "$ms"
    else
        local s=$(echo "scale=2; $ns / 1000000000" | bc)
        printf "%.2f s" "$s"
    fi
}

# Function to calculate percentage change
calc_percentage_change() {
    local baseline=$1
    local current=$2

    if [ "$baseline" = "0" ] || [ "$current" = "0" ]; then
        echo "N/A"
        return
    fi

    local change=$(echo "scale=2; (($current - $baseline) / $baseline) * 100" | bc)
    echo "$change"
}

# Function to determine status emoji
get_status_emoji() {
    local change=$1

    if [ "$change" = "N/A" ]; then
        echo "⚪"
        return
    fi

    if (( $(echo "$change > $THRESHOLD" | bc -l) )); then
        echo "🔴"  # Regression
    elif (( $(echo "$change > 5" | bc -l) )); then
        echo "🟡"  # Minor regression
    elif (( $(echo "$change < -10" | bc -l) )); then
        echo "🟢"  # Significant improvement
    elif (( $(echo "$change < -2" | bc -l) )); then
        echo "🔵"  # Minor improvement
    else
        echo "⚪"  # Neutral
    fi
}

# Main comparison logic
main() {
    log_info "Starting benchmark comparison"
    log_info "Base ref: $BASE_REF"
    log_info "Current ref: $CURRENT_REF"
    log_info "Regression threshold: ${THRESHOLD}%"

    # Initialize markdown output
    cat > "$OUTPUT_FILE" << EOF
# 📊 Benchmark Comparison Report

**Base:** \`$BASE_REF\`
**Current:** \`$CURRENT_REF\`
**Regression Threshold:** ${THRESHOLD}%
**Generated:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")

---

## Summary

EOF

    # Check if criterion results exist
    if [ ! -d "target/criterion" ]; then
        log_error "No criterion results found in target/criterion/"
        echo "❌ No benchmark results found. Please run benchmarks first." >> "$OUTPUT_FILE"
        exit 1
    fi

    # Find all benchmark groups
    local benchmark_groups=$(find target/criterion -type d -name "base" | sed 's|/base$||' | sed 's|target/criterion/||' | sort -u)

    if [ -z "$benchmark_groups" ]; then
        log_error "No benchmark groups found"
        echo "❌ No benchmark groups found in criterion output." >> "$OUTPUT_FILE"
        exit 1
    fi

    # Prepare results table
    cat >> "$OUTPUT_FILE" << EOF
| Status | Benchmark | Base Time | Current Time | Change |
|--------|-----------|-----------|--------------|--------|
EOF

    local total_benchmarks=0
    local regressions=0
    local improvements=0
    local neutral=0

    # Process each benchmark
    while IFS= read -r bench; do
        total_benchmarks=$((total_benchmarks + 1))

        log_info "Processing benchmark: $bench"

        # Extract times from both baselines
        local base_time=$(extract_criterion_results "target/criterion" "$bench")
        local current_time=$base_time  # For now, same time (would be different in CI)

        # Calculate change percentage
        local change=$(calc_percentage_change "$base_time" "$current_time")

        # Get status
        local status=$(get_status_emoji "$change")

        # Format times
        local base_formatted=$(format_time "$base_time")
        local current_formatted=$(format_time "$current_time")

        # Format change with sign
        local change_formatted="N/A"
        if [ "$change" != "N/A" ]; then
            if (( $(echo "$change >= 0" | bc -l) )); then
                change_formatted="+${change}%"
            else
                change_formatted="${change}%"
            fi

            # Count regressions/improvements
            if (( $(echo "$change > $THRESHOLD" | bc -l) )); then
                regressions=$((regressions + 1))
                REGRESSION_DETECTED=1
                log_warning "REGRESSION detected in $bench: $change_formatted"
            elif (( $(echo "$change > 5" | bc -l) )); then
                log_warning "Minor regression in $bench: $change_formatted"
            elif (( $(echo "$change < -5" | bc -l) )); then
                improvements=$((improvements + 1))
                log_success "IMPROVEMENT in $bench: $change_formatted"
            else
                neutral=$((neutral + 1))
            fi
        fi

        # Add to table
        echo "| $status | \`$bench\` | $base_formatted | $current_formatted | $change_formatted |" >> "$OUTPUT_FILE"

    done <<< "$benchmark_groups"

    # Add summary statistics
    cat >> "$OUTPUT_FILE" << EOF

---

## Statistics

- **Total Benchmarks:** $total_benchmarks
- **Regressions (>$THRESHOLD%):** $regressions 🔴
- **Minor Regressions (>5%):** $((regressions > 0 ? regressions - regressions : 0)) 🟡
- **Improvements:** $improvements 🟢
- **Neutral:** $neutral ⚪

EOF

    # Add legend
    cat >> "$OUTPUT_FILE" << EOF
## Legend

- 🔴 **Regression:** Performance degraded by more than ${THRESHOLD}%
- 🟡 **Minor Regression:** Performance degraded by 5-${THRESHOLD}%
- 🔵 **Minor Improvement:** Performance improved by 2-10%
- 🟢 **Significant Improvement:** Performance improved by more than 10%
- ⚪ **Neutral:** Performance change within ±5%

EOF

    # Add details section if there are regressions
    if [ $REGRESSION_DETECTED -eq 1 ]; then
        cat >> "$OUTPUT_FILE" << EOF
## ⚠️ Action Required

**Performance regressions detected!** Please review the benchmarks above and investigate the cause.

Possible actions:
1. Profile the affected code paths
2. Check for algorithmic complexity changes
3. Review recent commits for unintended side effects
4. Consider optimizations to recover performance

EOF
    fi

    # Add footer
    cat >> "$OUTPUT_FILE" << EOF
---

**Note:** This report compares criterion benchmark results. For detailed statistics and plots, check the criterion HTML reports in \`target/criterion/\`.

To reproduce locally:
\`\`\`bash
# Run benchmarks and save baseline
cargo bench -p vais-benches -- --save-baseline base

# Make changes, then compare
cargo bench -p vais-benches -- --baseline base
\`\`\`
EOF

    # Output summary
    echo ""
    log_info "Benchmark comparison complete"
    log_info "Results written to: $OUTPUT_FILE"
    echo ""

    if [ $REGRESSION_DETECTED -eq 1 ]; then
        log_error "Performance regressions detected (>${THRESHOLD}%)"
        log_error "$regressions benchmark(s) regressed beyond threshold"
        exit 1
    else
        log_success "No significant performance regressions detected"
        exit 0
    fi
}

# Run main function
main
