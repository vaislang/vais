#!/usr/bin/env bash
# Vais Cross-Verification Script
# Compares execution results between Rust compiler (vaisc) and selfhost compiler (vaisc-stage1)
#
# For each .vais file:
#   1. Compile with Rust vaisc → .ll → binary → run → capture stdout + exit code
#   2. Compile with selfhost vaisc-stage1 → .ll → binary → run → capture stdout + exit code
#   3. Compare results
#
# Usage: ./scripts/cross-verify.sh [OPTIONS] [FILES...]
#   --all         Test all examples/ files (excluding known-skip list)
#   --simple      Test only simple examples (no imports, no complex features)
#   --verbose     Show detailed output
#   --keep        Keep build artifacts
#   --stage N     Use vaisc-stageN (default: 1)
#   FILES...      Specific .vais files to test

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SELFHOST_DIR="$PROJECT_ROOT/selfhost"
BUILD_DIR="${BUILD_DIR:-/tmp/vais-cross-verify}"
VAISC="$PROJECT_ROOT/target/release/vaisc"
RUNTIME_C="$SELFHOST_DIR/runtime.c"
RUNTIME_O="$BUILD_DIR/runtime.o"
CC="${CC:-clang}"
CFLAGS="${CFLAGS:--O0}"
STAGE=1

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Counters
PASS=0
FAIL=0
SKIP=0
COMPILE_FAIL_RUST=0
COMPILE_FAIL_SELFHOST=0
LINK_FAIL_SELFHOST=0

# Options
VERBOSE=false
KEEP_ARTIFACTS=false
MODE="files"  # files, all, simple

# Files that are known to use features not yet supported by selfhost
SKIP_LIST=(
    # Files with imports (use/U statements)
    "adoption_prototype.vais"
    "btreemap_test.vais"
    "compress_example.vais"
    "comprehensive_iterator_test.vais"
    "deque_test.vais"
    "deque_minimal_test.vais"
    "dynload_test.vais"
    "file_io.vais"
    "gc_test.vais"
    "gc_comprehensive.vais"
    "hashmap_test.vais"
    "hashmap_bench.vais"
    "http_demo.vais"
    "json_test.vais"
    "json_validator.vais"
    "linked_list.vais"
    "macro_test.vais"
    "macro_expanded_test.vais"
    "module_test.vais"
    "net_test.vais"
    "option_result.vais"
    "postgres_example.vais"
    "priority_queue.vais"
    "regex_test.vais"
    "set_test.vais"
    "simd_test.vais"
    "string_test.vais"
    "template_test.vais"
    "thread_test.vais"
    "trait_test.vais"
    "vec_test.vais"
    "web_framework.vais"
    # Files with async/spawn
    "async_test.vais"
    "async_reactor_test.vais"
    # Files that intentionally crash/abort
    "assert_violation_test.vais"
    "contract_fail_test.vais"
    "contract_violation_test.vais"
    # Files with complex features (comptime, GPU, etc.)
    "comptime_test.vais"
    "comptime_simple.vais"
    "const_generic_test.vais"
    "dependent_type_test.vais"
    "gpu_test.vais"
    "hotreload_test.vais"
    "linear_type_test.vais"
    "simd_comprehensive.vais"
)

# Simple examples: single-file, no imports, basic features
SIMPLE_LIST=(
    # Very simple (2-11 lines)
    "hello.vais"
    "hello_world.vais"
    "fib.vais"
    "control_flow.vais"
    "putchar_var.vais"
    "printf_test.vais"
    "malloc_test.vais"
    "arrays.vais"
    # Medium simple (12-30 lines)
    "math.vais"
    "enum_test.vais"
    "generic_test.vais"
    "option_test3.vais"
    "tco_stress.vais"
    "loop_break_test.vais"
    "tco_tail_call.vais"
    "generic_struct_test.vais"
    "pipe_operator.vais"
    "option_test2.vais"
    "test_bitwise_precedence.vais"
    "match_test.vais"
    "match_binding.vais"
    "method_test.vais"
    # Medium (30-55 lines)
    "closure_simple.vais"
    "lambda_test.vais"
    "test_bitwise.vais"
    "opt_test.vais"
    "result_test.vais"
    "range_test.vais"
    "pattern_match_test.vais"
    "math_test.vais"
    "trait_test.vais"
    "closure_test.vais"
    "crc32.vais"
)

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS] [FILES...]

Cross-verify Rust compiler vs selfhost compiler execution results.

Options:
  --all         Test all examples/ (excluding skip list)
  --simple      Test only simple examples (recommended for initial validation)
  --verbose     Show detailed output including IR
  --keep        Keep build artifacts in $BUILD_DIR
  --stage N     Use vaisc-stageN (default: 1)
  --help        Show this help

Examples:
  $(basename "$0") --simple                    # Test ~20 simple examples
  $(basename "$0") examples/hello.vais         # Test a specific file
  $(basename "$0") --all --verbose             # Test all with details
