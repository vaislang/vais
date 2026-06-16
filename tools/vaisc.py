#!/usr/bin/env python3
"""Vais compiler CLI."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Callable


ROOT = Path(__file__).resolve().parents[1]
FIXPOINT_FULL = ROOT / "compiler" / "self" / "fixpoint_full.vais"
CORE_LL = ROOT / "compiler" / "self" / "vaisc_core.ll"
SELF_HOST_TIER_SOURCES = {
    (ROOT / "compiler" / "self" / "fixpoint.vais").resolve(),
    (ROOT / "compiler" / "self" / "fixpoint2.vais").resolve(),
    (ROOT / "compiler" / "self" / "fixpoint3.vais").resolve(),
    (ROOT / "compiler" / "self" / "fixpoint_full.vais").resolve(),
}

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
        "enum declarations beyond payload-free tags or small Int-coded payload enums are not in the Vais native front subset yet",
        "use payload-free enum tags or Int/self-recursive payload enums with simple return-arm match; keep broader payload enums on the full compiler path.",
    ),
    (
        re.compile(r"\bmatch\b"),
        "`match` beyond simple enum return arms is not in the Vais native front subset yet",
        "use if/else for native sources, or keep payload match code on the full compiler path.",
    ),
    (
        re.compile(r"\bfor\b"),
        "`for` loops are not in the Vais native day-1 front subset yet",
        "use `while` with an explicit mutable index for now.",
    ),
    (
        re.compile(r"\bChar\b"),
        "Char is not in the verified native front subset yet",
        "use `Int`, `Str`, or `Bool` in this slice.",
    ),
    (
        re.compile(r"\bMap\s*<"),
        "Map<K,V> is specified but not verified in the Vais native front subset yet",
        "the first planned Map slice is `Map<Int,Int>` with `{}`, `insert`, `get(key, default)`, `contains`, and `len`; wait for that gate before using Map in public examples.",
    ),
    (
        re.compile(r"\b(Option|Result)\s*<|\blist\s*\("),
        "sum-result types and the `list()` constructor are not in the Vais native front subset yet",
        "use verified scalar/List syntax, or keep this source on the full compiler path until the native parity gate grows to it.",
    ),
    (
        re.compile(r"\b(break|continue)\b"),
        "loop control statements are not in the Vais native day-1 front subset yet",
        "rewrite the loop with an explicit condition variable for now.",
    ),
    (
        re.compile(r"\b(trait|impl)\b"),
        "traits and impl blocks are not in the Vais native day-1 front subset yet",
        "use plain functions for this slice.",
    ),
    (
        re.compile(r"\|[^|]*\|"),
        "closures beyond the single-Int closure-return slice are not in the Vais native front subset yet",
        "use a single Int capture returning `fn(Int) -> Int`, or write a named function for broader closure cases.",
    ),
    (
        re.compile(r"\.(?!(?:push|len|is_empty|last|pop|sum)\s*\()[A-Za-z_][A-Za-z0-9_]*\s*\("),
        "method calls beyond push/len/is_empty/last/pop/sum are not in the Vais native front subset yet",
        "use a plain function call, or keep this source on the full compiler path until that method is promoted.",
    ),
    (
        re.compile(r"\?"),
        "error propagation with `?` is not in the Vais native day-1 front subset yet",
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
        "the Vais list type is `List<T>`, not `Vec<T>`",
        "replace `Vec<T>` with `List<T>`.",
        lambda m, raw: raw[: m.start()] + "List" + raw[m.end() - 1 :],
    ),
    FrontRule(
        re.compile(r"\bHashMap\b"),
        "the Vais map spelling is `Map<K,V>`, not `HashMap<K,V>`",
        "use `Map<K,V>` only after the planned Map gate lands; the native front does not verify Map yet.",
        replace_once("HashMap", "Map"),
    ),
    FrontRule(
        re.compile(
            r"(?<![A-Za-z0-9_])(i8|i16|i32|i64|i128|u8|u16|u32|u64|u128|f32|f64|usize|isize)(?![A-Za-z0-9_])"
        ),
        "Vais scalar types are capitalized, not Rust scalar names",
        "use `Int`, `Int8..Int128`, `UInt8..UInt128`, `F32`, or `F64`.",
        fix_scalar_type,
    ),
    FrontRule(
        re.compile(r"\bString\b"),
        "the string type is `Str`, not `String`",
        "use `Str`.",
        replace_once("String", "Str"),
    ),
    FrontRule(
        re.compile(r"[+\-*/%]=(?!=)"),
        "compound assignment is not Vais syntax",
        "write it out, e.g. `x = x + 1`.",
    ),
]


FRONT_FN_HEADER = re.compile(
    r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*(?:->\s*([A-Za-z_][A-Za-z0-9_]*))?"
)
FRONT_SCALAR_PARAM = re.compile(r"[A-Za-z_][A-Za-z0-9_]*\s*:\s*(?:Int|Str|Bool)\Z")
FRONT_SCALAR_RETURNS = {"Int", "Str", "Bool"}


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

        if return_type not in FRONT_SCALAR_RETURNS:
            issues.append(
                FrontIssue(
                    line_no,
                    match.start() + 1,
                    "Vais native helper functions must return a verified scalar type",
                    "write helpers as `fn name(a: Int, ...) -> Int`, `-> Bool`, or `-> Str`.",
                )
            )

        if params and any(not FRONT_SCALAR_PARAM.fullmatch(part.strip()) for part in params.split(",")):
            issues.append(
                FrontIssue(
                    line_no,
                    match.start(2) + 1,
                    "Vais native helper parameters must use verified scalar types",
                    "use `Int`, `Str`, or `Bool` parameters in this slice.",
                )
            )

    return issues, has_main, has_bad_main


def check_front_contract(source: Path, display_source: Path | None = None) -> None:
    lines = source.read_text().splitlines()
    shown_source = display_source or source
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
        message = "Vais native day-1 front requires `fn main() -> Int` exactly"
        help_text = "write the entrypoint as `fn main() -> Int { ... }`."
        issues.insert(0, FrontIssue(1, 1, message, help_text))
    elif not has_main:
        message = "Vais native day-1 front requires `fn main() -> Int`"
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
                f"  --> {shown_source}:{issue.line}:{issue.col}",
                f"  {raw}",
                f"  {caret_at(issue.col)}",
                f"  help: {issue.help}",
            ]
        )
        if issue.fix:
            formatted.append(f"  fix: {issue.fix}")
        formatted.append("")
    raise FrontContractError("\n".join(formatted).rstrip())


SIMPLE_ENUM_DECL_RE = re.compile(r"^\s*enum\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{\s*([^{}()]*)\s*\}\s*$")
SIMPLE_MATCH_RE = re.compile(r"^(\s*)match\s+(.+?)\s*\{\s*$")
SIMPLE_MATCH_ARM_RE = re.compile(r"^(.+?)\s*=>\s*(.+?),?\s*$")
PAYLOAD_ENUM_DECL_RE = re.compile(r"^\s*enum\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{\s*(.+?)\s*\}\s*$")
PAYLOAD_ENUM_BASE = 1_000_000
IDENT_TEXT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


@dataclass(frozen=True)
class PayloadVariant:
    enum_name: str
    name: str
    tag: int
    tag_count: int
    fields: tuple[str, ...]


@dataclass(frozen=True)
class PayloadEnum:
    name: str
    variants: dict[str, PayloadVariant]


def split_top_level_commas(text: str) -> list[str] | None:
    parts: list[str] = []
    depth = 0
    start = 0
    string_delim: str | None = None
    escaped = False
    for idx, ch in enumerate(text):
        if escaped:
            escaped = False
            continue
        if string_delim == '"' and ch == "\\":
            escaped = True
            continue
        if ch in ('"', "`"):
            if string_delim is None:
                string_delim = ch
            elif string_delim == ch:
                string_delim = None
            continue
        if string_delim is not None:
            continue
        if ch in "([{":
            depth += 1
            continue
        if ch in ")]}":
            depth -= 1
            if depth < 0:
                return None
            continue
        if ch == "," and depth == 0:
            parts.append(text[start:idx].strip())
            start = idx + 1
    if depth != 0 or string_delim is not None:
        return None
    parts.append(text[start:].strip())
    return parts


def lower_simple_enum_match_text(text: str) -> str:
    """Lower payload-free enum/match sugar into the current Int-native subset."""
    lines = text.splitlines()
    enum_tags: dict[str, dict[str, int]] = {}
    kept: list[str] = []
    saw_enum = False

    for raw in lines:
        code = code_only(raw).strip()
        match = SIMPLE_ENUM_DECL_RE.match(code)
        if match:
            enum_name = match.group(1)
            variants = [part.strip() for part in match.group(2).split(",") if part.strip()]
            if not variants:
                return text
            tags: dict[str, int] = {}
            for idx, variant in enumerate(variants):
                if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", variant) is None:
                    return text
                tags[variant] = idx
            enum_tags[enum_name] = tags
            saw_enum = True
            continue
        if re.search(r"\benum\b", code):
            return text
        kept.append(raw)

    if not saw_enum:
        return text

    lowered: list[str] = []
    for raw in kept:
        line = raw
        for enum_name, tags in enum_tags.items():
            line = re.sub(rf":\s*{enum_name}\b", ": Int", line)
            line = re.sub(rf"->\s*{enum_name}\b", "-> Int", line)
            for variant, tag in tags.items():
                line = re.sub(rf"\b{enum_name}\s*\.\s*{variant}\b", str(tag), line)
        lowered.append(line)

    out: list[str] = []
    i = 0
    while i < len(lowered):
        raw = lowered[i]
        match = SIMPLE_MATCH_RE.match(code_only(raw))
        if match is None:
            out.append(raw)
            i += 1
            continue

        indent = match.group(1)
        expr = match.group(2).strip()
        arms: list[tuple[str, str]] = []
        i += 1
        while i < len(lowered):
            arm_code = code_only(lowered[i]).strip()
            if arm_code == "}":
                break
            arm_match = SIMPLE_MATCH_ARM_RE.match(arm_code)
            if arm_match is None:
                return text
            pattern = arm_match.group(1).strip()
            body = arm_match.group(2).strip().rstrip(",").rstrip()
            if re.fullmatch(r"-?\d+", pattern) is None or not body.startswith("return "):
                return text
            arms.append((pattern, body))
            i += 1
        if i >= len(lowered) or code_only(lowered[i]).strip() != "}":
            return text
        if not arms:
            return text

        for idx, (pattern, body) in enumerate(arms):
            keyword = "if" if idx == 0 else "else if"
            out.append(f"{indent}{keyword} {expr} == {pattern} {{")
            out.append(f"{indent}    {body}")
            out.append(f"{indent}}}")
        out.append(f"{indent}return 0")
        i += 1

    return "\n".join(out) + ("\n" if text.endswith("\n") else "")


def parse_payload_enum_decl(code: str) -> PayloadEnum | None:
    match = PAYLOAD_ENUM_DECL_RE.match(code)
    if match is None:
        return None
    enum_name = match.group(1)
    specs = split_top_level_commas(match.group(2))
    if not specs:
        return None

    parsed: list[tuple[str, tuple[str, ...]]] = []
    saw_payload = False
    for spec in specs:
        item = re.fullmatch(r"([A-Za-z_][A-Za-z0-9_]*)(?:\((.*)\))?", spec)
        if item is None:
            return None
        fields_text = item.group(2)
        fields: tuple[str, ...] = ()
        if fields_text is not None:
            saw_payload = True
            field_parts = split_top_level_commas(fields_text)
            if field_parts is None:
                return None
            fields = tuple(part.strip() for part in field_parts if part.strip())
            if any(field not in ("Int", enum_name) for field in fields):
                return None
        parsed.append((item.group(1), fields))

    if not saw_payload:
        return None

    tag_count = len(parsed)
    variants: dict[str, PayloadVariant] = {}
    for tag, (name, fields) in enumerate(parsed):
        if name in variants:
            return None
        variants[name] = PayloadVariant(enum_name, name, tag, tag_count, fields)
    return PayloadEnum(enum_name, variants)


def payload_variant_lookup(enums: dict[str, PayloadEnum]) -> dict[str, PayloadVariant] | None:
    lookup: dict[str, PayloadVariant] = {}
    for enum in enums.values():
        for name, variant in enum.variants.items():
            if name in lookup:
                return None
            lookup[name] = variant
    return lookup


def replace_payload_enum_types(line: str, enums: dict[str, PayloadEnum]) -> str:
    out = line
    for enum_name in enums:
        out = re.sub(rf":\s*{enum_name}\b", ": Int", out)
        out = re.sub(rf"->\s*{enum_name}\b", "-> Int", out)
    return out


def find_matching_paren_mask(mask: str, open_index: int) -> int:
    depth = 0
    for idx in range(open_index, len(mask)):
        ch = mask[idx]
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return idx
    return -1


def encode_payload_variant_call(
    variant: PayloadVariant,
    args: list[str],
    lookup: dict[str, PayloadVariant],
    enums: dict[str, PayloadEnum],
) -> str | None:
    if len(args) != len(variant.fields):
        return None
    terms: list[str] = []
    for idx, arg in enumerate(args):
        lowered = rewrite_payload_constructor_calls(arg, lookup, enums)
        if lowered is None:
            return None
        factor = PAYLOAD_ENUM_BASE**idx
        if factor == 1:
            terms.append(f"({lowered.strip()})")
        else:
            terms.append(f"{factor} * ({lowered.strip()})")
    payload = " + ".join(terms) if terms else "0"
    return f"({variant.tag} + {variant.tag_count} * ({payload}))"


def rewrite_payload_constructor_calls(
    text: str,
    lookup: dict[str, PayloadVariant],
    enums: dict[str, PayloadEnum],
) -> str | None:
    mask = code_only(text)
    out: list[str] = []
    i = 0
    while i < len(text):
        if not (mask[i].isalpha() or mask[i] == "_"):
            out.append(text[i])
            i += 1
            continue

        j = i + 1
        while j < len(mask) and (mask[j].isalnum() or mask[j] == "_"):
            j += 1
        first = mask[i:j]
        cursor = j
        while cursor < len(mask) and mask[cursor].isspace():
            cursor += 1

        variant: PayloadVariant | None = None
        replace_end = j
        if first in enums and cursor < len(mask) and mask[cursor] == ".":
            vstart = cursor + 1
            while vstart < len(mask) and mask[vstart].isspace():
                vstart += 1
            vend = vstart
            while vend < len(mask) and (mask[vend].isalnum() or mask[vend] == "_"):
                vend += 1
            variant = enums[first].variants.get(mask[vstart:vend])
            cursor = vend
            replace_end = vend
            while cursor < len(mask) and mask[cursor].isspace():
                cursor += 1
        else:
            variant = lookup.get(first)

        if variant is None:
            out.append(text[i:j])
            i = j
            continue

        if variant.fields:
            if cursor >= len(mask) or mask[cursor] != "(":
                out.append(text[i:j])
                i = j
                continue
            close = find_matching_paren_mask(mask, cursor)
            if close < 0:
                return None
            args = split_top_level_commas(text[cursor + 1 : close])
            if args is None:
                return None
            encoded = encode_payload_variant_call(variant, args, lookup, enums)
            if encoded is None:
                return None
            out.append(encoded)
            i = close + 1
            continue

        out.append(str(variant.tag))
        i = replace_end

    return "".join(out)


def parse_payload_pattern(
    pattern: str,
    lookup: dict[str, PayloadVariant],
    enums: dict[str, PayloadEnum],
) -> tuple[PayloadVariant, list[str]] | None:
    match = re.fullmatch(r"(?:(?P<enum>[A-Za-z_][A-Za-z0-9_]*)\s*\.\s*)?(?P<variant>[A-Za-z_][A-Za-z0-9_]*)(?:\((?P<fields>.*)\))?", pattern)
    if match is None:
        return None
    enum_name = match.group("enum")
    variant_name = match.group("variant")
    if enum_name:
        enum = enums.get(enum_name)
        if enum is None:
            return None
        variant = enum.variants.get(variant_name)
    else:
        variant = lookup.get(variant_name)
    if variant is None:
        return None
    fields_text = match.group("fields")
    binders = [] if fields_text is None else split_top_level_commas(fields_text)
    if binders is None or len(binders) != len(variant.fields):
        return None
    if any(IDENT_TEXT_RE.fullmatch(binder) is None for binder in binders):
        return None
    return variant, binders


def lower_payload_enum_match_text(text: str) -> str:
    """Lower small Int/self-recursive payload enums into Int tag/payload codes."""
    lines = text.splitlines()
    enums: dict[str, PayloadEnum] = {}
    kept: list[str] = []
    saw_payload_enum = False

    for raw in lines:
        code = code_only(raw).strip()
        enum_def = parse_payload_enum_decl(code)
        if enum_def is not None:
            enums[enum_def.name] = enum_def
            saw_payload_enum = True
            continue
        if re.search(r"\benum\b", code):
            return text
        kept.append(raw)

    if not saw_payload_enum:
        return text

    lookup = payload_variant_lookup(enums)
    if lookup is None:
        return text

    typed_lines = [replace_payload_enum_types(raw, enums) for raw in kept]
    out: list[str] = []
    i = 0
    while i < len(typed_lines):
        raw = typed_lines[i]
        match = SIMPLE_MATCH_RE.match(code_only(raw))
        if match is None:
            out.append(raw)
            i += 1
            continue

        indent = match.group(1)
        expr = match.group(2).strip()
        arms: list[tuple[PayloadVariant, list[str], str]] = []
        i += 1
        while i < len(typed_lines):
            arm_code = code_only(typed_lines[i]).strip()
            if arm_code == "}":
                break
            arm_match = SIMPLE_MATCH_ARM_RE.match(arm_code)
            if arm_match is None:
                return text
            parsed = parse_payload_pattern(arm_match.group(1).strip(), lookup, enums)
            if parsed is None:
                return text
            body = arm_match.group(2).strip().rstrip(",").rstrip()
            if not body.startswith("return "):
                return text
            body_expr = rewrite_payload_constructor_calls(body[len("return ") :], lookup, enums)
            if body_expr is None:
                return text
            arms.append((parsed[0], parsed[1], body_expr))
            i += 1
        if i >= len(typed_lines) or code_only(typed_lines[i]).strip() != "}":
            return text
        if not arms:
            return text

        for idx, (variant, binders, body_expr) in enumerate(arms):
            keyword = "if" if idx == 0 else "else if"
            out.append(f"{indent}{keyword} {expr} % {variant.tag_count} == {variant.tag} {{")
            for field_idx, binder in enumerate(binders):
                denom = variant.tag_count * (PAYLOAD_ENUM_BASE**field_idx)
                out.append(f"{indent}    let {binder} = ({expr} / {denom}) % {PAYLOAD_ENUM_BASE}")
            out.append(f"{indent}    return {body_expr}")
            out.append(f"{indent}}}")
        out.append(f"{indent}return 0")
        i += 1

    rewritten: list[str] = []
    for raw in out:
        lowered = rewrite_payload_constructor_calls(raw, lookup, enums)
        if lowered is None:
            return text
        rewritten.append(lowered)
    return "\n".join(rewritten) + ("\n" if text.endswith("\n") else "")


CLOSURE_RETURN_FN_RE = re.compile(
    r"^(\s*)fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*Int\s*\)\s*->\s*fn\s*\(\s*Int\s*\)\s*->\s*Int\s*\{\s*$"
)
CLOSURE_RETURN_LINE_RE = re.compile(r"^\s*return\s*\|\s*([A-Za-z_][A-Za-z0-9_]*)\s*\|\s*(.+?)\s*$")


def replace_word(expr: str, name: str, replacement: str) -> str:
    return re.sub(rf"\b{re.escape(name)}\b", replacement, expr)


def rewrite_closure_var_calls(text: str, closure_vars: dict[str, str]) -> str | None:
    mask = code_only(text)
    out: list[str] = []
    i = 0
    while i < len(text):
        if not (mask[i].isalpha() or mask[i] == "_"):
            out.append(text[i])
            i += 1
            continue

        j = i + 1
        while j < len(mask) and (mask[j].isalnum() or mask[j] == "_"):
            j += 1
        name = mask[i:j]
        apply_name = closure_vars.get(name)
        if apply_name is None:
            out.append(text[i:j])
            i = j
            continue

        cursor = j
        while cursor < len(mask) and mask[cursor].isspace():
            cursor += 1
        if cursor >= len(mask) or mask[cursor] != "(":
            out.append(text[i:j])
            i = j
            continue
        close = find_matching_paren_mask(mask, cursor)
        if close < 0:
            return None
        args = split_top_level_commas(text[cursor + 1 : close])
        if args is None or len(args) != 1:
            return None
        out.append(f"{apply_name}({name}, {args[0].strip()})")
        i = close + 1
    return "".join(out)


def lower_simple_closure_return_text(text: str) -> str:
    """Closure-convert `fn make(n)->fn(Int)->Int { return |x| ... }` to Int env."""
    lines = text.splitlines()
    lowered: list[str] = []
    makers: dict[str, str] = {}
    changed = False
    i = 0
    while i < len(lines):
        raw = lines[i]
        match = CLOSURE_RETURN_FN_RE.match(code_only(raw))
        if match is None:
            lowered.append(raw)
            i += 1
            continue

        if i + 2 >= len(lines):
            return text
        ret_match = CLOSURE_RETURN_LINE_RE.match(code_only(lines[i + 1]).strip())
        if ret_match is None or code_only(lines[i + 2]).strip() != "}":
            return text

        indent = match.group(1)
        maker_name = match.group(2)
        capture_name = match.group(3)
        closure_arg = ret_match.group(1)
        body_expr = ret_match.group(2).strip().rstrip(",").rstrip()
        if IDENT_TEXT_RE.fullmatch(closure_arg) is None:
            return text
        apply_name = f"{maker_name}__apply"
        apply_expr = replace_word(body_expr, capture_name, "env")

        lowered.append(f"{indent}fn {maker_name}({capture_name}: Int) -> Int {{")
        lowered.append(f"{indent}    return {capture_name}")
        lowered.append(f"{indent}}}")
        lowered.append("")
        lowered.append(f"{indent}fn {apply_name}(env: Int, {closure_arg}: Int) -> Int {{")
        lowered.append(f"{indent}    return {apply_expr}")
        lowered.append(f"{indent}}}")
        makers[maker_name] = apply_name
        changed = True
        i += 3

    if not changed:
        return text

    rewritten: list[str] = []
    closure_vars: dict[str, str] = {}
    brace_depth = 0
    for raw in lowered:
        code = code_only(raw)
        if re.match(r"^\s*fn\b", code):
            closure_vars = {}

        let_match = re.match(r"^\s*let\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(", code)
        if let_match and let_match.group(2) in makers:
            closure_vars[let_match.group(1)] = makers[let_match.group(2)]

        line = rewrite_closure_var_calls(raw, closure_vars)
        if line is None:
            return text
        rewritten.append(line)

        brace_depth += code.count("{") - code.count("}")
        if brace_depth <= 0:
            closure_vars = {}
            brace_depth = 0

    return "\n".join(rewritten) + ("\n" if text.endswith("\n") else "")


def strip_vais_line_comment(line: str) -> str:
    string_delim: str | None = None
    escaped = False
    for idx, ch in enumerate(line):
        if escaped:
            escaped = False
            continue
        if string_delim == '"' and ch == "\\":
            escaped = True
            continue
        if ch in ('"', "`"):
            if string_delim is None:
                string_delim = ch
            elif string_delim == ch:
                string_delim = None
            continue
        if string_delim is None and ch == "#":
            return line[:idx].rstrip()
    return line.rstrip()


def lower_struct_field_types_text(text: str) -> str:
    def lower_one_line(match: re.Match[str]) -> str:
        prefix, body, suffix = match.group(1), match.group(2), match.group(3)
        parts = split_top_level_commas(body)
        if parts is None:
            return match.group(0)
        lowered: list[str] = []
        changed = False
        for part in parts:
            if not part:
                continue
            next_part = re.sub(r"^([A-Za-z_][A-Za-z0-9_]*)\s*:\s*[^,{}]+$", r"\1", part.strip())
            changed = changed or next_part != part.strip()
            lowered.append(next_part)
        if not changed:
            return match.group(0)
        return f"{prefix}{', '.join(lowered)}{suffix}"

    text = re.sub(r"(\bstruct\s+[A-Za-z_][A-Za-z0-9_]*\s*\{)([^{}]*)(\})", lower_one_line, text)
    lines = text.splitlines()
    out: list[str] = []
    in_struct = False
    depth = 0
    for raw in lines:
        stripped = raw.strip()
        if not in_struct and re.match(r"^struct\s+[A-Za-z_][A-Za-z0-9_]*\b", stripped) and "{" in stripped and "}" not in stripped:
            in_struct = True
            depth = stripped.count("{") - stripped.count("}")
            out.append(raw)
            continue
        if in_struct:
            match = re.match(r"^(\s*)([A-Za-z_][A-Za-z0-9_]*)\s*:\s*[^,{}]+(,?)\s*$", raw)
            if match:
                comma = match.group(3) or ","
                out.append(f"{match.group(1)}{match.group(2)}{comma}")
            else:
                out.append(raw)
            depth += stripped.count("{") - stripped.count("}")
            if depth <= 0:
                in_struct = False
                depth = 0
            continue
        out.append(raw)
    return "\n".join(out) + ("\n" if text.endswith("\n") else "")


def statement_needs_semicolon(stripped: str) -> bool:
    if not stripped or stripped.endswith(";"):
        return False
    if stripped.startswith(("fn ", "struct ", "enum ", "if ", "else", "while ", "for ")):
        return False
    if stripped in {"{", "}"}:
        return False
    if stripped.endswith("{") or stripped.endswith(","):
        return False
    if stripped.startswith(("let ", "return ", "print(", "putchar(", "puts(")):
        return True
    if re.match(r"^[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*\s*=", stripped):
        return True
    if re.match(r"^[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*\s*\(", stripped):
        return True
    if re.match(r"^[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)+\s*\(", stripped):
        return True
    return False


def add_statement_semicolons_text(text: str) -> str:
    out: list[str] = []
    for raw in text.splitlines():
        stripped = raw.strip()
        if statement_needs_semicolon(stripped):
            out.append(f"{raw};")
        else:
            out.append(raw)
    return "\n".join(out) + ("\n" if text.endswith("\n") else "")


def normalize_full_source_text(text: str) -> str:
    no_comments = "\n".join(strip_vais_line_comment(line) for line in text.splitlines())
    if text.endswith("\n"):
        no_comments += "\n"
    lowered_structs = lower_struct_field_types_text(no_comments)
    return add_statement_semicolons_text(lowered_structs)


def lower_front_source_text(raw: str) -> str:
    lowered = normalize_full_source_text(raw)
    for lowerer in (
        lower_simple_enum_match_text,
        lower_payload_enum_match_text,
        lower_simple_closure_return_text,
    ):
        new_lowered = lowerer(lowered)
        if new_lowered != lowered:
            lowered = new_lowered
    return add_statement_semicolons_text(lowered)


def lower_int_annotations_for_core_text(text: str) -> str:
    out: list[str] = []
    for raw in text.splitlines():
        line = raw
        if re.match(r"^\s*fn\b", code_only(line)):
            def lower_params(match: re.Match[str]) -> str:
                params = re.sub(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*Int\b", r"\1", match.group(2))
                return f"{match.group(1)}{params}{match.group(3)}"

            line = re.sub(r"^(\s*fn\s+[A-Za-z_][A-Za-z0-9_]*\s*\()([^)]*)(\).*)$", lower_params, line)
            line = re.sub(r"\)\s*->\s*Int\b", ")", line)
        line = re.sub(r"^(\s*let\s+(?:mut\s+)?[A-Za-z_][A-Za-z0-9_]*)\s*:\s*Int\s*=", r"\1 =", line)
        out.append(line)
    return "\n".join(out) + ("\n" if text.endswith("\n") else "")


def lower_print_for_core_text(text: str) -> str:
    out: list[str] = []
    for raw in text.splitlines():
        mask = code_only(raw)
        line = raw
        for match in reversed(list(re.finditer(r"\bprint\s*\(", mask))):
            line = line[: match.start()] + "puts(" + line[match.end() :]
        out.append(line)
    return "\n".join(out) + ("\n" if text.endswith("\n") else "")


def prepare_front_source(source: Path, tmp: Path) -> Path:
    raw = source.read_text()
    lowered = lower_front_source_text(raw)
    if lowered == raw:
        return source
    lowered_path = tmp / "front_lowered.vais"
    lowered_path.write_text(lowered)
    return lowered_path


def prepare_full_source(front_source: Path, tmp: Path) -> Path:
    raw = front_source.read_text()
    lowered = lower_print_for_core_text(lower_int_annotations_for_core_text(raw))
    if lowered == raw:
        return front_source
    lowered_path = tmp / "core_lowered.vais"
    lowered_path.write_text(lowered)
    return lowered_path


def is_self_host_tier_source(source: Path) -> bool:
    try:
        resolved = source.resolve()
    except OSError:
        return False
    if resolved in SELF_HOST_TIER_SOURCES:
        return True
    if source.suffix != ".vais":
        return False
    trust_roots = os.environ.get("VAISC_SELF_HOST_TRUST_ROOTS", "")
    for raw_root in trust_roots.split(os.pathsep):
        if not raw_root:
            continue
        root = Path(raw_root).expanduser()
        try:
            trusted = (root / "compiler" / "self" / source.name).resolve()
        except OSError:
            continue
        if resolved == trusted:
            return True
    return False


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
                "use integer literals and arithmetic here, or use the full compiler engine.",
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
            "use the full compiler engine for helper functions until the direct emitter grows calls.",
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
    validate_vais_source(source)
    if args.engine == "direct":
        return direct_emit_ir(source, ir_out)
    return full_emit_ir(source, ir_out, args)


def validate_vais_source(source: Path) -> None:
    if source.suffix != ".vais":
        raise CompileError(f"Vais source files must use the .vais extension: {source}")


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


def write_core_runner(path: Path) -> None:
    path.write_text(
        r"""
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern int64_t compile(char *src);

