# expect: 0
# nl self-host compiler — CX1: arithmetic + variables (written in nl).
#
# Input "language": a program string of statements separated by ';':
#   let <var> = <arith> ;   (bind a single-letter variable a..z)
#   return <arith>          (final value)
# <arith> is + - * / over numbers AND variables (single letters a..z).
# Pipeline: source string -> evaluate (with a 26-slot symbol table) -> emit IR.
#
# Bootstrap: this .nl -> seed transpiler -> Vais -> vaisc -> gen1.
# Single-pass / index based (avoids Vais Vec-recursion limit task_54658a43 and
# && short-circuit limit task_492f7e17 via nested guards). Variable storage uses
# a fixed 26-int table keyed by (letter - 'a') to avoid string-map Vais limits.

fn is_digit(c: Int) -> Bool {
    return c >= 48 and c <= 57
}
fn is_lower(c: Int) -> Bool {
    return c >= 97 and c <= 122
}
fn is_space(c: Int) -> Bool {
    if c == 32 { return true }
    if c == 9 { return true }
    if c == 10 { return true }
    return false
}

# Evaluate one arithmetic expression from src[start..end) using the symbol
# table `vars` (26 ints, index = letter-'a'). Returns the value.
# Handles * / before + - (term accumulation), left-assoc, multi-digit, vars.
fn eval_arith(src: Str, start: Int, end: Int, vars: List<Int>) -> Int {
    let mut total = 0
    let mut term = 0
    let mut term_started = false
    let mut pending = 43      # '+' or '-' applying term -> total
    let mut termop = 42       # '*' or '/' applying operand -> term
    let mut i = start
    while i < end {
        let c = src[i]
        if is_space(c) {
            i = i + 1
        } else if is_digit(c) {
            # read a number
            let mut num = 0
            let mut go = true
            while go {
                if i >= end {
                    go = false
                } else if is_digit(src[i]) {
                    num = num * 10 + (src[i] - 48)
                    i = i + 1
                } else {
                    go = false
                }
            }
            if term_started {
                if termop == 42 { term = term * num } else { term = term / num }
            } else {
                term = num
                term_started = true
            }
        } else if is_lower(c) {
            # variable reference (single letter)
            let v = vars[c - 97]
            i = i + 1
            if term_started {
                if termop == 42 { term = term * v } else { term = term / v }
            } else {
                term = v
                term_started = true
            }
        } else {
            # operator
            if c == 43 {
                if pending == 43 { total = total + term } else { total = total - term }
                pending = 43
                term_started = false
            } else if c == 45 {
                if pending == 43 { total = total + term } else { total = total - term }
                pending = 45
                term_started = false
            } else {
                termop = c
            }
            i = i + 1
        }
    }
    if term_started {
        if pending == 43 { total = total + term } else { total = total - term }
    }
    return total
}

# Run a program: a ';'-separated list of `let x = expr` and a final `return expr`.
fn run_program(src: Str) -> Int {
    let n = src.len()
    # symbol table: 26 slots
    let mut vars: List<Int> = []
    let mut k = 0
    while k < 26 {
        vars.push(0)
        k = k + 1
    }
    let mut result = 0
    let mut i = 0
    while i < n {
        # find end of this statement (';' or end)
        let stmt_start = i
        let mut j = i
        let mut go = true
        while go {
            if j >= n {
                go = false
            } else if src[j] == 59 {
                go = false
            } else {
                j = j + 1
            }
        }
        # stmt is src[stmt_start..j). Determine kind by first non-space token.
        # find 'l' (let) or 'r' (return)
        let mut p = stmt_start
        let mut sgo = true
        while sgo {
            if p >= j {
                sgo = false
            } else if is_space(src[p]) {
                p = p + 1
            } else {
                sgo = false
            }
        }
        if p < j {
            let first = src[p]
            if first == 108 {
                # "let <var> = <expr>": find var letter and '='
                # skip "let"
                let mut q = p + 3
                # skip spaces
                let mut g2 = true
                while g2 {
                    if q >= j { g2 = false }
                    else if is_space(src[q]) { q = q + 1 }
                    else { g2 = false }
                }
                let var_letter = src[q]   # single letter
                # find '=' after var
                let mut e = q
                let mut g3 = true
                while g3 {
                    if e >= j { g3 = false }
                    else if src[e] == 61 { g3 = false }
                    else { e = e + 1 }
                }
                let val = eval_arith(src, e + 1, j, vars)
                vars[var_letter - 97] = val
            } else if first == 114 {
                # "return <expr>": eval after "return"
                result = eval_arith(src, p + 6, j, vars)
            }
        }
        i = j + 1
    }
    return result
}

fn emit_ir(value: Int) -> Int {
    print("define i64 @main() {")
    print("  ret i64 {value}")
    print("}")
    return 0
}

fn main() -> Int {
    # Program: a=2, b=3, return a + b * 4  ->  2 + 3*4 = 14
    let value = run_program("let a = 2; let b = 3; return a + b * 4")
    return emit_ir(value)
}
