# expect: 50
# Multi-field struct with a computed function over its fields (w * h).
struct Rect { w: Int, h: Int }

fn area(r: Rect) -> Int {
    return r.w * r.h
}

fn main() -> Int {
    let r = Rect { w: 10, h: 5 }
    return area(r)
}
