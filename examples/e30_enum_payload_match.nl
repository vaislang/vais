# expect: 42
# Enum variants carrying a payload, matched with the payload bound per arm.
# (e02 builds payload enums; this matches them and uses the bound value.)
enum Shape { Circle(Int), Square(Int) }

fn area_ish(s: Shape) -> Int {
    match s {
        Circle(r) => return r * 3,
        Square(side) => return side * side,
    }
}

fn main() -> Int {
    return area_ish(Circle(14))
}
