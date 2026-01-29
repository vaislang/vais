#!/usr/bin/env bash
# Vais Self-Hosting Bootstrap Verification Script
# Verifies the complete bootstrapping cycle (Stage 1 → Stage 2 → Stage 3)
#
# Stage 0: Rust compiler builds vaisc (cargo build)
# Stage 1: vaisc compiles selfhost/main_entry.vais → vaisc-stage1
# Stage 2: vaisc-stage1 compiles main_entry.vais → vaisc-stage2
# Stage 3: vaisc-stage2 compiles main_entry.vais → vaisc-stage3
# Verification: stage2 IR == stage3 IR (fixed-point reached)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SELFHOST_DIR="$PROJECT_ROOT/selfhost"
BUILD_DIR="${BUILD_DIR:-/tmp/vais-bootstrap}"
VAISC="$PROJECT_ROOT/target/release/vaisc"
RUNTIME_C="$SELFHOST_DIR/runtime.c"
SOURCE="$SELFHOST_DIR/main_entry.vais"
CC="${CC:-clang}"
CFLAGS="${CFLAGS:--O0}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info()  { echo -e "${CYAN}[INFO]${NC} $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_fail()  { echo -e "${RED}[FAIL]${NC} $*"; }

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Options:
  --stage NUM     Run up to stage NUM (1, 2, or 3; default: 3)
  --keep          Keep build artifacts after completion
  --verbose       Show detailed output
  --no-build      Skip cargo build (use existing vaisc binary)
  --help          Show this help message

Environment:
  CC              C compiler (default: clang)
  CFLAGS          C compiler flags (default: -O0)
  BUILD_DIR       Build directory (default: /tmp/vais-bootstrap)
EOF
    exit 0
}

# Parse arguments
MAX_STAGE=3
KEEP_ARTIFACTS=false
VERBOSE=false
SKIP_BUILD=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --stage)  MAX_STAGE="$2"; shift 2 ;;
        --keep)   KEEP_ARTIFACTS=true; shift ;;
        --verbose) VERBOSE=true; shift ;;
        --no-build) SKIP_BUILD=true; shift ;;
        --help)   usage ;;
        *)        echo "Unknown option: $1"; usage ;;
    esac
done

cleanup() {
    if [[ "$KEEP_ARTIFACTS" == "false" ]]; then
        rm -rf "$BUILD_DIR"
    fi
}
trap cleanup EXIT

# Setup build directory
mkdir -p "$BUILD_DIR"

log_info "=== Vais Bootstrap Verification ==="
log_info "Project root: $PROJECT_ROOT"
log_info "Build dir: $BUILD_DIR"
log_info "Max stage: $MAX_STAGE"
echo ""

# --- Stage 0: Build Rust compiler ---
if [[ "$SKIP_BUILD" == "false" ]]; then
    log_info "Stage 0: Building Rust compiler (cargo build --release)..."
    cd "$PROJECT_ROOT"
    cargo build --release --package vaisc 2>&1 | if [[ "$VERBOSE" == "true" ]]; then cat; else tail -1; fi
    log_ok "Stage 0 complete: $VAISC"
else
    if [[ ! -f "$VAISC" ]]; then
        log_fail "vaisc binary not found at $VAISC"
        log_info "Run without --no-build or build manually: cargo build --release --package vaisc"
        exit 1
    fi
    log_info "Stage 0: Skipped (using existing binary)"
fi
echo ""

# --- Compile runtime.c ---
log_info "Compiling runtime.c..."
RUNTIME_O="$BUILD_DIR/runtime.o"
$CC $CFLAGS -c "$RUNTIME_C" -o "$RUNTIME_O"
log_ok "Runtime compiled: $RUNTIME_O"
echo ""

# --- Stage 1: Rust vaisc compiles selfhost ---
log_info "Stage 1: Rust vaisc → selfhost/main_entry.vais → vaisc-stage1"
STAGE1_LL="$BUILD_DIR/stage1.ll"
STAGE1_BIN="$BUILD_DIR/vaisc-stage1"

"$VAISC" "$SOURCE" --emit-ir -o "$STAGE1_LL" 2>&1 | if [[ "$VERBOSE" == "true" ]]; then cat; else tail -3; fi
$CC $CFLAGS "$STAGE1_LL" "$RUNTIME_O" -o "$STAGE1_BIN" -lm
log_ok "Stage 1 complete: $STAGE1_BIN ($(wc -l < "$STAGE1_LL") lines IR, $(du -h "$STAGE1_BIN" | cut -f1) binary)"
echo ""

if [[ "$MAX_STAGE" -lt 2 ]]; then
    log_ok "Bootstrap verification complete (Stage 1 only)"
    exit 0
fi

# --- Stage 2: vaisc-stage1 compiles selfhost ---
log_info "Stage 2: vaisc-stage1 → selfhost/main_entry.vais → vaisc-stage2"
STAGE2_LL="$BUILD_DIR/stage2.ll"
STAGE2_BIN="$BUILD_DIR/vaisc-stage2"

