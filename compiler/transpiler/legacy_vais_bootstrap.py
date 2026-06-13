#!/usr/bin/env python3
"""
New Vais -> Legacy Vais bootstrap adapter (prototype v0.1).

Strategy (per NEW-LANGUAGE-README): reuse the Legacy Vais backend while the New
Vais self-host compiler matures. New Vais' v0.2 surface syntax maps almost 1:1
to Legacy Vais, so this adapter is the cheapest path to a working prototype
without hand-maintaining a second native backend. Unsupported constructs are
reported, not silently mangled (honesty over coverage).

Usage:  python3 legacy_vais_bootstrap.py input.vais > output.vais
        (then: vaisc build output.vais -o out && ./out)

NOT a real lexer/parser yet — it is a line/token rewriter. The point is to prove
the "New Vais surface -> Legacy Vais backend" pipeline runs end-to-end and
computes correct values while the self-host compiler closes the remaining path
to a native New Vais compiler.
"""
import re
import sys

# --- Type name map (nl -> Vais) ---
TYPE_MAP = {
    "Int": "i64", "Int8": "i8", "Int16": "i16", "Int32": "i32", "Int64": "i64",
    "Int128": "i128",
    "UInt8": "u8", "UInt16": "u16", "UInt32": "u32", "UInt64": "u64", "UInt128": "u128",
    "F32": "f32", "F64": "f64", "Bool": "bool", "Str": "str", "Char": "char",
    # `String` is a common Rust-habit spelling of the string type; map it to the
    # working Vais `str` so it doesn't silently mismatch a `str` literal (E001).
    # (vais-check still flags `String` -> the canonical New Vais name is `Str`.)
    "String": "str",
    # Collection type names: nl List/Map -> Vais Vec/HashMap.
    "List": "Vec", "Map": "HashMap",
}

# --- Keyword/operator token map (whole-word) ---
WORD_MAP = {
    "and": "&&",
    "or": "||",
    "not": "!",
}


# Scalar conversions: nl writes `Int(x)` / `F64(x)` / `Str(x)` (P4: conversion is
# a call, not `x as Int`). Numeric conversions map to Vais casts; string
# conversion maps to the Vais surface builtin `to_string(x)`. Do this BEFORE
# map_types (which would otherwise produce invalid `i64(x)` / `str(x)` calls).
# Bool/Char are not handled here, and Some/Ok/struct constructors stay intact.
_CONV_TYPES = {
    "Int": "i64", "Int8": "i8", "Int16": "i16", "Int32": "i32", "Int64": "i64",
    "Int128": "i128",
    "UInt8": "u8", "UInt16": "u16", "UInt32": "u32", "UInt64": "u64", "UInt128": "u128",
    "F32": "f32", "F64": "f64",
}


def map_conversions(line: str) -> str:
    def rewrite(p: str) -> str:
        p = re.sub(
            r"\bStr\(([^()]*)\)",
            lambda m: f"to_string({m.group(1).strip()})",
            p,
        )
        for nm, vt in _CONV_TYPES.items():
            # NumType(<expr without nested parens>) -> (<expr> as vais_type)
            p = re.sub(
                r"\b" + nm + r"\(([^()]*)\)",
                lambda m, _vt=vt: f"({m.group(1).strip()} as {_vt})",
                p,
            )
        return p
    return outside_strings(line, rewrite)


def map_types(line: str) -> str:
    # Replace nl type names with Vais types, whole-word only. NOT inside strings
    # (code-as-data: a string containing "Int"/"List" must stay verbatim).
    def repl(m):
        return TYPE_MAP.get(m.group(0), m.group(0))
    return outside_strings(line, lambda p: re.sub(r"\b[A-Za-z_][A-Za-z0-9_]*\b", repl, p))


def outside_strings(line: str, fn) -> str:
    """Apply `fn` to the parts of `line` that are NOT inside double-quoted string
    literals. String contents are left verbatim. This protects code-as-data: an
    nl source string like run_program("return if x ...") must keep `if` intact
    (the keyword-rewrite would otherwise corrupt the embedded program text)."""
    parts = re.split(r'("(?:[^"\\]|\\.)*")', line)
    out = []
    for i, p in enumerate(parts):
        if i % 2 == 1:  # string literal — leave as-is
            out.append(p)
        else:
            out.append(fn(p))
    return "".join(out)


def map_words(line: str) -> str:
    # Logic words -> operators. Whole-word, but NOT inside strings.
    def rewrite(p: str) -> str:
        for k, v in WORD_MAP.items():
            p = re.sub(rf"\b{k}\b", v, p)
        return p
    return outside_strings(line, rewrite)


