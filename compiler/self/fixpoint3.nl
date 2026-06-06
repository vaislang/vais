# expect: 0
# nl self-host — fixpoint compiler v3: multi-char FUNCTIONS on the List pipeline.
#
# Extends fixpoint2.nl (multi-char identifiers via List<Token> + List<Var>) with
# function definitions and calls, BOTH using multi-character names:
#   fn <name>(<p>) { return <expr> }      (single param, body is `return <expr>`)
#   fn <name>(<p>, <q>) { return <expr> } (two params)
# Calls `<name>(<arg>)` / `<name>(<arg>, <arg>)` may be nested/recursive.
#
# Function bodies are stored as TOKEN-INDEX ranges; the evaluator threads the
# borrowed token list, function table, and a fresh per-call variable scope
# (all `&List<...>`) through recursion — possible thanks to the Vais `&Vec`
# borrow-recursion fix (compiler 214c97cf). Three simultaneous `&List` params in
# a recursive function are verified to work.

# Token kinds: 0=num,1=ident,2='+',3='*',4='-',5='=',6=';',7=let,8=return,
#              9='(',10=')',11='{',12='}',13=fn,14=','
struct Token { kind: Int, value: Int, nstart: Int, nlen: Int }
struct Var { nstart: Int, nlen: Int, value: Int }
# A function: name range, up to two param name ranges (p2len=0 if single-arg),
# and the body's token-index range [bstart, bend).
struct Fn {
    nstart: Int, nlen: Int,
    p1s: Int, p1l: Int,
    p2s: Int, p2l: Int,
    bstart: Int, bend: Int
}

fn is_digit(c: Int) -> Bool { return c >= 48 and c <= 57 }
fn is_alpha(c: Int) -> Bool {
    if c >= 97 and c <= 122 { return true }
    if c >= 65 and c <= 90 { return true }
    if c == 95 { return true }
    return false
}
fn is_alnum(c: Int) -> Bool {
    if is_alpha(c) { return true }
    if is_digit(c) { return true }
    return false
}
fn is_space(c: Int) -> Bool {
    if c == 32 { return true }
    if c == 9 { return true }
    if c == 10 { return true }
    return false
}
fn word_is(src: Str, a: Int, alen: Int, w0: Int, w1: Int, w2: Int, w3: Int, w4: Int, w5: Int, wlen: Int) -> Int {
    if alen != wlen { return 0 }
    if alen >= 1 { if src[a] != w0 { return 0 } }
    if alen >= 2 { if src[a + 1] != w1 { return 0 } }
    if alen >= 3 { if src[a + 2] != w2 { return 0 } }
    if alen >= 4 { if src[a + 3] != w3 { return 0 } }
    if alen >= 5 { if src[a + 4] != w4 { return 0 } }
    if alen >= 6 { if src[a + 5] != w5 { return 0 } }
    return 1
}

