#!/usr/bin/env bash
set -euo pipefail

# Incremental Compilation Benchmark for Vais (Phase 42)
# Measures: cold build, no-change rebuild, 1-file change rebuild
# Compares: Vais (per-module) vs C (make -j4)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BENCH_DIR="$PROJECT_ROOT/target/incremental-bench"
VAISC="$PROJECT_ROOT/target/release/vaisc"
GEN="$SCRIPT_DIR/gen_bench_project.py"

CYAN='\033[0;36m'
GREEN='\033[0;32m'
NC='\033[0m'

log() { echo -e "${CYAN}[bench]${NC} $*"; }

# Measure median of 3 runs (ms)
measure() {
    local cmd="$1"
    local times=()
    for i in 1 2 3; do
        local s e
        s=$(python3 -c "import time; print(int(time.time_ns()))")
        eval "$cmd" >/dev/null 2>&1 || true
        e=$(python3 -c "import time; print(int(time.time_ns()))")
        times+=("$(( (e - s) / 1000000 ))")
    done
    printf '%s\n' "${times[@]}" | sort -n | sed -n '2p'
}

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Vais Incremental Compilation Benchmark (Phase 42)"
echo "  Per-module codegen + IR-hash .o caching + rayon parallel"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

for TL in 3000 12000 30000; do
    echo ""
    log "=== ~${TL} lines ==="

    vd="$BENCH_DIR/vais_${TL}l"
    cd_dir="$BENCH_DIR/c_${TL}l"
    rm -rf "$vd" "$cd_dir"

    vm=$(python3 "$GEN" vais "$TL" "$vd")
    python3 "$GEN" c "$TL" "$cd_dir" >/dev/null
    vl=$(cat "$vd"/*.vais | wc -l | tr -d ' ')
    log "  Vais: $vm modules, ${vl} lines"

    # --- Vais ---
    rm -rf "$vd/.vais-cache"
    vc=$(measure "$VAISC build $vd/main.vais -O2 -o $vd/bench")
    vn=$(measure "$VAISC build $vd/main.vais -O2 -o $vd/bench")
    echo "F bench_extra_fn(x: i64) -> i64 { R x + 42 }" >> "$vd/mod0.vais"
    v1=$(measure "$VAISC build $vd/main.vais -O2 -o $vd/bench")

    # --- C ---
    make -C "$cd_dir" clean >/dev/null 2>&1 || true
    cc=$(measure "make -C $cd_dir -j4")
    cn=$(measure "make -C $cd_dir -j4")
    echo "// modified" >> "$cd_dir/mod0.c"
    c1=$(measure "make -C $cd_dir -j4")

    printf "\n  %-20s  %8s  %10s  %8s\n" "" "Cold" "No-change" "1-file"
    printf "  %-20s  %6sms  %8sms  %6sms\n" "Vais (per-module)" "$vc" "$vn" "$v1"
    printf "  %-20s  %6sms  %8sms  %6sms\n" "C (make -j4)" "$cc" "$cn" "$c1"
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
