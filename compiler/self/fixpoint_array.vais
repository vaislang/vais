# expect: 0
# nl self-host — fixpoint ARRAY code generator (data structures, toward
# self-compile). Generates real LLVM IR for fixed-size integer arrays:
#   let <name> = [v0, v1, ...];   -> alloca [N x i64] + element stores
#   <name>[<expr>]                -> getelementptr + load (runtime index)
#   <name>[<expr>] = <expr>;      -> getelementptr + store
# plus scalar mutable variables, `while`, and `return` (reused machinery).
#
# Arrays are the basis for the List the real nl compiler is built on; this is a
# concrete data-structure codegen step. Requires the Vais fixes (214c97cf,
# e711dac1).

# Token kinds: 0=num,1=ident,2='+',3='*',4='-',5='=',6=';',7=let,8=return,21=mut,
#              18='<',19='>',20='==',22=while,11='{',12='}',23='[',24=']',25=','
struct Token { kind: Int, value: Int, nstart: Int, nlen: Int }
struct Op { kind: Int, val: Int, next: Int }
# A variable slot. is_arr=1 means it's an array (alloca [len x i64]); else scalar.
struct Slot { nstart: Int, nlen: Int, slot: Int, is_arr: Int, alen: Int }

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
                toks.push(Token { kind: 22, value: 0, nstart: start, nlen: len })
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
        else if c == 91 { toks.push(Token { kind: 23, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 93 { toks.push(Token { kind: 24, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 44 { toks.push(Token { kind: 25, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
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
fn emit_op(o: Op) -> Int {
    if o.kind == 0 { pint(o.val) }
    else { putchar(37); putchar(116); pint(o.val) }
    return 0
}

# Slot lookups (return the matching Slot's fields).
fn slot_of(slots: &List<Slot>, src: Str, qs: Int, ql: Int) -> Int {
    let m = slots.len()
    let mut i = 0
    while i < m {
        let s = slots[i]
        if name_eq(src, s.nstart, s.nlen, qs, ql) == 1 { return s.slot }
        i = i + 1
    }
    return 0 - 1
}
fn is_arr_of(slots: &List<Slot>, src: Str, qs: Int, ql: Int) -> Int {
    let m = slots.len()
    let mut i = 0
    while i < m {
        let s = slots[i]
        if name_eq(src, s.nstart, s.nlen, qs, ql) == 1 { return s.is_arr }
        i = i + 1
    }
    return 0
}
fn len_of(slots: &List<Slot>, src: Str, qs: Int, ql: Int) -> Int {
    let m = slots.len()
    let mut i = 0
    while i < m {
        let s = slots[i]
        if name_eq(src, s.nstart, s.nlen, qs, ql) == 1 { return s.alen }
        i = i + 1
    }
    return 0
}

# A factor: number, array index `name[expr]` (GEP+load), or scalar var (load).
fn gen_factor(toks: &List<Token>, slots: &List<Slot>, src: Str, i: Int, counter: Int) -> Op {
    let t = toks[i]
    if t.kind == 0 { return Op { kind: 0, val: t.value, next: counter } }
    if t.kind == 1 {
        let nx = toks[i + 1]
        if nx.kind == 23 {
            # array index: name [ <expr> ]
            let slot = slot_of(slots, src, t.nstart, t.nlen)
            let alen = len_of(slots, src, t.nstart, t.nlen)
            let idx = gen_expr(toks, slots, src, i + 2, counter)
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
        # scalar variable load
        let slot = slot_of(slots, src, t.nstart, t.nlen)
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

# Advance past one factor: a number/scalar = 1 token; an index `name[expr]` =
# name '[' expr ']'.
fn skip_factor(toks: &List<Token>, i: Int) -> Int {
    let t = toks[i]
    if t.kind == 1 {
        let nx = toks[i + 1]
        if nx.kind == 23 {
            return bracket_end(toks, i + 2) + 1
        }
    }
    return i + 1
}
fn bracket_end(toks: &List<Token>, i: Int) -> Int {
    let mut j = i
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

fn gen_term(toks: &List<Token>, slots: &List<Slot>, src: Str, i: Int, stop: Int, acc: Op) -> Op {
    if i >= stop { return acc }
    let t = toks[i]
    if t.kind == 3 {
        let rf = gen_factor(toks, slots, src, i + 1, acc.next)
        let dest = emit_binop("mul", acc, rf, rf.next)
        let nacc = Op { kind: 1, val: dest, next: dest + 1 }
        let after = skip_factor(toks, i + 1)
        return gen_term(toks, slots, src, after, stop, nacc)
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
fn gen_expr(toks: &List<Token>, slots: &List<Slot>, src: Str, i: Int, counter: Int) -> Op {
    let stop = expr_end(toks, i)
    let f0 = gen_factor(toks, slots, src, i, counter)
    let af = skip_factor(toks, i)
    let t0 = gen_term(toks, slots, src, af, stop, f0)
    let after = skip_term(toks, af, stop)
    return gen_fold(toks, slots, src, after, stop, t0)
}
# An expression ends at ';' '}' ']' or a comparison/'=' at the top level. We scan
# to the first such delimiter not nested in brackets.
fn expr_end(toks: &List<Token>, i: Int) -> Int {
    let mut j = i
    let mut depth = 0
    let mut go = true
    while go {
        let t = toks[j]
        if t.kind == 23 { depth = depth + 1; j = j + 1 }
        else if t.kind == 24 {
            if depth == 0 { go = false } else { depth = depth - 1; j = j + 1 }
        }
        else if depth == 0 {
            if t.kind == 6 { go = false }
            else if t.kind == 11 { go = false }
            else if t.kind == 18 { go = false }
            else if t.kind == 19 { go = false }
            else if t.kind == 20 { go = false }
            else if t.kind == 5 { go = false }
            else { j = j + 1 }
        }
        else { j = j + 1 }
    }
    return j
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

# First pass: count array length for an array-literal `let` (commas + 1), and
# emit each variable's alloca (scalar i64 or [N x i64]). Returns the slot table.
fn collect_slots(toks: &List<Token>, n: Int, src: Str) -> List<Slot> {
    let mut slots: List<Slot> = []
    let mut next_slot = 0
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 7 {
            let nx = toks[i + 1]
            let mut npos = i + 1
            if nx.kind == 21 { npos = i + 2 }
            let name = toks[npos]
            # '=' at npos+1; RHS at npos+2. Array if RHS starts with '['.
            let rhs = toks[npos + 2]
            if rhs.kind == 23 {
                # count elements: commas in [..] + 1 (assume non-empty)
                let bend = bracket_end(toks, npos + 3)
                let mut commas = 0
                let mut q = npos + 3
                while q < bend {
                    let qt = toks[q]
                    if qt.kind == 25 { commas = commas + 1 }
                    q = q + 1
                }
                let alen = commas + 1
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 1, alen: alen })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [")
                pint(alen)
                emit_str(" x i64]")
                putchar(10)
            } else {
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, is_arr: 0, alen: 0 })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca i64")
                putchar(10)
            }
            next_slot = next_slot + 1
        }
        i = i + 1
    }
    return slots
}

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

# Generate statements in [i0, end). Returns next free SSA temp.
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
            let rhs = toks[npos + 2]
            let slot = slot_of(slots, src, name.nstart, name.nlen)
            if rhs.kind == 23 {
                # array literal: store each element
                let alen = len_of(slots, src, name.nstart, name.nlen)
                let mut q = npos + 3
                let mut idx = 0
                let bend = bracket_end(toks, npos + 3)
                while q < bend {
                    let estop = elem_end(toks, q, bend)
                    let e = gen_expr(toks, slots, src, q, counter)
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
                let e = gen_expr(toks, slots, src, npos + 2, counter)
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
            if nx.kind == 23 {
                # array element assignment: name [ idx ] = expr ;
                let slot = slot_of(slots, src, t.nstart, t.nlen)
                let alen = len_of(slots, src, t.nstart, t.nlen)
                let bend = bracket_end(toks, i + 2)
                # idx is [i+2, bend); '=' at bend+1; value from bend+2
                let idx = gen_expr(toks, slots, src, i + 2, counter)
                counter = idx.next
                let stop = find_semi(toks, bend + 2, end)
                let val = gen_expr(toks, slots, src, bend + 2, counter)
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
                let slot = slot_of(slots, src, t.nstart, t.nlen)
                let stop = find_semi(toks, i + 2, end)
                let e = gen_expr(toks, slots, src, i + 2, counter)
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
            let e = gen_expr(toks, slots, src, i + 1, counter)
            emit_str("  ret i64 ")
            emit_op(e)
            putchar(10)
            counter = e.next
            i = stop + 1
        } else if t.kind == 22 {
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
            let lbl = counter
            counter = counter + 1
            emit_str("  br label %loop")
            pint(lbl)
            putchar(10)
            emit_str("loop")
            pint(lbl)
            emit_str(":")
            putchar(10)
            let lhs = gen_expr(toks, slots, src, cstart, counter)
            let rhs = gen_expr(toks, slots, src, oppos + 1, lhs.next)
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

# Find the end of an array-literal element: next ',' or the closing ']' (bend).
fn elem_end(toks: &List<Token>, i: Int, bend: Int) -> Int {
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
    # Array: let a = [10, 20, 30]; return a[1] + a[2] -> 50.
    return compile("let a = [10, 20, 30]; return a[1] + a[2];")
}
