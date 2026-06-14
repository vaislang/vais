#!/usr/bin/env python3
"""Embed a normalized self-host source file into fixpoint_full.vais.

This is a source embedding helper, not the language frontend. The current
fixpoint_full compiler consumes a compact semicolon-oriented subset from a
single string literal. Real Vais source files are line-oriented, contain comments,
use double-quoted strings, and write typed struct fields. This helper normalizes
that surface just enough to feed real source files through the current
fixpoint_full path.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


COMPILE_RE = re.compile(r'compile\("(?:[^"\\]|\\.)*"\)')
PRESERVED_DQUOTE_WITH_BACKTICK_RE = re.compile(r'"(?:[^"\\]|\\.)*`(?:[^"\\]|\\.)*"')


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
    def unescape_emit_body(body: str) -> str:
        out: list[str] = []
        i = 0
        while i < len(body):
            ch = body[i]
            if ch == "\\" and i + 1 < len(body):
                nxt = body[i + 1]
                if nxt == "\\" or nxt == '"':
                    out.append(nxt)
                    i += 2
                    continue
            out.append(ch)
            i += 1
        return "".join(out)

    def repl(match: re.Match[str]) -> str:
        body = match.group(0)[1:-1]
        if "`" in body:
            return match.group(0)
        prefix = line[: match.start()].rstrip()
        if re.search(r"\bemit_str\s*\(\s*$", prefix):
            body = unescape_emit_body(body)
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
    string_delim: str | None = None
    escaped = False
    i = 0
    while i < len(program):
        ch = program[i]
        if escaped:
            out.append(ch)
            escaped = False
            i += 1
        elif string_delim == '"' and ch == "\\":
            out.append(ch)
            escaped = True
            i += 1
        elif ch in ('"', "`"):
            if string_delim is None:
                string_delim = ch
            elif string_delim == ch:
                string_delim = None
            out.append(ch)
            i += 1
        elif string_delim is not None and program.startswith("{{", i):
            out.append("{")
            i += 2
        elif string_delim is not None and program.startswith("}}", i):
            out.append("}")
            i += 2
        else:
            out.append(ch)
            i += 1
    return "".join(out)


INLINE_STMT_RE = re.compile(
    r"^(?:let\b|return\b|print\s*\(|putchar\s*\(|[A-Za-z_][A-Za-z0-9_]*(?:\s*=|\[|\.|\())"
)


def inline_body_needs_semi(body: str) -> bool:
    stripped = body.strip()
    if not stripped or stripped.endswith(";") or stripped.endswith("}"):
        return False
    if stripped.startswith(("if ", "else", "while ", "for ", "fn ", "struct ")):
        return False
    return INLINE_STMT_RE.match(stripped) is not None


def add_inline_semis(line: str) -> str:
    # Compact one-line helpers such as `fn is_digit(...) { return ... }` and
    # one-line branch bodies such as `if ok { go = false }`.
    def block_repl(match: re.Match[str]) -> str:
        body = match.group(1)
        if inline_body_needs_semi(body):
            return "{" + body.rstrip() + "; }"
        return match.group(0)

    prev = None
    while prev != line:
        prev = line
        line = re.sub(r"\{([^{}]*)\}", block_repl, line)

    def tail_repl(match: re.Match[str]) -> str:
        body = match.group(1)
        if inline_body_needs_semi(body):
            return ";" + body.rstrip() + "; }"
        return match.group(0)

    prev = None
    while prev != line:
        prev = line
        line = re.sub(r";([^{};]*)\s*}", tail_repl, line)
    return line


def needs_line_semi(line: str) -> bool:
    stripped = line.strip()
    if not stripped or stripped.endswith(";"):
        return False
    if stripped.startswith("} else") or stripped.startswith("struct "):
        return False
    if stripped.endswith("{") or stripped == "else" or stripped.startswith("else "):
        return False
    if stripped.endswith("}"):
        if stripped.startswith(("let ", "return ")):
            return True
        if re.match(r"^[A-Za-z_][A-Za-z0-9_]*(\.|\[|\s*=)", stripped):
            return True
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
        if PRESERVED_DQUOTE_WITH_BACKTICK_RE.search(line) is None:
            line = add_inline_semis(line)
        if needs_line_semi(line):
            line = line.rstrip() + ";"
        out.append(line.strip())

    program = " ".join(out)
    program = strip_struct_field_types(program)
    program = collapse_string_brace_escapes(program)
    program = program.replace("\\", "\\\\").replace('"', '\\"')
    return program


def main() -> int:
    if len(sys.argv) != 4:
        print(
            "usage: embed_self_source.py FIXPOINT_FULL.vais SOURCE.vais OUT.vais",
            file=sys.stderr,
        )
        return 2

    harness = Path(sys.argv[1])
    source = Path(sys.argv[2])
    out_path = Path(sys.argv[3])
    for path in (harness, source, out_path):
        if path.suffix != ".vais":
            print(f"error: expected .vais path: {path}", file=sys.stderr)
            return 2

    program = normalize_source(source)
    text = harness.read_text()
    replaced, count = COMPILE_RE.subn(lambda _m: 'compile("' + program + '")', text, count=1)
    if count != 1:
        print("error: could not find a compile(\"...\") call to replace", file=sys.stderr)
        return 1
    out_path.write_text(replaced)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
