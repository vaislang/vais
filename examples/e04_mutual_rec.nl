# expect: 1
fn is_even(n: Int) -> Int {
    if n == 0 { return 1 }
    return is_odd(n - 1)
}
fn is_odd(n: Int) -> Int {
    if n == 0 { return 0 }
    return is_even(n - 1)
}
fn main() -> Int {
    return is_even(10)
}
