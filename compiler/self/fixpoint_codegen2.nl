# expect: 0
# nl self-host — fixpoint code generator v2: VARIABLES, real codegen.
#
# Combines fixpoint_codegen.nl (emits runtime LLVM IR for arithmetic) with
# multi-char variables (fixpoint2.nl). A program is
#   let <name> = <expr>; ... return <expr>
# and the compiler emits IR that COMPUTES it at runtime. Variables use an SSA
# model: each `let` binds a name to the OPERAND (literal value or `%tN` temp)
# that its expression produced — immutable bindings need no alloca. A variable
# reference resolves to that operand.
#
# Requires the Vais fixes: `&Vec` borrow recursion (214c97cf) and literal-`%`
# escaping (e711dac1).
#
# Grammar: `let`/`return` over arithmetic (+ - *, * tighter, left-assoc) with
# multi-digit integers and multi-char variable names.

# Token kinds: 0=num,1=ident,2='+',3='*',4='-',5='=',6=';',7=let,8=return
struct Token { kind: Int, value: Int, nstart: Int, nlen: Int }
# An operand: kind 0=literal(val), 1=temp(%t<val>). `next` = next free SSA temp.
struct Op { kind: Int, val: Int, next: Int }
# A bound variable: source name range -> its operand (kind/val).
struct SymOp { nstart: Int, nlen: Int, kind: Int, val: Int }

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
            } else {
                toks.push(Token { kind: 1, value: 0, nstart: start, nlen: len })
            }
        } else if c == 43 { toks.push(Token { kind: 2, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 42 { toks.push(Token { kind: 3, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 45 { toks.push(Token { kind: 4, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
        else if c == 61 { toks.push(Token { kind: 5, value: 0, nstart: 0, nlen: 0 }); i = i + 1 }
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

# Emit a binop and return the dest temp number (= counter).
fn emit_binop(op_s: Str, lk: Int, lv: Int, rk: Int, rv: Int, counter: Int) -> Int {
    if lk == 0 {
        if rk == 0 { print("  %t{counter} = {op_s} i64 {lv}, {rv}") }
        else { print("  %t{counter} = {op_s} i64 {lv}, %t{rv}") }
    } else {
        if rk == 0 { print("  %t{counter} = {op_s} i64 %t{lv}, {rv}") }
        else { print("  %t{counter} = {op_s} i64 %t{lv}, %t{rv}") }
    }
    return counter
}

# A factor: number literal -> Op kind 0; identifier -> resolve to its bound Op.
fn gen_factor(toks: &List<Token>, vars: &List<SymOp>, src: Str, i: Int, counter: Int) -> Op {
    let t = toks[i]
    if t.kind == 0 { return Op { kind: 0, val: t.value, next: counter } }
    if t.kind == 1 {
        # variable reference: look up its operand
        let m = vars.len()
        let mut j = 0
        while j < m {
            let v = vars[j]
            if name_eq(src, v.nstart, v.nlen, t.nstart, t.nlen) == 1 {
                return Op { kind: v.kind, val: v.val, next: counter }
            }
            j = j + 1
        }
        return Op { kind: 0, val: 0, next: counter }
    }
    return Op { kind: 0, val: 0, next: counter }
}

fn gen_term(toks: &List<Token>, vars: &List<SymOp>, src: Str, i: Int, stop: Int, ak: Int, av: Int, counter: Int) -> Op {
    if i >= stop { return Op { kind: ak, val: av, next: counter } }
    let t = toks[i]
    if t.kind == 3 {
        let rf = gen_factor(toks, vars, src, i + 1, counter)
        let dest = emit_binop("mul", ak, av, rf.kind, rf.val, rf.next)
        return gen_term(toks, vars, src, i + 2, stop, 1, dest, dest + 1)
    }
    return Op { kind: ak, val: av, next: counter }
}
fn skip_term(toks: &List<Token>, i: Int, stop: Int) -> Int {
    if i >= stop { return stop }
    let t = toks[i]
    if t.kind == 3 { return skip_term(toks, i + 2, stop) }
    return i
}

fn gen_expr(toks: &List<Token>, vars: &List<SymOp>, src: Str, i: Int, stop: Int, counter: Int) -> Op {
    let f0 = gen_factor(toks, vars, src, i, counter)
    let t0 = gen_term(toks, vars, src, i + 1, stop, f0.kind, f0.val, f0.next)
    let after = skip_term(toks, i + 1, stop)
    return gen_fold(toks, vars, src, after, stop, t0.kind, t0.val, t0.next)
}
fn gen_fold(toks: &List<Token>, vars: &List<SymOp>, src: Str, i: Int, stop: Int, ak: Int, av: Int, counter: Int) -> Op {
    if i >= stop { return Op { kind: ak, val: av, next: counter } }
    let op = toks[i]
    if op.kind == 2 {
        let rf = gen_factor(toks, vars, src, i + 1, counter)
        let rt = gen_term(toks, vars, src, i + 2, stop, rf.kind, rf.val, rf.next)
        let after = skip_term(toks, i + 2, stop)
        let dest = emit_binop("add", ak, av, rt.kind, rt.val, rt.next)
        return gen_fold(toks, vars, src, after, stop, 1, dest, dest + 1)
    }
    if op.kind == 4 {
        let rf = gen_factor(toks, vars, src, i + 1, counter)
        let rt = gen_term(toks, vars, src, i + 2, stop, rf.kind, rf.val, rf.next)
        let after = skip_term(toks, i + 2, stop)
        let dest = emit_binop("sub", ak, av, rt.kind, rt.val, rt.next)
        return gen_fold(toks, vars, src, after, stop, 1, dest, dest + 1)
    }
    return Op { kind: ak, val: av, next: counter }
}

fn find_semi(toks: &List<Token>, i: Int, n: Int) -> Int {
    if i >= n { return n }
    let t = toks[i]
    if t.kind == 6 { return i }
    return find_semi(toks, i + 1, n)
}

# Compile: emit the module header, generate each statement's IR, return.
fn compile(src: Str) -> Int {
    let toks = tokenize(src)
    let n = toks.len()
    let mut vars: List<SymOp> = []
    let mut counter = 1
    let mut ret_kind = 0
    let mut ret_val = 0
    print("define i64 @main() {")
    let mut i = 0
    while i < n {
        let t = toks[i]
        if t.kind == 7 {
            # let <name> = <expr> ;
            let name = toks[i + 1]
            let stop = find_semi(toks, i + 3, n)
            let e = gen_expr(&toks, &vars, src, i + 3, stop, counter)
            vars.push(SymOp { nstart: name.nstart, nlen: name.nlen, kind: e.kind, val: e.val })
            counter = e.next
            i = stop + 1
        } else if t.kind == 8 {
            let stop = find_semi(toks, i + 1, n)
            let e = gen_expr(&toks, &vars, src, i + 1, stop, counter)
            ret_kind = e.kind
            ret_val = e.val
            counter = e.next
            i = stop + 1
        } else {
            i = i + 1
        }
    }
    if ret_kind == 0 { print("  ret i64 {ret_val}") }
    else { print("  ret i64 %t{ret_val}") }
    print("}")
    return 0
}

fn main() -> Int {
    # Real codegen with variables: let x = 5; let y = x * 2; return y + 1 -> 11.
    return compile("let x = 5; let y = x * 2; return y + 1;")
}
