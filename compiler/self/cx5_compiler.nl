# expect: 0
# nl self-host compiler — CX5: function definitions + (recursive) calls.
#
# A program is ';'-separated:
#   fn <f>(<p>) { return <expr> }    define single-letter fn `f` with param `p`
#   return <expr>                    final result (printed as LLVM IR)
# <expr>: arithmetic (+ - *) over numbers, single-letter vars/params, and calls
# `<f>(<argexpr>)`. Calls may be recursive/nested (a body may call any fn, incl.
# itself), as long as the recursion terminates.
#
# Vais-limit workaround (task_54658a43): the evaluation environment `Env` and the
# function table `Defs` are fixed-field STRUCTS, not Vecs, so they pass through
# RECURSIVE call evaluation without E022 (Vec recursion = move error; struct
# recursion is clean — measured). Source is an immutable Str (passes fine).
# Up to 3 user functions (Defs has 3 slots).

struct Env { a: Int, b: Int, c: Int, d: Int, e: Int, f: Int, n: Int, x: Int }
struct Defs {
    n0name: Int, n0param: Int, n0bs: Int, n0be: Int,
    n1name: Int, n1param: Int, n1bs: Int, n1be: Int,
    n2name: Int, n2param: Int, n2bs: Int, n2be: Int,
    count: Int
}

fn is_digit(c: Int) -> Bool { return c >= 48 and c <= 57 }
fn is_lower(c: Int) -> Bool { return c >= 97 and c <= 122 }
fn is_sp(c: Int) -> Bool {
    if c == 32 { return true }
    if c == 9 { return true }
    if c == 10 { return true }
    return false
}
fn skip_sp(src: Str, p: Int, end: Int) -> Int {
    let mut q = p
    let mut go = true
    while go {
        if q >= end { go = false }
        else if is_sp(src[q]) { q = q + 1 }
        else { go = false }
    }
    return q
}

# --- Env: 8 slots a b c d e f n x ---
fn eget(env: Env, ch: Int) -> Int {
    if ch == 97 { return env.a }
    if ch == 98 { return env.b }
    if ch == 99 { return env.c }
    if ch == 100 { return env.d }
    if ch == 101 { return env.e }
    if ch == 102 { return env.f }
    if ch == 110 { return env.n }
    if ch == 120 { return env.x }
    return 0
}
fn eset(env: Env, ch: Int, v: Int) -> Env {
    if ch == 97 { return Env { a: v, b: env.b, c: env.c, d: env.d, e: env.e, f: env.f, n: env.n, x: env.x } }
    if ch == 98 { return Env { a: env.a, b: v, c: env.c, d: env.d, e: env.e, f: env.f, n: env.n, x: env.x } }
    if ch == 99 { return Env { a: env.a, b: env.b, c: v, d: env.d, e: env.e, f: env.f, n: env.n, x: env.x } }
    if ch == 100 { return Env { a: env.a, b: env.b, c: env.c, d: v, e: env.e, f: env.f, n: env.n, x: env.x } }
    if ch == 101 { return Env { a: env.a, b: env.b, c: env.c, d: env.d, e: v, f: env.f, n: env.n, x: env.x } }
    if ch == 102 { return Env { a: env.a, b: env.b, c: env.c, d: env.d, e: env.e, f: v, n: env.n, x: env.x } }
    if ch == 110 { return Env { a: env.a, b: env.b, c: env.c, d: env.d, e: env.e, f: env.f, n: v, x: env.x } }
    if ch == 120 { return Env { a: env.a, b: env.b, c: env.c, d: env.d, e: env.e, f: env.f, n: env.n, x: v } }
    return env
}
fn zero_env() -> Env {
    return Env { a: 0, b: 0, c: 0, d: 0, e: 0, f: 0, n: 0, x: 0 }
}

# --- Defs lookup: given a fn-name letter, return its slot index (0..2) or -1 ---
fn def_name(d: Defs, i: Int) -> Int {
    if i == 0 { return d.n0name }
    if i == 1 { return d.n1name }
    if i == 2 { return d.n2name }
    return 0
}
fn def_param(d: Defs, i: Int) -> Int {
    if i == 0 { return d.n0param }
    if i == 1 { return d.n1param }
    if i == 2 { return d.n2param }
    return 0
}
fn def_bs(d: Defs, i: Int) -> Int {
    if i == 0 { return d.n0bs }
    if i == 1 { return d.n1bs }
    if i == 2 { return d.n2bs }
    return 0
}
fn def_be(d: Defs, i: Int) -> Int {
    if i == 0 { return d.n0be }
    if i == 1 { return d.n1be }
    if i == 2 { return d.n2be }
    return 0
}
fn find_def(d: Defs, name: Int) -> Int {
    let mut i = 0
    let mut found = 0 - 1
    while i < d.count {
        if def_name(d, i) == name { found = i }
        i = i + 1
    }
    return found
}

