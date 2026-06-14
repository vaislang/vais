#!/usr/bin/env bash
# Standalone install/package gate for the native Vais compiler.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
fail=0
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

prefix="$tmp/prefix"
out_dir="$tmp/dist"
src="$tmp/install_smoke.vais"
want_version="$(sed -n 's/^#define VAIS_VERSION "\(.*\)"/\1/p' "$HERE/tools/vaisc_native.c" | head -1)"
if [ -z "$want_version" ]; then
    echo "error: cannot determine Vais version" >&2
    exit 1
fi

cat > "$src" <<'SRC'
fn main() -> Int {
    return 42
}
SRC

if "$HERE/scripts/install-vaisc.sh" --prefix "$prefix" >"$tmp/install.out" 2>"$tmp/install.err" &&
    [ -x "$prefix/bin/vaisc" ]; then
    echo "  PASS standalone install creates bin/vaisc"
else
    echo "  FAIL standalone install"
    cat "$tmp/install.out" "$tmp/install.err"
    exit 1
fi

if "$prefix/bin/vaisc" --version >"$tmp/version.out" 2>"$tmp/version.err" &&
    grep -q "^vaisc $want_version$" "$tmp/version.out"; then
    echo "  PASS installed vaisc --version"
else
    echo "  FAIL installed vaisc --version"
    cat "$tmp/version.out" "$tmp/version.err"
    fail=1
fi

if "$prefix/bin/vaisc" doctor >"$tmp/doctor.out" 2>"$tmp/doctor.err" &&
    grep -q 'self-host core: linked' "$tmp/doctor.out"; then
    echo "  PASS installed vaisc doctor"
else
    echo "  FAIL installed vaisc doctor"
    cat "$tmp/doctor.out" "$tmp/doctor.err"
    fail=1
fi

"$prefix/bin/vaisc" run "$src" >"$tmp/run.out" 2>"$tmp/run.err"
run_rc=$?
if [ "$run_rc" = "42" ]; then
    echo "  PASS installed vaisc run exits 42"
else
    echo "  FAIL installed vaisc run got=$run_rc want=42"
    cat "$tmp/run.err"
    fail=1
fi

if "$HERE/scripts/package-vaisc-release.sh" --out-dir "$out_dir" >"$tmp/package.out" 2>"$tmp/package.err"; then
    archive="$(sed -n 's/^archive: //p' "$tmp/package.out" | tail -1)"
    if [ -f "$archive" ]; then
        echo "  PASS release archive created"
    else
        echo "  FAIL release archive missing"
        cat "$tmp/package.out"
        fail=1
    fi
else
    echo "  FAIL release archive creation"
    cat "$tmp/package.out" "$tmp/package.err"
    fail=1
fi

extract="$tmp/extract"
mkdir -p "$extract"
if [ "$fail" -eq 0 ] && tar -C "$extract" -xzf "$archive"; then
    packaged_vaisc="$(find "$extract" -path '*/bin/vaisc' -type f | head -1)"
    if [ -n "$packaged_vaisc" ] && [ -x "$packaged_vaisc" ]; then
        "$packaged_vaisc" run "$src" >"$tmp/package-run.out" 2>"$tmp/package-run.err"
        package_run_rc=$?
        if [ "$package_run_rc" = "42" ]; then
            echo "  PASS packaged vaisc run exits 42"
        else
            echo "  FAIL packaged vaisc run got=$package_run_rc want=42"
            cat "$tmp/package-run.err"
            fail=1
        fi
    else
        echo "  FAIL packaged bin/vaisc missing"
        fail=1
    fi
else
    echo "  FAIL release archive extraction"
    fail=1
fi

if "$HERE/scripts/uninstall-vaisc.sh" --prefix "$prefix" >"$tmp/uninstall.out" 2>"$tmp/uninstall.err" &&
    [ ! -e "$prefix/bin/vaisc" ]; then
    echo "  PASS standalone uninstall removes bin/vaisc"
else
    echo "  FAIL standalone uninstall"
    cat "$tmp/uninstall.out" "$tmp/uninstall.err"
    fail=1
fi

if [ "$fail" -eq 0 ]; then
    echo "RESULT: Vais standalone install/package gate OK"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
