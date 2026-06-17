#!/usr/bin/env python3
"""
vais-check — lint Vais source for non-Vais (usually Rust/other-language intuition)
constructs and report them with `help:` + the exact fix (design principle P4:
errors give the cure, not just the symptom — the key lever for AI self-correction).

WHY this exists: Rust-intuition spellings such as `&&` or `x as Int` silently
open two ways to write one thing. vais-check enforces the single Vais spelling
and tells the user the concrete correction.

Usage:  python3 vais-check.py file.vais
Exit 0 = clean; exit 1 = issues found (printed with help:).

Format per issue:
    error: <what>
      --> file:line:col
      <source line>
      help: <fix> -> `<corrected code>`

Scope: line-based lint (prototype). Skips string literals and comments.

Catalog (Rust/other-language intuition -> the single Vais form):
  &&/||/!         -> and / or / not
  x as Type       -> Type(x)
  Path::x         -> Path.x       (dot, not ::)
  Type<..>::new() -> a literal ([] / [1,2,3] / {})
  vec![..]        -> [..]
  Vec<T>          -> List<T>
  HashMap<K,V>    -> Map<K,V>
  module/package -> future Phase 2 surface, not implemented yet
  i8..i128/u8..u128/f32/f64/usize/isize -> Int / Int8..Int128 / UInt8.. / F32 / F64
  x.to_string()   -> Str(x)
  .unwrap()/.expect() -> match the Option/Result, or `?`
  if let          -> match
  elsif/elif      -> else if
  x += 1 (+=/-=/*=//=)  -> x = x + 1  (no compound assignment in Vais)
"""
import re
import sys

# Each rule: (regex on the non-string part, message, help text builder).
# help builder takes the regex match and returns the suggested fix string.

def _strip_strings_comments(line: str):
    """Return the line with string literals blanked and comment removed, plus a
    map so column reporting stays roughly right. Prototype: just blank them."""
    # remove trailing comment
    # (naive: first '#' not in a string — we blank strings first)
    parts = re.split(r'("(?:[^"\\]|\\.)*")', line)
    rebuilt = []
    for i, p in enumerate(parts):
        rebuilt.append(" " * len(p) if i % 2 == 1 else p)
    s = "".join(rebuilt)
    h = s.find("#")
    if h != -1:
        s = s[:h] + " " * (len(s) - h)
    return s