# --- Call-aware arithmetic evaluator ---
# Evaluates src[start..end) under env + defs. Recognizes `<f>(<arg>)` calls.
# Grammar (precedence * over + -, left assoc):
#   expr   = term (('+'|'-') term)*
#   term   = factor ('*' factor)*
#   factor = number | call | var
#   call   = lower '(' expr ')'
#   var    = lower
# Single-pass with a cursor; recursion into eval_factor for call args + bodies.

# Parse a number starting at p; returns value, advances via a 1-slot "cursor" array.
# To keep within Vais limits we return value and the next position is recomputed by
# the caller scanning; instead we implement a small recursive-descent with explicit
# index passed by value and the "next position" returned through a 2-field struct.
struct Cur { val: Int, pos: Int }

fn eval_factor(src: Str, p0: Int, end: Int, env: Env, defs: Defs) -> Cur {
    let p = skip_sp(src, p0, end)
    if p >= end { return Cur { val: 0, pos: p } }
    let c = src[p]
    if is_digit(c) {
        # number run
        let mut q = p
        let mut v = 0
        let mut go = true
        while go {
            if q >= end { go = false }
            else if is_digit(src[q]) {
                v = v * 10 + (src[q] - 48)
                q = q + 1
            } else { go = false }
        }
        return Cur { val: v, pos: q }
    }
    if is_lower(c) {
        # var or call: lookahead for '('
        let after = skip_sp(src, p + 1, end)
        if after < end {
            if src[after] == 40 {
                # call: c is fn name, evaluate arg expr inside (...)
                let argc = eval_expr(src, after + 1, end, env, defs)
                # argc.pos should be at ')'
                let close = skip_sp(src, argc.pos, end)
                let mut np = close
                if np < end {
                    if src[np] == 41 { np = np + 1 }
                }
                # dispatch
                let idx = find_def(defs, c)
                let mut r = 0
                if idx >= 0 {
                    let param = def_param(defs, idx)
                    let bs = def_bs(defs, idx)
                    let be = def_be(defs, idx)
                    let callee = eset(zero_env(), param, argc.val)
                    let body = eval_expr(src, bs, be, callee, defs)
                    r = body.val
                }
                return Cur { val: r, pos: np }
            }
        }
        # plain var
        return Cur { val: eget(env, c), pos: p + 1 }
    }
    # parenthesized expr
    if c == 40 {
        let inner = eval_expr(src, p + 1, end, env, defs)
        let close = skip_sp(src, inner.pos, end)
        let mut np = close
        if np < end {
            if src[np] == 41 { np = np + 1 }
        }
        return Cur { val: inner.val, pos: np }
    }
    return Cur { val: 0, pos: p + 1 }
}

fn eval_term(src: Str, p0: Int, end: Int, env: Env, defs: Defs) -> Cur {
    let mut cur = eval_factor(src, p0, end, env, defs)
    let mut go = true
    while go {
        let p = skip_sp(src, cur.pos, end)
        if p >= end { go = false }
        else if src[p] == 42 {
            let rhs = eval_factor(src, p + 1, end, env, defs)
            cur = Cur { val: cur.val * rhs.val, pos: rhs.pos }
        } else { go = false }
    }
    return cur
}

fn eval_expr(src: Str, p0: Int, end: Int, env: Env, defs: Defs) -> Cur {
    let mut cur = eval_term(src, p0, end, env, defs)
    let mut go = true
    while go {
        let p = skip_sp(src, cur.pos, end)
        if p >= end { go = false }
        else if src[p] == 43 {
            let rhs = eval_term(src, p + 1, end, env, defs)
            cur = Cur { val: cur.val + rhs.val, pos: rhs.pos }
        } else if src[p] == 45 {
            let rhs = eval_term(src, p + 1, end, env, defs)
            cur = Cur { val: cur.val - rhs.val, pos: rhs.pos }
        } else { go = false }
    }
    return cur
}