def _vec_elem_from_annotation(ty):
    """Extract the element type from a `: List<T>` / `: Vec<T>` annotation,
    preserving NESTING: `List<List<Int>>` -> Vec element `Vec<Int>`. Returns the
    nl element-type string (map_types later maps Int->i64 etc.), or None."""
    if not ty:
        return None
    m = re.search(r"(?:List|Vec)\s*<(.*)>\s*$", ty.strip())
    if not m:
        return None
    inner = m.group(1).strip()
    # if the element is itself a List/Vec, recurse so nesting is preserved
    if re.match(r"^(?:List|Vec)\s*<", inner):
        sub = _vec_elem_from_annotation(inner)
        return f"Vec<{sub}>" if sub else inner
    return inner


def _split_top_commas(s):
    """Split a comma list at bracket-depth 0 (so `[1,2],[3,4]` -> two items)."""
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch == "[":
            depth += 1
        elif ch == "]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur)
    return out


def _infer_vec_elem(rhs_inner):
    """Infer the Vec element type from a non-empty list literal's contents
    (the text between the outer brackets). Handles NESTED lists recursively:
    `[1,2],[3,4]` -> `Vec<Int>`; `1.0,2.0` -> `F64`; `1,2,3` -> `Int`."""
    first = _split_top_commas(rhs_inner)[0].strip()
    if first.startswith("["):
        sub = _infer_vec_elem(first[1:-1])
        return f"Vec<{sub}>"
    return "F64" if re.match(r"^-?\d+\.\d+", first) else "Int"


def _list_binding(indent, name, ty, rhs):
    """Render a Vais binding when the RHS is a list literal `[..]` (possibly with
    a `: List<T>`/`: Vec<T>` annotation). Returns the Vais line, or None if rhs
    is not a list literal. Rules (verified against Vais):
      - empty `[]` -> `name: Vec<T> = Vec::new()` (literal `[]` CRASHES for
        Vec<struct>; Vec::new() is safe). Element type from annotation (required).
      - non-empty `[1,2,3]` -> `name: Vec<T> = [1,2,3]` (literal works).
        T from annotation if present, else inferred from first element.
      - NESTED `[[1,2],[3,4]]` -> `name: Vec<Vec<Int>> = ...` (element type is
        inferred/annotation-derived recursively so the nesting is preserved).
    """
    r = rhs.rstrip()
    if not (r.startswith("[") and r.endswith("]")):
        return None
    # element type: prefer annotation `: List<T>` / `: Vec<T>` (nesting-aware)
    elem = _vec_elem_from_annotation(ty)
    if r == "[]":
        if elem is None:
            elem = "Int"  # last resort; better: require annotation
        return f"{indent}{name}: Vec<{elem}> = Vec::new()"
    if elem is None:
        elem = _infer_vec_elem(r[1:-1])
    return f"{indent}{name}: Vec<{elem}> = {rhs}"


def map_let(line: str) -> str:
    # `let (a, b) = expr` -> `(a, b) := expr`  (tuple destructuring -- Vais binds
    # tuple patterns with `:=` and no `let`).
    mt = re.match(r"^(\s*)let\s+(\([^)]*\))\s*=\s*(.*)$", line)
    if mt:
        indent, pat, rhs = mt.groups()
        return f"{indent}{pat} := {rhs}"
    # `let mut x = expr`  -> `x := mut expr`   (list literals: typed Vec binding)
    # `let x = expr`      -> `x := expr`
    m = re.match(r"^(\s*)let\s+mut\s+([A-Za-z_][A-Za-z0-9_]*)\s*(:\s*[^=]+?)?\s*=\s*(.*)$", line)
    if m:
        indent, name, ty, rhs = m.groups()
        lb = _list_binding(indent, name, ty, rhs)
        if lb is not None:
            # Vec is push-mutable in Vais without `mut` on the binding, so a
            # typed Vec binding needs no `mut` keyword (mut on a typed collection
            # binding isn't Vais syntax). The push works regardless.
            return lb
        return f"{indent}{name} := mut {rhs}"
    m = re.match(r"^(\s*)let\s+([A-Za-z_][A-Za-z0-9_]*)\s*(:\s*[^=]+?)?\s*=\s*(.*)$", line)
    if m:
        indent, name, ty, rhs = m.groups()
        lb = _list_binding(indent, name, ty, rhs)
        if lb is not None:
            return lb
        return f"{indent}{name} := {rhs}"
    return line


