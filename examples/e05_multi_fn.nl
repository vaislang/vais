# expect: 24
fn add(a: Int, b: Int) -> Int { return a + b }
fn mul(a: Int, b: Int) -> Int { return a * b }
fn main() -> Int {
    return add(mul(2, 3), mul(3, 6))
}