static char *read_source(const char *path) {
    FILE *fp = fopen(path, "rb");
    if (fp == NULL) {
        fprintf(stderr, "vaisc: cannot open %s: %s\n", path, strerror(errno));
        return NULL;
    }
    if (fseek(fp, 0, SEEK_END) != 0) {
        fprintf(stderr, "vaisc: cannot seek %s\n", path);
        fclose(fp);
        return NULL;
    }
    long size = ftell(fp);
    if (size < 0) {
        fprintf(stderr, "vaisc: cannot size %s\n", path);
        fclose(fp);
        return NULL;
    }
    if (fseek(fp, 0, SEEK_SET) != 0) {
        fprintf(stderr, "vaisc: cannot rewind %s\n", path);
        fclose(fp);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)size + 1);
    if (buf == NULL) {
        fprintf(stderr, "vaisc: out of memory reading %s\n", path);
        fclose(fp);
        return NULL;
    }
    size_t nread = fread(buf, 1, (size_t)size, fp);
    if (nread != (size_t)size) {
        fprintf(stderr, "vaisc: short read for %s\n", path);
        free(buf);
        fclose(fp);
        return NULL;
    }
    buf[size] = '\0';
    fclose(fp);
    return buf;
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: vaisc-core-runner input.vais\n");
        return 2;
    }
    char *src = read_source(argv[1]);
    if (src == NULL) {
        return 1;
    }
    int64_t rc = compile(src);
    free(src);
    return (int)rc;
}
""".lstrip()
    )


def write_reusable_core(path: Path) -> None:
    if not CORE_LL.is_file():
        raise CompileError(f"missing Vais compiler core: {CORE_LL}")
    text = CORE_LL.read_text()
    main_def = re.compile(r"^define i64 @main\(\) \{$", re.MULTILINE)
    count = len(main_def.findall(text))
    if count != 1:
        raise CompileError(f"invalid Vais compiler core: expected one @main, found {count}")
    path.write_text(main_def.sub("define i64 @vais_selftest_main() {", text, count=1))


def tmpdir_from_args(args: argparse.Namespace) -> tuple[Path, tempfile.TemporaryDirectory[str] | None]:
    if args.keep_tmp:
        path = Path(tempfile.mkdtemp(prefix="vaisc-"))
        print(f"debug: keeping temp dir {path}", file=sys.stderr)
        return path, None
    holder = tempfile.TemporaryDirectory(prefix="vaisc-")
    return Path(holder.name), holder


def full_emit_ir(source: Path, ir_out: Path | None, args: argparse.Namespace) -> str | None:
    if not source.is_file():
        raise CompileError(f"source not found: {source}")

    tmp, holder = tmpdir_from_args(args)
    try:
        trusted_self_host = is_self_host_tier_source(source)
        front_source = prepare_front_source(source, tmp)
        if not trusted_self_host:
            check_front_contract(front_source, source)
        native_source = prepare_full_source(front_source, tmp)
        core_ll = tmp / "vaisc_core.ll"
        runner_c = tmp / "vaisc_core_runner.c"
        stage0 = tmp / "stage0-vaisc"
        build_log = tmp / "core-build.log"
        clang = getattr(args, "clang", os.environ.get("CLANG", "clang"))

        write_reusable_core(core_ll)
        write_core_runner(runner_c)
        with build_log.open("w") as log:
            run_checked(
                [clang, "-Wno-override-module", "-o", str(stage0), str(core_ll), str(runner_c)],
                stdout=log,
                stderr=subprocess.STDOUT,
            )

        if ir_out is None:
            proc = subprocess.run([str(stage0), str(native_source)], text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            if proc.returncode != 0:
                raise CompileError(f"generated Vais compiler exited {proc.returncode}: {proc.stderr}")
            return proc.stdout

        ir_out.parent.mkdir(parents=True, exist_ok=True)
        with ir_out.open("w") as out:
            proc = subprocess.run([str(stage0), str(native_source)], text=True, stdout=out, stderr=subprocess.PIPE)
        if proc.returncode != 0:
            raise CompileError(f"generated Vais compiler exited {proc.returncode}: {proc.stderr}")
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
        choices=("full", "direct"),
        default=os.environ.get("VAISC_ENGINE", "full"),
        help=(
            "Compiler engine. `full` uses the self-host compiler; "
            "`direct` uses the minimal LLVM emitter."
        ),
    )
    parser.add_argument(
        "--keep-tmp",
        action="store_true",
        help="Keep the temporary compiler directory for debugging.",
    )


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="vaisc",
        description="Vais compiler CLI. Accepts .vais source files.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    emit = sub.add_parser("emit-ir", help="Compile Vais source to LLVM IR.")
    emit.add_argument("source")
    emit.add_argument("-o", "--output", default="-", help="LLVM IR output path, or '-' for stdout.")
    add_common_flags(emit)
    emit.set_defaults(func=emit_ir)

    bld = sub.add_parser("build", help="Compile Vais source to a native binary.")
    bld.add_argument("source")
    bld.add_argument("-o", "--output", required=True, help="Native binary output path.")
    bld.add_argument("--ir-out", help="Optional path to also keep emitted LLVM IR.")
    bld.add_argument("--clang", default=os.environ.get("CLANG", "clang"))
    add_common_flags(bld)
    bld.set_defaults(func=build)

    run = sub.add_parser("run", help="Compile and run Vais source, returning the program exit code.")
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
