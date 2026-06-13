# expect: 2
# A tiny calculator: an Op enum dispatched by match, mixing arithmetic and a
# bitwise op (bitand). calc(And, 6, 3) = 6 & 3 = 2.
# (Written by a cold-start AI from the corpus -- combines enum dispatch (e22) +
# bitwise word-functions (e31) first-try; value-verified.)
enum Op { Add, Sub, And }

fn calc(op: Op, a: Int, b: Int) -> Int {
    match op {
        Op.Add => return a + b,
        Op.Sub => return a - b,
        Op.And => return bitand(a, b),
    }
}

fn main() -> Int {
    return calc(Op.And, 6, 3)
}
