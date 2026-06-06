#!/usr/bin/env python3
"""Unit tests for the nl->Vais transpiler (regression protection for the tool
itself, separate from the value-correctness runner). Checks that specific nl
snippets transpile to the expected Vais — so a transpiler change that breaks a
mapping is caught even if no example exercises it.

Run:  python3 tests/transpiler_test.py
Exit 0 iff all pass.
"""
import importlib.util
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TRANSPILER = os.path.join(HERE, "..", "compiler", "transpiler", "nl2vais.py")

spec = importlib.util.spec_from_file_location("nl2vais", TRANSPILER)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)


def transpile(src: str) -> str:
    # Exercise the same pipeline main() uses (for-expansion + per-line).
    src = mod.expand_for_loops(src)
    return "\n".join(mod.transpile_line(l + "\n") for l in src.splitlines())


# (name, nl_input, must_contain[], must_not_contain[])
CASES = [
    # `let x: Int = 5` -> `x := 5`: the annotation is intentionally dropped
    # (Vais `:=` infers); so neither Int nor i64 appears. Type mapping is
    # exercised via param/return positions instead (see "List type -> Vec").
    ("let typed drops annot", "let x: Int = 5", ["x := 5"], ["let ", "Int", "i64"]),
    ("type map in param", "fn f(a: Int) -> Int {", ["i64"], ["Int"]),
    ("let immut", "let x = 5", ["x := 5"], ["let "]),
    ("let mut", "let mut x = 5", ["x := mut 5"], []),
    ("if->I", "if x < 3 {", ["I x < 3 {"], []),
    ("else if", "} else if x == 0 {", ["else I x == 0"], ["else if"]),
    ("and->&&", "if a and b {", ["&&"], [" and "]),
    ("or->||", "if a or b {", ["||"], [" or "]),
    ("not->!", "let y = not x", ["!"], [" not "]),
    ("bitnot", "let y = bitnot(0)", ["(~0)"], ["bitnot"]),
    ("enum dot strip", "Color.Red => 1,", ["Red => 1"], ["Color.Red"]),
    ("arm return wrap", "Some(v) => return v,", ["=> { return v }"], []),
    ("list literal type", "let v = [1, 2, 3]", ["Vec<i64> = [1, 2, 3]"], []),
    # nested list literals infer the element type recursively so nesting is kept
    # (Vais can't yet codegen nested Vec -- TRACKED -- but the typing must be
    #  correct: Vec<Vec<i64>>, not the wrong flat Vec<i64>).
    ("nested list infer", "let rows = [[1, 2], [3, 4]]", ["Vec<Vec<i64>>"], []),
    ("nested list annotated", "let rows: List<List<Int>> = [[1, 2]]",
     ["Vec<Vec<i64>>"], ["List<"]),
    (".sum -> fold", "let s = v.sum()", [".fold(0,"], [".sum()"]),
    ("List type -> Vec", "fn f(x: List<Int>) -> Int {", ["Vec<i64>"], ["List<"]),
    ("field pub strip", "    pub name: Str,", ["name: str"], ["pub name"]),
    ("string interp kept", 'let s = "hi {name}"', ['"hi {name}"'], []),
    ("? kept", "let r = f()?", ["r := f()?"], ["let "]),
    # String literals are code-as-data: keyword/word rewrites must NOT touch them.
    # (Regression: `if`/`and` inside an embedded program string got corrupted to
    #  `I`/`&&`, breaking the self-host compiler's test inputs.)
    ("if in string kept", 'let p = run("return if a > b then a else b")',
     ['"return if a > b then a else b"'], ["return I "]),
    ("and in string kept", 'let p = run("x and y")', ['"x and y"'], ['"x && y"']),
    ("if in code still mapped", "    if a > b {", ["I a > b {"], []),
    # &List<T> -> &Vec<T> (borrow preserved); enables recursion over a list
    # (Vais Vec is move-by-value, so collections recurse by reference).
    ("ref List param", "fn f(v: &List<Int>) -> Int {", ["&Vec<i64>"], ["List<"]),
    ("ref arg preserved", "    return f(&v)", ["f(&v)"], []),
]


def check_for():
    # for-range inclusive
    out = transpile("for i in 0..=3 {\n  s = s + i\n}")
    assert "i := mut 0" in out and "I i > 3 { B }" in out and "i = i + 1" in out, out
    # for-range exclusive
    out = transpile("for i in 0..3 {\n  s = s + i\n}")
    assert "I i >= 3 { B }" in out, out
    # while
    out = transpile("while i < 5 {\n  i = i + 1\n}")
    assert "L {" in out and "I !(i < 5) { B }" in out, out
    return True


def main():
    failed = 0
    for name, src, must, mustnot in CASES:
        out = transpile(src)
        ok = all(m in out for m in must) and all(n not in out for n in mustnot)
        if ok:
            print(f"  PASS {name}")
        else:
            failed += 1
            print(f"  FAIL {name}: got {out!r}")
    try:
        check_for()
        print("  PASS for/while structural")
    except AssertionError as e:
        failed += 1
        print(f"  FAIL for/while structural: {e}")
    print(f"\nRESULT: {len(CASES)+1-failed}/{len(CASES)+1} pass")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
