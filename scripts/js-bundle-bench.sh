#!/usr/bin/env bash
set -euo pipefail

# JS Bundle Size Benchmark for vais-codegen-js
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== Vais JS Bundle Size Benchmark ==="
echo ""

# Build vaisc first
echo "Building vaisc..."
cargo build --bin vaisc --release -q 2>/dev/null || cargo build --bin vaisc -q
echo ""

VAISC="cargo run --bin vaisc --"

TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

# Create test programs of varying complexity
# Test 1: Hello world (minimal)
cat > "$TMPDIR/hello.vais" << 'EOF'
F main() -> i64 = 42
EOF

# Test 2: Functions
cat > "$TMPDIR/functions.vais" << 'EOF'
F add(a: i64, b: i64) -> i64 = a + b
F multiply(a: i64, b: i64) -> i64 = a * b
F main() -> i64 {
  x := add(10, 20)
  multiply(x, 2)
}
EOF

# Test 3: Struct + methods
cat > "$TMPDIR/struct.vais" << 'EOF'
S Point {
  x: f64,
  y: f64
}
F main() -> i64 = 0
EOF

# Test 4: Enum
cat > "$TMPDIR/enum.vais" << 'EOF'
E Color {
  Red,
  Green,
  Blue
}
F main() -> i64 = 0
EOF

# Test 5: Control flow
cat > "$TMPDIR/control.vais" << 'EOF'
F abs(x: i64) -> i64 {
  I x < 0 { 0 - x } E { x }
}
F main() -> i64 = abs(-42)
EOF

# Test 6: Loop
cat > "$TMPDIR/loop.vais" << 'EOF'
F sum_to_n(n: i64) -> i64 {
  total := mut 0
  i := mut 0
  L {
    I i >= n { B }
    total = total + i
    i = i + 1
  }
  total
}
F main() -> i64 = sum_to_n(10)
EOF

# Test 7: Match expression
cat > "$TMPDIR/match.vais" << 'EOF'
E Status {
  Active,
  Inactive,
  Pending
}
F check_status(s: Status) -> i64 {
  M s {
    Active => 1,
    Inactive => 0,
    Pending => 2
  }
}
F main() -> i64 = 0
EOF

# Test 8: Array operations
cat > "$TMPDIR/array.vais" << 'EOF'
F sum_array(arr: [i64], len: i64) -> i64 {
  total := mut 0
  i := mut 0
  L {
    I i >= len { B }
    total = total + arr[i]
    i = i + 1
  }
  total
}
F main() -> i64 = 0
EOF

# Compile and measure
printf "%-20s %10s %10s\n" "Program" "JS Size" "Lines"
printf "%-20s %10s %10s\n" "-------" "-------" "-----"

total_size=0
total_lines=0
success_count=0
fail_count=0

for src in hello functions struct enum control loop match array; do
  if $VAISC build "$TMPDIR/$src.vais" --target js -o "$TMPDIR/$src.js" 2>/dev/null; then
    if [ -f "$TMPDIR/$src.js" ]; then
      size=$(wc -c < "$TMPDIR/$src.js" | tr -d ' ')
      lines=$(wc -l < "$TMPDIR/$src.js" | tr -d ' ')
      total_size=$((total_size + size))
      total_lines=$((total_lines + lines))
      success_count=$((success_count + 1))
      printf "%-20s %8s B %8s\n" "$src.vais" "$size" "$lines"
    else
      printf "%-20s %10s %10s\n" "$src.vais" "FAILED" "-"
      fail_count=$((fail_count + 1))
    fi
  else
    printf "%-20s %10s %10s\n" "$src.vais" "FAILED" "-"
    fail_count=$((fail_count + 1))
  fi
done

echo ""
printf "%-20s %8s B %8s\n" "Total" "$total_size" "$total_lines"
if [ $success_count -gt 0 ]; then
  avg_size=$((total_size / success_count))
  avg_lines=$((total_lines / success_count))
  printf "%-20s %8s B %8s\n" "Average" "$avg_size" "$avg_lines"
fi
echo ""
echo "Results: $success_count succeeded, $fail_count failed"
echo ""
echo "=== Done ==="