# Stage 1 compiler may output to a hardcoded path, so we copy source to expected location
cd "$SELFHOST_DIR"
"$STAGE1_BIN" 2>&1 | if [[ "$VERBOSE" == "true" ]]; then cat; else tail -3; fi

# The stage1 compiler writes to selfhost/main_output.ll
if [[ -f "$SELFHOST_DIR/main_output.ll" ]]; then
    cp "$SELFHOST_DIR/main_output.ll" "$STAGE2_LL"
else
    log_fail "Stage 1 compiler did not produce main_output.ll"
    exit 1
fi

$CC $CFLAGS "$STAGE2_LL" "$RUNTIME_O" -o "$STAGE2_BIN" -lm
log_ok "Stage 2 complete: $STAGE2_BIN ($(wc -l < "$STAGE2_LL") lines IR, $(du -h "$STAGE2_BIN" | cut -f1) binary)"
echo ""

if [[ "$MAX_STAGE" -lt 3 ]]; then
    log_ok "Bootstrap verification complete (Stage 1-2)"
    exit 0
fi

# --- Stage 3: vaisc-stage2 compiles selfhost ---
log_info "Stage 3: vaisc-stage2 → selfhost/main_entry.vais → vaisc-stage3"
STAGE3_LL="$BUILD_DIR/stage3.ll"
STAGE3_BIN="$BUILD_DIR/vaisc-stage3"

cd "$SELFHOST_DIR"
"$STAGE2_BIN" 2>&1 | if [[ "$VERBOSE" == "true" ]]; then cat; else tail -3; fi

# Stage 2 compiler also writes to selfhost/main_output.ll
if [[ -f "$SELFHOST_DIR/main_output.ll" ]]; then
    cp "$SELFHOST_DIR/main_output.ll" "$STAGE3_LL"
else
    log_fail "Stage 2 compiler did not produce main_output.ll"
    exit 1
fi

$CC $CFLAGS "$STAGE3_LL" "$RUNTIME_O" -o "$STAGE3_BIN" -lm
log_ok "Stage 3 complete: $STAGE3_BIN ($(wc -l < "$STAGE3_LL") lines IR, $(du -h "$STAGE3_BIN" | cut -f1) binary)"
echo ""

# --- Verification: Compare Stage 2 and Stage 3 IR ---
log_info "=== Fixed-Point Verification ==="
log_info "Comparing Stage 2 IR vs Stage 3 IR..."

STAGE2_LINES=$(wc -l < "$STAGE2_LL")
STAGE3_LINES=$(wc -l < "$STAGE3_LL")
log_info "Stage 2 IR: $STAGE2_LINES lines"
log_info "Stage 3 IR: $STAGE3_LINES lines"

if diff -q "$STAGE2_LL" "$STAGE3_LL" > /dev/null 2>&1; then
    echo ""
    log_ok "============================================"
    log_ok "  FIXED POINT REACHED!"
    log_ok "  Stage 2 IR == Stage 3 IR (byte-identical)"
    log_ok "  Bootstrap verification: PASSED"
    log_ok "============================================"
    echo ""

    # Summary
    log_info "Summary:"
    log_info "  Stage 1 IR: $(wc -l < "$STAGE1_LL") lines"
    log_info "  Stage 2 IR: $STAGE2_LINES lines"
    log_info "  Stage 3 IR: $STAGE3_LINES lines"
    log_info "  Stage 1 bin: $(du -h "$STAGE1_BIN" | cut -f1)"
    log_info "  Stage 2 bin: $(du -h "$STAGE2_BIN" | cut -f1)"
    log_info "  Stage 3 bin: $(du -h "$STAGE3_BIN" | cut -f1)"

    if [[ "$KEEP_ARTIFACTS" == "true" ]]; then
        log_info "Artifacts kept at: $BUILD_DIR"
    fi
    exit 0
else
    echo ""
    log_warn "Stage 2 IR and Stage 3 IR differ."
    log_info "Generating diff report..."

    DIFF_FILE="$BUILD_DIR/stage2-vs-stage3.diff"
    diff -u "$STAGE2_LL" "$STAGE3_LL" > "$DIFF_FILE" || true

    DIFF_LINES=$(wc -l < "$DIFF_FILE")
    log_warn "Diff: $DIFF_LINES lines of differences"
    log_warn "Full diff saved to: $DIFF_FILE"

    if [[ "$VERBOSE" == "true" ]]; then
        echo ""
        log_info "First 50 lines of diff:"
        head -50 "$DIFF_FILE"
    fi

    # Check if differences are only in non-semantic areas (comments, metadata)
    SEMANTIC_DIFFS=$(grep -c '^[+-]' "$DIFF_FILE" | grep -v '^[+-][+-][+-]' || echo "0")
    if [[ "$SEMANTIC_DIFFS" -lt 10 ]]; then
        log_warn "Differences appear minor (< 10 changed lines)"
        log_warn "This may be due to non-deterministic register/label numbering"
        exit 1
    else
        log_fail "Significant differences detected ($SEMANTIC_DIFFS changed lines)"
        log_fail "The compiler has not reached a fixed point"
        exit 2
    fi
fi
