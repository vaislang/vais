#!/usr/bin/env python3
"""
nl-check — lint nl source for non-nl (usually Rust/other-language intuition)
constructs and report them with `help:` + the exact fix (design principle P4:
errors give the cure, not just the symptom — the key lever for AI self-correction).

WHY this exists: the nl->Vais transpiler currently *accepts* some Rust-intuition
spellings (e.g. `&&`, `as Int`) because Vais accepts them after mapping. That
silently opens TWO ways to write one thing — exactly the ambiguity nl forbids
(P1-P3). nl-check enforces "one way" by flagging the non-nl spelling BEFORE
transpile and telling the user the single correct form.

Usage:  python3 nl-check.py file.nl
Exit 0 = clean; exit 1 = issues found (printed with help:).

Format per issue:
    error: <what>
      --> file:line:col
      <source line>
      help: <fix> -> `<corrected code>`

Scope: line-based lint (prototype). Skips string literals and comments.
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
    (re.compile(r"=>\s*return\b"),
     None,  # allowed in nl (P6) — informational only, not an error
     None),
    (re.compile(r"\bfn\s+[A-Z]\b"),
     None, None),  # single-uppercase fn name is fine in nl (keyword space split)
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
        print(f"nl-check: {path} clean")
    return 1 if issues else 0


def main():
    if len(sys.argv) != 2:
        print("usage: nl-check.py file.nl", file=sys.stderr)
        sys.exit(2)
    sys.exit(check(sys.argv[1]))


if __name__ == "__main__":
    main()
