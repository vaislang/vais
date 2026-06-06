# expect: 0
# nl self-host compiler — L3.5 integrated mini compiler (written in nl).
#
# End-to-end: a SOURCE STRING of arithmetic ("1+2*3") -> lex bytes -> evaluate
# with precedence (* before +) -> emit LLVM IR that returns the value.
# This wires the three stages (lex / parse-eval / codegen) into one nl program.
#
# Bootstrap: this .nl -> seed transpiler -> Vais -> vaisc -> gen1. gen1 is an
# nl-written compiler that turns arithmetic source text into runnable IR.
#
# Single-pass over the source string (no token Vec passed to sub-fns — avoids the
# Vais Vec-recursion limit task_54658a43, and the && short-circuit limit
# task_492f7e17 via nested guards). A full AST/recursive parser awaits L3's own
# backend or those Vais fixes.

fn is_digit(c: Int) -> Bool {
    return c >= 48 and c <= 57
}

# Lex+evaluate an arithmetic source string directly (digits and + - * /).
# Returns the computed value. Handles * / before + - by accumulating a term.
fn compile_eval(src: Str) -> Int {
    let n = src.len()
    let mut total = 0       # running sum (committed terms)
    let mut term = 0        # current term (being multiplied)
    let mut cur = 0         # current number being read
    let mut have = false    # have we read a digit for `cur`?
    let mut pending = 43    # pending additive op for `term` -> total: 43='+', 45='-'
    let mut termop = 42     # pending multiplicative op for `cur` -> term: 42='*', 47='/'
    let mut term_started = false
    let mut i = 0
    while i < n {
        let c = src[i]
        if is_digit(c) {
            cur = cur * 10 + (c - 48)
            have = true
        } else {
            # c is an operator: fold `cur` into `term` using termop
            if term_started {
                if termop == 42 {
                    term = term * cur
                } else {
                    term = term / cur
                }
            } else {
                term = cur
                term_started = true
            }
            cur = 0
            have = false
            # if this operator is + or -, commit term into total and reset
            if c == 43 {
                if pending == 43 { total = total + term } else { total = total - term }
                pending = 43
                term_started = false
            } else if c == 45 {
                if pending == 43 { total = total + term } else { total = total - term }
                pending = 45
                term_started = false
            } else {
                # * or / : just set termop for the next number
                termop = c
            }
        }
        i = i + 1
    }
    # fold last number + last term
    if have {
        if term_started {
            if termop == 42 { term = term * cur } else { term = term / cur }
        } else {
            term = cur
            term_started = true
        }
    }
    if term_started {
        if pending == 43 { total = total + term } else { total = total - term }
    }
    return total
}

# Emit LLVM IR for a program that returns `value`.
fn emit_ir(value: Int) -> Int {
    print("define i64 @main() {")
    print("  ret i64 {value}")
    print("}")
    return 0
}

fn main() -> Int {
    # Compile the source "1+2*3" -> evaluate -> emit IR (value 7).
    let value = compile_eval("1+2*3")
    return emit_ir(value)
}
