# expect: 12
# List.map passes through to Vais Vec.map; sum is lowered to fold.
fn main() -> Int {
    let xs: List<Int> = []
    xs.push(1)
    xs.push(2)
    xs.push(3)
    let doubled = xs.map(|x| x * 2)
    return doubled.sum()
}
