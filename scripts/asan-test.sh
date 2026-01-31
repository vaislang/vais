#!/bin/bash
# Run compiler tests with AddressSanitizer
# Usage: ./scripts/asan-test.sh [package-name]
#
# Prerequisites:
#   - Rust nightly toolchain: rustup install nightly
#   - rust-src component: rustup component add rust-src --toolchain nightly

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "========================================"
echo "Running AddressSanitizer Tests"
echo "========================================"

# Check for nightly toolchain
if ! rustup run nightly rustc --version &> /dev/null; then
    echo "Error: Rust nightly toolchain required"
    echo "Install with: rustup install nightly"
    exit 1
fi

# Check for rust-src component
if ! rustup component list --toolchain nightly | grep -q "rust-src (installed)"; then
    echo "Installing rust-src component..."
    rustup component add rust-src --toolchain nightly
fi

# Detect host target
HOST_TARGET=$(rustc -vV | awk '/host/{print $2}')
echo "Host target: $HOST_TARGET"

# Create log directory
mkdir -p target/asan-logs

# Function to run ASan tests for a package
run_asan_test() {
    local package=$1
    local log_file="target/asan-logs/asan-${package}.log"

    echo ""
    echo "========================================"
    echo "Testing package: $package"
    echo "========================================"

    RUSTFLAGS="-Z sanitizer=address" \
    RUSTDOCFLAGS="-Z sanitizer=address" \
    cargo +nightly test -p "$package" \
        -Z build-std --target "$HOST_TARGET" \
        -- --test-threads=1 2>&1 | tee "$log_file"

    echo "Log saved to: $log_file"
}

# If a specific package is provided, test only that package
if [ $# -eq 1 ]; then
    run_asan_test "$1"
else
    # Test core packages
    echo "Testing core compiler packages with AddressSanitizer..."
    echo ""

    # Test vais-parser
    run_asan_test "vais-parser"

    # Test vais-types
    run_asan_test "vais-types"

    # Test vais-codegen
    run_asan_test "vais-codegen"

    # Test vais-lexer
    run_asan_test "vais-lexer"

    # Test vais-ast
    run_asan_test "vais-ast"

    # Test vais-gc (has unsafe code)
    run_asan_test "vais-gc"

    # Test vaisc integration tests
    echo ""
    echo "========================================"
    echo "Testing vaisc integration tests"
    echo "========================================"

    RUSTFLAGS="-Z sanitizer=address" \
    RUSTDOCFLAGS="-Z sanitizer=address" \
    cargo +nightly test -p vaisc \
        -Z build-std --target "$HOST_TARGET" \
        -- memory_safety --test-threads=1 2>&1 | tee target/asan-logs/asan-vaisc.log
fi

echo ""
echo "========================================"
echo "AddressSanitizer tests completed!"
echo "Logs saved to target/asan-logs/"
echo "========================================"
