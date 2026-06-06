# expect: 0
# nl self-host — fixpoint code generator v3: FUNCTIONS (define/call), real codegen.
#
# Emits real LLVM IR with multiple functions: each `fn <name>(<param>) { return
# <expr> }` becomes `define i64 @<name>(i64 %<param>) { ... }`, and a call
# `<name>(<arg>)` becomes a `call i64 @<name>(...)` instruction. The generated
# program computes at runtime (not interpreted). Multi-char names are emitted as
# LLVM identifiers by copying their source bytes (emit_name).
#
# Requires the Vais fixes: `&Vec` borrow recursion (214c97cf) + literal-`%`
# escaping (e711dac1).
#
# Grammar: `fn <name>(<param>) { return <expr> }` (single param) + a top-level
# `return <expr>` where <expr> is arithmetic over integers, the param, and calls.

# Token kinds: 0=num,1=ident,2='+',3='*',4='-',6=';',8=return,9='(',10=')',
#              11='{',12='}',13=fn
struct Token { kind: Int, value: Int, nstart: Int, nlen: Int }
# Operand: kind 0=literal(val), 1=temp(%t<val>), 2=named(%<src[ns..ns+nl]>).
struct Op { kind: Int, val: Int, ns: Int, nl: Int, next: Int }
# Function: name range, param range, body token range [bstart,bend).
struct Fn { nstart: Int, nlen: Int, ps: Int, pl: Int, bstart: Int, bend: Int }

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
fn kw4(src: Str, a: Int, alen: Int, w0: Int, w1: Int, w2: Int, w3: Int) -> Int {
    if alen != 4 { return 0 }
    if src[a] != w0 { return 0 }
    if src[a + 1] != w1 { return 0 }
    if src[a + 2] != w2 { return 0 }
    if src[a + 3] != w3 { return 0 }
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
            if kw6(src, start, len, 114, 101, 116, 117, 114, 110) == 1 {
                toks.push(Token { kind: 8, value: 0, nstart: start, nlen: len })
            } else if kw2(src, start, len, 102, 110) == 1 {
                toks.push(Token { kind: 13, value: 0, nstart: start, nlen: len })
            } else if kw2(src, start, len, 105, 102) == 1 {
                toks.push(Token { kind: 15, value: 0, nstart: start, nlen: len })   # if
            } else if kw4(src, start, len, 116, 104, 101, 110) == 1 {
                toks.push(Token { kind: 16, value: 0, nstart: start, nlen: len })   # then
            } else if kw4(src, start, len, 101, 108, 115, 101) == 1 {
                toks.push(Token { kind: 17, value: 0, nstart: start, nlen: len })   # else
            } else {
                toks.push(Token { kind: 1, value: 0, nstart: start, nlen: len })
            }
        } else if c == 43 { toks.push(Token { kind: 2, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 42 { toks.push(Token { kind: 3, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 45 { toks.push(Token { kind: 4, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 60 { toks.push(Token { kind: 18, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 62 { toks.push(Token { kind: 19, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 61 {
            if i + 1 < n {
                if src[i + 1] == 61 {
                    toks.push(Token { kind: 20, value: 0, nstart: 0, nlen: 0 }); i = i + 2
                } else { i = i + 1 }
            } else { i = i + 1 }
        }
        else if c == 59 { toks.push(Token { kind: 6, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 40 { toks.push(Token { kind: 9, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 41 { toks.push(Token { kind: 10, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 123 { toks.push(Token { kind: 11, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 125 { toks.push(Token { kind: 12, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
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
# Print the source name src[start..start+len] verbatim (for @name / %name).
fn emit_name(src: Str, start: Int, len: Int) -> Int {
    let mut k = 0
    while k < len {
        putchar(src[start + k])
        k = k + 1
    }
    return 0
}

# --- build the function table ---
fn build_fns(toks: &List<Token>, n: Int) -> List<Fn> {
    let mut fns: List<Fn> = []
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 13 {
            let nt = toks[i + 1]      # name
            # i+2 '(', i+3 param, i+4 ')', i+5 '{', body from i+6
            let p = toks[i + 3]
            let bstart = i + 6
            let mut be = bstart
            let mut go = true
            while go {
                if be >= n {
                    go = false
                } else {
                    let bt = toks[be]
                    if bt.kind == 12 { go = false } else { be = be + 1 }
                }
            }
            fns.push(Fn { nstart: nt.nstart, nlen: nt.nlen, ps: p.nstart, pl: p.nlen, bstart: bstart, bend: be })
            i = be + 1
        } else {
            i = i + 1
        }
    }
    return fns
}
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

# Print an operand inline (used as a binop/call argument or ret value).
fn emit_op(o: Op, src: Str) -> Int {
    if o.kind == 0 { print_int_inline(o.val) }
    else if o.kind == 1 { putchar(37); putchar(116); print_int_inline(o.val) }  # %t<n>
    else { putchar(37); emit_name(src, o.ns, o.nl) }                            # %<name>
    return 0
}
# Print an integer with no newline (helper, since print adds newline).
fn print_int_inline(x: Int) -> Int {
    if x < 0 { putchar(45); return print_int_inline(0 - x) }
    if x >= 10 { print_int_inline(x / 10) }
    putchar(48 + (x - (x / 10) * 10))
    return 0
}

# Emit `  %t<dest> = <op_s> i64 <lhs>, <rhs>\n` and return dest.
fn emit_binop(op_s: Str, l: Op, r: Op, src: Str, counter: Int) -> Int {
    putchar(32); putchar(32); putchar(37); putchar(116); print_int_inline(counter)
    putchar(32); putchar(61); putchar(32)   # " = "
    emit_str(op_s)
    putchar(32); putchar(105); putchar(54); putchar(52); putchar(32)  # " i64 "
    emit_op(l, src)
    putchar(44); putchar(32)   # ", "
    emit_op(r, src)
    putchar(10)
    return counter
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

# --- expression codegen (arithmetic + param refs + calls) ---
fn gen_factor(toks: &List<Token>, fns: &List<Fn>, src: Str, i: Int, ps: Int, pl: Int, counter: Int) -> Op {
    let t = toks[i]
    if t.kind == 0 { return Op { kind: 0, val: t.value, ns: 0, nl: 0, next: counter } }
    if t.kind == 1 {
        # call if next is '('
        let nx = toks[i + 1]
        if nx.kind == 9 {
            # call <name>(<argexpr>)
            let arg = gen_expr(toks, fns, src, i + 2, ps, pl, counter)
            # emit: %t<dest> = call i64 @<name>(i64 <arg>)
            let dest = arg.next
            putchar(32); putchar(32); putchar(37); putchar(116); print_int_inline(dest)
            putchar(32); putchar(61); putchar(32)
            emit_str("call i64 @")
            emit_name(src, t.nstart, t.nlen)
            putchar(40); putchar(105); putchar(54); putchar(52); putchar(32)  # "(i64 "
            emit_op(arg, src)
            putchar(41); putchar(10)   # ")\n"
            return Op { kind: 1, val: dest, ns: 0, nl: 0, next: dest + 1 }
        }
        # else: param reference -> named operand
        return Op { kind: 2, val: 0, ns: t.nstart, nl: t.nlen, next: counter }
    }
    return Op { kind: 0, val: 0, ns: 0, nl: 0, next: counter }
}
# skip one factor (number/param = 1 token; call = name ( ... ))
fn skip_factor(toks: &List<Token>, i: Int) -> Int {
    let t = toks[i]
    if t.kind == 1 {
        let nx = toks[i + 1]
        if nx.kind == 9 {
            let e = paren_end(toks, i + 2)
            return e + 1
        }
    }
    return i + 1
}
fn paren_end(toks: &List<Token>, i: Int) -> Int {
    let mut j = i
    let mut depth = 0
    let mut go = true
    while go {
        let t = toks[j]
        if t.kind == 9 { depth = depth + 1; j = j + 1 }
        else if t.kind == 10 {
            if depth == 0 { go = false } else { depth = depth - 1; j = j + 1 }
        } else { j = j + 1 }
    }
    return j
}

fn gen_term(toks: &List<Token>, fns: &List<Fn>, src: Str, i: Int, stop: Int, ps: Int, pl: Int, acc: Op) -> Op {
    if i >= stop { return acc }
    let t = toks[i]
    if t.kind == 3 {
        let rf = gen_factor(toks, fns, src, i + 1, ps, pl, acc.next)
        let dest = emit_binop("mul", acc, rf, src, rf.next)
        let nacc = Op { kind: 1, val: dest, ns: 0, nl: 0, next: dest + 1 }
        let after = skip_factor(toks, i + 1)
        return gen_term(toks, fns, src, after, stop, ps, pl, nacc)
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

fn gen_expr(toks: &List<Token>, fns: &List<Fn>, src: Str, i: Int, ps: Int, pl: Int, counter: Int) -> Op {
    # stop = the matching ')' or ';' — but for simplicity callers pass ranges that
    # end at a delimiter; we scan to the next ')' or ';' at depth 0.
    let stop = expr_end(toks, i)
    let f0 = gen_factor(toks, fns, src, i, ps, pl, counter)
    let af = skip_factor(toks, i)
    let t0 = gen_term(toks, fns, src, af, stop, ps, pl, f0)
    let after = skip_term(toks, af, stop)
    return gen_fold(toks, fns, src, after, stop, ps, pl, t0)
}
# Find the end of an expression: next ')' or ';' at paren-depth 0.
fn expr_end(toks: &List<Token>, i: Int) -> Int {
    let mut j = i
    let mut depth = 0
    let mut go = true
    while go {
        let t = toks[j]
        if t.kind == 9 { depth = depth + 1; j = j + 1 }
        else if t.kind == 10 {
            if depth == 0 { go = false } else { depth = depth - 1; j = j + 1 }
        }
        else if t.kind == 6 { go = false }
        else if t.kind == 12 { go = false }
        else { j = j + 1 }
    }
    return j
}
fn gen_fold(toks: &List<Token>, fns: &List<Fn>, src: Str, i: Int, stop: Int, ps: Int, pl: Int, acc: Op) -> Op {
    if i >= stop { return acc }
    let op = toks[i]
    if op.kind == 2 {
        let rf = gen_factor(toks, fns, src, i + 1, ps, pl, acc.next)
        let rt = gen_term(toks, fns, src, skip_factor(toks, i + 1), stop, ps, pl, rf)
        let dest = emit_binop("add", acc, rt, src, rt.next)
        let nacc = Op { kind: 1, val: dest, ns: 0, nl: 0, next: dest + 1 }
        return gen_fold(toks, fns, src, skip_term(toks, skip_factor(toks, i + 1), stop), stop, ps, pl, nacc)
    }
    if op.kind == 4 {
        let rf = gen_factor(toks, fns, src, i + 1, ps, pl, acc.next)
        let rt = gen_term(toks, fns, src, skip_factor(toks, i + 1), stop, ps, pl, rf)
        let dest = emit_binop("sub", acc, rt, src, rt.next)
        let nacc = Op { kind: 1, val: dest, ns: 0, nl: 0, next: dest + 1 }
        return gen_fold(toks, fns, src, skip_term(toks, skip_factor(toks, i + 1), stop), stop, ps, pl, nacc)
    }
    return acc
}

# Find the index of the first token of `kind` in [i, stop), else stop.
fn find_kind(toks: &List<Token>, i: Int, stop: Int, kind: Int) -> Int {
    if i >= stop { return stop }
    let t = toks[i]
    if t.kind == kind { return i }
    return find_kind(toks, i + 1, stop, kind)
}

# Codegen a body expression that may be `if <cond> then <e> else <e>`. Emits the
# icmp/br/phi control-flow structure for the conditional (so a recursive call in
# a branch only runs on that path), or delegates to gen_expr for a plain expr.
# `counter` is the next SSA temp; the returned Op holds the body value, and
# Op.next is the next free temp/label number. Labels reuse temp numbers.
fn gen_body(toks: &List<Token>, fns: &List<Fn>, src: Str, i: Int, stop: Int, ps: Int, pl: Int, counter: Int) -> Op {
    let t = toks[i]
    if t.kind == 15 {
        # if <cond-lhs> <cmp> <cond-rhs> then <thenE> else <elseE>
        let then_pos = find_kind(toks, i + 1, stop, 16)
        let else_pos = find_kind(toks, then_pos + 1, stop, 17)
        # comparison operator in [i+1, then_pos)
        let cmp_lt = find_kind(toks, i + 1, then_pos, 18)
        let cmp_gt = find_kind(toks, i + 1, then_pos, 19)
        let cmp_eq = find_kind(toks, i + 1, then_pos, 20)
        let mut oppos = then_pos
        let mut pred_lt = 0
        let mut pred_gt = 0
        let mut pred_eq = 0
        if cmp_lt < then_pos { oppos = cmp_lt; pred_lt = 1 }
        else if cmp_gt < then_pos { oppos = cmp_gt; pred_gt = 1 }
        else if cmp_eq < then_pos { oppos = cmp_eq; pred_eq = 1 }
        # codegen the two condition operands
        let lhs = gen_expr(toks, fns, src, i + 1, ps, pl, counter)
        let rhs = gen_expr(toks, fns, src, oppos + 1, ps, pl, lhs.next)
        let cnum = rhs.next
        # emit `  %t<cnum> = icmp <pred> i64 <lhs>, <rhs>`
        emit_str("  %t")
        print_int_inline(cnum)
        emit_str(" = icmp ")
        if pred_lt == 1 { emit_str("slt") } else if pred_gt == 1 { emit_str("sgt") } else { emit_str("eq") }
        emit_str(" i64 ")
        emit_op(lhs, src)
        emit_str(", ")
        emit_op(rhs, src)
        putchar(10)
        # labels use numbers based on cnum
        let lbl = cnum
        emit_str("  br i1 %t")
        print_int_inline(cnum)
        emit_str(", label %then")
        print_int_inline(lbl)
        emit_str(", label %else")
        print_int_inline(lbl)
        putchar(10)
        # then block
        emit_str("then")
        print_int_inline(lbl)
        emit_str(":")
        putchar(10)
        let thenE = gen_body(toks, fns, src, then_pos + 1, else_pos, ps, pl, cnum + 1)
        emit_str("  br label %merge")
        print_int_inline(lbl)
        putchar(10)
        # else block
        emit_str("else")
        print_int_inline(lbl)
        emit_str(":")
        putchar(10)
        let elseE = gen_body(toks, fns, src, else_pos + 1, stop, ps, pl, thenE.next)
        emit_str("  br label %merge")
        print_int_inline(lbl)
        putchar(10)
        # merge + phi
        emit_str("merge")
        print_int_inline(lbl)
        emit_str(":")
        putchar(10)
        let rnum = elseE.next
        emit_str("  %t")
        print_int_inline(rnum)
        emit_str(" = phi i64 [ ")
        emit_op(thenE, src)
        emit_str(", %then")
        print_int_inline(lbl)
        emit_str(" ], [ ")
        emit_op(elseE, src)
        emit_str(", %else")
        print_int_inline(lbl)
        emit_str(" ]")
        putchar(10)
        return Op { kind: 1, val: rnum, ns: 0, nl: 0, next: rnum + 1 }
    }
    return gen_expr(toks, fns, src, i, ps, pl, counter)
}

# Emit a function `define i64 @name(i64 %param) { <body> ret <expr> }`.
fn emit_fn(toks: &List<Token>, fns: &List<Fn>, src: Str, f: Fn) -> Int {
    emit_str("define i64 @")
    emit_name(src, f.nstart, f.nlen)
    emit_str("(i64 %")
    emit_name(src, f.ps, f.pl)
    emit_str(") {")
    putchar(10)
    # body is `return <expr>` from bstart; expr starts at bstart+1
    let e = gen_body(toks, fns, src, f.bstart + 1, f.bend, f.ps, f.pl, 1)
    emit_str("  ret i64 ")
    emit_op(e, src)
    putchar(10)
    emit_str("}")
    putchar(10)
    return 0
}

fn find_semi(toks: &List<Token>, i: Int, n: Int) -> Int {
    if i >= n { return n }
    let t = toks[i]
    if t.kind == 6 { return i }
    return find_semi(toks, i + 1, n)
}

fn compile(src: Str) -> Int {
    let toks = tokenize(src)
    let n = toks.len()
    let fns = build_fns(&toks, n)
    # emit each function definition
    let m = fns.len()
    let mut fi = 0
    while fi < m {
        let f = fns[fi]
        emit_fn(&toks, &fns, src, f)
        fi = fi + 1
    }
    # emit main with the top-level return
    emit_str("define i64 @main() {")
    putchar(10)
    let mut i = 0
    let mut done = 0
    while i < n {
        let t = toks[i]
        if t.kind == 13 {
            # skip fn def
            let mut j = i + 1
            let mut go = true
            while go {
                if j >= n {
                    go = false
                } else {
                    let jt = toks[j]
                    if jt.kind == 12 { go = false } else { j = j + 1 }
                }
            }
            i = j + 1
        } else if t.kind == 8 {
            if done == 0 {
                let e = gen_expr(&toks, &fns, src, i + 1, 0, 0, 1)
                emit_str("  ret i64 ")
                emit_op(e, src)
                putchar(10)
                done = 1
            }
            i = i + 1
        } else {
            i = i + 1
        }
    }
    emit_str("}")
    putchar(10)
    return 0
}

fn main() -> Int {
    # FP8: a RECURSIVE function codegen'd with icmp/br/phi control flow.
    # factorial(n) = if n < 2 then 1 else n * factorial(n - 1); factorial(5)=120.
    return compile("fn factorial(n) {{ return if n < 2 then 1 else n * factorial(n - 1) }}; return factorial(5);")
}
