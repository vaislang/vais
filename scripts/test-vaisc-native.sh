#!/usr/bin/env bash
# Native driver smoke gate. This proves the Python-free host can compile through
# the checked-in self-host core.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
fail=0
tmp="$(mktemp -d)"
native="$tmp/vaisc"
want_version="$(sed -n 's/^#define VAIS_VERSION "\(.*\)"/\1/p' "$HERE/tools/vaisc_native.c" | head -1)"
if [ -z "$want_version" ]; then
    echo "error: cannot determine Vais version" >&2
    exit 1
fi

if "$HERE/scripts/build-vaisc-native.sh" "$native" >"$tmp/build-native.out" 2>"$tmp/build-native.err"; then
    echo "  PASS native vaisc builds"
else
    echo "  FAIL native vaisc build"
    cat "$tmp/build-native.err"
    exit 1
fi

if "$native" --version >"$tmp/version.out" 2>"$tmp/version.err" && grep -q "^vaisc $want_version$" "$tmp/version.out"; then
    echo "  PASS native vaisc --version"
else
    echo "  FAIL native vaisc --version"
    cat "$tmp/version.out" "$tmp/version.err"
    fail=1
fi

if "$native" doctor >"$tmp/doctor.out" 2>"$tmp/doctor.err" && grep -q 'self-host core: linked' "$tmp/doctor.out"; then
    echo "  PASS native vaisc doctor"
else
    echo "  FAIL native vaisc doctor"
    cat "$tmp/doctor.out" "$tmp/doctor.err"
    fail=1
fi

src="$tmp/native_smoke.vais"
cat > "$src" <<'SRC'
fn add(a: Int, b: Int) -> Int {
    return a + b
}

fn main() -> Int {
    let x = add(20, 22)
    return x
}
SRC

if "$native" emit-ir "$src" -o "$tmp/native.ll" >"$tmp/emit.out" 2>"$tmp/emit.err" &&
    grep -q '^define i64 @add' "$tmp/native.ll" &&
    grep -q '^define i64 @main()' "$tmp/native.ll"; then
    echo "  PASS native emit-ir emits helper and main"
else
    echo "  FAIL native emit-ir"
    cat "$tmp/emit.err"
    fail=1
fi

if "$native" build "$src" -o "$tmp/native_bin" --ir-out "$tmp/native_build.ll" >"$tmp/build.out" 2>"$tmp/build.err"; then
    "$tmp/native_bin"
    got=$?
    if [ "$got" = "42" ]; then
        echo "  PASS native build binary runs (=42)"
    else
        echo "  FAIL native build got=$got want=42"
        fail=1
    fi
else
    echo "  FAIL native build"
    cat "$tmp/build.err"
    fail=1
fi

"$native" run "$src" >"$tmp/run.out" 2>"$tmp/run.err"
got=$?
if [ "$got" = "42" ]; then
    echo "  PASS native run exits 42"
else
    echo "  FAIL native run got=$got want=42"
    cat "$tmp/run.err"
    fail=1
fi

if [ "$fail" -eq 0 ]; then
    echo "RESULT: native vaisc smoke OK"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
