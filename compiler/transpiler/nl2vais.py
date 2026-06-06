#!/usr/bin/env python3
"""
nl -> Vais transpiler (prototype v0.1).

Strategy (per NEW-LANGUAGE-README): reuse the Vais backend. nl's v0.2 surface
syntax maps almost 1:1 to Vais, so a transpiler is the cheapest path to a working
prototype — no new parser/codegen. This deliberately covers only the small subset
validated in the design session (functions, let/mut, if/else, enums+match,
structs, Option, basic operators). Unsupported constructs are reported, not
silently mangled (honesty over coverage).

Usage:  python3 nl2vais.py input.nl > output.vais
        (then: vaisc build output.vais -o out && ./out)

NOT a real lexer/parser yet — it is a line/token rewriter. The point is to prove
the "nl surface -> Vais backend" pipeline runs end-to-end and computes correct
values, before investing in a real frontend.
"""
import re
import sys

# --- Type name map (nl -> Vais) ---
TYPE_MAP = {
    "Int": "i64", "Int8": "i8", "Int16": "i16", "Int32": "i32", "Int64": "i64",
    "Int128": "i128",
    "UInt8": "u8", "UInt16": "u16", "UInt32": "u32", "UInt64": "u64", "UInt128": "u128",
    "F32": "f32", "F64": "f64", "Bool": "bool", "Str": "str", "Char": "char",
    # Collection type names: nl List/Map -> Vais Vec/HashMap.
    "List": "Vec", "Map": "HashMap",
}

# --- Keyword/operator token map (whole-word) ---
WORD_MAP = {
    "and": "&&",
    "or": "||",
    "not": "!",
}


def map_types(line: str) -> str:
    # Replace nl type names with Vais types, whole-word only.
    def repl(m):
        return TYPE_MAP.get(m.group(0), m.group(0))
    return re.sub(r"\b[A-Za-z_][A-Za-z0-9_]*\b", repl, line)


def map_words(line: str) -> str:
    # Logic words -> operators. Whole-word, but NOT inside strings.
    # Simple approach: split on string literals and only rewrite outside them.
    parts = re.split(r'("(?:[^"\\]|\\.)*")', line)
    out = []
    for i, p in enumerate(parts):
        if i % 2 == 1:  # string literal — leave as-is
            out.append(p)
        else:
            for k, v in WORD_MAP.items():
                p = re.sub(rf"\b{k}\b", v, p)
            out.append(p)
    return "".join(out)


def map_let(line: str) -> str:
    # `let mut x = expr`  -> `x := mut expr`
    # `let x = expr`      -> `x := expr`
    m = re.match(r"^(\s*)let\s+mut\s+([A-Za-z_][A-Za-z0-9_]*)\s*(:\s*[^=]+)?=\s*(.*)$", line)
    if m:
        indent, name, ty, rhs = m.groups()
        return f"{indent}{name} := mut {rhs}"
    m = re.match(r"^(\s*)let\s+([A-Za-z_][A-Za-z0-9_]*)\s*(:\s*[^=]+)?=\s*(.*)$", line)
    if m:
        indent, name, ty, rhs = m.groups()
        rhs_stripped = rhs.rstrip()
        # List literal on RHS: Vais needs an explicit Vec<T> type annotation.
        # `let v = [10, 20, 30]` -> `v: Vec<i64> = [10, 20, 30]`. We infer the
        # element type from the first int/float literal (prototype: Int/F64).
        if rhs_stripped.startswith("[") and rhs_stripped.endswith("]") and rhs_stripped != "[]":
            first = rhs_stripped[1:-1].split(",")[0].strip()
            elem = "f64" if re.match(r"^-?\d+\.\d+", first) else "i64"
            return f"{indent}{name}: Vec<{elem}> = {rhs}"
        return f"{indent}{name} := {rhs}"
    return line


def map_if(line: str) -> str:
    # `if cond {`        -> `I cond {`
    # `} else if cond {` -> `} else I cond {`
    # `} else {`         -> stays
    line = re.sub(r"\belse\s+if\b", "else I", line)
    line = re.sub(r"(^|\}\s*|\s)if\b", lambda m: m.group(0).replace("if", "I"), line)
    return line


def map_bitnot(line: str) -> str:
    # `bitnot(x)` -> `(~x)`  (only the simple single-arg form)
    return re.sub(r"\bbitnot\(([^()]*)\)", r"(~\1)", line)