fn tokenize(src: Str) -> List<Token> {
    let mut toks: List<Token> = []
    let n = src.len()
    let mut i = 0
    while i < n {
        let c = src[i]
        if is_space(c) {
            i = i + 1
        } else if is_digit(c) {
            let mut v = 0
            let mut go = true
            while go {
                if i >= n { go = false }
                else if is_digit(src[i]) { v = v * 10 + (src[i] - 48); i = i + 1 }
                else { go = false }
            }
            toks.push(Token { kind: 0, value: v, nstart: 0, nlen: 0 })
        } else if is_alpha(c) {
            let start = i
            let mut go = true
            while go {
                if i >= n { go = false }
                else if is_alnum(src[i]) { i = i + 1 }
                else { go = false }
            }
            let len = i - start
            if word_is(src, start, len, 108, 101, 116, 0, 0, 0, 3) == 1 {
                toks.push(Token { kind: 7, value: 0, nstart: start, nlen: len })
            } else if word_is(src, start, len, 114, 101, 116, 117, 114, 110, 6) == 1 {
                toks.push(Token { kind: 8, value: 0, nstart: start, nlen: len })
            } else if word_is(src, start, len, 102, 110, 0, 0, 0, 0, 2) == 1 {
                toks.push(Token { kind: 13, value: 0, nstart: start, nlen: len })
            } else if word_is(src, start, len, 105, 102, 0, 0, 0, 0, 2) == 1 {
                toks.push(Token { kind: 15, value: 0, nstart: start, nlen: len })   # if
            } else if word_is(src, start, len, 116, 104, 101, 110, 0, 0, 4) == 1 {
                toks.push(Token { kind: 16, value: 0, nstart: start, nlen: len })   # then
            } else if word_is(src, start, len, 101, 108, 115, 101, 0, 0, 4) == 1 {
                toks.push(Token { kind: 17, value: 0, nstart: start, nlen: len })   # else
            } else {
                toks.push(Token { kind: 1, value: 0, nstart: start, nlen: len })
            }
        } else if c == 43 {
            toks.push(Token { kind: 2, value: 0, nstart: 0, nlen: 0 }); i = i + 1
        } else if c == 42 {
            toks.push(Token { kind: 3, value: 0, nstart: 0, nlen: 0 }); i = i + 1
        } else if c == 45 {
            toks.push(Token { kind: 4, value: 0, nstart: 0, nlen: 0 }); i = i + 1
        } else if c == 61 {
            # `==` (comparison, kind 20) vs `=` (assignment, kind 5)
            if i + 1 < n {
                if src[i + 1] == 61 {
                    toks.push(Token { kind: 20, value: 0, nstart: 0, nlen: 0 }); i = i + 2
                } else {
                    toks.push(Token { kind: 5, value: 0, nstart: 0, nlen: 0 }); i = i + 1
                }
            } else {
                toks.push(Token { kind: 5, value: 0, nstart: 0, nlen: 0 }); i = i + 1
            }
        } else if c == 60 {
            toks.push(Token { kind: 18, value: 0, nstart: 0, nlen: 0 }); i = i + 1   # <
        } else if c == 62 {
            toks.push(Token { kind: 19, value: 0, nstart: 0, nlen: 0 }); i = i + 1   # >
        } else if c == 59 {
            toks.push(Token { kind: 6, value: 0, nstart: 0, nlen: 0 }); i = i + 1
        } else if c == 40 {
            toks.push(Token { kind: 9, value: 0, nstart: 0, nlen: 0 }); i = i + 1
        } else if c == 41 {
            toks.push(Token { kind: 10, value: 0, nstart: 0, nlen: 0 }); i = i + 1
        } else if c == 123 {
            toks.push(Token { kind: 11, value: 0, nstart: 0, nlen: 0 }); i = i + 1
        } else if c == 125 {
            toks.push(Token { kind: 12, value: 0, nstart: 0, nlen: 0 }); i = i + 1
        } else if c == 44 {
            toks.push(Token { kind: 14, value: 0, nstart: 0, nlen: 0 }); i = i + 1
        } else {
            i = i + 1
        }
    }
    return toks
}

# --- symbol table ---
fn name_eq(src: Str, a: Int, alen: Int, b: Int, blen: Int) -> Int {
    if alen != blen { return 0 }
    let mut k = 0
    while k < alen {
        if src[a + k] == src[b + k] { k = k + 1 } else { return 0 }
    }
    return 1
}
fn lookup(vars: &List<Var>, src: Str, qs: Int, ql: Int) -> Int {
    let m = vars.len()
    let mut i = 0
    while i < m {
        let v = vars[i]
        if name_eq(src, v.nstart, v.nlen, qs, ql) == 1 { return v.value }
        i = i + 1
    }
    return 0
}
# Find a function by name; returns its index in the table or -1.
fn find_fn(fns: &List<Fn>, src: Str, qs: Int, ql: Int) -> Int {
    let m = fns.len()
    let mut i = 0
    while i < m {
        let f = fns[i]
        if name_eq(src, f.nstart, f.nlen, qs, ql) == 1 { return i }
        i = i + 1
    }
    return 0 - 1
}

# --- build the function table by scanning the token list for `fn` defs ---
# `fn <name> ( <p1> [, <p2>] ) { <body...> }` — record name/param ranges and the
# body token range (just inside `{` .. matching `}`).
fn build_fns(toks: &List<Token>, n: Int) -> List<Fn> {
    let mut fns: List<Fn> = []
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 13 {
            # fn name ( p1 [, p2] ) { body }
            let nt = toks[i + 1]
            # i+2 is '(', i+3 is p1
            let p1 = toks[i + 3]
            let mut p2s = 0
            let mut p2l = 0
            let mut j = i + 4
            # check for ', p2'
            let maybe = toks[j]
            if maybe.kind == 14 {
                let p2 = toks[j + 1]
                p2s = p2.nstart
                p2l = p2.nlen
                j = j + 2
            }
            # j is ')', j+1 is '{', body starts at j+2
            let bstart = j + 2
            # find matching '}' (no nesting in this grammar): scan for kind 12
            let mut be = bstart
            let mut go = true
            while go {
                if be >= n { go = false }
                else {
                    let bt = toks[be]
                    if bt.kind == 12 { go = false } else { be = be + 1 }
                }
            }
            fns.push(Fn {
                nstart: nt.nstart, nlen: nt.nlen,
                p1s: p1.nstart, p1l: p1.nlen,
                p2s: p2s, p2l: p2l,
                bstart: bstart, bend: be
            })
            i = be + 1
        } else {
            i = i + 1
        }
    }
    return fns
}

