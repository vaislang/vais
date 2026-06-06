# expect: 12
# A closure passed as a function argument (first-class functions).
# apply_fn(|n| n * 2, 6) calls the closure on 6 -> 12.
fn apply_fn(f: fn(Int) -> Int, x: Int) -> Int {
    return f(x)
}

fn main() -> Int {
    return apply_fn(|n| n * 2, 6)
}
