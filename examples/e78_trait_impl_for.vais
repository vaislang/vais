# expect: 42
# Rust-style trait impl syntax: impl Trait for Type.
trait Area {
    fn area(self) -> Int
}

struct Square {
    side: Int
}

impl Area for Square {
    fn area(self) -> Int {
        return self.side * self.side
    }
}

fn main() -> Int {
    let s = Square { side: 6 }
    return s.area() + 6
}
