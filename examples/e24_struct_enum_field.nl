# expect: 1
# A struct whose field is an enum; match on the field after construction.
# (Combines struct literals with enum matching — a common modeling shape.)
enum Color { Red, Green, Blue }
struct Pixel { c: Color, v: Int }

fn main() -> Int {
    let p = Pixel { c: Color.Green, v: 5 }
    match p.c {
        Color.Red => return 0,
        Color.Green => return 1,
        Color.Blue => return 2,
    }
}
