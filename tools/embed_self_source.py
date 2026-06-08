#!/usr/bin/env python3
"""Embed a normalized self-host source file into fixpoint_full.nl.

This is a bootstrap harness helper, not the language frontend. The current
fixpoint_full compiler consumes a compact semicolon-oriented subset from a
single string literal. Real nl source files are line-oriented, contain comments,
use double-quoted strings, and write typed struct fields. This helper normalizes
that surface just enough to feed real source files through the current
fixpoint_full path.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


COMPILE_RE = re.compile(r'compile\("(?:[^"\\]|\\.)*"\)')


def strip_comment(line: str) -> str:
    out: list[str] = []
    in_dquote = False
    in_backtick = False
    escaped = False
    for ch in line:
        if escaped:
            out.append(ch)
            escaped = False
            continue
        if ch == "\\" and in_dquote:
            out.append(ch)
            escaped = True
            continue
        if ch == '"' and not in_backtick:
            in_dquote = not in_dquote
            out.append(ch)
            continue
        if ch == "`" and not in_dquote:
            in_backtick = not in_backtick
            out.append(ch)
            continue
        if ch == "#" and not in_dquote and not in_backtick:
            break
        out.append(ch)
    return "".join(out).rstrip()


def double_strings_to_backticks(line: str) -> str:
    def repl(match: re.Match[str]) -> str:
        body = match.group(0)[1:-1]
        return "`" + body + "`"

    return re.sub(r'"(?:[^"\\]|\\.)*"', repl, line)


def strip_one_line_struct_field_types(line: str) -> str:
    stripped = line.lstrip()
    if not (stripped.startswith("struct ") and "{" in line and "}" in line):
        return line
    before, rest = line.split("{", 1)
    body, after = rest.rsplit("}", 1)
    fields = []
    for part in body.split(","):
        name = part.strip().split(":", 1)[0].strip()
        if name:
            fields.append(name)
    return before + "{ " + ", ".join(fields) + " }" + after


def strip_struct_field_types(program: str) -> str:
    def repl(match: re.Match[str]) -> str:
        before = match.group(1)
        body = match.group(2)
        fields = []
        for part in body.split(","):
            name = part.strip().split(":", 1)[0].strip()
            if name:
                fields.append(name)
        return before + "{ " + ", ".join(fields) + " }"

    return re.sub(r"\b(struct\s+[A-Za-z_][A-Za-z0-9_]*\s*)\{([^{}]*)\}", repl, program)


def collapse_string_brace_escapes(program: str) -> str:
    out: list[str] = []
    in_backtick = False
    i = 0
    while i < len(program):
        ch = program[i]
        if ch == "`":
            in_backtick = not in_backtick
            out.append(ch)
            i += 1
        elif in_backtick and program.startswith("{{", i):
            out.append("{")
            i += 2
        elif in_backtick and program.startswith("}}", i):
            out.append("}")
            i += 2
        else:
            out.append(ch)
            i += 1
    return "".join(out)


def add_inline_semis(line: str) -> str:
    # Compact one-line helpers such as `fn is_digit(...) { return ... }`.
    line = re.sub(
        r"\breturn\s+([^{};]+)\s*}",
        lambda m: "return " + m.group(1).strip() + "; }",
        line,
    )
    return line


def needs_line_semi(line: str) -> bool:
    stripped = line.strip()
    if not stripped or stripped.endswith(";"):
        return False
    if stripped.endswith("{") or stripped.endswith("}") or stripped.startswith("else"):
        return False
    if stripped.startswith("} else") or stripped.startswith("struct "):
        return False
    # Multi-line call endings, e.g. `fns.push(Fn { ... })`, need to remain a
    # separate statement after lines are joined into the compact source string.
    if stripped.endswith(")"):
        return True
    if stripped.startswith(("let ", "return ", "print(")):
        return True
    if re.match(r"^[A-Za-z_][A-Za-z0-9_]*(\.|\[|\s*=|\()", stripped):
        return True
    return False


def normalize_source(path: Path) -> str:
    out: list[str] = []
    for raw in path.read_text().splitlines():
        line = strip_comment(raw)
        if not line.strip():
            continue
        line = double_strings_to_backticks(line)
        line = strip_one_line_struct_field_types(line)
        line = add_inline_semis(line)
        if needs_line_semi(line):
            line = line.rstrip() + ";"
        out.append(line.strip())

    program = " ".join(out)
    program = strip_struct_field_types(program)
    program = collapse_string_brace_escapes(program)
    # Protect braces for the outer compile("...") string. The outer transpiler
    # lowers doubled braces to literal braces, so fixpoint_full sees the source
    # with single braces at runtime.
    program = program.replace("{", "{{").replace("}", "}}")
    program = program.replace("\\", "\\\\").replace('"', '\\"')
    return program


def main() -> int:
    if len(sys.argv) != 4:
        print(
            "usage: embed_self_source.py FIXPOINT_FULL.nl SOURCE.nl OUT.nl",
            file=sys.stderr,
        )
        return 2

    harness = Path(sys.argv[1])
    source = Path(sys.argv[2])
    out_path = Path(sys.argv[3])

    program = normalize_source(source)
    text = harness.read_text()
    replaced, count = COMPILE_RE.subn('compile("' + program + '")', text, count=1)
    if count != 1:
        print("error: could not find a compile(\"...\") call to replace", file=sys.stderr)
        return 1
    out_path.write_text(replaced)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