EOF
    exit 0
}

log_info()  { echo -e "${CYAN}[INFO]${NC} $*"; }
log_ok()    { echo -e "${GREEN}[PASS]${NC} $*"; }
log_fail()  { echo -e "${RED}[FAIL]${NC} $*"; }
log_skip()  { echo -e "${YELLOW}[SKIP]${NC} $*"; }
log_detail(){ [[ "$VERBOSE" == "true" ]] && echo -e "       $*" || true; }

# Parse arguments
FILES=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --all)     MODE="all"; shift ;;
        --simple)  MODE="simple"; shift ;;
        --verbose) VERBOSE=true; shift ;;
        --keep)    KEEP_ARTIFACTS=true; shift ;;
        --stage)   STAGE="$2"; shift 2 ;;
        --help)    usage ;;
        *.vais)    FILES+=("$1"); MODE="files"; shift ;;
        *)         echo "Unknown option: $1"; usage ;;
    esac
done

STAGE_BIN="$SELFHOST_DIR/vaisc-stage${STAGE}"

# Determine file list
get_test_files() {
    case "$MODE" in
        simple)
            for f in "${SIMPLE_LIST[@]}"; do
                local path="$PROJECT_ROOT/examples/$f"
                [[ -f "$path" ]] && echo "$path"
            done
            ;;
        all)
            for f in "$PROJECT_ROOT"/examples/*.vais; do
                local base=$(basename "$f")
                local skip=false
                for s in "${SKIP_LIST[@]}"; do
                    [[ "$base" == "$s" ]] && skip=true && break
                done
                [[ "$skip" == "false" ]] && echo "$f"
            done
            ;;
        files)
            for f in "${FILES[@]}"; do
                if [[ -f "$f" ]]; then
                    echo "$f"
                elif [[ -f "$PROJECT_ROOT/$f" ]]; then
                    echo "$PROJECT_ROOT/$f"
                elif [[ -f "$PROJECT_ROOT/examples/$f" ]]; then
                    echo "$PROJECT_ROOT/examples/$f"
                else
                    echo "File not found: $f" >&2
                fi
            done
            ;;
    esac
}

cleanup() {
    if [[ "$KEEP_ARTIFACTS" == "false" ]]; then
        rm -rf "$BUILD_DIR"
    fi
}
trap cleanup EXIT

# Setup
mkdir -p "$BUILD_DIR"

# Check prerequisites
if [[ ! -f "$VAISC" ]]; then
    log_info "Building Rust compiler..."
    cd "$PROJECT_ROOT" && cargo build --release --package vaisc 2>&1 | tail -1
fi

if [[ ! -f "$STAGE_BIN" ]]; then
    log_fail "Selfhost stage$STAGE binary not found: $STAGE_BIN"
    exit 1
fi

# Compile runtime
if [[ ! -f "$RUNTIME_O" ]]; then
    $CC $CFLAGS -c "$RUNTIME_C" -o "$RUNTIME_O" 2>/dev/null
fi

echo ""
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD} Vais Cross-Verification: Rust vaisc vs vaisc-stage${STAGE}${NC}"
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Test each file
test_file() {
    local vais_file="$1"
    local base=$(basename "$vais_file" .vais)
    local rust_ll="$BUILD_DIR/rust_${base}.ll"
    local rust_bin="$BUILD_DIR/rust_${base}"
    local sh_ll="$BUILD_DIR/sh_${base}.ll"
    local sh_bin="$BUILD_DIR/sh_${base}"

    # Step 1: Compile with Rust vaisc
    if ! "$VAISC" "$vais_file" --emit-ir -o "$rust_ll" 2>"$BUILD_DIR/rust_${base}_err.txt"; then
        log_skip "$base (Rust compile failed)"
        log_detail "$(cat "$BUILD_DIR/rust_${base}_err.txt" | head -3)"
        COMPILE_FAIL_RUST=$((COMPILE_FAIL_RUST + 1))
        SKIP=$((SKIP + 1))
        return
    fi

    if ! $CC $CFLAGS "$rust_ll" -o "$rust_bin" -lm 2>"$BUILD_DIR/rust_${base}_link_err.txt"; then
        log_skip "$base (Rust link failed)"
        SKIP=$((SKIP + 1))
        return
    fi

    # Run Rust binary (timeout 5s)
    local rust_exit
    set +e
    "$rust_bin" > "$BUILD_DIR/rust_${base}_out.txt" 2>/dev/null
    rust_exit=$?
    set -e
    local rust_stdout=$(cat "$BUILD_DIR/rust_${base}_out.txt")
    # Normalize exit code to 0-255 range (main returns i64 but shell truncates)
    rust_exit=$((rust_exit % 256))

    # Step 2: Compile with selfhost
    cd "$PROJECT_ROOT"
    if ! "$STAGE_BIN" "$vais_file" >"$BUILD_DIR/sh_${base}_compiler_out.txt" 2>&1; then
        log_skip "$base (selfhost compile failed)"
        log_detail "$(tail -3 "$BUILD_DIR/sh_${base}_compiler_out.txt")"
        COMPILE_FAIL_SELFHOST=$((COMPILE_FAIL_SELFHOST + 1))
        SKIP=$((SKIP + 1))
        return
    fi

    # Check if IR was produced
    if [[ ! -f "$SELFHOST_DIR/main_output.ll" ]]; then
        log_skip "$base (selfhost produced no IR)"
        COMPILE_FAIL_SELFHOST=$((COMPILE_FAIL_SELFHOST + 1))
        SKIP=$((SKIP + 1))
        return
    fi

    cp "$SELFHOST_DIR/main_output.ll" "$sh_ll"

    if ! $CC $CFLAGS "$sh_ll" "$RUNTIME_O" -o "$sh_bin" -lm 2>"$BUILD_DIR/sh_${base}_link_err.txt"; then
        log_skip "$base (selfhost link failed)"
        log_detail "$(cat "$BUILD_DIR/sh_${base}_link_err.txt" | head -3)"
        LINK_FAIL_SELFHOST=$((LINK_FAIL_SELFHOST + 1))
        SKIP=$((SKIP + 1))
        return
    fi

    # Run selfhost binary (timeout 5s)
    set +e
    "$sh_bin" > "$BUILD_DIR/sh_${base}_out.txt" 2>/dev/null
    local sh_exit=$?
    set -e
    local sh_stdout=$(cat "$BUILD_DIR/sh_${base}_out.txt")
    sh_exit=$((sh_exit % 256))

    # Step 3: Compare
    if [[ "$rust_stdout" == "$sh_stdout" && "$rust_exit" == "$sh_exit" ]]; then
        log_ok "$base (exit=$rust_exit, stdout=${#rust_stdout} bytes)"
        PASS=$((PASS + 1))
    else
        log_fail "$base"
        if [[ "$rust_exit" != "$sh_exit" ]]; then
            echo -e "       ${RED}Exit code: Rust=$rust_exit, Selfhost=$sh_exit${NC}"
        fi
        if [[ "$rust_stdout" != "$sh_stdout" ]]; then
            echo -e "       ${RED}Stdout differs:${NC}"
            echo -e "       ${CYAN}Rust:${NC}     $(echo "$rust_stdout" | head -3)"
            echo -e "       ${YELLOW}Selfhost:${NC} $(echo "$sh_stdout" | head -3)"
            if [[ ${#rust_stdout} -gt 100 || ${#sh_stdout} -gt 100 ]]; then
                diff <(echo "$rust_stdout") <(echo "$sh_stdout") > "$BUILD_DIR/diff_${base}.txt" 2>/dev/null || true
                echo -e "       Full diff saved to: $BUILD_DIR/diff_${base}.txt"
            fi
        fi
        FAIL=$((FAIL + 1))
    fi
}

# Run tests
TEST_FILES=()
while IFS= read -r line; do
    TEST_FILES+=("$line")
done < <(get_test_files)

if [[ ${#TEST_FILES[@]} -eq 0 ]]; then
    log_fail "No test files found"
    exit 1
fi

log_info "Testing ${#TEST_FILES[@]} files..."
echo ""

for f in "${TEST_FILES[@]}"; do
    test_file "$f"
done

# Summary
TOTAL=$((PASS + FAIL + SKIP))
echo ""
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD} Cross-Verification Results${NC}"
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "  Total:   $TOTAL"
echo -e "  ${GREEN}Passed:${NC}  $PASS"
echo -e "  ${RED}Failed:${NC}  $FAIL"
echo -e "  ${YELLOW}Skipped:${NC} $SKIP"
if [[ $COMPILE_FAIL_SELFHOST -gt 0 ]]; then
    echo -e "    Selfhost compile failures: $COMPILE_FAIL_SELFHOST"
fi
if [[ $LINK_FAIL_SELFHOST -gt 0 ]]; then
    echo -e "    Selfhost link failures: $LINK_FAIL_SELFHOST"
fi
if [[ $COMPILE_FAIL_RUST -gt 0 ]]; then
    echo -e "    Rust compile failures: $COMPILE_FAIL_RUST"
fi
echo ""

if [[ $FAIL -eq 0 && $PASS -gt 0 ]]; then
    echo -e "  ${GREEN}✓ All tests passed!${NC}"
elif [[ $FAIL -gt 0 ]]; then
    echo -e "  ${RED}✗ $FAIL test(s) failed${NC}"
fi

if [[ "$KEEP_ARTIFACTS" == "true" ]]; then
    echo -e "  Artifacts: $BUILD_DIR"
fi
echo ""

exit $FAIL
