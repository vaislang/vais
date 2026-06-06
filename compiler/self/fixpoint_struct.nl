# expect: 0
# nl self-host — fixpoint STRUCT code generator (data structures, toward
# self-compile). A struct is represented as a fixed [N x i64] (all fields i64 in
# this subset); a field name maps to its index. Generates real LLVM IR for:
#   struct Name {{ f0, f1, ... }}        -- type declaration (field -> index)
#   let p = Name {{ f0: v0, f1: v1 }};   -- alloca [N x i64] + field stores
#   p.field                              -- getelementptr field-index + load
#   p.field = expr;                      -- getelementptr field-index + store
# plus scalar vars, while, return (reused). Structs are the records the nl
# compiler uses (Token, Op, Fn, Slot). Requires Vais fixes 214c97cf + e711dac1.

# Token kinds: 0=num,1=ident,2='+',3='*',4='-',5='=',6=';',7=let,8=return,21=mut,
#              22=while,11='{',12='}',18='<',19='>',20='==',26=struct,27='.',25=','
struct Token { kind: Int, value: Int, nstart: Int, nlen: Int }
struct Op { kind: Int, val: Int, next: Int }
# A variable slot. ty = -1 scalar; >=0 = index into the struct-type table.
struct Slot { nstart: Int, nlen: Int, slot: Int, ty: Int }
# A struct type: name range + the source positions of up to 6 field names + count.
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
            } else if kw6(src, start, len, 115, 116, 114, 117, 99, 116) == 1 {
                toks.push(Token { kind: 26, value: 0, nstart: start, nlen: len })
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
        else if c == 46 { toks.push(Token { kind: 27, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
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

# --- struct type table lookups ---
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
# field index within struct-type ti for field named src[qs..qs+ql]; -1 if absent.
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

# --- slot table lookups ---
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
fn ty_of(slots: &List<Slot>, src: Str, qs: Int, ql: Int) -> Int {
    let m = slots.len()
    let mut i = 0
    while i < m {
        let s = slots[i]
        if name_eq(src, s.nstart, s.nlen, qs, ql) == 1 { return s.ty }
        i = i + 1
    }
    return 0 - 1
}

# A factor: number, field access `name.field` (GEP+load), or scalar var (load).
fn gen_factor(toks: &List<Token>, slots: &List<Slot>, defs: &List<StructDef>, src: Str, i: Int, counter: Int) -> Op {
    let t = toks[i]
    if t.kind == 0 { return Op { kind: 0, val: t.value, next: counter } }
    if t.kind == 1 {
        let nx = toks[i + 1]
        if nx.kind == 27 {
            # field access: name . field
            let fld = toks[i + 2]
            let slot = slot_of(slots, src, t.nstart, t.nlen)
            let ti = ty_of(slots, src, t.nstart, t.nlen)
            let nf = struct_nfields(defs, ti)
            let fi = field_index(defs, ti, src, fld.nstart, fld.nlen)
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

# Advance past one factor: number/scalar = 1 token; field access = name . field.
fn skip_factor(toks: &List<Token>, i: Int) -> Int {
    let t = toks[i]
    if t.kind == 1 {
        let nx = toks[i + 1]
        if nx.kind == 27 { return i + 3 }
    }
    return i + 1
}

fn gen_term(toks: &List<Token>, slots: &List<Slot>, defs: &List<StructDef>, src: Str, i: Int, stop: Int, acc: Op) -> Op {
    if i >= stop { return acc }
    let t = toks[i]
    if t.kind == 3 {
        let rf = gen_factor(toks, slots, defs, src, i + 1, acc.next)
        let dest = emit_binop("mul", acc, rf, rf.next)
        let nacc = Op { kind: 1, val: dest, next: dest + 1 }
        let after = skip_factor(toks, i + 1)
        return gen_term(toks, slots, defs, src, after, stop, nacc)
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
fn gen_expr(toks: &List<Token>, slots: &List<Slot>, defs: &List<StructDef>, src: Str, i: Int, stop: Int, counter: Int) -> Op {
    let f0 = gen_factor(toks, slots, defs, src, i, counter)
    let af = skip_factor(toks, i)
    let t0 = gen_term(toks, slots, defs, src, af, stop, f0)
    let after = skip_term(toks, af, stop)
    return gen_fold(toks, slots, defs, src, after, stop, t0)
}
fn gen_fold(toks: &List<Token>, slots: &List<Slot>, defs: &List<StructDef>, src: Str, i: Int, stop: Int, acc: Op) -> Op {
    if i >= stop { return acc }
    let op = toks[i]
    if op.kind == 2 {
        let rf = gen_factor(toks, slots, defs, src, i + 1, acc.next)
        let rt = gen_term(toks, slots, defs, src, skip_factor(toks, i + 1), stop, rf)
        let dest = emit_binop("add", acc, rt, rt.next)
        let nacc = Op { kind: 1, val: dest, next: dest + 1 }
        return gen_fold(toks, slots, defs, src, skip_term(toks, skip_factor(toks, i + 1), stop), stop, nacc)
    }
    if op.kind == 4 {
        let rf = gen_factor(toks, slots, defs, src, i + 1, acc.next)
        let rt = gen_term(toks, slots, defs, src, skip_factor(toks, i + 1), stop, rf)
        let dest = emit_binop("sub", acc, rt, rt.next)
        let nacc = Op { kind: 1, val: dest, next: dest + 1 }
        return gen_fold(toks, slots, defs, src, skip_term(toks, skip_factor(toks, i + 1), stop), stop, nacc)
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

# Build the struct-type table from `struct Name {{ f0, f1, ... }}` declarations.
fn build_defs(toks: &List<Token>, n: Int) -> List<StructDef> {
    let mut defs: List<StructDef> = []
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 26 {
            let nt = toks[i + 1]
            # i+2 '{', fields are idents separated by ',' until '}'
            let bopen = i + 2
            let bclose = match_brace(toks, bopen, n)
            # collect field name positions (up to 6)
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

# Collect variable slots (emitting allocas). A `let p = Name {{...}}` is a struct
# var (alloca [N x i64], ty=type-index); else scalar (alloca i64, ty=-1).
fn collect_slots(toks: &List<Token>, defs: &List<StructDef>, src: Str, n: Int) -> List<Slot> {
    let mut slots: List<Slot> = []
    let mut next_slot = 0
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 26 {
            # skip a struct declaration
            let bopen = i + 2
            let bclose = match_brace(toks, bopen, n)
            i = bclose + 1
        } else if t.kind == 7 {
            let nx = toks[i + 1]
            let mut npos = i + 1
            if nx.kind == 21 { npos = i + 2 }
            let name = toks[npos]
            # RHS at npos+2. Struct if it's an ident naming a known struct type
            # followed by '{'.
            let rhs = toks[npos + 2]
            let mut ti = 0 - 1
            if rhs.kind == 1 {
                let after = toks[npos + 3]
                if after.kind == 11 {
                    ti = struct_index_by_name(defs, src, rhs.nstart, rhs.nlen)
                }
            }
            if ti >= 0 {
                let nf = struct_nfields(defs, ti)
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, ty: ti })
                emit_str("  %v")
                pint(next_slot)
                emit_str(" = alloca [")
                pint(nf)
                emit_str(" x i64]")
                putchar(10)
            } else {
                slots.push(Slot { nstart: name.nstart, nlen: name.nlen, slot: next_slot, ty: 0 - 1 })
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

# Find the end of a struct-literal field value: next ',' or '}' at the field level.
fn fieldval_end(toks: &List<Token>, i: Int, bclose: Int) -> Int {
    let mut j = i
    let mut go = true
    while go {
        if j >= bclose { go = false }
        else {
            let t = toks[j]
            if t.kind == 25 { go = false } else { j = j + 1 }
        }
    }
    return j
}

fn gen_stmts(toks: &List<Token>, slots: &List<Slot>, defs: &List<StructDef>, src: Str, i0: Int, end: Int, counter0: Int) -> Int {
    let mut i = i0
    let mut counter = counter0
    while i < end {
        let t = toks[i]
        if t.kind == 7 {
            let nx = toks[i + 1]
            let mut npos = i + 1
            if nx.kind == 21 { npos = i + 2 }
            let name = toks[npos]
            let slot = slot_of(slots, src, name.nstart, name.nlen)
            let ti = ty_of(slots, src, name.nstart, name.nlen)
            if ti >= 0 {
                # struct literal: Name {{ f: v, f: v }} -> store each field
                let nf = struct_nfields(defs, ti)
                # '{' is at npos+3; fields from npos+4
                let bopen = npos + 3
                let bclose = match_brace(toks, bopen, end)
                let mut q = bopen + 1
                while q < bclose {
                    let fld = toks[q]
                    # fld . skip ':' (kind?) — we tokenize ':' as nothing; use '='? No.
                    # Field syntax is `name : value`. ':' isn't a token here; we
                    # rely on the value starting after the field name + ':' which
                    # the tokenizer dropped. So value starts at q+1 (no ':' token).
                    let fi = field_index(defs, ti, src, fld.nstart, fld.nlen)
                    let vstart = q + 1
                    let vstop = fieldval_end(toks, vstart, bclose)
                    let e = gen_expr(toks, slots, defs, src, vstart, vstop, counter)
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
            } else {
                let stop = find_semi(toks, npos + 2, end)
                let e = gen_expr(toks, slots, defs, src, npos + 2, stop, counter)
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
                # field assignment: name . field = expr ;
                let fld = toks[i + 2]
                let slot = slot_of(slots, src, t.nstart, t.nlen)
                let ti = ty_of(slots, src, t.nstart, t.nlen)
                let nf = struct_nfields(defs, ti)
                let fi = field_index(defs, ti, src, fld.nstart, fld.nlen)
                # '=' at i+3; value from i+4
                let stop = find_semi(toks, i + 4, end)
                let e = gen_expr(toks, slots, defs, src, i + 4, stop, counter)
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
            } else if nx.kind == 5 {
                let slot = slot_of(slots, src, t.nstart, t.nlen)
                let stop = find_semi(toks, i + 2, end)
                let e = gen_expr(toks, slots, defs, src, i + 2, stop, counter)
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
            let e = gen_expr(toks, slots, defs, src, i + 1, stop, counter)
            emit_str("  ret i64 ")
            emit_op(e)
            putchar(10)
            counter = e.next
            i = stop + 1
        } else {
            i = i + 1
        }
    }
    return counter
}

fn compile(src: Str) -> Int {
    let toks = tokenize(src)
    let n = toks.len()
    let defs = build_defs(&toks, n)
    emit_str("define i64 @main() {")
    putchar(10)
    let slots = collect_slots(&toks, defs, src, n)
    let last = gen_stmts(&toks, &slots, &defs, src, 0, n, 1)
    emit_str("}")
    putchar(10)
    return 0
}

fn main() -> Int {
    # Struct: declare Point, build one, read fields. p.x + p.y = 7.
    return compile("struct Point {{ x, y }}; let p = Point {{ x: 3, y: 4 }}; return p.x + p.y;")
}
