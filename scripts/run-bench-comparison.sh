#!/bin/bash
# Quick benchmark comparison script for local development
# Compares current branch against main (or specified base)

set -e

BASE_BRANCH=${1:-main}
CURRENT_BRANCH=$(git branch --show-current)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}====================================${NC}"
echo -e "${BLUE}Vais Benchmark Comparison${NC}"
echo -e "${BLUE}====================================${NC}"
echo ""
echo "Base branch:    $BASE_BRANCH"
echo "Current branch: $CURRENT_BRANCH"
echo ""

# Ensure we're on a branch
if [ -z "$CURRENT_BRANCH" ]; then
    echo -e "${RED}ERROR: Not on a branch (detached HEAD?)${NC}"
    exit 1
fi

# Ensure working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${YELLOW}WARNING: Working directory has uncommitted changes${NC}"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Run benchmarks on current branch
echo -e "${BLUE}[1/4]${NC} Running benchmarks on current branch ($CURRENT_BRANCH)..."
cargo bench -p vais-benches --bench compile_bench -- --save-baseline current --quiet
cargo bench -p vais-benches --bench compiler_benchmarks -- --save-baseline current --quiet

# Save current branch results
mkdir -p /tmp/vais-bench-comparison
cp -r target/criterion /tmp/vais-bench-comparison/current/

# Switch to base branch
echo ""
echo -e "${BLUE}[2/4]${NC} Switching to base branch ($BASE_BRANCH)..."
git checkout $BASE_BRANCH

# Run benchmarks on base branch
echo -e "${BLUE}[3/4]${NC} Running benchmarks on base branch ($BASE_BRANCH)..."
cargo bench -p vais-benches --bench compile_bench -- --save-baseline base --quiet
cargo bench -p vais-benches --bench compiler_benchmarks -- --save-baseline base --quiet

# Save base branch results
cp -r target/criterion /tmp/vais-bench-comparison/base/

# Switch back to current branch
echo ""
echo -e "${BLUE}[4/4]${NC} Switching back to current branch ($CURRENT_BRANCH)..."
git checkout $CURRENT_BRANCH

# Restore current results
rm -rf target/criterion
cp -r /tmp/vais-bench-comparison/current target/criterion

# Copy base results for comparison
echo ""
echo -e "${BLUE}Generating comparison report...${NC}"

# Use Python to generate comparison
python3 - << 'EOF'
import json
import os
from pathlib import Path

THRESHOLD = 10.0

def find_benchmarks(base_dir):
    benchmarks = {}
    criterion_dir = Path(base_dir)
    for estimates in criterion_dir.rglob('estimates.json'):
        parts = estimates.parts
        if 'criterion' in parts:
            idx = parts.index('criterion')
            if idx + 2 < len(parts):
                bench_name = '/'.join(parts[idx+1:-2])
                benchmarks[bench_name] = estimates
    return benchmarks

def load_estimate(path):
    try:
        with open(path) as f:
            data = json.load(f)
            return data['mean']['point_estimate']
    except:
        return None

def format_time(ns):
    if ns < 1000:
        return f"{ns:.2f} ns"
    elif ns < 1_000_000:
        return f"{ns/1000:.2f} µs"
    elif ns < 1_000_000_000:
        return f"{ns/1_000_000:.2f} ms"
    else:
        return f"{ns/1_000_000_000:.2f} s"

base_dir = '/tmp/vais-bench-comparison/base/criterion'
current_dir = 'target/criterion'

current_benchmarks = find_benchmarks(current_dir)
base_benchmarks = find_benchmarks(base_dir)

print("\n" + "="*70)
print("BENCHMARK COMPARISON REPORT")
print("="*70 + "\n")

regressions = []
improvements = []
total = 0

for bench_name in sorted(current_benchmarks.keys()):
    current_path = current_benchmarks[bench_name]

    # Find corresponding base benchmark
    base_path = None
    for base_name, path in base_benchmarks.items():
        if base_name == bench_name:
            base_path = path
            break

    if not base_path:
        continue

    current_time = load_estimate(current_path)
    base_time = load_estimate(base_path)

    if current_time and base_time:
        total += 1
        change_pct = ((current_time - base_time) / base_time) * 100

        # Determine status
        if change_pct > THRESHOLD:
            status = "🔴 REGRESSION"
            regressions.append((bench_name, change_pct))
        elif change_pct > 5:
            status = "🟡 MINOR REG"
        elif change_pct < -10:
            status = "🟢 IMPROVEMENT"
            improvements.append((bench_name, change_pct))
        elif change_pct < -2:
            status = "🔵 MINOR IMP"
        else:
            status = "⚪ NEUTRAL"

        print(f"{status:15} {bench_name:45} {format_time(base_time):>12} → {format_time(current_time):>12} ({change_pct:+6.2f}%)")

print("\n" + "="*70)
print(f"SUMMARY: {total} benchmarks")
print("="*70)

if regressions:
    print(f"\n🔴 REGRESSIONS ({len(regressions)}):")
    for name, pct in regressions:
        print(f"  - {name}: {pct:+.2f}%")

if improvements:
    print(f"\n🟢 IMPROVEMENTS ({len(improvements)}):")
    for name, pct in improvements:
        print(f"  - {name}: {pct:.2f}%")

print("\n" + "="*70)
if regressions:
    print(f"❌ FAILED: {len(regressions)} regression(s) exceed {THRESHOLD}% threshold")
    print("="*70 + "\n")
    exit(1)
else:
    print(f"✅ PASSED: No regressions exceed {THRESHOLD}% threshold")
    print("="*70 + "\n")
    exit(0)
EOF

comparison_exit=$?

# Cleanup
rm -rf /tmp/vais-bench-comparison

# Show detailed criterion reports
echo ""
echo -e "${BLUE}For detailed HTML reports, open:${NC}"
echo "  target/criterion/report/index.html"
echo ""

# Exit with comparison result
exit $comparison_exit
