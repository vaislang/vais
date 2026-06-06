# expect: 7
# Tuple return + destructuring: a function returns (Int, Int); the caller binds
# both with `let (a, b) = ...`.
fn pair() -> (Int, Int) {
    return (3, 4)
}

fn main() -> Int {
    let (a, b) = pair()
    return a + b
}
