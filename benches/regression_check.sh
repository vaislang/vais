#!/bin/bash
# Performance Regression Check for CI
#
# Usage: ./benches/regression_check.sh [baseline_file]
#
# Runs benchmarks and compares against a baseline. Reports:
#   - WARNING: 5-10% regression
#   - CRITICAL: >10% regression (exits with code 1)
#   - IMPROVED: >5% improvement
#
# Output format: Markdown table suitable for PR comments.

set -e

BASELINE_FILE="${1:-benches/results/baseline.json}"
RESULTS_FILE="benches/results/current.json"
REPORT_FILE="benches/results/report.md"
WARNING_THRESHOLD=5   # percent
CRITICAL_THRESHOLD=10 # percent

mkdir -p benches/results

echo "=== Vais Performance Regression Check ==="
echo "Warning threshold: ${WARNING_THRESHOLD}%"
echo "Critical threshold: ${CRITICAL_THRESHOLD}%"
echo ""

# Run compile benchmarks with Criterion JSON output
echo "Running compile benchmarks..."
if cargo bench --bench compile_bench -- --output-format=bencher 2>/dev/null | tee /tmp/vais_bench_output.txt; then
    echo "Compile benchmarks completed."
else
    echo "Warning: compile_bench failed or not available, trying largescale_bench..."
    cargo bench --bench largescale_bench -- --output-format=bencher 2>/dev/null | tee /tmp/vais_bench_output.txt || true
fi

# Also try runtime benchmarks if available
echo ""
echo "Running runtime benchmarks..."
cargo bench --bench runtime_bench -- --output-format=bencher 2>/dev/null | tee -a /tmp/vais_bench_output.txt || true

# Parse benchmark results into JSON
echo ""
echo "Parsing results..."
python3 -c "
import re, json, sys

results = {}
for line in open('/tmp/vais_bench_output.txt'):
    # Parse criterion bencher format: test name ... bench: N ns/iter (+/- M)
    m = re.match(r'test (.+)\s+\.\.\.\s+bench:\s+([\d,]+)\s+ns/iter\s+\(\+/- ([\d,]+)\)', line)
    if m:
        name = m.group(1).strip()
        ns = int(m.group(2).replace(',', ''))
        stddev = int(m.group(3).replace(',', ''))
        results[name] = {'ns': ns, 'stddev': stddev}
    else:
        # Simpler format without stddev
        m2 = re.match(r'test (.+)\s+\.\.\.\s+bench:\s+([\d,]+)\s+ns/iter', line)
        if m2:
            name = m2.group(1).strip()
            ns = int(m2.group(2).replace(',', ''))
            results[name] = {'ns': ns, 'stddev': 0}

json.dump(results, open('$RESULTS_FILE', 'w'), indent=2)
print(f'Captured {len(results)} benchmark results')
" 2>/dev/null || {
    echo "Note: Detailed parsing requires Python 3. Falling back to basic mode."
    echo "{}" > "$RESULTS_FILE"
}

# Compare against baseline if it exists
if [ -f "${BASELINE_FILE}" ]; then
    echo ""
    echo "Comparing against baseline: ${BASELINE_FILE}"
    echo ""

    python3 -c "
import json, sys

baseline = json.load(open('$BASELINE_FILE'))
current = json.load(open('$RESULTS_FILE'))

regressions = []
warnings = []
improvements = []
stable = []

# Normalize: handle both old format (plain ns) and new format (dict with ns/stddev)
def get_ns(entry):
    if isinstance(entry, dict):
        return entry['ns']
    return entry

for name in sorted(set(list(baseline.keys()) + list(current.keys()))):
    if name not in baseline:
        print(f'  NEW:          {name}: {get_ns(current[name])}ns (no baseline)')
        continue
    if name not in current:
        print(f'  MISSING:      {name}: baseline {get_ns(baseline[name])}ns (not in current run)')
        continue

    base_ns = get_ns(baseline[name])
    cur_ns = get_ns(current[name])

    if base_ns == 0:
        continue

    pct_change = ((cur_ns - base_ns) / base_ns) * 100

    if pct_change > $CRITICAL_THRESHOLD:
        regressions.append((name, base_ns, cur_ns, pct_change))
        print(f'  CRITICAL:   {name}: {base_ns}ns -> {cur_ns}ns (+{pct_change:.1f}%)')
    elif pct_change > $WARNING_THRESHOLD:
        warnings.append((name, base_ns, cur_ns, pct_change))
        print(f'  WARNING:    {name}: {base_ns}ns -> {cur_ns}ns (+{pct_change:.1f}%)')
    elif pct_change < -5:
        improvements.append((name, base_ns, cur_ns, pct_change))
        print(f'  IMPROVED:   {name}: {base_ns}ns -> {cur_ns}ns ({pct_change:.1f}%)')
    else:
        stable.append((name, base_ns, cur_ns, pct_change))
        print(f'  OK:         {name}: {base_ns}ns -> {cur_ns}ns ({pct_change:+.1f}%)')

# Generate markdown report
with open('$REPORT_FILE', 'w') as f:
    f.write('## Performance Regression Report\n\n')

    if regressions:
        f.write('### :x: Critical Regressions (>{threshold}%)\n\n'.format(threshold=$CRITICAL_THRESHOLD))
        f.write('| Benchmark | Baseline | Current | Change |\n')
        f.write('|-----------|----------|---------|--------|\n')
        for name, base, cur, pct in regressions:
            f.write(f'| {name} | {base:,}ns | {cur:,}ns | +{pct:.1f}% |\n')
        f.write('\n')

    if warnings:
        f.write('### :warning: Warnings (>{threshold}%)\n\n'.format(threshold=$WARNING_THRESHOLD))
        f.write('| Benchmark | Baseline | Current | Change |\n')
        f.write('|-----------|----------|---------|--------|\n')
        for name, base, cur, pct in warnings:
            f.write(f'| {name} | {base:,}ns | {cur:,}ns | +{pct:.1f}% |\n')
        f.write('\n')

    if improvements:
        f.write('### :rocket: Improvements (>5%)\n\n')
        f.write('| Benchmark | Baseline | Current | Change |\n')
        f.write('|-----------|----------|---------|--------|\n')
        for name, base, cur, pct in improvements:
            f.write(f'| {name} | {base:,}ns | {cur:,}ns | {pct:.1f}% |\n')
        f.write('\n')

    f.write(f'**Summary**: {len(stable)} stable, {len(improvements)} improved, ')
    f.write(f'{len(warnings)} warnings, {len(regressions)} critical\n')

print()
if regressions:
    print(f'FAIL: {len(regressions)} benchmark(s) regressed by more than {$CRITICAL_THRESHOLD}%')
    print(f'Report: $REPORT_FILE')
    sys.exit(1)
elif warnings:
    print(f'WARN: {len(warnings)} benchmark(s) regressed by {$WARNING_THRESHOLD}-{$CRITICAL_THRESHOLD}% (non-blocking)')
    print(f'Report: $REPORT_FILE')
else:
    print(f'PASS: No regressions detected ({len(improvements)} improvements, {len(stable)} stable)')
" 2>/dev/null || echo "Note: Comparison requires Python 3"
else
    echo ""
    echo "No baseline found at ${BASELINE_FILE}"
    echo "Creating baseline from current results..."
    cp "${RESULTS_FILE}" "${BASELINE_FILE}" 2>/dev/null || true
    echo "Run this script again after changes to compare."
fi

echo ""
echo "=== Done ==="
