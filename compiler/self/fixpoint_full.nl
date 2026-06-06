# expect: 0
# nl self-host — fixpoint FULL code generator: FUNCTIONS with IMPERATIVE BODIES.
#
# This is the merge most directly on the self-compile path: the nl compiler's own
# functions are `fn name(param) { let mut ...; while ...; if ...; return ... }`.
# Generates real LLVM IR where each function is a `define i64 @name(i64 %p)` with
# the param copied to a local alloca, body locals alloca'd, the imperative body
# (let/assign/while/if/return) emitted via gen_stmts, and calls `name(arg)` as
# `call` instructions.
#
# Requires the Vais fixes: `&Vec` borrow recursion (214c97cf) + literal-`%`
# escaping (e711dac1).

# Token kinds: 0=num,1=ident,2='+',3='*',4='-',5='=',6=';',7=let,8=return,21=mut,
#              18='<',19='>',20='==',22=while,11='{',12='}',9='(',10=')',13=fn,
#              15=if,17=else,25=','
struct Token { kind: Int, value: Int, nstart: Int, nlen: Int }
# Operand: kind 0=literal(val), 1=temp(%t<val>). next = next free SSA temp.
struct Op { kind: Int, val: Int, next: Int }
# A declared variable: source name range -> it lives at alloca %v<slot>.
struct Slot { nstart: Int, nlen: Int, slot: Int, is_arr: Int, alen: Int, sty: Int }
# A function: name range, param range, body token range [bstart, bend).
# A function: name range, up to 4 params (p0s/p0l..p3s/p3l), param count `npar`,
# and the body token range [bstart, bend).
struct Fn {
    nstart: Int, nlen: Int,
    p0s: Int, p0l: Int, p1s: Int, p1l: Int, p2s: Int, p2l: Int, p3s: Int, p3l: Int,
    npar: Int,
    bstart: Int, bend: Int
}
# A struct type: name range + up to 6 field name ranges + field count.
struct StructDef {
    nstart: Int, nlen: Int,
    f0s: Int, f0l: Int, f1s: Int, f1l: Int, f2s: Int, f2l: Int,
    f3s: Int, f3l: Int, f4s: Int, f4l: Int, f5s: Int, f5l: Int,
    nfields: Int
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
fn kw2(src: Str, a: Int, alen: Int, w0: Int, w1: Int) -> Int {
    if alen != 2 { return 0 }
    if src[a] != w0 { return 0 }
    if src[a + 1] != w1 { return 0 }
    return 1
}
fn kw3(src: Str, a: Int, alen: Int, w0: Int, w1: Int, w2: Int) -> Int {
    if alen != 3 { return 0 }
    if src[a] != w0 { return 0 }
    if src[a + 1] != w1 { return 0 }
    if src[a + 2] != w2 { return 0 }
    return 1
}
fn kw4(src: Str, a: Int, alen: Int, w0: Int, w1: Int, w2: Int, w3: Int) -> Int {
    if alen != 4 { return 0 }
    if src[a] != w0 { return 0 }
    if src[a + 1] != w1 { return 0 }
    if src[a + 2] != w2 { return 0 }
    if src[a + 3] != w3 { return 0 }
    return 1
}
fn kw5(src: Str, a: Int, alen: Int, w0: Int, w1: Int, w2: Int, w3: Int, w4: Int) -> Int {
    if alen != 5 { return 0 }
    if src[a] != w0 { return 0 }
    if src[a + 1] != w1 { return 0 }
    if src[a + 2] != w2 { return 0 }
    if src[a + 3] != w3 { return 0 }
    if src[a + 4] != w4 { return 0 }
    return 1
}
fn kw6(src: Str, a: Int, alen: Int, w0: Int, w1: Int, w2: Int, w3: Int, w4: Int, w5: Int) -> Int {
    if alen != 6 { return 0 }
    if src[a] != w0 { return 0 }
    if src[a + 1] != w1 { return 0 }
    if src[a + 2] != w2 { return 0 }
    if src[a + 3] != w3 { return 0 }
    if src[a + 4] != w4 { return 0 }
    if src[a + 5] != w5 { return 0 }
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
            if kw3(src, start, len, 108, 101, 116) == 1 {
                toks.push(Token { kind: 7, value: 0, nstart: start, nlen: len })
            } else if kw6(src, start, len, 114, 101, 116, 117, 114, 110) == 1 {
                toks.push(Token { kind: 8, value: 0, nstart: start, nlen: len })
            } else if kw3(src, start, len, 109, 117, 116) == 1 {
                toks.push(Token { kind: 21, value: 0, nstart: start, nlen: len })
            } else if kw5(src, start, len, 119, 104, 105, 108, 101) == 1 {
                toks.push(Token { kind: 22, value: 0, nstart: start, nlen: len })   # while
            } else if kw2(src, start, len, 105, 102) == 1 {
                toks.push(Token { kind: 15, value: 0, nstart: start, nlen: len })   # if
            } else if kw4(src, start, len, 101, 108, 115, 101) == 1 {
                toks.push(Token { kind: 17, value: 0, nstart: start, nlen: len })   # else
            } else if kw2(src, start, len, 102, 110) == 1 {
                toks.push(Token { kind: 13, value: 0, nstart: start, nlen: len })   # fn
            } else if kw6(src, start, len, 115, 116, 114, 117, 99, 116) == 1 {
                toks.push(Token { kind: 26, value: 0, nstart: start, nlen: len })   # struct
            } else {
                toks.push(Token { kind: 1, value: 0, nstart: start, nlen: len })
            }
        } else if c == 43 { toks.push(Token { kind: 2, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 42 { toks.push(Token { kind: 3, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 45 { toks.push(Token { kind: 4, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 40 { toks.push(Token { kind: 9, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 41 { toks.push(Token { kind: 10, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 60 { toks.push(Token { kind: 18, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 62 { toks.push(Token { kind: 19, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 61 {
            if i + 1 < n {
                if src[i + 1] == 61 {
                    toks.push(Token { kind: 20, value: 0, nstart: 0, nlen: 0 }); i = i + 2
                } else {
                    toks.push(Token { kind: 5, value: 0, nstart: 0, nlen: 0 }); i = i + 1
                }
            } else {
                toks.push(Token { kind: 5, value: 0, nstart: 0, nlen: 0 }); i = i + 1
            }
        }
        else if c == 123 { toks.push(Token { kind: 11, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 125 { toks.push(Token { kind: 12, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 91 { toks.push(Token { kind: 23, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 93 { toks.push(Token { kind: 24, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 44 { toks.push(Token { kind: 25, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 46 { toks.push(Token { kind: 27, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 59 { toks.push(Token { kind: 6, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else { i = i + 1 }
    }
    return toks
}

fn name_eq(src: Str, a: Int, alen: Int, b: Int, blen: Int) -> Int {
    if alen != blen { return 0 }
    let mut k = 0
    while k < alen {
        if src[a + k] == src[b + k] { k = k + 1 } else { return 0 }
    }
    return 1
}
fn emit_str(s: Str) -> Int {
    let n = s.len()
    let mut k = 0
    while k < n {
        putchar(s[k])
        k = k + 1
    }
    return 0
}
fn pint(x: Int) -> Int {
    if x < 0 { putchar(45); return pint(0 - x) }
    if x >= 10 { pint(x / 10) }
    putchar(48 + (x - (x / 10) * 10))
    return 0
}

# Look up a variable's alloca slot number by name; -1 if absent.
fn find_slot(slots: &List<Slot>, src: Str, qs: Int, ql: Int) -> Int {
    let m = slots.len()
    let mut i = 0
    while i < m {
        let s = slots[i]
        if name_eq(src, s.nstart, s.nlen, qs, ql) == 1 { return s.slot }
        i = i + 1
    }
    return 0 - 1
}
fn arrlen_of(slots: &List<Slot>, src: Str, qs: Int, ql: Int) -> Int {
    let m = slots.len()
    let mut i = 0
    while i < m {
        let s = slots[i]
        if name_eq(src, s.nstart, s.nlen, qs, ql) == 1 { return s.alen }
        i = i + 1
    }
    return 0
}
# Find the end of an array-literal element: next ',' or the closing ']' (bend).
fn arr_elem_end(toks: &List<Token>, i: Int, bend: Int) -> Int {
    let mut j = i
    let mut go = true
    while go {
        if j >= bend { go = false }
        else {
            let t = toks[j]
            if t.kind == 25 { go = false } else { j = j + 1 }
        }
    }
    return j
}
# Index just past the matching ']' for the '[' whose contents start at op2.
fn bracket_end(toks: &List<Token>, op2: Int) -> Int {
    let mut j = op2
    let mut depth = 0
    let mut go = true
    while go {
        let t = toks[j]
        if t.kind == 23 { depth = depth + 1; j = j + 1 }
        else if t.kind == 24 {
            if depth == 0 { go = false } else { depth = depth - 1; j = j + 1 }
        }
        else { j = j + 1 }
    }
    return j
}

# Emit a source name verbatim (for @name LLVM identifiers).
fn emit_name(src: Str, start: Int, len: Int) -> Int {
    let mut k = 0
    while k < len {
        putchar(src[start + k])
        k = k + 1
    }
    return 0
}

# Find a function index by name; -1 if absent.
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

# Build the function table by scanning for `fn name ( param ) { ... }`.
fn build_fns(toks: &List<Token>, n: Int) -> List<Fn> {
    let mut fns: List<Fn> = []
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 13 {
            let nt = toks[i + 1]
            # i+2 open-paren; params (idents, comma-separated) until close-paren; then body.
            let mut p0s = 0
            let mut p0l = 0
            let mut p1s = 0
            let mut p1l = 0
            let mut p2s = 0
            let mut p2l = 0
            let mut p3s = 0
            let mut p3l = 0
            let mut npar = 0
            let mut q = i + 3
            let mut gp = true
            while gp {
                let qt = toks[q]
                if qt.kind == 10 { gp = false }
                else if qt.kind == 1 {
                    if npar == 0 { p0s = qt.nstart; p0l = qt.nlen }
                    else if npar == 1 { p1s = qt.nstart; p1l = qt.nlen }
                    else if npar == 2 { p2s = qt.nstart; p2l = qt.nlen }
                    else { p3s = qt.nstart; p3l = qt.nlen }
                    npar = npar + 1
                    q = q + 1
                }
                else { q = q + 1 }
            }
            # q is close-paren; find the open-brace after it
            let mut bo = q + 1
            let mut gb = true
            while gb {
                let bt = toks[bo]
                if bt.kind == 11 { gb = false } else { bo = bo + 1 }
            }
            let bstart = bo + 1
            let mut be = bstart
            let mut depth = 1
            let mut go = true
            while go {
                if be >= n {
                    go = false
                } else {
                    let bt = toks[be]
                    if bt.kind == 11 { depth = depth + 1; be = be + 1 }
                    else if bt.kind == 12 {
                        depth = depth - 1
                        if depth == 0 { go = false } else { be = be + 1 }
                    }
                    else { be = be + 1 }
                }
            }
            fns.push(Fn {
                nstart: nt.nstart, nlen: nt.nlen,
                p0s: p0s, p0l: p0l, p1s: p1s, p1l: p1l, p2s: p2s, p2l: p2l, p3s: p3s, p3l: p3l,
                npar: npar,
                bstart: bstart, bend: be
            })
            i = be + 1
        } else {
            i = i + 1
        }
    }
    return fns
}

# Skip a `struct Name { ... }` declaration at `i` (the `struct` token); returns
# index past the closing brace.
fn skip_struct_def(toks: &List<Token>, i: Int, n: Int) -> Int {
    let mut b = i + 1
    let mut g1 = true
    while g1 {
        if b >= n { g1 = false }
        else {
            let bt = toks[b]
            if bt.kind == 11 { g1 = false } else { b = b + 1 }
        }
    }
    return match_brace(toks, b, n) + 1
}

# Build the struct-type table from `struct Name { f0, f1, ... }` declarations.
fn build_defs(toks: &List<Token>, n: Int) -> List<StructDef> {
    let mut defs: List<StructDef> = []
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 26 {
            let nt = toks[i + 1]
            let bopen = i + 2
            let bclose = match_brace(toks, bopen, n)
            let mut f0s = 0
            let mut f0l = 0
            let mut f1s = 0
            let mut f1l = 0
            let mut f2s = 0
            let mut f2l = 0
            let mut f3s = 0
            let mut f3l = 0
            let mut f4s = 0
            let mut f4l = 0
            let mut f5s = 0
            let mut f5l = 0
            let mut cnt = 0
            let mut q = bopen + 1
            while q < bclose {
                let qt = toks[q]
                if qt.kind == 1 {
                    if cnt == 0 { f0s = qt.nstart; f0l = qt.nlen }
                    else if cnt == 1 { f1s = qt.nstart; f1l = qt.nlen }
                    else if cnt == 2 { f2s = qt.nstart; f2l = qt.nlen }
                    else if cnt == 3 { f3s = qt.nstart; f3l = qt.nlen }
                    else if cnt == 4 { f4s = qt.nstart; f4l = qt.nlen }
                    else { f5s = qt.nstart; f5l = qt.nlen }
                    cnt = cnt + 1
                }
                q = q + 1
            }
            defs.push(StructDef {
                nstart: nt.nstart, nlen: nt.nlen,
                f0s: f0s, f0l: f0l, f1s: f1s, f1l: f1l, f2s: f2s, f2l: f2l,
                f3s: f3s, f3l: f3l, f4s: f4s, f4l: f4l, f5s: f5s, f5l: f5l,
                nfields: cnt
            })
            i = bclose + 1
        } else {
            i = i + 1
        }
    }
    return defs
}
fn struct_index_by_name(defs: &List<StructDef>, src: Str, qs: Int, ql: Int) -> Int {
    let m = defs.len()
    let mut i = 0
    while i < m {
        let d = defs[i]
        if name_eq(src, d.nstart, d.nlen, qs, ql) == 1 { return i }
        i = i + 1
    }
    return 0 - 1
}
fn struct_nfields(defs: &List<StructDef>, ti: Int) -> Int {
    let d = defs[ti]
    return d.nfields
}
fn field_index(defs: &List<StructDef>, ti: Int, src: Str, qs: Int, ql: Int) -> Int {
    let d = defs[ti]
    if d.nfields >= 1 { if name_eq(src, d.f0s, d.f0l, qs, ql) == 1 { return 0 } }
    if d.nfields >= 2 { if name_eq(src, d.f1s, d.f1l, qs, ql) == 1 { return 1 } }
    if d.nfields >= 3 { if name_eq(src, d.f2s, d.f2l, qs, ql) == 1 { return 2 } }
    if d.nfields >= 4 { if name_eq(src, d.f3s, d.f3l, qs, ql) == 1 { return 3 } }
    if d.nfields >= 5 { if name_eq(src, d.f4s, d.f4l, qs, ql) == 1 { return 4 } }
    if d.nfields >= 6 { if name_eq(src, d.f5s, d.f5l, qs, ql) == 1 { return 5 } }
    return 0 - 1
}
# Slot's struct-type index for a variable (-1 if not a struct).
fn sty_of(slots: &List<Slot>, src: Str, qs: Int, ql: Int) -> Int {
    let m = slots.len()
    let mut i = 0
    while i < m {
        let s = slots[i]
        if name_eq(src, s.nstart, s.nlen, qs, ql) == 1 { return s.sty }
        i = i + 1
    }
    return 0 - 1
}

# Print an operand inline.
fn emit_op(o: Op) -> Int {
    if o.kind == 0 { pint(o.val) }
    else { putchar(37); putchar(116); pint(o.val) }
    return 0
}

# A factor: number, or a variable (emit a load from its alloca -> a temp).
fn gen_factor(toks: &List<Token>, slots: &List<Slot>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, i: Int, counter: Int) -> Op {
    let t = toks[i]
    if t.kind == 0 { return Op { kind: 0, val: t.value, next: counter } }
    if t.kind == 1 {
        let nx = toks[i + 1]
        if nx.kind == 9 {
            # call: name ( arg0 [, arg1 [, arg2 [, arg3]]] ) — 0..4 args.
            let close = paren_end(toks, i + 2)
            # evaluate each argument (between commas at depth 0), capturing its op.
            let mut nargs = 0
            let mut a0k = 0
            let mut a0v = 0
            let mut a1k = 0
            let mut a1v = 0
            let mut a2k = 0
            let mut a2v = 0
            let mut a3k = 0
            let mut a3v = 0
            let mut cc = counter
            let mut q = i + 2
            let mut ga = true
            while ga {
                if q >= close { ga = false }
                else {
                    let astop = arg_comma_end(toks, q, close)
                    let e = gen_expr(toks, slots, fns, defs, src, q, astop, cc)
                    cc = e.next
                    if nargs == 0 { a0k = e.kind; a0v = e.val }
                    else if nargs == 1 { a1k = e.kind; a1v = e.val }
                    else if nargs == 2 { a2k = e.kind; a2v = e.val }
                    else { a3k = e.kind; a3v = e.val }
                    nargs = nargs + 1
                    q = astop + 1
                }
            }
            let dest = cc
            emit_str("  %t")
            pint(dest)
            emit_str(" = call i64 @")
            emit_name(src, t.nstart, t.nlen)
            emit_str("(")
            let mut ai = 0
            while ai < nargs {
                if ai > 0 { emit_str(", ") }
                emit_str("i64 ")
                let mut ak = a0k
                let mut av = a0v
                if ai == 1 { ak = a1k; av = a1v }
                else if ai == 2 { ak = a2k; av = a2v }
                else if ai == 3 { ak = a3k; av = a3v }
                emit_op(Op { kind: ak, val: av, next: 0 })
                ai = ai + 1
            }
            emit_str(")")
            putchar(10)
            return Op { kind: 1, val: dest, next: dest + 1 }
        }
        if nx.kind == 27 {
            # `name . X` — disambiguate by the variable's kind:
            #   struct (sty>=0) -> field read (GEP field-index + load)
            #   List           -> `.len` (load the length counter)
            let sti = sty_of(slots, src, t.nstart, t.nlen)
            if sti >= 0 {
                let fld = toks[i + 2]
                let slot = find_slot(slots, src, t.nstart, t.nlen)
                let nf = struct_nfields(defs, sti)
                let fi = field_index(defs, sti, src, fld.nstart, fld.nlen)
                emit_str("  %t")
                pint(counter)
                emit_str(" = getelementptr [")
                pint(nf)
                emit_str(" x i64], [")
                pint(nf)
                emit_str(" x i64]* %v")
                pint(slot)
                emit_str(", i64 0, i64 ")
                pint(fi)
                putchar(10)
                let loadc = counter + 1
                emit_str("  %t")
                pint(loadc)
                emit_str(" = load i64, i64* %t")
                pint(counter)
                putchar(10)
                return Op { kind: 1, val: loadc, next: loadc + 1 }
            }
            # list length: lst.len -> load the length counter at %v<slot+1>
            let slot = find_slot(slots, src, t.nstart, t.nlen)
            emit_str("  %t")
            pint(counter)
            emit_str(" = load i64, i64* %v")
            pint(slot + 1)
            putchar(10)
            return Op { kind: 1, val: counter, next: counter + 1 }
        }
        if nx.kind == 23 {
            # array/list index read: name [ <expr> ] -> GEP + load
            let slot = find_slot(slots, src, t.nstart, t.nlen)
            let alen = arrlen_of(slots, src, t.nstart, t.nlen)
            let bend = bracket_end(toks, i + 2)
            let idx = gen_expr(toks, slots, fns, defs, src, i + 2, bend, counter)
            let gepc = idx.next
            emit_str("  %t")
            pint(gepc)
            emit_str(" = getelementptr [")
            pint(alen)
            emit_str(" x i64], [")
            pint(alen)
            emit_str(" x i64]* %v")
            pint(slot)
            emit_str(", i64 0, i64 ")
            emit_op(idx)
            putchar(10)
            let loadc = gepc + 1
            emit_str("  %t")
            pint(loadc)
            emit_str(" = load i64, i64* %t")
            pint(gepc)
            putchar(10)
            return Op { kind: 1, val: loadc, next: loadc + 1 }
        }
        let slot = find_slot(slots, src, t.nstart, t.nlen)
        # emit `  %t<counter> = load i64, i64* %v<slot>`
        emit_str("  %t")
        pint(counter)
        emit_str(" = load i64, i64* %v")
        pint(slot)
        putchar(10)
        return Op { kind: 1, val: counter, next: counter + 1 }
    }
    return Op { kind: 0, val: 0, next: counter }
}

fn emit_binop(op_s: Str, l: Op, r: Op, counter: Int) -> Int {
    emit_str("  %t")
    pint(counter)
    emit_str(" = ")
    emit_str(op_s)
    emit_str(" i64 ")
    emit_op(l)
    emit_str(", ")
    emit_op(r)
    putchar(10)
    return counter
}

fn gen_term(toks: &List<Token>, slots: &List<Slot>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, i: Int, stop: Int, acc: Op) -> Op {
    if i >= stop { return acc }
    let t = toks[i]
    if t.kind == 3 {
        let rf = gen_factor(toks, slots, fns, defs, src, i + 1, acc.next)
        let dest = emit_binop("mul", acc, rf, rf.next)
        let nacc = Op { kind: 1, val: dest, next: dest + 1 }
        let after = skip_factor(toks, i + 1)
        return gen_term(toks, slots, fns, defs, src, after, stop, nacc)
    }
    return acc
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
fn gen_expr(toks: &List<Token>, slots: &List<Slot>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, i: Int, stop: Int, counter: Int) -> Op {
    let f0 = gen_factor(toks, slots, fns, defs, src, i, counter)
    let af = skip_factor(toks, i)
    let t0 = gen_term(toks, slots, fns, defs, src, af, stop, f0)
    let after = skip_term(toks, af, stop)
    return gen_fold(toks, slots, fns, defs, src, after, stop, t0)
}
# Index just past the matching ')' for the '(' at index `op2` (args start). `op2`
# is the token after '('. Returns the ')' index.
fn paren_end(toks: &List<Token>, op2: Int) -> Int {
    let mut j = op2
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
# Find the end of a call argument: next ',' or ')' at paren-depth 0 (so nested
# calls' commas/parens are skipped). `close` bounds the scan (the call's ')').
fn arg_comma_end(toks: &List<Token>, i: Int, close: Int) -> Int {
    let mut j = i
    let mut depth = 0
    let mut go = true
    while go {
        if j >= close { go = false }
        else {
            let t = toks[j]
            if t.kind == 9 { depth = depth + 1; j = j + 1 }
            else if t.kind == 10 {
                if depth == 0 { go = false } else { depth = depth - 1; j = j + 1 }
            }
            else if t.kind == 25 {
                if depth == 0 { go = false } else { j = j + 1 }
            }
            else { j = j + 1 }
        }
    }
    return j
}
# Advance past one factor: number/var = 1 token; call = name '(' ... ')';
# array index = name '[' ... ']'.
fn skip_factor(toks: &List<Token>, i: Int) -> Int {
    let t = toks[i]
    if t.kind == 1 {
        let nx = toks[i + 1]
        if nx.kind == 9 {
            return paren_end(toks, i + 2) + 1
        }
        if nx.kind == 23 {
            return bracket_end(toks, i + 2) + 1
        }
        if nx.kind == 27 {
            return i + 3
        }
    }
    return i + 1
}
fn gen_fold(toks: &List<Token>, slots: &List<Slot>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, i: Int, stop: Int, acc: Op) -> Op {
    if i >= stop { return acc }
    let op = toks[i]
    if op.kind == 2 {
        let rf = gen_factor(toks, slots, fns, defs, src, i + 1, acc.next)
        let rt = gen_term(toks, slots, fns, defs, src, skip_factor(toks, i + 1), stop, rf)
        let dest = emit_binop("add", acc, rt, rt.next)
        let nacc = Op { kind: 1, val: dest, next: dest + 1 }
        return gen_fold(toks, slots, fns, defs, src, skip_term(toks, skip_factor(toks, i + 1), stop), stop, nacc)
    }
    if op.kind == 4 {
        let rf = gen_factor(toks, slots, fns, defs, src, i + 1, acc.next)
        let rt = gen_term(toks, slots, fns, defs, src, skip_factor(toks, i + 1), stop, rf)
        let dest = emit_binop("sub", acc, rt, rt.next)
        let nacc = Op { kind: 1, val: dest, next: dest + 1 }
        return gen_fold(toks, slots, fns, defs, src, skip_term(toks, skip_factor(toks, i + 1), stop), stop, nacc)
    }
    return acc
}

fn find_semi(toks: &List<Token>, i: Int, n: Int) -> Int {
    if i >= n { return n }
    let t = toks[i]
    if t.kind == 6 { return i }
    return find_semi(toks, i + 1, n)
}

# First pass: collect all declared variables (assign each an alloca slot) and
# emit their `alloca` at the top of @main. Returns the slot table.
# Collect `let`-declared variable slots in token range [start, end), each given
# an alloca. `slot0` is the first slot number to use (params occupy lower slots).
# Skips nested fn bodies' lets? No — ranges passed in are already a single
# function's body (or the top level minus fn defs), so all `let`s here are local.
fn collect_slots_range(toks: &List<Token>, src: Str, start: Int, end: Int, slot0: Int) -> List<Slot> {
    let mut slots: List<Slot> = []
    let mut next_slot = slot0
    let mut i = start
    while i < end {
        let t = toks[i]
        if t.kind == 7 {
            let nx = toks[i + 1]
            let mut npos = i + 1
            if nx.kind == 21 { npos = i + 2 }
            let name = toks[npos]
            slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 0, alen: 0 , sty: 0 - 1 })
            emit_str("  %v")
            pint(next_slot)
            emit_str(" = alloca i64")
            putchar(10)
            next_slot = next_slot + 1
        }
        i = i + 1
    }
    return slots
}
# Append the variable slots of [start,end) into an existing slot list (so a
# function's slot table can hold both its param and its body locals). Returns the
# combined list. (Re-emits allocas for the body locals.)
# Count elements of an array literal whose '[' contents start at `astart`
# (commas + 1). Assumes non-empty.
fn count_arr_elems(toks: &List<Token>, astart: Int) -> Int {
    let bend = bracket_end(toks, astart)
    let mut commas = 0
    let mut q = astart
    while q < bend {
        let qt = toks[q]
        if qt.kind == 25 { commas = commas + 1 }
        q = q + 1
    }
    return commas + 1
}

# Is the RHS at npos+2 a `list()` constructor? (ident 'list' then '(').
fn rhs_is_list(toks: &List<Token>, src: Str, npos: Int) -> Int {
    let rhs = toks[npos + 2]
    if rhs.kind == 1 {
        if kw4(src, rhs.nstart, rhs.nlen, 108, 105, 115, 116) == 1 {
            let after = toks[npos + 3]
            if after.kind == 9 { return 1 }
        }
    }
    return 0
}
# If `let name = Name { ... }`, returns the struct-type index of Name, else -1.
fn rhs_struct_type(toks: &List<Token>, defs: &List<StructDef>, src: Str, npos: Int) -> Int {
    let rhs = toks[npos + 2]
    if rhs.kind == 1 {
        let after = toks[npos + 3]
        if after.kind == 11 {
            return struct_index_by_name(defs, src, rhs.nstart, rhs.nlen)
        }
    }
    return 0 - 1
}

fn add_local_slots(base: List<Slot>, toks: &List<Token>, defs: &List<StructDef>, src: Str, start: Int, end: Int, slot0: Int) -> List<Slot> {
    let mut slots = base
    let mut next_slot = slot0
    let mut i = start
    while i < end {
        let t = toks[i]
        if t.kind == 7 {
            let nx = toks[i + 1]
            let mut npos = i + 1
            if nx.kind == 21 { npos = i + 2 }
            let name = toks[npos]
            let rhs = toks[npos + 2]
            let sti = rhs_struct_type(toks, defs, src, npos)
            if sti >= 0 {
                # struct var: alloca [nfields x i64], sty = type index
                let nf = struct_nfields(defs, sti)
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 0, alen: 0 , sty: sti })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [")
                pint(nf)
                emit_str(" x i64]")
                putchar(10)
                next_slot = next_slot + 1
            } else if rhs_is_list(toks, src, npos) == 1 {
                # List: buffer at %v<slot> [64 x i64], length at %v<slot+1>=0
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 2, alen: 64 , sty: 0 - 1 })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [64 x i64]")
                putchar(10)
                emit_str("  %v")
                pint(next_slot + 1)
                emit_str(" = alloca i64")
                putchar(10)
                emit_str("  store i64 0, i64* %v")
                pint(next_slot + 1)
                putchar(10)
                next_slot = next_slot + 2
            } else if rhs.kind == 23 {
                let alen = count_arr_elems(toks, npos + 3)
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 1, alen: alen , sty: 0 - 1 })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [")
                pint(alen)
                emit_str(" x i64]")
                putchar(10)
                next_slot = next_slot + 1
            } else {
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 0, alen: 0 , sty: 0 - 1 })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca i64")
                putchar(10)
                next_slot = next_slot + 1
            }
        }
        i = i + 1
    }
    return slots
}

# Find the matching '}' for the '{' at index `op` (handles nesting). Returns its
# index. `op` must be the open-brace token index.
fn match_brace(toks: &List<Token>, op: Int, n: Int) -> Int {
    let mut j = op + 1
    let mut depth = 1
    let mut go = true
    while go {
        if j >= n { go = false }
        else {
            let t = toks[j]
            if t.kind == 11 { depth = depth + 1; j = j + 1 }
            else if t.kind == 12 {
                depth = depth - 1
                if depth == 0 { go = false } else { j = j + 1 }
            }
            else { j = j + 1 }
        }
    }
    return j
}

# Generate code for the statements in token range [i, end). Returns the next free
# SSA temp. Handles `let`, assignment, `return`, and `while` (recursing for the
# loop body). Labels reuse temp numbers to stay unique.
fn gen_stmts(toks: &List<Token>, slots: &List<Slot>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, i0: Int, end: Int, counter0: Int) -> Int {
    let mut i = i0
    let mut counter = counter0
    while i < end {
        let t = toks[i]
        if t.kind == 7 {
            let nx = toks[i + 1]
            let mut npos = i + 1
            if nx.kind == 21 { npos = i + 2 }
            let name = toks[npos]
            let slot = find_slot(slots, src, name.nstart, name.nlen)
            let rhs = toks[npos + 2]
            let lsti = rhs_struct_type(toks, defs, src, npos)
            if lsti >= 0 {
                # struct literal: Name { f: v, ... } -> store each field via GEP.
                # open-brace at npos+3; fields from npos+4; field colon dropped by lexer so
                # value follows the field name directly.
                let nf = struct_nfields(defs, lsti)
                let bopen = npos + 3
                let bclose = match_brace(toks, bopen, end)
                let mut q = bopen + 1
                while q < bclose {
                    let fld = toks[q]
                    let fi = field_index(defs, lsti, src, fld.nstart, fld.nlen)
                    let vstart = q + 1
                    let vstop = arr_elem_end(toks, vstart, bclose)
                    let e = gen_expr(toks, slots, fns, defs, src, vstart, vstop, counter)
                    counter = e.next
                    emit_str("  %t")
                    pint(counter)
                    emit_str(" = getelementptr [")
                    pint(nf)
                    emit_str(" x i64], [")
                    pint(nf)
                    emit_str(" x i64]* %v")
                    pint(slot)
                    emit_str(", i64 0, i64 ")
                    pint(fi)
                    putchar(10)
                    emit_str("  store i64 ")
                    emit_op(e)
                    emit_str(", i64* %t")
                    pint(counter)
                    putchar(10)
                    counter = counter + 1
                    q = vstop + 1
                }
                let stop = find_semi(toks, bclose, end)
                i = stop + 1
            } else if rhs_is_list(toks, src, npos) == 1 {
                # let lst = list();  — alloca/len already emitted in collect; skip
                let stop = find_semi(toks, npos + 2, end)
                i = stop + 1
            } else if rhs.kind == 23 {
                # array literal: store each element via GEP
                let alen = arrlen_of(slots, src, name.nstart, name.nlen)
                let bend = bracket_end(toks, npos + 3)
                let mut q = npos + 3
                let mut idx = 0
                while q < bend {
                    let estop = arr_elem_end(toks, q, bend)
                    let e = gen_expr(toks, slots, fns, defs, src, q, estop, counter)
                    counter = e.next
                    emit_str("  %t")
                    pint(counter)
                    emit_str(" = getelementptr [")
                    pint(alen)
                    emit_str(" x i64], [")
                    pint(alen)
                    emit_str(" x i64]* %v")
                    pint(slot)
                    emit_str(", i64 0, i64 ")
                    pint(idx)
                    putchar(10)
                    emit_str("  store i64 ")
                    emit_op(e)
                    emit_str(", i64* %t")
                    pint(counter)
                    putchar(10)
                    counter = counter + 1
                    idx = idx + 1
                    q = estop + 1
                }
                let stop = find_semi(toks, bend, end)
                i = stop + 1
            } else {
                let stop = find_semi(toks, npos + 2, end)
                let e = gen_expr(toks, slots, fns, defs, src, npos + 2, stop, counter)
                emit_str("  store i64 ")
                emit_op(e)
                emit_str(", i64* %v")
                pint(slot)
                putchar(10)
                counter = e.next
                i = stop + 1
            }
        } else if t.kind == 1 {
            let nx = toks[i + 1]
            let asti = sty_of(slots, src, t.nstart, t.nlen)
            if nx.kind == 27 {
              if asti >= 0 {
                # struct field write: p . field = expr ;
                let fld = toks[i + 2]
                let slot = find_slot(slots, src, t.nstart, t.nlen)
                let nf = struct_nfields(defs, asti)
                let fi = field_index(defs, asti, src, fld.nstart, fld.nlen)
                # '=' at i+3; value from i+4
                let stop = find_semi(toks, i + 4, end)
                let e = gen_expr(toks, slots, fns, defs, src, i + 4, stop, counter)
                counter = e.next
                emit_str("  %t")
                pint(counter)
                emit_str(" = getelementptr [")
                pint(nf)
                emit_str(" x i64], [")
                pint(nf)
                emit_str(" x i64]* %v")
                pint(slot)
                emit_str(", i64 0, i64 ")
                pint(fi)
                putchar(10)
                emit_str("  store i64 ")
                emit_op(e)
                emit_str(", i64* %t")
                pint(counter)
                putchar(10)
                counter = counter + 1
                i = stop + 1
              } else {
                # list push: lst.push(expr) ;  — store at buf[len]; len = len + 1
                let slot = find_slot(slots, src, t.nstart, t.nlen)
                # method name at i+2, '(' at i+3, arg from i+4
                let argstop = paren_end(toks, i + 4)
                let e = gen_expr(toks, slots, fns, defs, src, i + 4, argstop, counter)
                counter = e.next
                emit_str("  %t")
                pint(counter)
                emit_str(" = load i64, i64* %v")
                pint(slot + 1)
                putchar(10)
                let lenc = counter
                counter = counter + 1
                emit_str("  %t")
                pint(counter)
                emit_str(" = getelementptr [64 x i64], [64 x i64]* %v")
                pint(slot)
                emit_str(", i64 0, i64 %t")
                pint(lenc)
                putchar(10)
                let gepc = counter
                counter = counter + 1
                emit_str("  store i64 ")
                emit_op(e)
                emit_str(", i64* %t")
                pint(gepc)
                putchar(10)
                emit_str("  %t")
                pint(counter)
                emit_str(" = add i64 %t")
                pint(lenc)
                emit_str(", 1")
                putchar(10)
                let incc = counter
                counter = counter + 1
                emit_str("  store i64 %t")
                pint(incc)
                emit_str(", i64* %v")
                pint(slot + 1)
                putchar(10)
                let stop = find_semi(toks, argstop, end)
                i = stop + 1
              }
            } else if nx.kind == 23 {
                # array element write: name [ idx ] = expr ;
                let slot = find_slot(slots, src, t.nstart, t.nlen)
                let alen = arrlen_of(slots, src, t.nstart, t.nlen)
                let bend = bracket_end(toks, i + 2)
                let idx = gen_expr(toks, slots, fns, defs, src, i + 2, bend, counter)
                counter = idx.next
                let stop = find_semi(toks, bend + 2, end)
                let val = gen_expr(toks, slots, fns, defs, src, bend + 2, stop, counter)
                counter = val.next
                emit_str("  %t")
                pint(counter)
                emit_str(" = getelementptr [")
                pint(alen)
                emit_str(" x i64], [")
                pint(alen)
                emit_str(" x i64]* %v")
                pint(slot)
                emit_str(", i64 0, i64 ")
                emit_op(idx)
                putchar(10)
                emit_str("  store i64 ")
                emit_op(val)
                emit_str(", i64* %t")
                pint(counter)
                putchar(10)
                counter = counter + 1
                i = stop + 1
            } else if nx.kind == 5 {
                let slot = find_slot(slots, src, t.nstart, t.nlen)
                let stop = find_semi(toks, i + 2, end)
                let e = gen_expr(toks, slots, fns, defs, src, i + 2, stop, counter)
                emit_str("  store i64 ")
                emit_op(e)
                emit_str(", i64* %v")
                pint(slot)
                putchar(10)
                counter = e.next
                i = stop + 1
            } else {
                i = i + 1
            }
        } else if t.kind == 8 {
            let stop = find_semi(toks, i + 1, end)
            let e = gen_expr(toks, slots, fns, defs, src, i + 1, stop, counter)
            emit_str("  ret i64 ")
            emit_op(e)
            putchar(10)
            counter = e.next
            i = stop + 1
        } else if t.kind == 22 {
            # while <lhs> <cmp> <rhs> { <body> }
            # find the open-brace after the condition
            let mut bopen = i + 1
            let mut g1 = true
            while g1 {
                if bopen >= end { g1 = false }
                else {
                    let bt = toks[bopen]
                    if bt.kind == 11 { g1 = false } else { bopen = bopen + 1 }
                }
            }
            let bclose = match_brace(toks, bopen, end)
            # condition is [i+1, bopen); find comparison operator
            let cstart = i + 1
            let cend = bopen
            let mut oppos = cend
            let mut pred_lt = 0
            let mut pred_gt = 0
            let mut pred_eq = 0
            let mut q = cstart
            let mut g2 = true
            while g2 {
                if q >= cend { g2 = false }
                else {
                    let qt = toks[q]
                    if qt.kind == 18 { oppos = q; pred_lt = 1; g2 = false }
                    else if qt.kind == 19 { oppos = q; pred_gt = 1; g2 = false }
                    else if qt.kind == 20 { oppos = q; pred_eq = 1; g2 = false }
                    else { q = q + 1 }
                }
            }
            # label numbers from current counter; reserve 3 (loop/body/done)
            let lbl = counter
            counter = counter + 1
            emit_str("  br label %loop")
            pint(lbl)
            putchar(10)
            emit_str("loop")
            pint(lbl)
            emit_str(":")
            putchar(10)
            let lhs = gen_expr(toks, slots, fns, defs, src, cstart, oppos, counter)
            let rhs = gen_expr(toks, slots, fns, defs, src, oppos + 1, cend, lhs.next)
            let cnum = rhs.next
            emit_str("  %t")
            pint(cnum)
            emit_str(" = icmp ")
            if pred_lt == 1 { emit_str("slt") } else if pred_gt == 1 { emit_str("sgt") } else { emit_str("eq") }
            emit_str(" i64 ")
            emit_op(lhs)
            emit_str(", ")
            emit_op(rhs)
            putchar(10)
            emit_str("  br i1 %t")
            pint(cnum)
            emit_str(", label %body")
            pint(lbl)
            emit_str(", label %done")
            pint(lbl)
            putchar(10)
            emit_str("body")
            pint(lbl)
            emit_str(":")
            putchar(10)
            # body statements are [bopen+1, bclose)
            counter = gen_stmts(toks, slots, fns, defs, src, bopen + 1, bclose, cnum + 1)
            emit_str("  br label %loop")
            pint(lbl)
            putchar(10)
            emit_str("done")
            pint(lbl)
            emit_str(":")
            putchar(10)
            i = bclose + 1
        } else if t.kind == 15 {
            # if <lhs> <cmp> <rhs> { <then> } [else { <else> }]  (statement form)
            let mut bopen = i + 1
            let mut g1 = true
            while g1 {
                if bopen >= end { g1 = false }
                else {
                    let bt = toks[bopen]
                    if bt.kind == 11 { g1 = false } else { bopen = bopen + 1 }
                }
            }
            let bclose = match_brace(toks, bopen, end)
            # condition [i+1, bopen)
            let cstart = i + 1
            let cend = bopen
            let mut oppos = cend
            let mut pred_lt = 0
            let mut pred_gt = 0
            let mut pred_eq = 0
            let mut q = cstart
            let mut g2 = true
            while g2 {
                if q >= cend { g2 = false }
                else {
                    let qt = toks[q]
                    if qt.kind == 18 { oppos = q; pred_lt = 1; g2 = false }
                    else if qt.kind == 19 { oppos = q; pred_gt = 1; g2 = false }
                    else if qt.kind == 20 { oppos = q; pred_eq = 1; g2 = false }
                    else { q = q + 1 }
                }
            }
            # is there an `else { ... }` after the then-block?
            let mut has_else = 0
            let mut eopen = bclose
            let mut eclose = bclose
            let after_then = bclose + 1
            if after_then < end {
                let et = toks[after_then]
                if et.kind == 17 {
                    has_else = 1
                    # find the open-brace after else
                    let mut eo = after_then + 1
                    let mut g3 = true
                    while g3 {
                        if eo >= end { g3 = false }
                        else {
                            let ot = toks[eo]
                            if ot.kind == 11 { g3 = false } else { eo = eo + 1 }
                        }
                    }
                    eopen = eo
                    eclose = match_brace(toks, eo, end)
                }
            }
            let lbl = counter
            counter = counter + 1
            let lhs = gen_expr(toks, slots, fns, defs, src, cstart, oppos, counter)
            let rhs = gen_expr(toks, slots, fns, defs, src, oppos + 1, cend, lhs.next)
            let cnum = rhs.next
            emit_str("  %t")
            pint(cnum)
            emit_str(" = icmp ")
            if pred_lt == 1 { emit_str("slt") } else if pred_gt == 1 { emit_str("sgt") } else { emit_str("eq") }
            emit_str(" i64 ")
            emit_op(lhs)
            emit_str(", ")
            emit_op(rhs)
            putchar(10)
            emit_str("  br i1 %t")
            pint(cnum)
            emit_str(", label %ithen")
            pint(lbl)
            emit_str(", label %ielse")
            pint(lbl)
            putchar(10)
            # then block
            emit_str("ithen")
            pint(lbl)
            emit_str(":")
            putchar(10)
            counter = gen_stmts(toks, slots, fns, defs, src, bopen + 1, bclose, cnum + 1)
            emit_str("  br label %imerge")
            pint(lbl)
            putchar(10)
            # else block (empty if no else)
            emit_str("ielse")
            pint(lbl)
            emit_str(":")
            putchar(10)
            if has_else == 1 {
                counter = gen_stmts(toks, slots, fns, defs, src, eopen + 1, eclose, counter)
            }
            emit_str("  br label %imerge")
            pint(lbl)
            putchar(10)
            emit_str("imerge")
            pint(lbl)
            emit_str(":")
            putchar(10)
            if has_else == 1 { i = eclose + 1 } else { i = bclose + 1 }
        } else {
            i = i + 1
        }
    }
    return counter
}

# Skip a fn definition starting at `i` (the `fn` token); returns index past close-brace.
fn skip_fn_def(toks: &List<Token>, i: Int, n: Int) -> Int {
    # find the body open-brace
    let mut b = i + 1
    let mut g1 = true
    while g1 {
        if b >= n { g1 = false }
        else {
            let bt = toks[b]
            if bt.kind == 11 { g1 = false } else { b = b + 1 }
        }
    }
    let bc = match_brace(toks, b, n)
    return bc + 1
}

# Collect top-level `let` slots (emitting allocas), skipping fn-def bodies.
fn collect_top_slots(toks: &List<Token>, defs: &List<StructDef>, src: Str, n: Int) -> List<Slot> {
    let mut slots: List<Slot> = []
    let mut next_slot = 0
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 13 {
            i = skip_fn_def(toks, i, n)
        } else if t.kind == 26 {
            i = skip_struct_def(toks, i, n)
        } else if t.kind == 7 {
            let nx = toks[i + 1]
            let mut npos = i + 1
            if nx.kind == 21 { npos = i + 2 }
            let name = toks[npos]
            let rhs = toks[npos + 2]
            let sti = rhs_struct_type(toks, defs, src, npos)
            if sti >= 0 {
                let nf = struct_nfields(defs, sti)
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 0, alen: 0 , sty: sti })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [")
                pint(nf)
                emit_str(" x i64]")
                putchar(10)
                next_slot = next_slot + 1
            } else if rhs_is_list(toks, src, npos) == 1 {
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 2, alen: 64 , sty: 0 - 1 })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [64 x i64]")
                putchar(10)
                emit_str("  %v")
                pint(next_slot + 1)
                emit_str(" = alloca i64")
                putchar(10)
                emit_str("  store i64 0, i64* %v")
                pint(next_slot + 1)
                putchar(10)
                next_slot = next_slot + 2
            } else if rhs.kind == 23 {
                let alen = count_arr_elems(toks, npos + 3)
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 1, alen: alen , sty: 0 - 1 })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [")
                pint(alen)
                emit_str(" x i64]")
                putchar(10)
                next_slot = next_slot + 1
            } else {
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 0, alen: 0 , sty: 0 - 1 })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca i64")
                putchar(10)
                next_slot = next_slot + 1
            }
            i = i + 1
        } else {
            i = i + 1
        }
    }
    return slots
}

# Generate top-level statements (skip fn defs), threading the symbol+fn tables.
fn gen_top(toks: &List<Token>, fns: &List<Fn>, defs: &List<StructDef>, slots: &List<Slot>, src: Str, n: Int) -> Int {
    let mut counter = 1
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 13 {
            i = skip_fn_def(toks, i, n)
        } else if t.kind == 26 {
            i = skip_struct_def(toks, i, n)
        } else {
            # delegate a top-level run between fn/struct defs to gen_stmts.
            let mut j = i
            let mut g = true
            while g {
                if j >= n { g = false }
                else {
                    let jt = toks[j]
                    if jt.kind == 13 { g = false }
                    else if jt.kind == 26 { g = false }
                    else { j = j + 1 }
                }
            }
            counter = gen_stmts(toks, slots, fns, defs, src, i, j, counter)
            i = j
        }
    }
    return counter
}

# Emit one user function: `define i64 @name(i64 %p_in) { <body> }`. The param is
# copied to alloca %v0; body locals occupy slots 1+. Body = [bstart, bend).
fn emit_fn(toks: &List<Token>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, f: Fn) -> Int {
    emit_str("define i64 @")
    emit_name(src, f.nstart, f.nlen)
    emit_str("(")
    # incoming SSA params: %a0, %a1, ...
    let mut pi = 0
    while pi < f.npar {
        if pi > 0 { emit_str(", ") }
        emit_str("i64 %a")
        pint(pi)
        pi = pi + 1
    }
    emit_str(") {")
    putchar(10)
    # each param -> its own alloca slot %v0..%v<npar-1>, store %aN into it
    let mut slots: List<Slot> = []
    let mut s2 = 0
    while s2 < f.npar {
        let mut pns = f.p0s
        let mut pnl = f.p0l
        if s2 == 1 { pns = f.p1s; pnl = f.p1l }
        else if s2 == 2 { pns = f.p2s; pnl = f.p2l }
        else if s2 == 3 { pns = f.p3s; pnl = f.p3l }
        slots.push(Slot { nstart: pns, nlen: pnl, slot: s2, is_arr: 0, alen: 0 , sty: 0 - 1 })
        emit_str("  %v")
        pint(s2)
        emit_str(" = alloca i64")
        putchar(10)
        emit_str("  store i64 %a")
        pint(s2)
        emit_str(", i64* %v")
        pint(s2)
        putchar(10)
        s2 = s2 + 1
    }
    # body locals start at slot npar
    let allslots = add_local_slots(slots, toks, defs, src, f.bstart, f.bend, f.npar)
    let last = gen_stmts(toks, &allslots, fns, defs, src, f.bstart, f.bend, 1)
    emit_str("}")
    putchar(10)
    return 0
}

fn compile(src: Str) -> Int {
    let toks = tokenize(src)
    let n = toks.len()
    let fns = build_fns(&toks, n)
    let defs = build_defs(&toks, n)
    # emit each user function
    let m = fns.len()
    let mut fi = 0
    while fi < m {
        let f = fns[fi]
        emit_fn(&toks, &fns, &defs, src, f)
        fi = fi + 1
    }
    # emit @main from top-level statements (skip fn defs)
    emit_str("define i64 @main() {")
    putchar(10)
    let topslots = collect_top_slots(&toks, &defs, src, n)
    let last = gen_top(&toks, &fns, &defs, &topslots, src, n)
    emit_str("}")
    putchar(10)
    return 0
}

fn main() -> Int {
    # FP12: multi-param + zero-param functions (toward real nl source).
    # add3(a,b,c)=a+b+c; answer()=42; return add3(1,2,3) + answer() = 6 + 42 = 48.
    return compile("fn add3(a, b, c) {{ return a + b + c }}; fn answer() {{ return 42 }}; return add3(1, 2, 3) + answer();")
}
