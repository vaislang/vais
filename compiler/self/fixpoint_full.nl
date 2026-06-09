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

# Token kinds: 0=num,1=ident,2='+',3='*',4='-',38='/',5='=',6=';',7=let,8=return,21=mut,
#              18='<',19='>',20='==',22=while,11='{',12='}',9='(',10=')',13=fn,
#              15=if,17=else,25=','
struct Token { kind: Int, value: Int, nstart: Int, nlen: Int }
# Operand: kind 0=literal(val), 1=temp(%t<val>). next = next free SSA temp.
struct Op { kind: Int, val: Int, next: Int }
# A declared variable: source name range -> it lives at alloca %v<slot>.
struct Slot { nstart: Int, nlen: Int, slot: Int, is_arr: Int, alen: Int, sty: Int }
# A function: name range, up to 10 params (p0s/p0l..p9s/p9l + p0ty..p9ty), param
# count `npar`, and the body token range [bstart, bend). 10 params covers the
# file-source bootstrap helpers such as fixpoint2.nl's word_is.
struct Fn {
    nstart: Int, nlen: Int,
    p0s: Int, p0l: Int, p1s: Int, p1l: Int, p2s: Int, p2l: Int, p3s: Int, p3l: Int,
    p4s: Int, p4l: Int, p5s: Int, p5l: Int, p6s: Int, p6l: Int, p7s: Int, p7l: Int,
    p8s: Int, p8l: Int, p9s: Int, p9l: Int,
    p0ty: Int, p1ty: Int, p2ty: Int, p3ty: Int,
    p4ty: Int, p5ty: Int, p6ty: Int, p7ty: Int, p8ty: Int, p9ty: Int,
    npar: Int,
    bstart: Int, bend: Int,
    retlist: Int, retty: Int
}
# A struct type: name range + up to 8 field name ranges + field count.
struct StructDef {
    nstart: Int, nlen: Int,
    f0s: Int, f0l: Int, f1s: Int, f1l: Int, f2s: Int, f2l: Int,
    f3s: Int, f3l: Int, f4s: Int, f4l: Int, f5s: Int, f5l: Int,
    f6s: Int, f6l: Int, f7s: Int, f7l: Int,
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
# Is src[a..a+alen] the builtin name "puts" (p u t s)? The transpiler rewrites
# nl `print(` to `puts(`, so a print statement arrives here as a puts call.
fn is_puts(src: Str, a: Int, alen: Int) -> Int {
    if alen != 4 { return 0 }
    if src[a] != 112 { return 0 }
    if src[a + 1] != 117 { return 0 }
    if src[a + 2] != 116 { return 0 }
    if src[a + 3] != 115 { return 0 }
    return 1
}
# Does the string-literal source range [start, start+len) contain a `{` (123)?
# Marks a print/puts argument that needs printf interpolation rather than a plain
# puts. (A `{{` escape never survives to here -- the transpiler already collapsed
# nl `{{` to a single brace only for non-interpolated braces; interpolation `{x}`
# arrives as a single `{`.)
#
# Disambiguating literal braces from interpolation: by the time a string reaches
# compile(), the transpiler has collapsed BOTH nl `{{` (literal brace) and an
# interpolation `{x}` to single braces, so they look identical. The rule (matching
# Vais's own lexer): `{` begins an interpolation ONLY when immediately followed by
# an identifier (letter/`_`, then letters/digits/`_`) and a closing `}`. A lone `{`
# (e.g. the trailing `{` of `define i64 @main() {`) is a literal brace.
#
# interp_end: if src[start+k] == '{' starts a valid `{ident}`, returns the offset
# (relative to start) of the closing `}`; else returns -1. `len` bounds the scan.
fn interp_end(src: Str, start: Int, len: Int, k: Int) -> Int {
    if k >= len { return 0 - 1 }
    if src[start + k] != 123 { return 0 - 1 }
    let mut j = k + 1
    # first char after `{` must be an identifier start (A-Z a-z _)
    if j >= len { return 0 - 1 }
    let c0 = src[start + j]
    let lo = c0 >= 97 and c0 <= 122
    let up = c0 >= 65 and c0 <= 90
    let us = c0 == 95
    if lo == false and up == false and us == false { return 0 - 1 }
    j = j + 1
    let mut go = true
    while go {
        if j >= len { return 0 - 1 }
        let c = src[start + j]
        if c == 125 { return j }
        let dl = c >= 97 and c <= 122
        let du = c >= 65 and c <= 90
        let dd = c >= 48 and c <= 57
        let ds = c == 95
        if dl == false and du == false and dd == false and ds == false { return 0 - 1 }
        j = j + 1
    }
    return 0 - 1
}
fn lit_has_brace(src: Str, start: Int, len: Int) -> Int {
    let mut k = 0
    while k < len {
        let e = interp_end(src, start, len, k)
        if e >= 0 { return 1 }
        k = k + 1
    }
    return 0
}
# Is a named function parameter annotated as Str?
fn fn_param_is_str(f: Fn, src: Str, qs: Int, ql: Int) -> Int {
    if f.npar >= 1 { if f.p0ty == 1 { if name_eq(src, f.p0s, f.p0l, qs, ql) == 1 { return 1 } } }
    if f.npar >= 2 { if f.p1ty == 1 { if name_eq(src, f.p1s, f.p1l, qs, ql) == 1 { return 1 } } }
    if f.npar >= 3 { if f.p2ty == 1 { if name_eq(src, f.p2s, f.p2l, qs, ql) == 1 { return 1 } } }
    if f.npar >= 4 { if f.p3ty == 1 { if name_eq(src, f.p3s, f.p3l, qs, ql) == 1 { return 1 } } }
    if f.npar >= 5 { if f.p4ty == 1 { if name_eq(src, f.p4s, f.p4l, qs, ql) == 1 { return 1 } } }
    if f.npar >= 6 { if f.p5ty == 1 { if name_eq(src, f.p5s, f.p5l, qs, ql) == 1 { return 1 } } }
    if f.npar >= 7 { if f.p6ty == 1 { if name_eq(src, f.p6s, f.p6l, qs, ql) == 1 { return 1 } } }
    if f.npar >= 8 { if f.p7ty == 1 { if name_eq(src, f.p7s, f.p7l, qs, ql) == 1 { return 1 } } }
    if f.npar >= 9 { if f.p8ty == 1 { if name_eq(src, f.p8s, f.p8l, qs, ql) == 1 { return 1 } } }
    if f.npar >= 10 { if f.p9ty == 1 { if name_eq(src, f.p9s, f.p9l, qs, ql) == 1 { return 1 } } }
    return 0
}

# Scan earlier lets in [start,end) and report whether name was bound from a string
# literal. This is used only by the module pre-pass to choose %s vs %d for a
# printf format global, so it intentionally stays narrower than full typechecking.
fn range_let_is_str(toks: &List<Token>, src: Str, start: Int, end: Int, qs: Int, ql: Int) -> Int {
    let mut i = start
    while i < end {
        let t = toks[i]
        if t.kind == 13 {
            i = skip_fn_def(toks, i, end)
        } else if t.kind == 26 {
            i = skip_struct_def(toks, i, end)
        } else if t.kind == 7 {
            let nx = toks[i + 1]
            let mut npos = i + 1
            if nx.kind == 21 { npos = i + 2 }
            let name = toks[npos]
            if name_eq(src, name.nstart, name.nlen, qs, ql) == 1 {
                let vp = rhs_pos(toks, npos)
                let rhs = toks[vp]
                if rhs.kind == 28 { return 1 }
            }
            i = i + 1
        } else {
            i = i + 1
        }
    }
    return 0
}

# For the string-literal token at `lit_i`, decide whether `{ident}` should be a
# string interpolation (%s) by checking the surrounding function's Str params and
# earlier string-literal lets. Everything else remains integer interpolation (%d).
fn interp_name_is_str(toks: &List<Token>, fns: &List<Fn>, src: Str, n: Int, lit_i: Int, qs: Int, ql: Int) -> Int {
    let m = fns.len()
    let mut fi = 0
    while fi < m {
        let f = fns[fi]
        if lit_i >= f.bstart and lit_i < f.bend {
            if fn_param_is_str(f, src, qs, ql) == 1 { return 1 }
            return range_let_is_str(toks, src, f.bstart, lit_i, qs, ql)
        }
        fi = fi + 1
    }
    return range_let_is_str(toks, src, 0, lit_i, qs, ql)
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

fn string_lit_value_len(src: Str, start: Int, raw_len: Int, delim: Int) -> Int {
    if delim != 34 { return raw_len }
    let mut k = 0
    let mut out = 0
    while k < raw_len {
        if src[start + k] == 92 {
            if k + 1 < raw_len {
                out = out + 1
                k = k + 2
            } else {
                out = out + 1
                k = k + 1
            }
        } else {
            out = out + 1
            k = k + 1
        }
    }
    return out
}

fn tokenize(src: Str) -> List<Token> {
    let mut toks: List<Token> = []
    let n = src.len()
    let mut i = 0
    while i < n {
        let c = src[i]
        if is_space(c) {
            i = i + 1
        } else if c == 96 or c == 34 {
            # string literal delimited by backtick (96) or double quote (34);
            # kind 28 carries the content source range (nstart/nlen) and length.
            let delim = c
            let sstart = i + 1
            let mut j = sstart
            let mut sgo = true
            while sgo {
                if j >= n { sgo = false }
                else if delim == 34 and src[j] == 92 { j = j + 2 }
                else if src[j] == delim { sgo = false }
                else { j = j + 1 }
            }
            toks.push(Token { kind: 28, value: string_lit_value_len(src, sstart, j - sstart, delim), nstart: sstart, nlen: j - sstart })
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
            } else if kw3(src, start, len, 102, 111, 114) == 1 {
                toks.push(Token { kind: 34, value: 0, nstart: start, nlen: len })   # for
            } else if kw2(src, start, len, 105, 110) == 1 {
                toks.push(Token { kind: 35, value: 0, nstart: start, nlen: len })   # in
            } else {
                toks.push(Token { kind: 1, value: 0, nstart: start, nlen: len })
            }
        } else if c == 43 { toks.push(Token { kind: 2, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 42 { toks.push(Token { kind: 3, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 45 { toks.push(Token { kind: 4, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 47 { toks.push(Token { kind: 38, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
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
        else if c == 46 {
            # `.` field access (27), `..` exclusive range (36), `..=` inclusive (37)
            if i + 1 < n {
                if src[i + 1] == 46 {
                    if i + 2 < n {
                        if src[i + 2] == 61 { toks.push(Token { kind: 37, value: 0, nstart: 0, nlen: 0 }); i = i + 3 }
                        else { toks.push(Token { kind: 36, value: 0, nstart: 0, nlen: 0 }); i = i + 2 }
                    } else { toks.push(Token { kind: 36, value: 0, nstart: 0, nlen: 0 }); i = i + 2 }
                } else { toks.push(Token { kind: 27, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
            } else { toks.push(Token { kind: 27, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        }
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
# Emit one byte into an LLVM c"..." initializer. Backslash and quote must be
# escaped as hex escapes, or the LLVM string literal changes length / terminates.
fn emit_llvm_c_byte(c: Int) -> Int {
    if c == 92 { emit_str("\\5C"); return 0 }
    if c == 34 { emit_str("\\22"); return 0 }
    putchar(c)
    return 0
}
# Copy a source substring into a global initializer, escaping LLVM c-string
# metacharacters while preserving the source byte length.
fn emit_bytes(src: Str, start: Int, len: Int) -> Int {
    let mut k = 0
    while k < len {
        emit_llvm_c_byte(src[start + k])
        k = k + 1
    }
    return 0
}
fn emit_literal_bytes(src: Str, start: Int, raw_len: Int) -> Int {
    if start > 0 {
        if src[start - 1] == 34 {
            let mut k = 0
            while k < raw_len {
                if src[start + k] == 92 {
                    if k + 1 < raw_len {
                        emit_llvm_c_byte(src[start + k + 1])
                        k = k + 2
                    } else {
                        emit_llvm_c_byte(src[start + k])
                        k = k + 1
                    }
                } else {
                    emit_llvm_c_byte(src[start + k])
                    k = k + 1
                }
            }
            return 0
        }
    }
    return emit_bytes(src, start, raw_len)
}
# printf format length: bytes the @.fmt<nstart> global occupies for the literal
# [start,start+len), where each `%` doubles to `%%`, each VALID `{ident}` becomes
# a 2-byte printf directive (`%d` or `%s`), a lone `{` (literal brace) passes
# through 1:1, other bytes 1:1, plus a trailing newline (puts adds one, so printf
# must too). The \00 is added by the caller.
fn fmt_len(src: Str, start: Int, len: Int) -> Int {
    let mut k = 0
    let mut out = 0
    while k < len {
        let c = src[start + k]
        let e = interp_end(src, start, len, k)
        if e >= 0 {
            # valid `{ident}` -> printf directive (2 bytes); advance past `}`
            out = out + 2
            k = e + 1
        } else if c == 37 {
            out = out + 2
            k = k + 1
        } else {
            out = out + 1
            k = k + 1
        }
    }
    # trailing newline to match puts() line semantics
    return out + 1
}
# Emit the printf format bytes for the literal [start,start+len): `%` -> `%%`,
# valid `{ident}` -> `%d` or `%s`, lone `{`/other bytes verbatim, then a trailing
# `\0A` newline (to match puts). (Caller wraps in c"..\00".)
fn emit_fmt_bytes(toks: &List<Token>, fns: &List<Fn>, src: Str, n: Int, lit_i: Int, start: Int, len: Int) -> Int {
    let mut k = 0
    while k < len {
        let c = src[start + k]
        let e = interp_end(src, start, len, k)
        if e >= 0 {
            let astart = start + k + 1
            let alen = e - (k + 1)
            putchar(37)
            if interp_name_is_str(toks, fns, src, n, lit_i, astart, alen) == 1 { putchar(115) }
            else { putchar(100) }
            k = e + 1
        } else if c == 37 {
            putchar(37)
            putchar(37)
            k = k + 1
        } else {
            emit_llvm_c_byte(c)
            k = k + 1
        }
    }
    # trailing newline, written as the LLVM escape \0A
    emit_str("\\0A")
    return 0
}

# Module pre-pass: emit a `@.s<nstart> = [len+1 x i8] c"..\00"` global for every
# string-literal token, keyed by the literal's source nstart (unique per literal).
fn emit_str_globals(toks: &List<Token>, fns: &List<Fn>, src: Str, n: Int) -> Int {
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 28 {
            emit_str("@.s")
            pint(t.nstart)
            emit_str(" = private constant [")
            pint(t.value + 1)
            emit_str(" x i8] c\"")
            emit_literal_bytes(src, t.nstart, t.nlen)
            emit_str("\\00\"")
            putchar(10)
            # a brace-bearing literal also gets a printf format global @.fmt<nstart>
            # ({ident} -> %d/%s, % -> %%) for the print(...) interpolation path.
            if lit_has_brace(src, t.nstart, t.nlen) == 1 {
                emit_str("@.fmt")
                pint(t.nstart)
                emit_str(" = private constant [")
                pint(fmt_len(src, t.nstart, t.nlen) + 1)
                emit_str(" x i8] c\"")
                emit_fmt_bytes(toks, fns, src, n, i, t.nstart, t.nlen)
                emit_str("\\00\"")
                putchar(10)
            }
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
# Return callee param type encoding for argument position `argpos`:
#   0 Int, 1 Str, 2 List, 3+struct_index plain struct.
fn call_param_ty(fns: &List<Fn>, src: Str, qs: Int, ql: Int, argpos: Int) -> Int {
    let idx = find_fn(fns, src, qs, ql)
    if idx < 0 { return 0 }
    let f = fns[idx]
    if argpos == 0 { return f.p0ty }
    if argpos == 1 { return f.p1ty }
    if argpos == 2 { return f.p2ty }
    if argpos == 3 { return f.p3ty }
    if argpos == 4 { return f.p4ty }
    if argpos == 5 { return f.p5ty }
    if argpos == 6 { return f.p6ty }
    if argpos == 7 { return f.p7ty }
    if argpos == 8 { return f.p8ty }
    return f.p9ty
}
# If the token at `vp` is a call `<ident> (` to a List-returning function, returns
# that function's retty (element struct type, >= -1). Returns -2 if the RHS is not
# a call to a list-returning function. Lets `let ys = build()` size ys's buffer
# and pass it as the callee's hidden out-param.
fn call_retty(toks: &List<Token>, fns: &List<Fn>, src: Str, vp: Int) -> Int {
    let r = toks[vp]
    if r.kind == 1 {
        let after = toks[vp + 1]
        if after.kind == 9 {
            let idx = find_fn(fns, src, r.nstart, r.nlen)
            if idx >= 0 {
                let f = fns[idx]
                if f.retlist == 1 { return f.retty }
            }
        }
    }
    return 0 - 2
}
# If the token at `vp` is a call `<ident> (` to a struct-returning function,
# returns that function's struct type index. Returns -2 otherwise.
fn call_retsty(toks: &List<Token>, fns: &List<Fn>, src: Str, vp: Int) -> Int {
    let r = toks[vp]
    if r.kind == 1 {
        let after = toks[vp + 1]
        if after.kind == 9 {
            let idx = find_fn(fns, src, r.nstart, r.nlen)
            if idx >= 0 {
                let f = fns[idx]
                if f.retlist == 2 { return f.retty }
            }
        }
    }
    return 0 - 2
}

# Build the function table by scanning for `fn name ( param ) { ... }`.
fn build_fns(toks: &List<Token>, defs: &List<StructDef>, src: Str, n: Int) -> List<Fn> {
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
            let mut p8s = 0
            let mut p8l = 0
            let mut p9s = 0
            let mut p9l = 0
            let mut p0ty = 0
            let mut p1ty = 0
            let mut p2ty = 0
            let mut p3ty = 0
            let mut p4ty = 0
            let mut p5ty = 0
            let mut p6ty = 0
            let mut p7ty = 0
            let mut p8ty = 0
            let mut p9ty = 0
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
                    #   StructName  -> 3+struct_index  (struct by hidden pointer)
                    #   else        -> 0  (int)
                    let tyt = toks[q + 1]
                    let isstr = kw3(src, tyt.nstart, tyt.nlen, 83, 116, 114)
                    let islist = kw4(src, tyt.nstart, tyt.nlen, 76, 105, 115, 116)
                    let mut pty = isstr
                    if islist == 1 { pty = 2 }
                    else if isstr == 0 {
                        let pst = struct_index_by_name(defs, src, tyt.nstart, tyt.nlen)
                        if pst >= 0 { pty = 3 + pst }
                    }
                    # the type applies to the most recent param (index npar-1).
                    if npar == 1 { p0ty = pty }
                    else if npar == 2 { p1ty = pty }
                    else if npar == 3 { p2ty = pty }
                    else if npar == 4 { p3ty = pty }
                    else if npar == 5 { p4ty = pty }
                    else if npar == 6 { p5ty = pty }
                    else if npar == 7 { p6ty = pty }
                    else if npar == 8 { p7ty = pty }
                    else if npar == 9 { p8ty = pty }
                    else { p9ty = pty }
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
                    else if npar == 7 { p7s = qt.nstart; p7l = qt.nlen }
                    else if npar == 8 { p8s = qt.nstart; p8l = qt.nlen }
                    else { p9s = qt.nstart; p9l = qt.nlen }
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
            # Return type: `-> List<Type>` or `-> StructName` between ')' and
            # '{'. Both use a hidden out-param: retlist=1 for List, retlist=2
            # for struct; retty is the element/struct type index (or -1).
            let mut retlist = 0
            let mut retty = 0 - 1
            let mut rq = q + 1
            while rq < bo {
                let rt = toks[rq]
                if rt.kind == 1 {
                    if kw4(src, rt.nstart, rt.nlen, 76, 105, 115, 116) == 1 {
                        let lt = toks[rq + 1]
                        if lt.kind == 18 {
                            retlist = 1
                            let et = toks[rq + 2]
                            if et.kind == 1 {
                                retty = struct_index_by_name(defs, src, et.nstart, et.nlen)
                            }
                        }
                    } else if retlist == 0 {
                        let rst = struct_index_by_name(defs, src, rt.nstart, rt.nlen)
                        if rst >= 0 {
                            retlist = 2
                            retty = rst
                        }
                    }
                }
                rq = rq + 1
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
                p8s: p8s, p8l: p8l, p9s: p9s, p9l: p9l,
                p0ty: p0ty, p1ty: p1ty, p2ty: p2ty, p3ty: p3ty,
                p4ty: p4ty, p5ty: p5ty, p6ty: p6ty, p7ty: p7ty, p8ty: p8ty, p9ty: p9ty,
                npar: npar,
                bstart: bstart, bend: be,
                retlist: retlist, retty: retty
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
            let mut f6s = 0
            let mut f6l = 0
            let mut f7s = 0
            let mut f7l = 0
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
                    else if cnt == 5 { f5s = qt.nstart; f5l = qt.nlen }
                    else if cnt == 6 { f6s = qt.nstart; f6l = qt.nlen }
                    else { f7s = qt.nstart; f7l = qt.nlen }
                    cnt = cnt + 1
                }
                q = q + 1
            }
            defs.push(StructDef {
                nstart: nt.nstart, nlen: nt.nlen,
                f0s: f0s, f0l: f0l, f1s: f1s, f1l: f1l, f2s: f2s, f2l: f2l,
                f3s: f3s, f3l: f3l, f4s: f4s, f4l: f4l, f5s: f5s, f5l: f5l,
                f6s: f6s, f6l: f6l, f7s: f7s, f7l: f7l,
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
fn known_field_index(src: Str, ts: Int, tl: Int, qs: Int, ql: Int) -> Int {
    if kw2(src, ts, tl, 70, 110) == 1 {
        if kw6(src, qs, ql, 110, 115, 116, 97, 114, 116) == 1 { return 0 }
        if kw4(src, qs, ql, 110, 108, 101, 110) == 1 { return 1 }
        if ql == 3 {
            if src[qs] == 112 {
                let pd = src[qs + 1] - 48
                if pd >= 0 and pd <= 9 {
                    if src[qs + 2] == 115 { return 2 + pd * 2 }
                    if src[qs + 2] == 108 { return 3 + pd * 2 }
                }
            }
        }
        if ql == 4 {
            if src[qs] == 112 {
                let pd2 = src[qs + 1] - 48
                if pd2 >= 0 and pd2 <= 9 {
                    if src[qs + 2] == 116 and src[qs + 3] == 121 { return 22 + pd2 }
                }
            }
            if kw4(src, qs, ql, 110, 112, 97, 114) == 1 { return 32 }
            if kw4(src, qs, ql, 98, 101, 110, 100) == 1 { return 34 }
        }
        if kw6(src, qs, ql, 98, 115, 116, 97, 114, 116) == 1 { return 33 }
        if ql == 7 {
            if src[qs] == 114 and src[qs + 1] == 101 and src[qs + 2] == 116 and src[qs + 3] == 108 and src[qs + 4] == 105 and src[qs + 5] == 115 and src[qs + 6] == 116 { return 35 }
        }
        if kw5(src, qs, ql, 114, 101, 116, 116, 121) == 1 { return 36 }
    }
    if tl == 9 {
        if src[ts] == 83 and src[ts + 1] == 116 and src[ts + 2] == 114 and src[ts + 3] == 117 and src[ts + 4] == 99 and src[ts + 5] == 116 and src[ts + 6] == 68 and src[ts + 7] == 101 and src[ts + 8] == 102 {
            if kw6(src, qs, ql, 110, 115, 116, 97, 114, 116) == 1 { return 0 }
            if kw4(src, qs, ql, 110, 108, 101, 110) == 1 { return 1 }
            if ql == 3 {
                if src[qs] == 102 {
                    let fd = src[qs + 1] - 48
                    if fd >= 0 and fd <= 7 {
                        if src[qs + 2] == 115 { return 2 + fd * 2 }
                        if src[qs + 2] == 108 { return 3 + fd * 2 }
                    }
                }
            }
            if ql == 7 {
                if src[qs] == 110 and src[qs + 1] == 102 and src[qs + 2] == 105 and src[qs + 3] == 101 and src[qs + 4] == 108 and src[qs + 5] == 100 and src[qs + 6] == 115 { return 18 }
            }
        }
    }
    return 0 - 1
}
fn field_index(defs: &List<StructDef>, ti: Int, src: Str, qs: Int, ql: Int) -> Int {
    let d = defs[ti]
    let mut known = 0 - 1
    if d.nfields >= 19 { known = known_field_index(src, d.nstart, d.nlen, qs, ql) }
    if known >= 0 { return known }
    if d.nfields >= 1 { if name_eq(src, d.f0s, d.f0l, qs, ql) == 1 { return 0 } }
    if d.nfields >= 2 { if name_eq(src, d.f1s, d.f1l, qs, ql) == 1 { return 1 } }
    if d.nfields >= 3 { if name_eq(src, d.f2s, d.f2l, qs, ql) == 1 { return 2 } }
    if d.nfields >= 4 { if name_eq(src, d.f3s, d.f3l, qs, ql) == 1 { return 3 } }
    if d.nfields >= 5 { if name_eq(src, d.f4s, d.f4l, qs, ql) == 1 { return 4 } }
    if d.nfields >= 6 { if name_eq(src, d.f5s, d.f5l, qs, ql) == 1 { return 5 } }
    if d.nfields >= 7 { if name_eq(src, d.f6s, d.f6l, qs, ql) == 1 { return 6 } }
    if d.nfields >= 8 { if name_eq(src, d.f7s, d.f7l, qs, ql) == 1 { return 7 } }
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

# Fixed capacity for generated List buffers. Early self-host stages used 64,
# which is enough for snippets but not for a compiler reading a real source file
# again. Most generated Lists stay at 4096 to keep recursive compiler scopes
# stack-bounded; the 4-field Token shape gets a larger cap for file-sized tier
# inputs through fixpoint_full.nl full retarget.
fn list_cap() -> Int { return 4096 }
fn list_lenidx() -> Int { return list_cap() - 1 }
fn list_struct_cap(nf: Int) -> Int {
    if nf == 4 { return 65536 }
    return list_cap()
}
fn list_lenidx_for_nfields(nf: Int) -> Int { return list_struct_cap(nf) * nf }
fn list_bufsz_for_nfields(nf: Int) -> Int { return list_lenidx_for_nfields(nf) + 1 }
fn list_cap_for_sty(defs: &List<StructDef>, sty: Int) -> Int {
    if sty >= 0 { return list_struct_cap(struct_nfields(defs, sty)) }
    return list_cap()
}
fn list_lenidx_for_sty(defs: &List<StructDef>, sty: Int) -> Int {
    if sty >= 0 { return list_lenidx_for_nfields(struct_nfields(defs, sty)) }
    return list_lenidx()
}
fn list_bufsz_for_sty(defs: &List<StructDef>, sty: Int) -> Int {
    if sty >= 0 { return list_bufsz_for_nfields(struct_nfields(defs, sty)) }
    return list_cap()
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
            # print(...) with interpolation: the transpiler rewrote print->puts, so
            # this is a `puts(<string-lit>)` whose literal contains `{ident}`. Emit a
            # printf call against the @.fmt<nstart> format global, loading Int
            # variables as i64 varargs and Str variables as i8* varargs.
            let sarg = toks[i + 2]
            if is_puts(src, t.nstart, t.nlen) == 1 {
                if sarg.kind == 28 {
                    if lit_has_brace(src, sarg.nstart, sarg.nlen) == 1 {
                        # 1) load each {ident} operand to a temp; remember temp numbers
                        #    in a small fixed array (up to 8 interpolations).
                        let mut iv0 = 0
                        let mut iv1 = 0
                        let mut iv2 = 0
                        let mut iv3 = 0
                        let mut iv4 = 0
                        let mut iv5 = 0
                        let mut iv6 = 0
                        let mut iv7 = 0
                        let mut ik0 = 1
                        let mut ik1 = 1
                        let mut ik2 = 1
                        let mut ik3 = 1
                        let mut ik4 = 1
                        let mut ik5 = 1
                        let mut ik6 = 1
                        let mut ik7 = 1
                        let mut nv = 0
                        let mut cc2 = counter
                        let mut k = 0
                        while k < sarg.nlen {
                            let istop = interp_end(src, sarg.nstart, sarg.nlen, k)
                            if istop >= 0 {
                                # valid `{ident}`: the ident is [k+1, e); load it.
                                let astart = sarg.nstart + k + 1
                                let alen = istop - (k + 1)
                                let vslot = find_slot(slots, src, astart, alen)
                                let is_str_arg = isarr_of(slots, src, astart, alen) == 3
                                emit_str("  %t")
                                pint(cc2)
                                if is_str_arg { emit_str(" = load i8*, i8** %v") }
                                else { emit_str(" = load i64, i64* %v") }
                                pint(vslot)
                                putchar(10)
                                let mut vk = 1
                                if is_str_arg { vk = 2 }
                                if nv == 0 { iv0 = cc2; ik0 = vk }
                                else if nv == 1 { iv1 = cc2; ik1 = vk }
                                else if nv == 2 { iv2 = cc2; ik2 = vk }
                                else if nv == 3 { iv3 = cc2; ik3 = vk }
                                else if nv == 4 { iv4 = cc2; ik4 = vk }
                                else if nv == 5 { iv5 = cc2; ik5 = vk }
                                else if nv == 6 { iv6 = cc2; ik6 = vk }
                                else { iv7 = cc2; ik7 = vk }
                                nv = nv + 1
                                cc2 = cc2 + 1
                                k = istop + 1
                            } else {
                                k = k + 1
                            }
                        }
                        # 2) emit the printf call with the format global + i64 varargs
                        let flen = fmt_len(src, sarg.nstart, sarg.nlen) + 1
                        let dest = cc2
                        emit_str("  %t")
                        pint(dest)
                        emit_str(" = call i32 (i8*, ...) @printf(i8* getelementptr([")
                        pint(flen)
                        emit_str(" x i8], [")
                        pint(flen)
                        emit_str(" x i8]* @.fmt")
                        pint(sarg.nstart)
                        emit_str(", i64 0, i64 0)")
                        let mut vi = 0
                        while vi < nv {
                            let mut vt = iv0
                            let mut vk = ik0
                            if vi == 1 { vt = iv1; vk = ik1 }
                            else if vi == 2 { vt = iv2; vk = ik2 }
                            else if vi == 3 { vt = iv3; vk = ik3 }
                            else if vi == 4 { vt = iv4; vk = ik4 }
                            else if vi == 5 { vt = iv5; vk = ik5 }
                            else if vi == 6 { vt = iv6; vk = ik6 }
                            else if vi == 7 { vt = iv7; vk = ik7 }
                            if vk == 2 { emit_str(", i8* %t") }
                            else { emit_str(", i64 %t") }
                            pint(vt)
                            vi = vi + 1
                        }
                        emit_str(")")
                        putchar(10)
                        return Op { kind: 1, val: dest, next: dest + 1 }
                    }
                }
            }
            # call: name ( arg0 [, ... up to arg9] ) — 0..10 args.
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
            let mut a8k = 0
            let mut a8v = 0
            let mut a9k = 0
            let mut a9v = 0
            # slots of List-local args passed by pointer, to sync length AFTER the
            # call (the callee may have pushed). -1 = none. li/lb carry the length
            # index + buffer array-size per arg (scalar: 63/64; struct: 64*nf/64*nf+1).
            let mut ls0 = 0 - 1
            let mut ls1 = 0 - 1
            let mut ls2 = 0 - 1
            let mut ls3 = 0 - 1
            let mut ls4 = 0 - 1
            let mut ls5 = 0 - 1
            let mut ls6 = 0 - 1
            let mut ls7 = 0 - 1
            let mut ls8 = 0 - 1
            let mut ls9 = 0 - 1
            let mut li0 = list_lenidx()
            let mut li1 = list_lenidx()
            let mut li2 = list_lenidx()
            let mut li3 = list_lenidx()
            let mut li4 = list_lenidx()
            let mut li5 = list_lenidx()
            let mut li6 = list_lenidx()
            let mut li7 = list_lenidx()
            let mut li8 = list_lenidx()
            let mut li9 = list_lenidx()
            let mut lb0 = list_cap()
            let mut lb1 = list_cap()
            let mut lb2 = list_cap()
            let mut lb3 = list_cap()
            let mut lb4 = list_cap()
            let mut lb5 = list_cap()
            let mut lb6 = list_cap()
            let mut lb7 = list_cap()
            let mut lb8 = list_cap()
            let mut lb9 = list_cap()
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
                        let after = toks[q + 1]
                        if astop == q + 1 {
                            let want_ty = call_param_ty(fns, src, t.nstart, t.nlen, nargs)
                            let aarr = isarr_of(slots, src, argt.nstart, argt.nlen)
                            if aarr == 2 {
                                let lslot = find_slot(slots, src, argt.nstart, argt.nlen)
                                # length index + buffer size depend on element kind:
                                #   scalar List  -> length at [list_lenidx()], buffer [list_cap() x i64]
                                #   List<struct> -> length at [cap*nf], buffer [cap*nf+1]
                                let mut alsty = sty_of(slots, src, argt.nstart, argt.nlen)
                                let pst = param_list_elem_sty(toks, defs, src, toks.len(), t.nstart, t.nlen, nargs)
                                if pst >= 0 { alsty = pst }
                                let mut alenidx = list_lenidx()
                                let mut albufsz = list_cap()
                                if alsty >= 0 {
                                    let anf = struct_nfields(defs, alsty)
                                    alenidx = list_lenidx_for_nfields(anf)
                                    albufsz = list_bufsz_for_nfields(anf)
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
                                else if nargs == 4 { ls4 = lslot; li4 = alenidx; lb4 = albufsz }
                                else if nargs == 5 { ls5 = lslot; li5 = alenidx; lb5 = albufsz }
                                else if nargs == 6 { ls6 = lslot; li6 = alenidx; lb6 = albufsz }
                                else if nargs == 7 { ls7 = lslot; li7 = alenidx; lb7 = albufsz }
                                else if nargs == 8 { ls8 = lslot; li8 = alenidx; lb8 = albufsz }
                                else { ls9 = lslot; li9 = alenidx; lb9 = albufsz }
                                handled = 1
                            } else if aarr == 4 {
                                # List parameter forwarded to another function:
                                # load its i64* buffer pointer and pass it through.
                                let lslot = find_slot(slots, src, argt.nstart, argt.nlen)
                                emit_str("  %t")
                                pint(cc)
                                emit_str(" = load i64*, i64** %v")
                                pint(lslot)
                                putchar(10)
                                ekind = 3
                                eval2 = cc
                                cc = cc + 1
                                handled = 1
                            } else if aarr == 0 {
                                # Struct local passed by value: pass a pointer to
                                # its first field; callee copies into its local.
                                let mut ast = sty_of(slots, src, argt.nstart, argt.nlen)
                                if want_ty >= 3 { ast = want_ty - 3 }
                                if ast >= 0 and want_ty >= 3 {
                                    let aslot = find_slot(slots, src, argt.nstart, argt.nlen)
                                    let anf = struct_nfields(defs, ast)
                                    emit_str("  %t")
                                    pint(cc)
                                    emit_str(" = getelementptr [")
                                    pint(anf)
                                    emit_str(" x i64], [")
                                    pint(anf)
                                    emit_str(" x i64]* %v")
                                    pint(aslot)
                                    emit_str(", i64 0, i64 0")
                                    putchar(10)
                                    ekind = 3
                                    eval2 = cc
                                    cc = cc + 1
                                    handled = 1
                                }
                            }
                        } else if after.kind == 11 {
                            # Struct literal as an argument, e.g. emit_op(Op { ... }).
                            # Materialize a temporary [nf x i64] and pass its base.
                            let ast = struct_index_by_name(defs, src, argt.nstart, argt.nlen)
                            if ast >= 0 {
                                let anf = struct_nfields(defs, ast)
                                let aid = cc
                                emit_str("  %sa")
                                pint(aid)
                                emit_str(" = alloca [")
                                pint(anf)
                                emit_str(" x i64]")
                                putchar(10)
                                cc = cc + 1
                                let bopen = q + 1
                                let bclose = match_brace(toks, bopen, close)
                                let mut fq = bopen + 1
                                while fq < bclose {
                                    let fld = toks[fq]
                                    let fi = field_index(defs, ast, src, fld.nstart, fld.nlen)
                                    let mut vstart = fq + 1
                                    let colt = toks[fq + 1]
                                    if colt.kind == 16 { vstart = fq + 2 }
                                    let vstop = arr_elem_end(toks, vstart, bclose)
                                    let fe = gen_expr(toks, slots, fns, defs, src, vstart, vstop, cc)
                                    cc = fe.next
                                    emit_str("  %t")
                                    pint(cc)
                                    emit_str(" = getelementptr [")
                                    pint(anf)
                                    emit_str(" x i64], [")
                                    pint(anf)
                                    emit_str(" x i64]* %sa")
                                    pint(aid)
                                    emit_str(", i64 0, i64 ")
                                    pint(fi)
                                    putchar(10)
                                    emit_str("  store i64 ")
                                    emit_op(fe)
                                    emit_str(", i64* %t")
                                    pint(cc)
                                    putchar(10)
                                    cc = cc + 1
                                    fq = vstop + 1
                                }
                                emit_str("  %t")
                                pint(cc)
                                emit_str(" = getelementptr [")
                                pint(anf)
                                emit_str(" x i64], [")
                                pint(anf)
                                emit_str(" x i64]* %sa")
                                pint(aid)
                                emit_str(", i64 0, i64 0")
                                putchar(10)
                                ekind = 3
                                eval2 = cc
                                cc = cc + 1
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
                    else if nargs == 7 { a7k = ekind; a7v = eval2 }
                    else if nargs == 8 { a8k = ekind; a8v = eval2 }
                    else { a9k = ekind; a9v = eval2 }
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
                else if ai == 8 { ak = a8k; av = a8v }
                else if ai == 9 { ak = a9k; av = a9v }
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
            rc = sync_list_len(ls4, li4, lb4, rc)
            rc = sync_list_len(ls5, li5, lb5, rc)
            rc = sync_list_len(ls6, li6, lb6, rc)
            rc = sync_list_len(ls7, li7, lb7, rc)
            rc = sync_list_len(ls8, li8, lb8, rc)
            rc = sync_list_len(ls9, li9, lb9, rc)
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
                let mut plenidx2 = list_lenidx()
                if plsty >= 0 { plenidx2 = list_lenidx_for_nfields(struct_nfields(defs, plsty)) }
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
            # struct field read requires a SCALAR struct var. A List var (is_arr=2)
            # carries the element struct type in sty too, but `lst.X` is ALWAYS the
            # List `.len` (a List has no addressable struct fields) -- so List vars
            # must skip this branch and fall through to the `.len` length load below.
            let sti = sty_of(slots, src, t.nstart, t.nlen)
            if sti >= 0 and karr != 2 {
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
                    let lbuf = list_bufsz_for_nfields(nf)
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
        if isarr_of(slots, src, t.nstart, t.nlen) == 3 {
            emit_str("  %t")
            pint(counter)
            emit_str(" = load i8*, i8** %v")
            pint(slot)
            putchar(10)
            return Op { kind: 2, val: counter, next: counter + 1 }
        }
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
    if t.kind == 3 or t.kind == 38 {
        let rf = gen_factor(toks, slots, fns, defs, src, i + 1, acc.next)
        let mut dest = 0
        if t.kind == 38 { dest = emit_binop("sdiv", acc, rf, rf.next) }
        else { dest = emit_binop("mul", acc, rf, rf.next) }
        let nacc = Op { kind: 1, val: dest, next: dest + 1 }
        let after = skip_factor(toks, i + 1)
        return gen_term(toks, slots, fns, defs, src, after, stop, nacc)
    }
    return acc
}
fn skip_term(toks: &List<Token>, i: Int, stop: Int) -> Int {
    if i >= stop { return stop }
    let t = toks[i]
    if t.kind == 3 or t.kind == 38 {
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
# Find the end of a call argument: next ',' or ')' at delimiter-depth 0 (so
# nested calls, struct literals, and array literals are skipped). `close` bounds
# the scan (the call's ')').
fn arg_comma_end(toks: &List<Token>, i: Int, close: Int) -> Int {
    let mut j = i
    let mut depth = 0
    let mut bdepth = 0
    let mut sdepth = 0
    let mut go = true
    while go {
        if j >= close { go = false }
        else {
            let t = toks[j]
            if t.kind == 9 { depth = depth + 1; j = j + 1 }
            else if t.kind == 10 {
                if depth == 0 { go = false } else { depth = depth - 1; j = j + 1 }
            }
            else if t.kind == 11 { bdepth = bdepth + 1; j = j + 1 }
            else if t.kind == 12 { if bdepth == 0 { j = j + 1 } else { bdepth = bdepth - 1; j = j + 1 } }
            else if t.kind == 23 { sdepth = sdepth + 1; j = j + 1 }
            else if t.kind == 24 { if sdepth == 0 { j = j + 1 } else { sdepth = sdepth - 1; j = j + 1 } }
            else if t.kind == 25 {
                if depth == 0 and bdepth == 0 and sdepth == 0 { go = false } else { j = j + 1 }
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
            let bend = bracket_end(toks, vp + 2)
            let dot = toks[bend + 1]
            if dot.kind == 27 { return 0 - 1 }
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

fn add_local_slots(base: List<Slot>, toks: &List<Token>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, start: Int, end: Int, slot0: Int, n: Int) -> List<Slot> {
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
            } else if call_retsty(toks, fns, src, vp) != 0 - 2 {
                # `let op = gen_expr(...)` where gen_expr returns a struct: allocate
                # a local [nf x i64] result buffer and pass it as a hidden out-param.
                let rst = call_retsty(toks, fns, src, vp)
                let nf = struct_nfields(defs, rst)
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 0, alen: 0 , sty: rst })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [")
                pint(nf)
                emit_str(" x i64]")
                putchar(10)
                next_slot = next_slot + 1
            } else if call_retty(toks, fns, src, vp) != 0 - 2 {
                # `let ys = build()` where build returns a List: ys is a List local
                # ([64*nf+1 x i64] for struct elems, sty = retty) that the callee
                # fills via the hidden out-param. Allocate buffer + length slot here;
                # the call + length-sync are emitted in gen_stmts.
                let rty = call_retty(toks, fns, src, vp)
                let cbuf = list_bufsz_for_sty(defs, rty)
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 2, alen: list_cap_for_sty(defs, rty) , sty: rty })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [")
                pint(cbuf)
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
            } else if rhs.kind == 1 and find_semi(toks, vp, end) == vp + 1 and isarr_of(&slots, src, rhs.nstart, rhs.nlen) == 4 {
                # `let mut xs = list_param` aliases an incoming List buffer pointer.
                # This keeps self-host helper patterns like `let mut slots = base`
                # from being mistaken for scalar integers.
                let asty = sty_of(&slots, src, rhs.nstart, rhs.nlen)
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 4, alen: 0 , sty: asty })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca i64*")
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
                let mut lcap = list_cap()
                let mut lbuf = list_cap()
                # struct-element Lists: [64*nf + 1 x i64] -- the +1 reserves a length
                # header at index 64*nf so the buffer can be passed by-pointer as a
                # List-of-structs param (length stored there, past the data region;
                # avoids the buf[63] collision that stride>1 would otherwise hit).
                if lest >= 0 {
                    let lnf = struct_nfields(defs, lest)
                    lcap = list_struct_cap(lnf)
                    lbuf = list_bufsz_for_nfields(lnf)
                }
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 2, alen: lcap , sty: lest })
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
        } else if t.kind == 34 {
            # `for <var> in lo .. hi { body }` -- the loop variable is a scalar
            # i64 local (alloca at %v<slot>); its slot is reserved here, the loop
            # codegen is in gen_stmts.
            let fv = toks[i + 1]
            slots.push(Slot { nstart: fv.nstart, nlen: fv.nlen, slot: next_slot, is_arr: 0, alen: 0 , sty: 0 - 1 })
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
# Does the statement range [start, end) end such that control cannot fall through
# (i.e. every path returns)? True if the range is a `return` statement, or an
# if / else-if / else chain where the then-block and the whole else region both
# end in a return (recursively). Used so the if-handler can mark a provably-dead
# merge block with `unreachable` (LLVM requires every block to have a terminator;
# an all-branches-return if otherwise leaves an empty trailing merge block).
fn block_returns(toks: &List<Token>, start: Int, end: Int) -> Int {
    if start >= end { return 0 }
    let t = toks[start]
    if t.kind == 8 {
        # a `return` statement -- but only conclusive if it's the LAST statement.
        let semi = find_semi(toks, start + 1, end)
        if semi + 1 >= end { return 1 }
        return 0
    }
    if t.kind == 15 {
        # an `if` chain. Conclusive only if it spans to `end` and all branches return.
        let chainend = if_stmt_end(toks, start, end)
        if chainend < end { return 0 }
        # then-block open-brace
        let mut bo = start + 1
        let mut g = true
        while g {
            if bo >= end {
                g = false
            } else {
                let bt = toks[bo]
                if bt.kind == 11 {
                    g = false
                } else {
                    bo = bo + 1
                }
            }
        }
        let bclose = match_brace(toks, bo, end)
        if block_returns(toks, bo + 1, bclose) == 0 { return 0 }
        # must have an else (otherwise control can fall through when cond false)
        let at = bclose + 1
        if at >= end { return 0 }
        let et = toks[at]
        if et.kind != 17 { return 0 }
        let nxt = toks[at + 1]
        if nxt.kind == 15 {
            # else-if: recurse over the nested chain
            return block_returns(toks, at + 1, chainend)
        }
        # plain else: its brace interior must return
        let mut eo = at + 1
        let mut g2 = true
        while g2 {
            if eo >= end {
                g2 = false
            } else {
                let ot = toks[eo]
                if ot.kind == 11 {
                    g2 = false
                } else {
                    eo = eo + 1
                }
            }
        }
        let eclose = match_brace(toks, eo, end)
        return block_returns(toks, eo + 1, eclose)
    }
    return 0
}

# Emit a call to a struct-returning function. Regular args are evaluated first;
# a hidden trailing `i64*` out-param is then passed as `%t<out_val>`. `vp` is the
# call-name token index. Keep this helper at <=10 params: the self compiler's
# function metadata only tracks ten parameter names safely.
fn emit_struct_out_call(toks: &List<Token>, slots: &List<Slot>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, vp: Int, q0: Int, close: Int, counter: Int, out_val: Int) -> Int {
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
    let mut a8k = 0
    let mut a8v = 0
    let mut a9k = 0
    let mut a9v = 0
    let mut ls0 = 0 - 1
    let mut ls1 = 0 - 1
    let mut ls2 = 0 - 1
    let mut ls3 = 0 - 1
    let mut ls4 = 0 - 1
    let mut ls5 = 0 - 1
    let mut ls6 = 0 - 1
    let mut ls7 = 0 - 1
    let mut ls8 = 0 - 1
    let mut ls9 = 0 - 1
    let mut li0 = list_lenidx()
    let mut li1 = list_lenidx()
    let mut li2 = list_lenidx()
    let mut li3 = list_lenidx()
    let mut li4 = list_lenidx()
    let mut li5 = list_lenidx()
    let mut li6 = list_lenidx()
    let mut li7 = list_lenidx()
    let mut li8 = list_lenidx()
    let mut li9 = list_lenidx()
    let mut lb0 = list_cap()
    let mut lb1 = list_cap()
    let mut lb2 = list_cap()
    let mut lb3 = list_cap()
    let mut lb4 = list_cap()
    let mut lb5 = list_cap()
    let mut lb6 = list_cap()
    let mut lb7 = list_cap()
    let mut lb8 = list_cap()
    let mut lb9 = list_cap()
    let mut cc = counter
    let mut q = q0
    let mut ga = true
    while ga {
        if q >= close { ga = false }
        else {
            let astop = arg_comma_end(toks, q, close)
            let argt = toks[q]
            let mut ekind = 0
            let mut eval2 = 0
            let mut handled = 0
            if argt.kind == 1 {
                let after = toks[q + 1]
                if astop == q + 1 {
                    let cname_arg = toks[vp]
                    let want_ty = call_param_ty(fns, src, cname_arg.nstart, cname_arg.nlen, nargs)
                    let aarr = isarr_of(slots, src, argt.nstart, argt.nlen)
                    if aarr == 2 {
                        let lslot = find_slot(slots, src, argt.nstart, argt.nlen)
                        let mut alsty = sty_of(slots, src, argt.nstart, argt.nlen)
                        let pst = param_list_elem_sty(toks, defs, src, toks.len(), cname_arg.nstart, cname_arg.nlen, nargs)
                        if pst >= 0 { alsty = pst }
                        let mut alenidx = list_lenidx()
                        let mut albufsz = list_cap()
                        if alsty >= 0 {
                            let anf = struct_nfields(defs, alsty)
                            alenidx = list_lenidx_for_nfields(anf)
                            albufsz = list_bufsz_for_nfields(anf)
                        }
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
                        if nargs == 0 { ls0 = lslot; li0 = alenidx; lb0 = albufsz }
                        else if nargs == 1 { ls1 = lslot; li1 = alenidx; lb1 = albufsz }
                        else if nargs == 2 { ls2 = lslot; li2 = alenidx; lb2 = albufsz }
                        else if nargs == 3 { ls3 = lslot; li3 = alenidx; lb3 = albufsz }
                        else if nargs == 4 { ls4 = lslot; li4 = alenidx; lb4 = albufsz }
                        else if nargs == 5 { ls5 = lslot; li5 = alenidx; lb5 = albufsz }
                        else if nargs == 6 { ls6 = lslot; li6 = alenidx; lb6 = albufsz }
                        else if nargs == 7 { ls7 = lslot; li7 = alenidx; lb7 = albufsz }
                        else if nargs == 8 { ls8 = lslot; li8 = alenidx; lb8 = albufsz }
                        else { ls9 = lslot; li9 = alenidx; lb9 = albufsz }
                        handled = 1
                    } else if aarr == 4 {
                        let lslot = find_slot(slots, src, argt.nstart, argt.nlen)
                        emit_str("  %t")
                        pint(cc)
                        emit_str(" = load i64*, i64** %v")
                        pint(lslot)
                        putchar(10)
                        ekind = 3
                        eval2 = cc
                        cc = cc + 1
                        handled = 1
                    } else if aarr == 0 {
                        let mut ast = sty_of(slots, src, argt.nstart, argt.nlen)
                        if want_ty >= 3 { ast = want_ty - 3 }
                        if ast >= 0 and want_ty >= 3 {
                            let aslot = find_slot(slots, src, argt.nstart, argt.nlen)
                            let anf = struct_nfields(defs, ast)
                            emit_str("  %t")
                            pint(cc)
                            emit_str(" = getelementptr [")
                            pint(anf)
                            emit_str(" x i64], [")
                            pint(anf)
                            emit_str(" x i64]* %v")
                            pint(aslot)
                            emit_str(", i64 0, i64 0")
                            putchar(10)
                            ekind = 3
                            eval2 = cc
                            cc = cc + 1
                            handled = 1
                        }
                    }
                } else if after.kind == 11 {
                    let ast = struct_index_by_name(defs, src, argt.nstart, argt.nlen)
                    if ast >= 0 {
                        let anf = struct_nfields(defs, ast)
                        let aid = cc
                        emit_str("  %sa")
                        pint(aid)
                        emit_str(" = alloca [")
                        pint(anf)
                        emit_str(" x i64]")
                        putchar(10)
                        cc = cc + 1
                        let bopen = q + 1
                        let bclose = match_brace(toks, bopen, close)
                        let mut fq = bopen + 1
                        while fq < bclose {
                            let fld = toks[fq]
                            let fi = field_index(defs, ast, src, fld.nstart, fld.nlen)
                            let mut vstart = fq + 1
                            let colt = toks[fq + 1]
                            if colt.kind == 16 { vstart = fq + 2 }
                            let vstop = arr_elem_end(toks, vstart, bclose)
                            let fe = gen_expr(toks, slots, fns, defs, src, vstart, vstop, cc)
                            cc = fe.next
                            emit_str("  %t")
                            pint(cc)
                            emit_str(" = getelementptr [")
                            pint(anf)
                            emit_str(" x i64], [")
                            pint(anf)
                            emit_str(" x i64]* %sa")
                            pint(aid)
                            emit_str(", i64 0, i64 ")
                            pint(fi)
                            putchar(10)
                            emit_str("  store i64 ")
                            emit_op(fe)
                            emit_str(", i64* %t")
                            pint(cc)
                            putchar(10)
                            cc = cc + 1
                            fq = vstop + 1
                        }
                        emit_str("  %t")
                        pint(cc)
                        emit_str(" = getelementptr [")
                        pint(anf)
                        emit_str(" x i64], [")
                        pint(anf)
                        emit_str(" x i64]* %sa")
                        pint(aid)
                        emit_str(", i64 0, i64 0")
                        putchar(10)
                        ekind = 3
                        eval2 = cc
                        cc = cc + 1
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
            else if nargs == 7 { a7k = ekind; a7v = eval2 }
            else if nargs == 8 { a8k = ekind; a8v = eval2 }
            else { a9k = ekind; a9v = eval2 }
            nargs = nargs + 1
            q = astop + 1
        }
    }
    emit_str("  call void @")
    let cname2 = toks[vp]
    emit_name(src, cname2.nstart, cname2.nlen)
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
        else if ai == 8 { ak = a8k; av = a8v }
        else if ai == 9 { ak = a9k; av = a9v }
        if ak == 2 { emit_str("i8* ") } else if ak == 3 { emit_str("i64* ") } else { emit_str("i64 ") }
        emit_op(Op { kind: ak, val: av, next: 0 })
        ai = ai + 1
    }
    if nargs > 0 { emit_str(", ") }
    emit_str("i64* %t")
    pint(out_val)
    emit_str(")")
    putchar(10)
    let mut rc = cc
    rc = sync_list_len(ls0, li0, lb0, rc)
    rc = sync_list_len(ls1, li1, lb1, rc)
    rc = sync_list_len(ls2, li2, lb2, rc)
    rc = sync_list_len(ls3, li3, lb3, rc)
    rc = sync_list_len(ls4, li4, lb4, rc)
    rc = sync_list_len(ls5, li5, lb5, rc)
    rc = sync_list_len(ls6, li6, lb6, rc)
    rc = sync_list_len(ls7, li7, lb7, rc)
    rc = sync_list_len(ls8, li8, lb8, rc)
    rc = sync_list_len(ls9, li9, lb9, rc)
    return rc
}

# Generate code for the statements in token range [i, end). Returns the next free
# SSA temp. Handles `let`, assignment, `return`, and `while` (recursing for the
# loop body). Labels reuse temp numbers to stay unique.
fn gen_stmts(toks: &List<Token>, slots: &List<Slot>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, i0: Int, end: Int, counter0: Int, retout: Int, retkind: Int) -> Int {
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
                let lbufsz = list_bufsz_for_nfields(lnf)
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
            } else if call_retsty(toks, fns, src, vp) != 0 - 2 {
                # `let op = gen_expr(...)` where gen_expr returns a struct. The
                # local struct buffer was allocated in add_local_slots; pass its
                # first field pointer as the callee's hidden out-param.
                let rst = call_retsty(toks, fns, src, vp)
                let nf = struct_nfields(defs, rst)
                let cclose = paren_end(toks, vp + 2)
                emit_str("  %t")
                pint(counter)
                emit_str(" = getelementptr [")
                pint(nf)
                emit_str(" x i64], [")
                pint(nf)
                emit_str(" x i64]* %v")
                pint(slot)
                emit_str(", i64 0, i64 0")
                putchar(10)
                let outp = counter
                counter = counter + 1
                counter = emit_struct_out_call(toks, slots, fns, defs, src, vp, vp + 2, cclose, counter, outp)
                let stop = find_semi(toks, cclose, end)
                i = stop + 1
            } else if call_retty(toks, fns, src, vp) != 0 - 2 {
                # `let ys = build(args)` -- build returns a List. Emit the call with
                # build's regular args plus a hidden trailing `i64*` = ys's buffer
                # base; build copies its result into it. Then sync ys.len from the
                # out-param's length slot (buf[63] scalar / buf[64*nf] struct).
                let cname = toks[vp]
                let rty = call_retty(toks, fns, src, vp)
                let mut stride = 1
                let mut lenidx = list_lenidx()
                let mut cbuf = list_cap()
                if rty >= 0 {
                    stride = struct_nfields(defs, rty)
                    lenidx = list_lenidx_for_nfields(stride)
                    cbuf = list_bufsz_for_nfields(stride)
                }
                let cclose = paren_end(toks, vp + 2)
                # evaluate each comma-separated arg (scalar/string/List) into Ops.
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
                let mut a8k = 0
                let mut a8v = 0
                let mut a9k = 0
                let mut a9v = 0
                let mut nca = 0
                let mut cc = counter
                let mut q = vp + 2
                let mut ga = true
                while ga {
                    if q >= cclose { ga = false }
                    else {
                        let astop = arg_comma_end(toks, q, cclose)
                        let argt = toks[q]
                        let mut ek = 0
                        let mut ev = 0
                        let mut handled = 0
                        if argt.kind == 1 {
                            let after = toks[q + 1]
                            if astop == q + 1 {
                                let want_ty = call_param_ty(fns, src, cname.nstart, cname.nlen, nca)
                                let aarr = isarr_of(slots, src, argt.nstart, argt.nlen)
                                if aarr == 2 {
                                    # local List arg passed by pointer (write len->buf[63] then base)
                                    let lslot = find_slot(slots, src, argt.nstart, argt.nlen)
                                    let mut alsty = sty_of(slots, src, argt.nstart, argt.nlen)
                                    let pst = param_list_elem_sty(toks, defs, src, toks.len(), cname.nstart, cname.nlen, nca)
                                    if pst >= 0 { alsty = pst }
                                    let mut albuf = list_cap()
                                    let mut alidx = list_lenidx()
                                    if alsty >= 0 {
                                        let anf2 = struct_nfields(defs, alsty)
                                        albuf = list_bufsz_for_nfields(anf2)
                                        alidx = list_lenidx_for_nfields(anf2)
                                    }
                                    emit_str("  %t")
                                    pint(cc)
                                    emit_str(" = load i64, i64* %v")
                                    pint(lslot + 1)
                                    putchar(10)
                                    let lc = cc
                                    emit_str("  %t")
                                    pint(lc + 1)
                                    emit_str(" = getelementptr [")
                                    pint(albuf)
                                    emit_str(" x i64], [")
                                    pint(albuf)
                                    emit_str(" x i64]* %v")
                                    pint(lslot)
                                    emit_str(", i64 0, i64 ")
                                    pint(alidx)
                                    putchar(10)
                                    emit_str("  store i64 %t")
                                    pint(lc)
                                    emit_str(", i64* %t")
                                    pint(lc + 1)
                                    putchar(10)
                                    emit_str("  %t")
                                    pint(lc + 2)
                                    emit_str(" = getelementptr [")
                                    pint(albuf)
                                    emit_str(" x i64], [")
                                    pint(albuf)
                                    emit_str(" x i64]* %v")
                                    pint(lslot)
                                    emit_str(", i64 0, i64 0")
                                    putchar(10)
                                    ek = 3
                                    ev = lc + 2
                                    cc = lc + 3
                                    handled = 1
                                } else if aarr == 4 {
                                    # List parameter forwarded to a List-returning call.
                                    let lslot = find_slot(slots, src, argt.nstart, argt.nlen)
                                    emit_str("  %t")
                                    pint(cc)
                                    emit_str(" = load i64*, i64** %v")
                                    pint(lslot)
                                    putchar(10)
                                    ek = 3
                                    ev = cc
                                    cc = cc + 1
                                    handled = 1
                                } else if aarr == 0 {
                                    # Struct local passed by value: pass its base pointer.
                                    let mut ast = sty_of(slots, src, argt.nstart, argt.nlen)
                                    if want_ty >= 3 { ast = want_ty - 3 }
                                    if ast >= 0 and want_ty >= 3 {
                                        let aslot = find_slot(slots, src, argt.nstart, argt.nlen)
                                        let anf = struct_nfields(defs, ast)
                                        emit_str("  %t")
                                        pint(cc)
                                        emit_str(" = getelementptr [")
                                        pint(anf)
                                        emit_str(" x i64], [")
                                        pint(anf)
                                        emit_str(" x i64]* %v")
                                        pint(aslot)
                                        emit_str(", i64 0, i64 0")
                                        putchar(10)
                                        ek = 3
                                        ev = cc
                                        cc = cc + 1
                                        handled = 1
                                    }
                                }
                            } else if after.kind == 11 {
                                # Struct literal as an argument to a List-returning call.
                                let ast = struct_index_by_name(defs, src, argt.nstart, argt.nlen)
                                if ast >= 0 {
                                    let anf = struct_nfields(defs, ast)
                                    let aid = cc
                                    emit_str("  %sa")
                                    pint(aid)
                                    emit_str(" = alloca [")
                                    pint(anf)
                                    emit_str(" x i64]")
                                    putchar(10)
                                    cc = cc + 1
                                    let bopen = q + 1
                                    let bclose = match_brace(toks, bopen, cclose)
                                    let mut fq = bopen + 1
                                    while fq < bclose {
                                        let fld = toks[fq]
                                        let fi = field_index(defs, ast, src, fld.nstart, fld.nlen)
                                        let mut vstart = fq + 1
                                        let colt = toks[fq + 1]
                                        if colt.kind == 16 { vstart = fq + 2 }
                                        let vstop = arr_elem_end(toks, vstart, bclose)
                                        let fe = gen_expr(toks, slots, fns, defs, src, vstart, vstop, cc)
                                        cc = fe.next
                                        emit_str("  %t")
                                        pint(cc)
                                        emit_str(" = getelementptr [")
                                        pint(anf)
                                        emit_str(" x i64], [")
                                        pint(anf)
                                        emit_str(" x i64]* %sa")
                                        pint(aid)
                                        emit_str(", i64 0, i64 ")
                                        pint(fi)
                                        putchar(10)
                                        emit_str("  store i64 ")
                                        emit_op(fe)
                                        emit_str(", i64* %t")
                                        pint(cc)
                                        putchar(10)
                                        cc = cc + 1
                                        fq = vstop + 1
                                    }
                                    emit_str("  %t")
                                    pint(cc)
                                    emit_str(" = getelementptr [")
                                    pint(anf)
                                    emit_str(" x i64], [")
                                    pint(anf)
                                    emit_str(" x i64]* %sa")
                                    pint(aid)
                                    emit_str(", i64 0, i64 0")
                                    putchar(10)
                                    ek = 3
                                    ev = cc
                                    cc = cc + 1
                                    handled = 1
                                }
                            }
                        }
                        if handled == 0 {
                            let e = gen_expr(toks, slots, fns, defs, src, q, astop, cc)
                            cc = e.next
                            ek = e.kind
                            ev = e.val
                        }
                        if nca == 0 { a0k = ek; a0v = ev }
                        else if nca == 1 { a1k = ek; a1v = ev }
                        else if nca == 2 { a2k = ek; a2v = ev }
                        else if nca == 3 { a3k = ek; a3v = ev }
                        else if nca == 4 { a4k = ek; a4v = ev }
                        else if nca == 5 { a5k = ek; a5v = ev }
                        else if nca == 6 { a6k = ek; a6v = ev }
                        else if nca == 7 { a7k = ek; a7v = ev }
                        else if nca == 8 { a8k = ek; a8v = ev }
                        else { a9k = ek; a9v = ev }
                        nca = nca + 1
                        q = astop + 1
                    }
                }
                # ys buffer base pointer
                let ybase = cc
                emit_str("  %t")
                pint(ybase)
                emit_str(" = getelementptr [")
                pint(cbuf)
                emit_str(" x i64], [")
                pint(cbuf)
                emit_str(" x i64]* %v")
                pint(slot)
                emit_str(", i64 0, i64 0")
                putchar(10)
                counter = ybase + 1
                # emit the void call: build(args..., i64* ybase)
                emit_str("  call void @")
                emit_name(src, cname.nstart, cname.nlen)
                emit_str("(")
                let mut ai = 0
                while ai < nca {
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
                    else if ai == 8 { ak = a8k; av = a8v }
                    else if ai == 9 { ak = a9k; av = a9v }
                    if ak == 2 { emit_str("i8* ") } else if ak == 3 { emit_str("i64* ") } else { emit_str("i64 ") }
                    emit_op(Op { kind: ak, val: av, next: 0 })
                    ai = ai + 1
                }
                if nca > 0 { emit_str(", ") }
                emit_str("i64* %t")
                pint(ybase)
                emit_str(")")
                putchar(10)
                # sync ys.len (%v<slot+1>) from the out-param's length slot
                emit_str("  %t")
                pint(counter)
                emit_str(" = getelementptr [")
                pint(cbuf)
                emit_str(" x i64], [")
                pint(cbuf)
                emit_str(" x i64]* %v")
                pint(slot)
                emit_str(", i64 0, i64 ")
                pint(lenidx)
                putchar(10)
                let lgp = counter
                counter = counter + 1
                emit_str("  %t")
                pint(counter)
                emit_str(" = load i64, i64* %t")
                pint(lgp)
                putchar(10)
                emit_str("  store i64 %t")
                pint(counter)
                emit_str(", i64* %v")
                pint(slot + 1)
                putchar(10)
                counter = counter + 1
                let stop = find_semi(toks, cclose, end)
                i = stop + 1
            } else if rhs_is_list(toks, src, npos) == 1 {
                # let lst = list() / [];  — alloca/len already emitted in collect; skip
                let stop = find_semi(toks, vp, end)
                i = stop + 1
            } else if rhs.kind == 1 and find_semi(toks, vp, end) == vp + 1 and isarr_of(slots, src, rhs.nstart, rhs.nlen) == 4 and isarr_of(slots, src, name.nstart, name.nlen) == 4 {
                # `let alias = list_param`: copy the incoming buffer pointer into the
                # alias slot. Mutating either name updates the same List buffer.
                let rslot = find_slot(slots, src, rhs.nstart, rhs.nlen)
                emit_str("  %t")
                pint(counter)
                emit_str(" = load i64*, i64** %v")
                pint(rslot)
                putchar(10)
                emit_str("  store i64* %t")
                pint(counter)
                emit_str(", i64** %v")
                pint(slot)
                putchar(10)
                counter = counter + 1
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
                let plenidx = list_lenidx_for_nfields(pnf)
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
                emit_str(", i64 ")
                pint(list_lenidx())
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
                    let lbuf = list_bufsz_for_nfields(nf)
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
                  emit_str(" = getelementptr [")
                  pint(list_cap())
                  emit_str(" x i64], [")
                  pint(list_cap())
                  emit_str(" x i64]* %v")
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
            # `return <listvar>` in a `-> List` function (retout >= 0): copy the
            # list local's buffer into the hidden out-param %a<retout>, then ret 0.
            let rnext = toks[i + 1]
            let mut did_structret = 0
            if retkind == 2 {
                if rnext.kind == 1 {
                    let rafter = toks[i + 2]
                    if stop == i + 2 {
                        # return <struct-local>
                        let rarr2 = isarr_of(slots, src, rnext.nstart, rnext.nlen)
                        let rsty2 = sty_of(slots, src, rnext.nstart, rnext.nlen)
                        if rarr2 == 0 and rsty2 >= 0 {
                            did_structret = 1
                            let rslot2 = find_slot(slots, src, rnext.nstart, rnext.nlen)
                            let rnf2 = struct_nfields(defs, rsty2)
                            let mut fk2 = 0
                            while fk2 < rnf2 {
                                emit_str("  %t")
                                pint(counter)
                                emit_str(" = getelementptr [")
                                pint(rnf2)
                                emit_str(" x i64], [")
                                pint(rnf2)
                                emit_str(" x i64]* %v")
                                pint(rslot2)
                                emit_str(", i64 0, i64 ")
                                pint(fk2)
                                putchar(10)
                                let sp2 = counter
                                counter = counter + 1
                                emit_str("  %t")
                                pint(counter)
                                emit_str(" = load i64, i64* %t")
                                pint(sp2)
                                putchar(10)
                                let sv2 = counter
                                counter = counter + 1
                                emit_str("  %t")
                                pint(counter)
                                emit_str(" = getelementptr i64, i64* %a")
                                pint(retout)
                                emit_str(", i64 ")
                                pint(fk2)
                                putchar(10)
                                let dp2 = counter
                                counter = counter + 1
                                emit_str("  store i64 %t")
                                pint(sv2)
                                emit_str(", i64* %t")
                                pint(dp2)
                                putchar(10)
                                fk2 = fk2 + 1
                            }
                            emit_str("  ret void")
                            putchar(10)
                        }
                    } else if rafter.kind == 9 {
                        # return other_struct_returning_call(...)
                        let crs = call_retsty(toks, fns, src, i + 1)
                        if crs >= 0 {
                            did_structret = 1
                            let cclose2 = paren_end(toks, i + 3)
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = getelementptr i64, i64* %a")
                            pint(retout)
                            emit_str(", i64 0")
                            putchar(10)
                            let routp = counter
                            counter = counter + 1
                            counter = emit_struct_out_call(toks, slots, fns, defs, src, i + 1, i + 3, cclose2, counter, routp)
                            emit_str("  ret void")
                            putchar(10)
                        }
                    } else if rafter.kind == 11 {
                        # return StructName { field: value, ... }
                        let rst2 = struct_index_by_name(defs, src, rnext.nstart, rnext.nlen)
                        if rst2 >= 0 {
                            did_structret = 1
                            let rnf3 = struct_nfields(defs, rst2)
                            let bopen2 = i + 2
                            let bclose2 = match_brace(toks, bopen2, end)
                            let mut fq2 = bopen2 + 1
                            while fq2 < bclose2 {
                                let fld2 = toks[fq2]
                                let fi2 = field_index(defs, rst2, src, fld2.nstart, fld2.nlen)
                                let mut vstart2 = fq2 + 1
                                let colt2 = toks[fq2 + 1]
                                if colt2.kind == 16 { vstart2 = fq2 + 2 }
                                let vstop2 = arr_elem_end(toks, vstart2, bclose2)
                                let fe2 = gen_expr(toks, slots, fns, defs, src, vstart2, vstop2, counter)
                                counter = fe2.next
                                emit_str("  %t")
                                pint(counter)
                                emit_str(" = getelementptr i64, i64* %a")
                                pint(retout)
                                emit_str(", i64 ")
                                pint(fi2)
                                putchar(10)
                                emit_str("  store i64 ")
                                emit_op(fe2)
                                emit_str(", i64* %t")
                                pint(counter)
                                putchar(10)
                                counter = counter + 1
                                fq2 = vstop2 + 1
                            }
                            emit_str("  ret void")
                            putchar(10)
                        }
                    }
                }
            }
            let mut did_listret = 0
            if retkind == 1 and retout >= 0 {
                if rnext.kind == 1 {
                    if stop == i + 2 {
                        let rk = isarr_of(slots, src, rnext.nstart, rnext.nlen)
                        if rk == 2 {
                            did_listret = 1
                            let rslot = find_slot(slots, src, rnext.nstart, rnext.nlen)
                            let rsty = sty_of(slots, src, rnext.nstart, rnext.nlen)
                            let mut stride = 1
                            let mut lenidx = list_lenidx()
                            let mut bufsz = list_cap()
                            if rsty >= 0 {
                                stride = struct_nfields(defs, rsty)
                                lenidx = list_lenidx_for_nfields(stride)
                                bufsz = list_bufsz_for_nfields(stride)
                            }
                            # load the local length (%v<rslot+1>) and store to out[lenidx]
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = load i64, i64* %v")
                            pint(rslot + 1)
                            putchar(10)
                            let rlen = counter
                            counter = counter + 1
                            # n = len * stride (number of data slots to copy)
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = mul i64 %t")
                            pint(rlen)
                            emit_str(", ")
                            pint(stride)
                            putchar(10)
                            let ncopy = counter
                            counter = counter + 1
                            # copy loop: k = 0; while k < ncopy { out[k] = src[k]; k++ }
                            emit_str("  %v")
                            pint(rslot + 1)
                            emit_str("c = alloca i64")
                            putchar(10)
                            emit_str("  store i64 0, i64* %v")
                            pint(rslot + 1)
                            emit_str("c")
                            putchar(10)
                            let lbl = counter
                            counter = counter + 1
                            emit_str("  br label %rcL")
                            pint(lbl)
                            putchar(10)
                            emit_str("rcL")
                            pint(lbl)
                            emit_str(":")
                            putchar(10)
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = load i64, i64* %v")
                            pint(rslot + 1)
                            emit_str("c")
                            putchar(10)
                            let kv = counter
                            counter = counter + 1
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = icmp slt i64 %t")
                            pint(kv)
                            emit_str(", %t")
                            pint(ncopy)
                            putchar(10)
                            let kc = counter
                            counter = counter + 1
                            emit_str("  br i1 %t")
                            pint(kc)
                            emit_str(", label %rcB")
                            pint(lbl)
                            emit_str(", label %rcD")
                            pint(lbl)
                            putchar(10)
                            emit_str("rcB")
                            pint(lbl)
                            emit_str(":")
                            putchar(10)
                            # src elem ptr = [bufsz x i64]* %v<rslot> [0, kv]
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = getelementptr [")
                            pint(bufsz)
                            emit_str(" x i64], [")
                            pint(bufsz)
                            emit_str(" x i64]* %v")
                            pint(rslot)
                            emit_str(", i64 0, i64 %t")
                            pint(kv)
                            putchar(10)
                            let sp = counter
                            counter = counter + 1
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = load i64, i64* %t")
                            pint(sp)
                            putchar(10)
                            let sv = counter
                            counter = counter + 1
                            # dst elem ptr = i64* %a<retout> + kv
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = getelementptr i64, i64* %a")
                            pint(retout)
                            emit_str(", i64 %t")
                            pint(kv)
                            putchar(10)
                            let dp = counter
                            counter = counter + 1
                            emit_str("  store i64 %t")
                            pint(sv)
                            emit_str(", i64* %t")
                            pint(dp)
                            putchar(10)
                            # k = k + 1
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = add i64 %t")
                            pint(kv)
                            emit_str(", 1")
                            putchar(10)
                            let kn = counter
                            counter = counter + 1
                            emit_str("  store i64 %t")
                            pint(kn)
                            emit_str(", i64* %v")
                            pint(rslot + 1)
                            emit_str("c")
                            putchar(10)
                            emit_str("  br label %rcL")
                            pint(lbl)
                            putchar(10)
                            emit_str("rcD")
                            pint(lbl)
                            emit_str(":")
                            putchar(10)
                            # store length into out[lenidx]
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = getelementptr i64, i64* %a")
                            pint(retout)
                            emit_str(", i64 ")
                            pint(lenidx)
                            putchar(10)
                            let lp = counter
                            counter = counter + 1
                            emit_str("  store i64 %t")
                            pint(rlen)
                            emit_str(", i64* %t")
                            pint(lp)
                            putchar(10)
                            emit_str("  ret void")
                            putchar(10)
                        } else if rk == 4 {
                            did_listret = 1
                            let rslotp = find_slot(slots, src, rnext.nstart, rnext.nlen)
                            let rstyp = sty_of(slots, src, rnext.nstart, rnext.nlen)
                            let mut stridep = 1
                            let mut lenidxp = list_lenidx()
                            if rstyp >= 0 {
                                stridep = struct_nfields(defs, rstyp)
                                lenidxp = list_lenidx_for_nfields(stridep)
                            }
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = load i64*, i64** %v")
                            pint(rslotp)
                            putchar(10)
                            let rbp = counter
                            counter = counter + 1
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = getelementptr i64, i64* %t")
                            pint(rbp)
                            emit_str(", i64 ")
                            pint(lenidxp)
                            putchar(10)
                            let rlp = counter
                            counter = counter + 1
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = load i64, i64* %t")
                            pint(rlp)
                            putchar(10)
                            let rlenp = counter
                            counter = counter + 1
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = mul i64 %t")
                            pint(rlenp)
                            emit_str(", ")
                            pint(stridep)
                            putchar(10)
                            let ncopyp = counter
                            counter = counter + 1
                            emit_str("  %rpa")
                            pint(rslotp)
                            emit_str(" = alloca i64")
                            putchar(10)
                            emit_str("  store i64 0, i64* %rpa")
                            pint(rslotp)
                            putchar(10)
                            let lblp = counter
                            counter = counter + 1
                            emit_str("  br label %rpL")
                            pint(lblp)
                            putchar(10)
                            emit_str("rpL")
                            pint(lblp)
                            emit_str(":")
                            putchar(10)
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = load i64, i64* %rpa")
                            pint(rslotp)
                            putchar(10)
                            let kvp = counter
                            counter = counter + 1
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = icmp slt i64 %t")
                            pint(kvp)
                            emit_str(", %t")
                            pint(ncopyp)
                            putchar(10)
                            let kcp = counter
                            counter = counter + 1
                            emit_str("  br i1 %t")
                            pint(kcp)
                            emit_str(", label %rpB")
                            pint(lblp)
                            emit_str(", label %rpD")
                            pint(lblp)
                            putchar(10)
                            emit_str("rpB")
                            pint(lblp)
                            emit_str(":")
                            putchar(10)
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = getelementptr i64, i64* %t")
                            pint(rbp)
                            emit_str(", i64 %t")
                            pint(kvp)
                            putchar(10)
                            let spp = counter
                            counter = counter + 1
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = load i64, i64* %t")
                            pint(spp)
                            putchar(10)
                            let svp = counter
                            counter = counter + 1
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = getelementptr i64, i64* %a")
                            pint(retout)
                            emit_str(", i64 %t")
                            pint(kvp)
                            putchar(10)
                            let dpp = counter
                            counter = counter + 1
                            emit_str("  store i64 %t")
                            pint(svp)
                            emit_str(", i64* %t")
                            pint(dpp)
                            putchar(10)
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = add i64 %t")
                            pint(kvp)
                            emit_str(", 1")
                            putchar(10)
                            let knp = counter
                            counter = counter + 1
                            emit_str("  store i64 %t")
                            pint(knp)
                            emit_str(", i64* %rpa")
                            pint(rslotp)
                            putchar(10)
                            emit_str("  br label %rpL")
                            pint(lblp)
                            putchar(10)
                            emit_str("rpD")
                            pint(lblp)
                            emit_str(":")
                            putchar(10)
                            emit_str("  %t")
                            pint(counter)
                            emit_str(" = getelementptr i64, i64* %a")
                            pint(retout)
                            emit_str(", i64 ")
                            pint(lenidxp)
                            putchar(10)
                            let ldp = counter
                            counter = counter + 1
                            emit_str("  store i64 %t")
                            pint(rlenp)
                            emit_str(", i64* %t")
                            pint(ldp)
                            putchar(10)
                            emit_str("  ret void")
                            putchar(10)
                        }
                    }
                }
            }
            if did_listret == 0 and did_structret == 0 {
                let e = gen_expr(toks, slots, fns, defs, src, i + 1, stop, counter)
                if retkind >= 1 {
                    emit_str("  ret void")
                } else {
                    emit_str("  ret i64 ")
                    emit_op(e)
                }
                putchar(10)
                counter = e.next
            }
            i = stop + 1
        } else if t.kind == 34 {
            # for <var> in <lo> .. <hi> { <body> }   (.. exclusive, ..= inclusive)
            # layout: for(34) var in(35) <lo> ..(36)/..=(37) <hi> {(11) body }(12)
            # desugar to an induction-variable while-loop on the var's slot.
            let fvar = toks[i + 1]
            let fslot = find_slot(slots, src, fvar.nstart, fvar.nlen)
            # find the open-brace after the header
            let mut bopen = i + 1
            let mut g1 = true
            while g1 {
                if bopen >= end { g1 = false }
                else {
                    let bt = toks[bopen]
                    if bt.kind == 11 { g1 = false } else { bopen = bopen + 1 }
                }
            }
            # find the range operator (36 or 37) within the header
            let mut rop = i + 3
            let mut g2 = true
            while g2 {
                if rop >= bopen { g2 = false }
                else {
                    let rt = toks[rop]
                    if rt.kind == 36 { g2 = false } else { if rt.kind == 37 { g2 = false } else { rop = rop + 1 } }
                }
            }
            let inclusive = toks[rop].kind
            let bclose = match_brace(toks, bopen, end)
            # lo-expr [i+3, rop): store into the induction var slot
            let lo = gen_expr(toks, slots, fns, defs, src, i + 3, rop, counter)
            counter = lo.next
            emit_str("  store i64 ")
            emit_op(lo)
            emit_str(", i64* %v")
            pint(fslot)
            putchar(10)
            let lbl = counter
            counter = counter + 1
            emit_str("  br label %loop")
            pint(lbl)
            putchar(10)
            emit_str("loop")
            pint(lbl)
            emit_str(":")
            putchar(10)
            # reload induction var, compare against hi-expr [rop+1, bopen)
            let ivnum = counter
            counter = counter + 1
            emit_str("  %t")
            pint(ivnum)
            emit_str(" = load i64, i64* %v")
            pint(fslot)
            putchar(10)
            let hi = gen_expr(toks, slots, fns, defs, src, rop + 1, bopen, counter)
            counter = hi.next
            let cnum = counter
            counter = counter + 1
            emit_str("  %t")
            pint(cnum)
            if inclusive == 37 {
                emit_str(" = icmp sle i64 %t")
            } else {
                emit_str(" = icmp slt i64 %t")
            }
            pint(ivnum)
            emit_str(", ")
            emit_op(hi)
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
            counter = gen_stmts(toks, slots, fns, defs, src, bopen + 1, bclose, counter, retout, retkind)
            # increment: v = v + 1
            let incld = counter
            counter = counter + 1
            emit_str("  %t")
            pint(incld)
            emit_str(" = load i64, i64* %v")
            pint(fslot)
            putchar(10)
            let incsum = counter
            counter = counter + 1
            emit_str("  %t")
            pint(incsum)
            emit_str(" = add i64 %t")
            pint(incld)
            emit_str(", 1")
            putchar(10)
            emit_str("  store i64 %t")
            pint(incsum)
            emit_str(", i64* %v")
            pint(fslot)
            putchar(10)
            emit_str("  br label %loop")
            pint(lbl)
            putchar(10)
            emit_str("done")
            pint(lbl)
            emit_str(":")
            putchar(10)
            i = bclose + 1
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
            counter = gen_stmts(toks, slots, fns, defs, src, bopen + 1, bclose, cnum + 1, retout, retkind)
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
            counter = gen_stmts(toks, slots, fns, defs, src, bopen + 1, bclose, cnum + 1, retout, retkind)
            emit_str("  br label %imerge")
            pint(lbl)
            putchar(10)
            # else block (empty if no else)
            emit_str("ielse")
            pint(lbl)
            emit_str(":")
            putchar(10)
            if has_else == 1 {
                counter = gen_stmts(toks, slots, fns, defs, src, ebody_start, ebody_end, counter, retout, retkind)
            }
            emit_str("  br label %imerge")
            pint(lbl)
            putchar(10)
            emit_str("imerge")
            pint(lbl)
            emit_str(":")
            putchar(10)
            # If both branches return (control can't reach the merge), the merge
            # block is dead -- terminate it with `unreachable` so it's valid even
            # when no statement follows (LLVM requires a terminator per block).
            if has_else == 1 {
                let then_ret = block_returns(toks, bopen + 1, bclose)
                let else_ret = block_returns(toks, ebody_start, ebody_end)
                if then_ret == 1 {
                    if else_ret == 1 {
                        emit_str("  unreachable")
                        putchar(10)
                    }
                }
            }
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
fn collect_top_slots(toks: &List<Token>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, n: Int) -> List<Slot> {
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
                let mut lcap = list_cap()
                let mut lbuf = list_cap()
                if lest >= 0 {
                    let lnf = struct_nfields(defs, lest)
                    lcap = list_struct_cap(lnf)
                    lbuf = list_bufsz_for_nfields(lnf)
                }
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 2, alen: lcap , sty: lest })
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
        } else if t.kind == 34 {
            # top-level `for <var> in ...`: reserve a scalar i64 slot for the var.
            let fv = toks[i + 1]
            slots.push(Slot { nstart: fv.nstart, nlen: fv.nlen, slot: next_slot, is_arr: 0, alen: 0 , sty: 0 - 1 })
            emit_str("  %v")
            pint(next_slot)
            emit_str(" = alloca i64")
            putchar(10)
            next_slot = next_slot + 1
            i = i + 1
        } else {
            i = i + 1
        }
    }
    return slots
}

# Generate top-level statements (skip fn defs), threading the symbol+fn tables.
fn has_top_stmts(toks: &List<Token>, n: Int) -> Int {
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 13 {
            i = skip_fn_def(toks, i, n)
        } else if t.kind == 26 {
            i = skip_struct_def(toks, i, n)
        } else if t.kind == 6 {
            i = i + 1
        } else {
            return 1
        }
    }
    return 0
}

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
            counter = gen_stmts(toks, slots, fns, defs, src, i, j, counter, 0 - 1, 0)
            i = j
        }
    }
    return counter
}

# Emit one user function: `define i64 @name(i64 %p_in) { <body> }`. The param is
# copied to alloca %v0; body locals occupy slots 1+. Body = [bstart, bend).
fn emit_fn(toks: &List<Token>, fns: &List<Fn>, defs: &List<StructDef>, src: Str, f: Fn, n: Int) -> Int {
    if f.retlist >= 1 { emit_str("define void @") } else { emit_str("define i64 @") }
    emit_name(src, f.nstart, f.nlen)
    emit_str("(")
    # incoming SSA params: %a0, %a1, ...  (Str params are i8*, others i64)
    let mut pi = 0
    while pi < f.npar {
        if pi > 0 { emit_str(", ") }
        let mut pty = f.p0ty
        if pi == 1 { pty = f.p1ty } else if pi == 2 { pty = f.p2ty } else if pi == 3 { pty = f.p3ty }
        else if pi == 4 { pty = f.p4ty } else if pi == 5 { pty = f.p5ty } else if pi == 6 { pty = f.p6ty } else if pi == 7 { pty = f.p7ty }
        else if pi == 8 { pty = f.p8ty } else if pi == 9 { pty = f.p9ty }
        # Str -> i8*, List/Struct -> i64* (buffer pointer), else i64.
        if pty == 1 { emit_str("i8* %a") } else if pty == 2 { emit_str("i64* %a") } else if pty >= 3 { emit_str("i64* %a") } else { emit_str("i64 %a") }
        pint(pi)
        pi = pi + 1
    }
    # `-> List` and `-> Struct` functions take a hidden trailing out-param
    # `i64* %a<npar>`: the caller passes a buffer, and return copies into it.
    if f.retlist >= 1 {
        if f.npar > 0 { emit_str(", ") }
        emit_str("i64* %a")
        pint(f.npar)
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
        else if s2 == 8 { pns = f.p8s; pnl = f.p8l; pty = f.p8ty }
        else if s2 == 9 { pns = f.p9s; pnl = f.p9l; pty = f.p9ty }
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
        } else if pty >= 3 {
            # Struct param by value: incoming i64* points at nf fields. Copy into a
            # local [nf x i64] so existing field read/write code can use %v<slot>.
            let pst = pty - 3
            let nf = struct_nfields(defs, pst)
            slots.push(Slot { nstart: pns, nlen: pnl, slot: s2, is_arr: 0, alen: 0 , sty: pst })
            emit_str("  %v")
            pint(s2)
            emit_str(" = alloca [")
            pint(nf)
            emit_str(" x i64]")
            putchar(10)
            let mut fk = 0
            while fk < nf {
                emit_str("  %sp")
                pint(s2)
                emit_str("p")
                pint(fk)
                emit_str(" = getelementptr i64, i64* %a")
                pint(s2)
                emit_str(", i64 ")
                pint(fk)
                putchar(10)
                emit_str("  %sp")
                pint(s2)
                emit_str("v")
                pint(fk)
                emit_str(" = load i64, i64* %sp")
                pint(s2)
                emit_str("p")
                pint(fk)
                putchar(10)
                emit_str("  %sp")
                pint(s2)
                emit_str("d")
                pint(fk)
                emit_str(" = getelementptr [")
                pint(nf)
                emit_str(" x i64], [")
                pint(nf)
                emit_str(" x i64]* %v")
                pint(s2)
                emit_str(", i64 0, i64 ")
                pint(fk)
                putchar(10)
                emit_str("  store i64 %sp")
                pint(s2)
                emit_str("v")
                pint(fk)
                emit_str(", i64* %sp")
                pint(s2)
                emit_str("d")
                pint(fk)
                putchar(10)
                fk = fk + 1
            }
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
    let allslots = add_local_slots(slots, toks, fns, defs, src, f.bstart, f.bend, f.npar, n)
    # retout = the hidden out-param index for a `-> List` function, else -1.
    let mut retout = 0 - 1
    if f.retlist >= 1 { retout = f.npar }
    let last = gen_stmts(toks, &allslots, fns, defs, src, f.bstart, f.bend, 1, retout, f.retlist)
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
    # declare puts + printf so generated programs can `print(...)` (the transpiler
    # rewrites print->puts; brace-bearing literals route to printf interpolation).
    emit_str("declare i32 @puts(i8*)")
    putchar(10)
    emit_str("declare i32 @printf(i8*, ...)")
    putchar(10)
    let defs = build_defs(&toks, n)
    let fns = build_fns(&toks, &defs, src, n)
    # module-level string-literal globals (one per literal, keyed by source pos).
    # Function metadata is available here so interpolation formats can choose %s
    # for Str params while still emitting globals before any function definition.
    emit_str_globals(&toks, &fns, src, n)
    # emit each user function
    let m = fns.len()
    let mut fi = 0
    while fi < m {
        let f = fns[fi]
        emit_fn(&toks, &fns, &defs, src, f, n)
        fi = fi + 1
    }
    # emit synthetic @main only when the source has top-level executable
    # statements. Real source files often define their own fn main().
    if has_top_stmts(&toks, n) == 1 {
        emit_str("define i64 @main() {")
        putchar(10)
        let topslots = collect_top_slots(&toks, &fns, &defs, src, n)
        let last = gen_top(&toks, &fns, &defs, &topslots, src, n)
        emit_str("}")
        putchar(10)
    }
    return 0
}

fn main() -> Int {
    # Keep the embedded default free of backtick string literals. The string/List
    # tokenizer shape is covered by regression tests; this default must survive
    # source-file self embedding, where nested backticks would truncate the sample.
    return compile("fn tok() {{ return 42 }}; return tok();")
}
