# expect: 13
# Fibonacci via tree recursion (two recursive calls per step). fib(7) = 13.
# (e03 is linear factorial; this is the branching tree-recursion shape.)
fn fib(n: Int) -> Int {
    if n < 2 { return n }
    return fib(n - 1) + fib(n - 2)
}

fn main() -> Int {
    return fib(7)
}