def map_if(line: str) -> str:
    # `if cond {`        -> `I cond {`
    # `} else if cond {` -> `} else I cond {`
    # `} else {`         -> stays
    # NOT inside string literals (embedded program text must keep `if` intact).
    def rewrite(p: str) -> str:
        p = re.sub(r"\belse\s+if\b", "else I", p)
        p = re.sub(r"(^|\}\s*|\s)if\b", lambda m: m.group(0).replace("if", "I"), p)
        return p
    return outside_strings(line, rewrite)


def map_loop_keywords(line: str) -> str:
    # Loop control: nl `break`/`continue` -> Vais `B`/`C` (verified to work as
    # user statements inside a loop body). Whole-word, NOT inside strings.
    def rewrite(p: str) -> str:
        p = re.sub(r"\bbreak\b", "B", p)
        p = re.sub(r"\bcontinue\b", "C", p)
        return p
    return outside_strings(line, rewrite)


def map_bitnot(line: str) -> str:
    # `bitnot(x)` -> `(~x)`  (only the simple single-arg form). NOT inside strings
    # (code-as-data: a string containing "bitnot(x)" must stay verbatim).
    return outside_strings(line, lambda p: re.sub(r"\bbitnot\(([^()]*)\)", r"(~\1)", p))


# Binary bitwise prelude functions -> Vais operators (all verified to work).
# `bitand(a,b)` -> `(a & b)`, etc. Only the simple two-arg form (args without
# nested parens); complex args should bind to a variable first.
_BITWISE2 = {
    "bitand": "&",
    "bitor": "|",
    "bitxor": "^",
    "shl": "<<",
    "shr": ">>",
}


def map_bitwise2(line: str) -> str:
    def rewrite(p: str) -> str:
        for fn, op in _BITWISE2.items():
            p = re.sub(
                r"\b" + fn + r"\(([^(),]*),([^(),]*)\)",
                lambda m, _op=op: f"({m.group(1).strip()} {_op} {m.group(2).strip()})",
                p,
            )
        return p
    return outside_strings(line, rewrite)


def map_brace_escapes(line: str) -> str:
    # In nl strings, `{{`/`}}` mean a literal brace (standard escape); `{expr}` is
    # interpolation. Vais ALSO interpolates `{...}` in EVERY string literal (not
    # just printed ones) — so a `{ }` pair in any passed string would be parsed as
    # interpolation and fail. Vais's escape for a literal brace is `\{` / `\}`
    # (works in print AND in passed strings, e.g. embedding code-as-data like
    # `fn f(x) \{ return x \}`). So translate nl `{{`->`\{` and `}}`->`\}` inside
    # string literals only. Real interpolations `{expr}` are left untouched.
    parts = re.split(r'("(?:[^"\\]|\\.)*")', line)
    out = []
    for i, p in enumerate(parts):
        if i % 2 == 1:  # string literal
            p = p.replace("{{", "\\{").replace("}}", "\\}")
        out.append(p)
    return "".join(out)


def map_print(line: str) -> str:
    # nl `print(EXPR)` -> Vais `puts(EXPR)`. Vais's `puts` writes a line to
    # stdout and supports string interpolation ("{x}"), so nl's print maps
    # directly. (nl prelude exposes `print`; backend = Vais puts builtin.)
    return re.sub(r"\bprint\(", "puts(", line)


def map_collection_methods(line: str) -> str:
    # nl collection ops are methods. Vais std/vec has map/filter/fold but no
    # `sum`. Express sum via fold (verified: fold(0, |a,x| a+x) works).
    # All rewrites are outside strings (a string containing ".sum()" stays as-is).
    def rewrite(p: str) -> str:
        # `.sum()` -> `.fold(0, |__a, __x| __a + __x)`
        p = re.sub(r"\.sum\(\)", r".fold(0, |__a, __x| __a + __x)", p)
        # Vais `filter` wants `fn(T)->i64`, but nl predicates are Bool. Wrap a
        # boolean filter body into the stdlib's i64 predicate ABI:
        # `.filter(|x| COND)` -> `.filter(|x| I (COND) { 1 } else { 0 })`.
        def filt(m):
            var, cond = m.group(1), m.group(2)
            return f".filter(|{var}| I ({cond}) {{ 1 }} else {{ 0 }})"
        return re.sub(r"\.filter\(\|([A-Za-z_][A-Za-z0-9_]*)\|\s*([^)]+)\)", filt, p)
    return outside_strings(line, rewrite)


