# expect: 12
# A function returns a struct; the caller reads its fields.
# (AI often mishandles returning a struct by value then accessing fields.)
struct Point { x: Int, y: Int }

fn make(a: Int) -> Point {
    return Point { x: a, y: a + a }
}

fn main() -> Int {
    let p = make(4)
    return p.x + p.y
}