RULES = [
    (re.compile(r"&&"),
     "logical AND uses the word `and`, not `&&`",
     lambda m, ln: ln.replace("&&", "and")),
    (re.compile(r"\|\|"),
     "logical OR uses the word `or`, not `||`",
     lambda m, ln: ln.replace("||", "or")),
    (re.compile(r"(?<![<>=!])!(?!=)(?=\s*\w|\s*\()"),
     "logical NOT uses the word `not`, not `!`",
     lambda m, ln: re.sub(r"!(\s*)", r"not \1", ln, count=1)),
    (re.compile(r"\bas\s+([A-Za-z_]\w*)"),
     "type conversion is explicit `Type(x)`, not `x as Type`",
     lambda m, ln: f"... use {m.group(1)}(expr) instead of `as {m.group(1)}`"),
    (re.compile(r"\b[A-Z]\w*::"),
     "enum/path uses a dot `.`, not `::`",
     lambda m, ln: ln.replace("::", ".")),
    (re.compile(r"\b\w+<[^>]*>::new\b"),
     "no turbofish constructor; use a literal `[]` / `[1, 2, 3]` or `{}`",
     lambda m, ln: "... use a literal instead of `Type<..>::new()`"),
    # --- list/macro spellings ---
    (re.compile(r"\bvec!\s*\["),
     "list literals are just `[1, 2, 3]`, not `vec![...]`",
     lambda m, ln: re.sub(r"\bvec!\s*\[", "[", ln, count=1)),
    # --- collection TYPE name (Vais uses List<T>, not Vec<T>) ---
    (re.compile(r"\bVec\s*<"),
     "the list type is `List<T>`, not `Vec<T>`",
     lambda m, ln: re.sub(r"\bVec(\s*<)", r"List\1", ln, count=1)),
    (re.compile(r"\bHashMap\b"),
     "the map type spelling is `Map<K,V>`, not `HashMap<K,V>`; only local `Map<Int,Int>` is verified for now",
     lambda m, ln: re.sub(r"\bHashMap\b", "Map", ln, count=1)),
    (re.compile(r"^\s*(module|package)\b"),
     "module and package declarations are not implemented yet",
     lambda m, ln: "... omit the declaration until module/package declarations are implemented"),
    # --- Rust scalar type names (Vais uses Int/UInt8.. / F32 / F64, capitalized) ---
    (re.compile(r"(?<![A-Za-z0-9_])(i8|i16|i32|i64|i128|u8|u16|u32|u64|u128|f32|f64|usize|isize)(?![A-Za-z0-9_])"),
     "Vais scalar types are capitalized: `Int` / `Int8..Int128` / `UInt8..UInt128` / `F32` / `F64`",
     lambda m, ln: f"... use the Vais type name (e.g. Int) instead of `{m.group(1)}`"),
    # --- .to_string() is a Rust-ism; Vais uses explicit conversion calls. ---
    (re.compile(r"\.to_string\s*\("),
     "no `.to_string()` method in Vais (Rust-ism); use explicit `Str(x)` conversion",
     lambda m, ln: "... use `Str(expr)` instead of `.to_string()`"),
    # --- Option/Result unwrap (Vais uses match or `?`) ---
    (re.compile(r"\.unwrap\s*\("),
     "no `.unwrap()`; use a `match` arm (Some(v)/None) or `?` to propagate",
     lambda m, ln: "... match the Option/Result or use `?` instead of `.unwrap()`"),
    (re.compile(r"\.expect\s*\("),
     "no `.expect()`; use a `match` arm (Some(v)/None) or `?` to propagate",
     lambda m, ln: "... match the Option/Result or use `?` instead of `.expect()`"),
    # --- `if let` (Vais uses match) ---
    (re.compile(r"\bif\s+let\b"),
     "no `if let`; use a `match` (e.g. `match opt { Some(v) => ..., None => ... }`)",
     lambda m, ln: "... rewrite as a match on the Option/Result"),
    # --- `elsif`/`elif` typo (Vais uses `else if`) ---
    (re.compile(r"\b(elsif|elif)\b"),
     "the keyword is `else if` (two words), not `elsif`/`elif`",
     lambda m, ln: re.sub(r"\b(elsif|elif)\b", "else if", ln, count=1)),
    # --- `String` type (Rust-ism) -> Vais's type name is `Str` ---
    (re.compile(r"\bString\b"),
     "the string type is `Str`, not `String`",
     lambda m, ln: re.sub(r"\bString\b", "Str", ln, count=1)),
    # --- compound assignment (Vais has none; write it out: `x = x + e`). Match
    # `+= -= *= /= %=` but NOT comparison/equality (== <= >= != ) — the operator
    # char is preceded by + - * / % and followed by `=` not part of `==`. ---
    (re.compile(r"[+\-*/%]=(?!=)"),
     "no compound assignment in Vais; write it out, e.g. `x = x + 1` (not `x += 1`)",
     lambda m, ln: "... expand to `x = x <op> e` (Vais has no `+=`/`-=`/`*=`/`/=`)"),
    (re.compile(r"=>\s*return\b"),
     None,  # allowed in Vais (P6) — informational only, not an error
     None),
    (re.compile(r"\bfn\s+[A-Z]\b"),
     None, None),  # single-uppercase fn name is fine in Vais (keyword space split)
]


def check(path: str) -> int:
    issues = 0
    with open(path) as f:
        lines = f.readlines()
    for n, raw in enumerate(lines, 1):
        code = _strip_strings_comments(raw)
        for pat, msg, fix in RULES:
            if msg is None:
                continue
            m = pat.search(code)
            if m:
                col = m.start() + 1
                issues += 1
                print(f"error: {msg}")
                print(f"  --> {path}:{n}:{col}")
                print(f"  {raw.rstrip()}")
                if fix:
                    print(f"  help: {fix(m, raw.strip())}")
                print()
    if issues == 0:
        print(f"vais-check: {path} clean")
    return 1 if issues else 0


def main():
    if len(sys.argv) != 2:
        print("usage: vais-check.py file.vais", file=sys.stderr)
        sys.exit(2)
    sys.exit(check(sys.argv[1]))


if __name__ == "__main__":
    main()
