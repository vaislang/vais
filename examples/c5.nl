fn add_bonus(n: Int) -> Int {
    let bonus = 3
    let f = |x| x + bonus
    return f(n)
}
fn main() -> Int {
    return add_bonus(4)
}
