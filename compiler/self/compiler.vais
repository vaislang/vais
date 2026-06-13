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

fn is_space2(c: Int) -> Bool {
    if c == 32 { return true }
    if c == 9 { return true }
    if c == 10 { return true }
    return false
}

fn skip_spaces(src: Str, p: Int, end: Int) -> Int {
    let mut q = p
    let mut go = true
    while go {
        if q >= end { go = false }
        else if is_space2(src[q]) { q = q + 1 }
        else { go = false }
    }
    return q
}

fn starts_if(src: Str, p: Int, end: Int) -> Bool {
    if p + 2 > end { return false }
    if src[p] == 105 {
        if src[p + 1] == 102 { return true }
    }
    return false
}

# Find 4-letter keyword (w0..w3) start in src[p..end); returns index or end.
fn find_kw4(src: Str, p: Int, end: Int, w0: Int, w1: Int, w2: Int, w3: Int) -> Int {
    let mut q = p
    let mut go = true
    while go {
        if q + 4 > end { q = end; go = false }
        else if src[q] == w0 {
            if src[q + 1] == w1 {
                if src[q + 2] == w2 {
                    if src[q + 3] == w3 { go = false } else { q = q + 1 }
                } else { q = q + 1 }
            } else { q = q + 1 }
        } else { q = q + 1 }
    }
    return q
}

# Evaluate an expression that may be a conditional (CX4):
#   if <arith> <cmp> <arith> then <arith> else <arith>   (cmp: > < ==)
# otherwise a plain <arith>. To avoid the Vais Vec-move limit (task_54658a43,
# two straight-line calls passing `vars` move it), all sub-expression evals go
# through a single LOOP over their [start,end) ranges (loop-passing `vars` is OK).
fn eval_value(src: Str, start: Int, end: Int, vars: List<Int>) -> Int {
    let p = skip_spaces(src, start, end)
    let is_if = starts_if(src, p, end)
    # Build the list of [start,end) sub-expression ranges to evaluate.
    # Non-conditional: a single range (the whole expr).
    # Conditional `if L<cmp>R then T else E`: four ranges (L, R, T, E).
    # CRITICAL: `vars` is consumed in EXACTLY ONE place — the single loop below —
    # to avoid the Vais flow-insensitive Vec-move error (task_54658a43).
    let mut starts: List<Int> = []
    let mut ends: List<Int> = []
    let mut op = 0
    if is_if {
        let then_pos = find_kw4(src, p + 2, end, 116, 104, 101, 110)        # "then"
        let else_pos = find_kw4(src, then_pos + 4, end, 101, 108, 115, 101)  # "else"
        let cstart = p + 2
        let cend = then_pos
        let mut oppos = cend
        let mut qq = cstart
        let mut g = true
        while g {
            if qq >= cend { g = false }
            else if src[qq] == 62 { oppos = qq; op = 62; g = false }
            else if src[qq] == 60 { oppos = qq; op = 60; g = false }
            else if src[qq] == 61 { oppos = qq; op = 61; g = false }
            else { qq = qq + 1 }
        }
        let mut rstart = oppos + 1
        if op == 61 {
            if rstart < cend {
                if src[rstart] == 61 { rstart = rstart + 1 }
            }
        }
        starts.push(cstart)
        ends.push(oppos)
        starts.push(rstart)
        ends.push(cend)
        starts.push(then_pos + 4)
        ends.push(else_pos)
        starts.push(else_pos + 4)
        ends.push(end)
    } else {
        starts.push(start)
        ends.push(end)
    }
    let count = starts.len()
    let mut vals: List<Int> = []
    let mut k = 0
    while k < count {
        vals.push(eval_arith(src, starts[k], ends[k], vars))
        k = k + 1
    }
    if is_if {
        let lhs = vals[0]
        let rhs = vals[1]
        let mut cond = 0
        if op == 62 {
            if lhs > rhs { cond = 1 }
        } else if op == 60 {
            if lhs < rhs { cond = 1 }
        } else {
            if lhs == rhs { cond = 1 }
        }
        if cond == 1 { return vals[2] }
        return vals[3]
    }
    return vals[0]
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
                let val = eval_value(src, e + 1, j, vars)
                vars[var_letter - 97] = val
            } else if first == 114 {
                # "return <expr>": eval after "return" (may be an if-expr)
                result = eval_value(src, p + 6, j, vars)
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
    # CX4: conditional with variables in the condition AND the branches.
    # a=7, b=4 -> a > b is true -> return a + b = 11
    let value = run_program("let a = 7; let b = 4; return if a > b then a + b else a - b")
    return emit_ir(value)
}
