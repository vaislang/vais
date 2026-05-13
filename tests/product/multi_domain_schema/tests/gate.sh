#!/usr/bin/env bash
# Multi-domain shared schema product gate (Master Plan Step 14).
#
# Exit codes:
#   0 — gate pass
#   1 — product contract propagation failed
#   2 — fixture/toolchain drift

set -uo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
FIX="$(cd "$DIR/.." && pwd)"
COMPILER_ROOT="$(cd "$FIX/../../.." && pwd)"
WORKSPACE_ROOT="$(cd "$COMPILER_ROOT/.." && pwd)"

VAISC="${VAISC:-${COMPILER_ROOT}/target/release/vaisc}"
TSC="${TSC:-${WORKSPACE_ROOT}/lang/packages/vais-web/node_modules/.pnpm/node_modules/.bin/tsc}"
SCHEMA_SOURCE="${SCHEMA_SOURCE:-${FIX}/schema/user.vais}"
WEB_DB_SRC="${WORKSPACE_ROOT}/lang/packages/vais-web/packages/db/src"

ASSERTION_TOTAL=9
GATE_EXIT=0

record_fail() {
  local code="$1"
  if [[ "$GATE_EXIT" == "0" ]]; then
    GATE_EXIT="$code"
  fi
}

if [[ ! -x "$VAISC" ]]; then
  echo "FIXTURE_DRIFT: vaisc not found at $VAISC" >&2
  echo "  Build with: cd compiler && cargo build --release --bin vaisc" >&2
  exit 2
fi

if [[ ! -x "$TSC" ]]; then
  echo "FIXTURE_DRIFT: tsc not found at $TSC" >&2
  echo "  Install via: cd lang/packages/vais-web && pnpm install" >&2
  exit 2
fi

if [[ ! -f "$SCHEMA_SOURCE" ]]; then
  echo "FIXTURE_DRIFT: shared schema missing at $SCHEMA_SOURCE" >&2
  exit 2
fi

if [[ ! -f "$WEB_DB_SRC/schema.ts" || ! -f "$WEB_DB_SRC/types.ts" ]]; then
  echo "FIXTURE_DRIFT: @vaisx/db schema source missing under $WEB_DB_SRC" >&2
  exit 2
fi

WORK="$(mktemp -d)"
SCHEMA="$WORK/schema/user.vais"
GEN="$WORK/gen/user.d.ts"

cleanup() {
  rm -rf "$WORK"
  exit "$GATE_EXIT"
}
trap cleanup EXIT INT TERM

mkdir -p "$WORK/schema" "$WORK/gen" "$WORK/consumers" "$WORK/webdb/src" "$WORK/tmp"
cp "$SCHEMA_SOURCE" "$SCHEMA"
cp "$FIX/consumers/vaisdb_product.vais" "$WORK/consumers/vaisdb_product.vais"
cp "$FIX/consumers/vais_server_product.vais" "$WORK/consumers/vais_server_product.vais"
cp "$FIX/consumers/vais_web_product.ts" "$WORK/consumers/vais_web_product.ts"
cp "$WEB_DB_SRC/schema.ts" "$WORK/webdb/src/schema.ts"
cp "$WEB_DB_SRC/types.ts" "$WORK/webdb/src/types.ts"

render_vais_consumer() {
  local template="$1"
  local output="$2"
  while IFS= read -r line; do
    if [[ "$line" == "# __SHARED_USER_SCHEMA__" ]]; then
      cat "$SCHEMA"
    else
      printf '%s\n' "$line"
    fi
  done < "$template" > "$output"
}

std_path() {
  local tmp_std_root="/tmp/vais-lib"
  local std_link="$tmp_std_root/std"
  mkdir -p "$tmp_std_root"
  if [[ -L "$std_link" ]]; then
    local current
    current="$(readlink "$std_link")"
    if [[ "$current" != "$COMPILER_ROOT/std" ]]; then
      rm -f "$std_link"
    fi
  elif [[ -e "$std_link" ]]; then
    rm -rf "$std_link"
  fi
  if [[ ! -e "$std_link" ]]; then
    ln -s "$COMPILER_ROOT/std" "$std_link"
  fi
  printf '%s\n' "$std_link"
}

check_vais() {
  local source="$1"
  local dep_paths="$2"
  VAIS_STD_PATH="$(std_path)" VAIS_DEP_PATHS="$dep_paths" \
    "$VAISC" check "$source" >/dev/null 2>&1
}

check_vaisdb_product() {
  local rendered="$WORK/consumers/vaisdb_product.rendered.vais"
  render_vais_consumer "$WORK/consumers/vaisdb_product.vais" "$rendered"
  local deps="${WORKSPACE_ROOT}/lang/packages/vaisdb/src:$(std_path)"
  check_vais "$rendered" "$deps"
}

