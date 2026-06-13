# expect: 12
# Enum used as an operation selector, dispatched with match in a helper.
# (A tiny interpreter pattern: the value of `o` chooses the computation.)
enum Op { Add, Mul }

fn apply(o: Op, a: Int, b: Int) -> Int {
    match o {
        Op.Add => return a + b,
        Op.Mul => return a * b,
    }
}

fn main() -> Int {
    return apply(Op.Add, 8, 4)
}