fn empty_defs() -> Defs {
    return Defs {
        n0name: 0, n0param: 0, n0bs: 0, n0be: 0,
        n1name: 0, n1param: 0, n1bs: 0, n1be: 0,
        n2name: 0, n2param: 0, n2bs: 0, n2be: 0,
        count: 0
    }
}
# Append a def into the next free slot (by current count). Returns new Defs.
fn def_add(d: Defs, name: Int, param: Int, bs: Int, be: Int) -> Defs {
    if d.count == 0 {
        return Defs {
            n0name: name, n0param: param, n0bs: bs, n0be: be,
            n1name: d.n1name, n1param: d.n1param, n1bs: d.n1bs, n1be: d.n1be,
            n2name: d.n2name, n2param: d.n2param, n2bs: d.n2bs, n2be: d.n2be,
            count: 1
        }
    }
    if d.count == 1 {
        return Defs {
            n0name: d.n0name, n0param: d.n0param, n0bs: d.n0bs, n0be: d.n0be,
            n1name: name, n1param: param, n1bs: bs, n1be: be,
            n2name: d.n2name, n2param: d.n2param, n2bs: d.n2bs, n2be: d.n2be,
            count: 2
        }
    }
    return Defs {
        n0name: d.n0name, n0param: d.n0param, n0bs: d.n0bs, n0be: d.n0be,
        n1name: d.n1name, n1param: d.n1param, n1bs: d.n1bs, n1be: d.n1be,
        n2name: name, n2param: param, n2bs: bs, n2be: be,
        count: 3
    }
}

# Find the end of the current statement (';' or end-of-src) from p.
fn stmt_end(src: Str, p: Int, n: Int) -> Int {
    let mut j = p
    let mut go = true
    while go {
        if j >= n { go = false }
        else if src[j] == 59 { go = false }
        else { j = j + 1 }
    }
    return j
}

# Parse the whole program: collect `fn` defs, then evaluate the final `return`.
# `fn <f> ( <p> ) { return <expr> }` — we record name=f, param=p, body=[bs,be)
# where bs is just after "return" inside the braces and be is the '}'.
fn run_program(src: Str) -> Int {
    let n = src.len()
    let mut defs = empty_defs()
    let mut result = 0
    let mut i = 0
    while i < n {
        let p = skip_sp(src, i, n)
        let j = stmt_end(src, p, n)
        if p < j {
            let first = src[p]
            if first == 102 {
                # "fn": parse a definition. layout: fn <f> ( <param> ) { return <e> }
                let q = skip_sp(src, p + 2, n)
                let fname = src[q]
                # find '('
                let mut op = q + 1
                let mut g1 = true
                while g1 {
                    if op >= j { g1 = false }
                    else if src[op] == 40 { g1 = false }
                    else { op = op + 1 }
                }
                let pstart = skip_sp(src, op + 1, n)
                let pname = src[pstart]
                # find '{'
                let mut br = pstart
                let mut g2 = true
                while g2 {
                    if br >= j { g2 = false }
                    else if src[br] == 123 { g2 = false }
                    else { br = br + 1 }
                }
                # inside braces: skip to after "return"
                let inner = skip_sp(src, br + 1, n)
                # assume "return" keyword (6 chars); body starts after it
                let bs = inner + 6
                # body end = '}' (search to end of statement region or n)
                let mut be = bs
                let mut g3 = true
                while g3 {
                    if be >= n { g3 = false }
                    else if src[be] == 125 { g3 = false }
                    else { be = be + 1 }
                }
                defs = def_add(defs, fname, pname, bs, be)
                # advance past the '}' for this def (stmt_end stopped at ';' which
                # is after '}', but body may contain none; jump to be+1)
                i = be + 1
                # also skip a trailing ';' if present
                let after = skip_sp(src, i, n)
                if after < n {
                    if src[after] == 59 { i = after + 1 }
                }
            } else if first == 114 {
                # "return <expr>"
                let r = eval_expr(src, p + 6, j, zero_env(), defs)
                result = r.val
                i = j + 1
            } else {
                i = j + 1
            }
        } else {
            i = j + 1
        }
    }
    return result
}

fn emit_ir(value: Int) -> Int {
    print("define i64 @main() {")
    print("  ret i64 {value}")
    print("}")
    return 0
}

fn main() -> Int {
    # Program: define d(x)=x*2 and s(a)=a*a, then return d(21)+s(5) = 42+25 = 67.
    # Literal braces in the embedded program are written `{{`/`}}` (nl escape ->
    # Vais `\{`/`\}`), since Vais interpolates `{ }` in every string literal.
    let value = run_program("fn d(x) {{ return x * 2 }}; fn s(a) {{ return a * a }}; return d(21) + s(5)")
    return emit_ir(value)
}