def map_field_pub(line: str) -> str:
    # Vais structs don't support per-field `pub` (parser rejects it). nl has
    # field-level pub (design P-visibility). Strip `pub ` from struct fields so
    # it compiles (visibility semantics are lost — a real backend limit, noted).
    # Only strips `pub ` that precedes `name:` inside a struct body line.
    return re.sub(r"^(\s*)pub\s+([A-Za-z_][A-Za-z0-9_]*\s*:)", r"\1\2", line)


def map_arm_return(line: str) -> str:
    # nl allows `Pattern => return expr` (design P6), but the Vais backend
    # rejects a bare `return` as a match-arm body (Vais trap 6). Wrap it:
    # `Pattern => return expr,`  ->  `Pattern => { return expr },`
    # Handle optional trailing comma. Skip if there's no `=> return` OUTSIDE a
    # string (code-as-data: an embedded program string must stay verbatim).
    if '"' in line:
        # blank string literals, then check if `=> return` survives in code
        blanked = re.sub(r'"(?:[^"\\]|\\.)*"', lambda mm: " " * len(mm.group(0)), line)
        if not re.search(r"=>\s*return\b", blanked):
            return line
    m = re.match(r"^(\s*)(.+?)=>\s*return\s+(.+?)(,?)\s*$", line)
    if m:
        indent, pat, expr, comma = m.groups()
        return f"{indent}{pat}=> {{ return {expr} }}{comma}"
    return line


def map_enum_qualified(line: str) -> str:
    # `Color.Red` -> `Red` ONLY in match-arm / construction position.
    # Heuristic: replace `Identifier.Identifier` where the right side starts
    # uppercase (a variant) AND it is not a method call (no '(' immediately after
    # in a `.method(` sense — variants like Some(x) keep their args).
    # We only strip the `EnumName.` prefix before an uppercase Variant.
    # NOT inside strings (code-as-data: a string "Color.Red" stays verbatim).
    return outside_strings(
        line, lambda p: re.sub(r"\b[A-Z][A-Za-z0-9_]*\.([A-Z][A-Za-z0-9_]*)", r"\1", p))


def transpile_line(line: str) -> str:
    # Order matters: handle let before generic type-mapping of the whole line.
    stripped = line.rstrip("\n")
    # Skip comment lines (both use '#')
    if stripped.lstrip().startswith("#"):
        return stripped
    out = stripped
    out = map_collection_methods(out)
    out = map_print(out)
    out = map_brace_escapes(out)
    out = map_field_pub(out)
    out = map_let(out)
    out = map_if(out)
    out = map_loop_keywords(out)
    out = map_bitnot(out)
    out = map_bitwise2(out)
    out = map_arm_return(out)
    out = map_enum_qualified(out)
    out = map_words(out)
    out = map_conversions(out)  # NumType(x) -> (x as vais) BEFORE map_types
    out = map_types(out)
    return out


UNSUPPORTED = [
    # for-loops (incl. exclusive range), while-loops, Result `?`, string
    # interpolation, .sum()/.map()/.filter()/.fold(), and list literals ARE
    # supported. (Nothing currently flagged; kept for future gaps.)
]


def expand_nested_match_arms(src: str) -> str:
    """Pre-pass over raw nl source: wrap `Pattern => match ... { ... },` arms.

    nl allows a match arm to directly contain another match expression. Vais
    requires a block arm body in this shape, so rewrite:
        P => match x { ... },
    into:
        P => { match x { ... } },

    Brace-tracked so the wrapper closes at the nested match's matching `}`.
    """
    lines = src.splitlines()
    out = []
    i = 0
    while i < len(lines):
        line = lines[i]
        m = re.match(r"^(\s*)(.+?=>\s*)match\s+(.+?)\s*\{\s*$", line)
        if not m:
            out.append(line)
            i += 1
            continue

        indent, arm_head, expr = m.groups()
        depth = line.count("{") - line.count("}")
        body = []
        j = i + 1
        while j < len(lines) and depth > 0:
            depth += lines[j].count("{") - lines[j].count("}")
            if depth == 0:
                break
            body.append(lines[j])
            j += 1

        out.append(f"{indent}{arm_head}{{ match {expr} {{")
        out.extend(expand_nested_match_arms("\n".join(body)).splitlines())
        if j < len(lines):
            close = lines[j]
            cm = re.match(r"^(\s*)}(,?)\s*$", close)
            if cm:
                close_indent, comma = cm.groups()
                out.append(f"{close_indent}}} }}{comma}")
            else:
                out.append(close)
        i = j + 1
    return "\n".join(out)


_for_ctr = [0]


