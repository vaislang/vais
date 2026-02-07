#!/usr/bin/env bash
set -euo pipefail

# Large-scale compilation benchmark for Vais
# Measures: compile time, memory usage, throughput for 1K ~ 50K+ line projects
#
# Usage:
#   ./scripts/largescale-bench.sh             # Run all sizes
#   ./scripts/largescale-bench.sh 10000       # Run specific line count
#   ./scripts/largescale-bench.sh --criterion  # Run criterion benchmarks

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BENCH_DIR="$PROJECT_ROOT/target/largescale-bench"
VAISC="$PROJECT_ROOT/target/release/vaisc"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${CYAN}[bench]${NC} $*"; }
ok()  { echo -e "${GREEN}[ok]${NC} $*"; }
warn(){ echo -e "${YELLOW}[warn]${NC} $*"; }
err() { echo -e "${RED}[err]${NC} $*"; }

# Build vaisc in release mode
build_compiler() {
    log "Building vaisc (release)..."
    cargo build --release --bin vaisc --manifest-path "$PROJECT_ROOT/Cargo.toml" 2>/dev/null
    ok "vaisc built: $VAISC"
}

# Generate a large Vais project with the given target line count
generate_project() {
    local target_lines=$1
    local output_dir="$BENCH_DIR/project_${target_lines}"

    mkdir -p "$output_dir"

    local main_file="$output_dir/main.vais"

    # Use Python for fast code generation
    python3 -c "
import sys

target = $target_lines
lines = 0
code = []

module_count = max(1, target // 130)

for m in range(module_count):
    if lines >= target:
        break

    code.append(f'# Module {m}')
    lines += 1

    # Structs (5 per module)
    for s in range(5):
        if lines >= target: break
        code.append(f'S Mod{m}S{s} {{')
        code.append(f'    a: i64,')
        code.append(f'    b: i64,')
        code.append(f'    c: bool')
        code.append(f'}}')
        code.append('')
        lines += 6

    # Enums (3 per module)
    for e in range(3):
        if lines >= target: break
        code.append(f'E Mod{m}R{e} {{')
        code.append(f'    Ok(i64),')
        code.append(f'    Err(i64),')
        code.append(f'    None')
        code.append(f'}}')
        code.append('')
        lines += 6

    # Functions (20 per module, varied)
    for f in range(20):
        if lines >= target: break
        kind = f % 5
        if kind == 0:
            code.append(f'F m{m}_f{f}(x: i64, y: i64) -> i64 {{')
            code.append(f'    a := x * {f+1} + y')
            code.append(f'    b := a - {f%7+1} * x')
            code.append(f'    R a + b')
            code.append(f'}}')
            code.append('')
            lines += 6
        elif kind == 1:
            code.append(f'F m{m}_r{f}(n: i64) -> i64 {{')
            code.append(f'    I n <= 1 {{ R {f%3+1} }}')
            code.append(f'    R n * @(n - 1)')
            code.append(f'}}')
            code.append('')
            lines += 5
        elif kind == 2:
            code.append(f'F m{m}_c{f}(x: i64) -> i64 {{')
            code.append(f'    I x < {f*3} {{ R x * 2 }}')
            code.append(f'    I x < {f*10} {{ R x + {f} }}')
            code.append(f'    R x')
            code.append(f'}}')
            code.append('')
            lines += 6
        elif kind == 3:
            code.append(f'F m{m}_l{f}(n: i64) -> i64 {{')
            code.append(f'    sum := mut 0')
            code.append(f'    i := mut 0')
            code.append(f'    L {{')
            code.append(f'        I i >= n {{ R sum }}')
            code.append(f'        sum = sum + i * {f%5+1}')
            code.append(f'        i = i + 1')
            code.append(f'    }}')
            code.append(f'}}')
            code.append('')
            lines += 10
        else:
            code.append(f'F m{m}_x{f}(a: i64, b: i64, c: i64) -> i64 {{')
            code.append(f'    x := a * {f} + b')
            code.append(f'    y := b * {f%4+1} - c')
            code.append(f'    z := x + y + c * {f%3}')
            code.append(f'    R x + y + z')
            code.append(f'}}')
            code.append('')
            lines += 7

code.append('F main() -> i64 {')
code.append('    result := m0_f0(1, 2)')
code.append('    R result')
code.append('}')

print('\n'.join(code))
" > "$main_file"

    local actual_lines
    actual_lines=$(wc -l < "$main_file" | tr -d ' ')
    local actual_bytes
    actual_bytes=$(wc -c < "$main_file" | tr -d ' ')

    echo "$actual_lines $actual_bytes $main_file"
}

# Measure compilation time and memory for a given file
measure_compile() {
    local vais_file=$1
    local label=$2
    local runs=${3:-5}

    log "Benchmarking: $label ($runs runs)"

    local times=()
    local peak_mem=0

    for i in $(seq 1 "$runs"); do
        local output_ll="${vais_file%.vais}.ll"

        # Time measurement
        local start_ns end_ns elapsed_ms
        start_ns=$(python3 -c "import time; print(int(time.time_ns()))")

        "$VAISC" "$vais_file" 2>/dev/null || true

        end_ns=$(python3 -c "import time; print(int(time.time_ns()))")
        elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
        times+=("$elapsed_ms")

        rm -f "$output_ll"
    done

    # Memory measurement (single run with /usr/bin/time on macOS)
    local mem_output
    if [[ "$(uname)" == "Darwin" ]]; then
        mem_output=$(/usr/bin/time -l "$VAISC" "$vais_file" 2>&1 || true)
        peak_mem=$(echo "$mem_output" | grep "maximum resident" | awk '{print $1}')
        peak_mem=$((peak_mem / 1024)) # bytes -> KB
    else
        mem_output=$(/usr/bin/time -v "$VAISC" "$vais_file" 2>&1 || true)
        peak_mem=$(echo "$mem_output" | grep "Maximum resident" | awk '{print $NF}')
    fi
    rm -f "${vais_file%.vais}.ll"

    # Calculate stats
    local sum=0 min=999999999 max=0
    for t in "${times[@]}"; do
        sum=$((sum + t))
        (( t < min )) && min=$t
        (( t > max )) && max=$t
    done
    local avg=$((sum / runs))

    local peak_mem_mb
    peak_mem_mb=$(echo "scale=1; $peak_mem / 1024" | bc)

    printf "  %-30s  avg: %6d ms  min: %6d ms  max: %6d ms  mem: %s MB\n" \
        "$label" "$avg" "$min" "$max" "$peak_mem_mb"
}

# Run criterion benchmarks
run_criterion() {
    log "Running criterion large-scale benchmarks..."
    cargo bench -p vais-benches --bench largescale_bench -- --save-baseline largescale
    ok "Criterion results saved. Report: target/criterion/report/index.html"
}

# Main
main() {
    if [[ "${1:-}" == "--criterion" ]]; then
        run_criterion
        exit 0
    fi

    build_compiler

    mkdir -p "$BENCH_DIR"

    local sizes
    if [[ -n "${1:-}" ]]; then
        sizes=("$1")
    else
        sizes=(1000 5000 10000 25000 50000)
    fi

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Vais Large-Scale Compilation Benchmark"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    local results=()

    for target in "${sizes[@]}"; do
        log "Generating ${target}-line project..."
        read -r actual_lines actual_bytes main_file <<< "$(generate_project "$target")"
        ok "Generated: ${actual_lines} lines, ${actual_bytes} bytes"

        measure_compile "$main_file" "${actual_lines}lines" 5
        echo ""
    done

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Throughput calculation
    log "Calculating throughput..."
    for target in "${sizes[@]}"; do
        local info
        info=$(generate_project "$target")
        local lines bytes file
        read -r lines bytes file <<< "$info"

        local start_ns end_ns elapsed_ms throughput_klps throughput_mbps
        start_ns=$(python3 -c "import time; print(int(time.time_ns()))")
        "$VAISC" "$file" 2>/dev/null || true
        end_ns=$(python3 -c "import time; print(int(time.time_ns()))")
        elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))

        rm -f "${file%.vais}.ll"

        if (( elapsed_ms > 0 )); then
            throughput_klps=$(echo "scale=1; $lines / $elapsed_ms" | bc)
            throughput_mbps=$(echo "scale=2; $bytes / $elapsed_ms / 1000" | bc)
        else
            throughput_klps="inf"
            throughput_mbps="inf"
        fi

        printf "  %-12s  %s K lines/s  |  %s MB/s\n" "${lines}lines" "$throughput_klps" "$throughput_mbps"
    done

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    ok "Benchmark complete. Results in $BENCH_DIR"

    # Cleanup
    rm -rf "$BENCH_DIR"
}

main "$@"
