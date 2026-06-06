enum Color { Red, Green, Blue }
fn pick(c: Color) -> Int {
    match c {
        Color.Red => 1,
        Color.Green => 2,
        Color.Blue => 3,
    }
}
fn main() -> Int {
    let c = Color.Green
    return pick(c)
}