def expand_for_loops(src: str) -> str:
    """Pre-pass over raw nl source: rewrite `for`/`while` blocks into Vais loops.

    `for i in A..=B { BODY }`  (inclusive) ->
        i := mut A;  L { I i > B { B }  BODY  i = i + 1 }
    `for i in A..B { BODY }`   (exclusive) ->
        i := mut A;  L { I i >= B { B }  BODY  i = i + 1 }
    `for x in LIST { BODY }`   ->
        __idxN := mut 0;  L { I __idxN >= LIST.len() as Int { B } x := LIST[__idxN]  BODY  __idxN += 1 }
    `while COND { BODY }`      ->
        L { I !(COND) { B }  BODY }

    Brace-tracked so the increment lands at the matching close. Nesting handled
    by recursing on each loop body.
    """
    lines = src.splitlines()
    out = []
    i = 0
    while i < len(lines):
        line = lines[i]
        # `..=` (inclusive) must be tried before `..` (exclusive).
        m_rangei = re.match(r"^(\s*)for\s+([A-Za-z_]\w*)\s+in\s+(.+?)\.\.=\s*(.+?)\s*\{\s*$", line)
        m_rangee = re.match(r"^(\s*)for\s+([A-Za-z_]\w*)\s+in\s+(.+?)\.\.\s*(.+?)\s*\{\s*$", line)
        m_iter = re.match(r"^(\s*)for\s+([A-Za-z_]\w*)\s+in\s+([A-Za-z_]\w*)\s*\{\s*$", line)
        m_while = re.match(r"^(\s*)while\s+(.+?)\s*\{\s*$", line)
        if m_rangei or m_rangee or m_iter or m_while:
            depth = line.count("{") - line.count("}")
            body = []
            j = i + 1
            while j < len(lines) and depth > 0:
                depth += lines[j].count("{") - lines[j].count("}")
                if depth == 0:
                    break
                body.append(lines[j])
                j += 1
            inner = expand_for_loops("\n".join(body)).splitlines()
            if m_rangei or m_rangee:
                m = m_rangei or m_rangee
                indent, var, lo, hi = m.groups()
                cmp = ">" if m_rangei else ">="
                out.append(f"{indent}{var} := mut {lo}")
                out.append(f"{indent}L {{")
                out.append(f"{indent}    I {var} {cmp} {hi} {{ B }}")
                out.extend(inner)
                out.append(f"{indent}    {var} = {var} + 1")
                out.append(f"{indent}}}")
            elif m_iter:
                indent, var, coll = m_iter.groups()
                _for_ctr[0] += 1
                idx = f"__idx{_for_ctr[0]}"
                out.append(f"{indent}{idx} := mut 0")
                out.append(f"{indent}L {{")
                out.append(f"{indent}    I {idx} >= {coll}.len() as Int {{ B }}")
                out.append(f"{indent}    {var} := {coll}[{idx}]")
                out.extend(inner)
                out.append(f"{indent}    {idx} = {idx} + 1")
                out.append(f"{indent}}}")
            else:  # while
                indent, cond = m_while.groups()
                out.append(f"{indent}L {{")
                out.append(f"{indent}    I !({cond}) {{ B }}")
                out.extend(inner)
                out.append(f"{indent}}}")
            i = j + 1
        else:
            out.append(line)
            i += 1
    return "\n".join(out)


def main():
    if len(sys.argv) != 2:
        print("usage: legacy_vais_bootstrap.py input.(vais|nl)", file=sys.stderr)
        sys.exit(2)
    src = open(sys.argv[1]).read()
    src = expand_nested_match_arms(src)  # structural pre-pass: arm match -> block arm
    src = expand_for_loops(src)  # structural pre-pass: for-loops -> Vais loop form
    warnings = []
    out_lines = []
    for n, line in enumerate(src.splitlines(), 1):
        for pat, msg in UNSUPPORTED:
            if pat.search(line) and not line.lstrip().startswith("#"):
                warnings.append(f"# line {n}: UNSUPPORTED — {msg}: {line.strip()}")
        out_lines.append(transpile_line(line + "\n"))
    if warnings:
        sys.stderr.write("\n".join(warnings) + "\n")
    body = "\n".join(out_lines)
    # Auto-prepend `use std/vec` if a Vec is used and not already imported
    # (nl's design: collections are prelude/no-import — the transpiler injects
    # the Vais import the backend requires).
    imports = []
    if re.search(r"\bVec<", body) and "use std/vec" not in body:
        imports.append("use std/vec")
    if re.search(r"\bHashMap<", body) and "use std/hashmap" not in body:
        imports.append("use std/hashmap")
    if imports:
        body = "\n".join(imports) + "\n" + body
    print(body)


if __name__ == "__main__":
    main()
