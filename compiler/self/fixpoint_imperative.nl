# expect: 0
# nl self-host — fixpoint IMPERATIVE code generator (toward full self-compile).
#
# Generates real LLVM IR for imperative nl: mutable variables (`let mut`),
# assignment (`x = expr`), `while` loops, and `return`, using alloca/store/load
# so values can be MUTATED (the SSA-operand model of fixpoint_codegen* cannot
# express mutation/loops). This is a foundational step toward compiling the kind
# of imperative code the nl compiler itself is written in.
#
# Step FP10a (this file, first cut): `let [mut] <name> = <expr>; <name> = <expr>;
# ... return <expr>` — alloca per variable, arithmetic over ints + variables.
# (while loops: FP10b.)
#
# Requires the Vais fixes: `&Vec` borrow recursion (214c97cf) + literal-`%`
# escaping (e711dac1).

# Token kinds: 0=num,1=ident,2='+',3='*',4='-',5='=',6=';',7=let,8=return,21=mut
struct Token { kind: Int, value: Int, nstart: Int, nlen: Int }
# Operand: kind 0=literal(val), 1=temp(%t<val>). next = next free SSA temp.
struct Op { kind: Int, val: Int, next: Int }
# A declared variable: source name range -> it lives at alloca %v<slot>.
struct Slot { nstart: Int, nlen: Int, slot: Int }

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
fn kw3(src: Str, a: Int, alen: Int, w0: Int, w1: Int, w2: Int) -> Int {
    if alen != 3 { return 0 }
    if src[a] != w0 { return 0 }
    if src[a + 1] != w1 { return 0 }
    if src[a + 2] != w2 { return 0 }
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
                } else {
                    toks.push(Token { kind: 5, value: 0, nstart: 0, nlen: 0 }); i = i + 1
                }
            } else {
                toks.push(Token { kind: 5, value: 0, nstart: 0, nlen: 0 }); i = i + 1
            }
        }
        else if c == 123 { toks.push(Token { kind: 11, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 125 { toks.push(Token { kind: 12, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
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

# Print an operand inline.
fn emit_op(o: Op) -> Int {
    if o.kind == 0 { pint(o.val) }
    else { putchar(37); putchar(116); pint(o.val) }
    return 0
}

# A factor: number, or a variable (emit a load from its alloca -> a temp).
fn gen_factor(toks: &List<Token>, slots: &List<Slot>, src: Str, i: Int, counter: Int) -> Op {
    let t = toks[i]
    if t.kind == 0 { return Op { kind: 0, val: t.value, next: counter } }
    if t.kind == 1 {
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

fn gen_term(toks: &List<Token>, slots: &List<Slot>, src: Str, i: Int, stop: Int, acc: Op) -> Op {
    if i >= stop { return acc }
    let t = toks[i]
    if t.kind == 3 {
        let rf = gen_factor(toks, slots, src, i + 1, acc.next)
        let dest = emit_binop("mul", acc, rf, rf.next)
        let nacc = Op { kind: 1, val: dest, next: dest + 1 }
        return gen_term(toks, slots, src, i + 2, stop, nacc)
    }
    return acc
}
fn skip_term(toks: &List<Token>, i: Int, stop: Int) -> Int {
    if i >= stop { return stop }
    let t = toks[i]
    if t.kind == 3 { return skip_term(toks, i + 2, stop) }
    return i
}
fn gen_expr(toks: &List<Token>, slots: &List<Slot>, src: Str, i: Int, stop: Int, counter: Int) -> Op {
    let f0 = gen_factor(toks, slots, src, i, counter)
    let af = skip_factor(toks, i)
    let t0 = gen_term(toks, slots, src, af, stop, f0)
    let after = skip_term(toks, af, stop)
    return gen_fold(toks, slots, src, after, stop, t0)
}
fn skip_factor(toks: &List<Token>, i: Int) -> Int {
    return i + 1
}
fn gen_fold(toks: &List<Token>, slots: &List<Slot>, src: Str, i: Int, stop: Int, acc: Op) -> Op {
    if i >= stop { return acc }
    let op = toks[i]
    if op.kind == 2 {
        let rf = gen_factor(toks, slots, src, i + 1, acc.next)
        let rt = gen_term(toks, slots, src, skip_factor(toks, i + 1), stop, rf)
        let dest = emit_binop("add", acc, rt, rt.next)
        let nacc = Op { kind: 1, val: dest, next: dest + 1 }
        return gen_fold(toks, slots, src, skip_term(toks, skip_factor(toks, i + 1), stop), stop, nacc)
    }
    if op.kind == 4 {
        let rf = gen_factor(toks, slots, src, i + 1, acc.next)
        let rt = gen_term(toks, slots, src, skip_factor(toks, i + 1), stop, rf)
        let dest = emit_binop("sub", acc, rt, rt.next)
        let nacc = Op { kind: 1, val: dest, next: dest + 1 }
        return gen_fold(toks, slots, src, skip_term(toks, skip_factor(toks, i + 1), stop), stop, nacc)
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
fn collect_slots(toks: &List<Token>, n: Int, src: Str) -> List<Slot> {
    let mut slots: List<Slot> = []
    let mut next_slot = 0
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 7 {
            # let [mut] <name> ...  : name is at i+1, or i+2 if 'mut'
            let nx = toks[i + 1]
            let mut npos = i + 1
            if nx.kind == 21 { npos = i + 2 }
            let name = toks[npos]
            slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot })
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
# index. `op` must be the '{' token index.
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
fn gen_stmts(toks: &List<Token>, slots: &List<Slot>, src: Str, i0: Int, end: Int, counter0: Int) -> Int {
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
            let stop = find_semi(toks, npos + 2, end)
            let e = gen_expr(toks, slots, src, npos + 2, stop, counter)
            emit_str("  store i64 ")
            emit_op(e)
            emit_str(", i64* %v")
            pint(slot)
            putchar(10)
            counter = e.next
            i = stop + 1
        } else if t.kind == 1 {
            let nx = toks[i + 1]
            if nx.kind == 5 {
                let slot = find_slot(slots, src, t.nstart, t.nlen)
                let stop = find_semi(toks, i + 2, end)
                let e = gen_expr(toks, slots, src, i + 2, stop, counter)
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
            let e = gen_expr(toks, slots, src, i + 1, stop, counter)
            emit_str("  ret i64 ")
            emit_op(e)
            putchar(10)
            counter = e.next
            i = stop + 1
        } else if t.kind == 22 {
            # while <lhs> <cmp> <rhs> { <body> }
            # find '{' after the condition
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
            let lhs = gen_expr(toks, slots, src, cstart, oppos, counter)
            let rhs = gen_expr(toks, slots, src, oppos + 1, cend, lhs.next)
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
            counter = gen_stmts(toks, slots, src, bopen + 1, bclose, cnum + 1)
            emit_str("  br label %loop")
            pint(lbl)
            putchar(10)
            emit_str("done")
            pint(lbl)
            emit_str(":")
            putchar(10)
            i = bclose + 1
        } else {
            i = i + 1
        }
    }
    return counter
}

fn compile(src: Str) -> Int {
    let toks = tokenize(src)
    let n = toks.len()
    emit_str("define i64 @main() {")
    putchar(10)
    let slots = collect_slots(&toks, n, src)
    let last = gen_stmts(&toks, &slots, src, 0, n, 1)
    emit_str("}")
    putchar(10)
    return 0
}

fn main() -> Int {
    # Imperative WITH a loop: sum 1..5 = 15.
    # let mut s = 0; let mut i = 1; while i < 6 { s = s + i; i = i + 1 } return s
    return compile("let mut s = 0; let mut i = 1; while i < 6 {{ s = s + i; i = i + 1 }}; return s;")
}