# --- expression evaluator (threads toks + fns + vars by &borrow) ---
# A factor: number, call `name(args)`, or variable.
fn eval_factor(toks: &List<Token>, fns: &List<Fn>, vars: &List<Var>, src: Str, i: Int) -> Int {
    let t = toks[i]
    if t.kind == 0 { return t.value }
    if t.kind == 1 {
        # ident: call if followed by '('
        let nxt = toks[i + 1]
        if nxt.kind == 9 {
            return eval_call(toks, fns, vars, src, i)
        }
        return lookup(vars, src, t.nstart, t.nlen)
    }
    return 0
}

# Evaluate a call `name ( arg1 [, arg2] )` whose name token is at i.
fn eval_call(toks: &List<Token>, fns: &List<Fn>, vars: &List<Var>, src: Str, i: Int) -> Int {
    let nt = toks[i]
    let idx = find_fn(fns, src, nt.nstart, nt.nlen)
    if idx < 0 { return 0 }
    let f = fns[idx]
    # arg1 expr starts at i+2 (after name and '('); ends at ',' or ')'.
    let a1stop = arg_end(toks, i + 2)
    let arg1 = eval_expr(toks, fns, vars, src, i + 2, a1stop)
    # build callee scope
    let mut callee: List<Var> = []
    callee.push(Var { nstart: f.p1s, nlen: f.p1l, value: arg1 })
    let astop = toks[a1stop]
    if astop.kind == 14 {
        let a2stop = arg_end(toks, a1stop + 1)
        let arg2 = eval_expr(toks, fns, vars, src, a1stop + 1, a2stop)
        callee.push(Var { nstart: f.p2s, nlen: f.p2l, value: arg2 })
    }
    # body is `return <expr>` within [bstart, bend): expr starts at bstart+1.
    return eval_value(toks, fns, &callee, src, f.bstart + 1, f.bend)
}

# Find the end of an argument expression: index of the matching ',' or ')'
# (no nested parens in argument position for this grammar level beyond calls,
# which consume their own parens — but a call arg may itself contain a call, so
# track paren depth).
fn arg_end(toks: &List<Token>, i: Int) -> Int {
    let mut j = i
    let mut depth = 0
    let mut go = true
    while go {
        let t = toks[j]
        if t.kind == 9 { depth = depth + 1; j = j + 1 }
        else if t.kind == 10 {
            if depth == 0 { go = false } else { depth = depth - 1; j = j + 1 }
        }
        else if t.kind == 14 {
            if depth == 0 { go = false } else { j = j + 1 }
        }
        else { j = j + 1 }
    }
    return j
}

fn eval_term(toks: &List<Token>, fns: &List<Fn>, vars: &List<Var>, src: Str, i: Int, stop: Int, acc: Int) -> Int {
    if i >= stop { return acc }
    let t = toks[i]
    if t.kind == 3 {
        let rhs = eval_factor(toks, fns, vars, src, i + 1)
        let after = skip_factor(toks, i + 1)
        return eval_term(toks, fns, vars, src, after, stop, acc * rhs)
    }
    return acc
}
# Advance past one factor (a number/var is 1 token; a call is name '(' ... ')').
fn skip_factor(toks: &List<Token>, i: Int) -> Int {
    let t = toks[i]
    if t.kind == 1 {
        let nxt = toks[i + 1]
        if nxt.kind == 9 {
            # call: skip to after matching ')'
            let e = arg_end_call(toks, i + 2)
            return e + 1
        }
    }
    return i + 1
}
# Skip to the closing ')' of a call whose args start at i (returns ')' index).
fn arg_end_call(toks: &List<Token>, i: Int) -> Int {
    let mut j = i
    let mut depth = 0
    let mut go = true
    while go {
        let t = toks[j]
        if t.kind == 9 { depth = depth + 1; j = j + 1 }
        else if t.kind == 10 {
            if depth == 0 { go = false } else { depth = depth - 1; j = j + 1 }
        }
        else { j = j + 1 }
    }
    return j
}
fn skip_term(toks: &List<Token>, i: Int, stop: Int) -> Int {
    if i >= stop { return stop }
    let t = toks[i]
    if t.kind == 3 {
        let after = skip_factor(toks, i + 1)
        return skip_term(toks, after, stop)
    }
    return i
}

