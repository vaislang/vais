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
# A function: name range, up to 8 params (p0s/p0l..p7s/p7l + p0ty..p7ty), param
# count `npar`, and the body token range [bstart, bend). 8 params because the
# self-host core (gen_stmts/gen_expr/gen_fold/gen_term) takes 8.
struct Fn {
    nstart: Int, nlen: Int,
    p0s: Int, p0l: Int, p1s: Int, p1l: Int, p2s: Int, p2l: Int, p3s: Int, p3l: Int,
    p4s: Int, p4l: Int, p5s: Int, p5l: Int, p6s: Int, p6l: Int, p7s: Int, p7l: Int,
    p0ty: Int, p1ty: Int, p2ty: Int, p3ty: Int,
    p4ty: Int, p5ty: Int, p6ty: Int, p7ty: Int,
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
# Is src[a..a+alen] the builtin name "putchar" (p u t c h a r)?
fn is_putchar(src: Str, a: Int, alen: Int) -> Int {
    if alen != 7 { return 0 }
    if src[a] != 112 { return 0 }
    if src[a + 1] != 117 { return 0 }
    if src[a + 2] != 116 { return 0 }
    if src[a + 3] != 99 { return 0 }
    if src[a + 4] != 104 { return 0 }
    if src[a + 5] != 97 { return 0 }
    if src[a + 6] != 114 { return 0 }
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
        } else if c == 96 {
            # string literal delimited by backtick (96); kind 28 carries the
            # content source range (nstart/nlen) and length (value).
            let sstart = i + 1
            let mut j = sstart
            let mut sgo = true
            while sgo {
                if j >= n { sgo = false }
                else if src[j] == 96 { sgo = false }
                else { j = j + 1 }
            }
            toks.push(Token { kind: 28, value: j - sstart, nstart: sstart, nlen: j - sstart })
            i = j + 1
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
            } else if kw3(src, start, len, 97, 110, 100) == 1 {
                toks.push(Token { kind: 31, value: 0, nstart: start, nlen: len })   # and
            } else if kw2(src, start, len, 111, 114) == 1 {
                toks.push(Token { kind: 32, value: 0, nstart: start, nlen: len })   # or
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
        else if c == 60 {
            # `<` (18) or `<=` (29)
            if i + 1 < n {
                if src[i + 1] == 61 { toks.push(Token { kind: 29, value: 0, nstart: 0, nlen: 0 }); i = i + 2 }
                else { toks.push(Token { kind: 18, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
            } else { toks.push(Token { kind: 18, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        }
        else if c == 62 {
            # `>` (19) or `>=` (30)
            if i + 1 < n {
                if src[i + 1] == 61 { toks.push(Token { kind: 30, value: 0, nstart: 0, nlen: 0 }); i = i + 2 }
                else { toks.push(Token { kind: 19, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
            } else { toks.push(Token { kind: 19, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        }
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
        else if c == 58 { toks.push(Token { kind: 16, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 33 {
            # `!` — only `!=` (not-equal, kind 33) is supported; bare `!` is skipped.
            if i + 1 < n {
                if src[i + 1] == 61 { toks.push(Token { kind: 33, value: 0, nstart: 0, nlen: 0 }); i = i + 2 }
                else { i = i + 1 }
            } else { i = i + 1 }
        }
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
# Copy a source substring verbatim (string-literal content into a global init).
fn emit_bytes(src: Str, start: Int, len: Int) -> Int {
    let mut k = 0
    while k < len {
        putchar(src[start + k])
        k = k + 1
    }
    return 0
}
# Module pre-pass: emit a `@.s<nstart> = [len+1 x i8] c"..\00"` global for every
# string-literal token, keyed by the literal's source nstart (unique per literal).
fn emit_str_globals(toks: &List<Token>, src: Str, n: Int) -> Int {
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 28 {
            emit_str("@.s")
            pint(t.nstart)
            emit_str(" = private constant [")
            pint(t.value + 1)
            emit_str(" x i8] c\"")
            emit_bytes(src, t.nstart, t.nlen)
            emit_str("\\00\"")
            putchar(10)
        }
        i = i + 1
    }
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
# is_arr discriminator: 0=scalar, 1=array, 2=List, 3=string. -1 if absent.
fn isarr_of(slots: &List<Slot>, src: Str, qs: Int, ql: Int) -> Int {
    let m = slots.len()
    let mut i = 0
    while i < m {
        let s = slots[i]
        if name_eq(src, s.nstart, s.nlen, qs, ql) == 1 { return s.is_arr }
        i = i + 1
    }
    return 0 - 1
}
# For a string var (is_arr=3): the global key (the literal's source nstart),
# stored in the slot's sty field. The string global is named @.s<key>.
fn strkey_of(slots: &List<Slot>, src: Str, qs: Int, ql: Int) -> Int {
    let m = slots.len()
    let mut i = 0
    while i < m {
        let s = slots[i]
        if name_eq(src, s.nstart, s.nlen, qs, ql) == 1 { return s.sty }
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
fn build_fns(toks: &List<Token>, src: Str, n: Int) -> List<Fn> {
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
            let mut p4s = 0
            let mut p4l = 0
            let mut p5s = 0
            let mut p5l = 0
            let mut p6s = 0
            let mut p6l = 0
            let mut p7s = 0
            let mut p7l = 0
            let mut p0ty = 0
            let mut p1ty = 0
            let mut p2ty = 0
            let mut p3ty = 0
            let mut p4ty = 0
            let mut p5ty = 0
            let mut p6ty = 0
            let mut p7ty = 0
            let mut npar = 0
            let mut q = i + 3
            let mut gp = true
            while gp {
                let qt = toks[q]
                if qt.kind == 10 { gp = false }
                else if qt.kind == 16 {
                    # `:` -> the type of the most recent param follows. Types:
                    #   Str         -> 1  (string param, i8*)
                    #   List<...>   -> 2  (List param, buffer pointer; `&` already
                    #                      dropped by the tokenizer)
                    #   else        -> 0  (int)
                    let tyt = toks[q + 1]
                    let isstr = kw3(src, tyt.nstart, tyt.nlen, 83, 116, 114)
                    let islist = kw4(src, tyt.nstart, tyt.nlen, 76, 105, 115, 116)
                    let mut pty = isstr
                    if islist == 1 { pty = 2 }
                    # the type applies to the most recent param (index npar-1).
                    if npar == 1 { p0ty = pty }
                    else if npar == 2 { p1ty = pty }
                    else if npar == 3 { p2ty = pty }
                    else if npar == 4 { p3ty = pty }
                    else if npar == 5 { p4ty = pty }
                    else if npar == 6 { p5ty = pty }
                    else if npar == 7 { p6ty = pty }
                    else { p7ty = pty }
                    # advance past the type. List<...> spans to the closing `>`
                    # (kind 19); a plain type is a single ident.
                    if islist == 1 {
                        let mut tq = q + 2
                        let mut tg = true
                        while tg {
                            let tt = toks[tq]
                            if tt.kind == 19 { tg = false } else { tq = tq + 1 }
                        }
                        q = tq + 1
                    } else {
                        q = q + 2
                    }
                }
                else if qt.kind == 1 {
                    if npar == 0 { p0s = qt.nstart; p0l = qt.nlen }
                    else if npar == 1 { p1s = qt.nstart; p1l = qt.nlen }
                    else if npar == 2 { p2s = qt.nstart; p2l = qt.nlen }
                    else if npar == 3 { p3s = qt.nstart; p3l = qt.nlen }
                    else if npar == 4 { p4s = qt.nstart; p4l = qt.nlen }
                    else if npar == 5 { p5s = qt.nstart; p5l = qt.nlen }
                    else if npar == 6 { p6s = qt.nstart; p6l = qt.nlen }
                    else { p7s = qt.nstart; p7l = qt.nlen }
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
                p4s: p4s, p4l: p4l, p5s: p5s, p5l: p5l, p6s: p6s, p6l: p6l, p7s: p7s, p7l: p7l,
                p0ty: p0ty, p1ty: p1ty, p2ty: p2ty, p3ty: p3ty,
                p4ty: p4ty, p5ty: p5ty, p6ty: p6ty, p7ty: p7ty,
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

# Print an operand inline. kind 0 = literal int; kind 1 = i64 temp; kind 2 = i8*
# temp (string pointer) -- both temps print as %t<val>.
fn emit_op(o: Op) -> Int {
    if o.kind == 0 { pint(o.val) }
    else { putchar(37); putchar(116); pint(o.val) }
    return 0
}

# After a call that passed List-local slot `lslot` by pointer, copy the length the
# callee wrote into buf[63] back into the local's length slot (%v<lslot+1>), so the
# caller's xs.len / further passes see pushes. No-op if lslot < 0. Returns counter.
# lslot = the local List buffer slot, or -1 for none. lenidx = the index where
# the callee stored the updated length (63 for scalar Lists, 64*nf for
# List-of-structs), and bufsz = the buffer's array size ([bufsz x i64]). These
# must match the local List's allocation so the GEP type is well-typed.
fn sync_list_len(lslot: Int, lenidx: Int, bufsz: Int, counter: Int) -> Int {
    if lslot < 0 { return counter }
    let lp = counter
    emit_str("  %t")
    pint(lp)
    emit_str(" = getelementptr [")
    pint(bufsz)
    emit_str(" x i64], [")
    pint(bufsz)
    emit_str(" x i64]* %v")
    pint(lslot)
    emit_str(", i64 0, i64 ")
    pint(lenidx)
    putchar(10)
    let lv = lp + 1
    emit_str("  %t")
    pint(lv)
    emit_str(" = load i64, i64* %t")
    pint(lp)
    putchar(10)
    emit_str("  store i64 %t")
    pint(lv)
    emit_str(", i64* %v")
    pint(lslot + 1)
    putchar(10)
    return lv + 1
}

# A factor: number, or a variable (emit a load from its alloca -> a temp).
fn gen_factor(toks: &List<Token>, slots: &List<Slot>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, i: Int, counter: Int) -> Op {
    let t = toks[i]
    if t.kind == 0 { return Op { kind: 0, val: t.value, next: counter } }
    # boolean literals `true`/`false` are tokenized as identifiers (kind 1); treat
    # them as the integer constants 1/0 (Vais/nl bools are i64). Used by the common
    # self-host `let mut go = true; ... go = false` loop-flag pattern.
    if t.kind == 1 {
        if kw4(src, t.nstart, t.nlen, 116, 114, 117, 101) == 1 { return Op { kind: 0, val: 1, next: counter } }
        if kw5(src, t.nstart, t.nlen, 102, 97, 108, 115, 101) == 1 { return Op { kind: 0, val: 0, next: counter } }
    }
    if t.kind == 28 {
        # string literal as a value (e.g. a call argument): GEP the @.s<nstart>
        # global to an i8* temp. Op kind 2 = i8* pointer.
        emit_str("  %t")
        pint(counter)
        emit_str(" = getelementptr [")
        pint(t.value + 1)
        emit_str(" x i8], [")
        pint(t.value + 1)
        emit_str(" x i8]* @.s")
        pint(t.nstart)
        emit_str(", i64 0, i64 0")
        putchar(10)
        return Op { kind: 2, val: counter, next: counter + 1 }
    }
    if t.kind == 9 {
        # parenthesized group: `( <expr> )` -> evaluate the inner expression.
        let close = paren_end(toks, i + 1)
        return gen_expr(toks, slots, fns, defs, src, i + 1, close, counter)
    }
    if t.kind == 1 {
        let nx = toks[i + 1]
        if nx.kind == 9 {
            # call: name ( arg0 [, ... up to arg7] ) — 0..8 args.
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
            let mut a4k = 0
            let mut a4v = 0
            let mut a5k = 0
            let mut a5v = 0
            let mut a6k = 0
            let mut a6v = 0
            let mut a7k = 0
            let mut a7v = 0
            # slots of List-local args passed by pointer, to sync length AFTER the
            # call (the callee may have pushed). -1 = none. li/lb carry the length
            # index + buffer array-size per arg (scalar: 63/64; struct: 64*nf/64*nf+1).
            let mut ls0 = 0 - 1
            let mut ls1 = 0 - 1
            let mut ls2 = 0 - 1
            let mut ls3 = 0 - 1
            let mut li0 = 63
            let mut li1 = 63
            let mut li2 = 63
            let mut li3 = 63
            let mut lb0 = 64
            let mut lb1 = 64
            let mut lb2 = 64
            let mut lb3 = 64
            let mut cc = counter
            let mut q = i + 2
            let mut ga = true
            while ga {
                if q >= close { ga = false }
                else {
                    let astop = arg_comma_end(toks, q, close)
                    # List-local arg passed by pointer: a single ident that is a
                    # List local (is_arr=2). Write its length into buf[63], then
                    # pass the buffer base as i64* (Op kind 3). The callee reads
                    # length from [63] and indexes via GEP i64.
                    let argt = toks[q]
                    let mut ekind = 0
                    let mut eval2 = 0
                    let mut handled = 0
                    if argt.kind == 1 {
                        if astop == q + 1 {
                            let aarr = isarr_of(slots, src, argt.nstart, argt.nlen)
                            if aarr == 2 {
                                let lslot = find_slot(slots, src, argt.nstart, argt.nlen)
                                # length index + buffer size depend on element kind:
                                #   scalar List  -> length at [63], buffer [64 x i64]
                                #   List<struct> -> length at [64*nf], buffer [64*nf+1]
                                let alsty = sty_of(slots, src, argt.nstart, argt.nlen)
                                let mut alenidx = 63
                                let mut albufsz = 64
                                if alsty >= 0 {
                                    let anf = struct_nfields(defs, alsty)
                                    alenidx = 64 * anf
                                    albufsz = 64 * anf + 1
                                }
                                # load length (%v<slot+1>) and store to buf[lenidx]
                                let lc = cc
                                emit_str("  %t")
                                pint(lc)
                                emit_str(" = load i64, i64* %v")
                                pint(lslot + 1)
                                putchar(10)
                                let l63 = lc + 1
                                emit_str("  %t")
                                pint(l63)
                                emit_str(" = getelementptr [")
                                pint(albufsz)
                                emit_str(" x i64], [")
                                pint(albufsz)
                                emit_str(" x i64]* %v")
                                pint(lslot)
                                emit_str(", i64 0, i64 ")
                                pint(alenidx)
                                putchar(10)
                                emit_str("  store i64 %t")
                                pint(lc)
                                emit_str(", i64* %t")
                                pint(l63)
                                putchar(10)
                                # buffer base pointer
                                let bc = l63 + 1
                                emit_str("  %t")
                                pint(bc)
                                emit_str(" = getelementptr [")
                                pint(albufsz)
                                emit_str(" x i64], [")
                                pint(albufsz)
                                emit_str(" x i64]* %v")
                                pint(lslot)
                                emit_str(", i64 0, i64 0")
                                putchar(10)
                                ekind = 3
                                eval2 = bc
                                cc = bc + 1
                                # record this slot + its lenidx/bufsz to sync after the call
                                if nargs == 0 { ls0 = lslot; li0 = alenidx; lb0 = albufsz }
                                else if nargs == 1 { ls1 = lslot; li1 = alenidx; lb1 = albufsz }
                                else if nargs == 2 { ls2 = lslot; li2 = alenidx; lb2 = albufsz }
                                else if nargs == 3 { ls3 = lslot; li3 = alenidx; lb3 = albufsz }
                                handled = 1
                            }
                        }
                    }
                    if handled == 0 {
                        let e = gen_expr(toks, slots, fns, defs, src, q, astop, cc)
                        cc = e.next
                        ekind = e.kind
                        eval2 = e.val
                    }
                    if nargs == 0 { a0k = ekind; a0v = eval2 }
                    else if nargs == 1 { a1k = ekind; a1v = eval2 }
                    else if nargs == 2 { a2k = ekind; a2v = eval2 }
                    else if nargs == 3 { a3k = ekind; a3v = eval2 }
                    else if nargs == 4 { a4k = ekind; a4v = eval2 }
                    else if nargs == 5 { a5k = ekind; a5v = eval2 }
                    else if nargs == 6 { a6k = ekind; a6v = eval2 }
                    else { a7k = ekind; a7v = eval2 }
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
                let mut ak = a0k
                let mut av = a0v
                if ai == 1 { ak = a1k; av = a1v }
                else if ai == 2 { ak = a2k; av = a2v }
                else if ai == 3 { ak = a3k; av = a3v }
                else if ai == 4 { ak = a4k; av = a4v }
                else if ai == 5 { ak = a5k; av = a5v }
                else if ai == 6 { ak = a6k; av = a6v }
                else if ai == 7 { ak = a7k; av = a7v }
                # kind 2 = i8* (string pointer); else i64.
                if ak == 2 { emit_str("i8* ") } else if ak == 3 { emit_str("i64* ") } else { emit_str("i64 ") }
                emit_op(Op { kind: ak, val: av, next: 0 })
                ai = ai + 1
            }
            emit_str(")")
            putchar(10)
            # sync the length of any List-local args from buf[63] (the callee may
            # have pushed into them via the out-param). %v<slot+1> = load buf[63].
            let mut rc = dest + 1
            rc = sync_list_len(ls0, li0, lb0, rc)
            rc = sync_list_len(ls1, li1, lb1, rc)
            rc = sync_list_len(ls2, li2, lb2, rc)
            rc = sync_list_len(ls3, li3, lb3, rc)
            return Op { kind: 1, val: dest, next: rc }
        }
        if nx.kind == 27 {
            # `name . X` — disambiguate by the variable's kind:
            #   string (is_arr=3) -> `.len()` (compile-time length literal)
            #   struct (sty>=0)   -> field read (GEP field-index + load)
            #   List              -> `.len` (load the length counter)
            # NOTE: string check FIRST — a string slot stores the global key in
            # sty, which would otherwise be misread as a struct index.
            let karr = isarr_of(slots, src, t.nstart, t.nlen)
            if karr == 3 {
                let slen = arrlen_of(slots, src, t.nstart, t.nlen)
                if slen > 0 {
                    # local string literal: compile-time length.
                    return Op { kind: 0, val: slen, next: counter }
                }
                # string PARAMETER (length unknown): runtime strlen -- walk bytes
                # from the i8* base until the NUL terminator, counting.
                let sslot = find_slot(slots, src, t.nstart, t.nlen)
                let basec = counter
                emit_str("  %t")
                pint(basec)
                emit_str(" = load i8*, i8** %v")
                pint(sslot)
                putchar(10)
                # length counter alloca (reuse an SSA-numbered name via %slN)
                let lenslot = basec + 1
                emit_str("  %sl")
                pint(lenslot)
                emit_str(" = alloca i64")
                putchar(10)
                emit_str("  store i64 0, i64* %sl")
                pint(lenslot)
                putchar(10)
                emit_str("  br label %slL")
                pint(lenslot)
                putchar(10)
                emit_str("slL")
                pint(lenslot)
                emit_str(":")
                putchar(10)
                let ic = lenslot + 1
                emit_str("  %t")
                pint(ic)
                emit_str(" = load i64, i64* %sl")
                pint(lenslot)
                putchar(10)
                let gepc = ic + 1
                emit_str("  %t")
                pint(gepc)
                emit_str(" = getelementptr i8, i8* %t")
                pint(basec)
                emit_str(", i64 %t")
                pint(ic)
                putchar(10)
                let bc = gepc + 1
                emit_str("  %t")
                pint(bc)
                emit_str(" = load i8, i8* %t")
                pint(gepc)
                putchar(10)
                let cc2 = bc + 1
                emit_str("  %t")
                pint(cc2)
                emit_str(" = icmp eq i8 %t")
                pint(bc)
                emit_str(", 0")
                putchar(10)
                emit_str("  br i1 %t")
                pint(cc2)
                emit_str(", label %slD")
                pint(lenslot)
                emit_str(", label %slB")
                pint(lenslot)
                putchar(10)
                emit_str("slB")
                pint(lenslot)
                emit_str(":")
                putchar(10)
                let nc = cc2 + 1
                emit_str("  %t")
                pint(nc)
                emit_str(" = add i64 %t")
                pint(ic)
                emit_str(", 1")
                putchar(10)
                emit_str("  store i64 %t")
                pint(nc)
                emit_str(", i64* %sl")
                pint(lenslot)
                putchar(10)
                emit_str("  br label %slL")
                pint(lenslot)
                putchar(10)
                emit_str("slD")
                pint(lenslot)
                emit_str(":")
                putchar(10)
                let resc = nc + 1
                emit_str("  %t")
                pint(resc)
                emit_str(" = load i64, i64* %sl")
                pint(lenslot)
                putchar(10)
                return Op { kind: 1, val: resc, next: resc + 1 }
            }
            if karr == 4 {
                # List PARAMETER `.len`: load buffer ptr, GEP to the length slot,
                # load length. Scalar List: length at [63]. List-of-structs (sty>=0):
                # length at [64*nf] (past the strided data region).
                let pslot = find_slot(slots, src, t.nstart, t.nlen)
                let plsty = sty_of(slots, src, t.nstart, t.nlen)
                let mut plenidx2 = 63
                if plsty >= 0 { plenidx2 = 64 * struct_nfields(defs, plsty) }
                let bp = counter
                emit_str("  %t")
                pint(bp)
                emit_str(" = load i64*, i64** %v")
                pint(pslot)
                putchar(10)
                let lp = bp + 1
                emit_str("  %t")
                pint(lp)
                emit_str(" = getelementptr i64, i64* %t")
                pint(bp)
                emit_str(", i64 ")
                pint(plenidx2)
                putchar(10)
                let lv = lp + 1
                emit_str("  %t")
                pint(lv)
                emit_str(" = load i64, i64* %t")
                pint(lp)
                putchar(10)
                return Op { kind: 1, val: lv, next: lv + 1 }
            }
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
            # string byte index: s[<expr>] -> load i8* base, GEP i8, load i8, zext
            let karr = isarr_of(slots, src, t.nstart, t.nlen)
            if karr == 3 {
                let sslot = find_slot(slots, src, t.nstart, t.nlen)
                let sbend = bracket_end(toks, i + 2)
                let sidx = gen_expr(toks, slots, fns, defs, src, i + 2, sbend, counter)
                let basec = sidx.next
                emit_str("  %t")
                pint(basec)
                emit_str(" = load i8*, i8** %v")
                pint(sslot)
                putchar(10)
                let sgepc = basec + 1
                emit_str("  %t")
                pint(sgepc)
                emit_str(" = getelementptr i8, i8* %t")
                pint(basec)
                emit_str(", i64 ")
                emit_op(sidx)
                putchar(10)
                let sldc = sgepc + 1
                emit_str("  %t")
                pint(sldc)
                emit_str(" = load i8, i8* %t")
                pint(sgepc)
                putchar(10)
                let szc = sldc + 1
                emit_str("  %t")
                pint(szc)
                emit_str(" = zext i8 %t")
                pint(sldc)
                emit_str(" to i64")
                putchar(10)
                return Op { kind: 1, val: szc, next: szc + 1 }
            }
            if karr == 4 {
                let p4sty = sty_of(slots, src, t.nstart, t.nlen)
                if p4sty >= 0 {
                    # List-of-structs PARAMETER element field: out[<expr>].field ->
                    # load bufptr, load buf[idx*nf + field_index] (stride nf).
                    let p4bend = bracket_end(toks, i + 2)
                    let p4dot = toks[p4bend + 1]
                    if p4dot.kind == 27 {
                        let p4fld = toks[p4bend + 2]
                        let p4slot = find_slot(slots, src, t.nstart, t.nlen)
                        let p4nf = struct_nfields(defs, p4sty)
                        let p4fi = field_index(defs, p4sty, src, p4fld.nstart, p4fld.nlen)
                        let p4idx = gen_expr(toks, slots, fns, defs, src, i + 2, p4bend, counter)
                        let p4base = p4idx.next
                        emit_str("  %t")
                        pint(p4base)
                        emit_str(" = load i64*, i64** %v")
                        pint(p4slot)
                        putchar(10)
                        let p4mul = p4base + 1
                        emit_str("  %t")
                        pint(p4mul)
                        emit_str(" = mul i64 ")
                        emit_op(p4idx)
                        emit_str(", ")
                        pint(p4nf)
                        putchar(10)
                        let p4off = p4mul + 1
                        emit_str("  %t")
                        pint(p4off)
                        emit_str(" = add i64 %t")
                        pint(p4mul)
                        emit_str(", ")
                        pint(p4fi)
                        putchar(10)
                        let p4gep = p4off + 1
                        emit_str("  %t")
                        pint(p4gep)
                        emit_str(" = getelementptr i64, i64* %t")
                        pint(p4base)
                        emit_str(", i64 %t")
                        pint(p4off)
                        putchar(10)
                        let p4ld = p4gep + 1
                        emit_str("  %t")
                        pint(p4ld)
                        emit_str(" = load i64, i64* %t")
                        pint(p4gep)
                        putchar(10)
                        return Op { kind: 1, val: p4ld, next: p4ld + 1 }
                    }
                }
                # List PARAMETER index (scalar): load buffer ptr, GEP i64, load i64.
                let pslot = find_slot(slots, src, t.nstart, t.nlen)
                let pbend = bracket_end(toks, i + 2)
                let pidx = gen_expr(toks, slots, fns, defs, src, i + 2, pbend, counter)
                let pbase = pidx.next
                emit_str("  %t")
                pint(pbase)
                emit_str(" = load i64*, i64** %v")
                pint(pslot)
                putchar(10)
                let pgep = pbase + 1
                emit_str("  %t")
                pint(pgep)
                emit_str(" = getelementptr i64, i64* %t")
                pint(pbase)
                emit_str(", i64 ")
                emit_op(pidx)
                putchar(10)
                let pld = pgep + 1
                emit_str("  %t")
                pint(pld)
                emit_str(" = load i64, i64* %t")
                pint(pgep)
                putchar(10)
                return Op { kind: 1, val: pld, next: pld + 1 }
            }
            # List-of-structs element field read: toks[<expr>].field
            #   buffer [64*nf x i64], element stride nf -> load buf[idx*nf + fi].
            let lsty2 = sty_of(slots, src, t.nstart, t.nlen)
            if karr == 2 {
              if lsty2 >= 0 {
                let lbend = bracket_end(toks, i + 2)
                let dotk = toks[lbend + 1]
                if dotk.kind == 27 {
                    let fld = toks[lbend + 2]
                    let lslot = find_slot(slots, src, t.nstart, t.nlen)
                    let nf = struct_nfields(defs, lsty2)
                    let lbuf = 64 * nf + 1
                    let fi = field_index(defs, lsty2, src, fld.nstart, fld.nlen)
                    let lidx = gen_expr(toks, slots, fns, defs, src, i + 2, lbend, counter)
                    let mulc = lidx.next
                    emit_str("  %t")
                    pint(mulc)
                    emit_str(" = mul i64 ")
                    emit_op(lidx)
                    emit_str(", ")
                    pint(nf)
                    putchar(10)
                    let offc = mulc + 1
                    emit_str("  %t")
                    pint(offc)
                    emit_str(" = add i64 %t")
                    pint(mulc)
                    emit_str(", ")
                    pint(fi)
                    putchar(10)
                    let lgepc = offc + 1
                    emit_str("  %t")
                    pint(lgepc)
                    emit_str(" = getelementptr [")
                    pint(lbuf)
                    emit_str(" x i64], [")
                    pint(lbuf)
                    emit_str(" x i64]* %v")
                    pint(lslot)
                    emit_str(", i64 0, i64 %t")
                    pint(offc)
                    putchar(10)
                    let lldc = lgepc + 1
                    emit_str("  %t")
                    pint(lldc)
                    emit_str(" = load i64, i64* %t")
                    pint(lgepc)
                    putchar(10)
                    return Op { kind: 1, val: lldc, next: lldc + 1 }
                }
              }
            }
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
    if t.kind == 9 {
        # parenthesized group `( ... )` -> skip to just past the matching ')'.
        return paren_end(toks, i + 1) + 1
    }
    if t.kind == 1 {
        let nx = toks[i + 1]
        if nx.kind == 9 {
            return paren_end(toks, i + 2) + 1
        }
        if nx.kind == 23 {
            # name [ idx ]  -> just past ']'. But a List-of-structs element field
            # read `name [ idx ] . field` extends 2 more tokens (. field).
            let be = bracket_end(toks, i + 2)
            let pd = toks[be + 1]
            if pd.kind == 27 {
                return be + 3
            }
            return be + 1
        }
        if nx.kind == 27 {
            # name . field  = 3 tokens (struct field / List .len without parens).
            # name . len ( ) = 5 tokens (string .len() — skip the empty parens).
            let after = toks[i + 3]
            if after.kind == 9 {
                return paren_end(toks, i + 4) + 1
            }
            return i + 3
        }
    }
    return i + 1
}
# Find the next `and`(31)/`or`(32) at paren-depth 0 in [i, stop); returns stop if
# none. Used to bound a comparison's RHS so `and`/`or` stay lower-precedence.
fn next_logical(toks: &List<Token>, i: Int, stop: Int) -> Int {
    let mut j = i
    let mut depth = 0
    let mut go = true
    while go {
        if j >= stop { go = false }
        else {
            let t = toks[j]
            if t.kind == 9 { depth = depth + 1; j = j + 1 }
            else if t.kind == 10 { depth = depth - 1; j = j + 1 }
            else if depth == 0 {
                if t.kind == 31 { go = false } else if t.kind == 32 { go = false } else { j = j + 1 }
            }
            else { j = j + 1 }
        }
    }
    return j
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
    # comparison as a value: `<` (18) / `>` (19) / `==` (20) -> icmp + zext i1->i64.
    # Lower precedence than +/-, so the RHS is a full additive expression
    # (gen_expr from the operand). The boolean result is widened to i64 (1/0).
    if op.kind == 18 or op.kind == 19 or op.kind == 20 or op.kind == 29 or op.kind == 30 or op.kind == 33 {
        # RHS is the additive expr up to the next `and`/`or` (so logicals stay
        # lower-precedence): `c >= 48 and c <= 57` parses as `(c>=48) and (c<=57)`.
        # kind 33 = `!=` (not-equal); the others are < > == <= >=.
        let rstop = next_logical(toks, i + 1, stop)
        let rhs = gen_expr(toks, slots, fns, defs, src, i + 1, rstop, acc.next)
        let cnum = rhs.next
        emit_str("  %t")
        pint(cnum)
        emit_str(" = icmp ")
        if op.kind == 18 { emit_str("slt") } else if op.kind == 19 { emit_str("sgt") } else if op.kind == 29 { emit_str("sle") } else if op.kind == 30 { emit_str("sge") } else if op.kind == 33 { emit_str("ne") } else { emit_str("eq") }
        emit_str(" i64 ")
        emit_op(acc)
        emit_str(", ")
        emit_op(rhs)
        putchar(10)
        let zc = cnum + 1
        emit_str("  %t")
        pint(zc)
        emit_str(" = zext i1 %t")
        pint(cnum)
        emit_str(" to i64")
        putchar(10)
        let cacc = Op { kind: 1, val: zc, next: zc + 1 }
        # continue folding from rstop so a following `and`/`or` is applied.
        return gen_fold(toks, slots, fns, defs, src, rstop, stop, cacc)
    }
    # logical `and` (31) / `or` (32) as a value. Operands are 0/1 (from
    # comparisons), so bitwise `and i64`/`or i64` yields the correct boolean.
    # Lowest precedence -> RHS is a full expression (gen_expr).
    if op.kind == 31 or op.kind == 32 {
        # RHS is the next comparison/additive expr up to the following `and`/`or`
        # (left-associative): `a and b and c` -> `((a and b) and c)`.
        let rstop = next_logical(toks, i + 1, stop)
        let rhs = gen_expr(toks, slots, fns, defs, src, i + 1, rstop, acc.next)
        let dest = rhs.next
        emit_str("  %t")
        pint(dest)
        emit_str(" = ")
        if op.kind == 31 { emit_str("and") } else { emit_str("or") }
        emit_str(" i64 ")
        emit_op(acc)
        emit_str(", ")
        emit_op(rhs)
        putchar(10)
        let nacc = Op { kind: 1, val: dest, next: dest + 1 }
        return gen_fold(toks, slots, fns, defs, src, rstop, stop, nacc)
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

# Position of a let's RHS value (the token just after `=`), given `npos` = the
# name token index. Handles an optional type annotation `name : Type = rhs`
# (including `List<...>`, which spans to the closing `>`). Without an annotation
# `name = rhs`, the RHS is at npos+2 (npos+1 is `=`). With `name : Type = rhs`,
# we skip the `:` + Type tokens to land just past `=`.
fn rhs_pos(toks: &List<Token>, npos: Int) -> Int {
    let c1 = toks[npos + 1]
    if c1.kind == 16 {
        # `:` annotation. The type is `List<...>` (spans to '>') or a single ident.
        let ty = toks[npos + 2]
        let mut q = npos + 3
        if ty.kind == 1 {
            let after = toks[npos + 3]
            if after.kind == 18 {
                # `<...>` generic args: advance to the matching '>' (kind 19).
                let mut tq = npos + 4
                let mut tg = true
                while tg {
                    let tt = toks[tq]
                    if tt.kind == 19 { tg = false } else { tq = tq + 1 }
                }
                q = tq + 1
            }
        }
        # q now points at `=`; the RHS is the next token.
        return q + 1
    }
    # no annotation: npos+1 is `=`, RHS at npos+2.
    return npos + 2
}
# Is the RHS a `list()` constructor (ident 'list' then '(') OR an empty list
# literal `[]`? Both create an empty List. Handles typed lets (`name: Type = rhs`).
fn rhs_is_list(toks: &List<Token>, src: Str, npos: Int) -> Int {
    let vp = rhs_pos(toks, npos)
    let rhs = toks[vp]
    if rhs.kind == 1 {
        if kw4(src, rhs.nstart, rhs.nlen, 108, 105, 115, 116) == 1 {
            let after = toks[vp + 1]
            if after.kind == 9 { return 1 }
        }
    }
    # empty list literal: `[` immediately followed by `]` (kind 23 then 24).
    if rhs.kind == 23 {
        let after = toks[vp + 1]
        if after.kind == 24 { return 1 }
    }
    return 0
}
# If `let name = Name { ... }`, returns the struct-type index of Name, else -1.
fn rhs_struct_type(toks: &List<Token>, defs: &List<StructDef>, src: Str, npos: Int) -> Int {
    let vp = rhs_pos(toks, npos)
    let rhs = toks[vp]
    if rhs.kind == 1 {
        let after = toks[vp + 1]
        if after.kind == 11 {
            return struct_index_by_name(defs, src, rhs.nstart, rhs.nlen)
        }
    }
    return 0 - 1
}
# If the RHS is `<listvar>[<expr>]` where <listvar> is a List-of-structs (slot
# is_arr 2 (local) or 4 (param) with sty>=0), returns the element struct-type
# index, else -1. This lets `let t = toks[i]` bind a whole Token struct to a
# local (copying its fields) so `t.kind` / `t.value` work -- the core eval
# pattern `let t = toks[i]; if t.kind == 2 { ... }`.
fn rhs_los_elem_sty(toks: &List<Token>, slots: &List<Slot>, src: Str, npos: Int) -> Int {
    let vp = rhs_pos(toks, npos)
    let rhs = toks[vp]
    if rhs.kind == 1 {
        let after = toks[vp + 1]
        if after.kind == 23 {
            let karr = isarr_of(slots, src, rhs.nstart, rhs.nlen)
            if karr == 2 {
                return sty_of(slots, src, rhs.nstart, rhs.nlen)
            }
            if karr == 4 {
                return sty_of(slots, src, rhs.nstart, rhs.nlen)
            }
        }
    }
    return 0 - 1
}
# Element struct-type from a let's type annotation `name : List<Type> = ...`,
# given `npos` = the name token. Returns the struct-type index, or -1 (no
# annotation, or a non-struct element like List<Int>). This is the authoritative
# element type for `let toks: List<Token> = []` where no push reveals it.
fn let_anno_elem_sty(toks: &List<Token>, defs: &List<StructDef>, src: Str, npos: Int) -> Int {
    let c1 = toks[npos + 1]
    if c1.kind == 16 {
        let ty = toks[npos + 2]
        let islist = kw4(src, ty.nstart, ty.nlen, 76, 105, 115, 116)
        if islist == 1 {
            let lt = toks[npos + 3]
            if lt.kind == 18 {
                let et = toks[npos + 4]
                if et.kind == 1 {
                    return struct_index_by_name(defs, src, et.nstart, et.nlen)
                }
            }
        }
    }
    return 0 - 1
}
# Element struct-type of `let name = list()` by scanning [start,end) for the
# first struct push `name . push ( Type ...`. Returns the struct-type index, or
# -1 if the List holds scalars (no struct push found). This lets a List<Token>
# allocate a contiguous [64*nfields x i64] buffer with element stride = nfields.
fn list_elem_sty(toks: &List<Token>, defs: &List<StructDef>, src: Str, start: Int, end: Int, nstart: Int, nlen: Int) -> Int {
    let mut i = start
    while i < end {
        let t = toks[i]
        # match the token run:  name . push ( Type  followed by an open-brace.
        if t.kind == 1 {
            if name_eq(src, t.nstart, t.nlen, nstart, nlen) == 1 {
                let d = toks[i + 1]
                if d.kind == 27 {
                    let m = toks[i + 2]
                    if m.kind == 1 {
                        if kw4(src, m.nstart, m.nlen, 112, 117, 115, 104) == 1 {
                            let op = toks[i + 3]
                            if op.kind == 9 {
                                let ty = toks[i + 4]
                                if ty.kind == 1 {
                                    let br = toks[i + 5]
                                    if br.kind == 11 {
                                        return struct_index_by_name(defs, src, ty.nstart, ty.nlen)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        i = i + 1
    }
    return 0 - 1
}
# Element struct-type of the param at position `argpos` of `fn <callee>`, if that
# param is annotated `List<Struct>`. Scans the callee's signature in the token
# stream (params separated by commas at paren depth 1). Returns -1 otherwise.
# This lets a caller infer a local List's element type from the callee it is
# passed to (the push happens in the callee body, not the caller's).
fn param_list_elem_sty(toks: &List<Token>, defs: &List<StructDef>, src: Str, n: Int, cns: Int, cnl: Int, argpos: Int) -> Int {
    let mut i = 0
    while i < n {
        let t = toks[i]
        # find `fn <callee> (`
        if t.kind == 13 {
            let nm = toks[i + 1]
            if nm.kind == 1 {
                if name_eq(src, nm.nstart, nm.nlen, cns, cnl) == 1 {
                    # params start after '(' at i+2. Walk to the argpos-th param's
                    # type annotation. A param is `name` or `name : Type`.
                    let mut q = i + 3
                    let mut pos = 0
                    let mut go = true
                    while go {
                        let qt = toks[q]
                        if qt.kind == 10 { go = false }
                        else if qt.kind == 1 {
                            # this is a param name. Is it the one we want?
                            if pos == argpos {
                                # check for `: List < Type >`
                                let c1 = toks[q + 1]
                                if c1.kind == 16 {
                                    let ty = toks[q + 2]
                                    let islist = kw4(src, ty.nstart, ty.nlen, 76, 105, 115, 116)
                                    if islist == 1 {
                                        # element type is the ident after '<' (q+4)
                                        let et = toks[q + 4]
                                        if et.kind == 1 {
                                            return struct_index_by_name(defs, src, et.nstart, et.nlen)
                                        }
                                    }
                                }
                                go = false
                            } else {
                                # skip this param's type if annotated, then expect a comma
                                let c1 = toks[q + 1]
                                if c1.kind == 16 {
                                    let ty = toks[q + 2]
                                    let islist = kw4(src, ty.nstart, ty.nlen, 76, 105, 115, 116)
                                    if islist == 1 {
                                        # advance past List<...> to the closing '>'
                                        let mut tq = q + 3
                                        let mut tg = true
                                        while tg {
                                            let tt = toks[tq]
                                            if tt.kind == 19 { tg = false } else { tq = tq + 1 }
                                        }
                                        q = tq
                                    } else {
                                        q = q + 2
                                    }
                                }
                                pos = pos + 1
                            }
                            q = q + 1
                        }
                        else { q = q + 1 }
                    }
                    return 0 - 1
                }
            }
        }
        i = i + 1
    }
    return 0 - 1
}
# Element struct-type of `let name = list()` inferred from a call site: scan
# [start,end) for a call `f( ... name ... )` where `name` is a direct single-ident
# argument, then read f's matching param's `List<Struct>` annotation. Returns -1
# if no such call. `n` bounds the whole token stream (for the callee lookup).
fn call_arg_elem_sty(toks: &List<Token>, defs: &List<StructDef>, src: Str, start: Int, end: Int, nstart: Int, nlen: Int, n: Int) -> Int {
    let mut i = start
    while i < end {
        let t = toks[i]
        # a call: ident '(' ...
        if t.kind == 1 {
            let nx = toks[i + 1]
            if nx.kind == 9 {
                # walk args, tracking position; match `name` as a single-ident arg.
                let mut q = i + 2
                let mut pos = 0
                let mut go = true
                while go {
                    let qt = toks[q]
                    if qt.kind == 10 { go = false }
                    else if qt.kind == 25 { pos = pos + 1; q = q + 1 }
                    else {
                        # is this arg exactly `name` (single ident followed by , or ))?
                        if qt.kind == 1 {
                            let after = toks[q + 1]
                            if after.kind == 10 {
                                if name_eq(src, qt.nstart, qt.nlen, nstart, nlen) == 1 {
                                    let r = param_list_elem_sty(toks, defs, src, n, t.nstart, t.nlen, pos)
                                    if r >= 0 { return r }
                                }
                            }
                            if after.kind == 25 {
                                if name_eq(src, qt.nstart, qt.nlen, nstart, nlen) == 1 {
                                    let r = param_list_elem_sty(toks, defs, src, n, t.nstart, t.nlen, pos)
                                    if r >= 0 { return r }
                                }
                            }
                        }
                        q = q + 1
                    }
                }
            }
        }
        i = i + 1
    }
    return 0 - 1
}

fn add_local_slots(base: List<Slot>, toks: &List<Token>, defs: &List<StructDef>, src: Str, start: Int, end: Int, slot0: Int, n: Int) -> List<Slot> {
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
            # RHS value position handles typed lets `name : Type = rhs`.
            let vp = rhs_pos(toks, npos)
            let rhs = toks[vp]
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
            } else if rhs_los_elem_sty(toks, &slots, src, npos) >= 0 {
                # `let t = toks[i]` binding a List-of-structs element: t is a struct
                # local (alloca [nf x i64], sty=elem type). The field copy is emitted
                # in gen_stmts; here we just reserve the slot.
                let est = rhs_los_elem_sty(toks, &slots, src, npos)
                let nf = struct_nfields(defs, est)
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 0, alen: 0 , sty: est })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [")
                pint(nf)
                emit_str(" x i64]")
                putchar(10)
                next_slot = next_slot + 1
            } else if rhs_is_list(toks, src, npos) == 1 {
                # List: buffer at %v<slot>, length at %v<slot+1>=0.
                # Element type: a `: List<Type>` annotation is authoritative (covers
                # `let toks: List<Token> = []` where no push reveals it); else scan
                # the body for the first push; else infer from a callee (out-param).
                let mut lest = let_anno_elem_sty(toks, defs, src, npos)
                if lest < 0 { lest = list_elem_sty(toks, defs, src, vp, end, name.nstart, name.nlen) }
                if lest < 0 { lest = call_arg_elem_sty(toks, defs, src, vp, end, name.nstart, name.nlen, n) }
                let mut lbuf = 64
                # struct-element Lists: [64*nf + 1 x i64] -- the +1 reserves a length
                # header at index 64*nf so the buffer can be passed by-pointer as a
                # List-of-structs param (length stored there, past the data region;
                # avoids the buf[63] collision that stride>1 would otherwise hit).
                if lest >= 0 { lbuf = 64 * struct_nfields(defs, lest) + 1 }
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 2, alen: 64 , sty: lest })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [")
                pint(lbuf)
                emit_str(" x i64]")
                putchar(10)
                emit_str("  %v")
                pint(next_slot + 1)
                emit_str(" = alloca i64")
                putchar(10)
                emit_str("  store i64 0, i64* %v")
                pint(next_slot + 1)
                putchar(10)
                next_slot = next_slot + 2
            } else if rhs.kind == 28 {
                # string var: is_arr=3, alen=length, sty=global key (literal nstart).
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 3, alen: rhs.value , sty: rhs.nstart })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca i8*")
                putchar(10)
                emit_str("  %g")
                pint(next_slot)
                emit_str(" = getelementptr [")
                pint(rhs.value + 1)
                emit_str(" x i8], [")
                pint(rhs.value + 1)
                emit_str(" x i8]* @.s")
                pint(rhs.nstart)
                emit_str(", i64 0, i64 0")
                putchar(10)
                emit_str("  store i8* %g")
                pint(next_slot)
                emit_str(", i8** %v")
                pint(next_slot)
                putchar(10)
                next_slot = next_slot + 1
            } else if rhs.kind == 23 {
                let alen = count_arr_elems(toks, vp + 1)
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
# Index just past a complete `if [cond] { } [else if [cond] { }]* [else { }]`
# statement, given `ifpos` = the `if` token. Walks the whole else-if chain so the
# else-region of an outer `if` covers a nested `else if ... else ...` entirely
# (not just the first inner then-block). Returns the index after the last `}`.
fn if_stmt_end(toks: &List<Token>, ifpos: Int, n: Int) -> Int {
    # skip the condition to the then-block's `{`
    let mut bo = ifpos + 1
    let mut g1 = true
    while g1 {
        if bo >= n { g1 = false }
        else {
            let bt = toks[bo]
            if bt.kind == 11 { g1 = false } else { bo = bo + 1 }
        }
    }
    let mut cur = match_brace(toks, bo, n) + 1
    # consume any number of `else { }` / `else if [cond] { }` clauses
    let mut go = true
    while go {
        if cur >= n { go = false }
        else {
            let et = toks[cur]
            if et.kind == 17 {
                let nxt = toks[cur + 1]
                if nxt.kind == 15 {
                    # `else if` -> recurse over the nested if-chain and stop (the
                    # recursion consumes the rest of the chain).
                    cur = if_stmt_end(toks, cur + 1, n)
                    go = false
                } else {
                    # plain `else { }` -> skip to its `{`, match, and stop.
                    let mut eo = cur + 1
                    let mut g2 = true
                    while g2 {
                        if eo >= n { g2 = false }
                        else {
                            let ot = toks[eo]
                            if ot.kind == 11 { g2 = false } else { eo = eo + 1 }
                        }
                    }
                    cur = match_brace(toks, eo, n) + 1
                    go = false
                }
            } else { go = false }
        }
    }
    return cur
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
            # RHS value position handles typed lets `name : Type = rhs`.
            let vp = rhs_pos(toks, npos)
            let rhs = toks[vp]
            let los_est = rhs_los_elem_sty(toks, slots, src, npos)
            let lsti = rhs_struct_type(toks, defs, src, npos)
            if los_est >= 0 {
                # `let t = toks[idx]` -> copy the struct element into t's [nf x i64]
                # local: for each field k, t[k] = buf[idx*nf + k]. `vp` is the list
                # var token; `[` at vp+1, index expr from vp+2.
                let lvar = toks[vp]
                let lnf = struct_nfields(defs, los_est)
                let lkarr = isarr_of(slots, src, lvar.nstart, lvar.nlen)
                let lsrcslot = find_slot(slots, src, lvar.nstart, lvar.nlen)
                let libend = bracket_end(toks, vp + 2)
                let lidx = gen_expr(toks, slots, fns, defs, src, vp + 2, libend, counter)
                counter = lidx.next
                # base = idx * nf  (element offset into the i64 buffer)
                emit_str("  %t")
                pint(counter)
                emit_str(" = mul i64 ")
                emit_op(lidx)
                emit_str(", ")
                pint(lnf)
                putchar(10)
                let lbase = counter
                counter = counter + 1
                # for a param list (is_arr=4) load the buffer pointer once
                let mut lbufptr = 0
                if lkarr == 4 {
                    emit_str("  %t")
                    pint(counter)
                    emit_str(" = load i64*, i64** %v")
                    pint(lsrcslot)
                    putchar(10)
                    lbufptr = counter
                    counter = counter + 1
                }
                let lbufsz = 64 * lnf + 1
                let mut fk = 0
                while fk < lnf {
                    # source elem ptr = buf + (base + fk)
                    emit_str("  %t")
                    pint(counter)
                    emit_str(" = add i64 %t")
                    pint(lbase)
                    emit_str(", ")
                    pint(fk)
                    putchar(10)
                    let foff = counter
                    counter = counter + 1
                    if lkarr == 4 {
                        emit_str("  %t")
                        pint(counter)
                        emit_str(" = getelementptr i64, i64* %t")
                        pint(lbufptr)
                        emit_str(", i64 %t")
                        pint(foff)
                        putchar(10)
                    } else {
                        emit_str("  %t")
                        pint(counter)
                        emit_str(" = getelementptr [")
                        pint(lbufsz)
                        emit_str(" x i64], [")
                        pint(lbufsz)
                        emit_str(" x i64]* %v")
                        pint(lsrcslot)
                        emit_str(", i64 0, i64 %t")
                        pint(foff)
                        putchar(10)
                    }
                    let fsrc = counter
                    counter = counter + 1
                    emit_str("  %t")
                    pint(counter)
                    emit_str(" = load i64, i64* %t")
                    pint(fsrc)
                    putchar(10)
                    let fval = counter
                    counter = counter + 1
                    # dest = t[fk]  (t is a [nf x i64] struct local at %v<slot>)
                    emit_str("  %t")
                    pint(counter)
                    emit_str(" = getelementptr [")
                    pint(lnf)
                    emit_str(" x i64], [")
                    pint(lnf)
                    emit_str(" x i64]* %v")
                    pint(slot)
                    emit_str(", i64 0, i64 ")
                    pint(fk)
                    putchar(10)
                    let fdst = counter
                    counter = counter + 1
                    emit_str("  store i64 %t")
                    pint(fval)
                    emit_str(", i64* %t")
                    pint(fdst)
                    putchar(10)
                    fk = fk + 1
                }
                let stop = find_semi(toks, libend + 1, end)
                i = stop + 1
            } else if lsti >= 0 {
                # struct literal: Name { f: v, ... } -> store each field via GEP.
                # `vp` is the struct-name token; the open-brace is at vp+1. Field is
                # `name : value` (the `:` is tokenized as kind 16).
                let nf = struct_nfields(defs, lsti)
                let bopen = vp + 1
                let bclose = match_brace(toks, bopen, end)
                let mut q = bopen + 1
                while q < bclose {
                    let fld = toks[q]
                    let fi = field_index(defs, lsti, src, fld.nstart, fld.nlen)
                    let mut vstart = q + 1
                    let colt = toks[q + 1]
                    if colt.kind == 16 { vstart = q + 2 }
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
                # let lst = list() / [];  — alloca/len already emitted in collect; skip
                let stop = find_semi(toks, vp, end)
                i = stop + 1
            } else if rhs.kind == 23 {
                # array literal: store each element via GEP. `vp` is the `[`.
                let alen = arrlen_of(slots, src, name.nstart, name.nlen)
                let bend = bracket_end(toks, vp + 1)
                let mut q = vp + 1
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
            } else if rhs.kind == 28 {
                # let s = `lit`;  — string global + i8* alloca + ptr-store already
                # emitted in the slot collector; nothing to do here.
                let stop = find_semi(toks, vp, end)
                i = stop + 1
            } else {
                let stop = find_semi(toks, vp, end)
                let e = gen_expr(toks, slots, fns, defs, src, vp, stop, counter)
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
            if is_putchar(src, t.nstart, t.nlen) == 1 {
                # putchar(<expr>) ; -> trunc to i32 + call @putchar
                let argstop = paren_end(toks, i + 2)
                let e = gen_expr(toks, slots, fns, defs, src, i + 2, argstop, counter)
                counter = e.next
                emit_str("  %t")
                pint(counter)
                emit_str(" = trunc i64 ")
                emit_op(e)
                emit_str(" to i32")
                putchar(10)
                let tc = counter
                counter = counter + 1
                emit_str("  %t")
                pint(counter)
                emit_str(" = call i32 @putchar(i32 %t")
                pint(tc)
                emit_str(")")
                putchar(10)
                counter = counter + 1
                let stop = find_semi(toks, argstop, end)
                i = stop + 1
            } else if nx.kind == 9 {
                # bare call statement: name(args) ; -> emit the call, discard result.
                # (gen_expr/gen_factor generate the call IR.) Needed for out-param
                # patterns like `tokenize(src, out);`.
                let cstop = find_semi(toks, i + 2, end)
                let e = gen_expr(toks, slots, fns, defs, src, i, cstop, counter)
                counter = e.next
                i = cstop + 1
            } else if nx.kind == 27 {
              let astiarr = isarr_of(slots, src, t.nstart, t.nlen)
              # struct field write requires a SCALAR struct var (is_arr=0). A
              # List-of-structs (sty>=0, is_arr=2) must skip this and reach the
              # push handler in the final else.
              if asti >= 0 and astiarr == 0 {
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
              } else if isarr_of(slots, src, t.nstart, t.nlen) == 4 {
               let ppsty = sty_of(slots, src, t.nstart, t.nlen)
               if ppsty >= 0 {
                # push to a List-of-structs PARAMETER (is_arr=4, sty>=0): buffer
                # pointer with stride nf; length lives at ptr[64*nf]. Load len, store
                # each field at ptr[len*nf + fi], len+1 -> ptr[64*nf].
                let psslot = find_slot(slots, src, t.nstart, t.nlen)
                let pnf = struct_nfields(defs, ppsty)
                let plenidx = 64 * pnf
                let psargstop = paren_end(toks, i + 4)
                # load buffer ptr
                emit_str("  %t")
                pint(counter)
                emit_str(" = load i64*, i64** %v")
                pint(psslot)
                putchar(10)
                let psbp = counter
                counter = counter + 1
                # length pointer = ptr + 64*nf, load len
                emit_str("  %t")
                pint(counter)
                emit_str(" = getelementptr i64, i64* %t")
                pint(psbp)
                emit_str(", i64 ")
                pint(plenidx)
                putchar(10)
                let pslenp = counter
                counter = counter + 1
                emit_str("  %t")
                pint(counter)
                emit_str(" = load i64, i64* %t")
                pint(pslenp)
                putchar(10)
                let pslen = counter
                counter = counter + 1
                # base = len*nf
                emit_str("  %t")
                pint(counter)
                emit_str(" = mul i64 %t")
                pint(pslen)
                emit_str(", ")
                pint(pnf)
                putchar(10)
                let psbase = counter
                counter = counter + 1
                # iterate struct-literal fields. token layout: name . push ( Type
                # then the open-brace at i+5 starts the struct literal.
                let psbopen = i + 5
                let psbclose = match_brace(toks, psbopen, end)
                let mut psq = psbopen + 1
                while psq < psbclose {
                    let psfld = toks[psq]
                    let psfi = field_index(defs, ppsty, src, psfld.nstart, psfld.nlen)
                    let mut psvstart = psq + 1
                    let pscolt = toks[psq + 1]
                    if pscolt.kind == 16 { psvstart = psq + 2 }
                    let psvstop = arr_elem_end(toks, psvstart, psbclose)
                    let pse = gen_expr(toks, slots, fns, defs, src, psvstart, psvstop, counter)
                    counter = pse.next
                    # offset = base + fi
                    emit_str("  %t")
                    pint(counter)
                    emit_str(" = add i64 %t")
                    pint(psbase)
                    emit_str(", ")
                    pint(psfi)
                    putchar(10)
                    let psoff = counter
                    counter = counter + 1
                    # element ptr = bufptr + offset
                    emit_str("  %t")
                    pint(counter)
                    emit_str(" = getelementptr i64, i64* %t")
                    pint(psbp)
                    emit_str(", i64 %t")
                    pint(psoff)
                    putchar(10)
                    let psep = counter
                    counter = counter + 1
                    emit_str("  store i64 ")
                    emit_op(pse)
                    emit_str(", i64* %t")
                    pint(psep)
                    putchar(10)
                    psq = psvstop + 1
                }
                # len = len + 1 -> ptr[64*nf]
                emit_str("  %t")
                pint(counter)
                emit_str(" = add i64 %t")
                pint(pslen)
                emit_str(", 1")
                putchar(10)
                let psinc = counter
                counter = counter + 1
                emit_str("  store i64 %t")
                pint(psinc)
                emit_str(", i64* %t")
                pint(pslenp)
                putchar(10)
                let psstop = find_semi(toks, psargstop, end)
                i = psstop + 1
               } else {
                # push a SCALAR to a List PARAMETER (is_arr=4, buffer pointer): load
                # ptr, len from ptr[63], store v at ptr[len], len+1 back to ptr[63].
                let pslot = find_slot(slots, src, t.nstart, t.nlen)
                let pargstop = paren_end(toks, i + 4)
                let pe = gen_expr(toks, slots, fns, defs, src, i + 4, pargstop, counter)
                counter = pe.next
                let pbp = counter
                emit_str("  %t")
                pint(pbp)
                emit_str(" = load i64*, i64** %v")
                pint(pslot)
                putchar(10)
                let plenp = pbp + 1
                emit_str("  %t")
                pint(plenp)
                emit_str(" = getelementptr i64, i64* %t")
                pint(pbp)
                emit_str(", i64 63")
                putchar(10)
                let plen = plenp + 1
                emit_str("  %t")
                pint(plen)
                emit_str(" = load i64, i64* %t")
                pint(plenp)
                putchar(10)
                let pep = plen + 1
                emit_str("  %t")
                pint(pep)
                emit_str(" = getelementptr i64, i64* %t")
                pint(pbp)
                emit_str(", i64 %t")
                pint(plen)
                putchar(10)
                emit_str("  store i64 ")
                emit_op(pe)
                emit_str(", i64* %t")
                pint(pep)
                putchar(10)
                let pinc = pep + 1
                emit_str("  %t")
                pint(pinc)
                emit_str(" = add i64 %t")
                pint(plen)
                emit_str(", 1")
                putchar(10)
                emit_str("  store i64 %t")
                pint(pinc)
                emit_str(", i64* %t")
                pint(plenp)
                putchar(10)
                counter = pinc + 1
                let pstop = find_semi(toks, pargstop, end)
                i = pstop + 1
               }
              } else {
                # list push: lst.push(expr) ;
                let slot = find_slot(slots, src, t.nstart, t.nlen)
                let lsty = sty_of(slots, src, t.nstart, t.nlen)
                # method name at i+2, '(' at i+3, arg from i+4
                let argstop = paren_end(toks, i + 4)
                if lsty >= 0 {
                    # List-of-structs push: arg is `Type { f: v, ... }`. Store each
                    # field at buf[len*nf + field_index]; buffer is [64*nf+1 x i64]
                    # (the +1 length-header slot is unused for local push).
                    let nf = struct_nfields(defs, lsty)
                    let lbuf = 64 * nf + 1
                    # load len, compute base = len*nf
                    emit_str("  %t")
                    pint(counter)
                    emit_str(" = load i64, i64* %v")
                    pint(slot + 1)
                    putchar(10)
                    let lenc = counter
                    counter = counter + 1
                    emit_str("  %t")
                    pint(counter)
                    emit_str(" = mul i64 %t")
                    pint(lenc)
                    emit_str(", ")
                    pint(nf)
                    putchar(10)
                    let basec = counter
                    counter = counter + 1
                    # iterate the struct-literal fields. token layout: name . push
                    # ( Type then open-brace at i+5 (i=name, so i+5 is the brace).
                    let bopen = i + 5
                    let bclose = match_brace(toks, bopen, end)
                    let mut q = bopen + 1
                    while q < bclose {
                        let fld = toks[q]
                        let fi = field_index(defs, lsty, src, fld.nstart, fld.nlen)
                        let mut vstart = q + 1
                        let colt = toks[q + 1]
                        if colt.kind == 16 { vstart = q + 2 }
                        let vstop = arr_elem_end(toks, vstart, bclose)
                        let e = gen_expr(toks, slots, fns, defs, src, vstart, vstop, counter)
                        counter = e.next
                        # field offset = base + fi
                        emit_str("  %t")
                        pint(counter)
                        emit_str(" = add i64 %t")
                        pint(basec)
                        emit_str(", ")
                        pint(fi)
                        putchar(10)
                        let offc = counter
                        counter = counter + 1
                        emit_str("  %t")
                        pint(counter)
                        emit_str(" = getelementptr [")
                        pint(lbuf)
                        emit_str(" x i64], [")
                        pint(lbuf)
                        emit_str(" x i64]* %v")
                        pint(slot)
                        emit_str(", i64 0, i64 %t")
                        pint(offc)
                        putchar(10)
                        let gepc = counter
                        counter = counter + 1
                        emit_str("  store i64 ")
                        emit_op(e)
                        emit_str(", i64* %t")
                        pint(gepc)
                        putchar(10)
                        q = vstop + 1
                    }
                    # len = len + 1
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
                } else {
                  # scalar List push: store one value at buf[len]; len = len + 1
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
            # condition is [i+1, bopen). Evaluate the WHOLE condition as a value
            # (gen_expr handles comparisons, `and`/`or`, grouping), then branch on
            # nonzero. This supports compound conditions like `c >= 48 and c <= 57`.
            let cstart = i + 1
            let cend = bopen
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
            let cond = gen_expr(toks, slots, fns, defs, src, cstart, cend, counter)
            let cnum = cond.next
            emit_str("  %t")
            pint(cnum)
            emit_str(" = icmp ne i64 ")
            emit_op(cond)
            emit_str(", 0")
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
            # condition [i+1, bopen) -- evaluated as a whole value (supports
            # comparisons, `and`/`or`, grouping), then branched on nonzero.
            let cstart = i + 1
            let cend = bopen
            # is there an `else { ... }` or `else if ...` after the then-block?
            let mut has_else = 0
            let mut else_if = 0
            let mut ebody_start = bclose
            let mut ebody_end = bclose
            let mut resume = bclose + 1
            let after_then = bclose + 1
            if after_then < end {
                let et = toks[after_then]
                if et.kind == 17 {
                    has_else = 1
                    let nxt = toks[after_then + 1]
                    if nxt.kind == 15 {
                        # `else if ...` -> the else body is the entire nested if
                        # statement (a statement gen_stmts handles recursively).
                        else_if = 1
                        let chainend = if_stmt_end(toks, after_then + 1, end)
                        ebody_start = after_then + 1
                        ebody_end = chainend
                        resume = chainend
                    } else {
                        # plain `else { ... }` -> body is the brace interior.
                        let mut eo = after_then + 1
                        let mut g3 = true
                        while g3 {
                            if eo >= end { g3 = false }
                            else {
                                let ot = toks[eo]
                                if ot.kind == 11 { g3 = false } else { eo = eo + 1 }
                            }
                        }
                        ebody_start = eo + 1
                        ebody_end = match_brace(toks, eo, end)
                        resume = ebody_end + 1
                    }
                }
            }
            let lbl = counter
            counter = counter + 1
            let cond = gen_expr(toks, slots, fns, defs, src, cstart, cend, counter)
            let cnum = cond.next
            emit_str("  %t")
            pint(cnum)
            emit_str(" = icmp ne i64 ")
            emit_op(cond)
            emit_str(", 0")
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
                counter = gen_stmts(toks, slots, fns, defs, src, ebody_start, ebody_end, counter)
            }
            emit_str("  br label %imerge")
            pint(lbl)
            putchar(10)
            emit_str("imerge")
            pint(lbl)
            emit_str(":")
            putchar(10)
            if has_else == 1 { i = resume } else { i = bclose + 1 }
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
            # RHS value position handles typed lets `name : Type = rhs`.
            let vp = rhs_pos(toks, npos)
            let rhs = toks[vp]
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
                # List buffer: struct-element Lists get [64*nf+1 x i64] (the +1 is a
                # length header at index 64*nf for by-pointer param passing) with
                # sty = element struct type (stride = nfields); scalar Lists [64 x i64].
                # Element type: `: List<Type>` annotation first, else push-scan, else callee.
                let mut lest = let_anno_elem_sty(toks, defs, src, npos)
                if lest < 0 { lest = list_elem_sty(toks, defs, src, vp, n, name.nstart, name.nlen) }
                if lest < 0 { lest = call_arg_elem_sty(toks, defs, src, vp, n, name.nstart, name.nlen, n) }
                let mut lbuf = 64
                if lest >= 0 { lbuf = 64 * struct_nfields(defs, lest) + 1 }
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 2, alen: 64 , sty: lest })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [")
                pint(lbuf)
                emit_str(" x i64]")
                putchar(10)
                emit_str("  %v")
                pint(next_slot + 1)
                emit_str(" = alloca i64")
                putchar(10)
                emit_str("  store i64 0, i64* %v")
                pint(next_slot + 1)
                putchar(10)
                next_slot = next_slot + 2
            } else if rhs.kind == 28 {
                # string var: is_arr=3, alen=length, sty=global key (literal nstart).
                # alloca i8* + GEP the @.s<key> global's element 0 + store the ptr.
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 3, alen: rhs.value , sty: rhs.nstart })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca i8*")
                putchar(10)
                emit_str("  %g")
                pint(next_slot)
                emit_str(" = getelementptr [")
                pint(rhs.value + 1)
                emit_str(" x i8], [")
                pint(rhs.value + 1)
                emit_str(" x i8]* @.s")
                pint(rhs.nstart)
                emit_str(", i64 0, i64 0")
                putchar(10)
                emit_str("  store i8* %g")
                pint(next_slot)
                emit_str(", i8** %v")
                pint(next_slot)
                putchar(10)
                next_slot = next_slot + 1
            } else if rhs.kind == 23 {
                let alen = count_arr_elems(toks, vp + 1)
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
fn emit_fn(toks: &List<Token>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, f: Fn, n: Int) -> Int {
    emit_str("define i64 @")
    emit_name(src, f.nstart, f.nlen)
    emit_str("(")
    # incoming SSA params: %a0, %a1, ...  (Str params are i8*, others i64)
    let mut pi = 0
    while pi < f.npar {
        if pi > 0 { emit_str(", ") }
        let mut pty = f.p0ty
        if pi == 1 { pty = f.p1ty } else if pi == 2 { pty = f.p2ty } else if pi == 3 { pty = f.p3ty }
        else if pi == 4 { pty = f.p4ty } else if pi == 5 { pty = f.p5ty } else if pi == 6 { pty = f.p6ty } else if pi == 7 { pty = f.p7ty }
        # Str -> i8*, List -> i64* (buffer pointer; len read from buf[63]), else i64.
        if pty == 1 { emit_str("i8* %a") } else if pty == 2 { emit_str("i64* %a") } else { emit_str("i64 %a") }
        pint(pi)
        pi = pi + 1
    }
    emit_str(") {")
    putchar(10)
    # each param -> its own alloca slot %v0..%v<npar-1>, store %aN into it.
    # Str params: is_arr=3 string slot, `alloca i8*` + store i8*.
    let mut slots: List<Slot> = []
    let mut s2 = 0
    while s2 < f.npar {
        let mut pns = f.p0s
        let mut pnl = f.p0l
        let mut pty = f.p0ty
        if s2 == 1 { pns = f.p1s; pnl = f.p1l; pty = f.p1ty }
        else if s2 == 2 { pns = f.p2s; pnl = f.p2l; pty = f.p2ty }
        else if s2 == 3 { pns = f.p3s; pnl = f.p3l; pty = f.p3ty }
        else if s2 == 4 { pns = f.p4s; pnl = f.p4l; pty = f.p4ty }
        else if s2 == 5 { pns = f.p5s; pnl = f.p5l; pty = f.p5ty }
        else if s2 == 6 { pns = f.p6s; pnl = f.p6l; pty = f.p6ty }
        else if s2 == 7 { pns = f.p7s; pnl = f.p7l; pty = f.p7ty }
        if pty == 1 {
            slots.push(Slot { nstart: pns, nlen: pnl, slot: s2, is_arr: 3, alen: 0 , sty: 0 - 1 })
            emit_str("  %v")
            pint(s2)
            emit_str(" = alloca i8*")
            putchar(10)
            emit_str("  store i8* %a")
            pint(s2)
            emit_str(", i8** %v")
            pint(s2)
            putchar(10)
        } else if pty == 2 {
            # List param: is_arr=4 (List-by-pointer). The buffer pointer (i64*) is
            # stored in an i64** alloca. For a scalar List, xs[i] = GEP i64 and
            # xs.len = load buf[63]. For a List-of-structs param the slot's sty is
            # the element struct type and stride = nfields, length at buf[64*nf].
            # The element type comes from the param's OWN `List<Type>` annotation
            # (authoritative: works whether the body pushes or only reads it -- a
            # read-only consumer like `eval(toks: List<Token>)` has no push to scan).
            # Fall back to a body push-scan if the annotation lacks a struct type.
            let mut pest = param_list_elem_sty(toks, defs, src, n, f.nstart, f.nlen, s2)
            if pest < 0 { pest = list_elem_sty(toks, defs, src, f.bstart, f.bend, pns, pnl) }
            slots.push(Slot { nstart: pns, nlen: pnl, slot: s2, is_arr: 4, alen: 0 , sty: pest })
            emit_str("  %v")
            pint(s2)
            emit_str(" = alloca i64*")
            putchar(10)
            emit_str("  store i64* %a")
            pint(s2)
            emit_str(", i64** %v")
            pint(s2)
            putchar(10)
        } else {
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
        }
        s2 = s2 + 1
    }
    # body locals start at slot npar
    let allslots = add_local_slots(slots, toks, defs, src, f.bstart, f.bend, f.npar, n)
    let last = gen_stmts(toks, &allslots, fns, defs, src, f.bstart, f.bend, 1)
    emit_str("}")
    putchar(10)
    return 0
}

fn compile(src: Str) -> Int {
    let toks = tokenize(src)
    let n = toks.len()
    # declare the C putchar so generated code can emit output (the nl compiler's
    # own job is emitting IR text via putchar).
    emit_str("declare i32 @putchar(i32)")
    putchar(10)
    # module-level string-literal globals (one per literal, keyed by source pos)
    emit_str_globals(&toks, src, n)
    let fns = build_fns(&toks, src, n)
    let defs = build_defs(&toks, n)
    # emit each user function
    let m = fns.len()
    let mut fi = 0
    while fi < m {
        let f = fns[fi]
        emit_fn(&toks, &fns, &defs, src, f, n)
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
    # FP12d: the unified compiler now codegens STRINGS alongside everything else.
    # This demo program is the tokenizer core — a function that scans a string
    # literal byte by byte into a List (exactly the shape this compiler's own
    # tokenizer takes) — proving fixpoint_full can codegen its own source's shape.
    # tok() returns xs.len + xs[0] = 2 + 'H'(72) = 74. (Backtick delimits strings.)
    return compile("fn tok() {{ let s = `Hi`; let xs = list(); let mut i = 0; while i < s.len() {{ xs.push(s[i]); i = i + 1 }}; return xs.len + xs[0] }}; return tok();")
}
