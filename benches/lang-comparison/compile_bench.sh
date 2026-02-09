#!/bin/bash
# Compile speed benchmark: Vais vs Rust vs Go vs C vs Python
# Uses hyperfine for accurate measurement

set -e
BENCH_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$BENCH_DIR"

echo "================================================================"
echo "COMPILE SPEED BENCHMARK — Vais vs Rust vs Go vs C vs Python"
echo "================================================================"
echo ""
echo "Machine: $(uname -m) / $(uname -s)"
echo "Date: $(date +%Y-%m-%d)"
echo ""

# Build vaisc first
echo "Building vaisc..."
cargo build --release --bin vaisc -q 2>/dev/null
VAISC="$(cd ../.. && pwd)/target/release/vaisc"

PROGRAMS="fibonacci quicksort http_types linked_list"

for prog in $PROGRAMS; do
    echo ""
    echo "================================================================"
    echo "Program: $prog"
    echo "================================================================"
    echo ""

    # Clean previous outputs
    rm -f /tmp/bench_*

    echo "--- Compilation Speed (cold, single file) ---"
    echo ""

    # Vais
    if [ -f "vais/${prog}.vais" ]; then
        echo "[Vais] vaisc build → LLVM IR"
        hyperfine --warmup 2 --min-runs 10 \
            "$VAISC build vais/${prog}.vais --emit-ir -o /tmp/bench_vais.ll" \
            --export-json /tmp/bench_vais_${prog}.json 2>&1 | grep -E "Time|Range"
        echo ""
    fi

    # Rust
    if [ -f "rust/${prog}.rs" ]; then
        echo "[Rust] rustc → binary"
        hyperfine --warmup 2 --min-runs 10 \
            "rustc rust/${prog}.rs -o /tmp/bench_rust" \
            --export-json /tmp/bench_rust_${prog}.json 2>&1 | grep -E "Time|Range"
        echo ""
    fi

    # Go
    if [ -f "go/${prog}.go" ]; then
        echo "[Go] go build → binary"
        hyperfine --warmup 2 --min-runs 10 \
            "go build -o /tmp/bench_go go/${prog}.go" \
            --export-json /tmp/bench_go_${prog}.json 2>&1 | grep -E "Time|Range"
        echo ""
    fi

    # C (clang)
    if [ -f "c/${prog}.c" ]; then
        echo "[C] clang → binary"
        hyperfine --warmup 2 --min-runs 10 \
            "clang c/${prog}.c -o /tmp/bench_c" \
            --export-json /tmp/bench_c_${prog}.json 2>&1 | grep -E "Time|Range"
        echo ""
    fi

    # Python (startup + parse time)
    if [ -f "python/${prog}.py" ]; then
        echo "[Python] python3 -c 'import ast; ast.parse(...)' (parse only)"
        hyperfine --warmup 2 --min-runs 10 \
            "python3 -c \"import ast; ast.parse(open('python/${prog}.py').read())\"" \
            --export-json /tmp/bench_python_${prog}.json 2>&1 | grep -E "Time|Range"
        echo ""
    fi
done

echo ""
echo "================================================================"
echo "SUMMARY"
echo "================================================================"
echo ""

# Extract mean times from JSON
python3 -c "
import json, os, glob

languages = ['vais', 'rust', 'go', 'c', 'python']
programs = ['fibonacci', 'quicksort', 'http_types', 'linked_list']

print(f\"{'Program':<15} {'Vais':>10} {'Rust':>10} {'Go':>10} {'C':>10} {'Python':>10}\")
print('-' * 70)

totals = {l: 0.0 for l in languages}
counts = {l: 0 for l in languages}

for prog in programs:
    row = [f'{prog:<15}']
    for lang in languages:
        path = f'/tmp/bench_{lang}_{prog}.json'
        if os.path.exists(path):
            with open(path) as f:
                data = json.load(f)
            mean = data['results'][0]['mean']
            totals[lang] += mean
            counts[lang] += 1
            if mean < 1.0:
                row.append(f'{mean*1000:>8.1f}ms')
            else:
                row.append(f'{mean:>8.2f}s ')
        else:
            row.append(f'{'N/A':>10}')
    print(' '.join(row))

print('-' * 70)
row = ['AVERAGE        ']
for lang in languages:
    if counts[lang] > 0:
        avg = totals[lang] / counts[lang]
        if avg < 1.0:
            row.append(f'{avg*1000:>8.1f}ms')
        else:
            row.append(f'{avg:>8.2f}s ')
    else:
        row.append(f'{'N/A':>10}')
print(' '.join(row))

# Print Vais vs others
print()
print('--- Vais compile speed comparison ---')
vais_avg = totals['vais'] / counts['vais'] if counts['vais'] > 0 else 0
for lang in languages:
    if lang == 'vais' or counts[lang] == 0:
        continue
    other_avg = totals[lang] / counts[lang]
    if vais_avg > 0:
        ratio = other_avg / vais_avg
        print(f'Vais vs {lang}: {ratio:.1f}x ({\"faster\" if ratio > 1 else \"slower\"})')
"