fn eval_expr(toks: &List<Token>, fns: &List<Fn>, vars: &List<Var>, src: Str, i: Int, stop: Int) -> Int {
    let first = eval_factor(toks, fns, vars, src, i)
    let af = skip_factor(toks, i)
    let acc = eval_term(toks, fns, vars, src, af, stop, first)
    let after = skip_term(toks, af, stop)
    return eval_fold(toks, fns, vars, src, after, stop, acc)
}
fn eval_fold(toks: &List<Token>, fns: &List<Fn>, vars: &List<Var>, src: Str, i: Int, stop: Int, acc: Int) -> Int {
    if i >= stop { return acc }
    let op = toks[i]
    if op.kind == 2 {
        let rf = eval_factor(toks, fns, vars, src, i + 1)
        let af = skip_factor(toks, i + 1)
        let term = eval_term(toks, fns, vars, src, af, stop, rf)
        let after = skip_term(toks, af, stop)
        return eval_fold(toks, fns, vars, src, after, stop, acc + term)
    }
    if op.kind == 4 {
        let rf = eval_factor(toks, fns, vars, src, i + 1)
        let af = skip_factor(toks, i + 1)
        let term = eval_term(toks, fns, vars, src, af, stop, rf)
        let after = skip_term(toks, af, stop)
        return eval_fold(toks, fns, vars, src, after, stop, acc - term)
    }
    return acc
}

# Find the index of the first token of `kind` in [i, stop), or stop.
fn find_kind(toks: &List<Token>, i: Int, stop: Int, kind: Int) -> Int {
    if i >= stop { return stop }
    let t = toks[i]
    if t.kind == kind { return i }
    return find_kind(toks, i + 1, stop, kind)
}

# Evaluate a value: either `if <e> <cmp> <e> then <e> else <e>` (cmp: < > ==)
# or a plain arithmetic expression. Used for function bodies and `return`.
# FP3b: enables multi-char recursion (a body can branch on a base case).
fn eval_value(toks: &List<Token>, fns: &List<Fn>, vars: &List<Var>, src: Str, i: Int, stop: Int) -> Int {
    let t = toks[i]
    if t.kind == 15 {
        # if: condition is [i+1, then), branches split at then/else.
        let then_pos = find_kind(toks, i + 1, stop, 16)
        let else_pos = find_kind(toks, then_pos + 1, stop, 17)
        # comparison operator within [i+1, then_pos)
        let cmp_lt = find_kind(toks, i + 1, then_pos, 18)
        let cmp_gt = find_kind(toks, i + 1, then_pos, 19)
        let cmp_eq = find_kind(toks, i + 1, then_pos, 20)
        let mut oppos = then_pos
        let mut op = 0
        if cmp_lt < then_pos { oppos = cmp_lt; op = 18 }
        else if cmp_gt < then_pos { oppos = cmp_gt; op = 19 }
        else if cmp_eq < then_pos { oppos = cmp_eq; op = 20 }
        let lhs = eval_expr(toks, fns, vars, src, i + 1, oppos)
        let rhs = eval_expr(toks, fns, vars, src, oppos + 1, then_pos)
        let mut cond = 0
        if op == 18 { if lhs < rhs { cond = 1 } }
        else if op == 19 { if lhs > rhs { cond = 1 } }
        else { if lhs == rhs { cond = 1 } }
        if cond == 1 {
            return eval_value(toks, fns, vars, src, then_pos + 1, else_pos)
        }
        return eval_value(toks, fns, vars, src, else_pos + 1, stop)
    }
    return eval_expr(toks, fns, vars, src, i, stop)
}

fn find_semi(toks: &List<Token>, i: Int, n: Int) -> Int {
    if i >= n { return n }
    let t = toks[i]
    if t.kind == 6 { return i }
    return find_semi(toks, i + 1, n)
}

fn emit_ir(value: Int) -> Int {
    print("define i64 @main() {")
    print("  ret i64 {value}")
    print("}")
    return 0
}

# Run: build the function table, then execute top-level let/return statements.
fn run_program(src: Str) -> Int {
    let toks = tokenize(src)
    let n = toks.len()
    let fns = build_fns(&toks, n)
    let mut vars: List<Var> = []
    let mut result = 0
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 13 {
            # skip a fn definition at top level: advance past its '}'
            let mut j = i + 1
            let mut go = true
            while go {
                if j >= n { go = false }
                else {
                    let jt = toks[j]
                    if jt.kind == 12 { go = false } else { j = j + 1 }
                }
            }
            i = j + 1
        } else if t.kind == 7 {
            let name = toks[i + 1]
            let stop = find_semi(toks, i + 3, n)
            let val = eval_expr(&toks, &fns, &vars, src, i + 3, stop)
            vars.push(Var { nstart: name.nstart, nlen: name.nlen, value: val })
            i = stop + 1
        } else if t.kind == 8 {
            let stop = find_semi(toks, i + 1, n)
            result = eval_value(&toks, &fns, &vars, src, i + 1, stop)
            i = stop + 1
        } else {
            i = i + 1
        }
    }
    return result
}

fn main() -> Int {
    # FP3b: a multi-char RECURSIVE function. factorial(n) = if n < 2 then 1
    # else n * factorial(n - 1); factorial(5) = 120. Real name + recursion.
    let value = run_program("fn factorial(n) {{ return if n < 2 then 1 else n * factorial(n - 1) }}; return factorial(5);")
    return emit_ir(value)
}
