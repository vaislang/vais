# expect: 8
# Collatz step counter: two-argument recursion with modulo.
# collatz(6, 0) follows 6 -> 3 -> 10 -> 5 -> 16 -> 8 -> 4 -> 2 -> 1 = 8 steps.
fn collatz(n: Int, steps: Int) -> Int {
    if n == 1 { return steps }
    if n % 2 == 0 { return collatz(n / 2, steps + 1) }
    return collatz(3 * n + 1, steps + 1)
}

fn main() -> Int {
    return collatz(6, 0)
}
