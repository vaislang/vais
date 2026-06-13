#!/usr/bin/env python3
"""New Vais compiler CLI bootstrap wrapper.

This is the user-facing `vaisc` command contract for New Vais. During the
transition it bootstraps through Legacy Vais, but the emitted LLVM IR is produced
by the New Vais self-host compiler in compiler/self/fixpoint_full.nl.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Callable


ROOT = Path(__file__).resolve().parents[1]
FIXPOINT_FULL = ROOT / "compiler" / "self" / "fixpoint_full.nl"
EMBED_SELF_SOURCE = ROOT / "tools" / "embed_self_source.py"
TRANSPILER = ROOT / "compiler" / "transpiler" / "nl2vais.py"
DEFAULT_LEGACY_ROOT = Path("/Users/sswoo/study/projects/vais/compiler")


class CompileError(RuntimeError):
    pass


class FrontContractError(CompileError):
    pass


class DirectEmitError(CompileError):
    pass


@dataclass(frozen=True)
class FrontIssue:
    line: int
    col: int
    message: str
    help: str
    fix: str | None = None


@dataclass(frozen=True)
class FrontRule:
    pattern: re.Pattern[str]
    message: str
    help: str
    fix: Callable[[re.Match[str], str], str | None] | None = None


def code_only(line: str) -> str:
    out: list[str] = []
    string_delim: str | None = None
    escaped = False
    for ch in line:
        if escaped:
            out.append(" ")
            escaped = False
            continue
        if string_delim == '"' and ch == "\\":
            out.append(" ")
            escaped = True
            continue
        if string_delim is None and ch == "#":
            out.append(" " * (len(line) - len(out)))
            break
        if ch in ('"', "`"):
            if string_delim is None:
                string_delim = ch
            elif string_delim == ch:
                string_delim = None
            out.append(" ")
            continue
        out.append(" " if string_delim is not None else ch)
    return "".join(out)


def issue_from_match(
    match: re.Match[str],
    line_no: int,
    raw: str,
    rule: FrontRule,
) -> FrontIssue:
    fix = rule.fix(match, raw.rstrip()) if rule.fix else None
    return FrontIssue(line_no, match.start() + 1, rule.message, rule.help, fix)


def caret_at(col: int) -> str:
    return " " * max(col - 1, 0) + "^"


def replace_once(old: str, new: str) -> Callable[[re.Match[str], str], str]:
    def fix(_: re.Match[str], raw: str) -> str:
        return raw.replace(old, new, 1)

    return fix


def fix_not(match: re.Match[str], raw: str) -> str:
    return raw[: match.start()] + "not " + raw[match.end() :]


def fix_as(match: re.Match[str], raw: str) -> str:
    target = match.group(1)
    prefix = raw[: match.start()].rstrip()
    expr = re.search(r"([A-Za-z_][A-Za-z0-9_]*|\d+|\([^)]*\))\s*$", prefix)
    if expr is None:
        return f"use `{target}(expr)` instead of `expr as {target}`"
    expr_text = expr.group(1)
    return raw[: expr.start(1)] + f"{target}({expr_text})" + raw[match.end() :]


def fix_scalar_type(match: re.Match[str], raw: str) -> str:
    mapping = {
        "i8": "Int8",
        "i16": "Int16",
        "i32": "Int",
        "i64": "Int",
        "i128": "Int128",
        "u8": "UInt8",
        "u16": "UInt16",
        "u32": "UInt32",
        "u64": "UInt64",
        "u128": "UInt128",
        "f32": "F32",
        "f64": "F64",
        "usize": "Int",
        "isize": "Int",
    }
    return raw[: match.start()] + mapping[match.group(1)] + raw[match.end() :]


def fix_turbofish_new(match: re.Match[str], raw: str) -> str:
    return raw[: match.start()] + "[]" + raw[match.end() :]


FRONT_UNSUPPORTED_RULES: list[tuple[re.Pattern[str], str, str]] = [
    (
        re.compile(r"\benum\b"),
        "enum declarations are not in the New Vais native day-1 front subset yet",
        "model this day-1 case with Int tags, or keep enums on the Legacy bootstrap path until NV-C4.",
    ),
    (
        re.compile(r"\bmatch\b"),
        "`match` is not in the New Vais native day-1 front subset yet",
        "use if/else for day-1 native sources, or keep match-based code on the Legacy bootstrap path.",
    ),
    (
        re.compile(r"\bfor\b"),
        "`for` loops are not in the New Vais native day-1 front subset yet",
        "use `while` with an explicit mutable index for now.",
    ),
    (
        re.compile(r"\b(Str|Char|Bool)\b"),
        "only Int scalar typing is in the New Vais native day-1 front subset",
        "use Int parameters/locals for this slice; string, char, and bool surface types come later.",
    ),
    (
        re.compile(r"\b(true|false)\b"),
        "boolean literals are not in the New Vais native day-1 front subset yet",
        "use Int 0/1 values or comparisons in this slice.",
    ),
    (
        re.compile(r"\b(Map|Option|Result)\s*<|\blist\s*\("),
        "map and sum-result types are not in the New Vais native front subset yet",
        "keep this source on the Legacy bootstrap path until the native parity gate grows to it.",
    ),
    (
        re.compile(r"\b(break|continue)\b"),
        "loop control statements are not in the New Vais native day-1 front subset yet",
        "rewrite the loop with an explicit condition variable for now.",
    ),
    (
        re.compile(r"\b(trait|impl)\b"),
        "traits and impl blocks are not in the New Vais native day-1 front subset yet",
        "use plain functions for this slice.",
    ),
    (
        re.compile(r"\|[^|]*\|"),
        "closures are not in the New Vais native day-1 front subset yet",
        "write a named function for this slice; closure object parity is tracked after the front contract.",
    ),
    (
        re.compile(r"\.(?!(?:push|len|sum)\s*\()[A-Za-z_][A-Za-z0-9_]*\s*\("),
        "method calls beyond push/len/sum are not in the New Vais native front subset yet",
        "use a plain function call, or keep this source on the Legacy bootstrap path until that method is promoted.",
    ),
    (
        re.compile(r"\?"),
        "error propagation with `?` is not in the New Vais native day-1 front subset yet",
        "return an Int status code explicitly in this slice.",
    ),
]


FRONT_HELP_RULES: list[FrontRule] = [
    FrontRule(
        re.compile(r"&&"),
        "logical AND uses the word `and`, not `&&`",
        "replace `&&` with `and`.",
        replace_once("&&", "and"),
    ),
    FrontRule(
        re.compile(r"\|\|"),
        "logical OR uses the word `or`, not `||`",
        "replace `||` with `or`.",
        replace_once("||", "or"),
    ),
    FrontRule(
        re.compile(r"(?<![<>=!])!(?!=)(?=\s*\w|\s*\()"),
        "logical NOT uses the word `not`, not `!`",
        "replace `!expr` with `not expr`.",
        fix_not,
    ),
    FrontRule(
        re.compile(r"\bas\s+([A-Za-z_]\w*)"),
        "type conversion is explicit `Type(x)`, not `x as Type`",
        "write `Type(expr)` instead of `expr as Type`.",
        fix_as,
    ),
    FrontRule(
        re.compile(r"\b[A-Za-z_]\w*::"),
        "enum/path access uses `.`, not `::`",
        "replace `::` with `.`.",
        replace_once("::", "."),
    ),
    FrontRule(
        re.compile(r"\b\w+<[^>]*>::new\s*\(\)"),
        "no turbofish constructor; use a literal instead of `Type<...>::new()`",
        "use a list/map literal such as `[]`, `[1, 2]`, or `{}`.",
        fix_turbofish_new,
    ),
    FrontRule(
        re.compile(r"\bvec!\s*\["),
        "list literals are just `[a, b]`, not `vec![...]`",
        "replace `vec![...]` with `[ ... ]`.",
        lambda m, raw: raw[: m.start()] + "[" + raw[m.end() :],
    ),
    FrontRule(
        re.compile(r"\bVec\s*<"),
        "the New Vais list type is `List<T>`, not `Vec<T>`",
        "replace `Vec<T>` with `List<T>`.",
        lambda m, raw: raw[: m.start()] + "List" + raw[m.end() - 1 :],
    ),
    FrontRule(
        re.compile(r"\bHashMap\b"),
        "the New Vais map spelling is `Map<K,V>`, not `HashMap<K,V>`",
        "use `Map<K,V>` on the Legacy path for now; day-1 native front is scalar-only.",
        replace_once("HashMap", "Map"),
    ),
    FrontRule(
        re.compile(
            r"(?<![A-Za-z0-9_])(i8|i16|i32|i64|i128|u8|u16|u32|u64|u128|f32|f64|usize|isize)(?![A-Za-z0-9_])"
        ),
        "New Vais scalar types are capitalized, not Rust scalar names",
        "use `Int`, `Int8..Int128`, `UInt8..UInt128`, `F32`, or `F64`.",
        fix_scalar_type,
    ),
    FrontRule(
        re.compile(r"\bString\b"),
        "the string type is `Str`, not `String`",
        "use `Str` on the Legacy path for now; day-1 native front is scalar-only.",
        replace_once("String", "Str"),
    ),
    FrontRule(
        re.compile(r"[+\-*/%]=(?!=)"),
        "compound assignment is not New Vais syntax",
        "write it out, e.g. `x = x + 1`.",
    ),
]


FRONT_FN_HEADER = re.compile(
    r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*(?:->\s*([A-Za-z_][A-Za-z0-9_]*))?"
)
FRONT_INT_PARAM = re.compile(r"[A-Za-z_][A-Za-z0-9_]*\s*:\s*Int\Z")


def check_function_contracts(code: str, line_no: int) -> tuple[list[FrontIssue], bool, bool]:
    issues: list[FrontIssue] = []
    has_main = False
    has_bad_main = False

    for match in FRONT_FN_HEADER.finditer(code):
        name = match.group(1)
        params = match.group(2).strip()
        return_type = match.group(3)

        if name == "main":
            if not params and return_type == "Int":
                has_main = True
            else:
                has_bad_main = True
            continue

        if return_type != "Int":
            issues.append(
                FrontIssue(
                    line_no,
                    match.start() + 1,
                    "New Vais native day-1 helper functions must return `Int`",
                    "write helpers as `fn name(a: Int, ...) -> Int { ... }`.",
                )
            )

        if params and any(not FRONT_INT_PARAM.fullmatch(part.strip()) for part in params.split(",")):
            issues.append(
                FrontIssue(
                    line_no,
                    match.start(2) + 1,
                    "New Vais native day-1 helper parameters must be `name: Int`",
                    "use Int-typed helper parameters in this slice, e.g. `fn add(a: Int, b: Int) -> Int`.",
                )
            )

    return issues, has_main, has_bad_main


def check_front_contract(source: Path) -> None:
    lines = source.read_text().splitlines()
    issues: list[FrontIssue] = []
    has_main = False
    has_bad_main = False

    for line_no, raw in enumerate(lines, 1):
        code = code_only(raw)
        fn_issues, line_has_main, line_has_bad_main = check_function_contracts(code, line_no)
        issues.extend(fn_issues)
        has_main = has_main or line_has_main
        has_bad_main = has_bad_main or line_has_bad_main

        for rule in FRONT_HELP_RULES:
            match = rule.pattern.search(code)
            if match:
                issues.append(issue_from_match(match, line_no, raw, rule))
                break
        else:
            for pattern, message, help_text in FRONT_UNSUPPORTED_RULES:
                match = pattern.search(code)
                if match:
                    issues.append(FrontIssue(line_no, match.start() + 1, message, help_text))
                    break

    if has_bad_main and not has_main:
        message = "New Vais native day-1 front requires `fn main() -> Int` exactly"
        help_text = "write the entrypoint as `fn main() -> Int { ... }`."
        issues.insert(0, FrontIssue(1, 1, message, help_text))
    elif not has_main:
        message = "New Vais native day-1 front requires `fn main() -> Int`"
        help_text = "add `fn main() -> Int { return <int> }` as the program entrypoint."
        issues.insert(0, FrontIssue(1, 1, message, help_text))

    if not issues:
        return

    formatted: list[str] = []
    for issue in issues:
        raw = lines[issue.line - 1] if 0 <= issue.line - 1 < len(lines) else ""
        formatted.extend(
            [
                f"error: {issue.message}",
                f"  --> {source}:{issue.line}:{issue.col}",
                f"  {raw}",
                f"  {caret_at(issue.col)}",
                f"  help: {issue.help}",
            ]
        )
        if issue.fix:
            formatted.append(f"  fix: {issue.fix}")
        formatted.append("")
    raise FrontContractError("\n".join(formatted).rstrip())


@dataclass(frozen=True)
class DirectFunction:
    name: str
    params: str
    return_type: str | None
    body: str
    body_start: int
    line: int
    col: int


@dataclass(frozen=True)
class DirectToken:
    kind: str
    text: str
    line: int
    col: int


@dataclass(frozen=True)
class DirectValue:
    ty: str
    ref: str


DIRECT_FN_START = re.compile(
    r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*(?:->\s*([A-Za-z_][A-Za-z0-9_]*))?\s*\{"
)
DIRECT_INT_RE = re.compile(r"\d+")
DIRECT_OPS = {"==", "!=", "<=", ">=", "+", "-", "*", "/", "%", "(", ")", "<", ">"}
DIRECT_BINOPS = {
    "+": "add",
    "-": "sub",
    "*": "mul",
    "/": "sdiv",
    "%": "srem",
}
DIRECT_CMPOPS = {
    "==": "eq",
    "!=": "ne",
    "<": "slt",
    "<=": "sle",
    ">": "sgt",
    ">=": "sge",
}


def line_col_at(text: str, index: int) -> tuple[int, int]:
    line = text.count("\n", 0, index) + 1
    prev = text.rfind("\n", 0, index)
    col = index + 1 if prev < 0 else index - prev
    return line, col


def source_line(source: Path, line: int) -> str:
    try:
        lines = source.read_text().splitlines()
    except OSError:
        return ""
    if 1 <= line <= len(lines):
        return lines[line - 1]
    return ""


def direct_error(
    source: Path,
    line: int,
    col: int,
    message: str,
    help_text: str,
    fix_text: str | None = None,
) -> DirectEmitError:
    raw = source_line(source, line)
    formatted = [
        f"error: {message}",
        f"  --> {source}:{line}:{col}",
        f"  {raw}",
        f"  {caret_at(col)}",
        f"  help: {help_text}",
    ]
    if fix_text:
        formatted.append(f"  fix: {fix_text}")
    return DirectEmitError("\n".join(formatted))


def source_code_view(text: str) -> str:
    return "\n".join(code_only(line) for line in text.splitlines())


def find_matching_brace(text: str, open_index: int, source: Path) -> int:
    depth = 0
    for index in range(open_index, len(text)):
        ch = text[index]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return index
    line, col = line_col_at(text, open_index)
    raise direct_error(
        source,
        line,
        col,
        "direct LLVM emitter found an unterminated function body",
        "close the function body with `}`.",
    )


def extract_direct_functions(source: Path, code: str) -> list[DirectFunction]:
    functions: list[DirectFunction] = []
    cursor = 0
    while True:
        match = DIRECT_FN_START.search(code, cursor)
        if match is None:
            break
        open_index = match.end() - 1
        close_index = find_matching_brace(code, open_index, source)
        line, col = line_col_at(code, match.start())
        functions.append(
            DirectFunction(
                name=match.group(1),
                params=match.group(2).strip(),
                return_type=match.group(3),
                body=code[open_index + 1 : close_index],
                body_start=open_index + 1,
                line=line,
                col=col,
            )
        )
        cursor = close_index + 1
    return functions


def advance_position(text: str, line: int, col: int) -> tuple[int, int]:
    for ch in text:
        if ch == "\n":
            line += 1
            col = 1
        else:
            col += 1
    return line, col


def tokenize_direct_expr(source: Path, expr: str, start_line: int, start_col: int) -> list[DirectToken]:
    tokens: list[DirectToken] = []
    index = 0
    line = start_line
    col = start_col
    while index < len(expr):
        ch = expr[index]
        if ch.isspace():
            line, col = advance_position(ch, line, col)
            index += 1
            continue

        int_match = DIRECT_INT_RE.match(expr, index)
        if int_match:
            text = int_match.group(0)
            tokens.append(DirectToken("int", text, line, col))
            index = int_match.end()
            line, col = advance_position(text, line, col)
            continue

        two = expr[index : index + 2]
        if two in DIRECT_OPS:
            tokens.append(DirectToken(two, two, line, col))
            index += 2
            line, col = advance_position(two, line, col)
            continue

        if ch in DIRECT_OPS:
            tokens.append(DirectToken(ch, ch, line, col))
            index += 1
            line, col = advance_position(ch, line, col)
            continue

        if ch.isalpha() or ch == "_":
            raise direct_error(
                source,
                line,
                col,
                "direct LLVM emitter currently supports literal Int expressions only",
                "use integer literals and arithmetic here, or use the default bootstrap engine.",
                "return 40 + 2",
            )

        raise direct_error(
            source,
            line,
            col,
            f"direct LLVM emitter cannot parse token `{ch}`",
            "use integer literals, arithmetic operators, comparisons, and parentheses.",
        )

    tokens.append(DirectToken("eof", "", line, col))
    return tokens


class DirectExprParser:
    def __init__(self, source: Path, tokens: list[DirectToken]) -> None:
        self.source = source
        self.tokens = tokens
        self.pos = 0
        self.tmp = 0
        self.instructions: list[str] = []

    def current(self) -> DirectToken:
        return self.tokens[self.pos]

    def take(self, kind: str) -> DirectToken | None:
        if self.current().kind == kind:
            token = self.current()
            self.pos += 1
            return token
        return None

    def expect(self, kind: str, help_text: str) -> DirectToken:
        token = self.current()
        if token.kind != kind:
            raise direct_error(
                self.source,
                token.line,
                token.col,
                f"direct LLVM emitter expected `{kind}`",
                help_text,
            )
        self.pos += 1
        return token

    def new_tmp(self) -> str:
        self.tmp += 1
        return f"%t{self.tmp}"

    def ensure_i64(self, value: DirectValue) -> DirectValue:
        if value.ty == "i64":
            return value
        if value.ty == "i1":
            out = self.new_tmp()
            self.instructions.append(f"  {out} = zext i1 {value.ref} to i64")
            return DirectValue("i64", out)
        raise AssertionError(f"unknown direct value type {value.ty}")

    def emit_i64_binop(self, op: str, left: DirectValue, right: DirectValue) -> DirectValue:
        left = self.ensure_i64(left)
        right = self.ensure_i64(right)
        out = self.new_tmp()
        self.instructions.append(f"  {out} = {DIRECT_BINOPS[op]} i64 {left.ref}, {right.ref}")
        return DirectValue("i64", out)

    def emit_cmp(self, op: str, left: DirectValue, right: DirectValue) -> DirectValue:
        left = self.ensure_i64(left)
        right = self.ensure_i64(right)
        out = self.new_tmp()
        self.instructions.append(f"  {out} = icmp {DIRECT_CMPOPS[op]} i64 {left.ref}, {right.ref}")
        return DirectValue("i1", out)

    def parse(self) -> DirectValue:
        value = self.parse_compare()
        token = self.current()
        if token.kind != "eof":
            raise direct_error(
                self.source,
                token.line,
                token.col,
                f"direct LLVM emitter found trailing token `{token.text}`",
                "keep the return expression to one arithmetic/comparison expression.",
            )
        return self.ensure_i64(value)

    def parse_compare(self) -> DirectValue:
        value = self.parse_sum()
        while self.current().kind in DIRECT_CMPOPS:
            op = self.current().kind
            self.pos += 1
            value = self.emit_cmp(op, value, self.parse_sum())
        return value

    def parse_sum(self) -> DirectValue:
        value = self.parse_term()
        while self.current().kind in {"+", "-"}:
            op = self.current().kind
            self.pos += 1
            value = self.emit_i64_binop(op, value, self.parse_term())
        return value

    def parse_term(self) -> DirectValue:
        value = self.parse_unary()
        while self.current().kind in {"*", "/", "%"}:
            op = self.current().kind
            self.pos += 1
            value = self.emit_i64_binop(op, value, self.parse_unary())
        return value

    def parse_unary(self) -> DirectValue:
        if self.take("-"):
            value = self.ensure_i64(self.parse_unary())
            if value.ref.isdigit():
                return DirectValue("i64", f"-{value.ref}")
            out = self.new_tmp()
            self.instructions.append(f"  {out} = sub i64 0, {value.ref}")
            return DirectValue("i64", out)
        return self.parse_primary()

    def parse_primary(self) -> DirectValue:
        token = self.current()
        if token.kind == "int":
            self.pos += 1
            return DirectValue("i64", token.text)
        if self.take("("):
            value = self.parse_compare()
            self.expect(")", "close the parenthesized expression.")
            return value
        raise direct_error(
            self.source,
            token.line,
            token.col,
            "direct LLVM emitter expected an Int literal or parenthesized expression",
            "use a return expression such as `40 + 2`.",
        )


def extract_direct_return_expr(
    source: Path,
    code: str,
    function: DirectFunction,
) -> tuple[str, int, int]:
    match = re.fullmatch(r"\s*return\s+(.+?)\s*;?\s*", function.body, re.DOTALL)
    if match is None:
        line, col = line_col_at(code, function.body_start)
        raise direct_error(
            source,
            line,
            col,
            "direct LLVM emitter currently supports a single `return` statement in `main`",
            "write `fn main() -> Int { return 40 + 2 }` for the NV-C2 direct slice.",
            "fn main() -> Int { return 40 + 2 }",
        )
    line, col = line_col_at(code, function.body_start + match.start(1))
    return match.group(1).strip(), line, col


def direct_emit_ir(source: Path, ir_out: Path | None) -> str | None:
    if not source.is_file():
        raise CompileError(f"source not found: {source}")
    check_front_contract(source)

    text = source.read_text()
    code = source_code_view(text)
    functions = extract_direct_functions(source, code)
    if len(functions) != 1 or functions[0].name != "main":
        raise direct_error(
            source,
            1,
            1,
            "direct LLVM emitter currently supports only a single `fn main() -> Int`",
            "use the default bootstrap engine for helper functions until the direct emitter grows calls.",
        )

    main_fn = functions[0]
    expr, expr_line, expr_col = extract_direct_return_expr(source, code, main_fn)
    tokens = tokenize_direct_expr(source, expr, expr_line, expr_col)
    parser = DirectExprParser(source, tokens)
    result = parser.parse()
    ir_lines = ["define i64 @main() {", *parser.instructions, f"  ret i64 {result.ref}", "}", ""]
    ir = "\n".join(ir_lines)

    if ir_out is None:
        return ir
    ir_out.parent.mkdir(parents=True, exist_ok=True)
    ir_out.write_text(ir)
    return None


def emit_ir_for_args(source: Path, ir_out: Path | None, args: argparse.Namespace) -> str | None:
    if args.engine == "direct":
        return direct_emit_ir(source, ir_out)
    return bootstrap_emit_ir(source, ir_out, args)


def run_checked(
    cmd: list[str],
    *,
    cwd: Path | None = None,
    stdout=None,
    stderr=None,
) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        cmd,
        cwd=str(cwd) if cwd else None,
        text=True,
        stdout=stdout,
        stderr=stderr,
    )
    if proc.returncode != 0:
        raise CompileError(
            f"command failed ({proc.returncode}): {' '.join(cmd)}"
        )
    return proc


def resolve_legacy_vaisc(legacy_root: Path) -> str:
    override = os.environ.get("LEGACY_VAISC")
    candidates: list[Path] = []
    if override:
        candidates.append(Path(override))
    candidates.extend(
        [
            legacy_root / "target" / "debug" / "vaisc",
            legacy_root / "target" / "release" / "vaisc",
            DEFAULT_LEGACY_ROOT / "target" / "debug" / "vaisc",
            DEFAULT_LEGACY_ROOT / "target" / "release" / "vaisc",
        ]
    )
    for candidate in candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)

    found = shutil.which("vaisc")
    if found:
        resolved = Path(found).resolve()
        repo_wrappers = {
            (ROOT / "scripts" / "vaisc").resolve(),
            Path(__file__).resolve(),
        }
        if resolved not in repo_wrappers:
            return found

    raise CompileError(
        "could not find Legacy Vais bootstrap compiler. "
        "Set LEGACY_VAISC=/path/to/legacy/vaisc or build "
        "/Users/sswoo/study/projects/vais/compiler."
    )


def tmpdir_from_args(args: argparse.Namespace) -> tuple[Path, tempfile.TemporaryDirectory[str] | None]:
    if args.keep_tmp:
        path = Path(tempfile.mkdtemp(prefix="vaisc-"))
        print(f"debug: keeping temp dir {path}", file=sys.stderr)
        return path, None
    holder = tempfile.TemporaryDirectory(prefix="vaisc-")
    return Path(holder.name), holder


def bootstrap_emit_ir(source: Path, ir_out: Path | None, args: argparse.Namespace) -> str | None:
    if not source.is_file():
        raise CompileError(f"source not found: {source}")
    check_front_contract(source)

    tmp, holder = tmpdir_from_args(args)
    try:
        harness = tmp / "compiler_harness.nl"
        harness_vais = tmp / "compiler_harness.vais"
        stage0 = tmp / "stage0-vaisc"
        transpile_err = tmp / "transpile.err"
        build_log = tmp / "legacy-build.log"

        run_checked(
            [
                sys.executable,
                str(EMBED_SELF_SOURCE),
                str(FIXPOINT_FULL),
                str(source),
                str(harness),
            ]
        )
        with harness_vais.open("w") as out, transpile_err.open("w") as err:
            run_checked([sys.executable, str(TRANSPILER), str(harness)], stdout=out, stderr=err)

        legacy_root = Path(args.legacy_root).resolve()
        legacy_vaisc = resolve_legacy_vaisc(legacy_root)
        shutil.rmtree("/tmp/.vais-cache", ignore_errors=True)
        with build_log.open("w") as log:
            run_checked(
                [legacy_vaisc, "build", str(harness_vais), "-o", str(stage0)],
                cwd=legacy_root,
                stdout=log,
                stderr=subprocess.STDOUT,
            )

        if ir_out is None:
            proc = subprocess.run([str(stage0)], text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            if proc.returncode != 0:
                raise CompileError(f"generated New Vais compiler exited {proc.returncode}: {proc.stderr}")
            return proc.stdout

        ir_out.parent.mkdir(parents=True, exist_ok=True)
        with ir_out.open("w") as out:
            proc = subprocess.run([str(stage0)], text=True, stdout=out, stderr=subprocess.PIPE)
        if proc.returncode != 0:
            raise CompileError(f"generated New Vais compiler exited {proc.returncode}: {proc.stderr}")
        return None
    finally:
        if holder is not None:
            holder.cleanup()


def emit_ir(args: argparse.Namespace) -> int:
    source = Path(args.source).resolve()
    if args.output == "-":
        text = emit_ir_for_args(source, None, args)
        assert text is not None
        sys.stdout.write(text)
        return 0

    emit_ir_for_args(source, Path(args.output).resolve(), args)
    return 0


def build(args: argparse.Namespace) -> int:
    source = Path(args.source).resolve()
    out = Path(args.output).resolve()
    tmp, holder = tmpdir_from_args(args)
    try:
        ir_path = Path(args.ir_out).resolve() if args.ir_out else tmp / "out.ll"
        emit_ir_for_args(source, ir_path, args)
        out.parent.mkdir(parents=True, exist_ok=True)
        run_checked(
            [args.clang, "-Wno-override-module", "-o", str(out), str(ir_path)],
            stderr=subprocess.PIPE,
        )
        return 0
    finally:
        if holder is not None:
            holder.cleanup()


def run_program(args: argparse.Namespace) -> int:
    tmp, holder = tmpdir_from_args(args)
    try:
        bin_path = tmp / "a.out"
        build_args = argparse.Namespace(**vars(args))
        build_args.output = str(bin_path)
        build_args.ir_out = None
        build(build_args)
        proc = subprocess.run([str(bin_path)])
        return proc.returncode
    finally:
        if holder is not None:
            holder.cleanup()


def add_common_flags(parser: argparse.ArgumentParser) -> None:
    parser.add_argument(
        "--engine",
        choices=("bootstrap", "direct"),
        default=os.environ.get("VAISC_ENGINE", "bootstrap"),
        help=(
            "Compiler engine. `bootstrap` uses the full transitional self-host path; "
            "`direct` uses the NV-C2 minimal LLVM emitter."
        ),
    )
    parser.add_argument(
        "--legacy-root",
        default=os.environ.get("VAIS_COMPILER_ROOT", str(DEFAULT_LEGACY_ROOT)),
        help="Legacy Vais compiler repo used only as bootstrap/oracle backend.",
    )
    parser.add_argument(
        "--keep-tmp",
        action="store_true",
        help="Keep the temporary bootstrap directory for debugging.",
    )


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="vaisc",
        description=(
            "New Vais compiler CLI. Accepts .vais and transitional .nl sources "
            "that fit the native day-1 front contract."
        ),
    )
    sub = parser.add_subparsers(dest="command", required=True)

    emit = sub.add_parser("emit-ir", help="Compile New Vais source to LLVM IR.")
    emit.add_argument("source")
    emit.add_argument("-o", "--output", default="-", help="LLVM IR output path, or '-' for stdout.")
    add_common_flags(emit)
    emit.set_defaults(func=emit_ir)

    bld = sub.add_parser("build", help="Compile New Vais source to a native binary.")
    bld.add_argument("source")
    bld.add_argument("-o", "--output", required=True, help="Native binary output path.")
    bld.add_argument("--ir-out", help="Optional path to also keep emitted LLVM IR.")
    bld.add_argument("--clang", default=os.environ.get("CLANG", "clang"))
    add_common_flags(bld)
    bld.set_defaults(func=build)

    run = sub.add_parser("run", help="Compile and run New Vais source, returning the program exit code.")
    run.add_argument("source")
    run.add_argument("--clang", default=os.environ.get("CLANG", "clang"))
    add_common_flags(run)
    run.set_defaults(func=run_program)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = make_parser()
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except (FrontContractError, DirectEmitError) as exc:
        print(str(exc), file=sys.stderr)
        return 1
    except CompileError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
