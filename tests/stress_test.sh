#!/bin/bash
# Stress test for vais compiler scalability
# Tests the compiler with progressively larger inputs

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMP_DIR="$PROJECT_ROOT/tmp/stress_test"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   VAIS COMPILER STRESS TEST SUITE${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Create temp directory
mkdir -p "$TEMP_DIR"

# Function to generate test file
generate_test_file() {
    local size=$1
    local output_file=$2

    echo -e "${YELLOW}Generating test file with $size items...${NC}"

    # Calculate distribution: 30% structs, 60% functions, 10% enums
    local struct_count=$((size * 30 / 100))
    local func_count=$((size * 60 / 100))
    local enum_count=$((size * 10 / 100))

    > "$output_file"

    # Generate structs
    for ((i=0; i<struct_count; i++)); do
        echo "S Struct${i} { field_a: i64, field_b: i64, field_c: bool }" >> "$output_file"
    done

    # Generate enums
    for ((i=0; i<enum_count; i++)); do
        echo "E Enum${i} { Variant${i}A, Variant${i}B(i64) }" >> "$output_file"
    done

    # Generate functions
    for ((i=0; i<func_count; i++)); do
        echo "F func${i}(x: i64) -> i64 = x + ${i}" >> "$output_file"
    done

    local actual_size=$(wc -l < "$output_file" | tr -d ' ')
    echo -e "${GREEN}Generated $actual_size lines${NC}"
}

# Function to compile and measure
compile_and_measure() {
    local input_file=$1
    local size=$2

    echo -e "${YELLOW}Compiling $size items...${NC}"

    # Measure compile time
    local start_time=$(date +%s%N)

    if "$PROJECT_ROOT/target/release/vaisc" "$input_file" --emit=llvm-ir -o "$TEMP_DIR/output_${size}.ll" 2>&1; then
        local end_time=$(date +%s%N)
        local elapsed=$((($end_time - $start_time) / 1000000)) # Convert to ms

        echo -e "${GREEN}✓ Success${NC} - Compilation time: ${elapsed} ms"

        # Calculate throughput
        local lines=$(wc -l < "$input_file" | tr -d ' ')
        local throughput=$(awk "BEGIN {printf \"%.1f\", ($lines / $elapsed) * 1000}")
        echo -e "  Throughput: ${throughput} lines/sec"

        return 0
    else
        echo -e "${RED}✗ Failed${NC}"
        return 1
    fi
}

# Check if vaisc is built
if [ ! -f "$PROJECT_ROOT/target/release/vaisc" ]; then
    echo -e "${YELLOW}Building vaisc in release mode...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --release --bin vaisc
    echo ""
fi

# Test sizes (number of items)
TEST_SIZES=(100 500 1000 2500 5000 10000)

echo -e "${BLUE}Running stress tests...${NC}"
echo ""

# Track results
declare -a RESULTS

# Run tests
for size in "${TEST_SIZES[@]}"; do
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}Test: $size items${NC}"
    echo -e "${BLUE}========================================${NC}"

    test_file="$TEMP_DIR/test_${size}.vais"

    # Generate test file
    generate_test_file "$size" "$test_file"

    # Compile and measure
    if compile_and_measure "$test_file" "$size"; then
        RESULTS+=("$size:SUCCESS")
    else
        RESULTS+=("$size:FAILED")
    fi

    echo ""
done

# Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}           SUMMARY${NC}"
echo -e "${BLUE}========================================${NC}"

for result in "${RESULTS[@]}"; do
    size="${result%%:*}"
    status="${result##*:}"

    if [ "$status" = "SUCCESS" ]; then
        echo -e "${size} items: ${GREEN}✓ PASS${NC}"
    else
        echo -e "${size} items: ${RED}✗ FAIL${NC}"
    fi
done

echo ""
echo -e "${BLUE}Test artifacts in: $TEMP_DIR${NC}"
echo ""

# Cleanup option
read -p "Delete test artifacts? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf "$TEMP_DIR"
    echo -e "${GREEN}Cleanup complete${NC}"
fi

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   STRESS TEST COMPLETE${NC}"
echo -e "${BLUE}========================================${NC}"
