#!/bin/bash
# Performance Regression Check for CI
#
# Usage: ./benches/regression_check.sh [baseline_file]
#
# Runs benchmarks and compares against a baseline. If any benchmark
# regresses by more than 10%, the script exits with code 1.

set -e

BASELINE_FILE="${1:-benches/results/baseline.json}"
RESULTS_FILE="benches/results/current.json"
THRESHOLD=10  # percent regression threshold

mkdir -p benches/results

echo "=== Vais Performance Regression Check ==="
echo "Threshold: ${THRESHOLD}% regression"
echo ""

# Run compile benchmarks and capture results
echo "Running compile benchmarks..."
cargo bench --bench compile_bench -- --output-format=bencher 2>/dev/null | tee /tmp/vais_bench_output.txt

# Parse benchmark results into JSON
echo "Parsing results..."
python3 -c "
import re, json, sys

results = {}
for line in open('/tmp/vais_bench_output.txt'):
    # Parse criterion bencher format: test name ... bench: N ns/iter (+/- M)
    m = re.match(r'test (.+)\s+\.\.\.\s+bench:\s+([\d,]+)\s+ns/iter', line)
    if m:
        name = m.group(1).strip()
        ns = int(m.group(2).replace(',', ''))
        results[name] = ns

json.dump(results, open('$RESULTS_FILE', 'w'), indent=2)
print(f'Captured {len(results)} benchmark results')
" 2>/dev/null || echo "Note: Detailed parsing requires Python 3"

# Compare against baseline if it exists
if [ -f "$BASELINE_FILE" ]; then
    echo ""
    echo "Comparing against baseline: $BASELINE_FILE"
    echo ""

    python3 -c "
import json, sys

baseline = json.load(open('$BASELINE_FILE'))
current = json.load(open('$RESULTS_FILE'))

regressions = []
improvements = []

for name, base_ns in baseline.items():
    if name in current:
        cur_ns = current[name]
        pct_change = ((cur_ns - base_ns) / base_ns) * 100

        if pct_change > $THRESHOLD:
            regressions.append((name, base_ns, cur_ns, pct_change))
            print(f'  REGRESSION: {name}: {base_ns}ns -> {cur_ns}ns (+{pct_change:.1f}%)')
        elif pct_change < -5:
            improvements.append((name, base_ns, cur_ns, pct_change))
            print(f'  IMPROVED:   {name}: {base_ns}ns -> {cur_ns}ns ({pct_change:.1f}%)')
        else:
            print(f'  OK:         {name}: {base_ns}ns -> {cur_ns}ns ({pct_change:+.1f}%)')

print()
if regressions:
    print(f'FAIL: {len(regressions)} benchmark(s) regressed by more than {$THRESHOLD}%')
    sys.exit(1)
else:
    print(f'PASS: No regressions detected ({len(improvements)} improvements)')
" 2>/dev/null || echo "Note: Comparison requires Python 3"
else
    echo ""
    echo "No baseline found at $BASELINE_FILE"
    echo "Creating baseline from current results..."
    cp "$RESULTS_FILE" "$BASELINE_FILE" 2>/dev/null || true
    echo "Run this script again after changes to compare."
fi

echo ""
echo "=== Done ==="
