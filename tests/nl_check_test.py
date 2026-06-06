#!/usr/bin/env python3
"""Unit tests for nl-check (P4 error infra): flags non-nl spellings with help:,
and must NOT flag valid nl. Run: python3 tests/nl_check_test.py"""
import importlib.util
import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location(
    "nlcheck", os.path.join(HERE, "..", "tools", "nl-check.py"))
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)


def run(src: str) -> int:
    with tempfile.NamedTemporaryFile("w", suffix=".nl", delete=False) as f:
        f.write(src)
        path = f.name
    # capture stdout
    import io, contextlib
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        rc = mod.check(path)
    os.unlink(path)
    return rc, buf.getvalue()


# (name, src, should_flag)
CASES = [
    ("&& flagged", "fn main() -> Int {\n  if a && b { return 1 }\n  return 0\n}", True),
    ("|| flagged", "fn main() -> Int {\n  if a || b { return 1 }\n  return 0\n}", True),
    ("as cast flagged", "fn main() -> Int {\n  let x = 5 as Int\n  return x\n}", True),
    (":: flagged", "fn main() -> Int {\n  match c { Color::Red => 1 }\n}", True),
    ("turbofish flagged", "fn main() -> Int {\n  let v = List<Int>::new()\n  return 0\n}", True),
    # valid nl must NOT be flagged:
    ("and clean", "fn main() -> Int {\n  if a and b { return 1 }\n  return 0\n}", False),
    ("dot enum clean", "fn main() -> Int {\n  match c { Color.Red => 1 }\n}", False),
    ("Int() cast clean", "fn main() -> Int {\n  let x = Int(y)\n  return x\n}", False),
    ("=> return clean (P6 allowed)", "fn f() -> Int {\n  match c { Color.Red => return 1 }\n}", False),
    ("string with && not flagged", 'fn main() -> Int {\n  let s = "a && b"\n  return 0\n}', False),
    ("comment with :: not flagged", "fn main() -> Int {\n  # use Foo::bar style elsewhere\n  return 0\n}", False),
]


def main():
    failed = 0
    for name, src, should in CASES:
        rc, out = run(src)
        flagged = (rc == 1)
        if flagged == should:
            print(f"  PASS {name}")
        else:
            failed += 1
            print(f"  FAIL {name}: flagged={flagged} expected={should}\n{out}")
    print(f"\nRESULT: {len(CASES)-failed}/{len(CASES)} pass")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
