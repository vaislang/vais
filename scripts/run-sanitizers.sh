#!/bin/bash
# Vais Compiler Sanitizer Test Script
#
# Usage:
#   ./scripts/run-sanitizers.sh [address|memory|undefined|thread|all]
#
# Prerequisites:
#   - Rust nightly toolchain: rustup install nightly
#   - rust-src component: rustup component add rust-src --toolchain nightly

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "========================================"
echo "Running Vais Compiler Sanitizer Tests"
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

# Create target directory for logs
mkdir -p target/sanitizer-logs

# Function to run tests with a sanitizer
run_sanitizer() {
    local sanitizer=$1
    local name=$2

    echo ""
    echo "========================================"
    echo "Running $name"
    echo "========================================"

    local log_file="target/sanitizer-logs/sanitizer-${sanitizer}.log"

    case $sanitizer in
        "address")
            # Address Sanitizer - detects memory errors
            RUSTFLAGS="-Zsanitizer=address" \
            RUSTDOCFLAGS="-Zsanitizer=address" \
            cargo +nightly test --workspace \
                -Zbuild-std --target $(rustc -vV | grep host | cut -d' ' -f2) \
                -- --test-threads=1 2>&1 | tee "$log_file"
            ;;
        "memory")
            # Memory Sanitizer - detects uninitialized memory reads
            # Note: MSAN requires all dependencies to be instrumented
            echo "MSAN: Testing vais-lexer only (requires instrumented libc)"
            RUSTFLAGS="-Zsanitizer=memory -Zsanitizer-memory-track-origins" \
            cargo +nightly test -p vais-lexer \
                -Zbuild-std --target $(rustc -vV | grep host | cut -d' ' -f2) \
                -- --test-threads=1 2>&1 | tee "$log_file" || {
                echo "MSAN tests may fail due to uninstrumented dependencies"
                return 0
            }
            ;;
        "undefined")
            # Undefined Behavior Sanitizer
            RUSTFLAGS="-Zsanitizer=undefined" \
            cargo +nightly test --workspace \
                -Zbuild-std --target $(rustc -vV | grep host | cut -d' ' -f2) \
                -- --test-threads=1 2>&1 | tee "$log_file"
            ;;
        "thread")
            # Thread Sanitizer - detects data races
            RUSTFLAGS="-Zsanitizer=thread" \
            cargo +nightly test --workspace \
                -Zbuild-std --target $(rustc -vV | grep host | cut -d' ' -f2) \
                -- --test-threads=1 2>&1 | tee "$log_file"
            ;;
    esac

    echo "Log saved to: $log_file"
}

# Parse arguments
SANITIZER=${1:-"all"}

case $SANITIZER in
    "address"|"asan")
        run_sanitizer "address" "Address Sanitizer (ASAN)"
        ;;
    "memory"|"msan")
        run_sanitizer "memory" "Memory Sanitizer (MSAN)"
        ;;
    "undefined"|"ubsan")
        run_sanitizer "undefined" "Undefined Behavior Sanitizer (UBSAN)"
        ;;
    "thread"|"tsan")
        run_sanitizer "thread" "Thread Sanitizer (TSAN)"
        ;;
    "all")
        run_sanitizer "address" "Address Sanitizer (ASAN)"
        run_sanitizer "undefined" "Undefined Behavior Sanitizer (UBSAN)"
        echo ""
        echo "Note: MSAN and TSAN skipped by default. Run explicitly if needed:"
        echo "  ./scripts/run-sanitizers.sh memory"
        echo "  ./scripts/run-sanitizers.sh thread"
        ;;
    *)
        echo "Usage: $0 [address|memory|undefined|thread|all]"
        echo ""
        echo "Sanitizers:"
        echo "  address (asan)    - Detect memory errors (buffer overflow, use-after-free)"
        echo "  memory (msan)     - Detect uninitialized memory reads"
        echo "  undefined (ubsan) - Detect undefined behavior"
        echo "  thread (tsan)     - Detect data races"
        echo "  all               - Run ASAN and UBSAN (recommended)"
        exit 1
        ;;
esac

echo ""
echo "========================================"
echo "Sanitizer tests completed!"
echo "Logs saved to target/sanitizer-logs/"
echo "========================================"