def map_collection_methods(line: str) -> str:
    # nl collection ops are methods. Vais std/vec has map/filter/fold but no
    # `sum`. Express sum via fold (verified: fold(0, |a,x| a+x) works).
    # `.sum()` -> `.fold(0, |__a, __x| __a + __x)`
    line = re.sub(r"\.sum\(\)", r".fold(0, |__a, __x| __a + __x)", line)
    # Vais `filter` wants `fn(T)->i64`, but nl predicates are Bool. Wrap a
    # boolean filter body into i64: `.filter(|x| COND)` ->
    # `.filter(|x| I (COND) { 1 } else { 0 })`. (Backend limit: Vais filter
    # predicate returns i64, not bool.)
    def filt(m):
        var, cond = m.group(1), m.group(2)
        return f".filter(|{var}| I ({cond}) {{ 1 }} else {{ 0 }})"
    line = re.sub(r"\.filter\(\|([A-Za-z_][A-Za-z0-9_]*)\|\s*([^)]+)\)", filt, line)
    return line


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
    # Handle optional trailing comma.
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
    return re.sub(r"\b[A-Z][A-Za-z0-9_]*\.([A-Z][A-Za-z0-9_]*)", r"\1", line)


def transpile_line(line: str) -> str:
    # Order matters: handle let before generic type-mapping of the whole line.
    stripped = line.rstrip("\n")
    # Skip comment lines (both use '#')
    if stripped.lstrip().startswith("#"):
        return stripped
    out = stripped
    out = map_collection_methods(out)
    out = map_field_pub(out)
    out = map_let(out)
    out = map_if(out)
    out = map_bitnot(out)
    out = map_arm_return(out)
    out = map_enum_qualified(out)
    out = map_words(out)
    out = map_types(out)
    return out


UNSUPPORTED = [
    (re.compile(r"\bwhile\b"), "while-loops not yet transpiled"),
    # Note: for-loops, Result `?`, string interpolation, .sum()/.map()/.filter()/
    # .fold(), and list literals ARE supported (Vais accepts ? and "{..}" directly).
]

_for_ctr = [0]


def expand_for_loops(src: str) -> str:
    """Pre-pass over raw nl source: rewrite `for` blocks into Vais loop form.

    `for i in A..=B {  BODY  }`  ->
        i := mut A
        L { I i > B { B } BODY  i = i + 1 }
    `for x in LIST {  BODY  }`   ->
        __idxN := mut 0
        L { I __idxN >= LIST.len() as Int { B } x := LIST[__idxN]  BODY  __idxN = __idxN + 1 }

    Brace-tracked so the increment lands at the matching close. Handles nesting
    by processing the outermost `for` and recursing on the remainder. Range form
    only supports `..=` (inclusive) in this prototype.
    """
    lines = src.splitlines()
    out = []
    i = 0
    while i < len(lines):
        line = lines[i]
        m_range = re.match(r"^(\s*)for\s+([A-Za-z_]\w*)\s+in\s+(.+?)\.\.=\s*(.+?)\s*\{\s*$", line)
        m_iter = re.match(r"^(\s*)for\s+([A-Za-z_]\w*)\s+in\s+([A-Za-z_]\w*)\s*\{\s*$", line)
        if m_range or m_iter:
            # Find matching close brace by depth from this line's opening `{`.
            depth = line.count("{") - line.count("}")
            body = []
            j = i + 1
            while j < len(lines) and depth > 0:
                depth += lines[j].count("{") - lines[j].count("}")
                if depth == 0:
                    break
                body.append(lines[j])
                j += 1
            # body[] is the loop body; lines[j] is the closing `}` line.
            inner = expand_for_loops("\n".join(body)).splitlines()
            if m_range:
                indent, var, lo, hi = m_range.groups()
                out.append(f"{indent}{var} := mut {lo}")
                out.append(f"{indent}L {{")
                out.append(f"{indent}    I {var} > {hi} {{ B }}")
                out.extend(inner)
                out.append(f"{indent}    {var} = {var} + 1")
                out.append(f"{indent}}}")
            else:
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
            i = j + 1
        else:
            out.append(line)
            i += 1
    return "\n".join(out)


def main():
    if len(sys.argv) != 2:
        print("usage: nl2vais.py input.nl", file=sys.stderr)
        sys.exit(2)
    src = open(sys.argv[1]).read()
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