check_server_product() {
  local rendered="$WORK/consumers/vais_server_product.rendered.vais"
  render_vais_consumer "$WORK/consumers/vais_server_product.vais" "$rendered"
  local deps="${WORKSPACE_ROOT}/lang/packages/vais-server:${WORKSPACE_ROOT}/lang/packages/vaisdb/src:$(std_path)"
  check_vais "$rendered" "$deps"
}

check_web_product() {
  "$TSC" --noEmit --strict --target ES2020 --module NodeNext \
    --moduleResolution NodeNext \
    "$GEN" \
    "$WORK/consumers/vais_web_product.ts" \
    "$WORK/webdb/src/schema.ts" \
    "$WORK/webdb/src/types.ts" \
    >/dev/null 2>&1
}

sed_inplace() {
  local pattern="$1"
  local file="$2"
  sed -i.bak "$pattern" "$file"
  rm -f "${file}.bak"
}

assert_diff() {
  local label="$1"
  if diff -q "$SCHEMA" "$SCHEMA_SOURCE" >/dev/null 2>&1; then
    echo "FIXTURE_DRIFT: $label produced no schema diff" >&2
    record_fail 2
    return 1
  fi
  return 0
}

echo ">>> Multi-domain product baseline"

if ! "$VAISC" emit-ts "$SCHEMA" --output "$GEN" >/dev/null 2>&1; then
  echo "FIXTURE_DRIFT: emit-ts failed on shared schema" >&2
  record_fail 2
else
  echo "  [1] emit-ts shared schema              OK"
fi

if [[ "$GATE_EXIT" == "0" ]] && ! check_vaisdb_product; then
  echo "FIXTURE_DRIFT: VaisDB product consumer failed to type-check on shared schema" >&2
  record_fail 2
else
  [[ "$GATE_EXIT" == "0" ]] && echo "  [2] VaisDB catalog product type-check   OK"
fi

if [[ "$GATE_EXIT" == "0" ]] && ! check_server_product; then
  echo "FIXTURE_DRIFT: vais-server product consumer failed to type-check on shared schema" >&2
  record_fail 2
else
  [[ "$GATE_EXIT" == "0" ]] && echo "  [3] vais-server context type-check      OK"
fi

if [[ "$GATE_EXIT" == "0" ]] && ! check_web_product; then
  echo "FIXTURE_DRIFT: vais-web product consumer failed to type-check" >&2
  record_fail 2
else
  [[ "$GATE_EXIT" == "0" ]] && echo "  [4] vais-web DB schema type-check       OK"
fi

echo ">>> Multi-domain product propagation"

if [[ "$GATE_EXIT" == "0" ]]; then
  sed_inplace 's/email: str/mail: str/' "$SCHEMA"
  assert_diff "email -> mail mutation"
fi
[[ "$GATE_EXIT" == "0" ]] && echo "  [5] schema rename produced real diff    OK"

if [[ "$GATE_EXIT" == "0" ]] && ! "$VAISC" emit-ts "$SCHEMA" --output "$GEN" >/dev/null 2>&1; then
  echo "FIXTURE_DRIFT: emit-ts failed on renamed schema" >&2
  record_fail 2
else
  [[ "$GATE_EXIT" == "0" ]] && echo "  [6] emit-ts renamed schema              OK"
fi

if [[ "$GATE_EXIT" == "0" ]] && check_vaisdb_product; then
  echo "GATE FAIL: VaisDB product consumer still type-checks after email rename" >&2
  record_fail 1
else
  [[ "$GATE_EXIT" == "0" ]] && echo "  [7] VaisDB product now fails            OK"
fi

if [[ "$GATE_EXIT" == "0" ]] && check_server_product; then
  echo "GATE FAIL: vais-server product consumer still type-checks after email rename" >&2
  record_fail 1
else
  [[ "$GATE_EXIT" == "0" ]] && echo "  [8] vais-server product now fails       OK"
fi

if [[ "$GATE_EXIT" == "0" ]] && check_web_product; then
  echo "GATE FAIL: vais-web product consumer still type-checks after email rename" >&2
  record_fail 1
else
  [[ "$GATE_EXIT" == "0" ]] && echo "  [9] vais-web product now fails          OK"
fi

if [[ "$GATE_EXIT" == "0" ]]; then
  echo ""
  echo "GATE ASSERTIONS: ${ASSERTION_TOTAL}/${ASSERTION_TOTAL}"
  echo "GATE PASS: multi-domain shared schema product contract held."
else
  echo ""
  echo "GATE ASSERTIONS: 0/${ASSERTION_TOTAL}"
fi
