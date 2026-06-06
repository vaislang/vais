# expect: 0
# nl self-host — fixpoint REAL CODE GENERATOR.
#
# Unlike fixpoint.nl/fixpoint2/3 (which EVALUATE to a value and emit a constant
# `ret i64 <value>`), this compiler emits LLVM IR that COMPUTES the result at
# runtime — real `mul`/`add`/`sub` instructions with SSA temporaries. This is the
# codegen half of a compiler (the generated program does the arithmetic, not the
# compiler).
#
# Pipeline: source -> tokenize -> List<Token> -> recursive code generation that
# `print`s instructions in evaluation order, threading an SSA temp counter, and
# returns each subexpression's operand form (literal value or `%tN` temp).
#
# Requires the Vais fixes: `&Vec` borrow recursion (214c97cf) for the token list,
# and literal-`%` escaping (e711dac1) so `%tN` register names survive printing.
#
# Grammar: arithmetic `+ - *` over multi-digit integers, `*` tighter, left-assoc.

# Token kinds: 0=number(value), 2='+', 3='*', 4='-'.
struct Token { kind: Int, value: Int }
# An operand: kind 0 = literal (use `val` directly), 1 = temp (use `%t<val>`).
# `next` is the next free SSA temp number after generating this operand.
struct Op { kind: Int, val: Int, next: Int }

fn is_digit(c: Int) -> Bool { return c >= 48 and c <= 57 }
fn is_space(c: Int) -> Bool {
    if c == 32 { return true }
    if c == 9 { return true }
    return false
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
            toks.push(Token { kind: 0, value: v })
        } else if c == 43 { toks.push(Token { kind: 2, value: 0 }); i = i + 1 }
        else if c == 42 { toks.push(Token { kind: 3, value: 0 }); i = i + 1 }
        else if c == 45 { toks.push(Token { kind: 4, value: 0 }); i = i + 1 }
        else { i = i + 1 }
    }
    return toks
}

# Emit `%t<counter> = <op_s> i64 <lhs>, <rhs>` for operands that are each either
# a literal or a temp (4 cases). Returns the dest temp number (= counter).
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

# A factor is a number literal -> an Op of kind 0 (no instruction emitted).
fn gen_factor(toks: &List<Token>, i: Int, counter: Int) -> Op {
    let t = toks[i]
    return Op { kind: 0, val: t.value, next: counter }
}

# Generate a term `factor (* factor)*`, folding left into mul instructions.
# `acc` carries the running operand (kind/val); returns the final Op.
fn gen_term(toks: &List<Token>, i: Int, n: Int, ak: Int, av: Int, counter: Int) -> Op {
    if i >= n { return Op { kind: ak, val: av, next: counter } }
    let t = toks[i]
    if t.kind == 3 {
        let rf = gen_factor(toks, i + 1, counter)
        let dest = emit_binop("mul", ak, av, rf.kind, rf.val, rf.next)
        return gen_term(toks, i + 2, n, 1, dest, dest + 1)
    }
    return Op { kind: ak, val: av, next: counter }
}
fn skip_term(toks: &List<Token>, i: Int, n: Int) -> Int {
    if i >= n { return n }
    let t = toks[i]
    if t.kind == 3 { return skip_term(toks, i + 2, n) }
    return i
}

# Generate an expression `term ((+|-) term)*`, folding left.
fn gen_expr(toks: &List<Token>, i: Int, n: Int) -> Op {
    let f0 = gen_factor(toks, i, 1)
    let t0 = gen_term(toks, i + 1, n, f0.kind, f0.val, f0.next)
    let after = skip_term(toks, i + 1, n)
    return gen_fold(toks, after, n, t0.kind, t0.val, t0.next)
}
fn gen_fold(toks: &List<Token>, i: Int, n: Int, ak: Int, av: Int, counter: Int) -> Op {
    if i >= n { return Op { kind: ak, val: av, next: counter } }
    let op = toks[i]
    if op.kind == 2 {
        let rf = gen_factor(toks, i + 1, counter)
        let rt = gen_term(toks, i + 2, n, rf.kind, rf.val, rf.next)
        let after = skip_term(toks, i + 2, n)
        let dest = emit_binop("add", ak, av, rt.kind, rt.val, rt.next)
        return gen_fold(toks, after, n, 1, dest, dest + 1)
    }
    if op.kind == 4 {
        let rf = gen_factor(toks, i + 1, counter)
        let rt = gen_term(toks, i + 2, n, rf.kind, rf.val, rf.next)
        let after = skip_term(toks, i + 2, n)
        let dest = emit_binop("sub", ak, av, rt.kind, rt.val, rt.next)
        return gen_fold(toks, after, n, 1, dest, dest + 1)
    }
    return Op { kind: ak, val: av, next: counter }
}

# Compile a source string to a full LLVM IR module that computes it at runtime.
fn compile(src: Str) -> Int {
    let toks = tokenize(src)
    let n = toks.len()
    print("define i64 @main() {")
    let result = gen_expr(&toks, 0, n)
    # emit the return using the final operand (literal or temp)
    if result.kind == 0 {
        print("  ret i64 {result.val}")
    } else {
        print("  ret i64 %t{result.val}")
    }
    print("}")
    return 0
}

fn main() -> Int {
    # Real codegen: "12 + 3 * 4" -> IR with mul + add -> runtime 24.
    return compile("12 + 3 * 4")
}
