# expect: 0
# nl self-host — fixpoint dynamic LIST code generator (the last data structure
# toward self-compile). A List is modeled as a fixed-capacity backing buffer
# (alloca [CAP x i64]) plus a length counter (alloca i64) — so push/len/index
# work without true heap growth (capacity 64 here). Generates real LLVM IR for:
#   let lst = list();         -- buffer alloca + length alloca = 0
#   lst.push(expr);           -- store at buf[len]; len = len + 1
#   lst.len                   -- load the length counter
#   lst[expr]                 -- getelementptr buf + load
# plus scalar vars, while, return (reused). List<T> is THE structure the nl
# compiler is built on (List<Token>, List<Fn>, ...). Vais fixes 214c97cf+e711dac1.

# Token kinds: 0=num,1=ident,2='+',3='*',4='-',5='=',6=';',7=let,8=return,21=mut,
#  22=while,11='{',12='}',18='<',19='>',20='==',23='[',24=']',9='(',10=')',27='.'
struct Token { kind: Int, value: Int, nstart: Int, nlen: Int }
struct Op { kind: Int, val: Int, next: Int }
# Variable slot. kind 0 = scalar (alloca i64 at %v<slot>); kind 1 = List
# (buffer at %v<slot>, length at %v<slot+1>).
struct Slot { nstart: Int, nlen: Int, slot: Int, kind: Int }

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
        else if c == 40 { toks.push(Token { kind: 9, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 41 { toks.push(Token { kind: 10, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
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
fn emit_op(o: Op) -> Int {
    if o.kind == 0 { pint(o.val) }
    else { putchar(37); putchar(116); pint(o.val) }
    return 0
}

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
fn kind_of(slots: &List<Slot>, src: Str, qs: Int, ql: Int) -> Int {
    let m = slots.len()
    let mut i = 0
    while i < m {
        let s = slots[i]
        if name_eq(src, s.nstart, s.nlen, qs, ql) == 1 { return s.kind }
        i = i + 1
    }
    return 0
}

# A factor: number, list len `lst.len`, list index `lst[expr]`, or scalar var.
fn gen_factor(toks: &List<Token>, slots: &List<Slot>, src: Str, i: Int, counter: Int) -> Op {
    let t = toks[i]
    if t.kind == 0 { return Op { kind: 0, val: t.value, next: counter } }
    if t.kind == 1 {
        let nx = toks[i + 1]
        if nx.kind == 27 {
            # lst.len  -> load length counter at %v<slot+1>
            let slot = slot_of(slots, src, t.nstart, t.nlen)
            emit_str("  %t")
            pint(counter)
            emit_str(" = load i64, i64* %v")
            pint(slot + 1)
            putchar(10)
            return Op { kind: 1, val: counter, next: counter + 1 }
        }
        if nx.kind == 23 {
            # lst[expr] -> GEP buf + load
            let slot = slot_of(slots, src, t.nstart, t.nlen)
            let bend = bracket_end(toks, i + 2)
            let idx = gen_expr(toks, slots, src, i + 2, bend, counter)
            let gepc = idx.next
            emit_str("  %t")
            pint(gepc)
            emit_str(" = getelementptr [64 x i64], [64 x i64]* %v")
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
        # scalar var
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
# Advance past one factor: number/scalar = 1 token; lst.len = 3 tokens; lst[expr]
# = name '[' expr ']'.
fn skip_factor(toks: &List<Token>, i: Int) -> Int {
    let t = toks[i]
    if t.kind == 1 {
        let nx = toks[i + 1]
        if nx.kind == 27 { return i + 3 }
        if nx.kind == 23 { return bracket_end(toks, i + 2) + 1 }
    }
    return i + 1
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
fn gen_expr(toks: &List<Token>, slots: &List<Slot>, src: Str, i: Int, stop: Int, counter: Int) -> Op {
    let f0 = gen_factor(toks, slots, src, i, counter)
    let af = skip_factor(toks, i)
    let t0 = gen_term(toks, slots, src, af, stop, f0)
    let after = skip_term(toks, af, stop)
    return gen_fold(toks, slots, src, after, stop, t0)
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

# Collect slots. A `let lst = list()` is a List (2 slots: buffer + length); else
# scalar (1 slot). Emits allocas.
fn collect_slots(toks: &List<Token>, src: Str, n: Int) -> List<Slot> {
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
            # RHS at npos+2. List if it's `list ( )`: ident 'list' then '('.
            let rhs = toks[npos + 2]
            let mut is_list = 0
            if rhs.kind == 1 {
                if kw4(src, rhs.nstart, rhs.nlen, 108, 105, 115, 116) == 1 {
                    let after = toks[npos + 3]
                    if after.kind == 9 { is_list = 1 }
                }
            }
            if is_list == 1 {
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, kind: 1 })
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
            } else {
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, kind: 0 })
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
            let k = kind_of(slots, src, name.nstart, name.nlen)
            if k == 1 {
                # let lst = list();  (alloca + len=0 already emitted in collect)
                let stop = find_semi(toks, npos + 2, end)
                i = stop + 1
            } else {
                let slot = slot_of(slots, src, name.nstart, name.nlen)
                let stop = find_semi(toks, npos + 2, end)
                let e = gen_expr(toks, slots, src, npos + 2, stop, counter)
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
            if nx.kind == 27 {
                # lst.push(expr) ;  (method call)
                let meth = toks[i + 2]
                # method name 'push' followed by '(' at i+3
                let slot = slot_of(slots, src, t.nstart, t.nlen)
                # arg expr from i+4 to matching ')'
                let mut ap = i + 4
                let argstop = paren_end(toks, i + 4)
                let e = gen_expr(toks, slots, src, i + 4, argstop, counter)
                counter = e.next
                # load len
                emit_str("  %t")
                pint(counter)
                emit_str(" = load i64, i64* %v")
                pint(slot + 1)
                putchar(10)
                let lenc = counter
                counter = counter + 1
                # GEP buf[len]
                emit_str("  %t")
                pint(counter)
                emit_str(" = getelementptr [64 x i64], [64 x i64]* %v")
                pint(slot)
                emit_str(", i64 0, i64 %t")
                pint(lenc)
                putchar(10)
                let gepc = counter
                counter = counter + 1
                # store value
                emit_str("  store i64 ")
                emit_op(e)
                emit_str(", i64* %t")
                pint(gepc)
                putchar(10)
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
            } else if nx.kind == 23 {
                # lst[idx] = expr ;
                let slot = slot_of(slots, src, t.nstart, t.nlen)
                let bend = bracket_end(toks, i + 2)
                let idx = gen_expr(toks, slots, src, i + 2, bend, counter)
                counter = idx.next
                let stop = find_semi(toks, bend + 2, end)
                let val = gen_expr(toks, slots, src, bend + 2, stop, counter)
                counter = val.next
                emit_str("  %t")
                pint(counter)
                emit_str(" = getelementptr [64 x i64], [64 x i64]* %v")
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

fn compile(src: Str) -> Int {
    let toks = tokenize(src)
    let n = toks.len()
    emit_str("define i64 @main() {")
    putchar(10)
    let slots = collect_slots(&toks, src, n)
    let last = gen_stmts(&toks, &slots, src, 0, n, 1)
    emit_str("}")
    putchar(10)
    return 0
}

fn main() -> Int {
    # Dynamic List: push 3 values, read len + an element.
    # let xs = list(); xs.push(10); xs.push(20); xs.push(30); return xs.len + xs[1]
    #   = 3 + 20 = 23.
    return compile("let xs = list(); xs.push(10); xs.push(20); xs.push(30); return xs.len + xs[1];")
}
